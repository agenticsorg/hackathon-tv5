/// Example usage of the CacheMiddleware in the Media Gateway API
///
/// This demonstrates how to integrate the Redis-backed response caching middleware
/// into an Actix-web application.
use actix_web::{web, App, HttpResponse, HttpServer};
use media_gateway_api::middleware::{CacheConfig, CacheMiddleware};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Redis connection string
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    // Create custom cache configuration
    let cache_config = CacheConfig {
        default_ttl: 60,            // 1 minute for general endpoints
        content_ttl: 300,           // 5 minutes for content endpoints
        cache_authenticated: false, // Don't cache user-specific responses
        skip_paths: vec![
            "/api/user/".to_string(),
            "/api/sync/".to_string(),
            "/api/admin/".to_string(),
            "/health".to_string(),
        ],
        skip_query_params: vec![
            "nocache".to_string(),
            "timestamp".to_string(),
            "random".to_string(),
        ],
    };

    // Initialize cache middleware
    let cache_middleware = CacheMiddleware::new(&redis_url, cache_config)
        .await
        .expect("Failed to initialize cache middleware");

    let cache_middleware_data = web::Data::new(cache_middleware.clone());

    println!("Starting server with cache middleware at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            // Apply cache middleware globally
            .wrap(cache_middleware.clone())
            .app_data(cache_middleware_data.clone())
            // Routes
            .route("/api/content/{id}", web::get().to(get_content))
            .route("/api/search", web::get().to(search))
            .route("/api/user/profile", web::get().to(get_user_profile))
            .route("/health", web::get().to(health))
            // Cache management endpoints
            .route("/cache/invalidate", web::post().to(invalidate_cache))
            .route(
                "/cache/invalidate/{pattern}",
                web::delete().to(invalidate_pattern),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// Example: Cacheable content endpoint (will be cached for 5 minutes)
async fn get_content(path: web::Path<String>) -> HttpResponse {
    let content_id = path.into_inner();

    // Simulate database lookup
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    HttpResponse::Ok().json(serde_json::json!({
        "id": content_id,
        "title": "Example Content",
        "description": "This response will be cached",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

// Example: Search endpoint with query parameters
async fn search(query: web::Query<SearchQuery>) -> HttpResponse {
    // Simulate search operation
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    HttpResponse::Ok().json(serde_json::json!({
        "query": query.q,
        "results": vec![
            {"id": "1", "title": "Result 1"},
            {"id": "2", "title": "Result 2"},
        ],
        "cached": false, // Will become true when served from cache
    }))
}

// Example: User-specific endpoint (will NOT be cached by default)
async fn get_user_profile() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "user_id": "user123",
        "name": "John Doe",
        "email": "john@example.com",
        "note": "This response is NOT cached because it's user-specific",
    }))
}

// Health check (skipped by cache configuration)
async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

// Cache invalidation endpoint
async fn invalidate_cache(
    cache: web::Data<CacheMiddleware>,
    body: web::Json<InvalidateRequest>,
) -> HttpResponse {
    match cache.invalidate(&body.key).await {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": format!("Cache key '{}' invalidated", body.key),
        })),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "message": format!("Cache key '{}' not found", body.key),
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
        })),
    }
}

// Pattern-based cache invalidation
async fn invalidate_pattern(
    cache: web::Data<CacheMiddleware>,
    pattern: web::Path<String>,
) -> HttpResponse {
    match cache.invalidate_pattern(&pattern.into_inner()).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "invalidated": count,
            "message": format!("Invalidated {} cache entries", count),
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
        })),
    }
}

#[derive(serde::Deserialize)]
struct SearchQuery {
    q: String,
}

#[derive(serde::Deserialize)]
struct InvalidateRequest {
    key: String,
}
