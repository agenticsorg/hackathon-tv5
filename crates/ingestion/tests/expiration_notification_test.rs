//! Integration tests for content expiration notifications
//!
//! These tests verify the expiration notification system including:
//! - Detection of expiring content
//! - Notification tracking to prevent duplicates
//! - Kafka event emission
//! - API endpoint functionality
//!
//! Run with: cargo test --test expiration_notification_test -- --test-threads=1

use chrono::{Duration, Utc};
use media_gateway_ingestion::normalizer::{
    AvailabilityInfo, CanonicalContent, ContentType, ImageSet,
};
use media_gateway_ingestion::notifications::{
    ExpirationNotificationConfig, ExpirationNotificationJob, NotificationWindow,
};
use media_gateway_ingestion::repository::{ContentRepository, PostgresContentRepository};
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use uuid::Uuid;

/// Database URL for integration tests
fn get_test_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/media_gateway_test".to_string())
}

/// Setup test database pool
async fn setup_test_pool() -> sqlx::PgPool {
    let database_url = get_test_database_url();

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Create test content with expiration date
fn create_expiring_content(
    title: &str,
    platform: &str,
    region: &str,
    expires_in_days: i64,
) -> CanonicalContent {
    let mut external_ids = HashMap::new();
    external_ids.insert(
        "imdb".to_string(),
        format!("tt{}", Uuid::new_v4().as_u128() % 10000000),
    );

    let expires_at = Utc::now() + Duration::days(expires_in_days);

    CanonicalContent {
        platform_content_id: format!("exp_test_{}", Uuid::new_v4()),
        platform_id: platform.to_string(),
        entity_id: None,
        title: title.to_string(),
        overview: Some("Test content for expiration notifications".to_string()),
        content_type: ContentType::Movie,
        release_year: Some(2024),
        runtime_minutes: Some(120),
        genres: vec!["Drama".to_string()],
        external_ids,
        availability: AvailabilityInfo {
            regions: vec![region.to_string()],
            subscription_required: true,
            purchase_price: None,
            rental_price: None,
            currency: Some("USD".to_string()),
            available_from: Some(Utc::now()),
            available_until: Some(expires_at),
        },
        images: ImageSet::default(),
        rating: Some("PG-13".to_string()),
        user_rating: Some(7.5),
        embedding: None,
        updated_at: Utc::now(),
    }
}

/// Cleanup helper
async fn cleanup_test_content(pool: &sqlx::PgPool, content_id: Uuid) {
    sqlx::query("DELETE FROM content WHERE id = $1")
        .bind(content_id)
        .execute(pool)
        .await
        .expect("Failed to cleanup test content");
}

#[tokio::test]
#[ignore]
async fn test_initialize_tracking_table() {
    let pool = setup_test_pool().await;
    let config = ExpirationNotificationConfig::default();

    let job = ExpirationNotificationJob::new(pool.clone(), config)
        .expect("Failed to create notification job");

    // Initialize tracking table
    job.initialize_tracking_table()
        .await
        .expect("Failed to initialize tracking table");

    // Verify table exists
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_name = 'expiration_notifications'
        )
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to check table existence");

    assert!(exists, "Tracking table should exist");
}

#[tokio::test]
#[ignore]
async fn test_detect_expiring_content() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    // Create content expiring in 5 days
    let content = create_expiring_content("Expiring Soon Movie", "netflix", "US", 5);
    let content_id = repo
        .upsert(&content)
        .await
        .expect("Failed to insert content");

    // Find content expiring within 7 days
    let expiring = repo
        .find_expiring_within(Duration::days(7))
        .await
        .expect("Failed to find expiring content");

    // Should find our test content
    let found = expiring.iter().any(|c| c.content_id == content_id);
    assert!(found, "Should find content expiring in 5 days");

    // Should NOT find when looking only 3 days ahead
    let expiring_3d = repo
        .find_expiring_within(Duration::days(3))
        .await
        .expect("Failed to find expiring content");

    let found_3d = expiring_3d.iter().any(|c| c.content_id == content_id);
    assert!(
        !found_3d,
        "Should not find content when window is too short"
    );

    // Cleanup
    cleanup_test_content(&pool, content_id).await;
}

