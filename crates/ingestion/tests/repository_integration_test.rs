//! Integration tests for PostgreSQL repository operations
//!
//! These tests require a running PostgreSQL database.
//! Run with: cargo test --test repository_integration_test -- --test-threads=1

use chrono::Utc;
use media_gateway_ingestion::normalizer::{
    AvailabilityInfo, CanonicalContent, ContentType, ImageSet,
};
use media_gateway_ingestion::repository::{ContentRepository, PostgresContentRepository};
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use uuid::Uuid;

/// Database URL for integration tests
/// Set via environment variable: DATABASE_URL=postgres://user:pass@localhost/media_gateway_test
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

/// Create a test CanonicalContent instance
fn create_test_content(title: &str, platform: &str, platform_id: &str) -> CanonicalContent {
    let mut external_ids = HashMap::new();
    external_ids.insert("imdb".to_string(), "tt1234567".to_string());
    external_ids.insert("tmdb".to_string(), "12345".to_string());

    CanonicalContent {
        platform_content_id: platform_id.to_string(),
        platform_id: platform.to_string(),
        entity_id: None,
        title: title.to_string(),
        overview: Some("A test movie about testing".to_string()),
        content_type: ContentType::Movie,
        release_year: Some(2024),
        runtime_minutes: Some(120),
        genres: vec!["Action".to_string(), "Thriller".to_string()],
        external_ids,
        availability: AvailabilityInfo {
            regions: vec!["US".to_string(), "CA".to_string()],
            subscription_required: true,
            purchase_price: Some(9.99),
            rental_price: Some(3.99),
            currency: Some("USD".to_string()),
            available_from: Some(Utc::now()),
            available_until: None,
        },
        images: ImageSet {
            poster_small: Some("https://example.com/poster_small.jpg".to_string()),
            poster_medium: Some("https://example.com/poster_medium.jpg".to_string()),
            poster_large: Some("https://example.com/poster_large.jpg".to_string()),
            backdrop: Some("https://example.com/backdrop.jpg".to_string()),
        },
        rating: Some("PG-13".to_string()),
        user_rating: Some(8.5),
        embedding: None,
        updated_at: Utc::now(),
    }
}

