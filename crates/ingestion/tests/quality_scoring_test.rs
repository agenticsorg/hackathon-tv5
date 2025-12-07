use chrono::Utc;
use media_gateway_ingestion::normalizer::{
    AvailabilityInfo, CanonicalContent, ContentType, ImageSet,
};
use media_gateway_ingestion::quality::{
    batch_score_content, generate_quality_report, QualityScorer, QualityWeights,
};
use std::collections::HashMap;

fn create_complete_content() -> CanonicalContent {
    let mut external_ids = HashMap::new();
    external_ids.insert("imdb_id".to_string(), "tt1234567".to_string());
    external_ids.insert("tmdb_id".to_string(), "12345".to_string());

    CanonicalContent {
        platform_id: "netflix".to_string(),
        platform_content_id: "nf123".to_string(),
        content_type: ContentType::Movie,
        title: "High Quality Movie".to_string(),
        overview: Some("This is a detailed and comprehensive overview of a great movie with lots of information.".to_string()),
        release_year: Some(2023),
        runtime_minutes: Some(120),
        genres: vec!["Action".to_string(), "Thriller".to_string()],
        rating: Some("PG-13".to_string()),
        user_rating: Some(8.5),
        images: ImageSet {
            poster_small: Some("http://example.com/poster-small.jpg".to_string()),
            poster_medium: Some("http://example.com/poster-medium.jpg".to_string()),
            poster_large: Some("http://example.com/poster-large.jpg".to_string()),
            backdrop: Some("http://example.com/backdrop.jpg".to_string()),
        },
        external_ids,
        availability: AvailabilityInfo {
            regions: vec!["US".to_string()],
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

fn create_minimal_content() -> CanonicalContent {
    CanonicalContent {
        platform_id: "test".to_string(),
        platform_content_id: "test123".to_string(),
        content_type: ContentType::Movie,
        title: "Minimal Movie".to_string(),
        overview: None,
        release_year: None,
        runtime_minutes: None,
        genres: vec![],
        rating: None,
        user_rating: None,
        images: ImageSet::default(),
        external_ids: HashMap::new(),
        availability: AvailabilityInfo {
            regions: vec![],
            subscription_required: false,
            purchase_price: None,
            rental_price: None,
            currency: None,
            available_from: None,
            available_until: None,
        },
        entity_id: None,
        embedding: None,
        updated_at: Utc::now(),
    }
}

#[test]
fn test_score_content_complete() {
    let scorer = QualityScorer::default();
    let content = create_complete_content();

    let score = scorer.score_content(&content);

    // Complete content should have high score (close to 1.0)
    assert!(
        score >= 0.8,
        "Complete content should score >= 0.8, got {}",
        score
    );
    assert!(score <= 1.0, "Score should not exceed 1.0, got {}", score);
}

#[test]
fn test_score_content_minimal() {
    let scorer = QualityScorer::default();
    let content = create_minimal_content();

    let score = scorer.score_content(&content);

    // Minimal content should have low score
    assert!(
        score < 0.3,
        "Minimal content should score < 0.3, got {}",
        score
    );
    assert!(score >= 0.0, "Score should not be negative, got {}", score);
}

#[test]
fn test_score_with_partial_data() {
    let scorer = QualityScorer::default();
    let mut content = create_minimal_content();

    // Add some metadata
    content.overview = Some("A good movie".to_string());
    content.release_year = Some(2023);
    content.genres = vec!["Action".to_string()];

    let score = scorer.score_content(&content);

    // Partial content should have medium score
    assert!(
        score >= 0.3,
        "Partial content should score >= 0.3, got {}",
        score
    );
    assert!(
        score < 0.6,
        "Partial content should score < 0.6, got {}",
        score
    );
}

#[test]
fn test_custom_weights() {
    // Create scorer with custom weights (prioritize description)
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
    let mut content = create_minimal_content();
    content.overview = Some("Great description".to_string());
    content.images.poster_medium = Some("http://example.com/poster.jpg".to_string());

    let score = scorer.score_content(&content);

    // With custom weights, should score exactly 1.0 (0.5 + 0.5)
    assert!(
        (score - 1.0).abs() < 0.01,
        "Custom weights should give score of 1.0, got {}",
        score
    );
}

#[test]
fn test_batch_score_content() {
    let scorer = QualityScorer::default();
    let now = Utc::now();

    let content_items = vec![
        (create_complete_content(), now),
        (create_minimal_content(), now),
    ];

    let scores = futures::executor::block_on(batch_score_content(&scorer, content_items));

    assert_eq!(scores.len(), 2);
    assert!(
        scores[0].1 > scores[1].1,
        "Complete content should score higher than minimal"
    );
}

#[test]
fn test_quality_report_generation() {
    let complete = create_complete_content();
    let minimal = create_minimal_content();

    let content_items = vec![(complete.clone(), 0.9), (minimal.clone(), 0.2)];

    let report = generate_quality_report(content_items, 0.6);

    assert_eq!(report.total_content, 2);
    assert_eq!(report.low_quality_content.len(), 1); // Only minimal should be below threshold
    assert!(report.average_score > 0.0);
    assert!(!report.score_distribution.is_empty());
}

#[test]
fn test_quality_score_distribution() {
    let content_items = vec![
        (create_minimal_content(), 0.1),
        (create_minimal_content(), 0.3),
        (create_complete_content(), 0.7),
        (create_complete_content(), 0.9),
    ];

    let report = generate_quality_report(content_items, 0.5);

    assert_eq!(report.total_content, 4);
    assert_eq!(report.average_score, 0.5); // (0.1 + 0.3 + 0.7 + 0.9) / 4

    // Check distribution buckets
    let distribution = &report.score_distribution;
    assert!(distribution.iter().any(|d| d.range == "0.0-0.2"));
    assert!(distribution.iter().any(|d| d.range == "0.8-1.0"));
}

#[test]
fn test_missing_fields_summary() {
    let mut minimal1 = create_minimal_content();
    minimal1.title = "Movie 1".to_string();

    let mut minimal2 = create_minimal_content();
    minimal2.title = "Movie 2".to_string();

    let content_items = vec![(minimal1, 0.1), (minimal2, 0.2)];

    let report = generate_quality_report(content_items, 0.5);

    assert!(!report.missing_fields_summary.is_empty());

    // Both items should be missing overview, so count should be 2
    let overview_missing = report
        .missing_fields_summary
        .iter()
        .find(|f| f.field == "overview");

    assert!(overview_missing.is_some());
    assert_eq!(overview_missing.unwrap().missing_count, 2);
}

#[test]
fn test_freshness_decay() {
    use chrono::Duration;
    use media_gateway_ingestion::quality::canonical_adapter::score_canonical_with_decay;

    let scorer = QualityScorer::default();
    let content = create_complete_content();

    // Recent update (today)
    let recent_score = score_canonical_with_decay(&content, Utc::now(), &scorer.weights);

    // Old update (1 year ago)
    let old_date = Utc::now() - Duration::days(365);
    let old_score = score_canonical_with_decay(&content, old_date, &scorer.weights);

    // Recent content should score higher due to freshness
    assert!(
        recent_score > old_score,
        "Recent content should score higher than old content"
    );
}

#[test]
fn test_image_quality_scoring() {
    let scorer = QualityScorer::default();

    // Content with high-res images
    let mut content_high_res = create_minimal_content();
    content_high_res.images.poster_large = Some("http://example.com/poster-4k.jpg".to_string());
    content_high_res.images.backdrop = Some("http://example.com/backdrop-4k.jpg".to_string());

    // Content with only medium-res images
    let mut content_medium_res = create_minimal_content();
    content_medium_res.images.poster_medium = Some("http://example.com/poster.jpg".to_string());

    let high_res_score = scorer.score_content(&content_high_res);
    let medium_res_score = scorer.score_content(&content_medium_res);

    // High-res should score higher
    assert!(
        high_res_score > medium_res_score,
        "High-res images should score higher"
    );
}

#[test]
fn test_external_ratings_scoring() {
    let scorer = QualityScorer::default();

    // Content with IMDB rating
    let mut content_with_rating = create_minimal_content();
    content_with_rating.user_rating = Some(8.5);
    content_with_rating
        .external_ids
        .insert("imdb_id".to_string(), "tt1234567".to_string());

    // Content without rating
    let content_without_rating = create_minimal_content();

    let with_rating_score = scorer.score_content(&content_with_rating);
    let without_rating_score = scorer.score_content(&content_without_rating);

    // Content with external ratings should score higher
    assert!(
        with_rating_score > without_rating_score,
        "Content with external ratings should score higher"
    );
}

#[test]
fn test_metadata_completeness_dimensions() {
    let scorer = QualityScorer::default();
    let mut content = create_minimal_content();

    let base_score = scorer.score_content(&content);

    // Add description
    content.overview = Some("A great movie about adventure".to_string());
    let with_description = scorer.score_content(&content);
    assert!(
        with_description > base_score,
        "Adding description should increase score"
    );

    // Add poster
    content.images.poster_medium = Some("http://example.com/poster.jpg".to_string());
    let with_poster = scorer.score_content(&content);
    assert!(
        with_poster > with_description,
        "Adding poster should increase score"
    );

    // Add runtime
    content.runtime_minutes = Some(120);
    let with_runtime = scorer.score_content(&content);
    assert!(
        with_runtime > with_poster,
        "Adding runtime should increase score"
    );
}

#[test]
fn test_score_clamping() {
    // Test that scores are always between 0.0 and 1.0
    let scorer = QualityScorer::default();

    let content_items = vec![create_complete_content(), create_minimal_content()];

    for content in content_items {
        let score = scorer.score_content(&content);
        assert!(score >= 0.0, "Score should be >= 0.0, got {}", score);
        assert!(score <= 1.0, "Score should be <= 1.0, got {}", score);
    }
}

#[test]
fn test_low_quality_threshold() {
    let complete = create_complete_content();
    let minimal = create_minimal_content();

    let content_items = vec![(complete.clone(), 0.9), (minimal.clone(), 0.2)];

    // Test with strict threshold (0.8)
    let strict_report = generate_quality_report(content_items.clone(), 0.8);
    assert_eq!(strict_report.low_quality_content.len(), 1); // Only 0.2 is below 0.8

    // Test with lenient threshold (0.5)
    let lenient_report = generate_quality_report(content_items, 0.5);
    assert_eq!(lenient_report.low_quality_content.len(), 1); // Only 0.2 is below 0.5
}