#[tokio::test]
#[ignore]
async fn test_notification_tracking() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());
    let config = ExpirationNotificationConfig::default();

    let job = ExpirationNotificationJob::new(pool.clone(), config)
        .expect("Failed to create notification job");

    job.initialize_tracking_table()
        .await
        .expect("Failed to initialize tracking table");

    // Create content expiring in 7 days
    let content = create_expiring_content("Track Notification Movie", "netflix", "US", 7);
    let content_id = repo
        .upsert(&content)
        .await
        .expect("Failed to insert content");

    // Get the expiring content
    let expiring = repo
        .find_expiring_within(Duration::days(7))
        .await
        .expect("Failed to find expiring content");

    let test_content = expiring
        .iter()
        .find(|c| c.content_id == content_id)
        .expect("Should find test content");

    // Check not notified initially
    let is_notified = job
        .is_already_notified(test_content, NotificationWindow::SevenDays)
        .await
        .expect("Failed to check notification status");

    assert!(!is_notified, "Should not be notified initially");

    // Mark as notified
    job.mark_as_notified(test_content, NotificationWindow::SevenDays)
        .await
        .expect("Failed to mark as notified");

    // Check now notified
    let is_notified_after = job
        .is_already_notified(test_content, NotificationWindow::SevenDays)
        .await
        .expect("Failed to check notification status");

    assert!(is_notified_after, "Should be marked as notified");

    // Different window should still be not notified
    let is_notified_3d = job
        .is_already_notified(test_content, NotificationWindow::ThreeDays)
        .await
        .expect("Failed to check notification status");

    assert!(!is_notified_3d, "Different window should not be notified");

    // Cleanup
    cleanup_test_content(&pool, content_id).await;
}

#[tokio::test]
#[ignore]
async fn test_notification_history() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());
    let config = ExpirationNotificationConfig::default();

    let job = ExpirationNotificationJob::new(pool.clone(), config)
        .expect("Failed to create notification job");

    job.initialize_tracking_table()
        .await
        .expect("Failed to initialize tracking table");

    // Create content expiring in 7 days
    let content = create_expiring_content("History Test Movie", "netflix", "US", 7);
    let content_id = repo
        .upsert(&content)
        .await
        .expect("Failed to insert content");

    // Get expiring content
    let expiring = repo
        .find_expiring_within(Duration::days(7))
        .await
        .expect("Failed to find expiring content");

    let test_content = expiring
        .iter()
        .find(|c| c.content_id == content_id)
        .expect("Should find test content");

    // Mark as notified for multiple windows
    job.mark_as_notified(test_content, NotificationWindow::SevenDays)
        .await
        .expect("Failed to mark 7d");

    job.mark_as_notified(test_content, NotificationWindow::ThreeDays)
        .await
        .expect("Failed to mark 3d");

    // Get notification history
    let history = job
        .get_notification_history(content_id)
        .await
        .expect("Failed to get history");

    assert_eq!(history.len(), 2, "Should have 2 notification records");

    // Verify windows
    let windows: Vec<NotificationWindow> = history.iter().map(|h| h.window).collect();
    assert!(windows.contains(&NotificationWindow::SevenDays));
    assert!(windows.contains(&NotificationWindow::ThreeDays));

    // Cleanup
    cleanup_test_content(&pool, content_id).await;
}

#[tokio::test]
#[ignore]
async fn test_check_and_notify_no_duplicates() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    let mut config = ExpirationNotificationConfig::default();
    config.enable_kafka = false; // Disable Kafka for this test
    config.notification_windows = vec![7, 3, 1];

    let job = ExpirationNotificationJob::new(pool.clone(), config)
        .expect("Failed to create notification job");

    job.initialize_tracking_table()
        .await
        .expect("Failed to initialize tracking table");

    // Create content expiring in 7 days
    let content = create_expiring_content("No Duplicate Movie", "netflix", "US", 7);
    let content_id = repo
        .upsert(&content)
        .await
        .expect("Failed to insert content");

    // First check - should send notification
    let count1 = job
        .check_and_notify()
        .await
        .expect("Failed to check and notify");
    assert!(count1 > 0, "Should send at least one notification");

    // Second check - should not send duplicate
    let count2 = job
        .check_and_notify()
        .await
        .expect("Failed to check and notify");
    assert_eq!(count2, 0, "Should not send duplicate notifications");

    // Cleanup
    cleanup_test_content(&pool, content_id).await;
}

