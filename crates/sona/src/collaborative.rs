//! Real-Time Collaborative Filtering Pipeline
//!
//! Implements user-item collaborative filtering using ALS matrix factorization
//! with implicit feedback collection and Qdrant vector storage.

use crate::matrix_factorization::{ALSConfig, MatrixFactorization, SparseMatrix};
use anyhow::{Context, Result};
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::{
    CreateCollection, Distance, PointStruct, SearchPoints, UpsertPointsBuilder,
    Value as QdrantValue, VectorParams, VectorsConfig,
};
use qdrant_client::Payload;
use qdrant_client::Qdrant;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

const USER_EMBEDDINGS_COLLECTION: &str = "user_embeddings";
const ITEM_EMBEDDINGS_COLLECTION: &str = "item_embeddings";
const BATCH_SIZE: usize = 1000;
const SIMILARITY_THRESHOLD: f32 = 0.7;

/// Interaction type for implicit feedback
#[derive(Debug, Clone, Copy)]
pub enum InteractionType {
    View,
    Completion,
    Rating(f32),
    Like,
    Dislike,
}

impl InteractionType {
    /// Convert interaction to implicit rating
    pub fn to_rating(&self, watch_progress: Option<f32>) -> f32 {
        match self {
            InteractionType::View => {
                if let Some(progress) = watch_progress {
                    if progress >= 0.9 {
                        1.0
                    } else if progress >= 0.5 {
                        0.5
                    } else {
                        0.2
                    }
                } else {
                    0.2
                }
            }
            InteractionType::Completion => 1.0,
            InteractionType::Rating(r) => r / 5.0,
            InteractionType::Like => 1.0,
            InteractionType::Dislike => 0.0,
        }
    }
}

/// User interaction record
#[derive(Debug, Clone)]
pub struct Interaction {
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub interaction_type: InteractionType,
    pub watch_progress: Option<f32>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Collaborative Filtering Engine
pub struct CollaborativeFilteringEngine {
    pool: PgPool,
    qdrant: Qdrant,
    als_config: ALSConfig,
    model: Option<MatrixFactorization>,
    incremental_buffer: Vec<Interaction>,
}

impl CollaborativeFilteringEngine {
    pub fn new(pool: PgPool, qdrant: Qdrant) -> Self {
        Self {
            pool,
            qdrant,
            als_config: ALSConfig::default(),
            model: None,
            incremental_buffer: Vec::new(),
        }
    }

    pub fn with_config(mut self, config: ALSConfig) -> Self {
        self.als_config = config;
        self
    }

