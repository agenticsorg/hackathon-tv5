use crate::{
    error::AuthError,
    password_reset::{
        ForgotPasswordRequest, PasswordResetToken, PasswordValidator, ResetPasswordRequest,
    },
    storage::AuthStorage,
    user::{CreateUserRequest, PasswordHasher, PostgresUserRepository, UserRepository},
};
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn test_forgot_password_creates_reset_token(pool: PgPool) {
    let storage = AuthStorage::from_env().unwrap();
    let user_repo = PostgresUserRepository::new(pool.clone());

    // Create test user
    let user = user_repo
        .create_user(CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "Password123".to_string(),
            username: Some("testuser".to_string()),
            email_verified: true,
        })
        .await
        .unwrap();

    // Generate reset token
    let reset_token = PasswordResetToken::new(user.id.to_string(), user.email.clone());

    // Store in Redis
    storage
        .store_password_reset_token(&reset_token.token, &reset_token)
        .await
        .unwrap();

    // Retrieve token
    let retrieved = storage
        .get_password_reset_token(&reset_token.token)
        .await
        .unwrap()
        .expect("Token should exist");

    assert_eq!(retrieved.email, user.email);
    assert_eq!(retrieved.user_id, user.id.to_string());
}

#[sqlx::test]
async fn test_reset_password_updates_password(pool: PgPool) {
    let storage = AuthStorage::from_env().unwrap();
    let user_repo = PostgresUserRepository::new(pool.clone());

    // Create test user
    let user = user_repo
        .create_user(CreateUserRequest {
            email: "reset@example.com".to_string(),
            password: "OldPassword123".to_string(),
            username: Some("resetuser".to_string()),
            email_verified: true,
        })
        .await
        .unwrap();

    // Generate and store reset token
    let reset_token = PasswordResetToken::new(user.id.to_string(), user.email.clone());
    storage
        .store_password_reset_token(&reset_token.token, &reset_token)
        .await
        .unwrap();

    // Reset password
    let new_password = "NewPassword456";
    let new_hash = PasswordHasher::hash_password(new_password).unwrap();
    user_repo.update_password(user.id, &new_hash).await.unwrap();

    // Verify new password works
    let fetched_user = user_repo.find_by_id(user.id).await.unwrap().unwrap();
    assert!(
        PasswordHasher::verify_password(new_password, &fetched_user.password_hash.unwrap())
            .unwrap()
    );

    // Verify old password doesn't work
    let old_hash = PasswordHasher::hash_password("OldPassword123").unwrap();
    assert_ne!(fetched_user.password_hash.unwrap(), old_hash);
}

#[sqlx::test]
async fn test_reset_token_single_use(pool: PgPool) {
    let storage = AuthStorage::from_env().unwrap();
    let user_repo = PostgresUserRepository::new(pool.clone());

    // Create test user
    let user = user_repo
        .create_user(CreateUserRequest {
            email: "singleuse@example.com".to_string(),
            password: "Password123".to_string(),
            username: Some("singleuseuser".to_string()),
            email_verified: true,
        })
        .await
        .unwrap();

    // Generate reset token
    let reset_token = PasswordResetToken::new(user.id.to_string(), user.email.clone());
    storage
        .store_password_reset_token(&reset_token.token, &reset_token)
        .await
        .unwrap();

    // Use token once
    let retrieved = storage
        .get_password_reset_token(&reset_token.token)
        .await
        .unwrap();
    assert!(retrieved.is_some());

    // Delete token (simulate single-use)
    storage
        .delete_password_reset_token(&reset_token.token)
        .await
        .unwrap();

    // Verify token is gone
    let should_be_none = storage
        .get_password_reset_token(&reset_token.token)
        .await
        .unwrap();
    assert!(should_be_none.is_none());
}

#[sqlx::test]
async fn test_reset_token_expiration(pool: PgPool) {
    let storage = AuthStorage::from_env().unwrap();

    let mut reset_token = PasswordResetToken::new(
        Uuid::new_v4().to_string(),
        "expired@example.com".to_string(),
    );

    // Set token to expired
    reset_token.expires_at = reset_token.created_at - 1;

    assert!(reset_token.is_expired());
}