#[tokio::test]
#[ignore]
async fn test_multiple_windows() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    // Create content expiring at different times
    let content_7d = create_expiring_content("Expiring 7 Days", "netflix", "US", 7);
    let content_3d = create_expiring_content("Expiring 3 Days", "netflix", "US", 3);
    let content_1d = create_expiring_content("Expiring 1 Day", "netflix", "US", 1);

    let id_7d = repo.upsert(&content_7d).await.expect("Failed to insert");
    let id_3d = repo.upsert(&content_3d).await.expect("Failed to insert");
    let id_1d = repo.upsert(&content_1d).await.expect("Failed to insert");

    // Find with 7-day window - should find all
    let expiring_7d = repo
        .find_expiring_within(Duration::days(7))
        .await
        .expect("Failed to find");

    assert!(expiring_7d.iter().any(|c| c.content_id == id_7d));
    assert!(expiring_7d.iter().any(|c| c.content_id == id_3d));
    assert!(expiring_7d.iter().any(|c| c.content_id == id_1d));

    // Find with 3-day window - should find 3d and 1d only
    let expiring_3d = repo
        .find_expiring_within(Duration::days(3))
        .await
        .expect("Failed to find");

    assert!(!expiring_3d.iter().any(|c| c.content_id == id_7d));
    assert!(expiring_3d.iter().any(|c| c.content_id == id_3d));
    assert!(expiring_3d.iter().any(|c| c.content_id == id_1d));

    // Find with 1-day window - should find 1d only
    let expiring_1d = repo
        .find_expiring_within(Duration::days(1))
        .await
        .expect("Failed to find");

    assert!(!expiring_1d.iter().any(|c| c.content_id == id_7d));
    assert!(!expiring_1d.iter().any(|c| c.content_id == id_3d));
    assert!(expiring_1d.iter().any(|c| c.content_id == id_1d));

    // Cleanup
    cleanup_test_content(&pool, id_7d).await;
    cleanup_test_content(&pool, id_3d).await;
    cleanup_test_content(&pool, id_1d).await;
}

#[tokio::test]
#[ignore]
async fn test_cleanup_old_notifications() {
    let pool = setup_test_pool().await;
    let config = ExpirationNotificationConfig::default();

    let job = ExpirationNotificationJob::new(pool.clone(), config)
        .expect("Failed to create notification job");

    job.initialize_tracking_table()
        .await
        .expect("Failed to initialize tracking table");

    // Insert old notification record (100 days ago)
    let old_date = Utc::now() - Duration::days(100);

    sqlx::query(
        r#"
        INSERT INTO expiration_notifications (
            content_id, platform, region, notification_window, expires_at, notified_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind("netflix")
    .bind("US")
    .bind("7d")
    .bind(Utc::now())
    .bind(old_date)
    .execute(&pool)
    .await
    .expect("Failed to insert old record");

    // Cleanup old notifications
    let deleted = job
        .cleanup_old_notifications()
        .await
        .expect("Failed to cleanup");

    assert!(deleted > 0, "Should delete old notifications");
}

#[tokio::test]
#[ignore]
async fn test_notification_window_helpers() {
    assert_eq!(NotificationWindow::SevenDays.duration().num_days(), 7);
    assert_eq!(NotificationWindow::ThreeDays.duration().num_days(), 3);
    assert_eq!(NotificationWindow::OneDay.duration().num_days(), 1);

    assert_eq!(NotificationWindow::SevenDays.identifier(), "7d");
    assert_eq!(NotificationWindow::ThreeDays.identifier(), "3d");
    assert_eq!(NotificationWindow::OneDay.identifier(), "1d");

    assert_eq!(
        NotificationWindow::from_days(7),
        NotificationWindow::SevenDays
    );
    assert_eq!(
        NotificationWindow::from_days(3),
        NotificationWindow::ThreeDays
    );
    assert_eq!(NotificationWindow::from_days(1), NotificationWindow::OneDay);
}
