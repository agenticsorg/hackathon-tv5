use actix_web::http::StatusCode;
use actix_web::{test, web, App, HttpResponse};
use media_gateway_auth::{
    jwt::JwtManager,
    middleware::{RateLimitConfig, RateLimitMiddleware},
    oauth::OAuthConfig,
    oauth::OAuthManager,
    rbac::RbacManager,
    scopes::ScopeManager,
    server::AppState,
    session::SessionManager,
    storage::AuthStorage,
    token_family::TokenFamilyManager,
};
use std::sync::Arc;

async fn test_handler() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "ok"})))
}

fn setup_redis_client() -> redis::Client {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    redis::Client::open(redis_url).expect("Failed to create Redis client")
}

fn setup_app_state() -> web::Data<AppState> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Use test JWT keys (RSA 2048-bit for testing)
    let private_key = b"-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEAu1SU1LfVLPHCozMxH2Mo4lgOEePzNm0tRgeLezV6ffAt0gun
VTLw7onLRnrq0/IzW7yWR7QkrmBL7jTKEn5u+qKhbwKfBstIs+bMY2Zkp18gnTxK
LxoS2tFczGkPLPgizskuemMghRniWaoLcyehkd3qqGElvW/VDL5AaWTg0nLVkjRo
9z+40RQzuVaE8AkAFmxZzow3x+VJYKdjykkJ0iT9wCS0DRTXu269V264Vf/3jvWa
dxqQXx+KQzEHlf1nJpxA9y7Ts1QkbnrJeGxnPmkqJYWBj7DZPNWWjhJ0Jf4k4e8X
WxMjwkBdlxqTxvv7XrxKzIjcjYlPjMw5M8XuuwIDAQABAoIBAE/sSNGLJnLvN3DU
/A4KdPMKLaRNx3MqjYnKqfTjTCCp5eAj9cHOsHN9P4nlLbqHV3LWqr8JLZbwQkNe
KdYqN3bxdLrW9vLYQ0zHMrMBd0L0rljDkMOuF6dJcfzLOdw9y5m0F+3EXpOUiLlP
bGH9Y9WHCJQlLTGgaJJa1S0BmCPt6ZcJqHqUhsZa9Jdh3rz1TUvHZaCnCcY7y+vG
RTn+MLqM1mSQJGnMGwUlpqBW8c8eLm+FJvXMnQqEzCsD1u8g6M7RKcL9lVLnOHzZ
L9oVMUjLwH7MdEhQPW8pF7xMbvMKqOGy3WXYl9T7vXjLBKJDxqI7jKFXLhYYNrqP
TCvfY/ECgYEA4Q3E3K3aVJLdpG5xCX5r2jYKIHOXzXx7h8AuQLDJ7FKPpPxGC6EQ
bJVMNWjPZU7qZNNi+0MYjWGpU4rDHXnNL/rLJY6gZM8GqJDgq9g0F7lJoNEQPZxN
LfQ8VcpFCVZp5cDhLPPL1xqvFUJ3cJA0vWPBLLG3J2LN0hYNpIBvJwsCgYEA1LJj
yCgVvlQMHCmHDuN7GmDvHvEhYi+5F4LEJkBPJ4uiPVt3L8n9tTnPV1mQZZjb5yfq
7j3LvOmGvXkDJRyZzKphQJLmPPJM1IXlYPi9LPKQdUKLlPFQnLVFRUdPB8qROzGJ
SqhUIqjjQxFNYI6r8tLsIvLBxrPbCOVNcQkK9hkCgYEAp5TYc0vGYwHPTH4cFQG5
qCLMb3sWVLnYL9tKGRnxwTb3WEoLXKJvLUPJx8L3pBz4QvBaQYNWqNxj2Mm1hSd7
xFPhQKGHLzTlpqLdCHLsNt7YI9+TqRd3AQx/+BcWvNqF8M4/LLdXZPR+qxOGJJjq
hTnMNqTJPkU7pUUOSqBm2TkCgYBTF6QJBqP9HcFFOKMWj9c3x3lkLPZBMPQCPsJU
P1OJdPNvQi0h0z9QUxPWnF8LqNqO4/l9Kqv8oKpqP1qHLxhKH1KlAJPJgGMOKlTP
jbIqM7GECbXN7PYCThUYpPCwUyqN0pqMN0FYZT2YqBqJQQGsJQq3cHtJJHKvFhTx
3LdCQQKBgQDKNvD5L1SN7f6vIhCpJmFJF4gHXhN6F7nxQcJMK3lJJLm3sQBBQH7x
VoGhkYyN2MQIuGJB4gvzOCKJT0pMKQzYqPQ3u6AhMvGbJY8zxJ1wkLKF5aYPVRBU
TJqTGLKqALJa1vdBPx8VQxhYNqLZVKKTcPFQGzKQKQPNb0wN3dL8lQ==
-----END RSA PRIVATE KEY-----";

    let public_key = b"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAu1SU1LfVLPHCozMxH2Mo
