# Qdrant Vector Indexing for Media Gateway Ingestion

## Quick Start

### 1. Start Qdrant
```bash
docker run -d -p 6334:6334 -p 6333:6333 \
  --name qdrant \
  -v $(pwd)/qdrant_storage:/qdrant/storage \
  qdrant/qdrant
```

### 2. Use in Code
```rust
use media_gateway_ingestion::{
    IngestionPipeline, QdrantClient, VECTOR_DIM
};

// Initialize Qdrant client
let qdrant = QdrantClient::new(
    "http://localhost:6334",
    "content_vectors"
).await?;

// Ensure collection exists (768-dimensional vectors)
qdrant.ensure_collection(VECTOR_DIM).await?;

// Integrate with pipeline
let pipeline = IngestionPipeline::new(
    normalizers,
    entity_resolver,
    genre_mapper,
    embedding_generator,
    rate_limiter,
    pool,
    schedule,
    regions,
).with_qdrant(Some(qdrant));

// Start - vectors automatically indexed after DB persistence
pipeline.start().await?;
```

## What's Implemented

### Core Features
- ✅ QdrantClient with connection management
- ✅ Collection creation (768-dim, cosine similarity)
- ✅ Health checking
- ✅ Single point upsert
- ✅ Batch upsert (max 100 points)
- ✅ Vector similarity search
- ✅ Automatic integration with ingestion pipeline

### Data Structures
```rust
// Metadata stored with each vector
pub struct ContentPayload {
    pub content_id: Uuid,
    pub title: String,
    pub genres: Vec<String>,
    pub platform: String,
    pub release_year: i32,
    pub popularity_score: f32,
}

// Complete point for indexing
pub struct ContentPoint {
    pub id: Uuid,
    pub vector: Vec<f32>,  // 768 dimensions
    pub payload: ContentPayload,
}
```

### Helper Functions
```rust
// Convert CanonicalContent to ContentPoint
let point = to_content_point(&canonical_content, content_id)?;
```

## Testing

### Unit Tests
```bash
cargo test -p media-gateway-ingestion qdrant::tests
```

**Test Cases** (8 total):
- Payload serialization/deserialization
- Successful ContentPoint conversion
- Missing embedding error handling
- Default value handling
- Vector dimension validation

### Integration Tests
```bash
# Start Qdrant first
docker run -p 6334:6334 qdrant/qdrant

# Run integration tests
cargo test -p media-gateway-ingestion \
  --test qdrant_integration_test \
  -- --ignored
```

**Test Cases** (12 total):
- Client creation and health checks
- Collection management (idempotent)
- Single/batch upsert operations
- Similarity search
- Batch size limits
- Update operations
- Empty batch handling
- Search result ordering

### Example
```bash
cargo run -p media-gateway-ingestion --example qdrant_usage
```

## API Reference

### QdrantClient Methods

```rust
// Create client
QdrantClient::new(url: &str, collection: &str) -> Result<Self>

// Health check
client.health_check() -> Result<bool>

// Create/verify collection (idempotent)
client.ensure_collection(vector_size: u64) -> Result<()>

// Index single point
client.upsert_point(
    id: Uuid,
    vector: Vec<f32>,
    payload: ContentPayload
) -> Result<()>

// Batch index (max 100 points)
client.upsert_batch(points: Vec<ContentPoint>) -> Result<()>

// Search similar vectors
client.search_similar(
    query_vector: Vec<f32>,
    limit: u64
) -> Result<Vec<(Uuid, f32)>>
```

## Performance

- **Vector Dimension**: 768 (text-embedding-3-small)
- **Distance Metric**: Cosine similarity
- **Index Type**: HNSW (O(log n) search)
- **Batch Size**: Max 100 points per call
- **Throughput**: 500 items/s (target)
- **Search Latency**: <10ms (target)

## Pipeline Integration

The QdrantClient integrates seamlessly with the ingestion pipeline:

