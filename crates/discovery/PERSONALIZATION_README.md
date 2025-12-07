# Discovery Search Personalization

## Overview

The Discovery service personalization feature enhances search results by integrating user preference scores from the SONA Personalization Engine. This allows search results to be reranked based on individual user preferences, improving content relevance and discovery.

## Features

- **User-Aware Search**: Personalizes results based on user viewing history and preferences
- **SONA Integration**: Fetches real-time personalization scores from SONA service
- **Redis Caching**: Caches user preferences with 5-minute TTL for performance
- **A/B Testing**: Supports multiple personalization strategies via experiment variants
- **Low Latency**: Adds <50ms overhead to search requests
- **Graceful Degradation**: Falls back to unpersonalized results on failures
- **Observable**: Comprehensive logging and tracing

## Architecture

### Search Pipeline Integration

Personalization is Phase 4 of the hybrid search pipeline:

```
1. Intent Parsing
2. Parallel Search (Vector + Keyword)
3. Reciprocal Rank Fusion
4. ⭐ Personalization (NEW) ⭐
5. Facet Computation
6. Pagination
```

### Flow Diagram

```
User Search Request
       ↓
   user_id present?
       ↓ Yes
   Check Redis Cache
       ↓
   Cache Miss?
       ↓ Yes
   Call SONA /personalization/score
       ↓
   Get Scores for All Content
       ↓
   Apply Boost Weight
       ↓
   Rerank Results
       ↓
   Cache Scores (5 min TTL)
       ↓
   Return Personalized Results
```

## Usage

### Basic Usage

```rust
use media_gateway_discovery::search::{
    HybridSearchService, SearchRequest, PersonalizationService
};

// Create search request with user_id
let request = SearchRequest {
    query: "action movies".to_string(),
    filters: None,
    page: 1,
    page_size: 20,
    user_id: Some(user_id),  // Personalization happens if present
    experiment_variant: None, // Optional A/B test variant
};

// Execute search
let response = search_service.search(request).await?;

// Results are automatically personalized if user_id was provided
```

### A/B Testing

```rust
// Test different personalization strengths
let request = SearchRequest {
    query: "sci-fi shows".to_string(),
    user_id: Some(user_id),
    experiment_variant: Some("high_boost".to_string()),  // 0.40 weight
    // ...
};
```

### Custom Configuration

```rust
use media_gateway_discovery::config::PersonalizationConfig;

let config = PersonalizationConfig {
    sona_url: "http://sona:8082".to_string(),
    boost_weight: 0.30,        // Custom boost weight
    timeout_ms: 40,            // Tighter timeout
    cache_ttl_sec: 600,        // 10-minute cache
    enabled: true,
};

let service = HybridSearchService::new_with_personalization(
    discovery_config,
    intent_parser,
    vector_search,
    keyword_search,
    db_pool,
    cache,
    config,  // Custom personalization config
);
```

## Configuration

### Environment Variables

```bash
# SONA Service URL
DISCOVERY_PERSONALIZATION_SONA_URL=http://sona:8082

# Boost weight (0.0 - 1.0)
DISCOVERY_PERSONALIZATION_BOOST_WEIGHT=0.25

# Timeout in milliseconds
DISCOVERY_PERSONALIZATION_TIMEOUT_MS=50

# Cache TTL in seconds
DISCOVERY_PERSONALIZATION_CACHE_TTL_SEC=300

# Enable/disable personalization
DISCOVERY_PERSONALIZATION_ENABLED=true
```

### Config File (config/discovery.toml)

```toml
[personalization]
sona_url = "http://localhost:8082"
boost_weight = 0.25
timeout_ms = 50
cache_ttl_sec = 300
enabled = true
```

## A/B Testing Variants

The personalization service supports multiple boost weight variants for experimentation:

| Variant | Weight | Description |
|---------|--------|-------------|
| `control` | 0.0 | No personalization (baseline) |
| `low_boost` | 0.15 | Subtle personalization |
| `medium_boost` | 0.25 | Default personalization |
| `high_boost` | 0.40 | Strong personalization |
| `aggressive_boost` | 0.60 | Maximum personalization |

### Score Calculation

Final relevance score is calculated as:

```
final_score = original_score × (1 - weight) + personalization_score × weight
```

**Example** (medium_boost, weight=0.25):
- Original score: 0.60
- Personalization score: 0.90
- Final score: 0.60 × 0.75 + 0.90 × 0.25 = 0.675

## SONA API Integration

### Request Format

```json
POST /api/v1/personalization/score

{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "content_id": "660e8400-e29b-41d4-a716-446655440001"
}
```

### Response Format

```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "content_id": "660e8400-e29b-41d4-a716-446655440001",
  "score": 0.85,
  "components": {
    "collaborative": 0.35,
    "content_based": 0.25,
    "graph_based": 0.30,
    "context": 0.10,
    "lora_boost": 0.15
  }
}
```

