# Omega TV Brain

Thin integration layer for TV intelligence using existing omega-* crates.

## Architecture

This crate integrates 6 existing omega-* crates from crates.io:

- **omega-agentdb** (1.0.0) - SIMD HNSW vector DB with 13-41x acceleration
- **omega-memory** (1.0.0) - 12-tier cosmic memory system
- **omega-loops** (1.0.0) - 7 temporal feedback loops
- **omega-persistence** (1.0.0) - SQLite storage with ACID guarantees
- **omega-runtime** (1.0.0) - Production orchestration and health monitoring
- **omega-core** (1.0.0) - Core types and traits

## Key Features

- **384-dimensional embeddings** (MiniLM model, not default 4096)
- **<15ms recommendation latency** with SIMD-accelerated vector search
- **~1KB delta sync** every 5-15 minutes
- **~5KB global patterns** received from constellation
- **Real-time learning** via Reflexive loop (100ms feedback)

## Configuration

```rust
use omega_tv_brain::{OmegaTVBrain, TVBrainConfig};

// Production configuration
let config = TVBrainConfig::production();

// Development configuration
let config = TVBrainConfig::development();

// Custom configuration
let config = TVBrainConfig {
    dimension: 384,              // MiniLM embeddings
    hnsw_m: 32,                  // Graph connectivity
    hnsw_ef: 100,                // Search accuracy
    max_patterns: 10_000,        // Max patterns per TV
    sync_interval_secs: 600,     // 10 minutes
    storage_path: PathBuf::from("/data/omega"),
    constellation_url: "https://constellation.exogenesis-omega.io".to_string(),
    device_id: "tv-12345".to_string(),
    enable_neural_training: true,
    recommend_timeout_ms: 15,
};
```

## Usage

### Initialize

```rust
let brain = OmegaTVBrain::init(config).await?;
```

### Get Recommendations

```rust
let context = ViewContext {
    user_id: "user123".to_string(),
    device_id: "tv-device-456".to_string(),
    time_of_day: 20,             // 8 PM
    day_of_week: 5,              // Friday
    current_genre: Some("action".to_string()),
    previous_content: vec!["movie-123".to_string()],
    session_duration_mins: 45,
};

let recommendations = brain.recommend(&context).await?;
// Returns Vec<Recommendation> in <15ms
```

### Record Viewing Event

```rust
let event = ViewingEvent {
    event_id: uuid::Uuid::new_v4().to_string(),
    session_id: "session-789".to_string(),
    user_id: "user123".to_string(),
    content_id: "movie-456".to_string(),
    content_type: "movie".to_string(),
    genre: "action".to_string(),
    watch_percentage: 0.95,      // Watched 95%
    engagement_score: 0.88,      // High engagement
    timestamp: chrono::Utc::now(),
    metadata: serde_json::json!({}),
};

brain.observe(event).await?;
// Stores in AgentDB + CosmicMemory, executes Reflexive loop
```

### Sync with Constellation

```rust
let result = brain.sync().await?;
println!(
    "Synced: pushed {} patterns, received {} global patterns",
    result.patterns_pushed,
    result.patterns_received
);
// Delta: ~1KB uploaded, ~5KB downloaded
```

## Memory Tier Mapping

| Omega Memory Tier | TV Recommendation Usage |
|-------------------|------------------------|
| **Instant (1ms)** | Current viewing session buffer |
| **Session (min)** | Recent recommendations shown |
| **Episodic (hrs)** | Today's viewing history |
| **Semantic (days)** | Learned genre preferences |
| **Collective** | Global patterns from constellation |
| **Evolutionary** | Long-term taste evolution |

## Temporal Loops

| Loop | Timescale | TV Application |
|------|-----------|----------------|
| **Reflexive** | 100ms | Real-time viewing feedback |
| **Reactive** | 5s | UI interaction response |
| **Adaptive** | 30min | Session-based learning |
| **Deliberative** | 24h | Daily preference update |
| **Evolutionary** | 7d | Weekly pattern consolidation |
| **Transformative** | 1y | Taste profile evolution |
| **Transcendent** | 10y | Long-term preference shifts |

## Performance

- **Vector search**: <1ms (SIMD accelerated)
- **Memory recall**: <5ms
- **Total recommendation**: <15ms (target)
- **Event observation**: <10ms
- **Sync preparation**: <100ms
- **Full sync cycle**: <2s

## Code Size

This thin integration layer is **~200 lines** vs **~2000 lines** if implemented from scratch:

| Component | Custom Code | Using Omega Crates |
|-----------|-------------|-------------------|
| Vector DB | 500 lines | 0 lines (omega-agentdb) |
| Memory System | 400 lines | 0 lines (omega-memory) |
| Temporal Loops | 300 lines | 0 lines (omega-loops) |
| Persistence | 200 lines | 0 lines (omega-persistence) |
| Core Types | 600 lines | 0 lines (omega-core) |
| **Total** | **~2000 lines** | **~200 lines** |

## Testing

```bash
cargo test -p omega-tv-brain
```

## License

MIT OR Apache-2.0
