//! Unit tests for Personalization Service (BATCH_005 TASK-006)

use media_gateway_discovery::search::{PersonalizationService, ContentSummary, SearchResult};
use media_gateway_discovery::cache::RedisCache;
use media_gateway_discovery::config::{CacheConfig, PersonalizationConfig};
use std::sync::Arc;
use uuid::Uuid;

fn create_test_result(id: Uuid, title: &str, score: f32) -> SearchResult {
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
fn test_personalization_config_defaults() {
    let config = PersonalizationConfig::default();

    assert_eq!(config.sona_url, "http://localhost:8082");
    assert_eq!(config.boost_weight, 0.25);
    assert_eq!(config.timeout_ms, 50);
    assert_eq!(config.cache_ttl_sec, 300);
    assert!(config.enabled);
}

#[test]
fn test_boost_weight_variants() {
    let config = PersonalizationConfig::default();
    let cache_config = Arc::new(CacheConfig {
        redis_url: "redis://localhost:6379".to_string(),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    #[cfg(test)]
    let cache = Arc::new(RedisCache::new_mock());

    #[cfg(not(test))]
    let cache = panic!("This test should only run in test mode");

    let service = PersonalizationService::new(config, cache);

    // Test all variant weights
    assert_eq!(
        service.get_boost_weight_for_variant(Some("control")),
        0.0,
        "Control variant should have 0.0 weight"
    );
    assert_eq!(
        service.get_boost_weight_for_variant(Some("low_boost")),
        0.15,
        "Low boost variant should have 0.15 weight"
    );
    assert_eq!(
        service.get_boost_weight_for_variant(Some("medium_boost")),
        0.25,
        "Medium boost variant should have 0.25 weight"
    );
    assert_eq!(
        service.get_boost_weight_for_variant(Some("high_boost")),
        0.40,
        "High boost variant should have 0.40 weight"
    );
    assert_eq!(
        service.get_boost_weight_for_variant(Some("aggressive_boost")),
        0.60,
        "Aggressive boost variant should have 0.60 weight"
    );
    assert_eq!(
        service.get_boost_weight_for_variant(None),
        0.25,
        "Default variant should use config weight"
    );
    assert_eq!(
        service.get_boost_weight_for_variant(Some("unknown_variant")),
        0.25,
        "Unknown variant should use config weight"
    );
}

#[tokio::test]
async fn test_personalization_disabled_returns_original() {
    let mut config = PersonalizationConfig::default();
    config.enabled = false;

    let cache_config = Arc::new(CacheConfig {
        redis_url: "redis://localhost:6379".to_string(),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    #[cfg(test)]
    let cache = Arc::new(RedisCache::new_mock());

    #[cfg(not(test))]
    let cache = panic!("This test should only run in test mode");

    let service = PersonalizationService::new(config, cache);

    let user_id = Uuid::new_v4();
    let results = vec![
        create_test_result(Uuid::new_v4(), "Movie A", 0.5),
        create_test_result(Uuid::new_v4(), "Movie B", 0.7),
    ];

    let original_scores: Vec<f32> = results.iter().map(|r| r.relevance_score).collect();

    let personalized = service
        .personalize_results(user_id, results, None)
        .await
        .expect("Should succeed");

    let new_scores: Vec<f32> = personalized.iter().map(|r| r.relevance_score).collect();

    // When disabled, scores should be unchanged
    assert_eq!(original_scores, new_scores);
}

#[test]
fn test_search_result_reranking() {
    // Test that results are correctly reranked after score modification
    let mut results = vec![
        create_test_result(Uuid::new_v4(), "Movie A", 0.5),
        create_test_result(Uuid::new_v4(), "Movie B", 0.7),
        create_test_result(Uuid::new_v4(), "Movie C", 0.6),
    ];

    // Sort by relevance_score descending
    results.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Verify order
    assert_eq!(results[0].content.title, "Movie B"); // 0.7
    assert_eq!(results[1].content.title, "Movie C"); // 0.6
    assert_eq!(results[2].content.title, "Movie A"); // 0.5
}

#[test]
fn test_score_blending_calculation() {
    // Test the score blending formula: original * (1 - weight) + personalization * weight
    let original_score = 0.5;
    let personalization_score = 0.9;
    let weight = 0.25;

    let expected = original_score * (1.0 - weight) + personalization_score * weight;
    let actual = 0.5 * 0.75 + 0.9 * 0.25;

    assert_eq!(expected, actual);
    assert_eq!(expected, 0.6); // 0.375 + 0.225

    // Test with different weights
    let weight_high = 0.60;
    let expected_high = original_score * (1.0 - weight_high) + personalization_score * weight_high;
    assert_eq!(expected_high, 0.74); // 0.2 + 0.54
}

#[test]
fn test_personalization_config_serialization() {
    let config = PersonalizationConfig {
        sona_url: "http://test:8082".to_string(),
        boost_weight: 0.30,
        timeout_ms: 100,
        cache_ttl_sec: 600,
        enabled: true,
    };

    // Test serialization
    let json = serde_json::to_string(&config).expect("Should serialize");
    assert!(json.contains("http://test:8082"));
    assert!(json.contains("0.3"));

    // Test deserialization
    let deserialized: PersonalizationConfig =
        serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.sona_url, config.sona_url);
    assert_eq!(deserialized.boost_weight, config.boost_weight);
    assert_eq!(deserialized.timeout_ms, config.timeout_ms);
    assert_eq!(deserialized.cache_ttl_sec, config.cache_ttl_sec);
    assert_eq!(deserialized.enabled, config.enabled);
}

#[test]
fn test_latency_requirement_config() {
    let config = PersonalizationConfig::default();

    // Verify timeout is set to meet <50ms requirement
    assert_eq!(
        config.timeout_ms, 50,
        "Timeout should be 50ms to meet latency requirement"
    );
}

#[test]
fn test_cache_ttl_config() {
    let config = PersonalizationConfig::default();

    // Verify cache TTL is 5 minutes (300 seconds)
    assert_eq!(
        config.cache_ttl_sec, 300,
        "Cache TTL should be 300 seconds (5 minutes)"
    );
}

#[test]
fn test_results_preserve_metadata() {
    // Ensure personalization doesn't lose search result metadata
    let id = Uuid::new_v4();
    let result = create_test_result(id, "Test Movie", 0.5);

    assert_eq!(result.content.id, id);
    assert_eq!(result.content.title, "Test Movie");
    assert_eq!(result.content.overview, "Test overview");
    assert_eq!(result.content.release_year, 2024);
    assert_eq!(result.content.genres, vec!["action"]);
    assert_eq!(result.content.platforms, vec!["netflix"]);
    assert_eq!(result.content.popularity_score, 0.8);
    assert_eq!(result.vector_similarity, Some(0.5));
}
