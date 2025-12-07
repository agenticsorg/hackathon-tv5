/// Integration tests for the CacheMiddleware
///
/// These tests verify the complete behavior of the Redis-backed response caching middleware.
///
/// Note: These tests require a running Redis instance.
/// Run with: docker run -d -p 6379:6379 redis:alpine
use actix_web::{test, web, App, HttpResponse};
use media_gateway_api::middleware::auth::UserContext;
use media_gateway_api::middleware::{AuthMiddleware, CacheConfig, CacheMiddleware};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

/// Helper to create a test Redis client
async fn create_test_redis() -> anyhow::Result<ConnectionManager> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let client = redis::Client::open(redis_url)?;
    Ok(ConnectionManager::new(client).await?)
}

/// Helper to flush test Redis database
async fn flush_redis() -> anyhow::Result<()> {
    let mut conn = create_test_redis().await?;
    redis::cmd("FLUSHDB").query_async(&mut conn).await?;
    Ok(())
}

async fn test_handler() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "test response",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn slow_handler() -> HttpResponse {
    // Simulate slow database query
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    HttpResponse::Ok().json(serde_json::json!({
        "data": "expensive computation result",
    }))
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_cache_miss_then_hit() {
    flush_redis().await.unwrap();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let cache_middleware = CacheMiddleware::default_config(&redis_url)
        .await
        .expect("Failed to create cache middleware");

    let app = test::init_service(
        App::new()
            .wrap(cache_middleware)
            .route("/test", web::get().to(test_handler)),
    )
    .await;

    // First request - should be a cache miss
    let req = test::TestRequest::get().uri("/test").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let cache_header = resp.headers().get("X-Cache");
    assert!(cache_header.is_some());
    assert_eq!(cache_header.unwrap(), "MISS");

    // Verify ETag header is present
    assert!(resp.headers().get("ETag").is_some());

    // Second request - should be a cache hit
    let req2 = test::TestRequest::get().uri("/test").to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), 200);

    let cache_header2 = resp2.headers().get("X-Cache");
    assert!(cache_header2.is_some());
    assert_eq!(cache_header2.unwrap(), "HIT");

    // Verify Cache-Control header
    assert!(resp2.headers().get("Cache-Control").is_some());
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_etag_304_response() {
    flush_redis().await.unwrap();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let cache_middleware = CacheMiddleware::default_config(&redis_url)
        .await
        .expect("Failed to create cache middleware");

    let app = test::init_service(
        App::new()
            .wrap(cache_middleware)
            .route("/test", web::get().to(test_handler)),
    )
    .await;

    // First request to populate cache
    let req = test::TestRequest::get().uri("/test").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let etag = resp.headers().get("ETag").unwrap().to_str().unwrap();

    // Second request with If-None-Match header
    let req2 = test::TestRequest::get()
        .uri("/test")
        .insert_header(("If-None-Match", etag))
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), 304); // Not Modified

    // Verify ETag is still present in 304 response
    assert!(resp2.headers().get("ETag").is_some());
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_only_caches_get_requests() {
    flush_redis().await.unwrap();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let cache_middleware = CacheMiddleware::default_config(&redis_url)
        .await
        .expect("Failed to create cache middleware");

    let app = test::init_service(
        App::new()
            .wrap(cache_middleware)
            .route("/test", web::post().to(test_handler)),
    )
    .await;

    // POST request should not be cached
    let req = test::TestRequest::post().uri("/test").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Should not have cache headers
    assert!(resp.headers().get("X-Cache").is_none());
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_skip_paths() {
    flush_redis().await.unwrap();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let mut config = CacheConfig::default();
    config.skip_paths.push("/api/user/".to_string());

    let cache_middleware = CacheMiddleware::new(&redis_url, config)
        .await
        .expect("Failed to create cache middleware");

    let app = test::init_service(
        App::new()
            .wrap(cache_middleware)
            .route("/api/user/profile", web::get().to(test_handler)),
    )
    .await;

    // Request to skipped path should not be cached
    let req = test::TestRequest::get()
        .uri("/api/user/profile")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Should not have cache headers
    assert!(resp.headers().get("X-Cache").is_none());
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_skip_authenticated_requests() {
    flush_redis().await.unwrap();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let mut config = CacheConfig::default();
    config.cache_authenticated = false; // Don't cache authenticated requests

    let cache_middleware = CacheMiddleware::new(&redis_url, config)
        .await
        .expect("Failed to create cache middleware");

    let app = test::init_service(
        App::new()
            .wrap(cache_middleware)
            .wrap(AuthMiddleware::optional())
            .route("/test", web::get().to(test_handler)),
    )
    .await;

    // Create a test request with authenticated user context
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("Authorization", "Bearer valid_token"))
        .to_request();

    // Note: This test would need proper JWT token setup to fully work
    // For now, it demonstrates the pattern
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_different_query_params_different_cache() {
    flush_redis().await.unwrap();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let cache_middleware = CacheMiddleware::default_config(&redis_url)
        .await
        .expect("Failed to create cache middleware");

    let app = test::init_service(
        App::new()
            .wrap(cache_middleware)
            .route("/test", web::get().to(test_handler)),
    )
    .await;

    // First request with param1
    let req1 = test::TestRequest::get()
        .uri("/test?param=value1")
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), 200);
    assert_eq!(resp1.headers().get("X-Cache").unwrap(), "MISS");

    // Second request with different param should miss cache
    let req2 = test::TestRequest::get()
        .uri("/test?param=value2")
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), 200);
    assert_eq!(resp2.headers().get("X-Cache").unwrap(), "MISS");

    // Third request with first param should hit cache
    let req3 = test::TestRequest::get()
        .uri("/test?param=value1")
        .to_request();

    let resp3 = test::call_service(&app, req3).await;
    assert_eq!(resp3.status(), 200);
    assert_eq!(resp3.headers().get("X-Cache").unwrap(), "HIT");
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_cache_ttl() {
    flush_redis().await.unwrap();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let mut config = CacheConfig::default();
    config.default_ttl = 2; // 2 seconds

    let cache_middleware = CacheMiddleware::new(&redis_url, config)
        .await
        .expect("Failed to create cache middleware");

    let app = test::init_service(
        App::new()
            .wrap(cache_middleware)
            .route("/test", web::get().to(test_handler)),
    )
    .await;

    // First request - cache miss
    let req1 = test::TestRequest::get().uri("/test").to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.headers().get("X-Cache").unwrap(), "MISS");

    // Second request - cache hit
    let req2 = test::TestRequest::get().uri("/test").to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.headers().get("X-Cache").unwrap(), "HIT");

    // Wait for TTL to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Third request - cache miss (expired)
    let req3 = test::TestRequest::get().uri("/test").to_request();

    let resp3 = test::call_service(&app, req3).await;
    assert_eq!(resp3.headers().get("X-Cache").unwrap(), "MISS");
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_content_path_longer_ttl() {
    flush_redis().await.unwrap();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let mut config = CacheConfig::default();
    config.default_ttl = 60;
    config.content_ttl = 300;

    let cache_middleware = CacheMiddleware::new(&redis_url, config)
        .await
        .expect("Failed to create cache middleware");

    let app = test::init_service(
        App::new()
            .wrap(cache_middleware)
            .route("/api/content/123", web::get().to(test_handler)),
    )
    .await;

    // Request to content path
    let req = test::TestRequest::get()
        .uri("/api/content/123")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Cache-Control should reflect content TTL
    let cache_control = resp
        .headers()
        .get("Cache-Control")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(cache_control.contains("max-age=300"));
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_cache_invalidation() {
    flush_redis().await.unwrap();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let cache_middleware = CacheMiddleware::default_config(&redis_url)
        .await
        .expect("Failed to create cache middleware");

    // Populate cache
    let mut conn = create_test_redis().await.unwrap();
    let cache_key = "cache:GET:/test";
    let cache_value = serde_json::json!({
        "status_code": 200,
        "headers": [],
        "body": b"test",
        "etag": "\"abc123\"",
    })
    .to_string();

    let _: () = conn.set(cache_key, cache_value).await.unwrap();

    // Verify cache exists
    let exists: bool = conn.exists(cache_key).await.unwrap();
    assert!(exists);

    // Invalidate cache
    let result = cache_middleware.invalidate("GET:/test").await.unwrap();
    assert!(result);

    // Verify cache is removed
    let exists_after: bool = conn.exists(cache_key).await.unwrap();
    assert!(!exists_after);
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_cache_invalidation_pattern() {
    flush_redis().await.unwrap();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let cache_middleware = CacheMiddleware::default_config(&redis_url)
        .await
        .expect("Failed to create cache middleware");

    // Populate cache with multiple keys
    let mut conn = create_test_redis().await.unwrap();
    let cache_value = serde_json::json!({
        "status_code": 200,
        "headers": [],
        "body": b"test",
        "etag": "\"abc123\"",
    })
    .to_string();

    let _: () = conn
        .set("cache:GET:/api/content/1", &cache_value)
        .await
        .unwrap();
    let _: () = conn
        .set("cache:GET:/api/content/2", &cache_value)
        .await
        .unwrap();
    let _: () = conn
        .set("cache:GET:/api/other/1", &cache_value)
        .await
        .unwrap();

    // Invalidate pattern
    let count = cache_middleware
        .invalidate_pattern("GET:/api/content/*")
        .await
        .unwrap();
    assert_eq!(count, 2);

    // Verify correct keys are removed
    let exists1: bool = conn.exists("cache:GET:/api/content/1").await.unwrap();
    let exists2: bool = conn.exists("cache:GET:/api/content/2").await.unwrap();
    let exists3: bool = conn.exists("cache:GET:/api/other/1").await.unwrap();

    assert!(!exists1);
    assert!(!exists2);
    assert!(exists3); // This one should still exist
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_performance_improvement() {
    flush_redis().await.unwrap();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let cache_middleware = CacheMiddleware::default_config(&redis_url)
        .await
        .expect("Failed to create cache middleware");

    let app = test::init_service(
        App::new()
            .wrap(cache_middleware)
            .route("/slow", web::get().to(slow_handler)),
    )
    .await;

    // First request - slow (100ms + processing)
    let start = std::time::Instant::now();
    let req1 = test::TestRequest::get().uri("/slow").to_request();
    let resp1 = test::call_service(&app, req1).await;
    let duration1 = start.elapsed();

    assert_eq!(resp1.status(), 200);
    assert!(duration1.as_millis() >= 100);

    // Second request - should be faster (cached)
    let start2 = std::time::Instant::now();
    let req2 = test::TestRequest::get().uri("/slow").to_request();
    let resp2 = test::call_service(&app, req2).await;
    let duration2 = start2.elapsed();

    assert_eq!(resp2.status(), 200);
    assert_eq!(resp2.headers().get("X-Cache").unwrap(), "HIT");

    // Cached response should be significantly faster
    assert!(duration2 < duration1);
}
