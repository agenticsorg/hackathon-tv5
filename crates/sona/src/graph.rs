//! Graph-Based Recommendation Engine
//!
//! Implements graph traversal and similarity scoring based on:
//! - Content-content relationships (genre, cast, director, themes)
//! - User-user collaborative graphs (co-viewing patterns)
//! - Weighted graph affinity scoring

use anyhow::{anyhow, Result};
use sqlx::{PgPool, Row};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

const MAX_GRAPH_DEPTH: usize = 3;
const GENRE_SIMILARITY_WEIGHT: f32 = 0.35;
const CAST_SIMILARITY_WEIGHT: f32 = 0.25;
const DIRECTOR_SIMILARITY_WEIGHT: f32 = 0.20;
const THEME_SIMILARITY_WEIGHT: f32 = 0.20;
const COLLABORATIVE_DECAY: f32 = 0.85;

/// Graph-based recommender using PostgreSQL-backed graph queries
pub struct GraphRecommender {
    pool: PgPool,
}

impl GraphRecommender {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Generate graph-based recommendations for a user
    ///
    /// Algorithm:
    /// 1. Get user's watch history (seed nodes)
    /// 2. Build content-content similarity graph
    /// 3. Traverse user-user collaborative graph
    /// 4. Compute weighted graph affinity scores
    /// 5. Return ranked content IDs with scores
    pub async fn recommend(&self, user_id: Uuid, limit: usize) -> Result<Vec<(Uuid, f32)>> {
        // Get user's watch history as seed nodes
        let seed_content = self.get_user_watch_history(user_id, 50).await?;

        if seed_content.is_empty() {
            return Ok(Vec::new());
        }

        // Compute content-content similarity scores
        let content_scores = self
            .compute_content_similarity_scores(&seed_content)
            .await?;

        // Compute user-user collaborative scores
        let collaborative_scores = self
            .compute_collaborative_scores(user_id, &seed_content)
            .await?;

        // Merge scores with weighted combination
        let mut merged_scores: HashMap<Uuid, f32> = HashMap::new();

        for (content_id, score) in content_scores {
            *merged_scores.entry(content_id).or_insert(0.0) += score * 0.6;
        }

        for (content_id, score) in collaborative_scores {
            *merged_scores.entry(content_id).or_insert(0.0) += score * 0.4;
        }

        // Filter out already watched content
        let watched_set: HashSet<Uuid> = seed_content.into_iter().collect();
        merged_scores.retain(|k, _| !watched_set.contains(k));

        // Sort by score and take top N
        let mut results: Vec<(Uuid, f32)> = merged_scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }

    /// Get user's watch history (most recent watched content)
    async fn get_user_watch_history(&self, user_id: Uuid, limit: usize) -> Result<Vec<Uuid>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT content_id
            FROM watch_progress
            WHERE user_id = $1
            ORDER BY last_watched DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let content_ids: Result<Vec<Uuid>, _> =
            rows.iter().map(|row| row.try_get("content_id")).collect();

