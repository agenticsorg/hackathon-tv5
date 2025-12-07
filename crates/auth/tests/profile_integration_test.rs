use media_gateway_auth::{
    profile::{storage::ProfileStorage, types::UpdateProfileRequest},
    AuthError,
};
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/test_auth".to_string());

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

async fn create_test_user(pool: &PgPool, email: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, display_name)
        VALUES ($1, $2, $3, $4)
        "#,
        user_id,
        email,
        "hashed_password",
        "Test User"
    )
    .execute(pool)
    .await
    .expect("Failed to create test user");

    user_id
}

async fn cleanup_user(pool: &PgPool, user_id: Uuid) {
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_get_user_profile_returns_profile_with_oauth_providers() {
    let pool = setup_test_db().await;
    let storage = ProfileStorage::new(pool.clone());
    let user_id = create_test_user(&pool, "get_profile@example.com").await;

    // Add OAuth provider
    sqlx::query!(
        r#"
        INSERT INTO oauth_providers (user_id, provider, provider_user_id)
        VALUES ($1, $2, $3)
        "#,
        user_id,
        "google",
        "google_123"
    )
    .execute(&pool)
    .await
    .expect("Failed to add OAuth provider");

    let profile = storage
        .get_user_profile(user_id)
        .await
        .expect("Failed to get profile")
        .expect("Profile not found");

    assert_eq!(profile.id, user_id);
    assert_eq!(profile.email, "get_profile@example.com");
    assert_eq!(profile.display_name, "Test User");
    assert!(profile.oauth_providers.contains(&"google".to_string()));
    assert_eq!(profile.preferences, serde_json::json!({}));

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_update_user_profile_updates_all_fields() {
    let pool = setup_test_db().await;
    let storage = ProfileStorage::new(pool.clone());
    let user_id = create_test_user(&pool, "update_all@example.com").await;

    let update_request = UpdateProfileRequest {
        display_name: Some("Updated Name".to_string()),
        avatar_url: Some("https://example.com/avatar.jpg".to_string()),
        preferences: Some(serde_json::json!({"theme": "dark", "language": "en"})),
    };

    let updated_profile = storage
        .update_user_profile(user_id, &update_request)
        .await
        .expect("Failed to update profile");

    assert_eq!(updated_profile.display_name, "Updated Name");
    assert_eq!(
        updated_profile.avatar_url,
        Some("https://example.com/avatar.jpg".to_string())
    );
    assert_eq!(
        updated_profile.preferences,
        serde_json::json!({"theme": "dark", "language": "en"})
    );

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_update_user_profile_partial_update() {
    let pool = setup_test_db().await;
    let storage = ProfileStorage::new(pool.clone());
    let user_id = create_test_user(&pool, "partial_update@example.com").await;

    // First update
    let update_request = UpdateProfileRequest {
        display_name: Some("First Update".to_string()),
        avatar_url: Some("https://example.com/avatar1.jpg".to_string()),
        preferences: Some(serde_json::json!({"theme": "dark"})),
    };

    storage
        .update_user_profile(user_id, &update_request)
        .await
        .expect("Failed to update profile");

    // Partial update - only display name
    let partial_request = UpdateProfileRequest {
        display_name: Some("Second Update".to_string()),
        avatar_url: None,
        preferences: None,
    };

    let updated_profile = storage
        .update_user_profile(user_id, &partial_request)
        .await
        .expect("Failed to update profile");

    assert_eq!(updated_profile.display_name, "Second Update");
    assert_eq!(
        updated_profile.avatar_url,
        Some("https://example.com/avatar1.jpg".to_string())
    );
    assert_eq!(
        updated_profile.preferences,
        serde_json::json!({"theme": "dark"})
    );

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_soft_delete_user_marks_deleted_at() {
    let pool = setup_test_db().await;
    let storage = ProfileStorage::new(pool.clone());
    let user_id = create_test_user(&pool, "soft_delete@example.com").await;

    let deleted_at = storage
        .soft_delete_user(user_id)
        .await
        .expect("Failed to soft delete user");

    assert!(deleted_at <= chrono::Utc::now());

    // Verify user is soft deleted (profile returns None for deleted users)
    let profile = storage
        .get_user_profile(user_id)
        .await
        .expect("Failed to get profile");
    assert!(profile.is_none());

    // Verify deleted_at is set in database
    let row = sqlx::query!("SELECT deleted_at FROM users WHERE id = $1", user_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch user");

    assert!(row.deleted_at.is_some());

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_can_recover_account_within_30_days() {
    let pool = setup_test_db().await;
    let storage = ProfileStorage::new(pool.clone());
    let user_id = create_test_user(&pool, "recover@example.com").await;

    storage
        .soft_delete_user(user_id)
        .await
        .expect("Failed to soft delete user");

    let can_recover = storage
        .can_recover_account(user_id)
        .await
        .expect("Failed to check recovery");

    assert!(can_recover);

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_can_recover_account_after_30_days() {
    let pool = setup_test_db().await;
    let storage = ProfileStorage::new(pool.clone());
    let user_id = create_test_user(&pool, "no_recover@example.com").await;

    // Manually set deleted_at to 31 days ago
    let past_date = chrono::Utc::now() - chrono::Duration::days(31);
    sqlx::query!(
        "UPDATE users SET deleted_at = $1 WHERE id = $2",
        past_date.naive_utc(),
        user_id
    )
    .execute(&pool)
    .await
    .expect("Failed to set deleted_at");

    let can_recover = storage
        .can_recover_account(user_id)
        .await
        .expect("Failed to check recovery");

    assert!(!can_recover);

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_audit_log_created_on_profile_update() {
    let pool = setup_test_db().await;
    let storage = ProfileStorage::new(pool.clone());
    let user_id = create_test_user(&pool, "audit@example.com").await;

    let update_request = UpdateProfileRequest {
        display_name: Some("Audited Update".to_string()),
        avatar_url: None,
        preferences: None,
    };

    storage
        .update_user_profile(user_id, &update_request)
        .await
        .expect("Failed to update profile");

    let logs = storage
        .get_audit_logs(user_id, 10)
        .await
        .expect("Failed to get audit logs");

    assert!(!logs.is_empty());
    assert_eq!(logs[0].action, "profile.update");
    assert_eq!(logs[0].resource_type, "user");
    assert_eq!(logs[0].user_id, user_id);
    assert!(logs[0].old_values.is_some());
    assert!(logs[0].new_values.is_some());

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_audit_log_created_on_soft_delete() {
    let pool = setup_test_db().await;
    let storage = ProfileStorage::new(pool.clone());
    let user_id = create_test_user(&pool, "audit_delete@example.com").await;

    storage
        .soft_delete_user(user_id)
        .await
        .expect("Failed to soft delete user");

    let logs = storage
        .get_audit_logs(user_id, 10)
        .await
        .expect("Failed to get audit logs");

    assert!(!logs.is_empty());
    assert_eq!(logs[0].action, "account.soft_delete");
    assert_eq!(logs[0].resource_type, "user");
    assert!(logs[0].new_values.is_some());

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_update_avatar_url() {
    let pool = setup_test_db().await;
    let storage = ProfileStorage::new(pool.clone());
    let user_id = create_test_user(&pool, "avatar@example.com").await;

    let avatar_url = "https://example.com/new-avatar.png".to_string();
    storage
        .update_avatar_url(user_id, avatar_url.clone())
        .await
        .expect("Failed to update avatar URL");

    let profile = storage
        .get_user_profile(user_id)
        .await
        .expect("Failed to get profile")
        .expect("Profile not found");

    assert_eq!(profile.avatar_url, Some(avatar_url));

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_validate_update_request_empty_display_name() {
    let request = UpdateProfileRequest {
        display_name: Some("".to_string()),
        avatar_url: None,
        preferences: None,
    };

    assert!(request.validate().is_err());
}

#[tokio::test]
async fn test_validate_update_request_long_display_name() {
    let request = UpdateProfileRequest {
        display_name: Some("a".repeat(101)),
        avatar_url: None,
        preferences: None,
    };

    assert!(request.validate().is_err());
}

#[tokio::test]
async fn test_validate_update_request_long_avatar_url() {
    let request = UpdateProfileRequest {
        display_name: None,
        avatar_url: Some("a".repeat(501)),
        preferences: None,
    };

    assert!(request.validate().is_err());
}

#[tokio::test]
async fn test_validate_update_request_valid() {
    let request = UpdateProfileRequest {
        display_name: Some("Valid Name".to_string()),
        avatar_url: Some("https://example.com/avatar.jpg".to_string()),
        preferences: Some(serde_json::json!({"key": "value"})),
    };

    assert!(request.validate().is_ok());
}

#[tokio::test]
async fn test_get_audit_logs_returns_latest_first() {
    let pool = setup_test_db().await;
    let storage = ProfileStorage::new(pool.clone());
    let user_id = create_test_user(&pool, "audit_order@example.com").await;

    // Create multiple audit log entries
    for i in 0..5 {
        let update_request = UpdateProfileRequest {
            display_name: Some(format!("Update {}", i)),
            avatar_url: None,
            preferences: None,
        };

        storage
            .update_user_profile(user_id, &update_request)
            .await
            .expect("Failed to update profile");

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    let logs = storage
        .get_audit_logs(user_id, 10)
        .await
        .expect("Failed to get audit logs");

    assert_eq!(logs.len(), 5);

    // Verify logs are ordered by created_at DESC
    for i in 0..logs.len() - 1 {
        assert!(logs[i].created_at >= logs[i + 1].created_at);
    }

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_get_user_profile_with_multiple_oauth_providers() {
    let pool = setup_test_db().await;
    let storage = ProfileStorage::new(pool.clone());
    let user_id = create_test_user(&pool, "multi_oauth@example.com").await;

    // Add multiple OAuth providers
    for provider in &["google", "github", "apple"] {
        sqlx::query!(
            r#"
            INSERT INTO oauth_providers (user_id, provider, provider_user_id)
            VALUES ($1, $2, $3)
            "#,
            user_id,
            provider,
            format!("{}_user_id", provider)
        )
        .execute(&pool)
        .await
        .expect("Failed to add OAuth provider");
    }

    let profile = storage
        .get_user_profile(user_id)
        .await
        .expect("Failed to get profile")
        .expect("Profile not found");

    assert_eq!(profile.oauth_providers.len(), 3);
    assert!(profile.oauth_providers.contains(&"google".to_string()));
    assert!(profile.oauth_providers.contains(&"github".to_string()));
    assert!(profile.oauth_providers.contains(&"apple".to_string()));

    cleanup_user(&pool, user_id).await;
}
