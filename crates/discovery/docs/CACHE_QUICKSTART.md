# Redis Cache Quick Start Guide

## 5-Minute Integration Guide

### 1. Add to Your Service (30 seconds)

```rust
use media_gateway_discovery::cache::RedisCache;
use std::sync::Arc;

// Initialize once at startup
let cache = RedisCache::new(Arc::new(config.cache)).await?;
```

### 2. Cache Search Results (1 minute)

```rust
// Define your types
#[derive(Serialize, Deserialize)]
struct Query { text: String, limit: usize }

#[derive(Serialize, Deserialize)]
struct Results { items: Vec<String> }

// Check cache first
if let Some(cached) = cache.get_search_results(&query).await? {
    return Ok(cached); // Cache hit - return immediately
}

// Cache miss - perform search
let results = perform_expensive_search(&query).await?;

// Cache for 30 minutes
cache.cache_search_results(&query, &results).await?;

Ok(results)
```

### 3. Cache Intents (1 minute)

```rust
#[derive(Serialize, Deserialize)]
struct Intent { category: String, confidence: f64 }

// Check cache
if let Some(intent) = cache.get_intent(&text).await? {
    return Ok(intent);
}

// Parse intent
let intent = parse_intent(&text).await?;

// Cache for 10 minutes
cache.cache_intent(&text, &intent).await?;
```

### 4. Cache Embeddings (1 minute)

```rust
// Check cache
if let Some(embedding) = cache.get_embedding(&text).await? {
    return Ok(embedding);
}

// Generate embedding
let embedding = generate_embedding(&text).await?;

// Cache for 1 hour
cache.cache_embedding(&text, &embedding).await?;
```

### 5. Add Health Check (30 seconds)

```rust
// In your health endpoint
async fn health_check(cache: Arc<RedisCache>) -> Result<HealthStatus> {
    let cache_healthy = cache.health_check().await.unwrap_or(false);

    Ok(HealthStatus {
        cache: cache_healthy,
        // ... other checks
    })
}
```

### 6. Monitor Performance (1 minute)

```rust
// Periodic monitoring
async fn log_cache_stats(cache: Arc<RedisCache>) {
    if let Ok(stats) = cache.stats().await {
        info!(
            "Cache stats - Hits: {}, Misses: {}, Hit Rate: {:.2}%",
            stats.hits,
            stats.misses,
            stats.hit_rate * 100.0
        );
    }
}
```

## Common Patterns

### Pattern 1: Cache-Aside (Lazy Loading)

```rust
pub async fn search(
    cache: &RedisCache,
    query: &Query
) -> Result<Results> {
    // Try cache first
    if let Some(cached) = cache.get_search_results(query).await? {
        return Ok(cached);
    }

    // Cache miss - load from source
    let results = load_from_database(query).await?;

    // Update cache asynchronously (fire and forget)
    let cache_clone = cache.clone();
    let query_clone = query.clone();
    let results_clone = results.clone();
    tokio::spawn(async move {
        let _ = cache_clone
            .cache_search_results(&query_clone, &results_clone)
            .await;
    });

    Ok(results)
}
```

### Pattern 2: Write-Through Caching

```rust
pub async fn update_search_index(
    cache: &RedisCache,
    doc: &Document
) -> Result<()> {
    // Update source
    database.insert(doc).await?;

    // Invalidate related caches
    cache.clear_search_cache().await?;

    Ok(())
}
```

### Pattern 3: Graceful Degradation

```rust
pub async fn get_with_fallback(
    cache: &RedisCache,
    query: &Query
) -> Result<Results> {
    match cache.get_search_results(query).await {
        Ok(Some(results)) => Ok(results),
        Ok(None) | Err(_) => {
            // Cache miss or error - use fallback
            warn!("Cache unavailable, using direct search");
            perform_search(query).await
        }
    }
}
```

## Configuration Cheat Sheet

### Development
```yaml
cache:
  redis_url: redis://localhost:6379
  search_ttl_sec: 300      # 5 min (faster iteration)
  intent_ttl_sec: 60       # 1 min
  embedding_ttl_sec: 600   # 10 min
```

### Production
```yaml
cache:
  redis_url: redis://redis-cluster:6379
  search_ttl_sec: 1800     # 30 min
  intent_ttl_sec: 600      # 10 min
  embedding_ttl_sec: 3600  # 1 hour
```

## Testing Checklist

- [ ] Start Redis: `docker run -d -p 6379:6379 redis:7-alpine`
- [ ] Run unit tests: `cargo test cache::tests`
- [ ] Run integration tests: `cargo test --test cache_integration_test`
- [ ] Run example: `cargo run --example cache_usage`
- [ ] Check health: `curl http://localhost:8081/health`
- [ ] Monitor stats: Check logs for cache hit rates

## Troubleshooting

**Cache not working?**
```rust
cache.health_check().await? // Returns false if Redis unavailable
```

**Low hit rate?**
```rust
let stats = cache.stats().await?;
println!("Hit rate: {:.2}%", stats.hit_rate * 100.0);
// Should be > 70% for search results
```

**Memory issues?**
```bash
redis-cli INFO memory
redis-cli FLUSHDB  # Clear all keys (use with caution!)
```

## Performance Tips

1. **Clone is cheap**: `RedisCache` uses `Arc` internally
2. **Batch operations**: Use pattern deletion for bulk invalidation
3. **Async all the way**: Never block on cache operations
4. **Set appropriate TTLs**: Balance freshness vs efficiency
5. **Monitor hit rates**: Adjust TTLs based on actual usage

## Next Steps

- Read full documentation: `docs/cache.md`
- Review example code: `examples/cache_usage.rs`
- Run integration tests: `tests/cache_integration_test.rs`
- Monitor production metrics: Hit rate, latency, errors

---

**Need help?** Check the main documentation at `docs/cache.md`
