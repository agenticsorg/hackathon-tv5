use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityWeights {
    pub has_description: f32,
    pub has_poster: f32,
    pub has_backdrop: f32,
    pub has_release_year: f32,
    pub has_runtime: f32,
    pub has_genres: f32,
    pub has_imdb_rating: f32,
    pub has_external_ids: f32,
    pub freshness_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessDecay {
    pub decay_rate: f64,
    pub min_score_ratio: f64,
}

impl Default for FreshnessDecay {
    fn default() -> Self {
        Self {
            decay_rate: 0.01,
            min_score_ratio: 0.5,
        }
    }
}

impl FreshnessDecay {
    pub fn new(decay_rate: f64, min_score_ratio: f64) -> Self {
        Self {
            decay_rate,
            min_score_ratio,
        }
    }

    pub fn calculate_decay(&self, base_score: f32, days_since_update: f64) -> f32 {
        let decay_factor = (-self.decay_rate * days_since_update).exp();
        let decayed_score = base_score * decay_factor as f32;
        let min_score = base_score * self.min_score_ratio as f32;
        decayed_score.max(min_score).clamp(0.0, 1.0)
    }
}

impl Default for QualityWeights {
    fn default() -> Self {
        Self {
            has_description: 0.15,
            has_poster: 0.15,
            has_backdrop: 0.10,
            has_release_year: 0.05,
            has_runtime: 0.05,
            has_genres: 0.10,
            has_imdb_rating: 0.15,
            has_external_ids: 0.10,
            freshness_weight: 0.15,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QualityScorer {
    pub weights: QualityWeights,
    pub freshness_decay: FreshnessDecay,
}

impl Default for QualityScorer {
    fn default() -> Self {
        Self {
            weights: QualityWeights::default(),
            freshness_decay: FreshnessDecay::default(),
        }
    }
}

impl QualityScorer {
    pub fn new(weights: QualityWeights) -> Self {
        Self {
            weights,
            freshness_decay: FreshnessDecay::default(),
        }
    }

    pub fn new_with_decay(weights: QualityWeights, freshness_decay: FreshnessDecay) -> Self {
        Self {
            weights,
            freshness_decay,
        }
    }

    /// Score content based on metadata completeness and quality dimensions
    ///
    /// Returns a score from 0.0 to 1.0 based on:
    /// - metadata_completeness: description, poster, runtime
    /// - image_quality: high-res images, multiple images
    /// - external_ratings: IMDB, Rotten Tomatoes ratings
    ///
    /// Note: This is a wrapper around canonical_adapter functions for
    /// backwards compatibility. For new code, use canonical_adapter directly.
    pub fn score_content(&self, content: &crate::normalizer::CanonicalContent) -> f32 {
        super::canonical_adapter::score_canonical_content(content, &self.weights)
    }

    pub fn score_content_with_freshness(
        &self,
        content: &crate::normalizer::CanonicalContent,
        last_updated_at: DateTime<Utc>,
    ) -> f32 {
        let base_score = self.score_content(content);
        let now = Utc::now();
        let days_since_update = (now - last_updated_at).num_days() as f64;
        self.freshness_decay
            .calculate_decay(base_score, days_since_update)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowQualityItem {
    pub id: String,
    pub title: String,
    pub quality_score: f32,
    pub missing_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDistribution {
    pub range: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingFieldsSummary {
    pub field: String,
    pub missing_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub total_content: u64,
    pub average_score: f32,
    pub score_distribution: Vec<ScoreDistribution>,
    pub low_quality_content: Vec<LowQualityItem>,
    pub missing_fields_summary: Vec<MissingFieldsSummary>,
}

impl QualityReport {
    pub fn new() -> Self {
        Self {
            total_content: 0,
            average_score: 0.0,
            score_distribution: vec![],
            low_quality_content: vec![],
            missing_fields_summary: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_weights() {
        let weights = QualityWeights::default();
        assert_eq!(weights.has_description, 0.15);
        assert_eq!(weights.has_poster, 0.15);
        assert_eq!(weights.freshness_weight, 0.15);
    }

    #[test]
    fn test_custom_weights() {
        let weights = QualityWeights {
            has_description: 0.5,
            has_poster: 0.5,
            has_backdrop: 0.0,
            has_release_year: 0.0,
            has_runtime: 0.0,
            has_genres: 0.0,
            has_imdb_rating: 0.0,
            has_external_ids: 0.0,
            freshness_weight: 0.0,
        };

        let scorer = QualityScorer::new(weights);
        assert_eq!(scorer.weights.has_description, 0.5);
        assert_eq!(scorer.weights.has_poster, 0.5);
    }

    #[test]
    fn test_quality_report_new() {
        let report = QualityReport::new();
        assert_eq!(report.total_content, 0);
        assert_eq!(report.average_score, 0.0);
        assert_eq!(report.low_quality_content.len(), 0);
    }

    #[test]
    fn test_freshness_decay_default() {
        let decay = FreshnessDecay::default();
        assert_eq!(decay.decay_rate, 0.01);
        assert_eq!(decay.min_score_ratio, 0.5);
    }

    #[test]
    fn test_freshness_decay_calculation() {
        let decay = FreshnessDecay::default();
        let base_score = 0.8;

        let score_0_days = decay.calculate_decay(base_score, 0.0);
        assert!((score_0_days - 0.8).abs() < 0.01);

        let score_30_days = decay.calculate_decay(base_score, 30.0);
        assert!(score_30_days < 0.8);
        assert!(score_30_days > 0.4);

        let score_365_days = decay.calculate_decay(base_score, 365.0);
        assert!((score_365_days - 0.4).abs() < 0.05);
    }

    #[test]
    fn test_freshness_decay_minimum_cap() {
        let decay = FreshnessDecay::default();
        let base_score = 0.8;

        let score_very_old = decay.calculate_decay(base_score, 10000.0);
        assert_eq!(score_very_old, 0.4);
    }

    #[test]
    fn test_custom_decay_rate() {
        let decay = FreshnessDecay::new(0.02, 0.5);
        let base_score = 1.0;

        let score_30_days = decay.calculate_decay(base_score, 30.0);
        assert!(score_30_days < 1.0);

        let score_very_old = decay.calculate_decay(base_score, 10000.0);
        assert_eq!(score_very_old, 0.5);
    }

    #[test]
    fn test_scorer_with_decay() {
        let weights = QualityWeights::default();
        let decay = FreshnessDecay::default();
        let scorer = QualityScorer::new_with_decay(weights, decay);

        assert_eq!(scorer.freshness_decay.decay_rate, 0.01);
        assert_eq!(scorer.freshness_decay.min_score_ratio, 0.5);
    }
}
