use auth::{
    email::{ConsoleProvider, EmailConfig, EmailManager, EmailService},
    password_reset::{ForgotPasswordRequest, PasswordResetToken, ResetPasswordRequest},
    storage::AuthStorage,
    user::{CreateUserRequest, PasswordHasher, PostgresUserRepository, User, UserRepository},
    AuthError,
};
use sqlx::PgPool;
use std::sync::Arc;

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/media_gateway_test".to_string()
    });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn setup_test_redis() -> redis::Client {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    redis::Client::open(redis_url).expect("Failed to connect to Redis")
}

async fn create_test_user(pool: &PgPool) -> User {
    let user_repo = PostgresUserRepository::new(pool.clone());
    let password_hash = PasswordHasher::hash_password("OldPassword123").unwrap();

    user_repo
        .create_user("test@example.com", &password_hash, "Test User")
        .await
        .expect("Failed to create test user")
}

async fn cleanup_test_user(pool: &PgPool, email: &str) {
    sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(email)
        .execute(pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_password_reset_token_generation() {
    let token = PasswordResetToken::new("user123".to_string(), "test@example.com".to_string());

    assert_eq!(token.user_id, "user123");
    assert_eq!(token.email, "test@example.com");
    assert_eq!(token.token.len(), 64); // 32 bytes = 64 hex chars
    assert!(token.expires_at > token.created_at);
    assert_eq!(token.expires_at - token.created_at, 3600); // 1 hour
    assert!(!token.is_expired());
}

#[tokio::test]
async fn test_password_reset_token_expiration() {
    let mut token = PasswordResetToken::new("user123".to_string(), "test@example.com".to_string());

    // Manually set expiration to past
    token.expires_at = token.created_at - 1;
    assert!(token.is_expired());
}

#[tokio::test]
async fn test_store_and_retrieve_password_reset_token() {
    let redis_client = setup_test_redis().await;
    let storage = AuthStorage::new(&redis_client.get_connection_info().addr.to_string())
        .expect("Failed to create storage");

    let reset_token =
        PasswordResetToken::new("user123".to_string(), "test@example.com".to_string());

    // Store token
    storage
        .store_password_reset_token(&reset_token.token, &reset_token)
        .await
        .expect("Failed to store token");

    // Retrieve token
    let retrieved = storage
        .get_password_reset_token(&reset_token.token)
        .await
        .expect("Failed to retrieve token")
        .expect("Token not found");

    assert_eq!(retrieved.token, reset_token.token);
    assert_eq!(retrieved.user_id, reset_token.user_id);
    assert_eq!(retrieved.email, reset_token.email);

    // Cleanup
    storage
        .delete_password_reset_token(&reset_token.token)
        .await
        .ok();
}

#[tokio::test]
async fn test_password_reset_token_single_use() {
    let redis_client = setup_test_redis().await;
    let storage = AuthStorage::new(&redis_client.get_connection_info().addr.to_string())
        .expect("Failed to create storage");

    let reset_token =
        PasswordResetToken::new("user123".to_string(), "test@example.com".to_string());

    // Store token
    storage
        .store_password_reset_token(&reset_token.token, &reset_token)
        .await
        .expect("Failed to store token");

    // Delete token (simulating use)
    storage
        .delete_password_reset_token(&reset_token.token)
        .await
        .expect("Failed to delete token");

    // Try to retrieve again - should be None
    let retrieved = storage
        .get_password_reset_token(&reset_token.token)
        .await
        .expect("Failed to retrieve token");

    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_password_reset_rate_limiting() {
    let redis_client = setup_test_redis().await;
    let storage = AuthStorage::new(&redis_client.get_connection_info().addr.to_string())
        .expect("Failed to create storage");

    let email = format!("test-rate-limit-{}@example.com", uuid::Uuid::new_v4());

    // First 3 requests should succeed
    for i in 0..3 {
        let remaining = storage
            .check_password_reset_rate_limit(&email)
            .await
            .expect("Failed to check rate limit");
        assert_eq!(
            remaining,
            3 - i - 1,
            "Expected {} remaining attempts",
            3 - i - 1
        );
    }

    // 4th request should be rate limited
    let remaining = storage
        .check_password_reset_rate_limit(&email)
        .await
        .expect("Failed to check rate limit");
    assert_eq!(remaining, 0, "Should be rate limited");
}

#[tokio::test]
async fn test_update_user_password() {
    let pool = setup_test_db().await;
    let user = create_test_user(&pool).await;
    let user_repo = PostgresUserRepository::new(pool.clone());

    let new_password = "NewPassword456";
    let new_password_hash = PasswordHasher::hash_password(new_password).unwrap();

    // Update password
    user_repo
        .update_password(user.id, &new_password_hash)
        .await
        .expect("Failed to update password");

    // Verify new password works
    let updated_user = user_repo
        .find_by_id(user.id)
        .await
        .expect("Failed to find user")
        .expect("User not found");

    assert!(PasswordHasher::verify_password(new_password, &updated_user.password_hash).unwrap());
    assert!(
        !PasswordHasher::verify_password("OldPassword123", &updated_user.password_hash).unwrap()
    );

    // Cleanup
    cleanup_test_user(&pool, "test@example.com").await;
}

#[tokio::test]
async fn test_delete_all_user_sessions_on_password_reset() {
    let redis_client = setup_test_redis().await;
    let storage = AuthStorage::new(&redis_client.get_connection_info().addr.to_string())
        .expect("Failed to create storage");

    let user_id = uuid::Uuid::new_v4().to_string();

    // Create some fake session keys
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .unwrap();
    use redis::AsyncCommands;

    conn.set::<_, _, ()>(format!("session:user:{}:session1", user_id), "data1", None)
        .await
        .unwrap();
    conn.set::<_, _, ()>(format!("session:user:{}:session2", user_id), "data2", None)
        .await
        .unwrap();
    conn.set::<_, _, ()>(format!("session:user:{}:session3", user_id), "data3", None)
        .await
        .unwrap();

    // Delete all user sessions
    storage
        .delete_user_sessions(&user_id)
        .await
        .expect("Failed to delete sessions");

    // Verify sessions are deleted
    let result1: Option<String> = conn
        .get(format!("session:user:{}:session1", user_id))
        .await
        .unwrap();
    let result2: Option<String> = conn
        .get(format!("session:user:{}:session2", user_id))
        .await
        .unwrap();
    let result3: Option<String> = conn
        .get(format!("session:user:{}:session3", user_id))
        .await
        .unwrap();

    assert!(result1.is_none());
    assert!(result2.is_none());
    assert!(result3.is_none());
}

#[tokio::test]
async fn test_full_password_reset_flow() {
    let pool = setup_test_db().await;
    let redis_client = setup_test_redis().await;
    let user = create_test_user(&pool).await;

    let storage = AuthStorage::new(&redis_client.get_connection_info().addr.to_string())
        .expect("Failed to create storage");
    let user_repo = PostgresUserRepository::new(pool.clone());

    // Step 1: Generate reset token
    let reset_token = PasswordResetToken::new(user.id.to_string(), user.email.clone());

    // Step 2: Store token in Redis
    storage
        .store_password_reset_token(&reset_token.token, &reset_token)
        .await
        .expect("Failed to store token");

    // Step 3: Verify token exists and is not expired
    let retrieved_token = storage
        .get_password_reset_token(&reset_token.token)
        .await
        .expect("Failed to retrieve token")
        .expect("Token not found");
    assert!(!retrieved_token.is_expired());

    // Step 4: Update password
    let new_password = "NewSecurePassword123";
    let new_password_hash = PasswordHasher::hash_password(new_password).unwrap();

    user_repo
        .update_password(user.id, &new_password_hash)
        .await
        .expect("Failed to update password");

    // Step 5: Delete token (single-use)
    storage
        .delete_password_reset_token(&reset_token.token)
        .await
        .expect("Failed to delete token");

    // Step 6: Verify token is deleted
    let deleted_token = storage
        .get_password_reset_token(&reset_token.token)
        .await
        .expect("Failed to retrieve token");
    assert!(deleted_token.is_none());

    // Step 7: Verify new password works
    let updated_user = user_repo
        .find_by_id(user.id)
        .await
        .expect("Failed to find user")
        .expect("User not found");

    assert!(PasswordHasher::verify_password(new_password, &updated_user.password_hash).unwrap());

    // Cleanup
    cleanup_test_user(&pool, "test@example.com").await;
}

#[tokio::test]
async fn test_password_reset_with_invalid_token() {
    let redis_client = setup_test_redis().await;
    let storage = AuthStorage::new(&redis_client.get_connection_info().addr.to_string())
        .expect("Failed to create storage");

    // Try to retrieve non-existent token
    let result = storage
        .get_password_reset_token("invalid_token_12345")
        .await
        .expect("Failed to retrieve token");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_email_manager_send_password_reset() {
    let redis_client = setup_test_redis().await;
    let email_config = EmailConfig::default();
    let provider = Arc::new(ConsoleProvider::new(
        email_config.base_url.clone(),
        email_config.from_name.clone(),
    ));

    let email_manager = EmailManager::new(provider, redis_client, email_config);

    let result = email_manager
        .send_password_reset_email("test@example.com".to_string(), "test_token_123".to_string())
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_email_manager_send_password_changed() {
    let redis_client = setup_test_redis().await;
    let email_config = EmailConfig::default();
    let provider = Arc::new(ConsoleProvider::new(
        email_config.base_url.clone(),
        email_config.from_name.clone(),
    ));

    let email_manager = EmailManager::new(provider, redis_client, email_config);

    let result = email_manager
        .send_password_changed_notification("test@example.com".to_string())
        .await;

    assert!(result.is_ok());
}
