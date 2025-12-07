# Redis Caching Layer Documentation

## Overview

The Redis caching layer provides high-performance caching for the Media Gateway Discovery service with intelligent TTL management, SHA256-based key generation, and comprehensive monitoring.

## Features

### Core Capabilities

- **Connection Pooling**: Async Redis connection manager for optimal performance
- **TTL-Based Expiration**: Different cache lifetimes for different data types
- **SHA256 Key Generation**: Deterministic cache keys from query parameters
- **JSON Serialization**: Support for complex nested data structures
- **Metrics & Tracing**: Built-in observability with tracing integration
- **Error Handling**: Graceful degradation when Redis is unavailable

### Cache Types

| Type | TTL | Use Case |
|------|-----|----------|
| Search Results | 30 minutes | Frequently updated, query-dependent |
| Parsed Intents | 10 minutes | Moderate volatility, NLP results |
| Embeddings | 1 hour | Stable vector representations |

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  RedisCache                              │
├─────────────────────────────────────────────────────────┤
│  Connection Manager (Async Pool)                        │
│  ├─ ConnectionManager                                   │
│  └─ Redis Client                                        │
├─────────────────────────────────────────────────────────┤
│  Cache Operations                                       │
│  ├─ get<T>(key) -> Option<T>                           │
│  ├─ set<T>(key, value, ttl)                            │
│  ├─ delete(key) -> u64                                  │
│  └─ delete_pattern(pattern) -> u64                      │
├─────────────────────────────────────────────────────────┤
│  Specialized Methods                                    │
│  ├─ cache_search_results(query, results)               │
│  ├─ get_search_results(query) -> Option<Results>       │
│  ├─ cache_intent(text, intent)                         │
│  ├─ get_intent(text) -> Option<Intent>                 │
│  ├─ cache_embedding(text, vec)                         │
│  └─ get_embedding(text) -> Option<Vec<f32>>            │
├─────────────────────────────────────────────────────────┤
│  Management & Monitoring                               │
│  ├─ health_check() -> bool                             │
│  ├─ stats() -> CacheStats                              │
│  ├─ clear_search_cache() -> u64                        │
│  ├─ clear_intent_cache() -> u64                        │
│  ├─ clear_embedding_cache() -> u64                     │
│  └─ clear_all() -> u64                                 │
└─────────────────────────────────────────────────────────┘
```

## Usage

### Initialization

```rust
use media_gateway_discovery::cache::RedisCache;
use media_gateway_discovery::config::CacheConfig;
use std::sync::Arc;

let config = Arc::new(CacheConfig {
    redis_url: "redis://localhost:6379".to_string(),
    search_ttl_sec: 1800,     // 30 minutes
    embedding_ttl_sec: 3600,   // 1 hour
    intent_ttl_sec: 600,       // 10 minutes
});

let cache = RedisCache::new(config).await?;
```

### Caching Search Results

```rust
#[derive(Serialize, Deserialize)]
struct SearchQuery {
    text: String,
    limit: usize,
}

#[derive(Serialize, Deserialize)]
struct SearchResults {
    items: Vec<String>,
    total: usize,
}

// Cache miss - perform search
let query = SearchQuery {
    text: "science fiction".to_string(),
    limit: 20
};

let results: Option<SearchResults> = cache
    .get_search_results(&query)
    .await?;

if results.is_none() {
    // Perform actual search
    let results = perform_search(&query).await?;

    // Cache for 30 minutes
    cache.cache_search_results(&query, &results).await?;
}
```

### Caching Parsed Intents

```rust
#[derive(Serialize, Deserialize)]
struct ParsedIntent {
    category: String,
    confidence: f64,
    entities: Vec<String>,
}

let text = "Find me action movies from the 90s";

// Check cache first
let intent: Option<ParsedIntent> = cache.get_intent(&text).await?;

if intent.is_none() {
    // Parse intent
    let intent = parse_intent(text).await?;

    // Cache for 10 minutes
    cache.cache_intent(&text, &intent).await?;
}
```

### Caching Embeddings

```rust
let text = "The quick brown fox jumps over the lazy dog";

// Check cache
let embedding: Option<Vec<f32>> = cache.get_embedding(&text).await?;

if embedding.is_none() {
    // Generate embedding
    let embedding = generate_embedding(text).await?;

    // Cache for 1 hour
    cache.cache_embedding(&text, &embedding).await?;
}
```

## Cache Key Generation

Cache keys are generated using SHA256 hashing for consistency:

```rust
use media_gateway_discovery::cache::RedisCache;

let query = SearchQuery {
    text: "test".to_string(),
    limit: 10
};

let key = RedisCache::generate_key("search", &query)?;
// Result: "search:a3c5f8d2e1b4f9c6a7e8d3f2b5c9e1a4..."
```

### Key Format

- **Prefix**: Identifies cache type (`search`, `intent`, `embedding`)
- **Hash**: SHA256 of JSON-serialized data (64 hex characters)
- **Format**: `{prefix}:{sha256_hash}`

### Benefits

1. **Deterministic**: Same input always produces same key
2. **Collision-Resistant**: SHA256 ensures uniqueness
3. **Type-Safe**: Prefix prevents cross-type collisions
4. **Pattern-Matchable**: Enables bulk operations

## Monitoring & Observability

### Health Checks

```rust
let healthy = cache.health_check().await?;
if !healthy {
    log::error!("Redis cache is unhealthy!");
}
```

### Cache Statistics

```rust
let stats = cache.stats().await?;
println!("Hits: {}", stats.hits);
println!("Misses: {}", stats.misses);
println!("Hit Rate: {:.2}%", stats.hit_rate * 100.0);
```

### Tracing Integration

All cache operations emit structured logs:

```
DEBUG cache: Generated cache key key="search:a3c5f8d2..." prefix="search"
DEBUG cache: Cache hit key="search:a3c5f8d2..."
DEBUG cache: Cache miss key="intent:b4d9e7f3..."
DEBUG cache: Cached search results key="search:..." ttl=1800
```

## Cache Management

### Clear Specific Cache Types

```rust
// Clear all search caches
let deleted = cache.clear_search_cache().await?;

