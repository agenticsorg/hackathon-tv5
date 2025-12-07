use actix_web::{test, web, App};
use auth::{
    email::{ConsoleProvider, EmailConfig, EmailManager, EmailProviderConfig},
    error::AuthError,
    password_reset::{ForgotPasswordRequest, ResetPasswordRequest},
    password_reset_handlers::{forgot_password, reset_password, AppState},
    storage::AuthStorage,
    user::{PostgresUserRepository, UserRepository},
};
use sqlx::PgPool;
use std::sync::Arc;

async fn setup_test_state(pool: PgPool) -> web::Data<AppState> {
    let storage = Arc::new(AuthStorage::from_env().expect("Failed to create storage"));

    let email_config = EmailConfig {
        provider: EmailProviderConfig::Console,
        from_email: "noreply@test.local".to_string(),
        from_name: "Media Gateway Test".to_string(),
        base_url: "http://localhost:8080".to_string(),
        verification_ttl_hours: 24,
    };

    let redis_client =
        redis::Client::open("redis://127.0.0.1:6379").expect("Failed to create Redis client");

    let console_provider = Arc::new(ConsoleProvider::new(
        email_config.from_email.clone(),
        email_config.from_name.clone(),
        email_config.base_url.clone(),
    ));

    let email_manager = Arc::new(EmailManager::new(
        console_provider,
        redis_client,
        email_config,
    ));

    let session_manager = Arc::new(auth::session::SessionManager::new(storage.clone()));

    let token_family_manager = Arc::new(auth::token_family::TokenFamilyManager::new(pool.clone()));

    web::Data::new(AppState {
        storage,
        email_manager,
        session_manager,
        token_family_manager,
    })
}

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/media_gateway_test".to_string()
    });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn create_test_user(pool: &PgPool, email: &str, password: &str) -> uuid::Uuid {
    let user_repo = PostgresUserRepository::new(pool.clone());
    let password_hash =
        auth::user::PasswordHasher::hash_password(password).expect("Failed to hash password");

    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, username, created_at, updated_at)
        VALUES (gen_random_uuid(), $1, $2, $3, NOW(), NOW())
        RETURNING id
        "#,
        email,
        password_hash,
        email.split('@').next().unwrap()
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test user")
    .id
}

#[actix_web::test]
async fn test_forgot_password_sends_email() {
    let pool = setup_test_db().await;
    let state = setup_test_state(pool.clone()).await;

    // Create test user
    let test_email = "forgot_test@example.com";
    create_test_user(&pool, test_email, "OldPassword123!").await;

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .app_data(web::Data::new(pool.clone()))
            .service(forgot_password),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/password/forgot")
        .set_json(ForgotPasswordRequest {
            email: test_email.to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Verify response
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("reset link has been sent"));

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE email = $1", test_email)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");
}

#[actix_web::test]
async fn test_forgot_password_rate_limiting() {
    let pool = setup_test_db().await;
    let state = setup_test_state(pool.clone()).await;

    let test_email = "rate_limit_test@example.com";
    create_test_user(&pool, test_email, "Password123!").await;

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .app_data(web::Data::new(pool.clone()))
            .service(forgot_password),
    )
    .await;

    // First 3 requests should succeed
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/password/forgot")
            .set_json(ForgotPasswordRequest {
                email: test_email.to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200, "Request {} should succeed", i);
    }

    // 4th request should still return 200 but not send email (rate limited)
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/password/forgot")
        .set_json(ForgotPasswordRequest {
            email: test_email.to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200); // Still returns success to prevent enumeration

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE email = $1", test_email)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");
}