4lgOEePzNm0tRgeLezV6ffAt0gunVTLw7onLRnrq0/IzW7yWR7QkrmBL7jTKEn5u
+qKhbwKfBstIs+bMY2Zkp18gnTxKLxoS2tFczGkPLPgizskuemMghRniWaoLcyeh
kd3qqGElvW/VDL5AaWTg0nLVkjRo9z+40RQzuVaE8AkAFmxZzow3x+VJYKdjykkJ
0iT9wCS0DRTXu269V264Vf/3jvWadxqQXx+KQzEHlf1nJpxA9y7Ts1QkbnrJeGxn
PmkqJYWBj7DZPNWWjhJ0Jf4k4e8XWxMjwkBdlxqTxvv7XrxKzIjcjYlPjMw5M8Xu
uwIDAQAB
-----END PUBLIC KEY-----";

    let jwt_manager = Arc::new(
        JwtManager::new(
            private_key,
            public_key,
            "https://test.mediagateway.io".to_string(),
            "test-audience".to_string(),
        )
        .expect("Failed to create JWT manager"),
    );

    let session_manager =
        Arc::new(SessionManager::new(&redis_url).expect("Failed to create session manager"));

    let token_family_manager = Arc::new(
        TokenFamilyManager::new(&redis_url).expect("Failed to create token family manager"),
    );

    let storage = Arc::new(AuthStorage::new(&redis_url).expect("Failed to create auth storage"));

    let oauth_config = OAuthConfig {
        providers: std::collections::HashMap::new(),
    };

    web::Data::new(AppState {
        jwt_manager,
        session_manager,
        oauth_manager: Arc::new(OAuthManager::new(oauth_config)),
        rbac_manager: Arc::new(RbacManager::new()),
        scope_manager: Arc::new(ScopeManager::new()),
        storage,
        token_family_manager,
    })
}

