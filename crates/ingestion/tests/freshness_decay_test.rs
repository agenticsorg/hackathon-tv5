use chrono::{Duration, Utc};
use media_gateway_ingestion::normalizer::{
    AvailabilityInfo, CanonicalContent, ContentType, ImageSet,
};
use media_gateway_ingestion::quality::{FreshnessDecay, QualityScorer, QualityWeights};
use std::collections::HashMap;

fn create_high_quality_content() -> CanonicalContent {
    let mut external_ids = HashMap::new();
    external_ids.insert("imdb_id".to_string(), "tt1234567".to_string());
    external_ids.insert("tmdb_id".to_string(), "12345".to_string());

    CanonicalContent {
        platform_id: "test".to_string(),
        platform_content_id: "test123".to_string(),
        content_type: ContentType::Movie,
        title: "High Quality Movie".to_string(),
        overview: Some("This is a comprehensive overview of a high quality movie with detailed plot description.".to_string()),
        release_year: Some(2023),
        runtime_minutes: Some(120),
        genres: vec!["Action".to_string(), "Thriller".to_string()],
        rating: None,
        user_rating: Some(8.5),
        images: ImageSet {
            poster_small: Some("http://example.com/poster-small.jpg".to_string()),
            poster_medium: Some("http://example.com/poster-medium.jpg".to_string()),
            poster_large: Some("http://example.com/poster-large.jpg".to_string()),
            backdrop: Some("http://example.com/backdrop.jpg".to_string()),
        },
        external_ids,
        availability: AvailabilityInfo {
            regions: vec!["US".to_string(), "UK".to_string()],
            subscription_required: true,
            purchase_price: None,
            rental_price: None,
            currency: Some("USD".to_string()),
            available_from: None,
            available_until: None,
        },
        entity_id: None,
        embedding: None,
        updated_at: Utc::now(),
    }
}

#[test]
fn test_freshness_decay_default_config() {
    let decay = FreshnessDecay::default();
    assert_eq!(decay.decay_rate, 0.01);
    assert_eq!(decay.min_score_ratio, 0.5);
}

#[test]
fn test_freshness_decay_custom_config() {
    let decay = FreshnessDecay::new(0.02, 0.6);
    assert_eq!(decay.decay_rate, 0.02);
    assert_eq!(decay.min_score_ratio, 0.6);
}

#[test]
fn test_decay_calculation_fresh_content() {
    let decay = FreshnessDecay::default();
    let base_score = 0.9;

    let score = decay.calculate_decay(base_score, 0.0);
    assert!(
        (score - 0.9).abs() < 0.001,
        "Fresh content should maintain full score"
    );
}

#[test]
fn test_decay_calculation_30_days() {
    let decay = FreshnessDecay::default();
    let base_score = 0.8;

    let score = decay.calculate_decay(base_score, 30.0);
    let expected = 0.8 * (-0.01 * 30.0_f64).exp() as f32;

    assert!((score - expected).abs() < 0.01);
    assert!(score < 0.8, "Score should decay after 30 days");
    assert!(
        score > 0.4,
        "Score should not hit minimum cap after 30 days"
    );
}

#[test]
fn test_decay_calculation_90_days() {
    let decay = FreshnessDecay::default();
    let base_score = 1.0;

    let score = decay.calculate_decay(base_score, 90.0);
    let expected = 1.0 * (-0.01 * 90.0_f64).exp() as f32;

    assert!((score - expected).abs() < 0.01);
    assert!(score < 1.0, "Score should decay after 90 days");
    assert!(
        score > 0.5,
        "Score should not hit minimum cap after 90 days"
    );
}

#[test]
fn test_decay_calculation_365_days() {
    let decay = FreshnessDecay::default();
    let base_score = 0.8;

    let score = decay.calculate_decay(base_score, 365.0);
    let min_score = 0.8 * 0.5;

    assert!(
        (score - min_score).abs() < 0.05,
        "Score should be near minimum after 1 year"
    );
}

#[test]
fn test_decay_minimum_cap() {
    let decay = FreshnessDecay::default();
    let base_score = 0.8;

    let score_very_old = decay.calculate_decay(base_score, 10000.0);
    let expected_min = 0.8 * 0.5;

    assert_eq!(
        score_very_old, expected_min,
        "Score should hit minimum cap for very old content"
    );
}

#[test]
fn test_decay_with_different_base_scores() {
    let decay = FreshnessDecay::default();
    let days = 100.0;

    let score_high = decay.calculate_decay(0.9, days);
    let score_low = decay.calculate_decay(0.5, days);

    assert!(
        score_high > score_low,
        "Higher base scores should maintain higher decayed scores"
    );
    assert!(
        score_high >= 0.45,
        "High quality content minimum should be 0.45"
    );
    assert!(
        score_low >= 0.25,
        "Low quality content minimum should be 0.25"
    );
}

#[test]
fn test_scorer_with_freshness_decay() {
    let weights = QualityWeights::default();
    let decay = FreshnessDecay::default();
    let scorer = QualityScorer::new_with_decay(weights, decay);

    let content = create_high_quality_content();
    let base_score = scorer.score_content(&content);

    assert!(
        base_score > 0.7,
        "High quality content should have high base score"
    );
}

#[test]
fn test_scorer_freshness_recent_content() {
    let weights = QualityWeights::default();
    let decay = FreshnessDecay::default();
    let scorer = QualityScorer::new_with_decay(weights, decay);

    let content = create_high_quality_content();
    let recent_date = Utc::now() - Duration::days(1);

    let score = scorer.score_content_with_freshness(&content, recent_date);
    let base_score = scorer.score_content(&content);

    assert!(
        (score - base_score).abs() < 0.05,
        "Recent content should have score close to base"
    );
}

