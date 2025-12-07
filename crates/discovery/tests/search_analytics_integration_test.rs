use chrono::Utc;
use discovery::analytics::{PeriodType, SearchAnalytics};
use sqlx::PgPool;
use std::collections::HashMap;

#[tokio::test]
#[ignore] // Integration test - requires PostgreSQL database
async fn test_complete_analytics_workflow() {
    let pool = setup_test_db().await;
    let analytics = SearchAnalytics::new(pool.clone());

    // Step 1: Log multiple search events
    let mut event_ids = Vec::new();
    for i in 0..50 {
        let query = if i < 20 {
            "action movies"
        } else if i < 35 {
            "comedy shows"
        } else if i < 45 {
            "drama series"
        } else {
            "nonexistent content xyz"
        };

        let result_count = if query.contains("nonexistent") {
            0
        } else {
            10 + i
        };
        let latency_ms = 100 + (i % 10) * 20;

        let mut filters = HashMap::new();
        if query.contains("action") {
            filters.insert("genre".to_string(), serde_json::json!("action"));
        }

        let event_id = analytics
            .query_log()
            .log_search(
                query,
                Some(&format!("user{}", i % 10)),
                result_count,
                latency_ms,
                filters,
            )
            .await
            .expect("Failed to log search");

        event_ids.push(event_id);
    }

    // Step 2: Log clicks on search results
    for (idx, event_id) in event_ids.iter().enumerate() {
        // Click on 60% of searches
        if idx % 5 < 3 {
            let content_id = uuid::Uuid::new_v4();
            analytics
                .query_log()
                .log_click(*event_id, content_id, (idx % 5) as i32)
                .await
                .expect("Failed to log click");
        }
    }

    // Step 3: Calculate latency stats
    let since = Utc::now() - chrono::Duration::hours(1);
    let latency_stats = analytics
        .calculate_latency_stats(since)
        .await
        .expect("Failed to calculate latency stats");

    assert!(
        latency_stats.p50 >= 100,
        "P50 latency should be at least 100ms"
    );
    assert!(
        latency_stats.p95 > latency_stats.p50,
        "P95 should be higher than P50"
    );
    assert!(
        latency_stats.p99 > latency_stats.p95,
        "P99 should be higher than P95"
    );
    assert!(
        latency_stats.avg > 0.0,
        "Average latency should be positive"
    );

    // Step 4: Get top queries
    let top_queries = analytics
        .get_top_queries(since, 5)
        .await
        .expect("Failed to get top queries");

    assert_eq!(top_queries.len(), 4, "Should have 4 unique queries");
    assert_eq!(
        top_queries[0].query, "action movies",
        "Most popular query should be first"
    );
    assert_eq!(
        top_queries[0].count, 20,
        "Action movies should have 20 searches"
    );
    assert!(top_queries[0].ctr > 0.0, "CTR should be calculated");

    // Step 5: Get zero-result queries
    let zero_results = analytics
        .get_zero_result_queries(since, 10)
        .await
        .expect("Failed to get zero-result queries");

    assert!(!zero_results.is_empty(), "Should have zero-result queries");
    assert_eq!(zero_results[0].query, "nonexistent content xyz");
    assert_eq!(
        zero_results[0].count, 5,
        "Should have 5 zero-result searches"
    );

    // Step 6: Calculate CTR
    let ctr = analytics
        .calculate_ctr(since)
        .await
        .expect("Failed to calculate CTR");

    assert!(
        ctr >= 0.55 && ctr <= 0.65,
        "CTR should be around 0.6 (30/50)"
    );

    // Step 7: Aggregate popular searches
    let period_start = Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    analytics
        .aggregate_popular_searches(PeriodType::Hourly, period_start)
        .await
        .expect("Failed to aggregate popular searches");

    // Verify aggregation
    let aggregated = sqlx::query!(
        r#"
        SELECT query_text, search_count, avg_results, avg_latency_ms, ctr
        FROM popular_searches
        WHERE query_text = 'action movies' AND period_type = 'hourly'
        ORDER BY period_start DESC
        LIMIT 1
        "#
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch aggregated data");

    assert_eq!(aggregated.search_count, 20, "Aggregated count should match");
    assert!(
        aggregated.avg_results.is_some(),
        "Should have average results"
    );
    assert!(
        aggregated.avg_latency_ms.is_some(),
        "Should have average latency"
    );
    assert!(aggregated.ctr.is_some(), "Should have CTR");

    // Step 8: Get dashboard
    let dashboard = analytics
        .get_dashboard("24h", 10)
        .await
        .expect("Failed to get dashboard");

    assert_eq!(dashboard.period, "24h");
    assert_eq!(dashboard.total_searches, 50);
    assert_eq!(dashboard.unique_queries, 4);
    assert!(dashboard.avg_latency_ms > 0.0);
    assert!(dashboard.p95_latency_ms > 0);
    assert!(dashboard.zero_result_rate > 0.0 && dashboard.zero_result_rate <= 0.2);
    assert!(dashboard.avg_ctr >= 0.5 && dashboard.avg_ctr <= 0.7);
    assert_eq!(dashboard.top_queries.len(), 4);
    assert_eq!(dashboard.zero_result_queries.len(), 1);

    cleanup_test_db(&pool).await;
}

#[tokio::test]
#[ignore] // Integration test - requires PostgreSQL database
async fn test_query_anonymization() {
    let pool = setup_test_db().await;
    let analytics = SearchAnalytics::new(pool.clone());

    let original_user_id = "sensitive_user_12345";
    let original_query = "action movies";

    // Log search with real user ID
    analytics
        .query_log()
        .log_search(
            original_query,
            Some(original_user_id),
            10,
            100,
            HashMap::new(),
        )
        .await
        .expect("Failed to log search");

    // Fetch the event from database
    let event = sqlx::query!(
        r#"
        SELECT user_id_hash, query_text
        FROM search_events
        WHERE query_text = $1
        LIMIT 1
        "#,
        original_query
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch event");

    // Verify user ID is anonymized
    assert!(event.user_id_hash.is_some(), "User ID hash should exist");
    assert_ne!(
        event.user_id_hash.as_ref().unwrap(),
        original_user_id,
        "User ID should be anonymized"
    );
    assert_eq!(
        event.user_id_hash.as_ref().unwrap().len(),
        64,
        "SHA-256 hash should be 64 characters"
    );

    cleanup_test_db(&pool).await;
}

#[tokio::test]
#[ignore] // Integration test - requires PostgreSQL database
async fn test_time_series_optimization() {
    let pool = setup_test_db().await;
    let analytics = SearchAnalytics::new(pool.clone());

    // Log many events to test time-series performance
    for i in 0..100 {
        analytics
            .query_log()
            .log_search(
                &format!("query {}", i % 20),
                Some("user123"),
                10,
                100 + i,
                HashMap::new(),
            )
            .await
            .expect("Failed to log search");
    }

    // Test time-based query performance
    let start = std::time::Instant::now();
    let since = Utc::now() - chrono::Duration::minutes(5);
    let recent_events = analytics
        .query_log()
        .get_recent_events(50)
        .await
        .expect("Failed to get recent events");
    let elapsed = start.elapsed();

    assert_eq!(
        recent_events.len(),
        50,
        "Should fetch 50 most recent events"
    );
    assert!(
        elapsed.as_millis() < 1000,
        "Query should complete in under 1 second"
    );

    // Verify events are in descending order by time
    for i in 1..recent_events.len() {
        assert!(
            recent_events[i - 1].created_at >= recent_events[i].created_at,
            "Events should be ordered by created_at DESC"
        );
    }

    cleanup_test_db(&pool).await;
}

#[tokio::test]
#[ignore] // Integration test - requires PostgreSQL database
async fn test_concurrent_analytics_operations() {
    let pool = setup_test_db().await;
    let analytics = SearchAnalytics::new(pool.clone());

    // Spawn multiple concurrent analytics operations
    let mut handles = vec![];

    for i in 0..10 {
        let analytics_clone = analytics.clone();
        let handle = tokio::spawn(async move {
            for j in 0..5 {
                let query = format!("query {} {}", i, j);
                analytics_clone
                    .query_log()
                    .log_search(
                        &query,
                        Some(&format!("user{}", i)),
                        10 + j,
                        100 + j * 10,
                        HashMap::new(),
                    )
                    .await
                    .expect("Failed to log search");
            }
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await.expect("Task panicked");
    }

    // Verify all events were logged
    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM search_events
        "#
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count events");

    assert_eq!(
        total, 50,
        "Should have 50 events from concurrent operations"
    );

    cleanup_test_db(&pool).await;
}

#[tokio::test]
#[ignore] // Integration test - requires PostgreSQL database
async fn test_click_tracking_and_ctr() {
    let pool = setup_test_db().await;
    let analytics = SearchAnalytics::new(pool.clone());

    // Create searches with varying click patterns
    let mut event_ids = Vec::new();

    // 5 searches with 1 click each
    for i in 0..5 {
        let event_id = analytics
            .query_log()
            .log_search("popular query", Some("user123"), 10, 100, HashMap::new())
            .await
            .expect("Failed to log search");
        event_ids.push((event_id, 1));
    }

    // 3 searches with 2 clicks each
    for i in 0..3 {
        let event_id = analytics
            .query_log()
            .log_search("very popular", Some("user456"), 10, 100, HashMap::new())
            .await
            .expect("Failed to log search");
        event_ids.push((event_id, 2));
    }

    // 2 searches with no clicks
    for i in 0..2 {
        let event_id = analytics
            .query_log()
            .log_search("unpopular", Some("user789"), 10, 100, HashMap::new())
            .await
            .expect("Failed to log search");
        event_ids.push((event_id, 0));
    }

    // Log clicks
    for (event_id, click_count) in &event_ids {
        for i in 0..*click_count {
            let content_id = uuid::Uuid::new_v4();
            analytics
                .query_log()
                .log_click(*event_id, content_id, i)
                .await
                .expect("Failed to log click");
        }
    }

    // Calculate overall CTR
    let since = Utc::now() - chrono::Duration::hours(1);
    let ctr = analytics
        .calculate_ctr(since)
        .await
        .expect("Failed to calculate CTR");

    // 5 searches with 1 click + 3 searches with 1 click = 8 searches with clicks
    // Total searches = 10
    // CTR = 8/10 = 0.8
    assert!(ctr >= 0.75 && ctr <= 0.85, "CTR should be around 0.8");

    cleanup_test_db(&pool).await;
}

// Test helpers
async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
    });

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::query(include_str!("../migrations/20251206_search_analytics.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migrations");

    // Clean up any existing test data
    cleanup_test_db(&pool).await;

    pool
}

async fn cleanup_test_db(pool: &PgPool) {
    sqlx::query("TRUNCATE search_events, search_clicks, popular_searches CASCADE")
        .execute(pool)
        .await
        .expect("Failed to cleanup test database");
}
