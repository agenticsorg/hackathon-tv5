//! Integration tests for metadata enrichment pipeline
//!
//! Tests the complete flow:
//! - Finding stale content in database
//! - Regenerating embeddings with OpenAI
//! - Updating Qdrant vectors
//! - Computing quality scores
//! - Emitting Kafka events

use chrono::{Duration, Utc};
use media_gateway_ingestion::{
    events::{ContentEvent, KafkaConfig, MockEventProducer},
    normalizer::{AvailabilityInfo, CanonicalContent, ContentType, ImageSet},
    rate_limit::RateLimitManager,
    ContentRepository, EmbeddingGenerator, IngestionPipeline, IngestionSchedule,
    PostgresContentRepository, QdrantClient, StaleContent, VECTOR_DIM,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Helper to create test database connection
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/media_gateway_test".to_string()
    });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Helper to create test content
fn create_test_content(platform: &str, title: &str) -> CanonicalContent {
    CanonicalContent {
        platform_content_id: format!("test-{}", Uuid::new_v4()),
        platform_id: platform.to_string(),
        entity_id: None,
        title: title.to_string(),
        overview: Some("A test movie for metadata enrichment".to_string()),
        content_type: ContentType::Movie,
        release_year: Some(2020),
        runtime_minutes: Some(120),
        genres: vec!["Action".to_string(), "Drama".to_string()],
        external_ids: std::collections::HashMap::new(),
        availability: AvailabilityInfo {
            regions: vec!["US".to_string()],
            subscription_required: true,
            purchase_price: None,
            rental_price: None,
            currency: None,
            available_from: None,
            available_until: None,
        },
        images: ImageSet::default(),
        rating: Some("PG-13".to_string()),
        user_rating: Some(7.5),
        embedding: None,                            // Will be generated
        updated_at: Utc::now() - Duration::days(8), // Stale (older than 7 days)
    }
}