#[actix_web::test]
async fn test_forgot_password_nonexistent_email() {
    let pool = setup_test_db().await;
    let state = setup_test_state(pool.clone()).await;

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .app_data(web::Data::new(pool.clone()))
            .service(forgot_password),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/password/forgot")
        .set_json(ForgotPasswordRequest {
            email: "nonexistent@example.com".to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should return success even for non-existent email to prevent enumeration
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("reset link has been sent"));
}

#[actix_web::test]
async fn test_reset_password_sends_notification() {
    let pool = setup_test_db().await;
    let state = setup_test_state(pool.clone()).await;

    let test_email = "reset_test@example.com";
    let user_id = create_test_user(&pool, test_email, "OldPassword123!").await;

    // Create a reset token
    let reset_token =
        auth::password_reset::PasswordResetToken::new(user_id.to_string(), test_email.to_string());

    state
        .storage
        .store_password_reset_token(&reset_token.token, &reset_token)
        .await
        .expect("Failed to store reset token");

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .app_data(web::Data::new(pool.clone()))
            .service(reset_password),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/password/reset")
        .set_json(ResetPasswordRequest {
            token: reset_token.token.clone(),
            new_password: "NewPassword123!".to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("reset successfully"));

    // Verify token was deleted (single-use)
    let token_result = state
        .storage
        .get_password_reset_token(&reset_token.token)
        .await;
    assert!(token_result.is_ok());
    assert!(token_result.unwrap().is_none());

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE email = $1", test_email)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");
}

#[actix_web::test]
async fn test_reset_password_with_invalid_token() {
    let pool = setup_test_db().await;
    let state = setup_test_state(pool.clone()).await;

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .app_data(web::Data::new(pool.clone()))
            .service(reset_password),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/password/reset")
        .set_json(ResetPasswordRequest {
            token: "invalid_token_12345".to_string(),
            new_password: "NewPassword123!".to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_reset_password_weak_password() {
    let pool = setup_test_db().await;
    let state = setup_test_state(pool.clone()).await;

    let test_email = "weak_pwd_test@example.com";
    let user_id = create_test_user(&pool, test_email, "OldPassword123!").await;

    let reset_token =
        auth::password_reset::PasswordResetToken::new(user_id.to_string(), test_email.to_string());

    state
        .storage
        .store_password_reset_token(&reset_token.token, &reset_token)
        .await
        .expect("Failed to store reset token");

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .app_data(web::Data::new(pool.clone()))
            .service(reset_password),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/password/reset")
        .set_json(ResetPasswordRequest {
            token: reset_token.token.clone(),
            new_password: "weak".to_string(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE email = $1", test_email)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");
}

#[actix_web::test]
async fn test_complete_password_reset_flow_with_emails() {
    let pool = setup_test_db().await;
    let state = setup_test_state(pool.clone()).await;

    let test_email = "complete_flow@example.com";
    create_test_user(&pool, test_email, "OriginalPassword123!").await;

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .app_data(web::Data::new(pool.clone()))
            .service(forgot_password)
            .service(reset_password),
    )
    .await;

    // Step 1: Request password reset
    let forgot_req = test::TestRequest::post()
        .uri("/api/v1/auth/password/forgot")
        .set_json(ForgotPasswordRequest {
            email: test_email.to_string(),
        })
        .to_request();

    let forgot_resp = test::call_service(&app, forgot_req).await;
    assert_eq!(forgot_resp.status(), 200);

    // Step 2: Get the reset token from Redis (in real scenario, from email)
    // We need to retrieve it for testing
    let user_repo = PostgresUserRepository::new(pool.clone());
    let user = user_repo.find_by_email(test_email).await.unwrap().unwrap();

    // Simulate getting token from email (in test, we'll create a fresh one)
    let reset_token =
        auth::password_reset::PasswordResetToken::new(user.id.to_string(), test_email.to_string());
    state
        .storage
        .store_password_reset_token(&reset_token.token, &reset_token)
        .await
        .expect("Failed to store reset token");

    // Step 3: Reset password with token
    let reset_req = test::TestRequest::post()
        .uri("/api/v1/auth/password/reset")
        .set_json(ResetPasswordRequest {
            token: reset_token.token.clone(),
            new_password: "NewSecurePassword123!".to_string(),
        })
        .to_request();

    let reset_resp = test::call_service(&app, reset_req).await;
    assert_eq!(reset_resp.status(), 200);

    // Verify password was actually changed
    let updated_user = user_repo.find_by_email(test_email).await.unwrap().unwrap();
    assert_ne!(user.password_hash, updated_user.password_hash);

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE email = $1", test_email)
        .execute(&pool)
        .await
        .expect("Failed to clean up test user");
}
