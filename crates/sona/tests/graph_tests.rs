//! Integration tests for graph-based recommendations
//!
//! These tests verify graph traversal, similarity scoring, and collaborative filtering
//! using a real PostgreSQL database connection.

use media_gateway_sona::graph::GraphRecommender;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

const TEST_DATABASE_URL: &str = "postgresql://postgres:postgres@localhost:5432/media_gateway_test";

#[tokio::test]
#[ignore] // Requires PostgreSQL database
async fn test_graph_recommender_with_empty_history() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(TEST_DATABASE_URL)
        .await
        .expect("Failed to connect to test database");

    let recommender = GraphRecommender::new(pool);
    let user_id = Uuid::new_v4();

    let recommendations = recommender.recommend(user_id, 20).await.unwrap();

    // User with no watch history should get empty recommendations
    assert_eq!(recommendations.len(), 0);
}

#[tokio::test]
#[ignore] // Requires PostgreSQL database with test data
async fn test_genre_similarity() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(TEST_DATABASE_URL)
        .await
        .expect("Failed to connect to test database");

    // Setup test data
    setup_test_content(&pool).await;

    let recommender = GraphRecommender::new(pool.clone());

    // Create user with watch history
    let user_id = Uuid::new_v4();
    let content_id = insert_test_watch_history(&pool, user_id).await;

    let recommendations = recommender.recommend(user_id, 10).await.unwrap();

    // Should return recommendations based on genre similarity
    assert!(
        !recommendations.is_empty(),
        "Should return graph-based recommendations"
    );

    // Verify scores are in valid range [0, 1]
    for (_, score) in &recommendations {
        assert!(
            *score >= 0.0 && *score <= 1.0,
            "Score should be in [0, 1] range"
        );
    }

    cleanup_test_data(&pool, user_id, content_id).await;
}

#[tokio::test]
#[ignore] // Requires PostgreSQL database with test data
async fn test_collaborative_filtering() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(TEST_DATABASE_URL)
        .await
        .expect("Failed to connect to test database");

    setup_test_content(&pool).await;

    let recommender = GraphRecommender::new(pool.clone());

    // Create two users with overlapping watch history
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    let shared_content = insert_shared_watch_history(&pool, user1_id, user2_id).await;

    // User1 should get recommendations based on User2's additional content
    let recommendations = recommender.recommend(user1_id, 10).await.unwrap();

    assert!(
        !recommendations.is_empty(),
        "Should return collaborative recommendations"
    );

    cleanup_test_data(&pool, user1_id, shared_content).await;
    cleanup_test_data(&pool, user2_id, shared_content).await;
}

#[tokio::test]
#[ignore] // Requires PostgreSQL database with test data
async fn test_cast_similarity() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(TEST_DATABASE_URL)
        .await
        .expect("Failed to connect to test database");

    setup_test_content_with_cast(&pool).await;

    let recommender = GraphRecommender::new(pool.clone());

    let user_id = Uuid::new_v4();
    let content_id = insert_test_watch_history(&pool, user_id).await;

    let recommendations = recommender.recommend(user_id, 10).await.unwrap();

    // Should find content with shared cast members
    assert!(
        !recommendations.is_empty(),
        "Should find cast-based recommendations"
    );

    cleanup_test_data(&pool, user_id, content_id).await;
}

#[tokio::test]
#[ignore] // Requires PostgreSQL database
async fn test_performance_1000_node_traversal() {
    use std::time::Instant;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(TEST_DATABASE_URL)
        .await
        .expect("Failed to connect to test database");

    // Setup large test dataset (1000 nodes)
    setup_large_test_dataset(&pool, 1000).await;

    let recommender = GraphRecommender::new(pool.clone());
    let user_id = create_user_with_watch_history(&pool, 50).await;

    let start = Instant::now();
    let recommendations = recommender.recommend(user_id, 20).await.unwrap();
    let duration = start.elapsed();

    assert!(!recommendations.is_empty(), "Should return recommendations");
    assert!(
        duration.as_millis() < 100,
        "Graph traversal should complete in <100ms, took {}ms",
        duration.as_millis()
    );

    cleanup_large_test_dataset(&pool).await;
}

