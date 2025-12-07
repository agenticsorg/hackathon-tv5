# Response Caching Middleware Implementation

## Overview

The Response Caching Middleware provides Redis-backed HTTP response caching for the Media Gateway API. It implements intelligent caching strategies with ETag support, configurable TTLs, and cache invalidation patterns.

**Implementation**: `/workspaces/media-gateway/crates/api/src/middleware/cache.rs`

## Features

### Core Capabilities

1. **Request Interception for GET Requests**
   - Only caches successful GET requests (2xx status codes)
   - POST, PUT, DELETE, and other methods bypass caching
   - Configurable path-based cache skipping

2. **Redis-Backed Storage**
   - Asynchronous Redis operations using `ConnectionManager`
   - Efficient key-value storage with TTL support
   - Automatic cleanup of expired entries

3. **ETag Generation & Validation**
   - SHA-256 hash-based ETag generation
   - If-None-Match header support
   - 304 Not Modified responses for matching ETags

4. **Configurable TTL**
   - Default TTL: 60 seconds
   - Content endpoint TTL: 300 seconds (5 minutes)
   - Path-based TTL selection

5. **Cache Key Generation**
   - Based on: HTTP method + path + query parameters + user_id (optional)
   - Query parameter filtering (skips dynamic params like `timestamp`)
   - User-specific caching support

6. **Cache Invalidation**
   - Single key invalidation: `invalidate(key)`
   - Pattern-based invalidation: `invalidate_pattern(pattern)`
   - Supports wildcard patterns

7. **Observability**
   - Structured logging via `tracing`
   - Cache hit/miss tracking with X-Cache header
   - Performance metrics support

## Architecture

```
Request Flow:
┌─────────────┐
│   Client    │
└──────┬──────┘
       │
       │ GET /api/content/123
       ▼
┌─────────────────────────┐
│  CacheMiddleware        │
│  ┌──────────────────┐   │
│  │ 1. Check Method  │   │
│  │    (GET only)    │   │
│  └────────┬─────────┘   │
│           │             │
│  ┌────────▼─────────┐   │
│  │ 2. Generate Key  │   │
│  │   (method+path+  │   │
│  │    query+user)   │   │
│  └────────┬─────────┘   │
│           │             │
│  ┌────────▼─────────┐   │
│  │ 3. Check Redis   │   │
│  │                  │   │
│  └────┬────────┬────┘   │
│       │        │        │
│   Hit │        │ Miss   │
│       │        │        │
│  ┌────▼────┐  │        │
│  │ Return  │  │        │
│  │ Cached  │  │        │
│  └─────────┘  │        │
│               │        │
│      ┌────────▼─────┐  │
│      │ Call Service │  │
│      └────────┬─────┘  │
│               │        │
│      ┌────────▼─────┐  │
│      │ Cache Result │  │
│      │ (async)      │  │
│      └──────────────┘  │
└─────────────────────────┘
```

## Configuration

### CacheConfig Structure

```rust
pub struct CacheConfig {
    /// Default TTL for cached responses (in seconds)
    pub default_ttl: u64,

    /// TTL for content endpoints (in seconds)
    pub content_ttl: u64,

    /// Whether to cache authenticated requests
    pub cache_authenticated: bool,

    /// Skip caching for these path patterns
    pub skip_paths: Vec<String>,

    /// Skip caching for these query parameters
    pub skip_query_params: Vec<String>,
}
```

### Default Configuration

```rust
CacheConfig {
    default_ttl: 60,           // 1 minute
    content_ttl: 300,          // 5 minutes
    cache_authenticated: false, // Don't cache user-specific data
    skip_paths: vec![
        "/api/user/",
        "/api/sync/",
        "/api/admin/",
    ],
    skip_query_params: vec![
        "nocache",
        "timestamp",
    ],
}
```

## Usage

### Basic Setup

