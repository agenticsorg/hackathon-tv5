use actix_web::{test, web, App};
use auth::{
    error::AuthError,
    jwt::JwtManager,
    middleware::rate_limit::{RateLimitConfig, RateLimitMiddleware},
    user::{
        handlers::{login, register, LoginRequest, UserHandlerState},
        password::PasswordHasher,
        repository::{CreateUserRequest, PostgresUserRepository},
    },
};
use sqlx::PgPool;
use std::sync::Arc;

/// Helper to create test database pool
async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
    });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Helper to create test JWT manager
fn setup_test_jwt_manager() -> Arc<JwtManager> {
    // In tests, use a simple key (in production, load from secure storage)
    let private_key = include_bytes!("../../../tests/fixtures/test_private_key.pem");
    let public_key = include_bytes!("../../../tests/fixtures/test_public_key.pem");

    Arc::new(
        JwtManager::new(
            private_key,
            public_key,
            "https://test.mediagateway.io".to_string(),
            "test-audience".to_string(),
        )
        .expect("Failed to create JWT manager"),
    )
}

/// Helper to create test Redis client
fn setup_test_redis() -> redis::Client {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    redis::Client::open(redis_url).expect("Failed to create Redis client")
}

#[actix_web::test]
#[ignore] // Requires database and Redis
async fn test_user_registration_success() {
    let pool = setup_test_db().await;
    let jwt_manager = setup_test_jwt_manager();
    let password_hasher = Arc::new(PasswordHasher::new());
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));

    let state = web::Data::new(UserHandlerState {
        user_repository: user_repo.clone(),
        password_hasher: password_hasher.clone(),
        jwt_manager: jwt_manager.clone(),
        require_email_verification: false,
    });

    let app = test::init_service(
        App::new()
            .app_data(state)
            .route("/api/v1/auth/register", web::post().to(register)),
    )
    .await;

    let register_request = CreateUserRequest {
        email: format!("test_{}@example.com", uuid::Uuid::new_v4()),
        password: "SecurePass123".to_string(),
        display_name: "Test User".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Cleanup
    sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&register_request.email)
        .execute(&pool)
        .await
        .ok();
}

#[actix_web::test]
#[ignore] // Requires database
async fn test_user_registration_weak_password() {
    let pool = setup_test_db().await;
    let jwt_manager = setup_test_jwt_manager();
    let password_hasher = Arc::new(PasswordHasher::new());
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));

    let state = web::Data::new(UserHandlerState {
        user_repository: user_repo.clone(),
        password_hasher: password_hasher.clone(),
        jwt_manager: jwt_manager.clone(),
        require_email_verification: false,
    });

    let app = test::init_service(
        App::new()
            .app_data(state)
            .route("/api/v1/auth/register", web::post().to(register)),
    )
    .await;

    // Test various weak passwords
    let weak_passwords = vec![
        "short",        // Too short
        "nouppercase1", // No uppercase
        "NOLOWERCASE1", // No lowercase
        "NoNumbers",    // No numbers
    ];

    for weak_password in weak_passwords {
        let register_request = CreateUserRequest {
            email: format!("test_{}@example.com", uuid::Uuid::new_v4()),
            password: weak_password.to_string(),
            display_name: "Test User".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(&register_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            500,
            "Weak password should fail: {}",
            weak_password
        );
    }
}

#[actix_web::test]
#[ignore] // Requires database
async fn test_user_registration_duplicate_email() {
    let pool = setup_test_db().await;
    let jwt_manager = setup_test_jwt_manager();
    let password_hasher = Arc::new(PasswordHasher::new());
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));

    let state = web::Data::new(UserHandlerState {
        user_repository: user_repo.clone(),
        password_hasher: password_hasher.clone(),
        jwt_manager: jwt_manager.clone(),
        require_email_verification: false,
    });

    let app = test::init_service(
        App::new()
            .app_data(state)
            .route("/api/v1/auth/register", web::post().to(register)),
    )
    .await;

    let email = format!("duplicate_{}@example.com", uuid::Uuid::new_v4());
    let register_request = CreateUserRequest {
        email: email.clone(),
        password: "SecurePass123".to_string(),
        display_name: "Test User".to_string(),
    };

    // First registration should succeed
    let req1 = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_request)
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), 201);

    // Second registration with same email should fail
    let req2 = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_request)
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), 500);

    // Cleanup
    sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(&pool)
        .await
        .ok();
}

#[actix_web::test]
#[ignore] // Requires database and Redis
async fn test_user_login_success() {
    let pool = setup_test_db().await;
    let jwt_manager = setup_test_jwt_manager();
    let password_hasher = Arc::new(PasswordHasher::new());
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));

    let state = web::Data::new(UserHandlerState {
        user_repository: user_repo.clone(),
        password_hasher: password_hasher.clone(),
        jwt_manager: jwt_manager.clone(),
        require_email_verification: false,
    });

    let app = test::init_service(
        App::new()
            .app_data(state)
            .route("/api/v1/auth/register", web::post().to(register))
            .route("/api/v1/auth/login", web::post().to(login)),
    )
    .await;

    let email = format!("login_test_{}@example.com", uuid::Uuid::new_v4());
    let password = "SecurePass123";

    // Register user
    let register_request = CreateUserRequest {
        email: email.clone(),
        password: password.to_string(),
        display_name: "Test User".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login
    let login_request = LoginRequest {
        email: email.clone(),
        password: password.to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
    assert_eq!(body["token_type"], "Bearer");

    // Cleanup
    sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&email)
        .execute(&pool)
        .await
        .ok();
}