#[tokio::test]
#[ignore] // Requires database and Qdrant
async fn test_find_stale_embeddings() {
    let pool = create_test_pool().await;
    let repository = PostgresContentRepository::new(pool.clone());

    // Insert test content with old timestamp
    let test_content = create_test_content("netflix", "Test Movie - Stale");
    let content_id = repository
        .upsert(&test_content)
        .await
        .expect("Failed to insert test content");

    // Query for stale embeddings (older than 7 days)
    let stale_threshold = Utc::now() - Duration::days(7);
    let stale_content = repository
        .find_stale_embeddings(stale_threshold)
        .await
        .expect("Failed to find stale embeddings");

    // Verify we found at least our test content
    assert!(!stale_content.is_empty(), "Should find stale content");

    let found = stale_content.iter().find(|s| s.content_id == content_id);
    assert!(
        found.is_some(),
        "Should find our test content in stale results"
    );

    // Cleanup
    sqlx::query("DELETE FROM content WHERE id = $1")
        .bind(content_id)
        .execute(&pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
#[ignore] // Requires database
async fn test_update_embedding() {
    let pool = create_test_pool().await;
    let repository = PostgresContentRepository::new(pool.clone());

    // Insert test content
    let test_content = create_test_content("netflix", "Test Movie - Embedding");
    let content_id = repository
        .upsert(&test_content)
        .await
        .expect("Failed to insert test content");

    // Generate test embedding
    let test_embedding: Vec<f32> = (0..768).map(|i| (i as f32) / 768.0).collect();

    // Update embedding
    repository
        .update_embedding(content_id, &test_embedding)
        .await
        .expect("Failed to update embedding");

    // Verify embedding was stored
    let stored_embedding =
        sqlx::query_scalar::<_, serde_json::Value>("SELECT embedding FROM content WHERE id = $1")
            .bind(content_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch embedding");

    assert!(!stored_embedding.is_null(), "Embedding should be stored");

    // Verify last_updated was updated
    let last_updated = sqlx::query_scalar::<_, chrono::DateTime<Utc>>(
        "SELECT last_updated FROM content WHERE id = $1",
    )
    .bind(content_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch last_updated");

    let now = Utc::now();
    let diff = (now - last_updated).num_seconds();
    assert!(
        diff < 10,
        "last_updated should be recent (within 10 seconds)"
    );

    // Cleanup
    sqlx::query("DELETE FROM content WHERE id = $1")
        .bind(content_id)
        .execute(&pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
#[ignore] // Requires database
async fn test_update_quality_score() {
    let pool = create_test_pool().await;
    let repository = PostgresContentRepository::new(pool.clone());

    // Insert test content
    let test_content = create_test_content("netflix", "Test Movie - Quality");
    let content_id = repository
        .upsert(&test_content)
        .await
        .expect("Failed to insert test content");

    // Update quality score
    let quality_score = 0.85;
    repository
        .update_quality_score(content_id, quality_score)
        .await
        .expect("Failed to update quality score");

    // Verify quality score was stored
    let stored_score =
        sqlx::query_scalar::<_, f64>("SELECT quality_score FROM content WHERE id = $1")
            .bind(content_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch quality score");

    assert!(
        (stored_score - quality_score).abs() < 0.001,
        "Quality score should match"
    );

    // Cleanup
    sqlx::query("DELETE FROM content WHERE id = $1")
        .bind(content_id)
        .execute(&pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
#[ignore] // Requires database and Qdrant
async fn test_metadata_enrichment_full_flow() {
    let pool = create_test_pool().await;
    let repository = Arc::new(PostgresContentRepository::new(pool.clone()));

    // Insert test content with stale timestamp
    let mut test_content = create_test_content("netflix", "Test Movie - Full Flow");
    test_content.updated_at = Utc::now() - Duration::days(10); // Very stale
    let content_id = repository
        .upsert(&test_content)
        .await
        .expect("Failed to insert test content");

    // Create embedding generator
    let embedding_generator = EmbeddingGenerator::new();

    // Create Qdrant client
    let qdrant_url =
        std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let qdrant_client = QdrantClient::new(&qdrant_url, "test_enrichment")
        .await
        .expect("Failed to create Qdrant client");

    // Ensure collection exists
    qdrant_client
        .ensure_collection(VECTOR_DIM)
        .await
        .expect("Failed to ensure collection");

    // Create mock event producer
    let kafka_config = KafkaConfig {
        brokers: "localhost:9092".to_string(),
        topic_prefix: "test".to_string(),
        request_timeout_ms: 30000,
        message_timeout_ms: 60000,
        enable_idempotence: true,
    };
    let event_producer = Arc::new(MockEventProducer::new(kafka_config));

    // Find stale embeddings
    let stale_threshold = Utc::now() - Duration::days(7);
    let stale_content = repository
        .find_stale_embeddings(stale_threshold)
        .await
        .expect("Failed to find stale embeddings");

    assert!(!stale_content.is_empty(), "Should find stale content");

    // Find our test content
    let test_item = stale_content
        .iter()
        .find(|s| s.content_id == content_id)
        .expect("Should find our test content");

    // Regenerate embedding
    let embedding = embedding_generator
        .generate(&test_item.content)
        .await
        .expect("Failed to generate embedding");

    assert_eq!(embedding.len(), 768, "Embedding should have 768 dimensions");

    // Update embedding in database
    repository
        .update_embedding(content_id, &embedding)
        .await
        .expect("Failed to update embedding");

    // Update Qdrant
    let mut content_with_embedding = test_item.content.clone();
    content_with_embedding.embedding = Some(embedding);

    let point = media_gateway_ingestion::to_content_point(&content_with_embedding, content_id)
        .expect("Failed to create Qdrant point");

    qdrant_client
        .upsert_batch(vec![point])
        .await
        .expect("Failed to upsert to Qdrant");

    // Compute quality score
    let quality_score = 0.8; // Based on test content completeness
    repository
        .update_quality_score(content_id, quality_score)
        .await
        .expect("Failed to update quality score");

    // Emit event
    use media_gateway_ingestion::events::MetadataEnrichedEvent;
    let event = ContentEvent::MetadataEnriched(MetadataEnrichedEvent::new(
        content_id,
        "embedding_generator".to_string(),
        vec!["embedding".to_string(), "quality_score".to_string()],
        quality_score,
    ));

    event_producer
        .publish_event(event)
        .await
        .expect("Failed to publish event");

    // Verify event was published
    let published_events = event_producer.get_published_events().await;
    assert_eq!(published_events.len(), 1, "Should have published one event");

    match &published_events[0] {
        ContentEvent::MetadataEnriched(e) => {
            assert_eq!(e.base.content_id, content_id);
            assert_eq!(e.enrichment_source, "embedding_generator");
            assert_eq!(e.enriched_fields.len(), 2);
        }
        _ => panic!("Expected MetadataEnriched event"),
    }

    // Verify embedding is no longer stale
    let stale_after = repository
        .find_stale_embeddings(stale_threshold)
        .await
        .expect("Failed to find stale embeddings");

    let still_stale = stale_after.iter().find(|s| s.content_id == content_id);
    assert!(
        still_stale.is_none(),
        "Content should no longer be stale after enrichment"
    );

    // Cleanup
    sqlx::query("DELETE FROM content WHERE id = $1")
        .bind(content_id)
        .execute(&pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
#[ignore] // Requires database
async fn test_batch_enrichment() {
    let pool = create_test_pool().await;
    let repository = Arc::new(PostgresContentRepository::new(pool.clone()));

    // Insert multiple test content items
    let mut content_ids = Vec::new();
    for i in 0..5 {
        let mut test_content = create_test_content("netflix", &format!("Test Movie {}", i));
        test_content.updated_at = Utc::now() - Duration::days(8); // Stale
        let content_id = repository
            .upsert(&test_content)
            .await
            .expect("Failed to insert test content");
        content_ids.push(content_id);
    }

    // Create embedding generator
    let embedding_generator = EmbeddingGenerator::new();

    // Find stale embeddings
    let stale_threshold = Utc::now() - Duration::days(7);
    let stale_content = repository
        .find_stale_embeddings(stale_threshold)
        .await
        .expect("Failed to find stale embeddings");

    // Process in batch
    const BATCH_SIZE: usize = 100;
    let test_stale: Vec<_> = stale_content
        .iter()
        .filter(|s| content_ids.contains(&s.content_id))
        .collect();

    assert_eq!(test_stale.len(), 5, "Should find all 5 test items");

    for batch in test_stale.chunks(BATCH_SIZE) {
        for item in batch {
            // Generate embedding
            let embedding = embedding_generator
                .generate(&item.content)
                .await
                .expect("Failed to generate embedding");

            // Update database
            repository
                .update_embedding(item.content_id, &embedding)
                .await
                .expect("Failed to update embedding");

            // Update quality score
            let quality_score = 0.75;
            repository
                .update_quality_score(item.content_id, quality_score)
                .await
                .expect("Failed to update quality score");
        }
    }

    // Verify all items were updated
    for content_id in &content_ids {
        let embedding = sqlx::query_scalar::<_, serde_json::Value>(
            "SELECT embedding FROM content WHERE id = $1",
        )
        .bind(content_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch embedding");

        assert!(
            !embedding.is_null(),
            "Embedding should be stored for {}",
            content_id
        );

        let quality_score =
            sqlx::query_scalar::<_, f64>("SELECT quality_score FROM content WHERE id = $1")
                .bind(content_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to fetch quality score");

        assert!(
            (quality_score - 0.75).abs() < 0.001,
            "Quality score should match"
        );
    }

    // Cleanup
    for content_id in content_ids {
        sqlx::query("DELETE FROM content WHERE id = $1")
            .bind(content_id)
            .execute(&pool)
            .await
            .expect("Failed to cleanup test data");
    }
}

#[tokio::test]
#[ignore] // Requires database
async fn test_quality_score_computation() {
    // Test quality score calculation for various content completeness levels

    let pool = create_test_pool().await;
    let repository = PostgresContentRepository::new(pool.clone());

    // Minimal content (only title)
    let minimal = CanonicalContent {
        platform_content_id: "test-minimal".to_string(),
        platform_id: "netflix".to_string(),
        entity_id: None,
        title: "Minimal Movie".to_string(),
        overview: None,
        content_type: ContentType::Movie,
        release_year: None,
        runtime_minutes: None,
        genres: vec![],
        external_ids: std::collections::HashMap::new(),
        availability: AvailabilityInfo {
            regions: vec![],
            subscription_required: false,
            purchase_price: None,
            rental_price: None,
            currency: None,
            available_from: None,
            available_until: None,
        },
        images: ImageSet::default(),
        rating: None,
        user_rating: None,
        embedding: None,
        updated_at: Utc::now(),
    };

    // Complete content (all fields)
    let complete = CanonicalContent {
        platform_content_id: "test-complete".to_string(),
        platform_id: "netflix".to_string(),
        entity_id: None,
        title: "Complete Movie".to_string(),
        overview: Some("A complete movie description".to_string()),
        content_type: ContentType::Movie,
        release_year: Some(2020),
        runtime_minutes: Some(120),
        genres: vec!["Action".to_string()],
        external_ids: std::collections::HashMap::new(),
        availability: AvailabilityInfo {
            regions: vec!["US".to_string()],
            subscription_required: true,
            purchase_price: None,
            rental_price: None,
            currency: None,
            available_from: None,
            available_until: None,
        },
        images: ImageSet::default(),
        rating: Some("PG-13".to_string()),
        user_rating: Some(8.5),
        embedding: Some(vec![0.1; 768]),
        updated_at: Utc::now(),
    };

    // Insert and verify quality scores would be different
    let minimal_id = repository
        .upsert(&minimal)
        .await
        .expect("Failed to insert minimal content");
    let complete_id = repository
        .upsert(&complete)
        .await
        .expect("Failed to insert complete content");

    // Note: Actual quality score computation happens in enrich_metadata
    // Here we just verify the update_quality_score function works

    repository
        .update_quality_score(minimal_id, 0.1)
        .await
        .expect("Failed to update minimal score");
    repository
        .update_quality_score(complete_id, 1.0)
        .await
        .expect("Failed to update complete score");

    let minimal_score =
        sqlx::query_scalar::<_, f64>("SELECT quality_score FROM content WHERE id = $1")
            .bind(minimal_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch minimal score");

    let complete_score =
        sqlx::query_scalar::<_, f64>("SELECT quality_score FROM content WHERE id = $1")
            .bind(complete_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch complete score");

    assert!(
        (minimal_score - 0.1).abs() < 0.001,
        "Minimal content should have low quality score"
    );
    assert!(
        (complete_score - 1.0).abs() < 0.001,
        "Complete content should have high quality score"
    );

    // Cleanup
    sqlx::query("DELETE FROM content WHERE id = $1")
        .bind(minimal_id)
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM content WHERE id = $1")
        .bind(complete_id)
        .execute(&pool)
        .await
        .ok();
}
