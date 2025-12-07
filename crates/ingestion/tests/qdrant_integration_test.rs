//! Integration tests for Qdrant vector indexing
//!
//! These tests verify the complete integration of Qdrant with the ingestion pipeline,
//! including collection management, batch operations, and similarity search.
//!
//! Note: These tests require a running Qdrant instance. They can be run with:
//! ```bash
//! docker run -p 6334:6334 qdrant/qdrant
//! cargo test --test qdrant_integration_test -- --ignored
//! ```

use media_gateway_ingestion::{
    normalizer::{AvailabilityInfo, CanonicalContent, ContentType, ImageSet},
    qdrant::{to_content_point, ContentPayload, ContentPoint, QdrantClient, VECTOR_DIM},
};
use std::collections::HashMap;
use uuid::Uuid;

const QDRANT_URL: &str = "http://localhost:6334";
const TEST_COLLECTION: &str = "test_content_vectors";

/// Helper function to create test content with embedding
fn create_test_content(title: &str, genres: Vec<String>, rating: f32) -> CanonicalContent {
    // Create a deterministic embedding based on title
    let mut embedding = vec![0.0; VECTOR_DIM as usize];
    for (i, byte) in title.bytes().enumerate() {
        if i < VECTOR_DIM as usize {
            embedding[i] = (byte as f32) / 255.0;
        }
    }

    // L2 normalize
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        embedding.iter_mut().for_each(|x| *x /= norm);
    }

    CanonicalContent {
        platform_content_id: format!("test-{}", title.replace(' ', "-").to_lowercase()),
        platform_id: "netflix".to_string(),
        entity_id: None,
        title: title.to_string(),
        overview: Some(format!("Test content: {}", title)),
        content_type: ContentType::Movie,
        release_year: Some(2024),
        runtime_minutes: Some(120),
        genres,
        external_ids: HashMap::new(),
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
        user_rating: Some(rating),
        embedding: Some(embedding),
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
#[ignore] // Requires running Qdrant instance
async fn test_qdrant_client_creation() {
    let client = QdrantClient::new(QDRANT_URL, TEST_COLLECTION)
        .await
        .expect("Failed to create Qdrant client");

    // Verify health check
    let healthy = client.health_check().await.expect("Health check failed");
    assert!(healthy, "Qdrant should be healthy");
}

#[tokio::test]
#[ignore] // Requires running Qdrant instance
async fn test_collection_creation() {
    let client = QdrantClient::new(QDRANT_URL, "test_collection_creation")
        .await
        .expect("Failed to create client");

    // Ensure collection is created
    client
        .ensure_collection(VECTOR_DIM)
        .await
        .expect("Failed to create collection");

    // Calling ensure_collection again should not error (idempotent)
    client
        .ensure_collection(VECTOR_DIM)
        .await
        .expect("Failed on second ensure_collection call");
}

#[tokio::test]
#[ignore] // Requires running Qdrant instance
async fn test_single_point_upsert() {
    let client = QdrantClient::new(QDRANT_URL, "test_single_upsert")
        .await
        .expect("Failed to create client");

    client
        .ensure_collection(VECTOR_DIM)
        .await
        .expect("Failed to create collection");

    let content_id = Uuid::new_v4();
    let content = create_test_content(
        "The Matrix",
        vec!["Action".to_string(), "Science Fiction".to_string()],
        8.7,
    );

    let point = to_content_point(&content, content_id).expect("Failed to create point");

    // Upsert single point
    client
        .upsert_point(point.id, point.vector, point.payload)
        .await
        .expect("Failed to upsert point");

    // Verify the point can be searched
    let search_vector = content.embedding.unwrap();
    let results = client
        .search_similar(search_vector, 1)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, content_id);
    assert!(results[0].1 > 0.99); // Should be nearly identical (cosine similarity)
}

#[tokio::test]
#[ignore] // Requires running Qdrant instance
async fn test_batch_upsert() {
    let client = QdrantClient::new(QDRANT_URL, "test_batch_upsert")
        .await
        .expect("Failed to create client");

    client
        .ensure_collection(VECTOR_DIM)
        .await
        .expect("Failed to create collection");

    // Create batch of test content
    let test_data = vec![
        ("The Matrix", vec!["Action", "Sci-Fi"], 8.7),
        ("Inception", vec!["Action", "Thriller"], 8.8),
        ("The Godfather", vec!["Crime", "Drama"], 9.2),
        ("Pulp Fiction", vec!["Crime", "Drama"], 8.9),
        ("The Dark Knight", vec!["Action", "Crime"], 9.0),
    ];

    let mut points = Vec::new();
    for (title, genres, rating) in test_data {
        let content_id = Uuid::new_v4();
        let genres_vec: Vec<String> = genres.iter().map(|s| s.to_string()).collect();
        let content = create_test_content(title, genres_vec, rating);
        let point = to_content_point(&content, content_id).expect("Failed to create point");
        points.push(point);
    }

    // Batch upsert
    client
        .upsert_batch(points.clone())
        .await
        .expect("Failed to upsert batch");

    // Verify all points were indexed
    let search_vector = points[0].vector.clone();
    let results = client
        .search_similar(search_vector, 5)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 5, "Should return all 5 indexed items");
}

