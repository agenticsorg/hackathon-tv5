//! Example demonstrating Qdrant integration with ingestion pipeline
//!
//! This example shows how to:
//! 1. Initialize a Qdrant client
//! 2. Create a collection for content vectors
//! 3. Process content through the pipeline with automatic vector indexing
//! 4. Search for similar content using vector similarity
//!
//! Run with:
//! ```bash
//! # Start Qdrant
//! docker run -p 6334:6334 qdrant/qdrant
//!
//! # Run example
//! cargo run --example qdrant_usage
//! ```

use media_gateway_ingestion::{
    normalizer::{AvailabilityInfo, CanonicalContent, ContentType, ImageSet},
    qdrant::{to_content_point, ContentPayload, QdrantClient, VECTOR_DIM},
    EmbeddingGenerator,
};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Qdrant Vector Indexing Example ===\n");

    // 1. Connect to Qdrant
    println!("1. Connecting to Qdrant at http://localhost:6334...");
    let qdrant_client = QdrantClient::new("http://localhost:6334", "content_vectors").await?;
    println!("   ✓ Connected successfully\n");

    // 2. Health check
    println!("2. Performing health check...");
    let healthy = qdrant_client.health_check().await?;
    println!("   ✓ Qdrant is healthy: {}\n", healthy);

    // 3. Ensure collection exists
    println!(
        "3. Creating/verifying collection with {} dimensions...",
        VECTOR_DIM
    );
    qdrant_client.ensure_collection(VECTOR_DIM).await?;
    println!("   ✓ Collection ready\n");

    // 4. Generate embeddings for sample content
    println!("4. Generating embeddings for sample content...");
    let embedding_generator = EmbeddingGenerator::new();

    let sample_content = vec![
        create_sample_content(
            "The Matrix",
            vec!["Action", "Science Fiction"],
            "A hacker discovers reality is a simulation",
            8.7,
        ),
        create_sample_content(
            "Inception",
            vec!["Action", "Thriller", "Science Fiction"],
            "A thief who steals corporate secrets through dreams",
            8.8,
        ),
        create_sample_content(
            "The Godfather",
            vec!["Crime", "Drama"],
            "The aging patriarch of an organized crime dynasty",
            9.2,
        ),
        create_sample_content(
            "Pulp Fiction",
            vec!["Crime", "Drama"],
            "The lives of two mob hitmen, a boxer, and others intertwine",
            8.9,
        ),
        create_sample_content(
            "The Dark Knight",
            vec!["Action", "Crime", "Drama"],
            "Batman faces the Joker in a battle for Gotham's soul",
            9.0,
        ),
    ];

    let mut content_with_embeddings = Vec::new();
    for mut content in sample_content {
        let embedding = embedding_generator.generate(&content).await?;
        content.embedding = Some(embedding);
        content_with_embeddings.push(content);
    }
    println!(
        "   ✓ Generated {} embeddings\n",
        content_with_embeddings.len()
    );

    // 5. Index content in Qdrant
    println!("5. Indexing content in Qdrant (batch upsert)...");
    let mut points = Vec::new();
    let mut content_ids = Vec::new();

    for content in &content_with_embeddings {
        let content_id = Uuid::new_v4();
        content_ids.push(content_id);
        let point = to_content_point(content, content_id)?;
        points.push(point);
        println!("   - {} (ID: {})", content.title, content_id);
    }

    qdrant_client.upsert_batch(points).await?;
    println!("   ✓ Indexed {} items\n", content_ids.len());

    // 6. Perform similarity search
    println!("6. Performing similarity search...");
    println!("   Query: Find movies similar to 'The Matrix'\n");

    let query_embedding = content_with_embeddings[0].embedding.as_ref().unwrap();
    let results = qdrant_client
        .search_similar(query_embedding.clone(), 3)
        .await?;

    println!("   Top 3 similar movies:");
    for (i, (id, score)) in results.iter().enumerate() {
        println!(
            "   {}. Content ID: {} (similarity: {:.4})",
            i + 1,
            id,
            score
        );
    }
    println!();

    // 7. Search for action movies
    println!("7. Searching for action/sci-fi content...");
    let inception_embedding = content_with_embeddings[1].embedding.as_ref().unwrap();
    let action_results = qdrant_client
        .search_similar(inception_embedding.clone(), 3)
        .await?;

    println!("   Top 3 matches:");
    for (i, (id, score)) in action_results.iter().enumerate() {
        println!(
            "   {}. Content ID: {} (similarity: {:.4})",
            i + 1,
            id,
            score
        );
    }
    println!();

    // 8. Demonstrate update operation
    println!("8. Demonstrating update (upsert with same ID)...");
    let update_id = content_ids[0];
    let mut updated_content = content_with_embeddings[0].clone();
    updated_content.title = "The Matrix Reloaded".to_string();
    updated_content.user_rating = Some(7.2);

    let new_embedding = embedding_generator.generate(&updated_content).await?;
    updated_content.embedding = Some(new_embedding);

    let updated_point = to_content_point(&updated_content, update_id)?;
    qdrant_client
        .upsert_point(
            updated_point.id,
            updated_point.vector,
            updated_point.payload,
        )
        .await?;

    println!("   ✓ Updated content ID {} with new title\n", update_id);

    println!("=== Example Complete ===");
    println!("\nKey features demonstrated:");
    println!("  ✓ Client initialization and health checking");
    println!("  ✓ Collection creation (768-dimensional vectors)");
    println!("  ✓ Batch indexing (up to 100 items per batch)");
    println!("  ✓ Vector similarity search");
    println!("  ✓ Update operations (upsert)");
    println!("\nIntegration with pipeline:");
    println!("  - Embeddings generated via EmbeddingGenerator");
    println!("  - Automatic indexing after DB persistence");
    println!("  - Metadata stored as payload (title, genres, platform, etc.)");

    Ok(())
}

/// Helper function to create sample content
fn create_sample_content(
    title: &str,
    genres: Vec<&str>,
    overview: &str,
    rating: f32,
) -> CanonicalContent {
    CanonicalContent {
        platform_content_id: format!("sample-{}", title.replace(' ', "-").to_lowercase()),
        platform_id: "netflix".to_string(),
        entity_id: None,
        title: title.to_string(),
        overview: Some(overview.to_string()),
        content_type: ContentType::Movie,
        release_year: Some(2024),
        runtime_minutes: Some(120),
        genres: genres.iter().map(|s| s.to_string()).collect(),
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
        embedding: None, // Will be populated later
        updated_at: chrono::Utc::now(),
    }
}
