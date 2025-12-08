//! Federation coordinator for cross-shard pattern aggregation
//!
//! Implements quality-weighted federated averaging to aggregate
//! high-quality patterns across all shards and detect trends.

use crate::{Result, ShardStats, TrendSignal, ViewingPattern};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, warn};

/// Federation configuration
#[derive(Debug, Clone)]
pub struct FederationConfig {
    pub aggregation_interval: Duration,
    pub min_quality_threshold: f32,
    pub trend_decay_rate: f32,
    pub min_source_count: usize,
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self {
            aggregation_interval: Duration::from_secs(3600), // 1 hour
            min_quality_threshold: 0.8,
            trend_decay_rate: 0.95,
            min_source_count: 3,
        }
    }
}

/// Federation coordinator
pub struct FederationCoordinator {
    config: FederationConfig,
}

impl FederationCoordinator {
    pub fn new(config: FederationConfig) -> Self {
        info!("Initializing federation coordinator");
        Self { config }
    }

    /// Run federation round
    pub async fn run_federation(
        &self,
        shard_patterns: Vec<ShardPatterns>,
    ) -> Result<FederationResult> {
        info!("Starting federation round with {} shards", shard_patterns.len());

        // 1. Quality-weighted federated averaging
        let aggregated = self.federated_average(shard_patterns.clone());

        info!("Aggregated {} pattern groups", aggregated.len());

        // 2. Detect trends
        let trends = self.detect_trends(&aggregated);

        info!("Detected {} trending patterns", trends.len());

        Ok(FederationResult {
            patterns_aggregated: aggregated.len(),
            trends_detected: trends.len(),
            aggregated_patterns: aggregated,
            trends,
        })
    }

    /// Quality-weighted federated averaging
    fn federated_average(&self, shard_patterns: Vec<ShardPatterns>) -> Vec<AggregatedPattern> {
        let mut pattern_groups: HashMap<PatternKey, Vec<(ViewingPattern, f32)>> = HashMap::new();

        // Group similar patterns by embedding similarity
        for shard in shard_patterns {
            for pattern in shard.patterns {
                if pattern.success_rate < self.config.min_quality_threshold {
                    continue;
                }

                let key = self.compute_pattern_key(&pattern.embedding);
                let weight = pattern.success_rate * pattern.sample_count as f32;
                pattern_groups
                    .entry(key)
                    .or_insert_with(Vec::new)
                    .push((pattern, weight));
            }
        }

        // Weighted average within each group
        pattern_groups
            .into_iter()
            .filter(|(_, group)| group.len() >= self.config.min_source_count)
            .filter_map(|(_, group)| {
                let total_weight: f32 = group.iter().map(|(_, w)| w).sum();

                if total_weight == 0.0 {
                    return None;
                }

                let dim = group[0].0.embedding.len();
                let avg_embedding: Vec<f32> = (0..dim)
                    .map(|i| {
                        group
                            .iter()
                            .map(|(p, w)| p.embedding[i] * w)
                            .sum::<f32>()
                            / total_weight
                    })
                    .collect();

                let avg_success_rate =
                    group.iter().map(|(p, _)| p.success_rate).sum::<f32>() / group.len() as f32;

                Some(AggregatedPattern {
                    embedding: avg_embedding,
                    quality: total_weight / group.len() as f32,
                    source_count: group.len(),
                    avg_success_rate,
                    sample_contexts: group
                        .iter()
                        .take(3)
                        .map(|(p, _)| p.context.clone())
                        .collect(),
                })
            })
            .collect()
    }

    /// Detect trends from aggregated patterns
    fn detect_trends(&self, patterns: &[AggregatedPattern]) -> Vec<TrendSignal> {
        let mut content_scores: HashMap<String, f32> = HashMap::new();

        for pattern in patterns {
            // Use genre as content identifier
            for context in &pattern.sample_contexts {
                for genre in &context.genre_hints {
                    let score = pattern.quality * pattern.avg_success_rate;
                    *content_scores.entry(genre.clone()).or_insert(0.0) += score;
                }
            }
        }

        let mut trends: Vec<TrendSignal> = content_scores
            .into_iter()
            .map(|(content_id, score)| TrendSignal {
                content_id,
                trending_score: score,
                region: "global".to_string(),
            })
            .collect();

        trends.sort_by(|a, b| {
            b.trending_score
                .partial_cmp(&a.trending_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        trends.into_iter().take(100).collect()
    }

    /// Compute pattern key for grouping similar patterns
    fn compute_pattern_key(&self, embedding: &[f32]) -> PatternKey {
        // Simple bucketing: quantize first 8 dimensions
        let mut key = [0u8; 8];
        for (i, val) in embedding.iter().take(8).enumerate() {
            key[i] = ((val + 1.0) * 127.5).clamp(0.0, 255.0) as u8;
        }
        PatternKey(key)
    }
}

/// Pattern key for grouping
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct PatternKey([u8; 8]);

/// Patterns from a shard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardPatterns {
    pub shard_id: u32,
    pub patterns: Vec<ViewingPattern>,
    pub stats: Option<ShardStats>,
}

/// Aggregated pattern from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedPattern {
    pub embedding: Vec<f32>,
    pub quality: f32,
    pub source_count: usize,
    pub avg_success_rate: f32,
    pub sample_contexts: Vec<crate::PatternContext>,
}

/// Federation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationResult {
    pub patterns_aggregated: usize,
    pub trends_detected: usize,
    pub aggregated_patterns: Vec<AggregatedPattern>,
    pub trends: Vec<TrendSignal>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PatternContext;
    use uuid::Uuid;

    fn create_test_pattern(embedding: Vec<f32>, success_rate: f32) -> ViewingPattern {
        ViewingPattern {
            id: Uuid::new_v4(),
            embedding,
            success_rate,
            sample_count: 10,
            context: PatternContext {
                time_of_day: "evening".to_string(),
                day_of_week: "weekend".to_string(),
                content_type: "movie".to_string(),
                genre_hints: vec!["thriller".to_string()],
            },
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        }
    }

    #[tokio::test]
    async fn test_federation() {
        let coordinator = FederationCoordinator::new(FederationConfig::default());

        let shard1 = ShardPatterns {
            shard_id: 1,
            patterns: vec![
                create_test_pattern(vec![0.5; 384], 0.9),
                create_test_pattern(vec![0.6; 384], 0.85),
            ],
            stats: None,
        };

        let shard2 = ShardPatterns {
            shard_id: 2,
            patterns: vec![
                create_test_pattern(vec![0.5; 384], 0.88),
                create_test_pattern(vec![0.7; 384], 0.92),
            ],
            stats: None,
        };

        let result = coordinator
            .run_federation(vec![shard1, shard2])
            .await
            .unwrap();

        assert!(result.patterns_aggregated > 0);
    }
}
