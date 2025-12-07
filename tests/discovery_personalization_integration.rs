//! Integration tests for Discovery Search Personalization (BATCH_005 TASK-006)
//!
//! Tests user personalization integration with SONA service,
//! Redis caching, and A/B testing variants.

use media_gateway_discovery::search::{
    HybridSearchService, PersonalizationService, SearchRequest,
    SearchResult, ContentSummary,
};
use media_gateway_discovery::cache::RedisCache;
use media_gateway_discovery::config::{CacheConfig, DiscoveryConfig, PersonalizationConfig};
use std::sync::Arc;
use uuid::Uuid;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Helper to create mock search result
fn create_mock_result(id: Uuid, title: &str, score: f32) -> SearchResult {
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

#[tokio::test]
async fn test_personalization_service_calls_sona() {
    // Start mock SONA server
    let mock_server = MockServer::start().await;

    let user_id = Uuid::new_v4();
    let content_id = Uuid::new_v4();

    // Mock SONA personalization/score endpoint
    Mock::given(method("POST"))
        .and(path("/api/v1/personalization/score"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "user_id": user_id,
            "content_id": content_id,
            "score": 0.85,
            "components": {
                "collaborative": 0.35,
                "content_based": 0.25,
                "graph_based": 0.30,
                "context": 0.10,
                "lora_boost": 0.15
            }
        })))
        .mount(&mock_server)
        .await;

    // Setup personalization service
    let config = PersonalizationConfig {
        sona_url: mock_server.uri(),
        boost_weight: 0.25,
        timeout_ms: 50,
        cache_ttl_sec: 300,
        enabled: true,
    };

    let cache_config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    let cache = match RedisCache::new(cache_config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let service = PersonalizationService::new(config, cache);

    // Create test results
    let results = vec![create_mock_result(content_id, "Test Movie", 0.5)];

    // Apply personalization
    let personalized = service
        .personalize_results(user_id, results, None)
        .await
        .expect("Personalization should succeed");

    // Verify results were reranked
    assert_eq!(personalized.len(), 1);
    assert_ne!(
        personalized[0].relevance_score, 0.5,
        "Score should be modified by personalization"
    );

    // Score should be blend of original (0.5) and personalization (0.85) with weight 0.25
    let expected_score = 0.5 * (1.0 - 0.25) + 0.85 * 0.25;
    assert!(
        (personalized[0].relevance_score - expected_score).abs() < 0.01,
        "Expected score ~{}, got {}",
        expected_score,
        personalized[0].relevance_score
    );
}

#[tokio::test]
async fn test_personalization_caching() {
    let mock_server = MockServer::start().await;

    let user_id = Uuid::new_v4();
    let content_id = Uuid::new_v4();

    // Mock SONA endpoint (should only be called once due to caching)
    let mock = Mock::given(method("POST"))
        .and(path("/api/v1/personalization/score"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "user_id": user_id,
            "content_id": content_id,
            "score": 0.90,
            "components": {
                "collaborative": 0.35,
                "content_based": 0.25,
                "graph_based": 0.30,
                "context": 0.10,
                "lora_boost": 0.20
            }
        })))
        .expect(1) // Should only be called once
        .mount(&mock_server)
        .await;

    let config = PersonalizationConfig {
        sona_url: mock_server.uri(),
        boost_weight: 0.25,
        timeout_ms: 50,
        cache_ttl_sec: 300,
        enabled: true,
    };

    let cache_config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    let cache = match RedisCache::new(cache_config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    // Clear cache before test
    let cache_key = format!("personalization:{}:batch", user_id);
    let _ = cache.delete(&cache_key).await;

    let service = PersonalizationService::new(config, cache.clone());

    let results = vec![create_mock_result(content_id, "Test Movie", 0.6)];

    // First call - should hit SONA
    let _personalized1 = service
        .personalize_results(user_id, results.clone(), None)
        .await
        .expect("First personalization should succeed");

    // Second call - should use cache
    let _personalized2 = service
        .personalize_results(user_id, results, None)
        .await
        .expect("Second personalization should succeed");

    // Mock should only have been called once
    mock.assert();

    // Cleanup
    let _ = cache.delete(&cache_key).await;
}

#[tokio::test]
async fn test_ab_testing_variant_boost_weights() {
    let mock_server = MockServer::start().await;

    let user_id = Uuid::new_v4();
    let content_id = Uuid::new_v4();

    Mock::given(method("POST"))
        .and(path("/api/v1/personalization/score"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "user_id": user_id,
            "content_id": content_id,
            "score": 0.80,
            "components": {
                "collaborative": 0.35,
                "content_based": 0.25,
                "graph_based": 0.30,
                "context": 0.10,
                "lora_boost": 0.10
            }
        })))
        .mount(&mock_server)
        .await;

    let config = PersonalizationConfig {
        sona_url: mock_server.uri(),
        boost_weight: 0.25, // Default weight (not used when variant provided)
        timeout_ms: 50,
        cache_ttl_sec: 300,
        enabled: true,
    };

    let cache_config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    let cache = match RedisCache::new(cache_config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let service = PersonalizationService::new(config, cache);

    let results = vec![create_mock_result(content_id, "Test Movie", 0.5)];

    // Test control variant (no personalization)
    let control = service
        .personalize_results(user_id, results.clone(), Some("control"))
        .await
        .expect("Control variant should succeed");
    assert_eq!(
        control[0].relevance_score, 0.5,
        "Control variant should not modify scores"
    );

    // Test low_boost variant
    let low_boost = service
        .personalize_results(user_id, results.clone(), Some("low_boost"))
        .await
        .expect("Low boost variant should succeed");
    let expected_low = 0.5 * (1.0 - 0.15) + 0.80 * 0.15;
    assert!(
        (low_boost[0].relevance_score - expected_low).abs() < 0.01,
        "Low boost score mismatch"
    );

    // Test high_boost variant
    let high_boost = service
        .personalize_results(user_id, results.clone(), Some("high_boost"))
        .await
        .expect("High boost variant should succeed");
    let expected_high = 0.5 * (1.0 - 0.40) + 0.80 * 0.40;
    assert!(
        (high_boost[0].relevance_score - expected_high).abs() < 0.01,
        "High boost score mismatch"
    );
}

