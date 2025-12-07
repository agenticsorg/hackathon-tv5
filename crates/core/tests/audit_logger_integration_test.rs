use chrono::{Duration, Utc};
use media_gateway_core::audit::{
    AuditAction, AuditEvent, AuditFilter, AuditLogger, PostgresAuditLogger,
};
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
    });

    let pool = PgPool::connect(&database_url).await.unwrap();

    sqlx::query("DROP TABLE IF EXISTS audit_logs CASCADE")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE audit_logs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            user_id UUID,
            action VARCHAR(50) NOT NULL,
            resource_type VARCHAR(50) NOT NULL,
            resource_id VARCHAR(255),
            details JSONB NOT NULL DEFAULT '{}',
            ip_address INET,
            user_agent TEXT
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

#[tokio::test]
async fn test_query_with_date_range_filter() {
    let pool = setup_test_db().await;
    let logger = PostgresAuditLogger::new(pool.clone());

    let now = Utc::now();
    let user_id = Uuid::new_v4();

    // Create events at different times
    let event1 = AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id);
    let event2 =
        AuditEvent::new(AuditAction::UserCreated, "user".to_string()).with_user_id(user_id);
    let event3 =
        AuditEvent::new(AuditAction::ContentCreated, "content".to_string()).with_user_id(user_id);

    logger
        .log_batch(vec![event1, event2, event3])
        .await
        .unwrap();

    // Query with date range
    let filter =
        AuditFilter::new().with_date_range(now - Duration::minutes(5), now + Duration::minutes(5));
    let results = logger.query(filter).await.unwrap();

    assert_eq!(
        results.len(),
        3,
        "Should return all events within date range"
    );
}

#[tokio::test]
async fn test_query_with_start_date_only() {
    let pool = setup_test_db().await;
    let logger = PostgresAuditLogger::new(pool.clone());

    let now = Utc::now();
    let user_id = Uuid::new_v4();

    let events = vec![
        AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id),
        AuditEvent::new(AuditAction::UserCreated, "user".to_string()).with_user_id(user_id),
    ];

    logger.log_batch(events).await.unwrap();

    // Query with only start_date
    let filter = AuditFilter {
        start_date: Some(now - Duration::hours(1)),
        end_date: None,
        user_id: None,
        action: None,
        resource_type: None,
        limit: Some(100),
        offset: Some(0),
    };
    let results = logger.query(filter).await.unwrap();

    assert!(!results.is_empty(), "Should return events after start_date");
}

#[tokio::test]
async fn test_query_with_end_date_only() {
    let pool = setup_test_db().await;
    let logger = PostgresAuditLogger::new(pool.clone());

    let now = Utc::now();
    let user_id = Uuid::new_v4();

    let events =
        vec![AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id)];

    logger.log_batch(events).await.unwrap();

    // Query with only end_date
    let filter = AuditFilter {
        start_date: None,
        end_date: Some(now + Duration::hours(1)),
        user_id: None,
        action: None,
        resource_type: None,
        limit: Some(100),
        offset: Some(0),
    };
    let results = logger.query(filter).await.unwrap();

    assert!(!results.is_empty(), "Should return events before end_date");
}

#[tokio::test]
async fn test_query_with_action_filter() {
    let pool = setup_test_db().await;
    let logger = PostgresAuditLogger::new(pool.clone());

    let user_id = Uuid::new_v4();

    let events = vec![
        AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id),
        AuditEvent::new(AuditAction::UserCreated, "user".to_string()).with_user_id(user_id),
        AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id),
        AuditEvent::new(AuditAction::ContentCreated, "content".to_string()).with_user_id(user_id),
    ];

    logger.log_batch(events).await.unwrap();

    // Query only AuthLogin actions
    let filter = AuditFilter::new().with_action(AuditAction::AuthLogin);
    let results = logger.query(filter).await.unwrap();

    assert_eq!(results.len(), 2, "Should return only AuthLogin events");
    assert!(results.iter().all(|e| e.action == AuditAction::AuthLogin));
}

#[tokio::test]
async fn test_query_with_limit_and_offset() {
    let pool = setup_test_db().await;
    let logger = PostgresAuditLogger::new(pool.clone());

    let user_id = Uuid::new_v4();

    // Create 10 events
    let mut events = Vec::new();
    for i in 0..10 {
        events.push(
            AuditEvent::new(AuditAction::AuthLogin, "user".to_string())
                .with_user_id(user_id)
                .with_resource_id(format!("login-{}", i)),
        );
    }

    logger.log_batch(events).await.unwrap();

    // Query with limit
    let filter = AuditFilter::new().with_limit(5);
    let results = logger.query(filter).await.unwrap();
    assert_eq!(results.len(), 5, "Should return only 5 events");

    // Query with limit and offset
    let filter = AuditFilter::new().with_limit(3).with_offset(2);
    let results = logger.query(filter).await.unwrap();
    assert_eq!(
        results.len(),
        3,
        "Should return 3 events starting from offset 2"
    );

    // Query second page
    let filter = AuditFilter::new().with_limit(5).with_offset(5);
    let results = logger.query(filter).await.unwrap();
    assert_eq!(results.len(), 5, "Should return second page of 5 events");
}

