# LoRA Model Persistence and Loading Infrastructure

## Overview

The `lora_storage` module provides PostgreSQL-backed persistence for UserLoRAAdapter models, enabling:

- **Sub-2ms retrieval latency** with optimized indexing
- **Efficient binary serialization** using bincode (~40KB per adapter)
- **Versioning support** for model evolution and A/B testing
- **Production-ready error handling** with graceful degradation

## Architecture

### Storage Schema

```sql
CREATE TABLE lora_adapters (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    adapter_name VARCHAR(100) NOT NULL DEFAULT 'default',
    version INTEGER NOT NULL DEFAULT 1,
    weights BYTEA NOT NULL,
    size_bytes BIGINT NOT NULL,
    training_iterations INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    UNIQUE (user_id, adapter_name, version)
);
```

### Indexes for Performance

```sql
-- Primary lookup: get latest adapter for user (<2ms target)
CREATE INDEX idx_lora_adapters_user_name_version
ON lora_adapters(user_id, adapter_name, version DESC);

-- Fast user lookup for list operations
CREATE INDEX idx_lora_adapters_user_updated
ON lora_adapters(user_id, updated_at DESC);

-- Statistics and monitoring
CREATE INDEX idx_lora_adapters_created
ON lora_adapters(created_at DESC);
```

## Usage Examples

### Basic Operations

```rust
use media_gateway_sona::{LoRAStorage, UserLoRAAdapter};
use sqlx::PgPool;
use uuid::Uuid;

// Initialize storage
let pool = PgPool::connect("postgres://...").await?;
let storage = LoRAStorage::new(pool);

// Create and save adapter
let user_id = Uuid::new_v4();
let mut adapter = UserLoRAAdapter::new(user_id);
adapter.initialize_random();

let version = storage.save_adapter(&adapter, "default").await?;
println!("Saved adapter version: {}", version);

// Load latest adapter
let loaded = storage.load_adapter(user_id, "default").await?;
assert_eq!(loaded.user_id, adapter.user_id);

// Load specific version
let v1 = storage.load_adapter_version(user_id, "default", 1).await?;

// List all adapters for user
let adapters = storage.list_adapters(user_id).await?;
for meta in adapters {
    println!(
        "Adapter: {} v{} ({} bytes, {} iterations)",
        meta.adapter_name,
        meta.version,
        meta.size_bytes,
        meta.training_iterations
    );
}

// Delete adapter
let deleted = storage.delete_adapter(user_id, "default").await?;
println!("Deleted {} version(s)", deleted);
```

### Versioning Workflow

```rust
// Save initial model
let v1 = storage.save_adapter(&adapter, "production").await?;

// Train and save improved version
adapter.training_iterations += 10;
let v2 = storage.save_adapter(&adapter, "production").await?;

// Load latest (v2)
let latest = storage.load_adapter(user_id, "production").await?;

// Rollback to v1 if needed
let rollback = storage.load_adapter_version(user_id, "production", 1).await?;
```

### A/B Testing

```rust
// Production model
storage.save_adapter(&production_adapter, "production").await?;

// Experimental model
storage.save_adapter(&experimental_adapter, "experimental").await?;

// Load based on user bucket
let adapter_name = if user_in_experiment {
    "experimental"
} else {
    "production"
};
let adapter = storage.load_adapter(user_id, adapter_name).await?;
```

### Monitoring and Statistics

```rust
// Get storage statistics
let stats = storage.get_storage_stats().await?;
println!("Total adapters: {}", stats.total_adapters);
println!("Unique users: {}", stats.unique_users);
println!("Total storage: {} MB", stats.total_bytes / 1_000_000);
println!("Average size: {} KB", stats.avg_bytes / 1_000.0);

// Count adapters for user
let count = storage.count_adapters(user_id).await?;
```

## Performance Characteristics

### Serialization

- **Format**: bincode (efficient binary serialization)
- **Size**: ~40KB for rank=8 adapter (8×512 + 768×8 f32 matrices)
- **Overhead**: Minimal metadata (~100 bytes)

### Database Operations

| Operation | Target Latency | Notes |
|-----------|----------------|-------|
| `save_adapter()` | <5ms | Includes serialization + INSERT |
| `load_adapter()` | <2ms | With proper indexing |
| `list_adapters()` | <10ms | Depends on adapter count |
| `delete_adapter()` | <3ms | Cascade delete enabled |

### Optimization Tips

1. **Connection Pooling**: Use `PgPoolOptions` with appropriate max_connections
2. **Prepared Statements**: SQLx automatically prepares statements
3. **Index Maintenance**: Run `ANALYZE lora_adapters` periodically
4. **Cleanup**: Delete old versions to prevent unbounded growth

```rust
// Configure connection pool
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(10)
    .acquire_timeout(std::time::Duration::from_secs(3))
    .connect(&database_url)
    .await?;
```

## Error Handling

### Common Errors

```rust
use anyhow::Result;

// Adapter not found
match storage.load_adapter(user_id, "nonexistent").await {
    Ok(adapter) => { /* use adapter */ },
    Err(e) if e.to_string().contains("not found") => {
        // Initialize new adapter
        let new_adapter = UserLoRAAdapter::new(user_id);
        storage.save_adapter(&new_adapter, "default").await?;
    },
    Err(e) => return Err(e),
}

// Serialization error (corrupted data)
match storage.load_adapter(user_id, "default").await {
    Ok(adapter) => { /* use adapter */ },
    Err(e) if e.to_string().contains("deserialize") => {
        tracing::error!("Corrupted adapter data: {}", e);
        // Fallback to fresh adapter
        let fallback = UserLoRAAdapter::new(user_id);
        storage.save_adapter(&fallback, "default").await?;
    },
    Err(e) => return Err(e),
}
```

