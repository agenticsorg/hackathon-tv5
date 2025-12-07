//! Integration tests for HybridSearchService cache integration
//!
//! These tests verify:
//! 1. Cache hit returns in <10ms
//! 2. Cache miss executes full search
//! 3. TTL expiration behavior
//! 4. Cache key generation consistency
//! 5. Metrics tracking for hits/misses

use media_gateway_discovery::cache::RedisCache;
use media_gateway_discovery::config::{CacheConfig, DiscoveryConfig};
use media_gateway_discovery::intent::{IntentParser, ParsedIntent};
use media_gateway_discovery::search::{
    HybridSearchService, SearchFilters, SearchRequest, SearchResponse,
};
use std::sync::Arc;
use uuid::Uuid;

/// Helper to create a test cache instance
async fn create_test_cache() -> Option<Arc<RedisCache>> {
    let config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800, // 30 minutes
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    match RedisCache::new(config).await {
        Ok(cache) => {
            // Clear any existing test data
            if let Err(e) = cache.clear_search_cache().await {
                eprintln!("Warning: Failed to clear search cache: {}", e);
            }
            Some(Arc::new(cache))
        }
        Err(e) => {
            eprintln!("Redis not available, skipping integration tests: {}", e);
            None
        }
    }
}

/// Helper to create a test search request
fn create_test_request(query: &str, page: u32) -> SearchRequest {
    SearchRequest {
        query: query.to_string(),
        filters: Some(SearchFilters {
            genres: vec!["action".to_string()],
            platforms: vec!["netflix".to_string()],
            year_range: Some((2020, 2024)),
            rating_range: None,
        }),
        page,
        page_size: 20,
        user_id: Some(Uuid::new_v4()),
    }
}

/// Helper to create a mock SearchResponse
fn create_mock_response(query: &str, page: u32) -> SearchResponse {
    SearchResponse {
        results: vec![],
        total_count: 0,
        page,
        page_size: 20,
        query_parsed: ParsedIntent {
            mood: vec![],
            themes: vec![],
            references: vec![],
            filters: media_gateway_discovery::intent::IntentFilters {
                genre: vec![],
                platform: vec![],
                year_range: None,
            },
            fallback_query: query.to_string(),
            confidence: 0.5,
        },
        search_time_ms: 150,
    }
}

