use crate::error::{AuthError, Result};
use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::{
    future::{ready, Ready},
    rc::Rc,
    task::{Context, Poll},
    time::{SystemTime, UNIX_EPOCH},
};

/// Rate limit configuration for different endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Rate limit for /auth/token endpoint (requests per minute)
    pub token_endpoint_limit: u32,
    /// Rate limit for /auth/device endpoint (requests per minute)
    pub device_endpoint_limit: u32,
    /// Rate limit for /auth/authorize endpoint (requests per minute)
    pub authorize_endpoint_limit: u32,
    /// Rate limit for /auth/revoke endpoint (requests per minute)
    pub revoke_endpoint_limit: u32,
    /// Rate limit for /auth/register endpoint (5 requests per hour per IP)
    pub register_endpoint_limit: u32,
    /// Rate limit for /auth/login endpoint (requests per minute)
    pub login_endpoint_limit: u32,
    /// Secret for internal service bypass (from X-Internal-Service header)
    pub internal_service_secret: Option<String>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            token_endpoint_limit: 10,
            device_endpoint_limit: 5,
            authorize_endpoint_limit: 20,
            revoke_endpoint_limit: 10,
            register_endpoint_limit: 5, // 5 registrations per hour per IP
            login_endpoint_limit: 10,   // 10 login attempts per minute per IP
            internal_service_secret: None,
        }
    }
}

impl RateLimitConfig {
    pub fn new(
        token_limit: u32,
        device_limit: u32,
        authorize_limit: u32,
        revoke_limit: u32,
        register_limit: u32,
        login_limit: u32,
    ) -> Self {
        Self {
            token_endpoint_limit: token_limit,
            device_endpoint_limit: device_limit,
            authorize_endpoint_limit: authorize_limit,
            revoke_endpoint_limit: revoke_limit,
            register_endpoint_limit: register_limit,
            login_endpoint_limit: login_limit,
            internal_service_secret: None,
        }
    }

    pub fn with_internal_secret(mut self, secret: String) -> Self {
        self.internal_service_secret = Some(secret);
        self
    }

    /// Get rate limit for a specific endpoint path
    /// Returns (limit, window_in_seconds)
    pub fn get_limit_for_path(&self, path: &str) -> Option<(u32, u64)> {
        if path.contains("/auth/register") {
            // 5 requests per hour (3600 seconds)
            Some((self.register_endpoint_limit, 3600))
        } else if path.contains("/auth/login") {
            // 10 requests per minute (60 seconds)
            Some((self.login_endpoint_limit, 60))
        } else if path.contains("/auth/token") {
            Some((self.token_endpoint_limit, 60))
        } else if path.contains("/auth/device") {
            Some((self.device_endpoint_limit, 60))
        } else if path.contains("/auth/authorize") {
            Some((self.authorize_endpoint_limit, 60))
        } else if path.contains("/auth/revoke") {
            Some((self.revoke_endpoint_limit, 60))
        } else {
            None
        }
    }
}

/// Rate limiting middleware using sliding window algorithm
pub struct RateLimitMiddleware {
    redis_client: redis::Client,
    config: RateLimitConfig,
}

impl RateLimitMiddleware {
    pub fn new(redis_client: redis::Client, config: RateLimitConfig) -> Self {
        Self {
            redis_client,
            config,
        }
    }

    /// Extract client ID from request
    fn extract_client_id(req: &ServiceRequest) -> String {
        // Try X-Client-ID header first
        if let Some(client_id) = req
            .headers()
            .get("X-Client-ID")
            .and_then(|h| h.to_str().ok())
        {
            return client_id.to_string();
        }

        // Fallback to IP address
        req.peer_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Check if request has valid internal service bypass
    fn check_internal_bypass(req: &ServiceRequest, config: &RateLimitConfig) -> bool {
        if let Some(secret) = &config.internal_service_secret {
            if let Some(header_secret) = req
                .headers()
                .get("X-Internal-Service")
                .and_then(|h| h.to_str().ok())
            {
                return header_secret == secret;
            }
        }
        false
    }

    /// Get current window start timestamp
    fn get_window_start(window_seconds: u64) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Round down to nearest window
        (now / window_seconds) * window_seconds
    }

