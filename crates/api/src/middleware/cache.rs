use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{header, Method, StatusCode},
    Error, HttpMessage, HttpResponse,
};
use bytes::Bytes;
use futures_util::future::LocalBoxFuture;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::future::{ready, Ready};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::middleware::auth::UserContext;

/// Configuration for the cache middleware
#[derive(Clone, Debug)]
pub struct CacheConfig {
    /// Default TTL for cached responses (in seconds)
    pub default_ttl: u64,
    /// TTL for content endpoints (in seconds)
    pub content_ttl: u64,
    /// Whether to cache authenticated requests
    pub cache_authenticated: bool,
    /// Skip caching for these path patterns
    pub skip_paths: Vec<String>,
    /// Skip caching for these query parameters (indicates dynamic content)
    pub skip_query_params: Vec<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: 60,            // 1 minute default
            content_ttl: 300,           // 5 minutes for content
            cache_authenticated: false, // Don't cache user-specific data by default
            skip_paths: vec![
                "/api/user/".to_string(),
                "/api/sync/".to_string(),
                "/api/admin/".to_string(),
            ],
            skip_query_params: vec!["nocache".to_string(), "timestamp".to_string()],
        }
    }
}

/// Cache middleware for Actix-web
#[derive(Clone)]
pub struct CacheMiddleware {
    redis: ConnectionManager,
    config: Arc<CacheConfig>,
}

impl CacheMiddleware {
    /// Create a new cache middleware instance
    pub async fn new(redis_url: &str, config: CacheConfig) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let redis = ConnectionManager::new(client).await?;

        info!("Cache middleware initialized with Redis at {}", redis_url);

        Ok(Self {
            redis,
            config: Arc::new(config),
        })
    }

    /// Create with default configuration
    pub async fn default_config(redis_url: &str) -> anyhow::Result<Self> {
        Self::new(redis_url, CacheConfig::default()).await
    }

    /// Invalidate cache entries matching a pattern
    pub async fn invalidate_pattern(&self, pattern: &str) -> anyhow::Result<u64> {
        let mut conn = self.redis.clone();
        let cache_key = format!("cache:{}", pattern);

        // Get all keys matching pattern
        let keys: Vec<String> = conn.keys(&cache_key).await?;

        if keys.is_empty() {
            return Ok(0);
        }

        // Delete all matching keys
        let count: u64 = conn.del(&keys).await?;

        info!(
            "Invalidated {} cache entries matching pattern: {}",
            count, pattern
        );

        Ok(count)
    }

    /// Invalidate a specific cache entry
    pub async fn invalidate(&self, key: &str) -> anyhow::Result<bool> {
        let mut conn = self.redis.clone();
        let cache_key = format!("cache:{}", key);

        let deleted: u64 = conn.del(&cache_key).await?;

        Ok(deleted > 0)
    }
}

impl<S, B> Transform<S, ServiceRequest> for CacheMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = CacheMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CacheMiddlewareService {
            service: Arc::new(service),
            redis: self.redis.clone(),
            config: Arc::clone(&self.config),
        }))
    }
}

pub struct CacheMiddlewareService<S> {
    service: Arc<S>,
    redis: ConnectionManager,
    config: Arc<CacheConfig>,
}

