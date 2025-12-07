use media_gateway_ingestion::quality::FreshnessDecay;

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
fn test_custom_decay_rate() {
    let fast_decay = FreshnessDecay::new(0.02, 0.5);
    let slow_decay = FreshnessDecay::new(0.005, 0.5);

    let base_score = 1.0;
    let days = 100.0;

    let score_fast = fast_decay.calculate_decay(base_score, days);
    let score_slow = slow_decay.calculate_decay(base_score, days);

    assert!(
        score_fast < score_slow,
        "Faster decay rate should result in lower score"
    );
}

#[test]
fn test_custom_min_score_ratio() {
    let high_min = FreshnessDecay::new(0.01, 0.7);
    let low_min = FreshnessDecay::new(0.01, 0.3);

    let base_score = 1.0;
    let very_old_days = 10000.0;

    let score_high_min = high_min.calculate_decay(base_score, very_old_days);
    let score_low_min = low_min.calculate_decay(base_score, very_old_days);

    assert!(
        (score_high_min - 0.7).abs() < 0.01,
        "High minimum should cap at 70%"
    );
    assert!(
        (score_low_min - 0.3).abs() < 0.01,
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
