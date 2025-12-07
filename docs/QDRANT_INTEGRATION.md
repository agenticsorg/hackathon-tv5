# Qdrant Vector Indexing Integration

## Overview

This document describes the Qdrant vector database integration for the Media Gateway ingestion pipeline. The integration enables semantic search and content recommendation by indexing 768-dimensional embeddings generated for each piece of content.

## Architecture

### Components

1. **QdrantClient** (`/workspaces/media-gateway/crates/ingestion/src/qdrant.rs`)
   - Manages connections to Qdrant server
   - Handles collection creation and management
   - Provides batch and single-point upsert operations
   - Implements similarity search

2. **ContentPayload** - Metadata stored with each vector:
   - `content_id`: Database UUID for the content
   - `title`: Content title
   - `genres`: List of genres
   - `platform`: Platform identifier (netflix, prime_video, etc.)
   - `release_year`: Release year
   - `popularity_score`: User rating/popularity metric

3. **ContentPoint** - Complete point structure:
   - `id`: UUID (matches content_id)
   - `vector`: 768-dimensional embedding
   - `payload`: ContentPayload metadata

### Integration Points

```rust
// In IngestionPipeline::process_batch()
for raw_content in batch {
    // 1. Normalize platform data
    let canonical = normalizer.normalize(raw_content)?;

    // 2. Resolve entity
    let entity_match = entity_resolver.resolve(&canonical).await?;

    // 3. Map genres
    canonical.genres = genre_mapper.map_genres(...);

    // 4. Generate embedding (768-dim vector)
    canonical.embedding = embedding_generator.generate(&canonical).await?;

    // 5. Persist to PostgreSQL
    let content_id = repository.upsert(&canonical).await?;

    // 6. Index in Qdrant (NEW)
    if let Some(qdrant_client) = qdrant_client {
        let point = to_content_point(&canonical, content_id)?;
        qdrant_points.push(point);
    }
}

// Batch upsert to Qdrant after DB persistence
qdrant_client.upsert_batch(qdrant_points).await?;
```

## Setup

### 1. Start Qdrant

```bash
# Using Docker
docker run -p 6334:6334 -p 6333:6333 \
  -v $(pwd)/qdrant_storage:/qdrant/storage \
  qdrant/qdrant

# Using Docker Compose
docker-compose up -d qdrant
```

### 2. Configure Pipeline

```rust
use media_gateway_ingestion::{IngestionPipeline, QdrantClient, VECTOR_DIM};

// Initialize Qdrant client
let qdrant_client = QdrantClient::new(
    "http://localhost:6334",
    "content_vectors"
).await?;

// Ensure collection exists
qdrant_client.ensure_collection(VECTOR_DIM).await?;

// Create pipeline with Qdrant integration
let pipeline = IngestionPipeline::new(
    normalizers,
    entity_resolver,
    genre_mapper,
    embedding_generator,
    rate_limiter,
    pool,
    schedule,
    regions,
).with_qdrant(Some(qdrant_client));

// Start pipeline
pipeline.start().await?;
```

## API Reference

### QdrantClient Methods

#### `new(url: &str, collection: &str) -> Result<Self>`
Create a new Qdrant client.

```rust
let client = QdrantClient::new("http://localhost:6334", "content_vectors").await?;
```

#### `health_check() -> Result<bool>`
Verify Qdrant server is healthy.

```rust
let healthy = client.health_check().await?;
assert!(healthy);
```

#### `ensure_collection(vector_size: u64) -> Result<()>`
Create collection if it doesn't exist (idempotent).

```rust
// Uses cosine similarity for 768-dimensional vectors
client.ensure_collection(768).await?;
```

#### `upsert_point(id: Uuid, vector: Vec<f32>, payload: ContentPayload) -> Result<()>`
Upsert a single point.

```rust
let content_id = Uuid::new_v4();
let embedding = vec![0.1; 768];
let payload = ContentPayload {
    content_id,
    title: "The Matrix".to_string(),
    genres: vec!["Action".to_string(), "Sci-Fi".to_string()],
    platform: "netflix".to_string(),
    release_year: 1999,
    popularity_score: 8.7,
};

client.upsert_point(content_id, embedding, payload).await?;
```

#### `upsert_batch(points: Vec<ContentPoint>) -> Result<()>`
Batch upsert (max 100 points per call).

```rust
let points = vec![point1, point2, point3];
client.upsert_batch(points).await?;
```

#### `search_similar(query_vector: Vec<f32>, limit: u64) -> Result<Vec<(Uuid, f32)>>`
Find similar content by vector similarity.

```rust
let query_embedding = vec![0.1; 768];
let results = client.search_similar(query_embedding, 10).await?;

for (content_id, similarity_score) in results {
    println!("Content {}: {:.4}", content_id, similarity_score);
}
```

### Helper Functions

#### `to_content_point(content: &CanonicalContent, content_id: Uuid) -> Result<ContentPoint>`
Convert CanonicalContent to ContentPoint.

```rust
let point = to_content_point(&canonical_content, content_id)?;
```

## Performance Characteristics

### Batch Operations
- **Recommended batch size**: 100 points
- **Maximum batch size**: 100 points (enforced)
- **Throughput**: Processes 500 items/s with batching

