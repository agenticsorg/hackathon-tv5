//! Content-based filtering implementation
//!
//! Uses content features and embeddings to find similar items

use anyhow::Result;
use qdrant_client::qdrant::{Condition, FieldCondition, Filter, Match, SearchPoints};
use qdrant_client::Qdrant;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

/// Content-based recommendation engine
pub struct ContentBasedEngine {
    pool: PgPool,
    qdrant: Qdrant,
    collection_name: String,
}

impl ContentBasedEngine {
    pub fn new(pool: PgPool, qdrant_url: &str, collection_name: String) -> Result<Self> {
        let qdrant = Qdrant::from_url(qdrant_url).build()?;
        Ok(Self {
            pool,
            qdrant,
            collection_name,
        })
    }

    /// Get content features from database
    pub async fn get_content_features(&self, content_id: Uuid) -> Result<Option<ContentFeatures>> {
        let row = sqlx::query(
            r#"
            SELECT id, title, genres, release_year, popularity_score, embedding
            FROM content.items
            WHERE id = $1
            "#,
        )
        .bind(content_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ContentFeatures {
            id: r.get("id"),
            title: r.get("title"),
            genres: r.get("genres"),
            release_year: r.get("release_year"),
            popularity_score: r.get("popularity_score"),
            embedding: r.get("embedding"),
        }))
    }

    /// Find similar content using vector similarity
    pub async fn find_similar_content(
        &self,
        content_id: Uuid,
        limit: usize,
        genre_filter: Option<Vec<String>>,
    ) -> Result<Vec<(Uuid, f32)>> {
        // Get the source content's embedding
        let content = self
            .get_content_features(content_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Content not found"))?;

        let embedding = content
            .embedding
            .ok_or_else(|| anyhow::anyhow!("Content has no embedding"))?;

        // Build filter if genres specified
        let filter = genre_filter.map(|genres| Filter {
            must: vec![qdrant_client::qdrant::Condition {
                condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    FieldCondition {
                        key: "genres".to_string(),
                        r#match: Some(Match {
                            match_value: Some(
                                qdrant_client::qdrant::r#match::MatchValue::Keywords(
                                    qdrant_client::qdrant::RepeatedStrings { strings: genres },
                                ),
                            ),
                        }),
                        ..Default::default()
                    },
                )),
            }],
            ..Default::default()
        });

        // Search Qdrant for similar vectors
        let search_result = self
            .qdrant
            .search_points(SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: embedding,
                filter,
                limit: (limit + 1) as u64, // +1 to exclude self
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await?;

        // Convert results, excluding the source content
        let mut results = Vec::new();
        for scored in search_result.result {
            let payload = &scored.payload;
            if let Some(id_value) = payload.get("id") {
                if let Some(id_str) = id_value.as_str() {
                    if let Ok(id) = Uuid::parse_str(id_str) {
                        if id != content_id {
                            results.push((id, scored.score));
                        }
                    }
                }
            }
        }

        results.truncate(limit);
        Ok(results)
    }

    /// Get content-based recommendations for a user based on their preferences
    pub async fn get_recommendations_for_user(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<(Uuid, f32)>> {
        // Get user's top-rated/watched content
        let seed_content = self.get_user_seed_content(user_id, 10).await?;

        if seed_content.is_empty() {
            return Ok(Vec::new());
        }

        // Get user's seen content to exclude
        let seen = self.get_user_seen_content(user_id).await?;

        // Find similar content for each seed
        let mut aggregated_scores: HashMap<Uuid, f32> = HashMap::new();

        for (content_id, user_score) in &seed_content {
            if let Ok(similar) = self.find_similar_content(*content_id, 20, None).await {
                for (similar_id, similarity) in similar {
                    if !seen.contains(&similar_id) {
                        let weighted_score = similarity * user_score;
                        *aggregated_scores.entry(similar_id).or_insert(0.0) += weighted_score;
                    }
                }
            }
        }

        // Sort and return top recommendations
        let mut recommendations: Vec<_> = aggregated_scores.into_iter().collect();
        recommendations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        recommendations.truncate(limit);

        Ok(recommendations)
    }

    /// Get user's seed content (highly rated items) for recommendations
    async fn get_user_seed_content(&self, user_id: Uuid, limit: usize) -> Result<Vec<(Uuid, f32)>> {
        let rows = sqlx::query(
            r#"
            SELECT content_id,
                   CASE
                       WHEN interaction_type = 'rate' AND rating >= 4 THEN rating / 5.0
                       WHEN interaction_type = 'like' THEN 1.0
                       WHEN interaction_type = 'watch' AND watch_progress >= 0.9 THEN 0.8
                       ELSE 0.5
                   END as score
            FROM users.interactions
            WHERE user_id = $1
              AND (
                  (interaction_type = 'rate' AND rating >= 4) OR
                  interaction_type = 'like' OR
                  (interaction_type = 'watch' AND watch_progress >= 0.8)
              )
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| {
                (
                    r.get::<Uuid, _>("content_id"),
                    r.get::<f64, _>("score") as f32,
                )
            })
            .collect())
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

    /// Compute feature similarity (for non-embedding features)
    pub fn compute_feature_similarity(&self, a: &ContentFeatures, b: &ContentFeatures) -> f32 {
        let mut score = 0.0f32;
        let mut weight_sum = 0.0f32;

        // Genre overlap (Jaccard similarity)
        let genre_sim = self.jaccard_similarity(&a.genres, &b.genres);
        score += genre_sim * 0.4;
        weight_sum += 0.4;

        // Year proximity (normalized)
        let year_diff = (a.release_year - b.release_year).abs() as f32;
        let year_sim = 1.0 - (year_diff / 50.0).min(1.0);
        score += year_sim * 0.2;
        weight_sum += 0.2;

        // Popularity similarity
        let pop_diff = (a.popularity_score - b.popularity_score).abs();
        let pop_sim = 1.0 - pop_diff.min(1.0);
        score += pop_sim * 0.1;
        weight_sum += 0.1;

        if weight_sum > 0.0 {
            score / weight_sum
        } else {
            0.0
        }
    }

    fn jaccard_similarity(&self, a: &[String], b: &[String]) -> f32 {
        if a.is_empty() && b.is_empty() {
            return 1.0;
        }
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        let set_a: std::collections::HashSet<_> = a.iter().collect();
        let set_b: std::collections::HashSet<_> = b.iter().collect();

        let intersection = set_a.intersection(&set_b).count();
        let union = set_a.union(&set_b).count();

        intersection as f32 / union as f32
    }
}

/// Content features for similarity computation
#[derive(Debug, Clone)]
pub struct ContentFeatures {
    pub id: Uuid,
    pub title: String,
    pub genres: Vec<String>,
    pub release_year: i32,
    pub popularity_score: f32,
    pub embedding: Option<Vec<f32>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaccard_similarity_calculation() {
        // Manual Jaccard test without engine
        let a = vec!["action".to_string(), "comedy".to_string()];
        let b = vec!["action".to_string(), "drama".to_string()];

        let set_a: std::collections::HashSet<_> = a.iter().collect();
        let set_b: std::collections::HashSet<_> = b.iter().collect();

        let intersection = set_a.intersection(&set_b).count();
        let union = set_a.union(&set_b).count();

        let jaccard = intersection as f32 / union as f32;

        // intersection = 1 (action), union = 3 (action, comedy, drama)
        assert!((jaccard - 0.333).abs() < 0.01);
    }
}
