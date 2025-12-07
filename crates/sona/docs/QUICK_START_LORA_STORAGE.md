# Quick Start: LoRA Storage

## 🚀 5-Minute Setup

### 1. Apply Database Migration
```bash
psql $DATABASE_URL -f infrastructure/db/postgres/migrations/002_lora_adapters.up.sql
```

### 2. Basic Usage
```rust
use media_gateway_sona::{LoRAStorage, UserLoRAAdapter};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

// Connect to database
let pool = PgPoolOptions::new()
    .max_connections(10)
    .connect(&database_url)
    .await?;

// Initialize storage
let storage = LoRAStorage::new(pool);

// Create adapter
let user_id = Uuid::new_v4();
let mut adapter = UserLoRAAdapter::new(user_id);
adapter.initialize_random();

// Save adapter
let version = storage.save_adapter(&adapter, "default").await?;
println!("Saved version: {}", version);

// Load adapter
let loaded = storage.load_adapter(user_id, "default").await?;
println!("Loaded adapter with {} training iterations", loaded.training_iterations);
```

### 3. Run Tests
```bash
# Unit tests (no database required)
cargo test --package media-gateway-sona --lib lora_storage

# Integration tests (requires PostgreSQL)
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/media_gateway_test"
cargo test --package media-gateway-sona --ignored

# Run example
cargo run --package media-gateway-sona --example lora_storage_example
```

## 📋 Common Operations

### Save New Version
```rust
adapter.training_iterations += 5;
let v2 = storage.save_adapter(&adapter, "default").await?;
```

### Load Specific Version
```rust
let old_version = storage.load_adapter_version(user_id, "default", 1).await?;
```

### List All Adapters
```rust
let adapters = storage.list_adapters(user_id).await?;
for meta in adapters {
    println!("{} v{}: {} bytes", meta.adapter_name, meta.version, meta.size_bytes);
}
```

### Delete Adapter
```rust
let deleted = storage.delete_adapter(user_id, "default").await?;
println!("Deleted {} version(s)", deleted);
```

### A/B Testing
```rust
// Save production and experimental
storage.save_adapter(&prod_adapter, "production").await?;
storage.save_adapter(&exp_adapter, "experimental").await?;

// Load based on user bucket
let name = if user_in_experiment { "experimental" } else { "production" };
let adapter = storage.load_adapter(user_id, name).await?;
```

## 📊 Monitoring

### Storage Statistics
```rust
let stats = storage.get_storage_stats().await?;
println!("Total: {} adapters, {} users, {} MB",
    stats.total_adapters,
    stats.unique_users,
    stats.total_bytes / 1_000_000
);
```

### Database Queries
```sql
-- Storage usage
SELECT
    COUNT(*) as total_adapters,
    COUNT(DISTINCT user_id) as unique_users,
    pg_size_pretty(SUM(size_bytes)) as total_size
FROM lora_adapters;

-- Latest adapters per user
SELECT DISTINCT ON (user_id, adapter_name)
    user_id, adapter_name, version, updated_at
FROM lora_adapters
ORDER BY user_id, adapter_name, version DESC;
```

## ⚡ Performance Tips

1. **Use connection pooling**: `PgPoolOptions::new().max_connections(10)`
2. **Verify indexes exist**: `\d lora_adapters`
3. **Run ANALYZE periodically**: `ANALYZE lora_adapters;`
4. **Clean old versions**: Keep only latest 3-5 versions

## 🔧 Troubleshooting

### Slow queries?
```sql
-- Check if indexes are being used
EXPLAIN ANALYZE
SELECT weights FROM lora_adapters
WHERE user_id = 'uuid' AND adapter_name = 'default'
ORDER BY version DESC LIMIT 1;
```

### Database connection issues?
```rust
// Add timeout and retry logic
let pool = PgPoolOptions::new()
    .max_connections(10)
    .acquire_timeout(Duration::from_secs(3))
    .connect(&database_url)
    .await?;
```

## 📚 Documentation

- **Full Guide**: `docs/lora_storage.md`
- **Implementation Details**: `LORA_STORAGE_README.md`
- **Examples**: `examples/lora_storage_example.rs`
- **Tests**: `src/tests/lora_storage_test.rs`

## ✅ Acceptance Criteria

- ✅ Serialize to PostgreSQL BYTEA
- ✅ <2ms retrieval latency
- ✅ Correct ndarray deserialization
- ✅ Epsilon <0.001 for weight preservation