#[actix_web::test]
#[ignore] // Requires database
async fn test_user_login_invalid_credentials() {
    let pool = setup_test_db().await;
    let jwt_manager = setup_test_jwt_manager();
    let password_hasher = Arc::new(PasswordHasher::new());
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));

    let state = web::Data::new(UserHandlerState {
        user_repository: user_repo.clone(),
        password_hasher: password_hasher.clone(),
        jwt_manager: jwt_manager.clone(),
        require_email_verification: false,
    });

    let app = test::init_service(
        App::new()
            .app_data(state)
            .route("/api/v1/auth/login", web::post().to(login)),
    )
    .await;

    let login_request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "WrongPassword123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
#[ignore] // Requires database and Redis
async fn test_registration_rate_limit() {
    let pool = setup_test_db().await;
    let jwt_manager = setup_test_jwt_manager();
    let password_hasher = Arc::new(PasswordHasher::new());
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let redis_client = setup_test_redis();

    // Check Redis connectivity
    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping rate limit test");
        return;
    }

    let state = web::Data::new(UserHandlerState {
        user_repository: user_repo.clone(),
        password_hasher: password_hasher.clone(),
        jwt_manager: jwt_manager.clone(),
        require_email_verification: false,
    });

    let rate_limit_config = RateLimitConfig {
        token_endpoint_limit: 10,
        device_endpoint_limit: 5,
        authorize_endpoint_limit: 20,
        revoke_endpoint_limit: 10,
        register_endpoint_limit: 3, // Lower limit for testing
        login_endpoint_limit: 10,
        internal_service_secret: None,
    };

    let app = test::init_service(
        App::new()
            .wrap(RateLimitMiddleware::new(
                redis_client.clone(),
                rate_limit_config,
            ))
            .app_data(state)
            .route("/api/v1/auth/register", web::post().to(register)),
    )
    .await;

    let client_id = format!("test-client-{}", uuid::Uuid::new_v4());
    let mut emails = Vec::new();

    // First 3 registrations should succeed
    for i in 1..=3 {
        let email = format!("rate_limit_{}@example.com", uuid::Uuid::new_v4());
        emails.push(email.clone());

        let register_request = CreateUserRequest {
            email: email.clone(),
            password: "SecurePass123".to_string(),
            display_name: format!("Test User {}", i),
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .insert_header(("X-Client-ID", client_id.clone()))
            .set_json(&register_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201, "Request {} should succeed", i);
    }

    // 4th registration should be rate limited
    let email = format!("rate_limit_{}@example.com", uuid::Uuid::new_v4());
    let register_request = CreateUserRequest {
        email: email.clone(),
        password: "SecurePass123".to_string(),
        display_name: "Test User 4".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .insert_header(("X-Client-ID", client_id.clone()))
        .set_json(&register_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 429, "4th request should be rate limited");

    // Check rate limit headers
    assert!(resp.headers().get("Retry-After").is_some());
    assert!(resp.headers().get("X-RateLimit-Limit").is_some());

    // Cleanup
    for email in emails {
        sqlx::query("DELETE FROM users WHERE email = $1")
            .bind(&email)
            .execute(&pool)
            .await
            .ok();
    }
}

#[actix_web::test]
#[ignore] // Requires database
async fn test_password_hashing_uniqueness() {
    let pool = setup_test_db().await;
    let jwt_manager = setup_test_jwt_manager();
    let password_hasher = Arc::new(PasswordHasher::new());
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));

    let password = "SecurePass123";
    let email1 = format!("hash_test_1_{}@example.com", uuid::Uuid::new_v4());
    let email2 = format!("hash_test_2_{}@example.com", uuid::Uuid::new_v4());

    // Create two users with the same password
    let hash1 = password_hasher.hash_password(password).unwrap();
    let hash2 = password_hasher.hash_password(password).unwrap();

    // Hashes should be different (different salts)
    assert_ne!(hash1, hash2, "Password hashes should be unique");

    // Both should verify correctly
    assert!(password_hasher.verify_password(password, &hash1).unwrap());
    assert!(password_hasher.verify_password(password, &hash2).unwrap());

    // Store in database
    let user1 = user_repo
        .create_user(&email1, &hash1, "User 1")
        .await
        .expect("Failed to create user 1");

    let user2 = user_repo
        .create_user(&email2, &hash2, "User 2")
        .await
        .expect("Failed to create user 2");

    assert_ne!(user1.password_hash, user2.password_hash);

    // Cleanup
    sqlx::query("DELETE FROM users WHERE email IN ($1, $2)")
        .bind(&email1)
        .bind(&email2)
        .execute(&pool)
        .await
        .ok();
}