### Vector Indexing
- **Dimension**: 768 (text-embedding-3-small)
- **Distance metric**: Cosine similarity
- **Index type**: HNSW (Hierarchical Navigable Small World)

### Search Performance
- **Complexity**: O(log n) average case
- **Latency**: <10ms for collections up to 1M vectors
- **Scalability**: Linear with cluster size

## Error Handling

```rust
use media_gateway_ingestion::IngestionError;

match client.upsert_batch(points).await {
    Ok(_) => println!("Successfully indexed {} points", points.len()),
    Err(IngestionError::ConfigError(msg)) => {
        eprintln!("Configuration error: {}", msg);
        // Handle batch size exceeded, etc.
    }
    Err(IngestionError::DatabaseError(msg)) => {
        eprintln!("Qdrant error: {}", msg);
        // Handle connection issues, collection errors, etc.
    }
    Err(e) => eprintln!("Unexpected error: {}", e),
}
```

## Testing

### Unit Tests

```bash
# Run unit tests for qdrant module
cargo test -p media-gateway-ingestion qdrant::tests
```

Tests include:
- Payload serialization/deserialization
- ContentPoint creation from CanonicalContent
- Error handling for missing embeddings
- Default value handling

### Integration Tests

```bash
# Start Qdrant first
docker run -p 6334:6334 qdrant/qdrant

# Run integration tests
cargo test -p media-gateway-ingestion --test qdrant_integration_test -- --ignored
```

Integration tests verify:
- Client creation and health checks
- Collection creation (idempotent)
- Single point upsert
- Batch upsert (various sizes)
- Similarity search
- Update operations (upsert same ID)
- Batch size limits
- Empty batch handling
- Search result ordering
- Result limit enforcement

### Example Usage

```bash
# Run the example
cargo run -p media-gateway-ingestion --example qdrant_usage
```

## Monitoring

### Metrics to Track

1. **Indexing Latency**
   - Time from DB persistence to Qdrant indexing
   - Target: <100ms per batch

2. **Search Latency**
   - Vector similarity search response time
   - Target: <10ms for top-10 results

3. **Indexing Throughput**
   - Points indexed per second
   - Target: 500+ points/s with batching

4. **Error Rate**
   - Failed upsert operations
   - Target: <0.1%

### Health Check Endpoint

```rust
// Add to service health check
async fn health_check() -> Result<HealthStatus> {
    let qdrant_healthy = qdrant_client.health_check().await?;

    HealthStatus {
        database: db_healthy,
        qdrant: qdrant_healthy,
        // ... other services
    }
}
```

## Production Deployment

### Recommended Configuration

```yaml
# docker-compose.yml
services:
  qdrant:
    image: qdrant/qdrant:latest
    ports:
      - "6333:6333"  # REST API
      - "6334:6334"  # gRPC API
    volumes:
      - ./qdrant_storage:/qdrant/storage
    environment:
      - QDRANT__SERVICE__GRPC_PORT=6334
      - QDRANT__LOG_LEVEL=INFO
    deploy:
      resources:
        limits:
          memory: 8G
          cpus: '4'
    restart: unless-stopped
```

### Scaling Considerations

1. **Vertical Scaling**
   - RAM: ~2GB per 1M vectors (768-dim)
   - CPU: Multi-core for concurrent searches

2. **Horizontal Scaling**
   - Qdrant supports clustering (distributed mode)
   - Sharding by collection or custom logic

3. **Backup Strategy**
   - Snapshot collection regularly
   - Backup `/qdrant/storage` volume
   - Can rebuild from PostgreSQL if needed

## Troubleshooting

### Common Issues

1. **Connection refused**
   ```
   Error: Failed to create Qdrant client: connection refused
   ```
   **Solution**: Ensure Qdrant is running on correct port (6334 for gRPC)

2. **Batch size exceeded**
   ```
   Error: Batch size 150 exceeds maximum of 100
   ```
   **Solution**: Split batch into chunks of 100 or less

3. **Missing embedding**
   ```
   Error: Content missing embedding vector
   ```
   **Solution**: Ensure embedding_generator runs before Qdrant indexing

4. **Collection not found**
   ```
   Error: Collection 'content_vectors' not found
   ```
   **Solution**: Call `ensure_collection()` before operations

## Future Enhancements

1. **Filtered Search**
   - Filter by genre, platform, release year
   - Combine vector similarity with metadata filters

2. **Hybrid Search**
   - Combine full-text search with vector similarity
   - Weighted ranking

3. **Multi-Vector Support**
   - Separate vectors for text, metadata, graph embeddings
   - Named vectors feature in Qdrant

4. **Automatic Re-indexing**
   - Detect stale embeddings
   - Periodic refresh based on model updates

5. **A/B Testing**
   - Multiple embedding models
   - Compare recommendation quality

## References

- [Qdrant Documentation](https://qdrant.tech/documentation/)
- [qdrant-client Rust Crate](https://docs.rs/qdrant-client/)
- [HNSW Algorithm](https://arxiv.org/abs/1603.09320)
- [Cosine Similarity](https://en.wikipedia.org/wiki/Cosine_similarity)