    /// Check rate limit using sliding window algorithm
    async fn check_rate_limit(
        redis_client: &redis::Client,
        endpoint: &str,
        client_id: &str,
        limit: u32,
        window_seconds: u64,
    ) -> Result<(bool, u32, u64)> {
        let mut conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AuthError::Internal(format!("Redis connection error: {}", e)))?;

        let window_start = Self::get_window_start(window_seconds);
        let key = format!("rate_limit:{}:{}:{}", endpoint, client_id, window_start);

        // Increment counter
        let count: u32 = conn
            .incr(&key, 1)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis INCR error: {}", e)))?;

        // Set expiration on first increment (2x window to cover current + previous window)
        if count == 1 {
            let ttl = (window_seconds * 2) as i64;
            let _: () = conn
                .expire(&key, ttl)
                .await
                .map_err(|e| AuthError::Internal(format!("Redis EXPIRE error: {}", e)))?;
        }

        let allowed = count <= limit;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let retry_after = if allowed {
            0
        } else {
            window_seconds - (now % window_seconds)
        };

        Ok((allowed, count, retry_after))
    }

    /// Create 429 Too Many Requests response
    fn create_rate_limit_response(
        retry_after: u64,
        current_count: u32,
        limit: u32,
    ) -> HttpResponse<BoxBody> {
        HttpResponse::build(StatusCode::TOO_MANY_REQUESTS)
            .insert_header(("Retry-After", retry_after.to_string()))
            .insert_header(("X-RateLimit-Limit", limit.to_string()))
            .insert_header(("X-RateLimit-Remaining", "0"))
            .insert_header(("X-RateLimit-Reset", retry_after.to_string()))
            .json(serde_json::json!({
                "error": "rate_limit_exceeded",
                "message": format!("Rate limit exceeded. Maximum {} requests per minute allowed.", limit),
                "retry_after": retry_after,
                "current_count": current_count,
                "limit": limit
            }))
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<std::result::Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service: Rc::new(service),
            redis_client: self.redis_client.clone(),
            config: self.config.clone(),
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: Rc<S>,
    redis_client: redis::Client,
    config: RateLimitConfig,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, std::result::Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let redis_client = self.redis_client.clone();
        let config = self.config.clone();

        Box::pin(async move {
            let path = req.path();

            // Check for internal service bypass
            if RateLimitMiddleware::check_internal_bypass(&req, &config) {
                tracing::debug!("Rate limit bypassed for internal service");
                return service.call(req).await.map(|res| res.map_into_left_body());
            }

            // Get rate limit for this endpoint
            let (limit, window_seconds) = match config.get_limit_for_path(path) {
                Some((l, w)) => (l, w),
                None => {
                    // No rate limit configured for this endpoint
                    return service.call(req).await.map(|res| res.map_into_left_body());
                }
            };

            // Extract client identifier
            let client_id = RateLimitMiddleware::extract_client_id(&req);
            let endpoint = path.to_string();

            // Check rate limit
            let (allowed, current_count, retry_after) = RateLimitMiddleware::check_rate_limit(
                &redis_client,
                &endpoint,
                &client_id,
                limit,
                window_seconds,
            )
            .await
            .map_err(|e| Error::from(e))?;

            if !allowed {
                tracing::warn!(
                    "Rate limit exceeded for client {} on endpoint {}: {}/{}",
                    client_id,
                    endpoint,
                    current_count,
                    limit
                );
                let response = RateLimitMiddleware::create_rate_limit_response(
                    retry_after,
                    current_count,
                    limit,
                );
                let (http_req, _) = req.into_parts();
                return Ok(ServiceResponse::new(http_req, response).map_into_right_body());
            }

            // Add rate limit headers to response
            let mut res = service.call(req).await?;
            let headers = res.headers_mut();
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-ratelimit-limit"),
                actix_web::http::header::HeaderValue::from_str(&limit.to_string()).unwrap(),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-ratelimit-remaining"),
                actix_web::http::header::HeaderValue::from_str(
                    &(limit - current_count).to_string(),
                )
                .unwrap(),
            );

            Ok(res.map_into_left_body())
        })
    }
}

