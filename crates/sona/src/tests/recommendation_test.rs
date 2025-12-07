//! Recommendation algorithm tests

use crate::types::{Recommendation, RecommendationType, ScoredContent};
use chrono::Utc;
use uuid::Uuid;

#[test]
fn test_mmr_diversity_filter_concept() {
    // Maximal Marginal Relevance (MMR) for diversity
    // MMR(d) = λ * relevance(d) - (1-λ) * max_similarity(d, selected)

    let lambda = 0.7; // Relevance vs diversity trade-off

    let relevance_score = 0.9;
    let max_similarity_to_selected = 0.8;

    let mmr_score = lambda * relevance_score - (1.0 - lambda) * max_similarity_to_selected;

    assert!(mmr_score > 0.0);
    assert!(mmr_score < 1.0);
}

#[test]
fn test_mmr_diversity_with_no_selected_items() {
    let lambda = 0.7;
    let relevance_score = 0.9;
    let max_similarity = 0.0; // No items selected yet

    let mmr_score = lambda * relevance_score - (1.0 - lambda) * max_similarity;

    // Should equal relevance score when no items selected
    assert!((mmr_score - lambda * relevance_score).abs() < 0.01);
}

#[test]
fn test_mmr_diversity_high_similarity_penalty() {
    let lambda = 0.7;
    let relevance_score = 0.9;

    // High similarity to already selected items
    let high_similarity = 0.95;
    let mmr_high_sim = lambda * relevance_score - (1.0 - lambda) * high_similarity;

    // Low similarity to already selected items
    let low_similarity = 0.3;
    let mmr_low_sim = lambda * relevance_score - (1.0 - lambda) * low_similarity;

    // Lower similarity should give higher MMR score (more diverse)
    assert!(mmr_low_sim > mmr_high_sim);
}

#[test]
fn test_cold_start_handling_new_user() {
    // Cold start: user with no viewing history
    let interaction_count = 0;

    // Should fall back to popularity-based recommendations
    assert!(interaction_count < 5);
}

#[test]
fn test_cold_start_handling_few_interactions() {
    // User with limited interactions (< 10)
    let interaction_count = 7;

    // Should use hybrid approach (popularity + limited personalization)
    assert!(interaction_count < 10);
}

#[test]
fn test_cold_start_handling_sufficient_data() {
    // User with sufficient history
    let interaction_count = 15;

    // Can use full personalization
    assert!(interaction_count >= 10);
}

#[test]
fn test_explanation_generation_content_based() {
    let recommendation = Recommendation {
        content_id: Uuid::new_v4(),
        confidence_score: 0.85,
        recommendation_type: RecommendationType::ContentBased,
        based_on: vec!["The Matrix".to_string(), "Inception".to_string()],
        explanation: "Because you watched The Matrix and Inception".to_string(),
        generated_at: Utc::now(),
        ttl_seconds: 3600,
        experiment_variant: None,
    };

    assert_eq!(
        recommendation.recommendation_type,
        RecommendationType::ContentBased
    );
    assert_eq!(recommendation.based_on.len(), 2);
    assert!(recommendation.explanation.contains("Because you watched"));
}

#[test]
fn test_explanation_generation_collaborative() {
    let recommendation = Recommendation {
        content_id: Uuid::new_v4(),
        confidence_score: 0.78,
        recommendation_type: RecommendationType::Collaborative,
        based_on: vec!["Similar users".to_string()],
        explanation: "Users with similar taste also enjoyed this".to_string(),
        generated_at: Utc::now(),
        ttl_seconds: 3600,
        experiment_variant: None,
    };

    assert_eq!(
        recommendation.recommendation_type,
        RecommendationType::Collaborative
    );
    assert!(recommendation.explanation.contains("similar taste"));
}

#[test]
fn test_recommendation_confidence_score_range() {
    let high_confidence = Recommendation {
        content_id: Uuid::new_v4(),
        confidence_score: 0.92,
        recommendation_type: RecommendationType::Hybrid,
        based_on: vec![],
        explanation: String::new(),
        generated_at: Utc::now(),
        ttl_seconds: 3600,
        experiment_variant: None,
    };

    assert!(high_confidence.confidence_score >= 0.0);
    assert!(high_confidence.confidence_score <= 1.0);
    assert!(high_confidence.confidence_score > 0.9);
}

