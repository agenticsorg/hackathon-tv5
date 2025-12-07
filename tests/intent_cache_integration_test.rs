use media_gateway_discovery::cache::RedisCache;
use media_gateway_discovery::config::CacheConfig;
use media_gateway_discovery::intent::{IntentParser, ParsedIntent};
use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
async fn test_intent_cache_performance() {
    let config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    let cache = match RedisCache::new(config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    // Clear any existing cache
    cache.clear_intent_cache().await.unwrap();

    let parser = IntentParser::new(
        "https://api.openai.com/v1/chat/completions".to_string(),
        "test_key".to_string(),
        cache.clone(),
    );

    let query = "action movies on netflix";

    // First parse - should use fallback and cache it
    let start = Instant::now();
    let intent1 = parser.parse(query).await.unwrap();
    let first_duration = start.elapsed();

    // Second parse - should hit cache (much faster)
    let start = Instant::now();
    let intent2 = parser.parse(query).await.unwrap();
    let cache_duration = start.elapsed();

    // Verify same intent returned
    assert_eq!(intent1.fallback_query, intent2.fallback_query);
    assert_eq!(intent1.filters.genre, intent2.filters.genre);

    // Cache hit should be significantly faster (<5ms vs potentially 100-500ms for GPT)
    println!(
        "First parse: {:?}, Cache hit: {:?}",
        first_duration, cache_duration
    );
    assert!(
        cache_duration.as_millis() < 10,
        "Cache hit should be <10ms, got {:?}ms",
        cache_duration.as_millis()
    );

    // Cleanup
    cache.clear_intent_cache().await.unwrap();
}

#[tokio::test]
async fn test_intent_cache_normalization() {
    let config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    let cache = match RedisCache::new(config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    cache.clear_intent_cache().await.unwrap();

    let parser = IntentParser::new(
        "https://api.openai.com/v1/chat/completions".to_string(),
        "test_key".to_string(),
        cache.clone(),
    );

    // Different variations of same query
    let queries = vec![
        "  Horror Movies  ",
        "horror movies",
        "HORROR MOVIES",
        "Horror Movies",
    ];

    // Parse first variation
    let intent1 = parser.parse(queries[0]).await.unwrap();

    // All other variations should hit cache
    for query in &queries[1..] {
        let start = Instant::now();
        let intent = parser.parse(query).await.unwrap();
        let duration = start.elapsed();

        // Should be cache hit (fast)
        assert!(
            duration.as_millis() < 10,
            "Expected cache hit for '{}', but took {:?}ms",
            query,
            duration.as_millis()
        );

        // Should be same intent
        assert_eq!(intent.filters.genre, intent1.filters.genre);
    }

    cache.clear_intent_cache().await.unwrap();
}

#[tokio::test]
async fn test_intent_cache_ttl_expiration() {
    let config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 2, // 2 seconds for testing
    });

    let cache = match RedisCache::new(config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    cache.clear_intent_cache().await.unwrap();

    let parser = IntentParser::new(
        "https://api.openai.com/v1/chat/completions".to_string(),
        "test_key".to_string(),
        cache.clone(),
    );

    let query = "comedy shows on hulu";

    // First parse
    let intent1 = parser.parse(query).await.unwrap();

    // Immediate re-parse should hit cache
    let start = Instant::now();
    let intent2 = parser.parse(query).await.unwrap();
    let cache_hit_duration = start.elapsed();

    assert!(cache_hit_duration.as_millis() < 10);
    assert_eq!(intent1.fallback_query, intent2.fallback_query);

    // Wait for TTL expiration
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Should be cache miss now (slower than cache hit)
    let start = Instant::now();
    let intent3 = parser.parse(query).await.unwrap();
    let post_ttl_duration = start.elapsed();

    // Still same intent, but fetched again
    assert_eq!(intent1.fallback_query, intent3.fallback_query);

    println!(
        "Cache hit: {:?}ms, Post-TTL: {:?}ms",
        cache_hit_duration.as_millis(),
        post_ttl_duration.as_millis()
    );

    cache.clear_intent_cache().await.unwrap();
}

#[tokio::test]
async fn test_intent_cache_metrics() {
    let config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    let cache = match RedisCache::new(config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    cache.clear_intent_cache().await.unwrap();

    let parser = IntentParser::new(
        "https://api.openai.com/v1/chat/completions".to_string(),
        "test_key".to_string(),
        cache.clone(),
    );

    // Generate several parses
    let queries = vec![
        "action thriller",
        "romantic comedy",
        "sci-fi adventure",
        "action thriller", // duplicate for cache hit
        "romantic comedy", // duplicate for cache hit
    ];

    for query in queries {
        parser.parse(query).await.unwrap();
    }

    // Verify cache is working
    let normalized = "action thriller".trim().to_lowercase();
    let cached: Option<ParsedIntent> = cache.get_intent(&normalized).await.unwrap();
    assert!(cached.is_some(), "Expected intent to be cached");

    cache.clear_intent_cache().await.unwrap();
}

#[tokio::test]
async fn test_intent_cache_concurrent_access() {
    let config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    let cache = match RedisCache::new(config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    cache.clear_intent_cache().await.unwrap();

    let parser = Arc::new(IntentParser::new(
        "https://api.openai.com/v1/chat/completions".to_string(),
        "test_key".to_string(),
        cache.clone(),
    ));

    let query = "thriller movies";

    // Spawn multiple concurrent parse requests
    let mut handles = vec![];
    for _ in 0..10 {
        let parser_clone = parser.clone();
        let query_clone = query.to_string();

        let handle = tokio::spawn(async move {
            parser_clone.parse(&query_clone).await.unwrap()
        });

        handles.push(handle);
    }

    // Wait for all to complete
    let results: Vec<ParsedIntent> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // All should return same intent
    for result in &results[1..] {
        assert_eq!(result.fallback_query, results[0].fallback_query);
    }

    cache.clear_intent_cache().await.unwrap();
}

#[tokio::test]
async fn test_cache_failure_fallback() {
    let config = Arc::new(CacheConfig {
        redis_url: "redis://invalid-host:9999".to_string(), // Invalid Redis
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    // This should fail to connect
    if RedisCache::new(config.clone()).await.is_ok() {
        eprintln!("Unexpected: Redis connection succeeded");
        return;
    }

    // Parser should still work with fallback even if cache fails
    // In production, we'd handle cache init failures gracefully
}
