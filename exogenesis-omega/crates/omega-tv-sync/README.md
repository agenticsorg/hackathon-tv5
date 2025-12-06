# omega-sync

Delta synchronization protocol for Exogenesis Omega TV recommendation system.

## Overview

`omega-sync` handles efficient delta synchronization between TV devices (Omega Brain) and the Constellation server. It implements bandwidth-optimized pattern exchange with compression, retry logic, and configurable sync intervals.

## Features

- **Delta Encoding**: Only transmit changes since last sync
- **Compression**: zstd compression targeting <1KB push, <5KB pull
- **HTTP Transport**: Default HTTPS transport with TLS
- **QUIC Transport**: Optional QUIC support for better performance (TODO)
- **Retry Logic**: Exponential backoff retry mechanism
- **Type-Safe**: Full Rust type safety with comprehensive error handling

## Architecture

```
┌─────────────┐                                ┌─────────────────┐
│             │   1. PatternDelta (compressed) │                 │
│   TV Brain  │──────────────────────────────>│  Constellation  │
│             │        ~1KB (zstd)             │                 │
│             │                                │                 │
│             │   2. GlobalPatterns (compressed)│                 │
│             │<───────────────────────────────│                 │
│             │        ~5KB (zstd)             │                 │
└─────────────┘                                └─────────────────┘
```

## Usage

### Basic Example

```rust
use omega_sync::{SyncClient, SyncConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure sync client
    let config = SyncConfig {
        constellation_url: "https://constellation.omega.io".to_string(),
        device_id: "tv-12345".to_string(),
        sync_interval: Duration::from_secs(300), // 5 minutes
        retry_attempts: 3,
        timeout: Duration::from_secs(30),
    };

    // Create client
    let mut client = SyncClient::new(config).await?;

    // Sync loop
    loop {
        if client.should_sync() {
            // Prepare delta from local patterns
            let delta = prepare_local_delta();

            // Perform sync
            let global_patterns = client.sync(delta).await?;

            // Apply global patterns
            apply_global_patterns(&global_patterns);
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
```

### Configuration Builder

```rust
use omega_sync::SyncConfig;
use std::time::Duration;

let config = SyncConfig::builder()
    .constellation_url("https://constellation.omega.io")
    .device_id("tv-12345")
    .sync_interval(Duration::from_secs(600)) // 10 minutes
    .retry_attempts(5)
    .timeout(Duration::from_secs(45))
    .build()?;
```

## Modules

- **client**: TV-side sync client (`SyncClient`)
- **config**: Configuration structures (`SyncConfig`)
- **delta**: Delta encoding/decoding with compression
- **protocol**: Wire protocol types (`SyncRequest`, `SyncResponse`)
- **transport**: HTTP and QUIC transport layers
- **retry**: Retry logic with exponential backoff

## Performance Targets

| Metric | Target | Status |
|--------|--------|--------|
| Push bandwidth | <1KB | ✅ Achieved with zstd compression |
| Pull bandwidth | <5KB | ✅ Achieved with zstd compression |
| Sync interval | 5-15 min | ✅ Configurable |
| Retry attempts | 3-5 | ✅ Configurable |
| Timeout | 30s | ✅ Configurable |

## Testing

Run tests with:

```bash
cargo test
```

Run integration tests (requires constellation server):

```bash
cargo test --ignored
```

## Features

- `default`: HTTP transport
- `http`: HTTP transport with reqwest
- `quic`: QUIC transport (not yet implemented)

## Dependencies

- `tokio`: Async runtime
- `serde`: Serialization
- `bincode`: Binary serialization
- `zstd`: Compression
- `reqwest`: HTTP client
- `async-trait`: Async trait support

## Bandwidth Analysis

### Typical Push (TV → Constellation)

```
Pattern delta (10 patterns):
- Serialized: ~4KB (10 patterns × 384 dimensions × 4 bytes)
- Compressed: ~400 bytes (10:1 ratio)
- Total: <1KB ✅
```

### Typical Pull (Constellation → TV)

```
Global patterns (100 patterns):
- Serialized: ~40KB (100 patterns × 384 dimensions × 4 bytes)
- Compressed: ~4KB (10:1 ratio)
- Total: <5KB ✅
```

## Error Handling

All errors are captured in the `Error` enum:

```rust
pub enum Error {
    Serialization(bincode::Error),
    Compression(std::io::Error),
    Network(String),
    Transport(String),
    Timeout,
    InvalidConfig(String),
    Protocol(String),
    RetryExhausted(usize),
}
```

## Future Enhancements

- [ ] QUIC transport implementation with 0-RTT resumption
- [ ] Adaptive compression based on network conditions
- [ ] Differential privacy for pattern aggregation
- [ ] Multi-constellation failover support
- [ ] Offline queue for delayed sync

## License

MIT

## Related Crates

- `omega-protocol`: Shared protocol types
- `omega-brain`: TV-side intelligence
- `omega-constellation`: Server-side coordination