/// Create test content with EIDR ID
fn create_test_content_with_eidr(title: &str, eidr: &str) -> CanonicalContent {
    let mut content = create_test_content(title, "netflix", "netflix_12345");
    content
        .external_ids
        .insert("eidr".to_string(), eidr.to_string());
    content
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_upsert_new_content() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    let content = create_test_content("Test Movie - New", "netflix", "netflix_new_001");

    // Insert new content
    let content_id = repo
        .upsert(&content)
        .await
        .expect("Failed to upsert content");

    // Verify content was inserted
    let result: (String, String, i32) = sqlx::query_as(
        "SELECT c.title, c.content_type, c.runtime_minutes
         FROM content c WHERE c.id = $1",
    )
    .bind(content_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch inserted content");

    assert_eq!(result.0, "Test Movie - New");
    assert_eq!(result.1, "movie");
    assert_eq!(result.2, 120);

    // Verify external IDs
    let ext_id: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT imdb_id, tmdb_id FROM external_ids WHERE content_id = $1")
            .bind(content_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch external IDs");

    assert_eq!(ext_id.0, Some("tt1234567".to_string()));
    assert_eq!(ext_id.1, Some(12345));

    // Verify genres
    let genre_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM content_genres WHERE content_id = $1")
            .bind(content_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count genres");

    assert_eq!(genre_count, 2);

    // Verify platform availability
    let avail_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM platform_availability WHERE content_id = $1")
            .bind(content_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count availability records");

    assert_eq!(avail_count, 2); // US and CA

    // Cleanup
    cleanup_test_content(&pool, content_id).await;
}

#[tokio::test]
#[ignore]
async fn test_upsert_existing_content_by_imdb() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    let mut content1 = create_test_content("Original Title", "netflix", "netflix_001");
    content1
        .external_ids
        .insert("imdb".to_string(), "tt9999999".to_string());

    // Insert first version
    let content_id_1 = repo
        .upsert(&content1)
        .await
        .expect("Failed to insert first version");

    // Create updated version with same IMDB ID but different title
    let mut content2 = create_test_content("Updated Title", "netflix", "netflix_002");
    content2
        .external_ids
        .insert("imdb".to_string(), "tt9999999".to_string());
    content2.runtime_minutes = Some(150);

    // Upsert should update, not create new record
    let content_id_2 = repo
        .upsert(&content2)
        .await
        .expect("Failed to upsert second version");

    // Should be the same ID
    assert_eq!(content_id_1, content_id_2);

    // Verify content was updated
    let result: (String, i32) =
        sqlx::query_as("SELECT c.title, c.runtime_minutes FROM content c WHERE c.id = $1")
            .bind(content_id_2)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch updated content");

    assert_eq!(result.0, "Updated Title");
    assert_eq!(result.1, 150);

    // Verify only one record exists with this IMDB ID
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM external_ids WHERE imdb_id = $1")
        .bind("tt9999999")
        .fetch_one(&pool)
        .await
        .expect("Failed to count IMDB records");

    assert_eq!(count, 1);

    // Cleanup
    cleanup_test_content(&pool, content_id_2).await;
}

#[tokio::test]
#[ignore]
async fn test_upsert_existing_content_by_eidr() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    let eidr = "10.5240/AAAA-BBBB-CCCC-DDDD";

    let content1 = create_test_content_with_eidr("EIDR Test Movie", eidr);
    let content_id_1 = repo
        .upsert(&content1)
        .await
        .expect("Failed to insert with EIDR");

    // Create another content with same EIDR
    let mut content2 = create_test_content_with_eidr("EIDR Test Movie Updated", eidr);
    content2.overview = Some("Updated overview".to_string());

    let content_id_2 = repo
        .upsert(&content2)
        .await
        .expect("Failed to upsert with EIDR");

    // Should match by EIDR and update
    assert_eq!(content_id_1, content_id_2);

    let overview: String = sqlx::query_scalar("SELECT overview FROM content WHERE id = $1")
        .bind(content_id_2)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch overview");

    assert_eq!(overview, "Updated overview");

    // Cleanup
    cleanup_test_content(&pool, content_id_2).await;
}

#[tokio::test]
#[ignore]
async fn test_upsert_existing_content_by_title_and_year() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    // Create content without EIDR or IMDB
    let mut content1 = create_test_content("Unique Title 2024", "disney", "disney_001");
    content1.external_ids.clear();
    content1.release_year = Some(2024);

    let content_id_1 = repo.upsert(&content1).await.expect("Failed to insert");

    // Create content with same title and year (within tolerance)
    let mut content2 = create_test_content("Unique Title 2024", "disney", "disney_002");
    content2.external_ids.clear();
    content2.release_year = Some(2024);
    content2.runtime_minutes = Some(140);

    let content_id_2 = repo.upsert(&content2).await.expect("Failed to upsert");

    // Should match by title + year
    assert_eq!(content_id_1, content_id_2);

    let runtime: i32 = sqlx::query_scalar("SELECT runtime_minutes FROM content WHERE id = $1")
        .bind(content_id_2)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch runtime");

    assert_eq!(runtime, 140);

    // Cleanup
    cleanup_test_content(&pool, content_id_2).await;
}

#[tokio::test]
#[ignore]
async fn test_upsert_batch_atomic() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    // Create batch of test content
    let contents = vec![
        create_test_content("Batch Movie 1", "netflix", "batch_001"),
        create_test_content("Batch Movie 2", "netflix", "batch_002"),
        create_test_content("Batch Movie 3", "netflix", "batch_003"),
        create_test_content("Batch Movie 4", "netflix", "batch_004"),
        create_test_content("Batch Movie 5", "netflix", "batch_005"),
    ];

    // Upsert batch
    let content_ids = repo
        .upsert_batch(&contents)
        .await
        .expect("Failed to upsert batch");

    assert_eq!(content_ids.len(), 5);

    // Verify all were inserted
    for (i, content_id) in content_ids.iter().enumerate() {
        let title: String = sqlx::query_scalar("SELECT title FROM content WHERE id = $1")
            .bind(content_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch title");

        assert_eq!(title, format!("Batch Movie {}", i + 1));
    }

    // Cleanup
    for content_id in content_ids {
        cleanup_test_content(&pool, content_id).await;
    }
}

#[tokio::test]
#[ignore]
async fn test_upsert_batch_large() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    // Create 25 items to test batching (should be 3 batches: 10, 10, 5)
    let mut contents = Vec::new();
    for i in 1..=25 {
        let mut content = create_test_content(
            &format!("Large Batch Movie {}", i),
            "netflix",
            &format!("large_batch_{:03}", i),
        );
        // Use unique IMDB IDs to ensure they're all new records
        content
            .external_ids
            .insert("imdb".to_string(), format!("tt{:07}", 8000000 + i));
        contents.push(content);
    }

    // Upsert batch
    let content_ids = repo
        .upsert_batch(&contents)
        .await
        .expect("Failed to upsert large batch");

    assert_eq!(content_ids.len(), 25);

    // Verify all were inserted
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM content WHERE id = ANY($1)")
        .bind(&content_ids)
        .fetch_one(&pool)
        .await
        .expect("Failed to count inserted content");

    assert_eq!(count, 25);

    // Cleanup
    for content_id in content_ids {
        cleanup_test_content(&pool, content_id).await;
    }
}

