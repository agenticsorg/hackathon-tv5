//! Integration tests for Collaborative Filtering Engine
//!
//! Tests the complete ALS-based collaborative filtering pipeline with real database.

use anyhow::Result;
use media_gateway_sona::{
    ALSConfig, CollaborativeFilteringEngine, Interaction, InteractionType, MatrixFactorization,
};
use qdrant_client::prelude::*;
use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

async fn setup_test_db() -> Result<sqlx::PgPool> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Create test schema if needed
    sqlx::query(
        r#"
        CREATE SCHEMA IF NOT EXISTS users
        "#,
    )
    .execute(&pool)
    .await
    .ok();

    // Create interactions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users.interactions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL,
            content_id UUID NOT NULL,
            interaction_type VARCHAR(50) NOT NULL,
            watch_progress DOUBLE PRECISION,
            rating DOUBLE PRECISION,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

async fn setup_qdrant() -> Result<QdrantClient> {
    let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let client = QdrantClient::from_url(&qdrant_url).build()?;
    Ok(client)
}

async fn cleanup_test_data(pool: &sqlx::PgPool) -> Result<()> {
    sqlx::query("DELETE FROM users.interactions")
        .execute(pool)
        .await?;
    Ok(())
}

async fn insert_test_interactions(pool: &sqlx::PgPool) -> Result<(Uuid, Uuid, Uuid, Uuid, Uuid)> {
    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();
    let item1 = Uuid::new_v4();
    let item2 = Uuid::new_v4();
    let item3 = Uuid::new_v4();

    // User 1 likes item 1 and 2
    sqlx::query(
        r#"
        INSERT INTO users.interactions (user_id, content_id, interaction_type, watch_progress, rating)
        VALUES
            ($1, $2, 'watch', 0.95, NULL),
            ($1, $3, 'like', NULL, NULL)
        "#,
    )
    .bind(user1)
    .bind(item1)
    .bind(item2)
    .execute(pool)
    .await?;

    // User 2 likes item 1 and 3
    sqlx::query(
        r#"
        INSERT INTO users.interactions (user_id, content_id, interaction_type, watch_progress, rating)
        VALUES
            ($1, $2, 'watch', 0.95, NULL),
            ($1, $3, 'rate', NULL, 5.0)
        "#,
    )
    .bind(user2)
    .bind(item1)
    .bind(item3)
    .execute(pool)
    .await?;

    Ok((user1, user2, item1, item2, item3))
}

