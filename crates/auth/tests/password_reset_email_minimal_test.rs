use auth::{
    email::{ConsoleProvider, EmailConfig, EmailManager, EmailProviderConfig},
    storage::AuthStorage,
};
use std::sync::Arc;

#[tokio::test]
async fn test_password_reset_email_service_integration() {
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

    let email_manager = EmailManager::new(console_provider, redis_client, email_config);

    // Test sending password reset email
    let result = email_manager
        .send_password_reset_email(
            "test@example.com".to_string(),
            "test_reset_token_123".to_string(),
        )
        .await;

    assert!(
        result.is_ok(),
        "Password reset email should send successfully"
    );
}

#[tokio::test]
async fn test_password_changed_notification_service() {
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

    let email_manager = EmailManager::new(console_provider, redis_client, email_config);

    // Test sending password changed notification
    let result = email_manager
        .send_password_changed_notification("test@example.com".to_string())
        .await;

    assert!(
        result.is_ok(),
        "Password changed notification should send successfully"
    );
}

#[tokio::test]
async fn test_password_reset_rate_limiting() {
    let storage = AuthStorage::from_env().expect("Failed to create storage");

    let test_email = "rate_limit_test@example.com";

    // First 3 attempts should succeed
    for i in 1..=3 {
        let remaining = storage
            .check_password_reset_rate_limit(test_email)
            .await
            .expect("Rate limit check should succeed");
        assert!(remaining > 0, "Attempt {} should not be rate limited", i);
    }

    // 4th attempt should be rate limited
    let remaining = storage
        .check_password_reset_rate_limit(test_email)
        .await
        .expect("Rate limit check should succeed");
    assert_eq!(remaining, 0, "4th attempt should be rate limited");
}
