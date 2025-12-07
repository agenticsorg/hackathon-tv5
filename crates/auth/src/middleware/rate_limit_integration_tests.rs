#![cfg(test)]
//! Integration tests for rate limiting middleware
//!
//! These tests require a running Redis instance.
//! Run with: cargo test --package media-gateway-auth -- --test-threads=1

use super::rate_limit::{RateLimitConfig, RateLimitMiddleware};
use actix_web::{test, web, App, Error, HttpResponse};
use redis::AsyncCommands;
use std::time::Duration;
use tokio::time::sleep;

async fn dummy_handler() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "success"})))
}

fn get_redis_client() -> redis::Client {
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    redis::Client::open(redis_url).expect("Failed to create Redis client")
}

async fn cleanup_redis(redis_client: &redis::Client, pattern: &str) {
    if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await
            .unwrap_or_default();

        for key in keys {
            let _: Result<(), redis::RedisError> = conn.del(&key).await;
        }
    }
}

#[actix_web::test]
async fn test_sliding_window_reset_after_60_seconds() {
    let redis_client = get_redis_client();

    // Skip test if Redis is not available
    if redis_client.get_multiplexed_async_connection().await.is_err() {
        println!("Redis not available, skipping test");
        return;
    }

    cleanup_redis(&redis_client, "rate_limit:*test-window-client*").await;

    let config = RateLimitConfig {
        token_endpoint_limit: 2,
        device_endpoint_limit: 5,
        authorize_endpoint_limit: 20,
        revoke_endpoint_limit: 10,
        internal_service_secret: None,
    };

    let app = test::init_service(
        App::new()
            .wrap(RateLimitMiddleware::new(redis_client.clone(), config))
            .route("/auth/token", web::post().to(dummy_handler)),
    )
    .await;

    let client_id = "test-window-client";

    // Make 2 requests (should succeed)
    for i in 1..=2 {
        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Client-ID", client_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200, "Request {} should succeed", i);
    }

    // 3rd request should fail
    let req = test::TestRequest::post()
        .uri("/auth/token")
        .insert_header(("X-Client-ID", client_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 429, "3rd request should be rate limited");

    println!("Waiting for window to reset (this may take up to 60 seconds)...");

    // Wait for the window to slide (61 seconds to be safe)
    sleep(Duration::from_secs(61)).await;

    // After window reset, requests should work again
    let req = test::TestRequest::post()
        .uri("/auth/token")
        .insert_header(("X-Client-ID", client_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Request after window reset should succeed");

    cleanup_redis(&redis_client, "rate_limit:*test-window-client*").await;
}

#[actix_web::test]
async fn test_11th_request_blocked() {
    let redis_client = get_redis_client();

    if redis_client.get_multiplexed_async_connection().await.is_err() {
        println!("Redis not available, skipping test");
        return;
    }

    cleanup_redis(&redis_client, "rate_limit:*test-11th-client*").await;

    let config = RateLimitConfig {
        token_endpoint_limit: 10,
        device_endpoint_limit: 5,
        authorize_endpoint_limit: 20,
        revoke_endpoint_limit: 10,
        internal_service_secret: None,
    };

    let app = test::init_service(
        App::new()
            .wrap(RateLimitMiddleware::new(redis_client.clone(), config))
            .route("/auth/token", web::post().to(dummy_handler)),
    )
    .await;

    let client_id = "test-11th-client";

    // First 10 requests should succeed
    for i in 1..=10 {
        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Client-ID", client_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200, "Request {} should succeed", i);

        // Verify rate limit headers are present
        let limit_header = resp.headers().get("x-ratelimit-limit");
        assert!(limit_header.is_some());
        assert_eq!(limit_header.unwrap().to_str().unwrap(), "10");

        let remaining_header = resp.headers().get("x-ratelimit-remaining");
        assert!(remaining_header.is_some());
    }

    // 11th request should be blocked with 429
    let req = test::TestRequest::post()
        .uri("/auth/token")
        .insert_header(("X-Client-ID", client_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 429, "11th request should be rate limited");

    // Check response headers
    assert!(resp.headers().get("Retry-After").is_some());
    assert!(resp.headers().get("X-RateLimit-Limit").is_some());
    assert_eq!(
        resp.headers().get("X-RateLimit-Limit").unwrap().to_str().unwrap(),
        "10"
    );

    cleanup_redis(&redis_client, "rate_limit:*test-11th-client*").await;
}

#[actix_web::test]
async fn test_bypass_mechanism_with_secret() {
    let redis_client = get_redis_client();

    if redis_client.get_multiplexed_async_connection().await.is_err() {
        println!("Redis not available, skipping test");
        return;
    }

    cleanup_redis(&redis_client, "rate_limit:*test-bypass-client*").await;

    let secret = "internal-service-secret-xyz";
    let config = RateLimitConfig {
        token_endpoint_limit: 3,
        device_endpoint_limit: 5,
        authorize_endpoint_limit: 20,
        revoke_endpoint_limit: 10,
        internal_service_secret: Some(secret.to_string()),
    };

    let app = test::init_service(
        App::new()
            .wrap(RateLimitMiddleware::new(redis_client.clone(), config))
            .route("/auth/token", web::post().to(dummy_handler)),
    )
    .await;

    let client_id = "test-bypass-client";

    // Make 20 requests with bypass header (all should succeed)
    for i in 1..=20 {
        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Client-ID", client_id))
            .insert_header(("X-Internal-Service", secret))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            200,
            "Request {} with bypass should succeed",
            i
        );
    }

    // Now try without bypass header (4th request should fail since limit is 3)
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Client-ID", format!("{}-no-bypass", client_id)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200, "Request {} should succeed", i);
    }

    let req = test::TestRequest::post()
        .uri("/auth/token")
        .insert_header(("X-Client-ID", format!("{}-no-bypass", client_id)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 429, "4th request without bypass should fail");

    cleanup_redis(&redis_client, "rate_limit:*test-bypass-client*").await;
}

#[actix_web::test]
async fn test_different_endpoints_different_limits() {
    let redis_client = get_redis_client();

    if redis_client.get_multiplexed_async_connection().await.is_err() {
        println!("Redis not available, skipping test");
        return;
    }

    cleanup_redis(&redis_client, "rate_limit:*test-multi-endpoint*").await;

    let config = RateLimitConfig {
        token_endpoint_limit: 5,
        device_endpoint_limit: 3,
        authorize_endpoint_limit: 10,
        revoke_endpoint_limit: 2,
        internal_service_secret: None,
    };

    let app = test::init_service(
        App::new()
            .wrap(RateLimitMiddleware::new(redis_client.clone(), config))
            .route("/auth/token", web::post().to(dummy_handler))
            .route("/auth/device", web::post().to(dummy_handler))
            .route("/auth/authorize", web::get().to(dummy_handler))
            .route("/auth/revoke", web::post().to(dummy_handler)),
    )
    .await;

    let client_id = "test-multi-endpoint";

    // Test /auth/device with limit of 3
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/auth/device")
            .insert_header(("X-Client-ID", client_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200, "Device request {} should succeed", i);
    }

    let req = test::TestRequest::post()
        .uri("/auth/device")
        .insert_header(("X-Client-ID", client_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 429, "4th device request should fail");

    // Test /auth/token with different limit (5) - should still work
    for i in 1..=5 {
        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Client-ID", client_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200, "Token request {} should succeed", i);
    }

    cleanup_redis(&redis_client, "rate_limit:*test-multi-endpoint*").await;
}

#[actix_web::test]
async fn test_rate_limit_response_format() {
    let redis_client = get_redis_client();

    if redis_client.get_multiplexed_async_connection().await.is_err() {
        println!("Redis not available, skipping test");
        return;
    }

    cleanup_redis(&redis_client, "rate_limit:*test-response-format*").await;

    let config = RateLimitConfig {
        token_endpoint_limit: 1,
        device_endpoint_limit: 5,
        authorize_endpoint_limit: 20,
        revoke_endpoint_limit: 10,
        internal_service_secret: None,
    };

    let app = test::init_service(
        App::new()
            .wrap(RateLimitMiddleware::new(redis_client.clone(), config))
            .route("/auth/token", web::post().to(dummy_handler)),
    )
    .await;

    let client_id = "test-response-format";

    // First request succeeds
    let req = test::TestRequest::post()
        .uri("/auth/token")
        .insert_header(("X-Client-ID", client_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Second request should be rate limited
    let req = test::TestRequest::post()
        .uri("/auth/token")
        .insert_header(("X-Client-ID", client_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 429);

    // Verify response body
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).expect("Invalid JSON");

    assert_eq!(json["error"], "rate_limit_exceeded");
    assert!(json["message"].as_str().unwrap().contains("Rate limit exceeded"));
    assert!(json["retry_after"].as_u64().is_some());
    assert_eq!(json["limit"], 1);
    assert!(json["current_count"].as_u64().unwrap() > 1);

    cleanup_redis(&redis_client, "rate_limit:*test-response-format*").await;
}
