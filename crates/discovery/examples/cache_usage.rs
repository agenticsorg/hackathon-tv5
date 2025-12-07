//! Redis Cache Usage Examples
//!
//! This example demonstrates how to use the RedisCache implementation
//! in the Media Gateway Discovery service.
//!
//! Run with: cargo run --example cache_usage

use media_gateway_discovery::cache::RedisCache;
use media_gateway_discovery::config::CacheConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SearchQuery {
    text: String,
    filters: Vec<String>,
    limit: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SearchResults {
    items: Vec<String>,
    total: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ParsedIntent {
    category: String,
    confidence: f64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for observability
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("=== Redis Cache Usage Examples ===\n");

    // 1. Initialize the cache
    println!("1. Initializing Redis cache...");
    let config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,    // 30 minutes
        embedding_ttl_sec: 3600, // 1 hour
        intent_ttl_sec: 600,     // 10 minutes
    });

    let cache = match RedisCache::new(config).await {
        Ok(c) => {
            println!("✓ Cache initialized successfully\n");
            c
        }
        Err(e) => {
            eprintln!("✗ Failed to initialize cache: {}", e);
            eprintln!("Make sure Redis is running on localhost:6379");
            return Err(e.into());
        }
    };

    // 2. Health check
    println!("2. Checking cache health...");
    let healthy = cache.health_check().await?;
    println!("✓ Cache is healthy: {}\n", healthy);

    // 3. Cache search results
    println!("3. Caching search results...");
    let search_query = SearchQuery {
        text: "science fiction movies".to_string(),
        filters: vec!["movies".to_string(), "sci-fi".to_string()],
        limit: 20,
    };

    let search_results = SearchResults {
        items: vec![
            "Blade Runner 2049".to_string(),
            "The Matrix".to_string(),
            "Interstellar".to_string(),
        ],
        total: 3,
    };

    // First request - cache miss
    println!("   First request (cache miss)...");
    let cached: Option<SearchResults> = cache.get_search_results(&search_query).await?;
    println!("   Result: {:?}", cached);

    // Cache the results
    cache
        .cache_search_results(&search_query, &search_results)
        .await?;
    println!("✓ Results cached with 30-minute TTL\n");

    // Second request - cache hit
    println!("   Second request (cache hit)...");
    let cached: Option<SearchResults> = cache.get_search_results(&search_query).await?;
    println!("   Result: {:?}", cached);
    println!("✓ Retrieved from cache\n");

    // 4. Cache parsed intents
    println!("4. Caching parsed intent...");
    let intent_text = "Find me action movies from the 90s";
    let parsed_intent = ParsedIntent {
        category: "movie_search".to_string(),
        confidence: 0.89,
    };

    cache.cache_intent(&intent_text, &parsed_intent).await?;
    println!("✓ Intent cached with 10-minute TTL\n");

    let cached_intent: Option<ParsedIntent> = cache.get_intent(&intent_text).await?;
    println!("   Retrieved intent: {:?}\n", cached_intent);

    // 5. Cache embeddings
    println!("5. Caching text embeddings...");
    let text = "The quick brown fox jumps over the lazy dog";
    let embedding = vec![0.12, 0.34, 0.56, 0.78, 0.90, 0.11, 0.22, 0.33, 0.44, 0.55];

    cache.cache_embedding(&text, &embedding).await?;
    println!(
        "✓ Embedding cached with 1-hour TTL (dimension: {})\n",
        embedding.len()
    );

    let cached_embedding = cache.get_embedding(&text).await?;
    println!("   Retrieved embedding: {:?}\n", cached_embedding);

    // 6. Cache statistics
    println!("6. Cache statistics...");
    let stats = cache.stats().await?;
    println!("   Hits: {}", stats.hits);
    println!("   Misses: {}", stats.misses);
    println!("   Hit Rate: {:.2}%\n", stats.hit_rate * 100.0);

    // 7. Manual cache operations
    println!("7. Manual cache operations...");
    let custom_key = "custom:example:key";
    let custom_value = "Custom cached value";

    cache.set(custom_key, &custom_value, 300).await?;
    println!("✓ Set custom key with 5-minute TTL");

    let retrieved: Option<String> = cache.get(custom_key).await?;
    println!("   Retrieved: {:?}", retrieved);

    cache.delete(custom_key).await?;
    println!("✓ Deleted custom key\n");

    // 8. Pattern-based deletion
    println!("8. Pattern-based cache clearing...");

    // Add some test keys
    for i in 0..5 {
        let key = format!("pattern:test:{}", i);
        cache.set(&key, &i, 60).await?;
    }
    println!("   Created 5 test keys");

    let deleted = cache.delete_pattern("pattern:test:*").await?;
    println!("✓ Deleted {} keys matching pattern\n", deleted);

    // 9. Clear specific cache types
    println!("9. Clearing cache types...");
    let cleared_search = cache.clear_search_cache().await?;
    println!("   Cleared {} search cache entries", cleared_search);

    let cleared_intent = cache.clear_intent_cache().await?;
    println!("   Cleared {} intent cache entries", cleared_intent);

    let cleared_embedding = cache.clear_embedding_cache().await?;
    println!("   Cleared {} embedding cache entries\n", cleared_embedding);

    // 10. Cache key generation
    println!("10. Cache key generation (SHA256)...");
    let key = RedisCache::generate_key("search", &search_query)?;
    println!("    Generated key: {}", key);
    println!(
        "    Key length: {} chars (prefix + 64 hex chars)\n",
        key.len()
    );

    println!("=== All examples completed successfully! ===");

    Ok(())
}