    /// Initialize Qdrant collections for embeddings
    pub async fn initialize_collections(&self) -> Result<()> {
        let embedding_dim = self.als_config.latent_factors as u64;

        // Create user embeddings collection
        self.qdrant
            .create_collection(CreateCollection {
                collection_name: USER_EMBEDDINGS_COLLECTION.to_string(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: embedding_dim,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            })
            .await
            .ok(); // Ignore if already exists

        // Create item embeddings collection
        self.qdrant
            .create_collection(CreateCollection {
                collection_name: ITEM_EMBEDDINGS_COLLECTION.to_string(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: embedding_dim,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            })
            .await
            .ok(); // Ignore if already exists

        Ok(())
    }

    /// Collect implicit feedback from database
    pub async fn collect_feedback(&self) -> Result<Vec<Interaction>> {
        let rows = sqlx::query(
            r#"
            SELECT
                user_id,
                content_id,
                interaction_type,
                watch_progress,
                rating,
                created_at as timestamp
            FROM users.interactions
            WHERE interaction_type IN ('watch', 'like', 'rate', 'dislike')
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut interactions = Vec::new();

        for row in rows {
            let user_id: Uuid = row.get("user_id");
            let content_id: Uuid = row.get("content_id");
            let interaction_type_str: String = row.get("interaction_type");
            let watch_progress: Option<f64> = row.get("watch_progress");
            let rating: Option<f64> = row.get("rating");
            let timestamp: chrono::DateTime<chrono::Utc> = row.get("timestamp");

            let interaction_type = match interaction_type_str.as_str() {
                "watch" => {
                    if watch_progress.map(|p| p >= 0.9).unwrap_or(false) {
                        InteractionType::Completion
                    } else {
                        InteractionType::View
                    }
                }
                "like" => InteractionType::Like,
                "dislike" => InteractionType::Dislike,
                "rate" => InteractionType::Rating(rating.unwrap_or(0.0) as f32),
                _ => continue,
            };

            interactions.push(Interaction {
                user_id,
                content_id,
                interaction_type,
                watch_progress: watch_progress.map(|p| p as f32),
                timestamp,
            });
        }

        Ok(interactions)
    }

    /// Build user-item matrix from interactions
    pub fn build_user_item_matrix(&self, interactions: &[Interaction]) -> Vec<(Uuid, Uuid, f32)> {
        let mut aggregated: HashMap<(Uuid, Uuid), f32> = HashMap::new();

        for interaction in interactions {
            let rating = interaction
                .interaction_type
                .to_rating(interaction.watch_progress);
            *aggregated
                .entry((interaction.user_id, interaction.content_id))
                .or_insert(0.0) += rating;
        }

        aggregated
            .into_iter()
            .map(|((user_id, content_id), rating)| (user_id, content_id, rating.min(1.0)))
            .collect()
    }

    /// Train ALS model on collected feedback
    pub async fn train_model(&mut self) -> Result<()> {
        let interactions = self.collect_feedback().await?;
        let matrix_data = self.build_user_item_matrix(&interactions);

        let mut model = MatrixFactorization::new(self.als_config.clone());
        let matrix = model.build_matrix(matrix_data)?;
        model.fit(&matrix)?;

        self.model = Some(model);

        // Store embeddings in Qdrant
        self.store_embeddings().await?;

        Ok(())
    }

    /// Store user and item embeddings in Qdrant
    async fn store_embeddings(&self) -> Result<()> {
        let model = self.model.as_ref().context("Model not trained yet")?;

        // Store user embeddings
        let mut user_points = Vec::new();
        for (user_id, &user_idx) in &model.user_id_map {
            let embedding = model.get_user_embedding(*user_id)?;
            let mut payload_map = HashMap::new();
            payload_map.insert(
                "user_id".to_string(),
                QdrantValue::from(user_id.to_string()),
            );
            let payload: Payload = payload_map.into();
            user_points.push(PointStruct::new(user_idx as u64, embedding, payload));
        }

        if !user_points.is_empty() {
            self.qdrant
                .upsert_points(UpsertPointsBuilder::new(
                    USER_EMBEDDINGS_COLLECTION,
                    user_points,
                ))
                .await?;
        }

        // Store item embeddings
        let mut item_points = Vec::new();
        for (item_id, &item_idx) in &model.item_id_map {
            let embedding = model.get_item_embedding(*item_id)?;
            let mut payload_map = HashMap::new();
            payload_map.insert(
                "content_id".to_string(),
                QdrantValue::from(item_id.to_string()),
            );
            let payload: Payload = payload_map.into();
            item_points.push(PointStruct::new(item_idx as u64, embedding, payload));
        }

        if !item_points.is_empty() {
            self.qdrant
                .upsert_points(UpsertPointsBuilder::new(
                    ITEM_EMBEDDINGS_COLLECTION,
                    item_points,
                ))
                .await?;
        }

        Ok(())
    }

    /// Compute user similarity using cosine similarity
    pub async fn compute_user_similarity(
        &self,
        user_id: Uuid,
        k: usize,
    ) -> Result<Vec<(Uuid, f32)>> {
        let model = self.model.as_ref().context("Model not trained yet")?;
        let user_embedding = model.get_user_embedding(user_id)?;

        let search_result = self
            .qdrant
            .search_points(SearchPoints {
                collection_name: USER_EMBEDDINGS_COLLECTION.to_string(),
                vector: user_embedding,
                limit: (k + 1) as u64, // +1 to exclude self
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await?;

        let mut similar_users = Vec::new();
        for point in search_result.result {
            if let Some(payload) = point.payload.get("user_id") {
                if let Some(user_id_str) = payload.as_str() {
                    let similar_user_id = Uuid::parse_str(user_id_str)?;
                    if similar_user_id != user_id {
                        // Exclude self
                        similar_users.push((similar_user_id, point.score));
                    }
                }
            }
        }

        similar_users.truncate(k);
        Ok(similar_users)
    }

    /// Compute item-item similarity
    pub async fn compute_item_similarity(
        &self,
        item_id: Uuid,
        k: usize,
    ) -> Result<Vec<(Uuid, f32)>> {
        let model = self.model.as_ref().context("Model not trained yet")?;
        let item_embedding = model.get_item_embedding(item_id)?;

        let search_result = self
            .qdrant
            .search_points(SearchPoints {
                collection_name: ITEM_EMBEDDINGS_COLLECTION.to_string(),
                vector: item_embedding,
                limit: (k + 1) as u64, // +1 to exclude self
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await?;

        let mut similar_items = Vec::new();
        for point in search_result.result {
            if let Some(payload) = point.payload.get("content_id") {
                if let Some(item_id_str) = payload.as_str() {
                    let similar_item_id = Uuid::parse_str(item_id_str)?;
                    if similar_item_id != item_id {
                        // Exclude self
                        similar_items.push((similar_item_id, point.score));
                    }
                }
            }
        }

        similar_items.truncate(k);
        Ok(similar_items)
    }

    /// Generate recommendations for user
    pub async fn recommend(&self, user_id: Uuid, limit: usize) -> Result<Vec<(Uuid, f32)>> {
        let model = self.model.as_ref().context("Model not trained yet")?;

        // Get user's already-seen content
        let seen_content = self.get_user_seen_content(user_id).await?;

        // Find similar users
        let similar_users = self.compute_user_similarity(user_id, 20).await?;

        if similar_users.is_empty() {
            return Ok(Vec::new());
        }

        // Aggregate recommendations from similar users
        let mut scores: HashMap<Uuid, f32> = HashMap::new();

        for (similar_user_id, similarity) in &similar_users {
            if similarity < &SIMILARITY_THRESHOLD {
                continue;
            }

            // Get items liked by similar user
            let similar_user_items = self.get_user_preferences(*similar_user_id).await?;

            for (item_id, item_score) in similar_user_items {
                if !seen_content.contains(&item_id) && item_score > 0.0 {
                    *scores.entry(item_id).or_insert(0.0) += similarity * item_score;
                }
            }
        }

        // Sort by score and return top-k
        let mut recommendations: Vec<_> = scores.into_iter().collect();
        recommendations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        recommendations.truncate(limit);

        Ok(recommendations)
    }

    /// Get user's preference scores for items
    async fn get_user_preferences(&self, user_id: Uuid) -> Result<HashMap<Uuid, f32>> {
        let rows = sqlx::query(
            r#"
            SELECT content_id,
                   SUM(CASE
                       WHEN interaction_type = 'watch' THEN
                           CASE WHEN watch_progress >= 0.9 THEN 1.0
                                WHEN watch_progress >= 0.5 THEN 0.5
                                ELSE 0.2 END
                       WHEN interaction_type = 'like' THEN 1.0
                       WHEN interaction_type = 'rate' THEN rating / 5.0
                       WHEN interaction_type = 'dislike' THEN 0.0
                       ELSE 0.0
                   END) as preference_score
            FROM users.interactions
            WHERE user_id = $1
            GROUP BY content_id
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut preferences = HashMap::new();
        for row in rows {
            let content_id: Uuid = row.get("content_id");
            let score: f64 = row.get("preference_score");
            preferences.insert(content_id, score as f32);
        }

        Ok(preferences)
    }

    /// Get content IDs the user has already seen
    async fn get_user_seen_content(&self, user_id: Uuid) -> Result<Vec<Uuid>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT content_id
            FROM users.interactions
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|r| r.get("content_id")).collect())
    }

    /// Add interaction to incremental update buffer
    pub fn add_interaction(&mut self, interaction: Interaction) {
        self.incremental_buffer.push(interaction);
    }

    /// Perform incremental update when buffer reaches batch size
    pub async fn incremental_update(&mut self) -> Result<bool> {
        if self.incremental_buffer.len() < BATCH_SIZE {
            return Ok(false);
        }

        // For simplicity, retrain the model
        // In production, implement true incremental ALS
        self.train_model().await?;
        self.incremental_buffer.clear();

        Ok(true)
    }

    /// Get "users who watched X also watched Y" recommendations
    pub async fn get_also_watched(&self, item_id: Uuid, limit: usize) -> Result<Vec<(Uuid, f32)>> {
        self.compute_item_similarity(item_id, limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_type_to_rating() {
        assert_eq!(InteractionType::Like.to_rating(None), 1.0);
        assert_eq!(InteractionType::Dislike.to_rating(None), 0.0);
        assert_eq!(InteractionType::Rating(4.0).to_rating(None), 0.8);
        assert_eq!(InteractionType::Completion.to_rating(None), 1.0);
        assert_eq!(InteractionType::View.to_rating(Some(0.95)), 1.0);
        assert_eq!(InteractionType::View.to_rating(Some(0.6)), 0.5);
        assert_eq!(InteractionType::View.to_rating(Some(0.3)), 0.2);
    }

    #[test]
    fn test_build_user_item_matrix() {
        let pool = unsafe { std::mem::zeroed() };
        let qdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();
        let engine = CollaborativeFilteringEngine::new(pool, qdrant);

        let user1 = Uuid::new_v4();
        let item1 = Uuid::new_v4();
        let item2 = Uuid::new_v4();

        let interactions = vec![
            Interaction {
                user_id: user1,
                content_id: item1,
                interaction_type: InteractionType::Like,
                watch_progress: None,
                timestamp: chrono::Utc::now(),
            },
            Interaction {
                user_id: user1,
                content_id: item2,
                interaction_type: InteractionType::View,
                watch_progress: Some(0.95),
                timestamp: chrono::Utc::now(),
            },
        ];

        let matrix_data = engine.build_user_item_matrix(&interactions);
        assert_eq!(matrix_data.len(), 2);

        // Find entries
        let entry1 = matrix_data
            .iter()
            .find(|(u, i, _)| *u == user1 && *i == item1);
        assert!(entry1.is_some());
        assert_eq!(entry1.unwrap().2, 1.0);

        let entry2 = matrix_data
            .iter()
            .find(|(u, i, _)| *u == user1 && *i == item2);
        assert!(entry2.is_some());
        assert_eq!(entry2.unwrap().2, 1.0);
    }

    #[test]
    fn test_incremental_buffer() {
        let pool = unsafe { std::mem::zeroed() };
        let qdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();
        let mut engine = CollaborativeFilteringEngine::new(pool, qdrant);

        let interaction = Interaction {
            user_id: Uuid::new_v4(),
            content_id: Uuid::new_v4(),
            interaction_type: InteractionType::Like,
            watch_progress: None,
            timestamp: chrono::Utc::now(),
        };

        engine.add_interaction(interaction);
        assert_eq!(engine.incremental_buffer.len(), 1);
    }
}