// Clear all intent caches
let deleted = cache.clear_intent_cache().await?;

// Clear all embedding caches
let deleted = cache.clear_embedding_cache().await?;
```

### Clear All Caches

```rust
let total_deleted = cache.clear_all().await?;
println!("Cleared {} cache entries", total_deleted);
```

### Pattern-Based Deletion

```rust
// Delete specific patterns
let deleted = cache.delete_pattern("search:test:*").await?;
```

## Error Handling

The cache implements graceful error handling:

```rust
use media_gateway_discovery::cache::CacheError;

match cache.get_search_results(&query).await {
    Ok(Some(results)) => {
        // Cache hit
        Ok(results)
    }
    Ok(None) => {
        // Cache miss - perform search
        perform_search(&query).await
    }
    Err(CacheError::Connection(e)) => {
        // Redis unavailable - fallback to direct search
        log::warn!("Cache unavailable: {}", e);
        perform_search(&query).await
    }
    Err(e) => Err(e.into())
}
```

### Error Types

- `CacheError::Connection`: Redis connection failure
- `CacheError::Serialization`: JSON serialization/deserialization error
- `CacheError::KeyGeneration`: Cache key generation failure
- `CacheError::Operation`: General operation failure

## Performance Considerations

### Connection Pooling

The cache uses Redis ConnectionManager for optimal performance:

- **Automatic reconnection** on connection loss
- **Connection reuse** across operations
- **Async/await** support for non-blocking operations

### TTL Strategy

Different TTLs optimize for different use cases:

- **Search (30min)**: Balances freshness with cache efficiency
- **Intent (10min)**: Accounts for evolving NLP models
- **Embedding (1hr)**: Leverages stability of vector representations

### Memory Management

Redis automatically evicts expired keys:

```redis
# Verify TTL
TTL search:a3c5f8d2...
# Returns: 1643 (seconds remaining)
```

## Testing

### Unit Tests

```bash
# Run unit tests (no Redis required)
cargo test --lib cache::tests
```

### Integration Tests

```bash
# Start Redis
docker run -d -p 6379:6379 redis:7-alpine

# Run integration tests
REDIS_URL=redis://localhost:6379 cargo test --test cache_integration_test
```

### Example Usage

```bash
# Run example (requires Redis)
cargo run --example cache_usage
```

## Configuration

### Environment Variables

```bash
# Redis URL
DISCOVERY_CACHE__REDIS_URL=redis://localhost:6379

# TTL settings (seconds)
DISCOVERY_CACHE__SEARCH_TTL_SEC=1800
DISCOVERY_CACHE__INTENT_TTL_SEC=600
DISCOVERY_CACHE__EMBEDDING_TTL_SEC=3600
```

### Config File (YAML)

```yaml
cache:
  redis_url: redis://localhost:6379
  search_ttl_sec: 1800    # 30 minutes
  intent_ttl_sec: 600     # 10 minutes
  embedding_ttl_sec: 3600 # 1 hour
```

## Production Deployment

### Redis Setup

```bash
# Production Redis with persistence
docker run -d \
  --name redis-cache \
  -p 6379:6379 \
  -v redis-data:/data \
  redis:7-alpine \
  redis-server --appendonly yes --maxmemory 2gb --maxmemory-policy allkeys-lru
```

### High Availability

For production, use Redis Cluster or Sentinel:

```rust
let config = Arc::new(CacheConfig {
    redis_url: "redis://redis-cluster:6379,redis-cluster:6380".to_string(),
    // ... other config
});
```

### Monitoring

Monitor these metrics in production:

- **Hit rate**: Should be > 70% for search results
- **Memory usage**: Track Redis memory consumption
- **Latency**: p50, p95, p99 cache operation times
- **Error rate**: Connection failures and timeouts

## Best Practices

1. **Always check cache first** before expensive operations
2. **Handle cache misses gracefully** - don't fail if cache is down
3. **Monitor hit rates** to optimize TTL values
4. **Use type-safe serialization** with Serde
5. **Implement circuit breakers** for cache failures
6. **Clear stale caches** during deployments
7. **Log cache operations** for debugging

## Troubleshooting

### Cache Not Working

```rust
// Verify Redis connection
let healthy = cache.health_check().await?;
println!("Cache healthy: {}", healthy);
```

### Low Hit Rate

- Check TTL values are appropriate
- Verify query parameters are consistent
- Monitor cache eviction rate

### Memory Issues

```bash
# Check Redis memory
redis-cli INFO memory

# Clear all caches if needed
redis-cli FLUSHDB
```

## Future Enhancements

- [ ] Cache warming on startup
- [ ] Automatic TTL adjustment based on hit rates
- [ ] Multi-level caching (L1: local, L2: Redis)
- [ ] Cache compression for large values
- [ ] Circuit breaker pattern for resilience
- [ ] Distributed tracing integration

## References

- [Redis Documentation](https://redis.io/docs/)
- [redis-rs Crate](https://docs.rs/redis/)
- [Caching Best Practices](https://docs.aws.amazon.com/AmazonElastiCache/latest/red-ug/BestPractices.html)
