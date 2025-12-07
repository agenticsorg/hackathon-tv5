use auth::{
    ConsoleProvider, CreateUserRequest, EmailConfig, EmailManager, PostgresUserRepository,
    UserRepository,
};
use redis::Client as RedisClient;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:password@localhost:5432/media_gateway_test".to_string()
    });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

async fn setup_test_redis() -> RedisClient {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    RedisClient::open(redis_url).expect("Failed to connect to Redis")
}

async fn setup_email_manager() -> EmailManager {
    let redis = setup_test_redis();
    let config = EmailConfig {
        provider: auth::email::EmailProviderConfig::Console,
        from_email: "test@example.com".to_string(),
        from_name: "Test App".to_string(),
        base_url: "http://localhost:8080".to_string(),
        verification_ttl_hours: 24,
    };

    let provider = Arc::new(ConsoleProvider::new(
        config.base_url.clone(),
        config.from_name.clone(),
    ));

    EmailManager::new(provider, redis, config)
}

#[tokio::test]
async fn test_email_verification_flow() {
    let pool = setup_test_db().await;
    let user_repo = PostgresUserRepository::new(pool.clone());
    let email_manager = setup_email_manager().await;

    // Create user with unverified email
    let create_req = CreateUserRequest {
        email: format!("test_{}@example.com", Uuid::new_v4()),
        password: "test_password_123".to_string(),
        username: Some("testuser".to_string()),
        email_verified: false,
    };

    let user = user_repo.create_user(create_req).await.unwrap();
    assert!(!user.email_verified);

    // Send verification email
    let token = email_manager
        .send_verification_email(user.id.to_string(), user.email.clone())
        .await
        .unwrap();

    assert_eq!(token.user_id, user.id.to_string());
    assert_eq!(token.email, user.email);

    // Verify token
    let verified = email_manager.verify_token(&token.token).await.unwrap();
    assert_eq!(verified.user_id, user.id.to_string());
    assert_eq!(verified.email, user.email);

    // Mark email as verified
    user_repo.mark_email_verified(user.id).await.unwrap();

    // Verify user is now verified
    let updated_user = user_repo
        .get_user_by_email(&user.email)
        .await
        .unwrap()
        .unwrap();
    assert!(updated_user.email_verified);
}

#[tokio::test]
async fn test_verification_token_expires_after_use() {
    let email_manager = setup_email_manager().await;

    // Create verification token
    let token = email_manager
        .create_verification_token("user123".to_string(), "test@example.com".to_string())
        .await
        .unwrap();

    // First verification succeeds
    let verified = email_manager.verify_token(&token.token).await.unwrap();
    assert_eq!(verified.user_id, "user123");

    // Second verification fails (token deleted)
    let result = email_manager.verify_token(&token.token).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_resend_verification_rate_limit() {
    let email_manager = setup_email_manager().await;
    let email = format!("ratelimit_{}@example.com", Uuid::new_v4());

    // First send succeeds
    let result1 = email_manager
        .send_verification_email("user123".to_string(), email.clone())
        .await;
    assert!(result1.is_ok());

    // Second send within 60 seconds fails
    let result2 = email_manager
        .send_verification_email("user123".to_string(), email.clone())
        .await;
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_invalid_verification_token() {
    let email_manager = setup_email_manager().await;

    let result = email_manager.verify_token("invalid_token_123").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_user_creation_with_verification_flag() {
    let pool = setup_test_db().await;
    let user_repo = PostgresUserRepository::new(pool.clone());

    // Create unverified user
    let unverified_req = CreateUserRequest {
        email: format!("unverified_{}@example.com", Uuid::new_v4()),
        password: "password123".to_string(),
        username: Some("unverified".to_string()),
        email_verified: false,
    };

    let unverified_user = user_repo.create_user(unverified_req).await.unwrap();
    assert!(!unverified_user.email_verified);

    // Create pre-verified user (e.g., admin created)
    let verified_req = CreateUserRequest {
        email: format!("verified_{}@example.com", Uuid::new_v4()),
        password: "password123".to_string(),
        username: Some("verified".to_string()),
        email_verified: true,
    };

    let verified_user = user_repo.create_user(verified_req).await.unwrap();
    assert!(verified_user.email_verified);
}

#[tokio::test]
async fn test_password_verification() {
    let pool = setup_test_db().await;
    let user_repo = PostgresUserRepository::new(pool.clone());

    let password = "secure_password_123";
    let create_req = CreateUserRequest {
        email: format!("pwtest_{}@example.com", Uuid::new_v4()),
        password: password.to_string(),
        username: Some("pwtest".to_string()),
        email_verified: true,
    };

    let user = user_repo.create_user(create_req).await.unwrap();

    // Correct password succeeds
    let verified_user = user_repo
        .verify_password(&user.email, password)
        .await
        .unwrap();
    assert_eq!(verified_user.id, user.id);

    // Wrong password fails
    let result = user_repo
        .verify_password(&user.email, "wrong_password")
        .await;
    assert!(result.is_err());

    // Non-existent user fails
    let result = user_repo
        .verify_password("nonexistent@example.com", password)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mark_email_verified_updates_database() {
    let pool = setup_test_db().await;
    let user_repo = PostgresUserRepository::new(pool.clone());

    // Create unverified user
    let create_req = CreateUserRequest {
        email: format!("markverified_{}@example.com", Uuid::new_v4()),
        password: "password123".to_string(),
        username: Some("markverified".to_string()),
        email_verified: false,
    };

    let user = user_repo.create_user(create_req).await.unwrap();
    assert!(!user.email_verified);

    // Mark as verified
    user_repo.mark_email_verified(user.id).await.unwrap();

    // Fetch again and verify
    let updated_user = user_repo.find_by_id(user.id).await.unwrap().unwrap();
    assert!(updated_user.email_verified);
}