impl<S, B> Service<ServiceRequest> for CacheMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Only cache GET requests
        if req.method() != Method::GET {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_boxed_body())
            });
        }

        // Check if this path should be skipped (clone to avoid borrowing)
        let path = req.path().to_string();
        for skip_path in &self.config.skip_paths {
            if path.starts_with(skip_path) {
                debug!("Skipping cache for path: {}", path);
                let fut = self.service.call(req);
                return Box::pin(async move {
                    let res = fut.await?;
                    Ok(res.map_into_boxed_body())
                });
            }
        }

        // Get user context to determine if request is authenticated
        let user_context = req.extensions().get::<UserContext>().cloned();
        let is_authenticated = user_context
            .as_ref()
            .map(|ctx| ctx.is_authenticated)
            .unwrap_or(false);

        // Skip caching authenticated requests unless configured to cache them
        if is_authenticated && !self.config.cache_authenticated {
            debug!("Skipping cache for authenticated request: {}", path);
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_boxed_body())
            });
        }

        // Generate cache key
        let cache_key = match generate_cache_key(&req, &user_context, &self.config) {
            Ok(key) => key,
            Err(e) => {
                warn!("Failed to generate cache key: {}", e);
                let fut = self.service.call(req);
                return Box::pin(async move {
                    let res = fut.await?;
                    Ok(res.map_into_boxed_body())
                });
            }
        };

        // Check for If-None-Match header
        let if_none_match = req
            .headers()
            .get(header::IF_NONE_MATCH)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let service = Arc::clone(&self.service);
        let mut redis = self.redis.clone();
        let config = Arc::clone(&self.config);

        Box::pin(async move {
            // Try to get cached response
            match redis.get::<_, Option<String>>(&cache_key).await {
                Ok(Some(cached_data)) => {
                    // Parse cached response
                    match serde_json::from_str::<CachedResponse>(&cached_data) {
                        Ok(cached) => {
                            // Check ETag match
                            if let Some(ref client_etag) = if_none_match {
                                if client_etag == &cached.etag {
                                    debug!("Cache hit with ETag match (304): {}", cache_key);

                                    let response = HttpResponse::NotModified()
                                        .insert_header((header::ETAG, cached.etag.clone()))
                                        .insert_header((
                                            header::CACHE_CONTROL,
                                            format!("max-age={}", config.default_ttl),
                                        ))
                                        .finish();

                                    return Ok(ServiceResponse::new(req.into_parts().0, response));
                                }
                            }

                            debug!("Cache hit (200): {}", cache_key);

                            // Return cached response
                            let mut response = HttpResponse::build(
                                StatusCode::from_u16(cached.status_code).unwrap_or(StatusCode::OK),
                            );

                            // Add cached headers
                            for (name, value) in &cached.headers {
                                if let Ok(header_name) =
                                    header::HeaderName::from_bytes(name.as_bytes())
                                {
                                    if let Ok(header_value) = header::HeaderValue::from_str(value) {
                                        response.insert_header((header_name, header_value));
                                    }
                                }
                            }

                            // Add cache headers
                            response.insert_header((header::ETAG, cached.etag.clone()));
                            response.insert_header((
                                header::CACHE_CONTROL,
                                format!("max-age={}", config.default_ttl),
                            ));
                            response.insert_header(("X-Cache", "HIT"));

                            let response = response.body(cached.body);

                            return Ok(ServiceResponse::new(req.into_parts().0, response));
                        }
                        Err(e) => {
                            warn!("Failed to deserialize cached response: {}", e);
                            // Continue to fetch fresh response
                        }
                    }
                }
                Ok(None) => {
                    debug!("Cache miss: {}", cache_key);
                }
                Err(e) => {
                    error!("Redis error reading cache: {}", e);
                    // Continue to fetch fresh response
                }
            }

            // Cache miss or error - fetch from service
            let res = service.call(req).await?;

            // Only cache successful responses (2xx)
            let status = res.status();
            if !status.is_success() {
                return Ok(res.map_into_boxed_body());
            }

            // Extract response parts
            let (req, res) = res.into_parts();
            let (res_parts, body) = res.into_parts();

            // Collect body bytes
            let body_bytes = match body.try_into_bytes() {
                Ok(bytes) => bytes,
                Err(body) => {
                    // If we can't collect the body, return the original response
                    let res = HttpResponse::from(res_parts).set_body(body);
                    return Ok(ServiceResponse::new(req, res).map_into_boxed_body());
                }
            };

            // Generate ETag from body hash
            let etag = generate_etag(&body_bytes);

            // Determine TTL based on path
            let ttl = if path.contains("/content/") || path.contains("/media/") {
                config.content_ttl
            } else {
                config.default_ttl
            };

            // Create cached response
            let cached = CachedResponse {
                status_code: res_parts.status().as_u16(),
                headers: res_parts
                    .headers()
                    .iter()
                    .filter_map(|(name, value)| {
                        value
                            .to_str()
                            .ok()
                            .map(|v| (name.to_string(), v.to_string()))
                    })
                    .collect(),
                body: body_bytes.to_vec(),
                etag: etag.clone(),
            };

            // Store in cache (fire and forget)
            let cache_data = match serde_json::to_string(&cached) {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to serialize response for caching: {}", e);
                    // Still return the response
                    let mut response = HttpResponse::from(res_parts);
                    response
                        .headers_mut()
                        .insert(header::ETAG, etag.parse().unwrap());
                    response
                        .headers_mut()
                        .insert("X-Cache".parse().unwrap(), "MISS".parse().unwrap());
                    let res = response.set_body(body_bytes);
                    return Ok(ServiceResponse::new(req, res).map_into_boxed_body());
                }
            };

            let cache_key_clone = cache_key.clone();
            let mut redis_clone = redis.clone();
            tokio::spawn(async move {
                if let Err(e) = redis_clone
                    .set_ex::<_, _, ()>(&cache_key_clone, cache_data, ttl)
                    .await
                {
                    error!("Failed to store response in cache: {}", e);
                }
            });

            // Build response with cache headers
            let mut response = HttpResponse::from(res_parts);
            response
                .headers_mut()
                .insert(header::ETAG, etag.parse().unwrap());
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                format!("max-age={}", ttl).parse().unwrap(),
            );
            response
                .headers_mut()
                .insert("X-Cache".parse().unwrap(), "MISS".parse().unwrap());

            let res = response.set_body(body_bytes);

            Ok(ServiceResponse::new(req, res).map_into_boxed_body())
        })
    }
}