## Caching Strategy

### Cache Key Format

```
personalization:{user_id}:batch
```

### Cache Behavior

- **TTL**: 5 minutes (300 seconds)
- **Storage**: Redis
- **Invalidation**: Automatic expiry + manual via `invalidate_cache()`
- **Miss Strategy**: Fetch from SONA and cache result

### Cache Invalidation

```rust
// Invalidate cache after user preference update
personalization_service
    .invalidate_cache(user_id)
    .await?;
```

## Performance

### Latency Targets

- **Total Overhead**: <50ms
- **Cache Hit**: <5ms
- **Cache Miss**: 40-50ms (SONA call)

### Optimization Techniques

1. **Parallel Fetching**: Scores fetched concurrently for all content
2. **Redis Caching**: Reduces repeated SONA calls
3. **Timeout Control**: 50ms timeout prevents long waits
4. **Connection Pooling**: HTTP client reuses connections

## Error Handling

### Graceful Degradation

Personalization failures **never** break search:

```rust
match personalization_service.personalize_results(...).await {
    Ok(personalized) => personalized,
    Err(e) => {
        tracing::warn!("Personalization failed: {}", e);
        original_results  // Fall back to unpersonalized
    }
}
```

### Common Errors

| Error | Handling |
|-------|----------|
| SONA timeout | Return original results |
| SONA 500 error | Return original results |
| Redis unavailable | Skip cache, call SONA directly |
| Invalid response | Log warning, return original |

## Monitoring

### Tracing

All operations are instrumented with `tracing`:

```rust
#[instrument(skip(self, results), fields(user_id = %user_id, num_results = results.len()))]
pub async fn personalize_results(...) -> Result<...> {
    // Automatically logs:
    // - user_id
    // - number of results
    // - elapsed time
    // - boost weight applied
}
```

### Metrics to Track

- Personalization latency (p50, p95, p99)
- Cache hit rate
- SONA error rate
- Personalized vs unpersonalized search CTR

## Testing

### Unit Tests

```bash
cargo test --package media-gateway-discovery --lib personalization
```

### Integration Tests

```bash
# Requires Redis at localhost:6379
cargo test --test discovery_personalization_integration
```

### Test Coverage

- ✅ SONA HTTP calls
- ✅ Redis caching
- ✅ A/B variant weights
- ✅ Latency requirements
- ✅ Failure scenarios
- ✅ Cache invalidation
- ✅ Score calculations

## Security Considerations

1. **User ID Validation**: User IDs are extracted from validated JWTs
2. **SONA Communication**: Should use HTTPS in production
3. **Rate Limiting**: SONA service should implement rate limits
4. **Cache Isolation**: User preferences are isolated by user_id

## Production Deployment

### Prerequisites

- SONA service running and accessible
- Redis instance for caching
- JWT authentication configured
- Environment variables set

### Health Checks

```bash
# Check Redis connectivity
redis-cli ping

# Check SONA availability
curl http://sona:8082/health

# Test personalization endpoint
curl -X POST http://sona:8082/api/v1/personalization/score \
  -H "Content-Type: application/json" \
  -d '{"user_id": "...", "content_id": "..."}'
```

### Rollout Strategy

1. **Phase 1**: Deploy with `enabled: false` (dark launch)
2. **Phase 2**: Enable for 5% of users (control=0.95)
3. **Phase 3**: A/B test boost weights
4. **Phase 4**: Roll out winning variant to 100%

## Troubleshooting

### Personalization Not Working

**Symptom**: Search results identical for all users

**Checks**:
1. Verify `user_id` is present in request
2. Check SONA service is running
3. Verify Redis connectivity
4. Check `enabled: true` in config
5. Review logs for errors

### High Latency

**Symptom**: Search requests taking >100ms

**Checks**:
1. Check SONA service response time
2. Verify Redis is responsive
3. Review cache hit rate
4. Check network latency to SONA
5. Consider reducing `timeout_ms`

### Cache Issues

**Symptom**: Stale personalization scores

**Solution**:
```rust
// Invalidate cache after preference update
personalization_service.invalidate_cache(user_id).await?;
```

## Future Enhancements

1. **Real-time Updates**: WebSocket connection to SONA for live preference changes
2. **Batch Endpoint**: Single SONA request for multiple content items
3. **Prefetching**: Proactive cache warming for active users
4. **Client-Side Caching**: Cache personalization scores in browser
5. **Fallback Models**: Use local ML model when SONA unavailable

## References

- [SONA Implementation](../../crates/sona/README.md)
- [A/B Testing Framework](../BATCH_004_TASK_004_SUMMARY.md)
- [JWT Authentication](../../crates/auth/README.md)
- [Redis Cache](./src/cache.rs)

---

**Last Updated**: 2025-12-06
**Maintainer**: Media Gateway Team