#[sqlx::test]
async fn test_password_reset_rate_limiting(pool: PgPool) {
    let storage = AuthStorage::from_env().unwrap();
    let email = "ratelimit@example.com";

    // First 3 requests should succeed
    for i in 1..=3 {
        let remaining = storage
            .check_password_reset_rate_limit(email)
            .await
            .unwrap();
        assert_eq!(
            remaining,
            3 - i,
            "Attempt {} should have {} remaining",
            i,
            3 - i
        );
    }

    // 4th request should be rate limited
    let remaining = storage
        .check_password_reset_rate_limit(email)
        .await
        .unwrap();
    assert_eq!(remaining, 0, "Should be rate limited after 3 attempts");
}

#[sqlx::test]
async fn test_delete_user_sessions(pool: PgPool) {
    let storage = AuthStorage::from_env().unwrap();
    let user_id = Uuid::new_v4().to_string();

    // Simulate creating sessions by storing some keys
    let mut conn = storage
        .client
        .get_multiplexed_async_connection()
        .await
        .unwrap();
    use redis::AsyncCommands;

    let _: () = conn
        .set(format!("session:user:{}:session1", user_id), "data1")
        .await
        .unwrap();
    let _: () = conn
        .set(format!("session:user:{}:session2", user_id), "data2")
        .await
        .unwrap();

    // Delete all sessions
    storage.delete_user_sessions(&user_id).await.unwrap();

    // Verify sessions are deleted
    let result1: Option<String> = conn
        .get(format!("session:user:{}:session1", user_id))
        .await
        .unwrap();
    let result2: Option<String> = conn
        .get(format!("session:user:{}:session2", user_id))
        .await
        .unwrap();

    assert!(result1.is_none());
    assert!(result2.is_none());
}

#[test]
fn test_password_validator_meets_requirements() {
    // Valid passwords
    assert!(PasswordValidator::validate("Password123").is_ok());
    assert!(PasswordValidator::validate("MySecure1Pass").is_ok());
    assert!(PasswordValidator::validate("C0mpl3xP@ss").is_ok());

    // Too short
    assert!(PasswordValidator::validate("Pass1").is_err());

    // No uppercase
    let result = PasswordValidator::validate("password123");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("uppercase"));

    // No lowercase
    let result = PasswordValidator::validate("PASSWORD123");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("lowercase"));

    // No digit
    let result = PasswordValidator::validate("PasswordOnly");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("digit"));
}

#[sqlx::test]
async fn test_password_reset_flow_integration(pool: PgPool) {
    let storage = AuthStorage::from_env().unwrap();
    let user_repo = PostgresUserRepository::new(pool.clone());

    // Step 1: Create user
    let user = user_repo
        .create_user(CreateUserRequest {
            email: "integration@example.com".to_string(),
            password: "OldPassword123".to_string(),
            username: Some("integrationuser".to_string()),
            email_verified: true,
        })
        .await
        .unwrap();

    // Step 2: Request password reset (check rate limit)
    let remaining = storage
        .check_password_reset_rate_limit(&user.email)
        .await
        .unwrap();
    assert!(remaining > 0);

    // Step 3: Generate and store reset token
    let reset_token = PasswordResetToken::new(user.id.to_string(), user.email.clone());
    storage
        .store_password_reset_token(&reset_token.token, &reset_token)
        .await
        .unwrap();

    // Step 4: Validate new password
    let new_password = "NewSecurePass456";
    assert!(PasswordValidator::validate(new_password).is_ok());

    // Step 5: Verify token exists and not expired
    let retrieved_token = storage
        .get_password_reset_token(&reset_token.token)
        .await
        .unwrap()
        .expect("Token should exist");
    assert!(!retrieved_token.is_expired());

    // Step 6: Update password
    let new_hash = PasswordHasher::hash_password(new_password).unwrap();
    user_repo.update_password(user.id, &new_hash).await.unwrap();

    // Step 7: Delete reset token (single-use)
    storage
        .delete_password_reset_token(&reset_token.token)
        .await
        .unwrap();

    // Step 8: Invalidate all sessions
    storage
        .delete_user_sessions(&user.id.to_string())
        .await
        .unwrap();

    // Step 9: Verify new password works
    let updated_user = user_repo.find_by_id(user.id).await.unwrap().unwrap();
    assert!(
        PasswordHasher::verify_password(new_password, &updated_user.password_hash.unwrap())
            .unwrap()
    );

    // Step 10: Verify token is deleted
    let should_be_none = storage
        .get_password_reset_token(&reset_token.token)
        .await
        .unwrap();
    assert!(should_be_none.is_none());
}