#[tokio::test]
#[ignore] // Requires database and Qdrant
async fn test_collaborative_filtering_engine_training() -> Result<()> {
    let pool = setup_test_db().await?;
    let qdrant = setup_qdrant().await?;

    cleanup_test_data(&pool).await?;
    let (_user1, _user2, _item1, _item2, _item3) = insert_test_interactions(&pool).await?;

    let mut engine = CollaborativeFilteringEngine::new(pool.clone(), qdrant);
    engine.initialize_collections().await?;

    // Train model
    engine.train_model().await?;

    // Verify model was trained
    assert!(engine.model.is_some());

    cleanup_test_data(&pool).await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires database and Qdrant
async fn test_collaborative_filtering_recommendations() -> Result<()> {
    let pool = setup_test_db().await?;
    let qdrant = setup_qdrant().await?;

    cleanup_test_data(&pool).await?;
    let (user1, user2, item1, item2, item3) = insert_test_interactions(&pool).await?;

    let mut engine = CollaborativeFilteringEngine::new(pool.clone(), qdrant);
    engine.initialize_collections().await?;
    engine.train_model().await?;

    // User 1 should get item 3 recommended (similar to user 2 who liked item 3)
    let recommendations = engine.recommend(user1, 5).await?;

    // Should recommend unseen items
    assert!(!recommendations.is_empty());

    // Should not recommend already seen items
    let recommended_ids: Vec<Uuid> = recommendations.iter().map(|(id, _)| *id).collect();
    assert!(!recommended_ids.contains(&item1)); // Already watched
    assert!(!recommended_ids.contains(&item2)); // Already liked

    cleanup_test_data(&pool).await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires database and Qdrant
async fn test_user_similarity_computation() -> Result<()> {
    let pool = setup_test_db().await?;
    let qdrant = setup_qdrant().await?;

    cleanup_test_data(&pool).await?;
    let (user1, user2, _item1, _item2, _item3) = insert_test_interactions(&pool).await?;

    let mut engine = CollaborativeFilteringEngine::new(pool.clone(), qdrant);
    engine.initialize_collections().await?;
    engine.train_model().await?;

    // Find similar users to user 1
    let similar_users = engine.compute_user_similarity(user1, 5).await?;

    // User 2 should be similar (both liked item 1)
    assert!(!similar_users.is_empty());

    cleanup_test_data(&pool).await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires database and Qdrant
async fn test_item_similarity_computation() -> Result<()> {
    let pool = setup_test_db().await?;
    let qdrant = setup_qdrant().await?;

    cleanup_test_data(&pool).await?;
    let (_user1, _user2, item1, _item2, _item3) = insert_test_interactions(&pool).await?;

    let mut engine = CollaborativeFilteringEngine::new(pool.clone(), qdrant);
    engine.initialize_collections().await?;
    engine.train_model().await?;

    // Find similar items to item 1
    let similar_items = engine.compute_item_similarity(item1, 5).await?;

    // Should find similar items
    assert!(!similar_items.is_empty());

    cleanup_test_data(&pool).await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires database and Qdrant
async fn test_also_watched_recommendations() -> Result<()> {
    let pool = setup_test_db().await?;
    let qdrant = setup_qdrant().await?;

    cleanup_test_data(&pool).await?;
    let (_user1, _user2, item1, _item2, _item3) = insert_test_interactions(&pool).await?;

    let mut engine = CollaborativeFilteringEngine::new(pool.clone(), qdrant);
    engine.initialize_collections().await?;
    engine.train_model().await?;

    // Get "also watched" for item 1
    let also_watched = engine.get_also_watched(item1, 5).await?;

    // Should return similar items
    assert!(!also_watched.is_empty());

    cleanup_test_data(&pool).await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires database and Qdrant
async fn test_incremental_update() -> Result<()> {
    let pool = setup_test_db().await?;
    let qdrant = setup_qdrant().await?;

    cleanup_test_data(&pool).await?;

    let mut engine = CollaborativeFilteringEngine::new(pool.clone(), qdrant);
    engine.initialize_collections().await?;

    // Add interactions to buffer
    for _ in 0..10 {
        let interaction = Interaction {
            user_id: Uuid::new_v4(),
            content_id: Uuid::new_v4(),
            interaction_type: InteractionType::Like,
            watch_progress: None,
            timestamp: chrono::Utc::now(),
        };
        engine.add_interaction(interaction);
    }

    assert_eq!(engine.incremental_buffer.len(), 10);

    // Should not update yet (buffer < 1000)
    let updated = engine.incremental_update().await?;
    assert!(!updated);

    cleanup_test_data(&pool).await?;
    Ok(())
}

#[test]
fn test_matrix_factorization_basic() -> Result<()> {
    let mut mf = MatrixFactorization::new(ALSConfig {
        latent_factors: 8,
        regularization: 0.1,
        iterations: 10,
        alpha: 40.0,
    });

    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();
    let user3 = Uuid::new_v4();
    let item1 = Uuid::new_v4();
    let item2 = Uuid::new_v4();
    let item3 = Uuid::new_v4();

    let interactions = vec![
        (user1, item1, 1.0),
        (user1, item2, 1.0),
        (user2, item1, 1.0),
        (user2, item3, 1.0),
        (user3, item2, 1.0),
        (user3, item3, 1.0),
    ];

    let matrix = mf.build_matrix(interactions)?;
    mf.fit(&matrix)?;

    // Verify embeddings exist
    let user1_emb = mf.get_user_embedding(user1)?;
    let item1_emb = mf.get_item_embedding(item1)?;

    assert_eq!(user1_emb.len(), 8);
    assert_eq!(item1_emb.len(), 8);

    // Predict known interaction
    let pred = mf.predict(user1, item1)?;
    assert!(pred > 0.0);

    Ok(())
}

#[test]
fn test_als_convergence() -> Result<()> {
    let mut mf = MatrixFactorization::new(ALSConfig {
        latent_factors: 16,
        regularization: 0.05,
        iterations: 20,
        alpha: 40.0,
    });

    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();
    let item1 = Uuid::new_v4();
    let item2 = Uuid::new_v4();

    // Simple pattern: user1 likes item1, user2 likes item2
    let interactions = vec![(user1, item1, 1.0), (user2, item2, 1.0)];

    let matrix = mf.build_matrix(interactions)?;
    mf.fit(&matrix)?;

    // After training, predictions for known interactions should be positive
    let pred1 = mf.predict(user1, item1)?;
    let pred2 = mf.predict(user2, item2)?;

    assert!(pred1 > 0.0);
    assert!(pred2 > 0.0);

    Ok(())
}

#[test]
fn test_interaction_type_rating_conversion() {
    assert_eq!(InteractionType::Like.to_rating(None), 1.0);
    assert_eq!(InteractionType::Dislike.to_rating(None), 0.0);
    assert_eq!(InteractionType::Completion.to_rating(None), 1.0);
    assert_eq!(InteractionType::Rating(5.0).to_rating(None), 1.0);
    assert_eq!(InteractionType::Rating(3.0).to_rating(None), 0.6);

    // View with progress
    assert_eq!(InteractionType::View.to_rating(Some(0.95)), 1.0);
    assert_eq!(InteractionType::View.to_rating(Some(0.7)), 0.5);
    assert_eq!(InteractionType::View.to_rating(Some(0.3)), 0.2);
    assert_eq!(InteractionType::View.to_rating(None), 0.2);
}

#[test]
fn test_cosine_similarity() {
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    assert!((MatrixFactorization::cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

    let c = vec![1.0, 0.0];
    let d = vec![0.0, 1.0];
    assert!((MatrixFactorization::cosine_similarity(&c, &d) - 0.0).abs() < 1e-6);

    let e = vec![1.0, 1.0];
    let f = vec![1.0, 1.0];
    assert!((MatrixFactorization::cosine_similarity(&e, &f) - 1.0).abs() < 1e-6);

    // Negative similarity
    let g = vec![1.0, 0.0];
    let h = vec![-1.0, 0.0];
    assert!((MatrixFactorization::cosine_similarity(&g, &h) - (-1.0)).abs() < 1e-6);
}
