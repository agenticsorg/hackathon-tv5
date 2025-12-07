//! User Personalization Service for Search Results
//!
//! Integrates with SONA Personalization Engine to apply user preference scoring
//! to search results with caching and A/B testing support.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::cache::RedisCache;
use crate::search::SearchResult;

// Re-export PersonalizationConfig from config module
pub use crate::config::PersonalizationConfig;

/// Personalization score response from SONA
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersonalizationScoreResponse {
    user_id: Uuid,
    content_id: Uuid,
    score: f32,
    components: ScoreComponents,
}

/// Score component breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScoreComponents {
    collaborative: f32,
    content_based: f32,
    graph_based: f32,
    context: f32,
    lora_boost: f32,
}

/// Cached user preference vector
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserPreferenceCache {
    user_id: Uuid,
    preference_scores: Vec<(Uuid, f32)>, // (content_id, score) pairs
    cached_at: i64,
}

/// Personalization service
pub struct PersonalizationService {
    config: PersonalizationConfig,
    http_client: reqwest::Client,
    cache: Arc<RedisCache>,
}

impl PersonalizationService {
    /// Create new personalization service
    pub fn new(config: PersonalizationConfig, cache: Arc<RedisCache>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
            cache,
        }
    }

    /// Apply personalization to search results
    ///
    /// # Arguments
    /// * `user_id` - User ID to personalize for
    /// * `results` - Search results to rerank
    /// * `experiment_variant` - Optional A/B test variant name
    ///
    /// # Returns
    /// Reranked results with personalization boost applied
    #[instrument(skip(self, results), fields(user_id = %user_id, num_results = results.len()))]
    pub async fn personalize_results(
        &self,
        user_id: Uuid,
        mut results: Vec<SearchResult>,
        experiment_variant: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        if !self.config.enabled {
            debug!("Personalization disabled, returning original results");
            return Ok(results);
        }

        let start_time = std::time::Instant::now();

        // Get boost weight (may vary by A/B test variant)
        let boost_weight = self.get_boost_weight_for_variant(experiment_variant);

        // Fetch personalization scores for all content
        let content_ids: Vec<Uuid> = results.iter().map(|r| r.content.id).collect();
        let personalization_scores = match self.fetch_scores_batch(user_id, content_ids).await {
            Ok(scores) => scores,
            Err(e) => {
                warn!(
                    error = %e,
                    user_id = %user_id,
                    "Failed to fetch personalization scores, using original ranking"
                );
                return Ok(results);
            }
        };

        // Apply personalization boost to relevance scores
        for result in &mut results {
            if let Some(&score) = personalization_scores.get(&result.content.id) {
                let original_score = result.relevance_score;
                result.relevance_score =
                    original_score * (1.0 - boost_weight) + score * boost_weight;

                debug!(
                    content_id = %result.content.id,
                    original_score = %original_score,
                    personalization_score = %score,
                    final_score = %result.relevance_score,
                    "Applied personalization boost"
                );
            }
        }

        // Rerank by new relevance scores
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let elapsed_ms = start_time.elapsed().as_millis() as u64;
        info!(
            user_id = %user_id,
            num_results = results.len(),
            elapsed_ms = %elapsed_ms,
            boost_weight = %boost_weight,
            "Personalization applied"
        );

        // Verify latency requirement (<50ms)
        if elapsed_ms > 50 {
            warn!(
                elapsed_ms = %elapsed_ms,
                "Personalization exceeded 50ms latency target"
            );
        }

        Ok(results)
    }

    /// Fetch personalization scores for a batch of content
    #[instrument(skip(self, content_ids), fields(user_id = %user_id, num_content = content_ids.len()))]
    async fn fetch_scores_batch(
        &self,
        user_id: Uuid,
        content_ids: Vec<Uuid>,
    ) -> Result<std::collections::HashMap<Uuid, f32>> {
        // Check cache first
        let cache_key = format!("personalization:{}:batch", user_id);
        if let Ok(Some(cached)) = self.cache.get::<UserPreferenceCache>(&cache_key).await {
            debug!(cache_key = %cache_key, "Cache hit for user preferences");
            return Ok(cached.preference_scores.into_iter().collect());
        }

        debug!(cache_key = %cache_key, "Cache miss, fetching from SONA");

        // Fetch scores from SONA in parallel
        let mut scores = std::collections::HashMap::new();
        let mut tasks = Vec::new();

        for content_id in &content_ids {
            let task = self.fetch_single_score(user_id, *content_id);
            tasks.push(async move {
                match task.await {
                    Ok(score) => Some((*content_id, score)),
                    Err(e) => {
                        debug!(error = %e, content_id = %content_id, "Failed to fetch score");
                        None
                    }
                }
            });
        }

        let results = futures::future::join_all(tasks).await;
        for result in results {
            if let Some((content_id, score)) = result {
                scores.insert(content_id, score);
            }
        }

        // Cache the results
        if !scores.is_empty() {
            let cache_entry = UserPreferenceCache {
                user_id,
                preference_scores: scores.iter().map(|(k, v)| (*k, *v)).collect(),
                cached_at: chrono::Utc::now().timestamp(),
            };

            if let Err(e) = self
                .cache
                .set(&cache_key, &cache_entry, self.config.cache_ttl_sec)
                .await
            {
                debug!(error = %e, "Failed to cache personalization scores");
            } else {
                debug!(
                    cache_key = %cache_key,
                    ttl = %self.config.cache_ttl_sec,
                    "Cached personalization scores"
                );
            }
        }

        Ok(scores)
    }

    /// Fetch personalization score for a single content item
    async fn fetch_single_score(&self, user_id: Uuid, content_id: Uuid) -> Result<f32> {
        let url = format!("{}/api/v1/personalization/score", self.config.sona_url);

        let request_body = serde_json::json!({
            "user_id": user_id,
            "content_id": content_id
        });

        let response = self
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to SONA")?;

        if !response.status().is_success() {
            anyhow::bail!("SONA returned error status: {}", response.status());
        }

        let score_response: PersonalizationScoreResponse = response
            .json()
            .await
            .context("Failed to parse SONA response")?;

        Ok(score_response.score)
    }

    /// Get boost weight based on A/B test variant
    pub(crate) fn get_boost_weight_for_variant(&self, variant: Option<&str>) -> f32 {
        match variant {
            Some("control") => 0.0,           // No personalization
            Some("low_boost") => 0.15,        // Low personalization
            Some("medium_boost") => 0.25,     // Default
            Some("high_boost") => 0.40,       // High personalization
            Some("aggressive_boost") => 0.60, // Aggressive personalization
            _ => self.config.boost_weight,    // Default from config
        }
    }

    /// Invalidate cache for a user (e.g., after preference update)
    pub async fn invalidate_cache(&self, user_id: Uuid) -> Result<()> {
        let cache_key = format!("personalization:{}:batch", user_id);
        self.cache
            .delete(&cache_key)
            .await
            .context("Failed to invalidate cache")?;
        info!(user_id = %user_id, cache_key = %cache_key, "Invalidated personalization cache");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{ContentSummary, SearchResult};

    fn create_mock_result(id: Uuid, title: &str, score: f32) -> SearchResult {
        SearchResult {
            content: ContentSummary {
                id,
                title: title.to_string(),
                overview: "Test overview".to_string(),
                release_year: 2024,
                genres: vec!["action".to_string()],
                platforms: vec!["netflix".to_string()],
                popularity_score: 0.8,
            },
            relevance_score: score,
            match_reasons: vec![],
            vector_similarity: Some(score),
            graph_score: None,
            keyword_score: None,
        }
    }

    #[test]
    fn test_boost_weight_for_variants() {
        let config = PersonalizationConfig::default();
        let cache_config = Arc::new(crate::config::CacheConfig {
            redis_url: "redis://localhost:6379".to_string(),
            search_ttl_sec: 1800,
            embedding_ttl_sec: 3600,
            intent_ttl_sec: 600,
        });

        // Create mock cache (will fail without Redis, but test doesn't need it)
        let cache = Arc::new(crate::cache::RedisCache::new_mock());
        let service = PersonalizationService::new(config, cache);

        assert_eq!(service.get_boost_weight_for_variant(Some("control")), 0.0);
        assert_eq!(
            service.get_boost_weight_for_variant(Some("low_boost")),
            0.15
        );
        assert_eq!(
            service.get_boost_weight_for_variant(Some("medium_boost")),
            0.25
        );
        assert_eq!(
            service.get_boost_weight_for_variant(Some("high_boost")),
            0.40
        );
        assert_eq!(
            service.get_boost_weight_for_variant(Some("aggressive_boost")),
            0.60
        );
        assert_eq!(service.get_boost_weight_for_variant(None), 0.25); // default
    }

    #[test]
    fn test_reranking_logic() {
        // Test that results are reranked correctly after personalization boost
        let mut results = vec![
            create_mock_result(Uuid::new_v4(), "Movie A", 0.5),
            create_mock_result(Uuid::new_v4(), "Movie B", 0.7),
            create_mock_result(Uuid::new_v4(), "Movie C", 0.6),
        ];

        // Simulate personalization boost: boost Movie A significantly
        let boost_weight = 0.3;
        results[0].relevance_score =
            results[0].relevance_score * (1.0 - boost_weight) + 0.95 * boost_weight; // High personalization score for Movie A

        // Rerank
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Movie B should still be first (0.7 > boosted 0.5)
        assert_eq!(results[0].content.title, "Movie B");
    }

    #[tokio::test]
    async fn test_personalization_disabled() {
        let mut config = PersonalizationConfig::default();
        config.enabled = false;

        let cache_config = Arc::new(crate::config::CacheConfig {
            redis_url: "redis://localhost:6379".to_string(),
            search_ttl_sec: 1800,
            embedding_ttl_sec: 3600,
            intent_ttl_sec: 600,
        });

        let cache = Arc::new(crate::cache::RedisCache::new_mock());
        let service = PersonalizationService::new(config, cache);

        let user_id = Uuid::new_v4();
        let results = vec![create_mock_result(Uuid::new_v4(), "Movie A", 0.5)];
        let original_scores: Vec<f32> = results.iter().map(|r| r.relevance_score).collect();

        let personalized = service
            .personalize_results(user_id, results, None)
            .await
            .unwrap();

        // Scores should be unchanged when personalization is disabled
        let new_scores: Vec<f32> = personalized.iter().map(|r| r.relevance_score).collect();
        assert_eq!(original_scores, new_scores);
    }
}