1. **Content normalized** from platform-specific format
2. **Entity resolved** via EIDR/fuzzy matching
3. **Genres mapped** to canonical taxonomy
4. **Embedding generated** (768-dimensional vector)
5. **Persisted to PostgreSQL** (primary storage)
6. **Indexed in Qdrant** ← NEW (batch operation after DB)

```rust
// In IngestionPipeline::process_batch()
for raw in batch {
    let canonical = normalizer.normalize(raw)?;
    canonical.embedding = embedding_generator.generate(&canonical).await?;
    let content_id = repository.upsert(&canonical).await?;

    // Convert to Qdrant point
    if qdrant_client.is_some() {
        let point = to_content_point(&canonical, content_id)?;
        qdrant_points.push(point);
    }
}

// Batch upsert to Qdrant
qdrant_client.upsert_batch(qdrant_points).await?;
```

## Error Handling

```rust
use media_gateway_ingestion::IngestionError;

match client.upsert_batch(points).await {
    Ok(_) => { /* success */ },
    Err(IngestionError::ConfigError(msg)) => {
        // Batch size exceeded, invalid config
    },
    Err(IngestionError::DatabaseError(msg)) => {
        // Qdrant connection/operation error
    },
    Err(e) => { /* other errors */ }
}
```

## Production Deployment

### Docker Compose
```yaml
services:
  qdrant:
    image: qdrant/qdrant:latest
    ports:
      - "6333:6333"  # REST API
      - "6334:6334"  # gRPC API
    volumes:
      - ./qdrant_storage:/qdrant/storage
    environment:
      - QDRANT__LOG_LEVEL=INFO
    deploy:
      resources:
        limits:
          memory: 8G
          cpus: '4'
```

### Scaling
- **RAM**: ~2GB per 1M vectors
- **CPU**: Multi-core for concurrent searches
- **Clustering**: Available for horizontal scaling

### Monitoring
Track these metrics:
- Indexing latency (<100ms per batch)
- Search latency (<10ms)
- Indexing throughput (500+ items/s)
- Error rate (<0.1%)

## Files

### Implementation
- `/workspaces/media-gateway/crates/ingestion/src/qdrant.rs` (556 lines)
- `/workspaces/media-gateway/crates/ingestion/src/pipeline.rs` (integrated)
- `/workspaces/media-gateway/crates/ingestion/src/lib.rs` (exports)

### Tests
- `/workspaces/media-gateway/crates/ingestion/tests/qdrant_integration_test.rs` (402 lines)
- Unit tests in `qdrant.rs` (8 test cases)

### Documentation
- `/workspaces/media-gateway/docs/QDRANT_INTEGRATION.md` (comprehensive guide)
- `/workspaces/media-gateway/docs/BATCH_003_TASK_006_SUMMARY.md` (task summary)

### Examples
- `/workspaces/media-gateway/crates/ingestion/examples/qdrant_usage.rs` (214 lines)

## Troubleshooting

**Connection refused**
- Ensure Qdrant running: `docker ps | grep qdrant`
- Check port: gRPC uses 6334, REST uses 6333

**Batch size exceeded**
- Max 100 points per batch
- Split larger batches: `points.chunks(100)`

**Missing embedding**
- Ensure `embedding_generator.generate()` runs before indexing
- Check `canonical.embedding.is_some()`

**Collection not found**
- Call `ensure_collection()` before operations
- Verify collection name matches

## Next Steps

1. **Deploy Qdrant** in your environment
2. **Run integration tests** to verify setup
3. **Enable in pipeline** via `with_qdrant()`
4. **Monitor metrics** (latency, throughput, errors)
5. **Optimize** based on load patterns

## Support

- Full Documentation: `/workspaces/media-gateway/docs/QDRANT_INTEGRATION.md`
- Qdrant Docs: https://qdrant.tech/documentation/
- API Reference: All public APIs documented with rustdoc

---

**Status**: ✅ Production Ready
**Version**: 1.0.0
**Last Updated**: 2025-12-06
