# Omega TV SDK

C FFI bindings for TV manufacturers to integrate the Omega Brain recommendation system.

## Features

- **Fast Recommendations**: <15ms latency for on-device AI
- **Privacy-First**: Federated learning, only patterns sync (not raw data)
- **Offline Capable**: Works without network connection
- **Thread-Safe**: C API safe for multi-threaded environments
- **JSON-Based**: Simple integration with JSON data exchange

## Quick Start

### 1. Build the SDK

```bash
cargo build --release
```

This creates:
- `target/release/libomega_tv_sdk.so` (Linux)
- `target/release/libomega_tv_sdk.dylib` (macOS)
- `target/release/omega_tv_sdk.dll` (Windows)

### 2. Link in Your C/C++ Project

```c
#include "omega_sdk.h"

int main() {
    // Initialize
    if (omega_init("/data/omega", "https://constellation.example.com") != OMEGA_SUCCESS) {
        fprintf(stderr, "Init failed: %s\n", omega_get_last_error());
        return 1;
    }

    // Get recommendations
    const char* context = "{\"genre\":\"action\",\"time\":\"evening\"}";
    char buffer[8192];
    if (omega_recommend(context, buffer, sizeof(buffer)) == OMEGA_SUCCESS) {
        printf("Recommendations: %s\n", buffer);
    }

    // Record viewing
    const char* event = "{\"content_id\":\"movie123\",\"watch_percentage\":0.85}";
    omega_observe(event);

    // Sync (optional, can be periodic)
    omega_sync();

    // Cleanup
    omega_shutdown();
    return 0;
}
```

### 3. Compile Your App

```bash
gcc your_app.c -L./target/release -lomega_tv_sdk -o your_app
```

### 4. Run

```bash
LD_LIBRARY_PATH=./target/release ./your_app
```

## API Reference

### Core Functions

| Function | Description | Latency |
|----------|-------------|---------|
| `omega_init()` | Initialize SDK | 100-500ms |
| `omega_recommend()` | Get recommendations | <15ms |
| `omega_observe()` | Record viewing event | <5ms |
| `omega_sync()` | Sync with constellation | 1-5s |
| `omega_shutdown()` | Cleanup resources | 100-200ms |

### Error Codes

| Code | Name | Description |
|------|------|-------------|
| 0 | `OMEGA_SUCCESS` | Operation successful |
| -1 | `OMEGA_ERR_INIT` | Initialization failed |
| -2 | `OMEGA_ERR_RECOMMEND` | Recommendation failed |
| -3 | `OMEGA_ERR_OBSERVE` | Observation failed |
| -4 | `OMEGA_ERR_SYNC` | Sync failed |
| -5 | `OMEGA_ERR_INVALID_ARG` | Invalid argument |
| -6 | `OMEGA_ERR_JSON_PARSE` | JSON parsing error |
| -7 | `OMEGA_ERR_BUFFER_TOO_SMALL` | Buffer too small |
| -8 | `OMEGA_ERR_NOT_INITIALIZED` | SDK not initialized |
| -9 | `OMEGA_ERR_ALREADY_INITIALIZED` | Already initialized |
| -10 | `OMEGA_ERR_INTERNAL` | Internal error |

## JSON Formats

### Context JSON (for recommendations)

```json
{
  "genre": "action",           // Optional
  "time": "evening",           // Optional
  "device": "main_tv",         // Optional
  "user_profile": "family",    // Optional
  "language": "en",            // Optional
  "current_content": "movie123" // Optional
}
```

### Event JSON (for observations)

```json
{
  "content_id": "movie123",      // Required
  "watch_percentage": 0.85,      // Required (0.0-1.0)
  "session_id": "session_abc",   // Optional
  "duration_seconds": 3600,      // Optional
  "user_rating": 4.5,            // Optional (0.0-5.0)
  "timestamp": "2025-12-06T10:30:00Z" // Optional
}
```

### Recommendation JSON (response)

```json
[
  {
    "content_id": "movie456",
    "title": "Action Movie",
    "score": 0.95,
    "reason": "Based on your viewing history"
  }
]
```

## Examples

See the `examples/` directory:

- `simple_usage.c` - Basic integration example
- `async_usage.c` - Multi-threaded with background sync

Build examples:

```bash
cd examples
gcc simple_usage.c -L../target/release -lomega_tv_sdk -o simple
gcc async_usage.c -L../target/release -lomega_tv_sdk -pthread -o async
```

## Architecture

```
┌─────────────────────────────────────┐
│         Your TV Application          │
│         (C/C++ codebase)             │
└──────────────┬──────────────────────┘
               │ omega_sdk.h
               ▼
┌─────────────────────────────────────┐
│       omega-tv-sdk (C FFI)           │
│     - Thread-safe global state       │
│     - JSON serialization             │
│     - Error handling                 │
└──────────────┬──────────────────────┘
               │ Rust API
               ▼
┌─────────────────────────────────────┐
│       omega-tv-brain                 │
│     - AgentDB (vectors)              │
│     - CosmicMemory (12-tier)         │
│     - LoopEngine (7 loops)           │
│     - Persistence (SQLite)           │
└──────────────┬──────────────────────┘
               │ Network
               ▼
┌─────────────────────────────────────┐
│     Constellation Server             │
│     - Federated aggregation          │
│     - Global patterns                │
└─────────────────────────────────────┘
```

## Performance

- **Recommendation latency**: <15ms (on-device)
- **Observation recording**: <5ms (async processing)
- **Sync bandwidth**: ~1KB upload, ~5KB download
- **Storage**: ~50MB per 10,000 patterns
- **Memory**: ~100MB RAM

## Privacy

✅ **Private by Design**:
- All viewing data processed locally
- Only high-quality patterns synced (success_rate ≥ 0.7)
- No raw viewing data leaves the device
- Federated learning aggregates patterns across devices
- User cannot be identified from synced patterns

## Thread Safety

All SDK functions are thread-safe:
- Use internal mutex for state protection
- Thread-local error messages
- Safe to call from multiple threads
- Background sync recommended

## Best Practices

### Initialization
```c
// Initialize once at app startup
omega_init("/data/omega", "https://constellation.example.com");
```

### Recommendations
```c
// Call on-demand, <15ms latency
omega_recommend(context_json, buffer, buffer_size);
```

### Observations
```c
// Record after viewing events
omega_observe(event_json);
```

### Sync
```c
// Background thread, every 10 minutes
pthread_create(&tid, NULL, sync_worker, NULL);
```

### Cleanup
```c
// Before app exit
omega_shutdown();
```

## Troubleshooting

### "SDK not initialized" error
Call `omega_init()` before any other SDK functions.

### "Buffer too small" error
Increase buffer size to at least 8192 bytes for recommendations.

### Sync failures
Normal if offline. Recommendations work without sync.

### High memory usage
Adjust `max_patterns` in `TVBrainConfig` (default: 10,000).

## License

MIT OR Apache-2.0

## Support

- Documentation: See `include/omega_sdk.h`
- Examples: See `examples/` directory
- Issues: Report on GitHub
