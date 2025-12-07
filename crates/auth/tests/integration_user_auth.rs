use actix_web::{test, web, App};
use media_gateway_auth::{
    login, register, CreateUserRequest, JwtManager, LoginRequest, PasswordHasher,
    PostgresUserRepository, UserHandlerState,
};
use sqlx::PgPool;
use std::sync::Arc;

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:password@localhost:5432/media_gateway_test".to_string()
    });

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

async fn cleanup_test_user(pool: &PgPool, email: &str) {
    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(email)
        .execute(pool)
        .await;
}

fn create_test_jwt_manager() -> JwtManager {
    // Use test keys for JWT
    let private_key = include_bytes!("../../../tests/fixtures/test_private_key.pem");
    let public_key = include_bytes!("../../../tests/fixtures/test_public_key.pem");

    JwtManager::new(
        private_key,
        public_key,
        "https://test.mediagateway.io".to_string(),
        "test-users".to_string(),
    )
    .expect("Failed to create JWT manager")
}

#[actix_web::test]
async fn test_user_registration_success() {
    let pool = setup_test_db().await;
    let test_email = "test_register@example.com";

    cleanup_test_user(&pool, test_email).await;

    let jwt_manager = Arc::new(create_test_jwt_manager());
    let user_handler_state = web::Data::new(UserHandlerState {
        user_repository: Arc::new(PostgresUserRepository::new(pool.clone())),
        password_hasher: Arc::new(PasswordHasher::new()),
        jwt_manager,
        require_email_verification: false,
    });

    let app = test::init_service(
        App::new()
            .app_data(user_handler_state.clone())
            .service(register),
    )
    .await;

    let req_body = CreateUserRequest {
        email: test_email.to_string(),
        password: "TestPassword123".to_string(),
        display_name: "Test User".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user"]["email"], test_email);
    assert_eq!(body["user"]["display_name"], "Test User");
    assert_eq!(body["user"]["email_verified"], false);

    cleanup_test_user(&pool, test_email).await;
}

#[actix_web::test]
async fn test_user_registration_weak_password() {
    let pool = setup_test_db().await;
    let test_email = "test_weak_pw@example.com";

    cleanup_test_user(&pool, test_email).await;

    let jwt_manager = Arc::new(create_test_jwt_manager());
    let user_handler_state = web::Data::new(UserHandlerState {
        user_repository: Arc::new(PostgresUserRepository::new(pool.clone())),
        password_hasher: Arc::new(PasswordHasher::new()),
        jwt_manager,
        require_email_verification: false,
    });

    let app = test::init_service(
        App::new()
            .app_data(user_handler_state.clone())
            .service(register),
    )
    .await;

    let req_body = CreateUserRequest {
        email: test_email.to_string(),
        password: "weak".to_string(),
        display_name: "Test User".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 500);

    cleanup_test_user(&pool, test_email).await;
}

#[actix_web::test]
async fn test_user_registration_duplicate_email() {
    let pool = setup_test_db().await;
    let test_email = "test_duplicate@example.com";

    cleanup_test_user(&pool, test_email).await;

    let jwt_manager = Arc::new(create_test_jwt_manager());
    let user_handler_state = web::Data::new(UserHandlerState {
        user_repository: Arc::new(PostgresUserRepository::new(pool.clone())),
        password_hasher: Arc::new(PasswordHasher::new()),
        jwt_manager,
        require_email_verification: false,
    });

    let app = test::init_service(
        App::new()
            .app_data(user_handler_state.clone())
            .service(register),
    )
    .await;

    let req_body = CreateUserRequest {
        email: test_email.to_string(),
        password: "TestPassword123".to_string(),
        display_name: "Test User".to_string(),
    };

    // First registration
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&req_body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Duplicate registration
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&req_body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 500);

    cleanup_test_user(&pool, test_email).await;
}

#[actix_web::test]
async fn test_user_login_success() {
    let pool = setup_test_db().await;
    let test_email = "test_login@example.com";

    cleanup_test_user(&pool, test_email).await;

    let jwt_manager = Arc::new(create_test_jwt_manager());
    let user_handler_state = web::Data::new(UserHandlerState {
        user_repository: Arc::new(PostgresUserRepository::new(pool.clone())),
        password_hasher: Arc::new(PasswordHasher::new()),
        jwt_manager,
        require_email_verification: false,
    });

    let app = test::init_service(
        App::new()
            .app_data(user_handler_state.clone())
            .service(register)
            .service(login),
    )
    .await;

    // Register user first
    let register_body = CreateUserRequest {
        email: test_email.to_string(),
        password: "TestPassword123".to_string(),
        display_name: "Test User".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Now login
    let login_body = LoginRequest {
        email: test_email.to_string(),
        password: "TestPassword123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
    assert_eq!(body["token_type"], "Bearer");
    assert_eq!(body["expires_in"], 3600);

    cleanup_test_user(&pool, test_email).await;
}

#[actix_web::test]
async fn test_user_login_invalid_credentials() {
    let pool = setup_test_db().await;
    let test_email = "test_invalid_login@example.com";

    cleanup_test_user(&pool, test_email).await;

    let jwt_manager = Arc::new(create_test_jwt_manager());
    let user_handler_state = web::Data::new(UserHandlerState {
        user_repository: Arc::new(PostgresUserRepository::new(pool.clone())),
        password_hasher: Arc::new(PasswordHasher::new()),
        jwt_manager,
        require_email_verification: false,
    });

    let app = test::init_service(
        App::new()
            .app_data(user_handler_state.clone())
            .service(register)
            .service(login),
    )
    .await;

    // Register user first
    let register_body = CreateUserRequest {
        email: test_email.to_string(),
        password: "TestPassword123".to_string(),
        display_name: "Test User".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Login with wrong password
    let login_body = LoginRequest {
        email: test_email.to_string(),
        password: "WrongPassword123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    cleanup_test_user(&pool, test_email).await;
}

#[actix_web::test]
async fn test_user_login_nonexistent_user() {
    let pool = setup_test_db().await;

    let jwt_manager = Arc::new(create_test_jwt_manager());
    let user_handler_state = web::Data::new(UserHandlerState {
        user_repository: Arc::new(PostgresUserRepository::new(pool.clone())),
        password_hasher: Arc::new(PasswordHasher::new()),
        jwt_manager,
        require_email_verification: false,
    });

    let app = test::init_service(
        App::new()
            .app_data(user_handler_state.clone())
            .service(login),
    )
    .await;

    let login_body = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "TestPassword123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_user_login_email_verification_required() {
    let pool = setup_test_db().await;
    let test_email = "test_verification@example.com";

    cleanup_test_user(&pool, test_email).await;

    let jwt_manager = Arc::new(create_test_jwt_manager());
    let user_handler_state = web::Data::new(UserHandlerState {
        user_repository: Arc::new(PostgresUserRepository::new(pool.clone())),
        password_hasher: Arc::new(PasswordHasher::new()),
        jwt_manager,
        require_email_verification: true,
    });

    let app = test::init_service(
        App::new()
            .app_data(user_handler_state.clone())
            .service(register)
            .service(login),
    )
    .await;

    // Register user
    let register_body = CreateUserRequest {
        email: test_email.to_string(),
        password: "TestPassword123".to_string(),
        display_name: "Test User".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Try to login without verification
    let login_body = LoginRequest {
        email: test_email.to_string(),
        password: "TestPassword123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 500);

    cleanup_test_user(&pool, test_email).await;
}