#[tokio::test]
async fn test_query_with_multiple_filters() {
    let pool = setup_test_db().await;
    let logger = PostgresAuditLogger::new(pool.clone());

    let now = Utc::now();
    let user_id_1 = Uuid::new_v4();
    let user_id_2 = Uuid::new_v4();

    let events = vec![
        AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id_1),
        AuditEvent::new(AuditAction::UserCreated, "user".to_string()).with_user_id(user_id_1),
        AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id_2),
        AuditEvent::new(AuditAction::ContentCreated, "content".to_string()).with_user_id(user_id_1),
        AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id_1),
    ];

    logger.log_batch(events).await.unwrap();

    // Query with multiple filters: user_id + action + date_range
    let filter = AuditFilter::new()
        .with_user_id(user_id_1)
        .with_action(AuditAction::AuthLogin)
        .with_date_range(now - Duration::hours(1), now + Duration::hours(1))
        .with_limit(10);

    let results = logger.query(filter).await.unwrap();

    assert_eq!(
        results.len(),
        2,
        "Should return 2 AuthLogin events for user_id_1"
    );
    assert!(results.iter().all(|e| e.action == AuditAction::AuthLogin));
    assert!(results.iter().all(|e| e.user_id == Some(user_id_1)));
}

#[tokio::test]
async fn test_query_with_resource_type_filter() {
    let pool = setup_test_db().await;
    let logger = PostgresAuditLogger::new(pool.clone());

    let user_id = Uuid::new_v4();

    let events = vec![
        AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id),
        AuditEvent::new(AuditAction::ContentCreated, "content".to_string()).with_user_id(user_id),
        AuditEvent::new(AuditAction::ContentUpdated, "content".to_string()).with_user_id(user_id),
        AuditEvent::new(AuditAction::UserCreated, "user".to_string()).with_user_id(user_id),
    ];

    logger.log_batch(events).await.unwrap();

    // Query only content-related events
    let filter = AuditFilter::new().with_resource_type("content".to_string());
    let results = logger.query(filter).await.unwrap();

    assert_eq!(
        results.len(),
        2,
        "Should return only content resource type events"
    );
    assert!(results.iter().all(|e| e.resource_type == "content"));
}

#[tokio::test]
async fn test_query_empty_results() {
    let pool = setup_test_db().await;
    let logger = PostgresAuditLogger::new(pool.clone());

    let user_id = Uuid::new_v4();

    // Insert one event
    let event = AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id);
    logger.log_batch(vec![event]).await.unwrap();

    // Query with non-matching filter
    let non_existent_user = Uuid::new_v4();
    let filter = AuditFilter::new().with_user_id(non_existent_user);
    let results = logger.query(filter).await.unwrap();

    assert_eq!(
        results.len(),
        0,
        "Should return empty results for non-matching filter"
    );
}

#[tokio::test]
async fn test_query_ordering() {
    let pool = setup_test_db().await;
    let logger = PostgresAuditLogger::new(pool.clone());

    let user_id = Uuid::new_v4();

    // Create events with slight delays to ensure different timestamps
    for i in 0..5 {
        let event = AuditEvent::new(AuditAction::AuthLogin, "user".to_string())
            .with_user_id(user_id)
            .with_resource_id(format!("login-{}", i));
        logger.log(event).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Flush buffer
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let filter = AuditFilter::new().with_user_id(user_id);
    let results = logger.query(filter).await.unwrap();

    // Results should be ordered by timestamp DESC (most recent first)
    assert_eq!(results.len(), 5);
    for i in 0..results.len() - 1 {
        assert!(
            results[i].timestamp >= results[i + 1].timestamp,
            "Results should be ordered by timestamp DESC"
        );
    }
}

#[tokio::test]
async fn test_query_with_all_filters() {
    let pool = setup_test_db().await;
    let logger = PostgresAuditLogger::new(pool.clone());

    let now = Utc::now();
    let user_id = Uuid::new_v4();

    // Create diverse set of events
    let events = vec![
        AuditEvent::new(AuditAction::AuthLogin, "user".to_string())
            .with_user_id(user_id)
            .with_resource_id("res-1".to_string()),
        AuditEvent::new(AuditAction::AuthLogin, "user".to_string())
            .with_user_id(user_id)
            .with_resource_id("res-2".to_string()),
        AuditEvent::new(AuditAction::UserCreated, "user".to_string()).with_user_id(user_id),
        AuditEvent::new(AuditAction::ContentCreated, "content".to_string()).with_user_id(user_id),
    ];

    logger.log_batch(events).await.unwrap();

    // Query with all available filters
    let filter = AuditFilter {
        start_date: Some(now - Duration::hours(1)),
        end_date: Some(now + Duration::hours(1)),
        user_id: Some(user_id),
        action: Some(AuditAction::AuthLogin),
        resource_type: Some("user".to_string()),
        limit: Some(10),
        offset: Some(0),
    };

    let results = logger.query(filter).await.unwrap();

    assert_eq!(
        results.len(),
        2,
        "Should return 2 AuthLogin events for user resource type"
    );
    assert!(results.iter().all(|e| e.action == AuditAction::AuthLogin));
    assert!(results.iter().all(|e| e.user_id == Some(user_id)));
    assert!(results.iter().all(|e| e.resource_type == "user"));
}