```rust
use media_gateway_api::middleware::{CacheConfig, CacheMiddleware};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let redis_url = "redis://localhost:6379";

    // Use default configuration
    let cache = CacheMiddleware::default_config(redis_url)
        .await
        .expect("Failed to initialize cache");

    HttpServer::new(move || {
        App::new()
            .wrap(cache.clone())
            .route("/api/content/{id}", web::get().to(handler))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

### Custom Configuration

```rust
let config = CacheConfig {
    default_ttl: 120,          // 2 minutes
    content_ttl: 600,          // 10 minutes
    cache_authenticated: true, // Cache user-specific responses
    skip_paths: vec![
        "/api/user/".to_string(),
        "/health".to_string(),
    ],
    skip_query_params: vec![
        "nocache".to_string(),
        "debug".to_string(),
    ],
};

let cache = CacheMiddleware::new(redis_url, config)
    .await
    .expect("Failed to initialize cache");
```

### Cache Invalidation

```rust
// Invalidate a specific cache entry
let cache_middleware = web::Data::<CacheMiddleware>::from_request(&req)
    .await
    .unwrap();

cache_middleware.invalidate("GET:/api/content/123").await?;

// Invalidate by pattern (all content endpoints)
let count = cache_middleware
    .invalidate_pattern("GET:/api/content/*")
    .await?;

println!("Invalidated {} cache entries", count);
```

## Cache Key Format

Cache keys are generated using the following format:

```
cache:{method}:{path}{?query}{:user:{user_id}}
```

Examples:
- Anonymous: `cache:GET:/api/content/123`
- With query: `cache:GET:/api/search?q=test&limit=10`
- Authenticated: `cache:GET:/api/content/123:user:user456`

## Response Headers

### Cache Miss (First Request)

```http
HTTP/1.1 200 OK
Content-Type: application/json
ETag: "a1b2c3d4e5f6g7h8"
Cache-Control: max-age=300
X-Cache: MISS
```

### Cache Hit (Subsequent Request)

```http
HTTP/1.1 200 OK
Content-Type: application/json
ETag: "a1b2c3d4e5f6g7h8"
Cache-Control: max-age=300
X-Cache: HIT
```

### 304 Not Modified (ETag Match)

```http
HTTP/1.1 304 Not Modified
ETag: "a1b2c3d4e5f6g7h8"
Cache-Control: max-age=300
```

## Performance Characteristics

### Cache Hit Performance

- **Without Cache**: ~100-500ms (depending on backend processing)
- **With Cache**: ~5-15ms (Redis lookup + serialization)
- **304 Response**: ~2-5ms (ETag comparison only)

### Storage Efficiency

- Average cache entry size: 1-10 KB
- Redis memory usage: ~1 MB per 100-1000 entries
- TTL-based automatic cleanup

### Scalability

- Supports horizontal scaling via Redis Cluster
- Connection pooling for optimal Redis performance
- Async operations prevent blocking

## Best Practices

### 1. TTL Configuration

```rust
// Short TTL for frequently changing data
config.default_ttl = 30; // 30 seconds

// Long TTL for static content
config.content_ttl = 3600; // 1 hour
```

### 2. Skip Paths Configuration

```rust
// Don't cache user-specific or admin endpoints
config.skip_paths = vec![
    "/api/user/".to_string(),
    "/api/admin/".to_string(),
    "/api/sync/".to_string(),
    "/health".to_string(),      // Health checks
    "/metrics".to_string(),     // Metrics endpoints
];
```

### 3. Query Parameter Filtering

```rust
// Skip dynamic parameters that change frequently
config.skip_query_params = vec![
    "nocache".to_string(),
    "timestamp".to_string(),
    "random".to_string(),
    "debug".to_string(),
];
```

### 4. Cache Invalidation Strategy

```rust
// Invalidate on content updates
async fn update_content(id: String, cache: web::Data<CacheMiddleware>) {
    // ... update content ...

    // Invalidate specific content
    cache.invalidate(&format!("GET:/api/content/{}", id)).await?;

    // Or invalidate all related content
    cache.invalidate_pattern("GET:/api/content/*").await?;
}
```

### 5. Monitoring Cache Performance

```rust
// Add custom metrics
async fn cache_metrics(cache: web::Data<CacheMiddleware>) -> HttpResponse {
    // Track cache hit/miss ratio
    // Monitor Redis connection health
    // Report cache size and memory usage

    HttpResponse::Ok().json(metrics)
}
```

## Testing

### Running Tests

```bash
# Start Redis
docker run -d -p 6379:6379 redis:alpine

