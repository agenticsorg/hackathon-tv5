# Omega SDK

The official SDK for integrating Exogenesis Omega distributed TV recommendation system.

## Overview

Omega SDK provides a simple, high-performance interface for TV manufacturers to integrate intelligent, privacy-preserving recommendations. All inference happens **locally on the TV**, with periodic pattern synchronization to the Omega Constellation.

## Key Features

- ✅ **Sub-15ms Recommendations**: Local ONNX inference with RuVector search
- ✅ **Privacy-First**: Viewing data never leaves the TV
- ✅ **Minimal Bandwidth**: ~1KB push, ~5KB pull per sync (every 5-15 minutes)
- ✅ **Memory Efficient**: ~200MB total footprint
- ✅ **C FFI**: Native integration for C/C++ TV platforms
- ✅ **WebAssembly**: Support for web-based smart TV platforms

## Quick Start

### Rust API

```rust
use omega_sdk::OmegaBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Omega Brain
    let mut brain = OmegaBuilder::new()
        .model_path("/data/omega/model.onnx")
        .storage_path("/data/omega/brain.db")
        .constellation_url("https://constellation.omega.tv")
        .build()
        .await?;

    // Get recommendations
    let context = r#"{"time":"evening","mood":"relaxed"}"#;
    let recommendations = brain.recommend(context).await?;

    // Record viewing event
    let event = r#"{"content_id":"movie123","watch_pct":0.95}"#;
    brain.observe(event).await?;

    // Sync with constellation (call every 5-15 minutes)
    brain.sync().await?;

    Ok(())
}
```

### C API

```c
#include "omega_sdk.h"

int main() {
    // Initialize
    int result = omega_init(
        "/data/omega/model.onnx",
        "/data/omega/brain.db"
    );
    if (result != 0) {
        fprintf(stderr, "Init failed: %s\n", omega_get_last_error());
        return 1;
    }

    // Get recommendations
    char recommendations[4096];
    result = omega_recommend(
        "{\"time\":\"evening\",\"mood\":\"relaxed\"}",
        recommendations,
        sizeof(recommendations)
    );

    if (result == 0) {
        printf("Recommendations: %s\n", recommendations);
    }

    // Record viewing
    omega_observe("{\"content_id\":\"movie123\",\"watch_pct\":0.95}");

    // Sync (call periodically)
    omega_sync();

    // Cleanup
    omega_shutdown();
    return 0;
}
```

## Architecture

```
┌─────────────────────────────────────┐
│         OMEGA BRAIN (~200MB)        │
├─────────────────────────────────────┤
│ RuVector (50MB)                     │
│  - HNSW index with SIMD             │
│  - <1ms vector search               │
├─────────────────────────────────────┤
│ ONNX Runtime (100MB)                │
│  - MiniLM 4-bit quantized           │
│  - <10ms inference                  │
├─────────────────────────────────────┤
│ AgentDB Memory (50MB)               │
│  - Pattern learning                 │
│  - Episode storage                  │
├─────────────────────────────────────┤
│ Sync Agent (5MB)                    │
│  - Delta-only protocol              │
│  - QUIC transport                   │
└─────────────────────────────────────┘
```

## Performance Targets

| Metric | Target |
|--------|--------|
| Recommendation latency | <15ms (p99) |
| Vector search | <1ms |
| ONNX inference | <10ms |
| Memory footprint | <200MB |
| Sync bandwidth | ~1KB push, ~5KB pull |

## Building

### Rust Library

```bash
cargo build --release
```

### C Library (Shared/Dynamic)

```bash
cargo build --release --lib
# Output: target/release/libomega_sdk.so (Linux)
#         target/release/libomega_sdk.dylib (macOS)
#         target/release/omega_sdk.dll (Windows)
```

### Static Library

```bash
# Edit Cargo.toml to include "staticlib" in crate-type
cargo build --release --lib
# Output: target/release/libomega_sdk.a
```

### WebAssembly

```bash
cargo build --release --target wasm32-unknown-unknown --features wasm
```

## Integration Guide

### For TV Manufacturers (C/C++)

