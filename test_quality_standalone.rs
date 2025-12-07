// Standalone test file to verify quality module compiles independently
// Run with: rustc --test test_quality_standalone.rs

use std::collections::HashMap;

#[derive(Debug, Clone)]
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
}

impl Default for QualityScorer {
    fn default() -> Self {
        Self {
            weights: QualityWeights::default(),
        }
    }
}

impl QualityScorer {
    pub fn new(weights: QualityWeights) -> Self {
        Self { weights }
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
    fn test_scorer_creation() {
        let scorer = QualityScorer::default();
        assert!(scorer.weights.has_description > 0.0);
        assert!(scorer.weights.freshness_weight > 0.0);
    }
}