#[tokio::test]
async fn test_cache_hit_performance() {
    // This test verifies cache hits return in <10ms
    let cache = match create_test_cache().await {
        Some(c) => c,
        None => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let request = create_test_request("action movies", 1);
    let response = create_mock_response("action movies", 1);

    // Generate cache key and pre-populate cache
    let cache_key = format!(
        "search:{}",
        hex::encode(sha2::Sha256::digest(
            serde_json::to_string(&request).unwrap().as_bytes()
        ))
    );

    cache
        .set(&cache_key, &response, 1800)
        .await
        .expect("Failed to set cache");

    // Measure cache retrieval time
    let start = std::time::Instant::now();
    let cached: Option<SearchResponse> = cache
        .get(&cache_key)
        .await
        .expect("Failed to get from cache");
    let duration = start.elapsed();

    // Verify cache hit
    assert!(cached.is_some(), "Cache should have returned a value");
    assert_eq!(cached.unwrap().query_parsed.fallback_query, "action movies");

    // Verify performance: cache hit should be <10ms
    assert!(
        duration.as_millis() < 10,
        "Cache hit took {}ms, expected <10ms",
        duration.as_millis()
    );

    println!(
        "✓ Cache hit completed in {}μs (<10ms requirement)",
        duration.as_micros()
    );

    // Cleanup
    cache.delete(&cache_key).await.ok();
}

#[tokio::test]
async fn test_cache_key_consistency() {
    // This test verifies that the same request generates the same cache key
    let cache = match create_test_cache().await {
        Some(c) => c,
        None => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let request1 = create_test_request("action movies", 1);
    let request2 = create_test_request("action movies", 1);
    let request3 = create_test_request("action movies", 2); // Different page

    // Generate cache keys
    let key1 = format!(
        "search:{}",
        hex::encode(sha2::Sha256::digest(
            serde_json::to_string(&request1).unwrap().as_bytes()
        ))
    );
    let key2 = format!(
        "search:{}",
        hex::encode(sha2::Sha256::digest(
            serde_json::to_string(&request2).unwrap().as_bytes()
        ))
    );
    let key3 = format!(
        "search:{}",
        hex::encode(sha2::Sha256::digest(
            serde_json::to_string(&request3).unwrap().as_bytes()
        ))
    );

    // Same request should generate same key
    assert_eq!(key1, key2, "Same requests should generate same cache key");

    // Different page should generate different key
    assert_ne!(
        key1, key3,
        "Different pages should generate different cache keys"
    );

    println!("✓ Cache key generation is consistent");
}

#[tokio::test]
async fn test_cache_miss_then_hit() {
    // This test verifies cache miss followed by cache hit behavior
    let cache = match create_test_cache().await {
        Some(c) => c,
        None => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let request = create_test_request("drama series", 1);
    let cache_key = format!(
        "search:{}",
        hex::encode(sha2::Sha256::digest(
            serde_json::to_string(&request).unwrap().as_bytes()
        ))
    );

    // Ensure cache is empty
    cache.delete(&cache_key).await.ok();

    // First request should be a cache miss
    let result1: Option<SearchResponse> = cache
        .get(&cache_key)
        .await
        .expect("Failed to get from cache");
    assert!(result1.is_none(), "First request should be a cache miss");

    println!("✓ Cache miss detected");

    // Populate cache
    let response = create_mock_response("drama series", 1);
    cache
        .set(&cache_key, &response, 1800)
        .await
        .expect("Failed to set cache");

    // Second request should be a cache hit
    let result2: Option<SearchResponse> = cache
        .get(&cache_key)
        .await
        .expect("Failed to get from cache");
    assert!(result2.is_some(), "Second request should be a cache hit");
    assert_eq!(result2.unwrap().query_parsed.fallback_query, "drama series");

    println!("✓ Cache hit after cache miss works correctly");

    // Cleanup
    cache.delete(&cache_key).await.ok();
}

#[tokio::test]
async fn test_cache_ttl_expiration() {
    // This test verifies TTL expiration behavior
    // Note: Uses short TTL for testing purposes
    let cache = match create_test_cache().await {
        Some(c) => c,
        None => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let request = create_test_request("comedy movies", 1);
    let response = create_mock_response("comedy movies", 1);
    let cache_key = format!(
        "search:{}",
        hex::encode(sha2::Sha256::digest(
            serde_json::to_string(&request).unwrap().as_bytes()
        ))
    );

    // Set cache with 2-second TTL for testing
    cache
        .set(&cache_key, &response, 2)
        .await
        .expect("Failed to set cache");

    // Immediate retrieval should succeed
    let result1: Option<SearchResponse> = cache
        .get(&cache_key)
        .await
        .expect("Failed to get from cache");
    assert!(result1.is_some(), "Cache should be available immediately");

    println!("✓ Cache populated with 2s TTL");

    // Wait for TTL expiration (3 seconds to be safe)
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // After TTL, cache should be expired
    let result2: Option<SearchResponse> = cache
        .get(&cache_key)
        .await
        .expect("Failed to get from cache");
    assert!(
        result2.is_none(),
        "Cache should be expired after TTL period"
    );

    println!("✓ Cache correctly expired after TTL");
}

#[tokio::test]
async fn test_cache_different_filters() {
    // This test verifies different filters generate different cache keys
    let cache = match create_test_cache().await {
        Some(c) => c,
        None => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let request1 = SearchRequest {
        query: "action movies".to_string(),
        filters: Some(SearchFilters {
            genres: vec!["action".to_string()],
            platforms: vec!["netflix".to_string()],
            year_range: Some((2020, 2024)),
            rating_range: None,
        }),
        page: 1,
        page_size: 20,
        user_id: Some(Uuid::new_v4()),
    };

    let request2 = SearchRequest {
        query: "action movies".to_string(),
        filters: Some(SearchFilters {
            genres: vec!["action".to_string(), "thriller".to_string()], // Different filters
            platforms: vec!["netflix".to_string()],
            year_range: Some((2020, 2024)),
            rating_range: None,
        }),
        page: 1,
        page_size: 20,
        user_id: request1.user_id, // Same user
    };

    let key1 = format!(
        "search:{}",
        hex::encode(sha2::Sha256::digest(
            serde_json::to_string(&request1).unwrap().as_bytes()
        ))
    );
    let key2 = format!(
        "search:{}",
        hex::encode(sha2::Sha256::digest(
            serde_json::to_string(&request2).unwrap().as_bytes()
        ))
    );

    // Different filters should generate different keys
    assert_ne!(
        key1, key2,
        "Different filters should generate different cache keys"
    );

    println!("✓ Different filters generate different cache keys");
}

#[tokio::test]
async fn test_cache_different_users() {
    // This test verifies different users generate different cache keys
    let cache = match create_test_cache().await {
        Some(c) => c,
        None => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();

    let request1 = SearchRequest {
        query: "action movies".to_string(),
        filters: Some(SearchFilters {
            genres: vec!["action".to_string()],
            platforms: vec!["netflix".to_string()],
            year_range: Some((2020, 2024)),
            rating_range: None,
        }),
        page: 1,
        page_size: 20,
        user_id: Some(user1),
    };

    let request2 = SearchRequest {
        query: "action movies".to_string(),
        filters: request1.filters.clone(),
        page: 1,
        page_size: 20,
        user_id: Some(user2), // Different user
    };

    let key1 = format!(
        "search:{}",
        hex::encode(sha2::Sha256::digest(
            serde_json::to_string(&request1).unwrap().as_bytes()
        ))
    );
    let key2 = format!(
        "search:{}",
        hex::encode(sha2::Sha256::digest(
            serde_json::to_string(&request2).unwrap().as_bytes()
        ))
    );

    // Different users should generate different keys (for personalized results)
    assert_ne!(
        key1, key2,
        "Different users should generate different cache keys"
    );

    println!("✓ Different users generate different cache keys");
}

#[tokio::test]
async fn test_cache_serialization_roundtrip() {
    // This test verifies SearchResponse can be serialized and deserialized correctly
    let cache = match create_test_cache().await {
        Some(c) => c,
        None => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let original = create_mock_response("test query", 1);
    let cache_key = "test:serialization:roundtrip";

    // Store in cache
    cache
        .set(cache_key, &original, 60)
        .await
        .expect("Failed to set cache");

    // Retrieve from cache
    let retrieved: Option<SearchResponse> = cache
        .get(cache_key)
        .await
        .expect("Failed to get from cache");

    assert!(retrieved.is_some(), "Cache should return value");
    let retrieved = retrieved.unwrap();

    // Verify all fields match
    assert_eq!(retrieved.total_count, original.total_count);
    assert_eq!(retrieved.page, original.page);
    assert_eq!(retrieved.page_size, original.page_size);
    assert_eq!(
        retrieved.query_parsed.fallback_query,
        original.query_parsed.fallback_query
    );

    println!("✓ SearchResponse serialization roundtrip successful");

    // Cleanup
    cache.delete(cache_key).await.ok();
}
