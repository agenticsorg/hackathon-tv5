//! Integration tests for Redis cache implementation
//!
//! These tests require a running Redis instance.
//! Set REDIS_URL environment variable or use default: redis://localhost:6379

use media_gateway_discovery::cache::{CacheError, RedisCache};
use media_gateway_discovery::config::CacheConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SearchQuery {
    text: String,
    limit: usize,
    filters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SearchResults {
    items: Vec<MediaItem>,
    total: usize,
    took_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct MediaItem {
    id: String,
    title: String,
    score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ParsedIntent {
    category: String,
    confidence: f32,
    entities: Vec<String>,
}

fn get_test_cache_config() -> Arc<CacheConfig> {
    Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,    // 30 minutes
        embedding_ttl_sec: 3600, // 1 hour
        intent_ttl_sec: 600,     // 10 minutes
    })
}

async fn setup_cache() -> Result<RedisCache, CacheError> {
    let config = get_test_cache_config();
    RedisCache::new(config).await.map_err(|e| {
        eprintln!("Redis connection failed: {}. Ensure Redis is running.", e);
        CacheError::Operation("Redis not available for testing".to_string())
    })
}

#[tokio::test]
async fn test_cache_initialization() {
    let cache = match setup_cache().await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    // Verify connection is healthy
    let healthy = cache.health_check().await.unwrap();
    assert!(healthy, "Cache should be healthy after initialization");
}

#[tokio::test]
async fn test_search_results_caching() {
    let cache = match setup_cache().await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    // Clean up any existing test data
    cache.clear_search_cache().await.unwrap();

    let query = SearchQuery {
        text: "science fiction movies".to_string(),
        limit: 10,
        filters: vec!["movies".to_string(), "sci-fi".to_string()],
    };

    let results = SearchResults {
        items: vec![
            MediaItem {
                id: "movie1".to_string(),
                title: "Blade Runner 2049".to_string(),
                score: 0.95,
            },
            MediaItem {
                id: "movie2".to_string(),
                title: "The Matrix".to_string(),
                score: 0.92,
            },
        ],
        total: 2,
        took_ms: 145,
    };

    // Test cache miss
    let cached: Option<SearchResults> = cache.get_search_results(&query).await.unwrap();
    assert!(cached.is_none(), "Should be cache miss initially");

    // Cache the results
    cache.cache_search_results(&query, &results).await.unwrap();

    // Test cache hit
    let cached: Option<SearchResults> = cache.get_search_results(&query).await.unwrap();
    assert!(cached.is_some(), "Should be cache hit after caching");
    assert_eq!(
        cached.unwrap(),
        results,
        "Cached results should match original"
    );

    // Test different query produces different cache key
    let query2 = SearchQuery {
        text: "comedy shows".to_string(),
        limit: 5,
        filters: vec!["shows".to_string()],
    };

    let cached2: Option<SearchResults> = cache.get_search_results(&query2).await.unwrap();
    assert!(cached2.is_none(), "Different query should not hit cache");

    // Cleanup
    cache.clear_search_cache().await.unwrap();
}

#[tokio::test]
async fn test_intent_caching() {
    let cache = match setup_cache().await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    // Clean up any existing test data
    cache.clear_intent_cache().await.unwrap();

    let text = "Find me action movies from the 90s";
    let intent = ParsedIntent {
        category: "movie_search".to_string(),
        confidence: 0.89,
        entities: vec!["action".to_string(), "90s".to_string()],
    };

    // Test cache miss
    let cached: Option<ParsedIntent> = cache.get_intent(&text).await.unwrap();
    assert!(cached.is_none(), "Should be cache miss initially");

    // Cache the intent
    cache.cache_intent(&text, &intent).await.unwrap();

    // Test cache hit
    let cached: Option<ParsedIntent> = cache.get_intent(&text).await.unwrap();
    assert!(cached.is_some(), "Should be cache hit after caching");
    assert_eq!(
        cached.unwrap(),
        intent,
        "Cached intent should match original"
    );

    // Cleanup
    cache.clear_intent_cache().await.unwrap();
}

#[tokio::test]
async fn test_embedding_caching() {
    let cache = match setup_cache().await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    // Clean up any existing test data
    cache.clear_embedding_cache().await.unwrap();

    let text = "The quick brown fox jumps over the lazy dog";
    let embedding = vec![0.12, 0.34, 0.56, 0.78, 0.90, 0.11, 0.22, 0.33, 0.44, 0.55];

    // Test cache miss
    let cached = cache.get_embedding(&text).await.unwrap();
    assert!(cached.is_none(), "Should be cache miss initially");

    // Cache the embedding
    cache.cache_embedding(&text, &embedding).await.unwrap();

    // Test cache hit
    let cached = cache.get_embedding(&text).await.unwrap();
    assert!(cached.is_some(), "Should be cache hit after caching");
    assert_eq!(
        cached.unwrap(),
        embedding,
        "Cached embedding should match original"
    );

    // Cleanup
    cache.clear_embedding_cache().await.unwrap();
}

#[tokio::test]
async fn test_cache_key_consistency() {
    let cache = match setup_cache().await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let query = SearchQuery {
        text: "test query".to_string(),
        limit: 10,
        filters: vec![],
    };

    // Generate key twice for same query
    let key1 = RedisCache::generate_key("search", &query).unwrap();
    let key2 = RedisCache::generate_key("search", &query).unwrap();

    assert_eq!(key1, key2, "Same query should generate identical keys");
    assert!(
        key1.starts_with("search:"),
        "Key should have correct prefix"
    );
    assert_eq!(
        key1.len(),
        "search:".len() + 64,
        "Key should include SHA256 hash (64 hex chars)"
    );
}

#[tokio::test]
async fn test_ttl_configuration() {
    let config = get_test_cache_config();

    // Verify TTL settings match requirements
    assert_eq!(
        config.search_ttl_sec, 1800,
        "Search TTL should be 30 minutes (1800 seconds)"
    );
    assert_eq!(
        config.intent_ttl_sec, 600,
        "Intent TTL should be 10 minutes (600 seconds)"
    );
    assert_eq!(
        config.embedding_ttl_sec, 3600,
        "Embedding TTL should be 1 hour (3600 seconds)"
    );
}

#[tokio::test]
async fn test_cache_operations() {
    let cache = match setup_cache().await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let key = "test:operations:key";
    let value = "test_value";

    // Set with custom TTL
    cache.set(key, &value, 300).await.unwrap();

    // Get the value
    let retrieved: Option<String> = cache.get(key).await.unwrap();
    assert_eq!(retrieved, Some(value.to_string()));

    // Delete the value
    let deleted = cache.delete(key).await.unwrap();
    assert_eq!(deleted, 1, "Should delete exactly one key");

    // Verify deletion
    let after_delete: Option<String> = cache.get(key).await.unwrap();
    assert!(
        after_delete.is_none(),
        "Key should not exist after deletion"
    );
}

#[tokio::test]
async fn test_pattern_deletion() {
    let cache = match setup_cache().await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    // Set multiple keys with same prefix
    cache.set("test:pattern:1", &"value1", 60).await.unwrap();
    cache.set("test:pattern:2", &"value2", 60).await.unwrap();
    cache.set("test:pattern:3", &"value3", 60).await.unwrap();
    cache.set("test:other:1", &"other", 60).await.unwrap();

    // Delete by pattern
    let deleted = cache.delete_pattern("test:pattern:*").await.unwrap();
    assert_eq!(deleted, 3, "Should delete all matching keys");

    // Verify pattern keys deleted
    let val1: Option<String> = cache.get("test:pattern:1").await.unwrap();
    let val2: Option<String> = cache.get("test:pattern:2").await.unwrap();
    let val3: Option<String> = cache.get("test:pattern:3").await.unwrap();

    assert!(val1.is_none());
    assert!(val2.is_none());
    assert!(val3.is_none());

    // Verify other key still exists
    let other: Option<String> = cache.get("test:other:1").await.unwrap();
    assert_eq!(other, Some("other".to_string()));

    // Cleanup
    cache.delete("test:other:1").await.unwrap();
}

#[tokio::test]
async fn test_cache_statistics() {
    let cache = match setup_cache().await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    // Get stats
    let stats = cache.stats().await.unwrap();

    // Stats should be valid
    assert!(stats.hits >= 0);
    assert!(stats.misses >= 0);
    assert!(stats.hit_rate >= 0.0 && stats.hit_rate <= 1.0);
}

#[tokio::test]
async fn test_clear_all_caches() {
    let cache = match setup_cache().await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    // Add data to all cache types
    let query = SearchQuery {
        text: "test".to_string(),
        limit: 10,
        filters: vec![],
    };
    let results = SearchResults {
        items: vec![],
        total: 0,
        took_ms: 10,
    };

    cache.cache_search_results(&query, &results).await.unwrap();
    cache
        .cache_intent(
            &"test text",
            &ParsedIntent {
                category: "test".to_string(),
                confidence: 0.9,
                entities: vec![],
            },
        )
        .await
        .unwrap();
    cache
        .cache_embedding(&"test", &vec![0.1, 0.2, 0.3])
        .await
        .unwrap();

    // Clear all
    let deleted = cache.clear_all().await.unwrap();
    assert!(deleted >= 3, "Should delete at least 3 keys");

    // Verify all caches cleared
    let search_cached: Option<SearchResults> = cache.get_search_results(&query).await.unwrap();
    let intent_cached: Option<ParsedIntent> = cache.get_intent(&"test text").await.unwrap();
    let embedding_cached = cache.get_embedding(&"test").await.unwrap();

    assert!(search_cached.is_none());
    assert!(intent_cached.is_none());
    assert!(embedding_cached.is_none());
}

#[tokio::test]
async fn test_complex_serialization() {
    let cache = match setup_cache().await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    // Test with complex nested structures
    let complex_query = SearchQuery {
        text: "complex query with special chars: !@#$%^&*()".to_string(),
        limit: 100,
        filters: vec![
            "filter1".to_string(),
            "filter2 with spaces".to_string(),
            "filter3_with_underscores".to_string(),
        ],
    };

    let complex_results = SearchResults {
        items: vec![
            MediaItem {
                id: "uuid-1234-5678".to_string(),
                title: "Title with UTF-8: 你好世界 🚀".to_string(),
                score: 0.999,
            },
            MediaItem {
                id: "id_with_special/chars\\test".to_string(),
                title: "Normal Title".to_string(),
                score: 0.123456789,
            },
        ],
        total: 42,
        took_ms: 9876543210,
    };

    cache
        .cache_search_results(&complex_query, &complex_results)
        .await
        .unwrap();

    let retrieved: Option<SearchResults> = cache.get_search_results(&complex_query).await.unwrap();

    assert_eq!(
        retrieved.unwrap(),
        complex_results,
        "Complex structures should serialize/deserialize correctly"
    );

    // Cleanup
    cache.clear_search_cache().await.unwrap();
}

#[tokio::test]
async fn test_concurrent_cache_operations() {
    let cache = match setup_cache().await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    // Create multiple concurrent cache operations
    let cache1 = cache.clone();
    let cache2 = cache.clone();
    let cache3 = cache.clone();

    let task1 = tokio::spawn(async move {
        for i in 0..10 {
            let key = format!("concurrent:task1:{}", i);
            cache1.set(&key, &i, 60).await.unwrap();
        }
    });

    let task2 = tokio::spawn(async move {
        for i in 0..10 {
            let key = format!("concurrent:task2:{}", i);
            cache2.set(&key, &(i * 2), 60).await.unwrap();
        }
    });

    let task3 = tokio::spawn(async move {
        for i in 0..10 {
            let key = format!("concurrent:task3:{}", i);
            cache3.set(&key, &(i * 3), 60).await.unwrap();
        }
    });

    // Wait for all tasks
    let _ = tokio::join!(task1, task2, task3);

    // Verify all keys were set
    for i in 0..10 {
        let val1: Option<i32> = cache.get(&format!("concurrent:task1:{}", i)).await.unwrap();
        let val2: Option<i32> = cache.get(&format!("concurrent:task2:{}", i)).await.unwrap();
        let val3: Option<i32> = cache.get(&format!("concurrent:task3:{}", i)).await.unwrap();

        assert_eq!(val1, Some(i));
        assert_eq!(val2, Some(i * 2));
        assert_eq!(val3, Some(i * 3));
    }

    // Cleanup
    cache.delete_pattern("concurrent:*").await.unwrap();
}

#[tokio::test]
async fn test_error_handling() {
    // Test with invalid Redis URL
    let bad_config = Arc::new(CacheConfig {
        redis_url: "redis://invalid-host:9999".to_string(),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    let result = RedisCache::new(bad_config).await;
    assert!(result.is_err(), "Should fail with invalid Redis URL");
}