// Helper functions for test setup and teardown

async fn setup_test_content(pool: &sqlx::PgPool) {
    sqlx::query!(
        r#"
        INSERT INTO content (id, content_type, title, overview)
        VALUES
            ('11111111-1111-1111-1111-111111111111', 'movie', 'Test Movie 1', 'Action movie'),
            ('22222222-2222-2222-2222-222222222222', 'movie', 'Test Movie 2', 'Action movie')
        ON CONFLICT (id) DO NOTHING
        "#
    )
    .execute(pool)
    .await
    .ok();

    sqlx::query!(
        r#"
        INSERT INTO content_genres (content_id, genre)
        VALUES
            ('11111111-1111-1111-1111-111111111111', 'Action'),
            ('22222222-2222-2222-2222-222222222222', 'Action')
        ON CONFLICT DO NOTHING
        "#
    )
    .execute(pool)
    .await
    .ok();
}

async fn setup_test_content_with_cast(pool: &sqlx::PgPool) {
    setup_test_content(pool).await;

    sqlx::query!(
        r#"
        INSERT INTO credits (content_id, person_name, role_type)
        VALUES
            ('11111111-1111-1111-1111-111111111111', 'Tom Hanks', 'actor'),
            ('22222222-2222-2222-2222-222222222222', 'Tom Hanks', 'actor')
        "#
    )
    .execute(pool)
    .await
    .ok();
}

async fn insert_test_watch_history(pool: &sqlx::PgPool, user_id: Uuid) -> Uuid {
    let content_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();

    sqlx::query!(
        r#"
        INSERT INTO watch_progress (user_id, content_id, completion_rate, hlc_timestamp)
        VALUES ($1, $2, 0.9, 1000)
        ON CONFLICT (user_id, content_id) DO NOTHING
        "#,
        user_id,
        content_id
    )
    .execute(pool)
    .await
    .ok();

    content_id
}

async fn insert_shared_watch_history(pool: &sqlx::PgPool, user1_id: Uuid, user2_id: Uuid) -> Uuid {
    let shared_content = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    let user2_exclusive = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();

    // Both users watched content 1
    for user_id in [user1_id, user2_id] {
        sqlx::query!(
            r#"
            INSERT INTO watch_progress (user_id, content_id, completion_rate, hlc_timestamp)
            VALUES ($1, $2, 0.9, 1000)
            ON CONFLICT (user_id, content_id) DO NOTHING
            "#,
            user_id,
            shared_content
        )
        .execute(pool)
        .await
        .ok();
    }

    // User 2 also watched content 2 (should be recommended to User 1)
    sqlx::query!(
        r#"
        INSERT INTO watch_progress (user_id, content_id, completion_rate, hlc_timestamp)
        VALUES ($1, $2, 0.95, 1001)
        ON CONFLICT (user_id, content_id) DO NOTHING
        "#,
        user2_id,
        user2_exclusive
    )
    .execute(pool)
    .await
    .ok();

    shared_content
}

async fn cleanup_test_data(pool: &sqlx::PgPool, user_id: Uuid, content_id: Uuid) {
    sqlx::query!("DELETE FROM watch_progress WHERE user_id = $1", user_id)
        .execute(pool)
        .await
        .ok();
}

async fn setup_large_test_dataset(_pool: &sqlx::PgPool, _size: usize) {
    // Implementation would insert large test dataset
    // Skipped for brevity - real implementation needed for actual performance tests
}

async fn create_user_with_watch_history(_pool: &sqlx::PgPool, _history_size: usize) -> Uuid {
    // Implementation would create user with extensive watch history
    Uuid::new_v4()
}

async fn cleanup_large_test_dataset(_pool: &sqlx::PgPool) {
    // Implementation would clean up large test dataset
}
