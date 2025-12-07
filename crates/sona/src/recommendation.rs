//! Hybrid Recommendation Engine
//!
//! Implements GenerateRecommendations algorithm from SPARC pseudocode.
//! Combines collaborative, content-based, graph-based, and context-aware filtering.

use crate::collaborative::CollaborativeFilteringEngine;
use crate::diversity::ApplyDiversityFilter;
use crate::graph::GraphRecommender;
use crate::lora::{compute_lora_score, UserLoRAAdapter};
use crate::profile::UserProfile;
use crate::types::{Recommendation, RecommendationContext, RecommendationType, ScoredContent};
use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

const COLLABORATIVE_WEIGHT: f32 = 0.35;
const CONTENT_WEIGHT: f32 = 0.25;
const GRAPH_WEIGHT: f32 = 0.30;
const CONTEXT_WEIGHT: f32 = 0.10;
const DIVERSITY_THRESHOLD: f32 = 0.3;
const MAX_RECOMMENDATIONS: usize = 20;

/// Generate personalized recommendations
///
/// Algorithm: GenerateRecommendations (from SPARC pseudocode Part 2)
///
/// Steps:
/// 1. Generate candidate pool from multiple sources (parallel)
/// 2. Merge and deduplicate candidates
/// 3. Filter already watched content
/// 4. Apply LoRA personalization
/// 5. Apply diversity filter (MMR)
/// 6. Generate explanations
pub struct GenerateRecommendations;

impl GenerateRecommendations {
    pub async fn execute(
        user_id: Uuid,
        profile: &UserProfile,
        context: Option<RecommendationContext>,
        lora_adapter: Option<&UserLoRAAdapter>,
        get_content_embedding: impl Fn(Uuid) -> Result<Vec<f32>>,
        db_pool: Option<&PgPool>,
        cf_engine: Option<&CollaborativeFilteringEngine>,
    ) -> Result<Vec<Recommendation>> {
        // Step 1: Generate candidate pool from multiple sources (parallel)
        // In a real implementation, these would be parallel async calls
        let mut all_candidates = Vec::new();

        // Collaborative filtering candidates (using ALS-based CF engine)
        let collaborative_candidates = if let Some(engine) = cf_engine {
            Self::get_collaborative_candidates_real(user_id, engine, 100).await?
        } else {
            Self::get_collaborative_candidates(user_id, 100).await?
        };
        for mut candidate in collaborative_candidates {
            candidate.score *= COLLABORATIVE_WEIGHT;
            all_candidates.push(candidate);
        }

        // Content-based candidates (simulated)
        let content_candidates = Self::get_content_based_candidates(profile, 100).await?;
        for mut candidate in content_candidates {
            candidate.score *= CONTENT_WEIGHT;
            all_candidates.push(candidate);
        }

        // Graph-based candidates
        let graph_candidates = Self::get_graph_based_candidates(user_id, db_pool, 100).await?;
        for mut candidate in graph_candidates {
            candidate.score *= GRAPH_WEIGHT;
            all_candidates.push(candidate);
        }

        // Context-aware candidates (if context provided)
        if let Some(ctx) = &context {
            let context_candidates = Self::get_context_aware_candidates(profile, ctx, 50).await?;
            for mut candidate in context_candidates {
                candidate.score *= CONTEXT_WEIGHT;
                all_candidates.push(candidate);
            }
        }

        // Step 2: Merge and deduplicate candidates
        let merged_candidates = Self::merge_candidates(all_candidates);

        // Step 3: Filter already watched content
        let watched_ids = Self::get_watched_content_ids(user_id).await?;
        let mut filtered_candidates: Vec<ScoredContent> = merged_candidates
            .into_iter()
            .filter(|c| !watched_ids.contains(&c.content_id))
            .collect();

        // Step 4: Apply LoRA personalization
        if let Some(adapter) = lora_adapter {
            for candidate in &mut filtered_candidates {
                let content_embedding = get_content_embedding(candidate.content_id)?;
                let lora_score =
                    compute_lora_score(adapter, &content_embedding, &profile.preference_vector)?;
                candidate.score *= 1.0 + lora_score * 0.3;
            }
        }

        // Step 5: Apply diversity filter (MMR - Maximal Marginal Relevance)
        let diverse_results = ApplyDiversityFilter::execute(
            filtered_candidates,
            DIVERSITY_THRESHOLD,
            MAX_RECOMMENDATIONS,
            get_content_embedding,
        )?;

        // Step 6: Generate explanations
        let mut recommendations = Vec::new();
        for result in diverse_results {
            let explanation = Self::generate_explanation(&result, profile);
            recommendations.push(Recommendation {
                content_id: result.content_id,
                confidence_score: result.score,
                recommendation_type: result.source,
                based_on: result.based_on,
                explanation,
                generated_at: Utc::now(),
                ttl_seconds: 3600,
                experiment_variant: None, // Will be set by A/B testing layer
            });
        }

        Ok(recommendations)
    }