### Graceful Degradation

```rust
// Fallback to in-memory if database unavailable
let adapter = match storage.load_adapter(user_id, "default").await {
    Ok(adapter) => adapter,
    Err(e) => {
        tracing::warn!("Failed to load adapter from DB: {}", e);
        // Use in-memory fallback
        let mut fallback = UserLoRAAdapter::new(user_id);
        fallback.initialize_random();
        fallback
    }
};
```

## Testing

### Unit Tests

Run with: `cargo test --lib`

- ✅ Serialization round-trip preserves weights (epsilon < 0.001)
- ✅ Bincode serialization size verification
- ✅ SerializableLoRAAdapter conversion correctness

### Integration Tests

Run with: `cargo test --ignored`

Requires PostgreSQL running with `DATABASE_URL` environment variable:

```bash
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/media_gateway_test"
cargo test --ignored
```

Tests:
- ✅ Save and load adapter round-trip
- ✅ Versioning behavior
- ✅ List adapters functionality
- ✅ Delete operations
- ✅ Retrieval latency (<2ms target)
- ✅ Storage statistics

### Performance Benchmarks

```bash
# Run latency benchmark
cargo test --ignored test_retrieval_latency -- --nocapture

# Example output:
# Average load latency: 1.2ms
```

## Migration Guide

### Applying Migration

```bash
# Apply migration
psql $DATABASE_URL -f infrastructure/db/postgres/migrations/002_lora_adapters.up.sql

# Verify table created
psql $DATABASE_URL -c "\d lora_adapters"
```

### Rollback Migration

```bash
psql $DATABASE_URL -f infrastructure/db/postgres/migrations/002_lora_adapters.down.sql
```

## Production Deployment

### Monitoring Queries

```sql
-- Check storage usage
SELECT
    COUNT(*) as total_adapters,
    COUNT(DISTINCT user_id) as unique_users,
    pg_size_pretty(SUM(size_bytes)) as total_size,
    pg_size_pretty(AVG(size_bytes)::bigint) as avg_size
FROM lora_adapters;

-- Find users with most versions
SELECT
    user_id,
    COUNT(*) as version_count,
    MAX(updated_at) as last_updated
FROM lora_adapters
GROUP BY user_id
ORDER BY version_count DESC
LIMIT 10;

-- Monitor latency (enable pg_stat_statements)
SELECT
    query,
    mean_exec_time,
    calls
FROM pg_stat_statements
WHERE query LIKE '%lora_adapters%'
ORDER BY mean_exec_time DESC;
```

### Cleanup Strategy

```sql
-- Keep only latest 3 versions per adapter
DELETE FROM lora_adapters
WHERE id IN (
    SELECT id FROM (
        SELECT
            id,
            ROW_NUMBER() OVER (
                PARTITION BY user_id, adapter_name
                ORDER BY version DESC
            ) as rn
        FROM lora_adapters
    ) sub
    WHERE rn > 3
);

-- Delete adapters older than 90 days
DELETE FROM lora_adapters
WHERE created_at < NOW() - INTERVAL '90 days';
```

### Backup and Recovery

```bash
# Backup all adapters
pg_dump $DATABASE_URL -t lora_adapters --data-only > lora_adapters_backup.sql

# Restore
psql $DATABASE_URL < lora_adapters_backup.sql
```

## Integration with SONA Engine

```rust
use media_gateway_sona::{LoRAStorage, SonaEngine, UpdateUserLoRA};

async fn train_and_persist(
    storage: &LoRAStorage,
    user_id: Uuid,
    events: Vec<ViewingEvent>,
) -> Result<()> {
    // Load or create adapter
    let mut adapter = match storage.load_adapter(user_id, "default").await {
        Ok(adapter) => adapter,
        Err(_) => {
            let mut new_adapter = UserLoRAAdapter::new(user_id);
            new_adapter.initialize_random();
            new_adapter
        }
    };

    // Train adapter
    let engine = SonaEngine::with_default_config();
    UpdateUserLoRA::execute(
        &mut adapter,
        &events,
        |content_id| get_content_embedding(content_id),
        &preference_vector,
    )
    .await?;

    // Persist updated adapter
    storage.save_adapter(&adapter, "default").await?;

    Ok(())
}
```

## Troubleshooting

### Issue: Slow load times (>2ms)

**Solutions:**
1. Verify indexes exist: `\d lora_adapters`
2. Update statistics: `ANALYZE lora_adapters;`
3. Check connection pool settings
4. Monitor disk I/O: `iostat -x 1`

### Issue: Large storage size

**Solutions:**
1. Implement version cleanup (keep latest N versions)
2. Compress old adapters
3. Archive inactive user adapters

### Issue: Serialization errors

**Solutions:**
1. Verify ndarray dimensions match constants
2. Check bincode version compatibility
3. Validate adapter structure before serialization

## References

- [LoRA Paper](https://arxiv.org/abs/2106.09685) - Low-Rank Adaptation of Large Language Models
- [SQLx Documentation](https://docs.rs/sqlx) - Async SQL toolkit
- [Bincode Documentation](https://docs.rs/bincode) - Binary serialization
- [ndarray Documentation](https://docs.rs/ndarray) - N-dimensional arrays

## License

See workspace LICENSE file.