#[tokio::test]
#[ignore]
async fn test_upsert_batch_with_updates() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    // Insert initial batch
    let initial_contents = vec![
        create_test_content_with_eidr("Batch Update 1", "10.5240/1111-1111-1111-1111"),
        create_test_content_with_eidr("Batch Update 2", "10.5240/2222-2222-2222-2222"),
    ];

    let initial_ids = repo
        .upsert_batch(&initial_contents)
        .await
        .expect("Failed to insert initial batch");

    // Create updated batch with same EIDRs
    let mut updated_contents = vec![
        create_test_content_with_eidr("Batch Update 1 - Modified", "10.5240/1111-1111-1111-1111"),
        create_test_content_with_eidr("Batch Update 2 - Modified", "10.5240/2222-2222-2222-2222"),
    ];
    updated_contents[0].runtime_minutes = Some(200);
    updated_contents[1].runtime_minutes = Some(210);

    let updated_ids = repo
        .upsert_batch(&updated_contents)
        .await
        .expect("Failed to upsert updated batch");

    // IDs should match (updates, not inserts)
    assert_eq!(initial_ids, updated_ids);

    // Verify updates
    let runtime1: i32 = sqlx::query_scalar("SELECT runtime_minutes FROM content WHERE id = $1")
        .bind(updated_ids[0])
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch runtime");

    assert_eq!(runtime1, 200);

    // Verify no duplicate records were created
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM external_ids WHERE eidr_id IN ($1, $2)")
            .bind("10.5240/1111-1111-1111-1111")
            .bind("10.5240/2222-2222-2222-2222")
            .fetch_one(&pool)
            .await
            .expect("Failed to count EIDR records");

    assert_eq!(count, 2);

    // Cleanup
    for content_id in updated_ids {
        cleanup_test_content(&pool, content_id).await;
    }
}

#[tokio::test]
#[ignore]
async fn test_genre_updates() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    let mut content1 = create_test_content("Genre Test", "netflix", "genre_test_001");
    content1
        .external_ids
        .insert("imdb".to_string(), "tt7777777".to_string());
    content1.genres = vec!["Action".to_string(), "Thriller".to_string()];

    let content_id = repo.upsert(&content1).await.expect("Failed to insert");

    // Verify initial genres
    let genre_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM content_genres WHERE content_id = $1")
            .bind(content_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count genres");

    assert_eq!(genre_count, 2);

    // Update with different genres
    let mut content2 = create_test_content("Genre Test", "netflix", "genre_test_002");
    content2
        .external_ids
        .insert("imdb".to_string(), "tt7777777".to_string());
    content2.genres = vec![
        "Drama".to_string(),
        "Romance".to_string(),
        "Comedy".to_string(),
    ];

    repo.upsert(&content2)
        .await
        .expect("Failed to update genres");

    // Verify updated genres
    let updated_genre_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM content_genres WHERE content_id = $1")
            .bind(content_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count updated genres");

    assert_eq!(updated_genre_count, 3);

    let genres: Vec<String> =
        sqlx::query_scalar("SELECT genre FROM content_genres WHERE content_id = $1 ORDER BY genre")
            .bind(content_id)
            .fetch_all(&pool)
            .await
            .expect("Failed to fetch genres");

    assert_eq!(genres, vec!["Comedy", "Drama", "Romance"]);

    // Cleanup
    cleanup_test_content(&pool, content_id).await;
}

#[tokio::test]
#[ignore]
async fn test_platform_availability_updates() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    let mut content1 = create_test_content("Availability Test", "netflix", "avail_test_001");
    content1
        .external_ids
        .insert("imdb".to_string(), "tt6666666".to_string());
    content1.availability.regions = vec!["US".to_string()];
    content1.availability.purchase_price = Some(9.99);

    let content_id = repo.upsert(&content1).await.expect("Failed to insert");

    // Verify initial availability
    let avail_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM platform_availability WHERE content_id = $1")
            .bind(content_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count availability");

    assert_eq!(avail_count, 1);

    // Update availability - add more regions
    let mut content2 = create_test_content("Availability Test", "netflix", "avail_test_002");
    content2
        .external_ids
        .insert("imdb".to_string(), "tt6666666".to_string());
    content2.availability.regions = vec!["US".to_string(), "CA".to_string(), "UK".to_string()];
    content2.availability.purchase_price = Some(12.99);

    repo.upsert(&content2)
        .await
        .expect("Failed to update availability");

    // Verify updated availability
    let updated_avail_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM platform_availability WHERE content_id = $1")
            .bind(content_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count updated availability");

    assert_eq!(updated_avail_count, 3);

    // Verify price was updated for US
    let price: i32 = sqlx::query_scalar(
        "SELECT price_cents FROM platform_availability
         WHERE content_id = $1 AND region = 'US'",
    )
    .bind(content_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch price");

    assert_eq!(price, 1299);

    // Cleanup
    cleanup_test_content(&pool, content_id).await;
}

/// Cleanup helper to remove test content and all related records
async fn cleanup_test_content(pool: &sqlx::PgPool, content_id: Uuid) {
    // Foreign key constraints with ON DELETE CASCADE should handle cleanup
    sqlx::query("DELETE FROM content WHERE id = $1")
        .bind(content_id)
        .execute(pool)
        .await
        .expect("Failed to cleanup test content");
}