        Ok(content_ids?)
    }

    /// Compute content-content similarity based on shared attributes
    ///
    /// Similarity factors:
    /// - Genre overlap (35%)
    /// - Cast overlap (25%)
    /// - Director match (20%)
    /// - Theme overlap (20%)
    async fn compute_content_similarity_scores(
        &self,
        seed_content: &[Uuid],
    ) -> Result<HashMap<Uuid, f32>> {
        let mut scores: HashMap<Uuid, f32> = HashMap::new();

        for seed_id in seed_content {
            // Get genre-based similar content
            let genre_similar = self.find_genre_similar(*seed_id, 30).await?;
            for (content_id, similarity) in genre_similar {
                *scores.entry(content_id).or_insert(0.0) += similarity * GENRE_SIMILARITY_WEIGHT;
            }

            // Get cast-based similar content
            let cast_similar = self.find_cast_similar(*seed_id, 20).await?;
            for (content_id, similarity) in cast_similar {
                *scores.entry(content_id).or_insert(0.0) += similarity * CAST_SIMILARITY_WEIGHT;
            }

            // Get director-based similar content
            let director_similar = self.find_director_similar(*seed_id, 15).await?;
            for (content_id, similarity) in director_similar {
                *scores.entry(content_id).or_insert(0.0) += similarity * DIRECTOR_SIMILARITY_WEIGHT;
            }

            // Get theme-based similar content
            let theme_similar = self.find_theme_similar(*seed_id, 20).await?;
            for (content_id, similarity) in theme_similar {
                *scores.entry(content_id).or_insert(0.0) += similarity * THEME_SIMILARITY_WEIGHT;
            }
        }

        // Normalize scores by number of seed items
        let seed_count = seed_content.len() as f32;
        for score in scores.values_mut() {
            *score /= seed_count;
        }

        Ok(scores)
    }

    /// Find content with similar genres
    async fn find_genre_similar(&self, content_id: Uuid, limit: usize) -> Result<Vec<(Uuid, f32)>> {
        let rows = sqlx::query(
            r#"
            WITH seed_genres AS (
                SELECT genre FROM content_genres WHERE content_id = $1
            ),
            genre_overlap AS (
                SELECT
                    cg.content_id,
                    COUNT(*) as overlap_count,
                    (SELECT COUNT(*) FROM seed_genres) as seed_count
                FROM content_genres cg
                WHERE cg.genre IN (SELECT genre FROM seed_genres)
                  AND cg.content_id != $1
                GROUP BY cg.content_id
            )
            SELECT
                content_id,
                (overlap_count::float / GREATEST(seed_count, 1)) as similarity
            FROM genre_overlap
            WHERE overlap_count > 0
            ORDER BY similarity DESC
            LIMIT $2
            "#,
        )
        .bind(content_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let results: Result<Vec<(Uuid, f32)>, sqlx::Error> = rows
            .iter()
            .map(|row| {
                let content_id: Uuid = row.try_get("content_id")?;
                let similarity: Option<f64> = row.try_get("similarity")?;
                Ok((content_id, similarity.unwrap_or(0.0) as f32))
            })
            .collect();

        Ok(results?)
    }

    /// Find content with shared cast members
    async fn find_cast_similar(&self, content_id: Uuid, limit: usize) -> Result<Vec<(Uuid, f32)>> {
        let rows = sqlx::query(
            r#"
            WITH seed_cast AS (
                SELECT person_name FROM credits
                WHERE content_id = $1 AND role_type = 'actor'
                LIMIT 10
            ),
            cast_overlap AS (
                SELECT
                    c.content_id,
                    COUNT(*) as overlap_count,
                    (SELECT COUNT(*) FROM seed_cast) as seed_count
                FROM credits c
                WHERE c.person_name IN (SELECT person_name FROM seed_cast)
                  AND c.content_id != $1
                  AND c.role_type = 'actor'
                GROUP BY c.content_id
            )
            SELECT
                content_id,
                (overlap_count::float / GREATEST(seed_count, 1)) as similarity
            FROM cast_overlap
            WHERE overlap_count > 0
            ORDER BY similarity DESC
            LIMIT $2
            "#,
        )
        .bind(content_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let results: Result<Vec<(Uuid, f32)>, sqlx::Error> = rows
            .iter()
            .map(|row| {
                let content_id: Uuid = row.try_get("content_id")?;
                let similarity: Option<f64> = row.try_get("similarity")?;
                Ok((content_id, similarity.unwrap_or(0.0) as f32))
            })
            .collect();

        Ok(results?)
    }

    /// Find content with same director
    async fn find_director_similar(
        &self,
        content_id: Uuid,
        limit: usize,
    ) -> Result<Vec<(Uuid, f32)>> {
        let rows = sqlx::query(
            r#"
            WITH seed_directors AS (
                SELECT person_name FROM credits
                WHERE content_id = $1 AND role_type = 'director'
            )
            SELECT
                c.content_id,
                1.0 as similarity
            FROM credits c
            WHERE c.person_name IN (SELECT person_name FROM seed_directors)
              AND c.content_id != $1
              AND c.role_type = 'director'
            LIMIT $2
            "#,
        )
        .bind(content_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let results: Result<Vec<(Uuid, f32)>, sqlx::Error> = rows
            .iter()
            .map(|row| {
                let content_id: Uuid = row.try_get("content_id")?;
                let similarity: f64 = row.try_get("similarity")?;
                Ok((content_id, similarity as f32))
            })
            .collect();

        Ok(results?)
    }

    /// Find content with similar themes
    async fn find_theme_similar(&self, content_id: Uuid, limit: usize) -> Result<Vec<(Uuid, f32)>> {
        let rows = sqlx::query(
            r#"
            WITH seed_themes AS (
                SELECT theme FROM content_themes WHERE content_id = $1
            ),
            theme_overlap AS (
                SELECT
                    ct.content_id,
                    COUNT(*) as overlap_count,
                    (SELECT COUNT(*) FROM seed_themes) as seed_count
                FROM content_themes ct
                WHERE ct.theme IN (SELECT theme FROM seed_themes)
                  AND ct.content_id != $1
                GROUP BY ct.content_id
            )
            SELECT
                content_id,
                (overlap_count::float / GREATEST(seed_count, 1)) as similarity
            FROM theme_overlap
            WHERE overlap_count > 0
            ORDER BY similarity DESC
            LIMIT $2
            "#,
        )
        .bind(content_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let results: Result<Vec<(Uuid, f32)>, sqlx::Error> = rows
            .iter()
            .map(|row| {
                let content_id: Uuid = row.try_get("content_id")?;
                let similarity: Option<f64> = row.try_get("similarity")?;
                Ok((content_id, similarity.unwrap_or(0.0) as f32))
            })
            .collect();

        Ok(results?)
    }

    /// Compute user-user collaborative graph
    ///
    /// Algorithm: "Users who liked X also liked Y"
    /// - Find similar users based on watch history overlap
    /// - Get their highly-rated content
    /// - Apply decay factor based on user similarity
    async fn compute_collaborative_scores(
        &self,
        user_id: Uuid,
        seed_content: &[Uuid],
    ) -> Result<HashMap<Uuid, f32>> {
        let similar_users = self.find_similar_users(user_id, seed_content, 20).await?;

        let mut scores: HashMap<Uuid, f32> = HashMap::new();

        for (similar_user_id, similarity) in similar_users {
            let user_content = self
                .get_user_highly_rated_content(similar_user_id, 30)
                .await?;

            for (content_id, rating) in user_content {
                let weighted_score = similarity * rating * COLLABORATIVE_DECAY;
                *scores.entry(content_id).or_insert(0.0) += weighted_score;
            }
        }

        Ok(scores)
    }

    /// Find users with similar watch patterns
    async fn find_similar_users(
        &self,
        user_id: Uuid,
        seed_content: &[Uuid],
        limit: usize,
    ) -> Result<Vec<(Uuid, f32)>> {
        if seed_content.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            WITH user_content AS (
                SELECT unnest($2::uuid[]) as content_id
            ),
            user_overlap AS (
                SELECT
                    wp.user_id,
                    COUNT(*) as overlap_count,
                    $3 as seed_count
                FROM watch_progress wp
                WHERE wp.content_id IN (SELECT content_id FROM user_content)
                  AND wp.user_id != $1
                GROUP BY wp.user_id
                HAVING COUNT(*) >= 3
            )
            SELECT
                user_id,
                (overlap_count::float / seed_count) as similarity
            FROM user_overlap
            ORDER BY similarity DESC
            LIMIT $4
            "#,
        )
        .bind(user_id)
        .bind(seed_content)
        .bind(seed_content.len() as i32)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let results: Result<Vec<(Uuid, f32)>, sqlx::Error> = rows
            .iter()
            .map(|row| {
                let user_id: Uuid = row.try_get("user_id")?;
                let similarity: Option<f64> = row.try_get("similarity")?;
                Ok((user_id, similarity.unwrap_or(0.0) as f32))
            })
            .collect();

        Ok(results?)
    }

    /// Get user's highly-rated content (completion_rate > 0.7)
    async fn get_user_highly_rated_content(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<(Uuid, f32)>> {
        let rows = sqlx::query(
            r#"
            SELECT
                content_id,
                completion_rate
            FROM watch_progress
            WHERE user_id = $1
              AND completion_rate >= 0.7
            ORDER BY completion_rate DESC, last_watched DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let results: Result<Vec<(Uuid, f32)>, sqlx::Error> = rows
            .iter()
            .map(|row| {
                let content_id: Uuid = row.try_get("content_id")?;
                let completion_rate: f64 = row.try_get("completion_rate")?;
                Ok((content_id, completion_rate as f32))
            })
            .collect();

        Ok(results?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_recommender_creation() {
        // Test requires actual DB connection, skipped in unit tests
        // Integration tests should verify actual graph queries
    }

    #[test]
    fn test_similarity_weights() {
        let total_weight = GENRE_SIMILARITY_WEIGHT
            + CAST_SIMILARITY_WEIGHT
            + DIRECTOR_SIMILARITY_WEIGHT
            + THEME_SIMILARITY_WEIGHT;

        assert!(
            (total_weight - 1.0).abs() < 0.01,
            "Similarity weights should sum to 1.0"
        );
    }

    #[test]
    fn test_collaborative_decay() {
        assert!(
            COLLABORATIVE_DECAY > 0.0 && COLLABORATIVE_DECAY < 1.0,
            "Collaborative decay should be between 0 and 1"
        );
    }
}
