# Media Gateway Sync Service

Real-time cross-device synchronization service with CRDT support for the Media Gateway platform.

## Features

### CRDT-Based Synchronization
- **Hybrid Logical Clock (HLC)**: 48-bit physical time + 16-bit logical counter for distributed timestamp ordering
- **LWW-Register**: Last-Writer-Wins for watch progress and preferences
- **OR-Set**: Observed-Remove Set for watchlists with add-wins semantics

### Real-Time Communication
- **WebSocket Support**: Bidirectional real-time sync with 30s heartbeat
- **PubNub Integration**: Pub/sub messaging for cross-device state propagation
- **Device Presence**: Track online/offline status with 60s timeout

### API Endpoints

#### HTTP REST API (Port 8083)
- `GET /health` - Health check
- `POST /api/v1/sync/watchlist` - Add/remove from watchlist
- `POST /api/v1/sync/progress` - Update watch progress
- `GET /api/v1/devices` - List user devices
- `POST /api/v1/devices/handoff` - Handoff content between devices

#### WebSocket API
- `GET /ws` - Establish real-time sync connection

## Architecture

### Channel Structure
```
user.{userId}.sync         - Watchlist, preferences, progress
user.{userId}.devices       - Device presence, heartbeat
user.{userId}.notifications - Alerts, recommendations
```

### CRDT Operations

#### Watchlist (OR-Set)
```rust
// Add to watchlist
let update = watchlist_sync.add_to_watchlist("content-1".to_string());

// Remove from watchlist
let updates = watchlist_sync.remove_from_watchlist("content-1");

// Concurrent add/remove → add wins
```

#### Watch Progress (LWW-Register)
```rust
// Update progress
let update = progress_sync.update_progress(
    "content-1".to_string(),
    100,          // position_seconds
    1000,         // duration_seconds
    PlaybackState::Playing,
);

// Concurrent updates → latest timestamp wins
```

## Performance Targets

| Metric | p50 | p95 | p99 |
|--------|-----|-----|-----|
| Cross-device sync latency | 50ms | 100ms | 150ms |
| WebSocket connection setup | <100ms | <150ms | <200ms |
| CRDT operation size | <500 bytes | <750 bytes | <1KB |

## Configuration

Environment variables:
- `SYNC_HOST` - Server host (default: `0.0.0.0`)
- `SYNC_PORT` - Server port (default: `8083`)
- `PUBNUB_PUBLISH_KEY` - PubNub publish key
- `PUBNUB_SUBSCRIBE_KEY` - PubNub subscribe key

## Usage

### Running the service
```bash
cd /workspaces/media-gateway/crates/sync
cargo run
```

### Running tests
```bash
cargo test
```

### Example API calls

**Add to watchlist:**
```bash
curl -X POST http://localhost:8083/api/v1/sync/watchlist \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "add",
    "content_id": "tmdb:550"
  }'
```

**Update watch progress:**
```bash
curl -X POST http://localhost:8083/api/v1/sync/progress \
  -H "Content-Type: application/json" \
  -d '{
    "content_id": "tmdb:550",
    "position_seconds": 3245,
    "duration_seconds": 8217,
    "state": "paused"
  }'
```

**Device handoff:**
```bash
curl -X POST http://localhost:8083/api/v1/devices/handoff \
  -H "Content-Type: application/json" \
  -d '{
    "target_device_id": "tv-samsung-living-room",
    "content_id": "tmdb:550"
  }'
```

## WebSocket Message Format

### Watchlist Update
```json
{
  "type": "watchlist_update",
  "operation": "add",
  "content_id": "tmdb:550",
  "timestamp": "1733500800000000"
}
```

### Progress Update
```json
{
  "type": "progress_update",
  "content_id": "tmdb:550",
  "position_seconds": 3245,
  "timestamp": "1733500800000000"
}
```

### Device Command
```json
{
  "type": "device_command",
  "target_device_id": "tv-samsung-living-room",
  "command": "play"
}
```

## Module Structure

```
src/
├── lib.rs                  # Public API exports
├── main.rs                 # Server entry point
├── server.rs               # Actix-web HTTP/WebSocket server
├── websocket.rs            # WebSocket connection handler
├── pubnub.rs               # PubNub client integration
├── device.rs               # Device management and presence
├── crdt/
│   ├── mod.rs              # CRDT module exports
│   ├── hlc.rs              # Hybrid Logical Clock
│   ├── lww_register.rs     # Last-Writer-Wins Register
│   └── or_set.rs           # Observed-Remove Set
└── sync/
    ├── mod.rs              # Sync module exports
    ├── watchlist.rs        # Watchlist synchronization
    └── progress.rs         # Watch progress synchronization
```

## Dependencies

- `actix-web` 4.9 - HTTP server framework
- `actix-web-actors` 4.3 - WebSocket support
- `tokio` 1.41 - Async runtime
- `serde` 1.0 - Serialization
- `reqwest` 0.12 - PubNub HTTP client
- `uuid` 1.11 - Unique tags for OR-Set
- `chrono` 0.4 - Timestamp handling
- `parking_lot` 0.12 - Efficient synchronization primitives

## CRDT Conflict Resolution

### LWW-Register (Watch Progress)
1. Compare HLC timestamps
2. Latest timestamp wins
3. Tie-breaker: lexicographic device_id comparison

### OR-Set (Watchlist)
1. Each add operation gets unique UUID tag
2. Remove operations mark tags as removed
3. Effective set = additions - removals
4. Concurrent add/remove → add wins (new tag created)

## Testing

Comprehensive test coverage includes:
- CRDT merge correctness
- Concurrent operation handling
- Timestamp ordering verification
- WebSocket connection management
- API endpoint validation

Run tests:
```bash
cargo test
```

Run with tracing:
```bash
RUST_LOG=debug cargo test -- --nocapture
```

## Production Considerations

1. **Authentication**: Add JWT validation middleware
2. **Multi-tenancy**: User/device isolation per request
3. **Persistence**: PostgreSQL + Memorystore for state
4. **Scaling**: Horizontal scaling with consistent hashing
5. **Monitoring**: OpenTelemetry tracing and metrics
6. **Rate Limiting**: Per-user/device rate limits
7. **Encryption**: End-to-end encryption for sensitive data

## License

See main repository LICENSE file.
