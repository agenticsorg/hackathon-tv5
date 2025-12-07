//! User Profile Embedding
//!
//! Implements BuildUserPreferenceVector algorithm from SPARC pseudocode.
//! Generates 512-dim preference vectors with temporal decay and engagement weighting.

use crate::types::{GenreAffinities, MoodState, PreferenceVector, TemporalContext, ViewingEvent};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use ndarray::{Array1, ArrayView1};
use std::collections::HashMap;
use uuid::Uuid;

const EMBEDDING_DIM: usize = 512;
const DECAY_RATE: f32 = 0.95;
const MIN_WATCH_THRESHOLD: f32 = 0.3;

/// User profile containing preference vectors and personalization data
#[derive(Debug, Clone)]
pub struct UserProfile {
    pub user_id: Uuid,
    pub preference_vector: PreferenceVector,
    pub genre_affinities: GenreAffinities,
    pub temporal_patterns: TemporalContext,
    pub mood_history: Vec<MoodState>,
    pub interaction_count: usize,
    pub last_update_time: DateTime<Utc>,
}

impl UserProfile {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            preference_vector: vec![0.0; EMBEDDING_DIM],
            genre_affinities: HashMap::new(),
            temporal_patterns: TemporalContext::default(),
            mood_history: Vec::new(),
            interaction_count: 0,
            last_update_time: Utc::now(),
        }
    }
}

/// Build user preference vector from viewing history
///
/// Algorithm: BuildUserPreferenceVector (from SPARC pseudocode Part 2)
/// - Filters events by MIN_WATCH_THRESHOLD (30% completion)
/// - Applies temporal decay (0.95^(days/30))
/// - Weights by engagement (completion, rating, rewatch)
/// - Aggregates embeddings with weighted average
/// - L2 normalizes final vector
pub struct BuildUserPreferenceVector;

impl BuildUserPreferenceVector {
    pub async fn execute(
        user_id: Uuid,
        viewing_history: &[ViewingEvent],
        get_content_embedding: impl Fn(Uuid) -> Result<Vec<f32>>,
    ) -> Result<PreferenceVector> {
        let current_time = Utc::now();

        // Filter and weight viewing events
        let mut weighted_events = Vec::new();

        for event in viewing_history {
            // Skip low-engagement content
            if event.completion_rate < MIN_WATCH_THRESHOLD {
                continue;
            }

            // Calculate temporal decay weight
            let days_since = (current_time - event.timestamp).num_days() as f32;
            let decay_weight = DECAY_RATE.powf(days_since / 30.0);

            // Calculate engagement weight
            let engagement_weight = Self::calculate_engagement_weight(event);

            // Combined weight
            let total_weight = decay_weight * engagement_weight;

            // Get content embedding
            let content_embedding = get_content_embedding(event.content_id)?;

            weighted_events.push((content_embedding, total_weight));
        }

        // Aggregate embeddings with weighted average
        if weighted_events.is_empty() {
            return Ok(vec![0.0; EMBEDDING_DIM]);
        }

        let total_weight: f32 = weighted_events.iter().map(|(_, w)| w).sum();
        let mut aggregated = Array1::<f32>::zeros(EMBEDDING_DIM);

        for (embedding, weight) in weighted_events {
            let normalized_weight = weight / total_weight;
            let emb_array = Array1::from_vec(embedding);
            aggregated = aggregated + &(emb_array * normalized_weight);
        }

        // L2 normalize
        let norm = aggregated.dot(&aggregated).sqrt();
        if norm > 0.0 {
            aggregated = aggregated / norm;
        }

        Ok(aggregated.to_vec())
    }

    /// Calculate engagement weight from viewing event
    ///
    /// Algorithm: CalculateEngagementWeight (from SPARC pseudocode Part 2)
    /// Weights:
    /// - COMPLETION_WEIGHT = 0.4
    /// - RATING_WEIGHT = 0.3
    /// - REWATCH_WEIGHT = 0.2
    /// - DISMISSAL_PENALTY = -0.5
    fn calculate_engagement_weight(event: &ViewingEvent) -> f32 {
        const COMPLETION_WEIGHT: f32 = 0.4;
        const RATING_WEIGHT: f32 = 0.3;
        const REWATCH_WEIGHT: f32 = 0.2;
        const DISMISSAL_PENALTY: f32 = -0.5;

        let mut weight = 0.0;

        // Completion rate (0.3 to 1.0 mapped to 0.5 to 1.0)
        let completion_score = 0.5 + (event.completion_rate - 0.3) / 1.4;
        weight += completion_score * COMPLETION_WEIGHT;

        // Explicit rating (if provided)
        if let Some(rating) = event.rating {
            let rating_score = (rating as f32 - 1.0) / 4.0; // 1-5 → 0-1
            weight += rating_score * RATING_WEIGHT;
        } else {
            // Implicit rating based on completion
            weight += completion_score * RATING_WEIGHT * 0.5;
        }

        // Rewatch bonus
        if event.is_rewatch {
            weight += REWATCH_WEIGHT;
        }

        // Dismissal penalty
        if event.dismissed {
            weight += DISMISSAL_PENALTY;
        }

        // Clamp to [0, 1]
        weight.max(0.0).min(1.0)
    }
}

/// Progressive personalization - update profile as user builds history
pub struct ProgressivePersonalization;

impl ProgressivePersonalization {
    pub fn update_genre_affinities(
        profile: &mut UserProfile,
        content_genres: &[String],
        engagement: f32,
    ) {
        for genre in content_genres {
            let current_affinity = profile.genre_affinities.get(genre).copied().unwrap_or(0.5);

            // Exponential moving average
            let new_affinity = current_affinity * 0.9 + engagement * 0.1;
            profile.genre_affinities.insert(genre.clone(), new_affinity);
        }
    }

    pub fn should_update_preference_vector(interaction_count: usize) -> bool {
        interaction_count % 5 == 0
    }

    pub fn should_train_lora(interaction_count: usize) -> bool {
        interaction_count >= 10 && interaction_count % 10 == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engagement_weight_calculation() {
        let event = ViewingEvent {
            content_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            completion_rate: 0.9,
            rating: Some(5),
            is_rewatch: true,
            dismissed: false,
        };

        let weight = BuildUserPreferenceVector::calculate_engagement_weight(&event);
        assert!(weight > 0.8); // High engagement
        assert!(weight <= 1.0);
    }

    #[test]
    fn test_progressive_personalization() {
        let mut profile = UserProfile::new(Uuid::new_v4());

        ProgressivePersonalization::update_genre_affinities(
            &mut profile,
            &["action".to_string(), "sci-fi".to_string()],
            0.8,
        );

        assert!(profile.genre_affinities.contains_key("action"));
        assert!(profile.genre_affinities["action"] > 0.5);
    }
}