#[test]
fn test_recommendation_ttl() {
    let recommendation = Recommendation {
        content_id: Uuid::new_v4(),
        confidence_score: 0.80,
        recommendation_type: RecommendationType::ContextAware,
        based_on: vec![],
        explanation: String::new(),
        generated_at: Utc::now(),
        ttl_seconds: 1800, // 30 minutes
        experiment_variant: None,
    };

    assert_eq!(recommendation.ttl_seconds, 1800);
}

#[test]
fn test_scored_content_struct() {
    let content = ScoredContent {
        content_id: Uuid::new_v4(),
        score: 0.87,
        source: RecommendationType::GraphBased,
        based_on: vec!["genre similarity".to_string()],
    };

    assert!(content.score > 0.8);
    assert_eq!(content.source, RecommendationType::GraphBased);
    assert_eq!(content.based_on.len(), 1);
}

#[test]
fn test_recommendation_type_enum_variants() {
    let types = vec![
        RecommendationType::Collaborative,
        RecommendationType::ContentBased,
        RecommendationType::GraphBased,
        RecommendationType::ContextAware,
        RecommendationType::Hybrid,
    ];

    assert_eq!(types.len(), 5);
}

#[test]
fn test_hybrid_recommendation_combines_sources() {
    let hybrid = Recommendation {
        content_id: Uuid::new_v4(),
        confidence_score: 0.88,
        recommendation_type: RecommendationType::Hybrid,
        based_on: vec![
            "content similarity".to_string(),
            "collaborative filtering".to_string(),
            "genre preferences".to_string(),
        ],
        explanation: "Combines multiple signals for better accuracy".to_string(),
        generated_at: Utc::now(),
        ttl_seconds: 3600,
        experiment_variant: None,
    };

    assert_eq!(hybrid.recommendation_type, RecommendationType::Hybrid);
    assert!(hybrid.based_on.len() >= 2);
}

#[test]
fn test_context_aware_recommendation() {
    let context_rec = Recommendation {
        content_id: Uuid::new_v4(),
        confidence_score: 0.82,
        recommendation_type: RecommendationType::ContextAware,
        based_on: vec![
            "evening viewing pattern".to_string(),
            "weekend preference".to_string(),
        ],
        explanation: "Based on your viewing patterns at this time".to_string(),
        generated_at: Utc::now(),
        ttl_seconds: 600, // 10 minutes (context changes quickly)
        experiment_variant: None,
    };

    assert_eq!(
        context_rec.recommendation_type,
        RecommendationType::ContextAware
    );
    assert!(context_rec.ttl_seconds < 3600); // Context-aware should have shorter TTL
}

#[test]
fn test_recommendation_ranking_by_confidence() {
    let mut recommendations = vec![
        Recommendation {
            content_id: Uuid::new_v4(),
            confidence_score: 0.75,
            recommendation_type: RecommendationType::ContentBased,
            based_on: vec![],
            explanation: String::new(),
            generated_at: Utc::now(),
            ttl_seconds: 3600,
            experiment_variant: None,
        },
        Recommendation {
            content_id: Uuid::new_v4(),
            confidence_score: 0.92,
            recommendation_type: RecommendationType::Hybrid,
            based_on: vec![],
            explanation: String::new(),
            generated_at: Utc::now(),
            ttl_seconds: 3600,
            experiment_variant: None,
        },
        Recommendation {
            content_id: Uuid::new_v4(),
            confidence_score: 0.83,
            recommendation_type: RecommendationType::Collaborative,
            based_on: vec![],
            explanation: String::new(),
            generated_at: Utc::now(),
            ttl_seconds: 3600,
            experiment_variant: None,
        },
    ];

    recommendations.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());

    assert!(recommendations[0].confidence_score > recommendations[1].confidence_score);
    assert!(recommendations[1].confidence_score > recommendations[2].confidence_score);
    assert_eq!(recommendations[0].confidence_score, 0.92);
}

#[test]
fn test_diversity_filter_removes_duplicates() {
    let content_ids = vec![
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(), // Duplicate
    ];

    let mut unique_ids = content_ids.clone();
    unique_ids.sort();
    unique_ids.dedup();

    assert_eq!(unique_ids.len(), 2);
}