    async fn get_collaborative_candidates_real(
        user_id: Uuid,
        cf_engine: &CollaborativeFilteringEngine,
        limit: usize,
    ) -> Result<Vec<ScoredContent>> {
        let recommendations = cf_engine.recommend(user_id, limit).await?;

        Ok(recommendations
            .into_iter()
            .map(|(content_id, score)| ScoredContent {
                content_id,
                score,
                source: RecommendationType::Collaborative,
                based_on: vec!["collaborative_filtering".to_string()],
            })
            .collect())
    }

    async fn get_collaborative_candidates(
        _user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<ScoredContent>> {
        // Simulated collaborative filtering (fallback)
        // In real implementation: query similar users and their preferences
        Ok(Vec::new())
    }

    async fn get_content_based_candidates(
        _profile: &UserProfile,
        limit: usize,
    ) -> Result<Vec<ScoredContent>> {
        // Simulated content-based filtering
        // In real implementation: find content similar to user's history
        Ok(Vec::new())
    }

    async fn get_graph_based_candidates(
        user_id: Uuid,
        db_pool: Option<&PgPool>,
        limit: usize,
    ) -> Result<Vec<ScoredContent>> {
        if let Some(pool) = db_pool {
            let graph_recommender = GraphRecommender::new(pool.clone());
            let recommendations = graph_recommender.recommend(user_id, limit).await?;

            Ok(recommendations
                .into_iter()
                .map(|(content_id, score)| ScoredContent {
                    content_id,
                    score,
                    source: RecommendationType::GraphBased,
                    based_on: vec!["graph_similarity".to_string()],
                })
                .collect())
        } else {
            // Fallback if no DB pool provided
            Ok(Vec::new())
        }
    }

    async fn get_context_aware_candidates(
        _profile: &UserProfile,
        _context: &RecommendationContext,
        limit: usize,
    ) -> Result<Vec<ScoredContent>> {
        // Simulated context-aware filtering
        // In real implementation: filter by time, device, mood
        Ok(Vec::new())
    }

    fn merge_candidates(candidates: Vec<ScoredContent>) -> Vec<ScoredContent> {
        use std::collections::HashMap;

        let mut merged: HashMap<Uuid, ScoredContent> = HashMap::new();

        for candidate in candidates {
            merged
                .entry(candidate.content_id)
                .and_modify(|existing| {
                    existing.score += candidate.score;
                    existing.based_on.extend(candidate.based_on.clone());
                })
                .or_insert(candidate);
        }

        merged.into_values().collect()
    }

    async fn get_watched_content_ids(_user_id: Uuid) -> Result<Vec<Uuid>> {
        // Simulated watched content lookup
        // In real implementation: query user's viewing history
        Ok(Vec::new())
    }

    fn generate_explanation(result: &ScoredContent, _profile: &UserProfile) -> String {
        if result.based_on.is_empty() {
            "Recommended for you".to_string()
        } else {
            format!("Based on: {}", result.based_on.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::UserProfile;

    #[tokio::test]
    async fn test_merge_candidates() {
        let content_id = Uuid::new_v4();
        let candidates = vec![
            ScoredContent {
                content_id,
                score: 0.5,
                source: RecommendationType::Collaborative,
                based_on: vec!["user1".to_string()],
            },
            ScoredContent {
                content_id,
                score: 0.3,
                source: RecommendationType::ContentBased,
                based_on: vec!["genre_match".to_string()],
            },
        ];

        let merged = GenerateRecommendations::merge_candidates(candidates);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].score, 0.8);
        assert_eq!(merged[0].based_on.len(), 2);
    }

    #[tokio::test]
    async fn test_get_collaborative_candidates_fallback() {
        let user_id = Uuid::new_v4();
        let candidates = GenerateRecommendations::get_collaborative_candidates(user_id, 10)
            .await
            .unwrap();
        assert_eq!(candidates.len(), 0); // Fallback returns empty
    }
}