/// Helper function to wire rate limiting into Actix-web App
pub fn configure_rate_limiting(
    redis_client: redis::Client,
    config: RateLimitConfig,
) -> RateLimitMiddleware {
    RateLimitMiddleware::new(redis_client, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    async fn test_handler() -> Result<HttpResponse, Error> {
        Ok(HttpResponse::Ok().json(serde_json::json!({"status": "ok"})))
    }

    fn setup_redis_client() -> redis::Client {
        redis::Client::open("redis://127.0.0.1:6379").expect("Failed to create Redis client")
    }

    #[actix_web::test]
    async fn test_rate_limit_config_defaults() {
        let config = RateLimitConfig::default();
        assert_eq!(config.token_endpoint_limit, 10);
        assert_eq!(config.device_endpoint_limit, 5);
        assert_eq!(config.authorize_endpoint_limit, 20);
        assert_eq!(config.revoke_endpoint_limit, 10);
        assert!(config.internal_service_secret.is_none());
    }

    #[actix_web::test]
    async fn test_rate_limit_config_with_secret() {
        let config = RateLimitConfig::default().with_internal_secret("test-secret-123".to_string());
        assert_eq!(
            config.internal_service_secret,
            Some("test-secret-123".to_string())
        );
    }

    #[actix_web::test]
    async fn test_get_limit_for_path() {
        let config = RateLimitConfig::default();
        assert_eq!(config.get_limit_for_path("/auth/register"), Some((5, 3600)));
        assert_eq!(config.get_limit_for_path("/auth/login"), Some((10, 60)));
        assert_eq!(config.get_limit_for_path("/auth/token"), Some((10, 60)));
        assert_eq!(config.get_limit_for_path("/auth/device"), Some((5, 60)));
        assert_eq!(config.get_limit_for_path("/auth/authorize"), Some((20, 60)));
        assert_eq!(config.get_limit_for_path("/auth/revoke"), Some((10, 60)));
        assert_eq!(config.get_limit_for_path("/health"), None);
    }

    #[actix_web::test]
    async fn test_window_start_calculation() {
        let window_start_60 = RateLimitMiddleware::get_window_start(60);
        assert_eq!(window_start_60 % 60, 0);

        let window_start_3600 = RateLimitMiddleware::get_window_start(3600);
        assert_eq!(window_start_3600 % 3600, 0);
    }

    #[actix_web::test]
    async fn test_rate_limit_enforcement() {
        let redis_client = setup_redis_client();

        // Check Redis connectivity
        if redis_client
            .get_multiplexed_async_connection()
            .await
            .is_err()
        {
            println!("Redis not available, skipping integration test");
            return;
        }

        let config = RateLimitConfig {
            token_endpoint_limit: 3,
            device_endpoint_limit: 5,
            authorize_endpoint_limit: 20,
            revoke_endpoint_limit: 10,
            register_endpoint_limit: 5,
            login_endpoint_limit: 10,
            internal_service_secret: None,
        };

        let app = test::init_service(
            App::new()
                .wrap(RateLimitMiddleware::new(
                    redis_client.clone(),
                    config.clone(),
                ))
                .route("/auth/token", web::post().to(test_handler)),
        )
        .await;

        // First 3 requests should succeed
        for i in 1..=3 {
            let req = test::TestRequest::post()
                .uri("/auth/token")
                .insert_header(("X-Client-ID", "test-client-123"))
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(
                resp.status(),
                StatusCode::OK,
                "Request {} should succeed",
                i
            );
        }

        // 4th request should be rate limited
        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Client-ID", "test-client-123"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

        // Check rate limit headers
        assert!(resp.headers().get("Retry-After").is_some());
        assert!(resp.headers().get("X-RateLimit-Limit").is_some());
    }

    #[actix_web::test]
    async fn test_internal_service_bypass() {
        let redis_client = setup_redis_client();

        if redis_client
            .get_multiplexed_async_connection()
            .await
            .is_err()
        {
            println!("Redis not available, skipping integration test");
            return;
        }

        let config = RateLimitConfig {
            token_endpoint_limit: 2,
            device_endpoint_limit: 5,
            authorize_endpoint_limit: 20,
            revoke_endpoint_limit: 10,
            register_endpoint_limit: 5,
            login_endpoint_limit: 10,
            internal_service_secret: Some("super-secret-key".to_string()),
        };

        let app = test::init_service(
            App::new()
                .wrap(RateLimitMiddleware::new(
                    redis_client.clone(),
                    config.clone(),
                ))
                .route("/auth/token", web::post().to(test_handler)),
        )
        .await;

        // Make requests beyond the limit with bypass header
        for i in 1..=5 {
            let req = test::TestRequest::post()
                .uri("/auth/token")
                .insert_header(("X-Client-ID", "test-client-bypass"))
                .insert_header(("X-Internal-Service", "super-secret-key"))
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(
                resp.status(),
                StatusCode::OK,
                "Request {} with bypass should succeed",
                i
            );
        }
    }

    #[actix_web::test]
    async fn test_different_clients_separate_limits() {
        let redis_client = setup_redis_client();

        if redis_client
            .get_multiplexed_async_connection()
            .await
            .is_err()
        {
            println!("Redis not available, skipping integration test");
            return;
        }

        let config = RateLimitConfig {
            token_endpoint_limit: 3,
            device_endpoint_limit: 5,
            authorize_endpoint_limit: 20,
            revoke_endpoint_limit: 10,
            register_endpoint_limit: 5,
            login_endpoint_limit: 10,
            internal_service_secret: None,
        };

        let app = test::init_service(
            App::new()
                .wrap(RateLimitMiddleware::new(
                    redis_client.clone(),
                    config.clone(),
                ))
                .route("/auth/token", web::post().to(test_handler)),
        )
        .await;

        // Client 1 makes 3 requests
        for _ in 1..=3 {
            let req = test::TestRequest::post()
                .uri("/auth/token")
                .insert_header(("X-Client-ID", "client-1"))
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::OK);
        }

        // Client 2 should still be able to make requests
        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Client-ID", "client-2"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_no_rate_limit_for_untracked_endpoints() {
        let redis_client = setup_redis_client();

        if redis_client
            .get_multiplexed_async_connection()
            .await
            .is_err()
        {
            println!("Redis not available, skipping integration test");
            return;
        }

        let config = RateLimitConfig::default();

        let app = test::init_service(
            App::new()
                .wrap(RateLimitMiddleware::new(
                    redis_client.clone(),
                    config.clone(),
                ))
                .route("/health", web::get().to(test_handler)),
        )
        .await;

        // Should allow unlimited requests to untracked endpoints
        for _ in 1..=50 {
            let req = test::TestRequest::get()
                .uri("/health")
                .insert_header(("X-Client-ID", "test-client"))
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }

    #[test]
    fn test_extract_client_id_from_header() {
        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Client-ID", "custom-client-id"))
            .to_srv_request();

        let client_id = RateLimitMiddleware::extract_client_id(&req);
        assert_eq!(client_id, "custom-client-id");
    }

    #[test]
    fn test_extract_client_id_fallback_to_ip() {
        let req = test::TestRequest::post()
            .uri("/auth/token")
            .peer_addr("192.168.1.1:8080".parse().unwrap())
            .to_srv_request();

        let client_id = RateLimitMiddleware::extract_client_id(&req);
        assert_eq!(client_id, "192.168.1.1");
    }

    #[test]
    fn test_check_internal_bypass_with_correct_secret() {
        let config = RateLimitConfig::default().with_internal_secret("my-secret".to_string());

        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Internal-Service", "my-secret"))
            .to_srv_request();

        assert!(RateLimitMiddleware::check_internal_bypass(&req, &config));
    }

    #[test]
    fn test_check_internal_bypass_with_wrong_secret() {
        let config = RateLimitConfig::default().with_internal_secret("my-secret".to_string());

        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Internal-Service", "wrong-secret"))
            .to_srv_request();

        assert!(!RateLimitMiddleware::check_internal_bypass(&req, &config));
    }

    #[test]
    fn test_check_internal_bypass_no_header() {
        let config = RateLimitConfig::default().with_internal_secret("my-secret".to_string());

        let req = test::TestRequest::post()
            .uri("/auth/token")
            .to_srv_request();

        assert!(!RateLimitMiddleware::check_internal_bypass(&req, &config));
    }
}