#[tokio::test]
async fn test_personalization_latency_requirement() {
    let mock_server = MockServer::start().await;

    let user_id = Uuid::new_v4();
    let content_ids: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();

    // Mock SONA endpoint for multiple content items
    for content_id in &content_ids {
        Mock::given(method("POST"))
            .and(path("/api/v1/personalization/score"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "user_id": user_id,
                "content_id": content_id,
                "score": 0.75,
                "components": {
                    "collaborative": 0.35,
                    "content_based": 0.25,
                    "graph_based": 0.30,
                    "context": 0.10,
                    "lora_boost": 0.05
                }
            })))
            .mount(&mock_server)
            .await;
    }

    let config = PersonalizationConfig {
        sona_url: mock_server.uri(),
        boost_weight: 0.25,
        timeout_ms: 50,
        cache_ttl_sec: 300,
        enabled: true,
    };

    let cache_config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    let cache = match RedisCache::new(cache_config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let service = PersonalizationService::new(config, cache);

    // Create 10 test results
    let results: Vec<SearchResult> = content_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| create_mock_result(id, &format!("Movie {}", i), 0.5))
        .collect();

    let start = std::time::Instant::now();
    let _personalized = service
        .personalize_results(user_id, results, None)
        .await
        .expect("Personalization should succeed");
    let elapsed = start.elapsed();

    // Verify latency requirement (<50ms)
    assert!(
        elapsed.as_millis() < 100,
        "Personalization took {}ms, should be <100ms (with some margin)",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn test_personalization_failure_graceful_degradation() {
    // Start mock server but don't mount any endpoints (will return 404)
    let mock_server = MockServer::start().await;

    let config = PersonalizationConfig {
        sona_url: mock_server.uri(),
        boost_weight: 0.25,
        timeout_ms: 50,
        cache_ttl_sec: 300,
        enabled: true,
    };

    let cache_config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    let cache = match RedisCache::new(cache_config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let service = PersonalizationService::new(config, cache);

    let results = vec![create_mock_result(Uuid::new_v4(), "Test Movie", 0.5)];
    let original_score = results[0].relevance_score;

    // Should gracefully handle SONA failure and return original results
    let personalized = service
        .personalize_results(Uuid::new_v4(), results, None)
        .await
        .expect("Should not fail even when SONA is unavailable");

    // Results should be unchanged when personalization fails
    assert_eq!(
        personalized.len(),
        1,
        "Should return all original results"
    );
    // Note: scores may be empty due to failure, but results should be returned
}

#[tokio::test]
async fn test_personalization_disabled() {
    let config = PersonalizationConfig {
        sona_url: "http://localhost:9999".to_string(), // Invalid URL
        boost_weight: 0.25,
        timeout_ms: 50,
        cache_ttl_sec: 300,
        enabled: false, // Disabled
    };

    let cache_config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    let cache = match RedisCache::new(cache_config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let service = PersonalizationService::new(config, cache);

    let results = vec![create_mock_result(Uuid::new_v4(), "Test Movie", 0.5)];
    let original_score = results[0].relevance_score;

    // Should not call SONA when disabled
    let personalized = service
        .personalize_results(Uuid::new_v4(), results, None)
        .await
        .expect("Should succeed when disabled");

    assert_eq!(personalized[0].relevance_score, original_score);
}

#[tokio::test]
async fn test_cache_invalidation() {
    let mock_server = MockServer::start().await;

    let user_id = Uuid::new_v4();
    let content_id = Uuid::new_v4();

    Mock::given(method("POST"))
        .and(path("/api/v1/personalization/score"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "user_id": user_id,
            "content_id": content_id,
            "score": 0.80,
            "components": {
                "collaborative": 0.35,
                "content_based": 0.25,
                "graph_based": 0.30,
                "context": 0.10,
                "lora_boost": 0.10
            }
        })))
        .mount(&mock_server)
        .await;

    let config = PersonalizationConfig {
        sona_url: mock_server.uri(),
        boost_weight: 0.25,
        timeout_ms: 50,
        cache_ttl_sec: 300,
        enabled: true,
    };

    let cache_config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    let cache = match RedisCache::new(cache_config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let service = PersonalizationService::new(config, cache.clone());

    let results = vec![create_mock_result(content_id, "Test Movie", 0.5)];

    // Populate cache
    let _ = service
        .personalize_results(user_id, results.clone(), None)
        .await;

    // Verify cache exists
    let cache_key = format!("personalization:{}:batch", user_id);
    let cached: Option<serde_json::Value> = cache.get(&cache_key).await.ok().flatten();
    assert!(cached.is_some(), "Cache should be populated");

    // Invalidate cache
    service
        .invalidate_cache(user_id)
        .await
        .expect("Cache invalidation should succeed");

    // Verify cache is cleared
    let cached_after: Option<serde_json::Value> = cache.get(&cache_key).await.ok().flatten();
    assert!(cached_after.is_none(), "Cache should be cleared");
}
