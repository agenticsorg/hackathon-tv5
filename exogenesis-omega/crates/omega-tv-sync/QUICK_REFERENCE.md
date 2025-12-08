# Omega-Sync Quick Reference

## Installation

```toml
[dependencies]
omega-sync = { path = "path/to/omega-sync" }
```

## Basic Usage

```rust
use omega_sync::{SyncClient, SyncConfig};
use std::time::Duration;

// 1. Create configuration
let config = SyncConfig::builder()
    .constellation_url("https://constellation.omega.io")
    .device_id("tv-12345")
    .sync_interval(Duration::from_secs(300))  // 5 minutes
    .build()?;

// 2. Initialize client
let mut client = SyncClient::new(config).await?;

// 3. Sync when needed
if client.should_sync() {
    let delta = prepare_local_delta();
    let global_patterns = client.sync(delta).await?;
    apply_global_patterns(&global_patterns);
}
```

## API Reference

### SyncClient

```rust
impl SyncClient {
    // Create new client
    pub async fn new(config: SyncConfig) -> Result<Self>;
    
    // Perform sync (push local delta, pull global patterns)
    pub async fn sync(&mut self, delta: PatternDelta) 
        -> Result<GlobalPatterns>;
    
    // Check if sync is needed
    pub fn should_sync(&self) -> bool;
    
    // Get time until next sync
    pub fn time_until_next_sync(&self) -> Option<Duration>;
    
    // Get current version
    pub fn local_version(&self) -> u64;
}
```

### Delta Encoding

```rust
// Encode delta to compressed bytes
pub fn encode_delta(delta: &PatternDelta) -> Result<Vec<u8>>;

// Decode compressed bytes to delta
pub fn decode_delta(compressed: &[u8]) -> Result<PatternDelta>;

// Compute diff between pattern sets
pub fn compute_diff(
    old: &[ViewingPattern],
    new: &[ViewingPattern],
    version: u64,
) -> PatternDelta;

// Apply delta to base patterns
pub fn apply_diff(
    base: &mut Vec<ViewingPattern>,
    delta: &PatternDelta,
) -> Result<()>;
```

### Retry Logic

```rust
use omega_sync::retry::{with_retry, RetryPolicy};

let policy = RetryPolicy::new(
    3,                           // max attempts
    Duration::from_secs(1),      // base delay
    Duration::from_secs(30),     // max delay
);

let result = with_retry(policy, || async {
    // Your async operation
    Ok(42)
}).await?;
```

## Configuration Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `constellation_url` | String | - | Constellation server URL |
| `device_id` | String | - | Unique device ID |
| `sync_interval` | Duration | 300s | Time between syncs |
| `retry_attempts` | usize | 3 | Max retry attempts |
| `timeout` | Duration | 30s | Request timeout |

## Performance Characteristics

| Metric | Value |
|--------|-------|
| Push bandwidth | <1KB (typical: 400B) |
| Pull bandwidth | <5KB (typical: 4KB) |
| Compression ratio | >10:1 |
| Sync interval | 5-15 minutes |
| Request timeout | 30 seconds |

## Error Handling

```rust
use omega_sync::Error;

match client.sync(delta).await {
    Ok(patterns) => { /* success */ },
    Err(Error::Network(msg)) => { /* network error */ },
    Err(Error::Timeout) => { /* timeout */ },
    Err(Error::RetryExhausted(n)) => { /* max retries */ },
    Err(e) => { /* other errors */ },
}
```

## Metrics

The client automatically records metrics:

```rust
metrics::histogram!("omega_sync_duration_ms").record(duration_ms);
metrics::histogram!("omega_sync_push_bytes").record(push_bytes);
metrics::histogram!("omega_sync_pull_bytes").record(pull_bytes);
metrics::counter!("omega_sync_success").increment(1);
```

## Testing

```bash
# Run all tests
cargo test -p omega-sync

# Run with output
cargo test -p omega-sync -- --nocapture

# Run specific test
cargo test -p omega-sync test_encode_decode_delta
```

## Feature Flags

```toml
# Default: HTTP transport
omega-sync = { path = "..." }

# With QUIC (TODO: not yet implemented)
omega-sync = { path = "...", features = ["quic"] }
```

## Troubleshooting

### "Configuration invalid" error
- Check that `constellation_url` and `device_id` are not empty
- Ensure `sync_interval` is at least 60 seconds
- Verify `timeout` is at least 5 seconds

### "Retry exhausted" error
- Check network connectivity
- Verify Constellation server is reachable
- Increase `retry_attempts` in config
- Check server logs for errors

### "Delta size exceeds 1KB target" warning
- Too many patterns in delta
- Consider increasing sync frequency
- Review pattern filtering logic

## Examples

See `examples/` directory for:
- Basic sync loop
- Custom retry policy
- Error handling patterns
- Metrics integration