/// Cached response data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedResponse {
    status_code: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    etag: String,
}

/// Generate a cache key from the request
fn generate_cache_key(
    req: &ServiceRequest,
    user_context: &Option<UserContext>,
    config: &CacheConfig,
) -> anyhow::Result<String> {
    let method = req.method().as_str();
    let path = req.path();

    // Parse query string and filter out skip parameters
    let query = req.query_string();
    let filtered_query = if !query.is_empty() {
        let params: Vec<(String, String)> = serde_urlencoded::from_str(query)?;
        let filtered: Vec<(String, String)> = params
            .into_iter()
            .filter(|(key, _)| !config.skip_query_params.contains(key))
            .collect();

        if filtered.is_empty() {
            String::new()
        } else {
            format!("?{}", serde_urlencoded::to_string(&filtered)?)
        }
    } else {
        String::new()
    };

    // Include user_id for authenticated requests if configured
    let user_suffix = if config.cache_authenticated {
        user_context
            .as_ref()
            .filter(|ctx| ctx.is_authenticated)
            .map(|ctx| format!(":user:{}", ctx.user_id))
            .unwrap_or_default()
    } else {
        String::new()
    };

    let key = format!("cache:{}:{}{}{}", method, path, filtered_query, user_suffix);

    Ok(key)
}

/// Generate an ETag from response body
fn generate_etag(body: &Bytes) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body);
    let hash = hasher.finalize();
    format!("\"{}\"", hex::encode(&hash[..16])) // Use first 16 bytes for shorter ETag
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_cache_key_generation() {
        let config = CacheConfig::default();

        // Create a test request
        let req = test::TestRequest::get()
            .uri("/api/content/123?param1=value1&param2=value2")
            .to_srv_request();

        let key = generate_cache_key(&req, &None, &config).unwrap();

        assert!(key.starts_with("cache:GET:/api/content/123"));
        assert!(key.contains("param1=value1"));
        assert!(key.contains("param2=value2"));
    }

    #[actix_web::test]
    async fn test_cache_key_filters_skip_params() {
        let mut config = CacheConfig::default();
        config.skip_query_params.push("timestamp".to_string());

        let req = test::TestRequest::get()
            .uri("/api/content/123?id=1&timestamp=12345")
            .to_srv_request();

        let key = generate_cache_key(&req, &None, &config).unwrap();

        assert!(key.contains("id=1"));
        assert!(!key.contains("timestamp"));
    }

    #[actix_web::test]
    async fn test_etag_generation() {
        let body1 = Bytes::from("test content");
        let body2 = Bytes::from("test content");
        let body3 = Bytes::from("different content");

        let etag1 = generate_etag(&body1);
        let etag2 = generate_etag(&body2);
        let etag3 = generate_etag(&body3);

        // Same content should generate same ETag
        assert_eq!(etag1, etag2);

        // Different content should generate different ETag
        assert_ne!(etag1, etag3);

        // ETag should be properly quoted
        assert!(etag1.starts_with('"'));
        assert!(etag1.ends_with('"'));
    }

    #[actix_web::test]
    async fn test_authenticated_cache_key() {
        let mut config = CacheConfig::default();
        config.cache_authenticated = true;

        let user_context = Some(UserContext {
            user_id: "user123".to_string(),
            tier: "pro".to_string(),
            is_authenticated: true,
        });

        let req = test::TestRequest::get()
            .uri("/api/content/123")
            .to_srv_request();

        let key = generate_cache_key(&req, &user_context, &config).unwrap();

        assert!(key.contains("user:user123"));
    }

    #[test]
    fn test_cache_config_defaults() {
        let config = CacheConfig::default();

        assert_eq!(config.default_ttl, 60);
        assert_eq!(config.content_ttl, 300);
        assert_eq!(config.cache_authenticated, false);
        assert!(!config.skip_paths.is_empty());
    }
}