#[actix_web::test]
async fn test_rate_limit_token_endpoint() {
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

    let config = RateLimitConfig::new(3, 5, 20, 10);
    let app_state = setup_app_state();

    let app = test::init_service(
        App::new()
            .wrap(RateLimitMiddleware::new(
                redis_client.clone(),
                config.clone(),
            ))
            .app_data(app_state.clone())
            .route("/auth/token", web::post().to(test_handler)),
    )
    .await;

    // First 3 requests should succeed
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Client-ID", "test-client-token-001"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Request {} should succeed",
            i
        );

        // Check rate limit headers
        assert!(resp.headers().get("x-ratelimit-limit").is_some());
        assert!(resp.headers().get("x-ratelimit-remaining").is_some());
    }

    // 4th request should be rate limited
    let req = test::TestRequest::post()
        .uri("/auth/token")
        .insert_header(("X-Client-ID", "test-client-token-001"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

    // Check 429 response headers
    assert!(resp.headers().get("Retry-After").is_some());
    assert!(resp.headers().get("X-RateLimit-Limit").is_some());
    assert_eq!(
        resp.headers()
            .get("X-RateLimit-Limit")
            .unwrap()
            .to_str()
            .unwrap(),
        "3"
    );
}

#[actix_web::test]
async fn test_rate_limit_device_endpoint() {
    let redis_client = setup_redis_client();

    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping integration test");
        return;
    }

    let config = RateLimitConfig::new(10, 5, 20, 10);
    let app_state = setup_app_state();

    let app = test::init_service(
        App::new()
            .wrap(RateLimitMiddleware::new(
                redis_client.clone(),
                config.clone(),
            ))
            .app_data(app_state.clone())
            .route("/auth/device", web::post().to(test_handler)),
    )
    .await;

    // First 5 requests should succeed
    for i in 1..=5 {
        let req = test::TestRequest::post()
            .uri("/auth/device")
            .insert_header(("X-Client-ID", "test-client-device-001"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Request {} should succeed",
            i
        );
    }

    // 6th request should be rate limited
    let req = test::TestRequest::post()
        .uri("/auth/device")
        .insert_header(("X-Client-ID", "test-client-device-001"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[actix_web::test]
async fn test_rate_limit_authorize_endpoint() {
    let redis_client = setup_redis_client();

    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping integration test");
        return;
    }

    let config = RateLimitConfig::new(10, 5, 3, 10);
    let app_state = setup_app_state();

    let app = test::init_service(
        App::new()
            .wrap(RateLimitMiddleware::new(
                redis_client.clone(),
                config.clone(),
            ))
            .app_data(app_state.clone())
            .route("/auth/authorize", web::get().to(test_handler)),
    )
    .await;

    // First 3 requests should succeed
    for i in 1..=3 {
        let req = test::TestRequest::get()
            .uri("/auth/authorize")
            .insert_header(("X-Client-ID", "test-client-authorize-001"))
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
    let req = test::TestRequest::get()
        .uri("/auth/authorize")
        .insert_header(("X-Client-ID", "test-client-authorize-001"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[actix_web::test]
async fn test_rate_limit_different_clients_isolated() {
    let redis_client = setup_redis_client();

    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping integration test");
        return;
    }

    let config = RateLimitConfig::new(3, 5, 20, 10);
    let app_state = setup_app_state();

    let app = test::init_service(
        App::new()
            .wrap(RateLimitMiddleware::new(
                redis_client.clone(),
                config.clone(),
            ))
            .app_data(app_state.clone())
            .route("/auth/token", web::post().to(test_handler)),
    )
    .await;

    // Client 1 makes 3 requests
    for _ in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Client-ID", "client-isolated-1"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // Client 2 should still be able to make requests
    for _ in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Client-ID", "client-isolated-2"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // Client 1 should now be rate limited
    let req = test::TestRequest::post()
        .uri("/auth/token")
        .insert_header(("X-Client-ID", "client-isolated-1"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

    // Client 2 should also now be rate limited
    let req = test::TestRequest::post()
        .uri("/auth/token")
        .insert_header(("X-Client-ID", "client-isolated-2"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[actix_web::test]
async fn test_rate_limit_internal_bypass() {
    let redis_client = setup_redis_client();

    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping integration test");
        return;
    }

    let config =
        RateLimitConfig::new(2, 5, 20, 10).with_internal_secret("test-secret-key".to_string());
    let app_state = setup_app_state();

    let app = test::init_service(
        App::new()
            .wrap(RateLimitMiddleware::new(
                redis_client.clone(),
                config.clone(),
            ))
            .app_data(app_state.clone())
            .route("/auth/token", web::post().to(test_handler)),
    )
    .await;

    // Make requests beyond the limit with bypass header
    for i in 1..=10 {
        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Client-ID", "test-client-bypass"))
            .insert_header(("X-Internal-Service", "test-secret-key"))
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
async fn test_rate_limit_retry_after_header() {
    let redis_client = setup_redis_client();

    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping integration test");
        return;
    }

    let config = RateLimitConfig::new(2, 5, 20, 10);
    let app_state = setup_app_state();

    let app = test::init_service(
        App::new()
            .wrap(RateLimitMiddleware::new(
                redis_client.clone(),
                config.clone(),
            ))
            .app_data(app_state.clone())
            .route("/auth/token", web::post().to(test_handler)),
    )
    .await;

    // Exhaust rate limit
    for _ in 1..=2 {
        let req = test::TestRequest::post()
            .uri("/auth/token")
            .insert_header(("X-Client-ID", "test-client-retry-after"))
            .to_request();
        let _ = test::call_service(&app, req).await;
    }

    // Next request should be rate limited
    let req = test::TestRequest::post()
        .uri("/auth/token")
        .insert_header(("X-Client-ID", "test-client-retry-after"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

    // Verify Retry-After header is present and is a positive integer
    let retry_after = resp
        .headers()
        .get("Retry-After")
        .expect("Retry-After header missing");
    let retry_after_value: u64 = retry_after
        .to_str()
        .expect("Invalid Retry-After header")
        .parse()
        .expect("Retry-After is not a number");
    assert!(retry_after_value > 0 && retry_after_value <= 60);
}

#[actix_web::test]
async fn test_rate_limit_health_endpoint_no_limit() {
    let redis_client = setup_redis_client();

    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping integration test");
        return;
    }

    let config = RateLimitConfig::new(2, 5, 20, 10);
    let app_state = setup_app_state();

    let app = test::init_service(
        App::new()
            .wrap(RateLimitMiddleware::new(
                redis_client.clone(),
                config.clone(),
            ))
            .app_data(app_state.clone())
            .route("/health", web::get().to(test_handler)),
    )
    .await;

    // Should allow unlimited requests to /health (no rate limit configured)
    for i in 1..=20 {
        let req = test::TestRequest::get()
            .uri("/health")
            .insert_header(("X-Client-ID", "test-client-health"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Request {} to /health should succeed",
            i
        );
    }
}
