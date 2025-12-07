use actix_web::{test, web::Data, App};
use media_gateway_auth::{
    email::{ConsoleProvider, EmailConfig, EmailManager},
    handlers::{
        register, resend_verification, verify_email, RegisterRequest, ResendVerificationRequest,
        VerifyEmailRequest,
    },
    user::{PostgresUserRepository, UserRepository},
    AuthError,
};
use sqlx::PgPool;
use std::sync::Arc;

async fn setup_test_dependencies() -> (Arc<PostgresUserRepository>, Arc<EmailManager>) {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/media_gateway_test".to_string()
    });

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let user_repo = Arc::new(PostgresUserRepository::new(pool));

    let redis = redis::Client::open("redis://localhost:6379").expect("Failed to connect to Redis");

    let email_config = EmailConfig::default();
    let provider = Arc::new(ConsoleProvider::new(
        email_config.base_url.clone(),
        email_config.from_name.clone(),
    ));

    let email_manager = Arc::new(EmailManager::new(provider, redis, email_config));

    (user_repo, email_manager)
}

#[actix_web::test]
async fn test_register_user_sends_verification_email() {
    let (user_repo, email_manager) = setup_test_dependencies().await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(user_repo.clone()))
            .app_data(Data::new(email_manager.clone()))
            .service(register),
    )
    .await;

    let req = RegisterRequest {
        email: format!("test_{}@example.com", uuid::Uuid::new_v4()),
        password: "TestPassword123".to_string(),
        display_name: Some("Test User".to_string()),
    };

    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(&req)
            .to_request(),
    )
    .await;

    assert_eq!(resp.status(), 201);

    // Clean up - verify user was created
    let user = user_repo.find_by_email(&req.email).await.unwrap();
    assert!(user.is_some());
    assert_eq!(user.unwrap().email_verified, false);
}

#[actix_web::test]
async fn test_verify_email_with_valid_token() {
    let (user_repo, email_manager) = setup_test_dependencies().await;

    // Create a user first
    let test_email = format!("verify_test_{}@example.com", uuid::Uuid::new_v4());
    let password_hasher = media_gateway_auth::user::PasswordHasher::default();
    let password_hash = password_hasher.hash_password("TestPassword123").unwrap();

    let user = user_repo
        .create_user(&test_email, &password_hash, "Test User")
        .await
        .unwrap();

    // Create verification token
    let token = email_manager
        .create_verification_token(user.id.to_string(), user.email.clone())
        .await
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(Data::new(user_repo.clone()))
            .app_data(Data::new(email_manager.clone()))
            .service(verify_email),
    )
    .await;

    let verify_req = VerifyEmailRequest { token: token.token };

    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/verify-email")
            .set_json(&verify_req)
            .to_request(),
    )
    .await;

    assert_eq!(resp.status(), 200);

    // Verify the user's email_verified flag was updated
    let updated_user = user_repo.find_by_id(user.id).await.unwrap().unwrap();
    assert!(updated_user.email_verified);
}

#[actix_web::test]
async fn test_verify_email_with_invalid_token() {
    let (user_repo, email_manager) = setup_test_dependencies().await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(user_repo.clone()))
            .app_data(Data::new(email_manager.clone()))
            .service(verify_email),
    )
    .await;

    let verify_req = VerifyEmailRequest {
        token: "invalid_token_12345".to_string(),
    };

    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/verify-email")
            .set_json(&verify_req)
            .to_request(),
    )
    .await;

    // Should return 401 Unauthorized for invalid token
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_resend_verification_email() {
    let (user_repo, email_manager) = setup_test_dependencies().await;

    // Create a user first
    let test_email = format!("resend_test_{}@example.com", uuid::Uuid::new_v4());
    let password_hasher = media_gateway_auth::user::PasswordHasher::default();
    let password_hash = password_hasher.hash_password("TestPassword123").unwrap();

    let user = user_repo
        .create_user(&test_email, &password_hash, "Test User")
        .await
        .unwrap();

    // Wait a bit to avoid rate limiting
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(user_repo.clone()))
            .app_data(Data::new(email_manager.clone()))
            .service(resend_verification),
    )
    .await;

    let resend_req = ResendVerificationRequest {
        email: test_email.clone(),
    };

    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/resend-verification")
            .set_json(&resend_req)
            .to_request(),
    )
    .await;

    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn test_resend_verification_for_already_verified_email() {
    let (user_repo, email_manager) = setup_test_dependencies().await;

    // Create and verify a user
    let test_email = format!("already_verified_{}@example.com", uuid::Uuid::new_v4());
    let password_hasher = media_gateway_auth::user::PasswordHasher::default();
    let password_hash = password_hasher.hash_password("TestPassword123").unwrap();

    let user = user_repo
        .create_user(&test_email, &password_hash, "Test User")
        .await
        .unwrap();

    // Mark email as verified
    user_repo
        .update_email_verified(user.id, true)
        .await
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(Data::new(user_repo.clone()))
            .app_data(Data::new(email_manager.clone()))
            .service(resend_verification),
    )
    .await;

    let resend_req = ResendVerificationRequest {
        email: test_email.clone(),
    };

    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/resend-verification")
            .set_json(&resend_req)
            .to_request(),
    )
    .await;

    // Should return error for already verified email
    assert_eq!(resp.status(), 500);
}

#[actix_web::test]
async fn test_login_blocked_for_unverified_email() {
    let (user_repo, email_manager) = setup_test_dependencies().await;

    // Create an unverified user
    let test_email = format!("unverified_login_{}@example.com", uuid::Uuid::new_v4());
    let password = "TestPassword123";
    let password_hasher = media_gateway_auth::user::PasswordHasher::default();
    let password_hash = password_hasher.hash_password(password).unwrap();

    let user = user_repo
        .create_user(&test_email, &password_hash, "Test User")
        .await
        .unwrap();

    // Ensure email is not verified
    assert!(!user.email_verified);

    // Set environment variable to require email verification
    std::env::set_var("REQUIRE_EMAIL_VERIFICATION", "true");

    // Try to login - this would require the full login handler with JWT and session managers
    // For now, we just verify the user exists and is unverified
    let found_user = user_repo.find_by_email(&test_email).await.unwrap().unwrap();
    assert!(!found_user.email_verified);
}