#[tokio::test]
#[ignore] // Requires running Qdrant instance
async fn test_similarity_search() {
    let client = QdrantClient::new(QDRANT_URL, "test_similarity_search")
        .await
        .expect("Failed to create client");

    client
        .ensure_collection(VECTOR_DIM)
        .await
        .expect("Failed to create collection");

    // Create action movies
    let action_content_id = Uuid::new_v4();
    let action_content = create_test_content("Action Movie A", vec!["Action".to_string()], 8.0);
    let action_point = to_content_point(&action_content, action_content_id)
        .expect("Failed to create action point");

    // Create drama movies
    let drama_content_id = Uuid::new_v4();
    let drama_content = create_test_content("Drama Movie B", vec!["Drama".to_string()], 8.5);
    let drama_point =
        to_content_point(&drama_content, drama_content_id).expect("Failed to create drama point");

    // Upsert both
    client
        .upsert_batch(vec![action_point.clone(), drama_point])
        .await
        .expect("Failed to upsert");

    // Search with action movie vector
    let results = client
        .search_similar(action_point.vector.clone(), 2)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 2);
    // First result should be the action movie itself (highest similarity)
    assert_eq!(results[0].0, action_content_id);
    assert!(
        results[0].1 > results[1].1,
        "First result should have higher similarity"
    );
}

#[tokio::test]
#[ignore] // Requires running Qdrant instance
async fn test_batch_size_limit() {
    let client = QdrantClient::new(QDRANT_URL, "test_batch_limit")
        .await
        .expect("Failed to create client");

    client
        .ensure_collection(VECTOR_DIM)
        .await
        .expect("Failed to create collection");

    // Create 101 points (exceeds max batch size of 100)
    let mut points = Vec::new();
    for i in 0..101 {
        let content_id = Uuid::new_v4();
        let content = create_test_content(&format!("Movie {}", i), vec!["Action".to_string()], 7.0);
        let point = to_content_point(&content, content_id).expect("Failed to create point");
        points.push(point);
    }

    // Should fail due to batch size limit
    let result = client.upsert_batch(points).await;
    assert!(result.is_err(), "Should fail with batch size > 100");
    assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
}

#[tokio::test]
#[ignore] // Requires running Qdrant instance
async fn test_upsert_updates_existing_point() {
    let client = QdrantClient::new(QDRANT_URL, "test_upsert_update")
        .await
        .expect("Failed to create client");

    client
        .ensure_collection(VECTOR_DIM)
        .await
        .expect("Failed to create collection");

    let content_id = Uuid::new_v4();

    // Initial upsert
    let content_v1 = create_test_content("Original Title", vec!["Action".to_string()], 7.0);
    let point_v1 = to_content_point(&content_v1, content_id).expect("Failed to create point");

    client
        .upsert_point(
            point_v1.id,
            point_v1.vector.clone(),
            point_v1.payload.clone(),
        )
        .await
        .expect("Failed to upsert v1");

    // Update with new data (same ID)
    let content_v2 = create_test_content("Updated Title", vec!["Drama".to_string()], 9.0);
    let point_v2 = to_content_point(&content_v2, content_id).expect("Failed to create point");

    client
        .upsert_point(point_v2.id, point_v2.vector.clone(), point_v2.payload)
        .await
        .expect("Failed to upsert v2");

    // Search should find the updated version
    let results = client
        .search_similar(point_v2.vector, 1)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, content_id);
}

#[tokio::test]
#[ignore] // Requires running Qdrant instance
async fn test_empty_batch_upsert() {
    let client = QdrantClient::new(QDRANT_URL, "test_empty_batch")
        .await
        .expect("Failed to create client");

    client
        .ensure_collection(VECTOR_DIM)
        .await
        .expect("Failed to create collection");

    // Empty batch should not error
    let result = client.upsert_batch(vec![]).await;
    assert!(result.is_ok(), "Empty batch should succeed");
}

#[tokio::test]
#[ignore] // Requires running Qdrant instance
async fn test_content_without_embedding_fails() {
    let content_id = Uuid::new_v4();

    let mut content = create_test_content("No Embedding", vec!["Action".to_string()], 8.0);

    // Remove embedding
    content.embedding = None;

    // Should fail when converting to point
    let result = to_content_point(&content, content_id);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("missing embedding"));
}

#[tokio::test]
#[ignore] // Requires running Qdrant instance
async fn test_search_with_limit() {
    let client = QdrantClient::new(QDRANT_URL, "test_search_limit")
        .await
        .expect("Failed to create client");

    client
        .ensure_collection(VECTOR_DIM)
        .await
        .expect("Failed to create collection");

    // Add 10 movies
    let mut points = Vec::new();
    for i in 0..10 {
        let content_id = Uuid::new_v4();
        let content = create_test_content(&format!("Movie {}", i), vec!["Action".to_string()], 7.5);
        let point = to_content_point(&content, content_id).expect("Failed to create point");
        points.push(point);
    }

    client
        .upsert_batch(points.clone())
        .await
        .expect("Failed to upsert");

    // Search with limit of 3
    let results = client
        .search_similar(points[0].vector.clone(), 3)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 3, "Should respect search limit");

    // Results should be ordered by similarity (descending)
    for i in 1..results.len() {
        assert!(
            results[i - 1].1 >= results[i].1,
            "Results should be ordered by similarity score"
        );
    }
}