# Run all tests
cargo test --test cache_middleware_test

# Run specific test
cargo test --test cache_middleware_test test_cache_miss_then_hit
```

### Test Coverage

- ✅ Cache miss and hit behavior
- ✅ ETag generation and validation
- ✅ 304 Not Modified responses
- ✅ Query parameter handling
- ✅ Authenticated request caching
- ✅ Path-based cache skipping
- ✅ TTL expiration
- ✅ Cache invalidation
- ✅ Pattern-based invalidation
- ✅ Performance improvements

## Integration with Existing Middleware

### Middleware Order

```rust
App::new()
    // 1. Request ID (first - for tracing)
    .wrap(RequestIdMiddleware)

    // 2. Logging (before cache for accurate metrics)
    .wrap(LoggingMiddleware)

    // 3. Auth (before cache to set user context)
    .wrap(AuthMiddleware::optional())

    // 4. Cache (after auth to access user context)
    .wrap(cache_middleware)

    // 5. Routes
    .route("/api/content/{id}", web::get().to(handler))
```

## Redis Configuration

### Recommended Redis Settings

```conf
# Memory management
maxmemory 2gb
maxmemory-policy allkeys-lru

# Persistence (optional for cache)
save ""
appendonly no

# Performance
tcp-backlog 511
timeout 0
tcp-keepalive 300
```

### Redis Connection Options

```rust
// Standard connection
let redis_url = "redis://localhost:6379";

// With authentication
let redis_url = "redis://:password@localhost:6379";

// Redis Cluster
let redis_url = "redis://node1:6379,node2:6379,node3:6379";

// With database selection
let redis_url = "redis://localhost:6379/2";
```

## Troubleshooting

### Common Issues

1. **Cache Not Working**
   - Verify Redis is running: `redis-cli ping`
   - Check Redis connection string
   - Ensure middleware is properly wrapped

2. **High Memory Usage**
   - Reduce TTL values
   - Add more skip_paths
   - Configure Redis maxmemory policy

3. **Cache Misses**
   - Check query parameter ordering
   - Verify cache key generation
   - Check TTL expiration

4. **Performance Issues**
   - Use Redis connection pooling
   - Monitor Redis server performance
   - Consider Redis Cluster for scaling

## Security Considerations

1. **User Data Isolation**
   - Set `cache_authenticated: false` for user-specific endpoints
   - Or include user_id in cache key when `cache_authenticated: true`

2. **Sensitive Data**
   - Never cache responses containing secrets or tokens
   - Use skip_paths for authentication endpoints

3. **Cache Poisoning**
   - Validate all cached data on retrieval
   - Use ETag validation
   - Implement cache invalidation on security events

## Future Enhancements

- [ ] Cache warming strategies
- [ ] Distributed cache invalidation (pub/sub)
- [ ] Cache analytics and reporting
- [ ] Adaptive TTL based on request patterns
- [ ] Compression for large responses
- [ ] Multi-tier caching (memory + Redis)
- [ ] Cache tags for group invalidation

## References

- **Implementation**: `/workspaces/media-gateway/crates/api/src/middleware/cache.rs`
- **Tests**: `/workspaces/media-gateway/crates/api/tests/cache_middleware_test.rs`
- **Example**: `/workspaces/media-gateway/crates/api/examples/cache_middleware_usage.rs`
- **Redis Documentation**: https://redis.io/documentation
- **Actix-web Middleware**: https://actix.rs/docs/middleware/