#[test]
fn test_scorer_freshness_old_content() {
    let weights = QualityWeights::default();
    let decay = FreshnessDecay::default();
    let scorer = QualityScorer::new_with_decay(weights, decay);

    let content = create_high_quality_content();
    let old_date = Utc::now() - Duration::days(365);

    let score = scorer.score_content_with_freshness(&content, old_date);
    let base_score = scorer.score_content(&content);

    assert!(
        score < base_score,
        "Old content should have lower score than base"
    );
    assert!(
        score >= base_score * 0.5,
        "Score should not go below 50% of base"
    );
}

#[test]
fn test_scorer_freshness_very_old_content() {
    let weights = QualityWeights::default();
    let decay = FreshnessDecay::default();
    let scorer = QualityScorer::new_with_decay(weights, decay);

    let content = create_high_quality_content();
    let very_old_date = Utc::now() - Duration::days(3650);

    let score = scorer.score_content_with_freshness(&content, very_old_date);
    let base_score = scorer.score_content(&content);
    let min_score = base_score * 0.5;

    assert_eq!(score, min_score, "Very old content should hit minimum cap");
}

#[test]
fn test_custom_decay_rate() {
    let weights = QualityWeights::default();
    let fast_decay = FreshnessDecay::new(0.02, 0.5);
    let slow_decay = FreshnessDecay::new(0.005, 0.5);

    let scorer_fast = QualityScorer::new_with_decay(weights.clone(), fast_decay);
    let scorer_slow = QualityScorer::new_with_decay(weights, slow_decay);

    let content = create_high_quality_content();
    let old_date = Utc::now() - Duration::days(100);

    let score_fast = scorer_fast.score_content_with_freshness(&content, old_date);
    let score_slow = scorer_slow.score_content_with_freshness(&content, old_date);

    assert!(
        score_fast < score_slow,
        "Faster decay rate should result in lower score"
    );
}

#[test]
fn test_custom_min_score_ratio() {
    let weights = QualityWeights::default();
    let high_min = FreshnessDecay::new(0.01, 0.7);
    let low_min = FreshnessDecay::new(0.01, 0.3);

    let scorer_high_min = QualityScorer::new_with_decay(weights.clone(), high_min);
    let scorer_low_min = QualityScorer::new_with_decay(weights, low_min);

    let content = create_high_quality_content();
    let very_old_date = Utc::now() - Duration::days(10000);

    let score_high_min = scorer_high_min.score_content_with_freshness(&content, very_old_date);
    let score_low_min = scorer_low_min.score_content_with_freshness(&content, very_old_date);
    let base_score = scorer_high_min.score_content(&content);

    assert!(
        (score_high_min - base_score * 0.7).abs() < 0.01,
        "High minimum should cap at 70%"
    );
    assert!(
        (score_low_min - base_score * 0.3).abs() < 0.01,
        "Low minimum should cap at 30%"
    );
}

#[test]
fn test_exponential_decay_formula() {
    let decay = FreshnessDecay::new(0.01, 0.5);
    let base_score = 1.0;

    let score_10 = decay.calculate_decay(base_score, 10.0);
    let score_20 = decay.calculate_decay(base_score, 20.0);
    let score_30 = decay.calculate_decay(base_score, 30.0);

    let expected_10 = (-0.01 * 10.0_f64).exp() as f32;
    let expected_20 = (-0.01 * 20.0_f64).exp() as f32;
    let expected_30 = (-0.01 * 30.0_f64).exp() as f32;

    assert!((score_10 - expected_10).abs() < 0.001);
    assert!((score_20 - expected_20).abs() < 0.001);
    assert!((score_30 - expected_30).abs() < 0.001);

    assert!(score_10 > score_20);
    assert!(score_20 > score_30);
}

#[test]
fn test_scorer_maintains_backwards_compatibility() {
    let weights = QualityWeights::default();
    let scorer_old = QualityScorer::new(weights.clone());
    let scorer_new = QualityScorer::new_with_decay(weights, FreshnessDecay::default());

    let content = create_high_quality_content();

    let score_old = scorer_old.score_content(&content);
    let score_new = scorer_new.score_content(&content);

    assert_eq!(
        score_old, score_new,
        "score_content should work the same way"
    );
}

#[test]
fn test_zero_days_equals_base_score() {
    let weights = QualityWeights::default();
    let decay = FreshnessDecay::default();
    let scorer = QualityScorer::new_with_decay(weights, decay);

    let content = create_high_quality_content();
    let now = Utc::now();

    let base_score = scorer.score_content(&content);
    let fresh_score = scorer.score_content_with_freshness(&content, now);

    assert!(
        (fresh_score - base_score).abs() < 0.001,
        "Zero days old should equal base score"
    );
}

#[test]
fn test_decay_boundary_conditions() {
    let decay = FreshnessDecay::default();

    let score_zero = decay.calculate_decay(0.0, 100.0);
    assert_eq!(score_zero, 0.0, "Zero base score should remain zero");

    let score_one = decay.calculate_decay(1.0, 0.0);
    assert_eq!(
        score_one, 1.0,
        "Perfect score with zero days should remain 1.0"
    );

    let score_negative_days = decay.calculate_decay(0.8, -10.0);
    assert!(
        score_negative_days >= 0.0 && score_negative_days <= 1.0,
        "Should handle negative days gracefully"
    );
}