1. **Copy files to your project**:
   - `libomega_sdk.so` (or `.a` for static linking)
   - `include/omega_sdk.h`

2. **Link in your build system**:
   ```cmake
   # CMakeLists.txt
   target_link_libraries(your_tv_app omega_sdk)
   ```

3. **Initialize on TV boot**:
   ```c
   omega_init("/data/omega/model.onnx", "/data/omega/brain.db");
   ```

4. **Call recommend() when user browses**:
   ```c
   char recs[4096];
   omega_recommend(context_json, recs, sizeof(recs));
   ```

5. **Record viewing events**:
   ```c
   omega_observe(event_json);
   ```

6. **Sync periodically** (background thread, every 5-15 min):
   ```c
   omega_sync();
   ```

### For Smart TV Web Apps (JavaScript)

```javascript
import init, { OmegaBrain } from './omega_sdk.js';

await init();
const brain = await OmegaBrain.init({
    modelPath: '/models/omega.onnx',
    storagePath: 'omega-brain-db'
});

const recommendations = await brain.recommend({
    time: 'evening',
    mood: 'relaxed'
});
```

## API Reference

### C API Functions

| Function | Description | Returns |
|----------|-------------|---------|
| `omega_init()` | Initialize Omega Brain | 0 on success |
| `omega_recommend()` | Get recommendations | 0 on success |
| `omega_observe()` | Record viewing event | 0 on success |
| `omega_sync()` | Sync with constellation | 0 on success |
| `omega_shutdown()` | Cleanup and save state | 0 on success |
| `omega_get_last_error()` | Get error message | String or NULL |
| `omega_free_error()` | Free error string | - |

### Error Codes

| Code | Constant | Description |
|------|----------|-------------|
| 0 | `OMEGA_SUCCESS` | Success |
| -1 | `OMEGA_ERR_INIT` | Initialization error |
| -2 | `OMEGA_ERR_INFERENCE` | Inference error |
| -3 | `OMEGA_ERR_SYNC` | Sync error |
| -4 | `OMEGA_ERR_STORAGE` | Storage error |
| -5 | `OMEGA_ERR_INVALID` | Invalid input |
| -99 | `OMEGA_ERR_INTERNAL` | Internal error |

## JSON Schemas

### Context JSON

```json
{
  "time": "evening",         // "morning" | "afternoon" | "evening" | "night"
  "mood": "relaxed",         // "relaxed" | "energetic" | "focused" | "bored"
  "recent": ["action", "thriller"]  // Recent genres or content IDs
}
```

### Event JSON

```json
{
  "content_id": "movie123",  // Required: unique content identifier
  "watch_pct": 0.95,         // Required: 0.0-1.0, percentage watched
  "duration_sec": 3600,      // Optional: total watch duration
  "rating": 4,               // Optional: 1-5 user rating
  "completed": true          // Optional: finished watching
}
```

### Recommendation Response JSON

```json
[
  {
    "content_id": "movie456",
    "score": 0.95,
    "metadata": {
      "title": "Thriller Movie",
      "genre": "thriller",
      "year": 2023
    }
  }
]
```

## Testing

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test error
cargo test ffi
cargo test builder

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

## Examples

See the `examples/` directory for complete working examples:

- `examples/simple.rs` - Basic Rust usage
- `examples/c_integration.c` - C integration example
- `examples/background_sync.rs` - Background sync thread example
- `examples/wasm_demo.html` - WebAssembly demo

## Troubleshooting

### Error: "Model file not found"
- Ensure the ONNX model file exists at the specified path
- Download the model from: https://omega.tv/downloads/model.onnx

### Error: "Failed to initialize ONNX runtime"
- Verify the model is a valid ONNX file
- Ensure the model is compatible with ONNX Runtime version

### High Memory Usage
- Check `max_patterns` configuration (default: 10,000)
- Reduce if memory is constrained

### Sync Failures
- Verify network connectivity
- Check constellation URL is correct
- Review firewall rules (port 50051 for gRPC)

## License

MIT License

## Support

- Documentation: https://docs.omega.tv/sdk
- Issues: https://github.com/exogenesis-omega/omega-sdk/issues
- Email: support@omega.tv
