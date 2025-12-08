# Exogenesis Omega v2 - Refactored Architecture

## Key Change: Leveraging Existing Omega Crates

This architecture leverages **7 existing crates from crates.io** to eliminate redundant work:

| Crate | Version | Purpose | Replaces |
|-------|---------|---------|----------|
| `omega-core` | 1.0.0 | Core types, 12-tier memory, 7 loops | Custom types |
| `omega-agentdb` | 1.0.0 | SIMD HNSW vector DB, Reflexion, Skills | Custom VectorStore |
| `omega-memory` | 1.0.0 | 12-tier cosmic memory system | Custom AgentMemory |
| `omega-loops` | 1.0.0 | 7 temporal feedback loops | Custom loop logic |
| `omega-runtime` | 1.0.0 | Production orchestration | Custom health checks |
| `omega-persistence` | 1.0.0 | SQLite storage with ACID | Custom persistence |
| `omega-meta-sona` | 1.0.0 | Self-optimizing architecture | - |

---

## Revised Crate Structure

```
exogenesis-omega/
├── Cargo.toml                    # Workspace using omega-* crates
│
├── crates/
│   ├── omega-tv-brain/           # TV-side integration (NEW - thin wrapper)
│   │   └── Uses: omega-agentdb, omega-memory, omega-loops, omega-persistence
│   │
│   ├── omega-tv-sync/            # Delta sync protocol (CUSTOM)
│   │   └── Uses: omega-core types
│   │
│   ├── omega-tv-sdk/             # C FFI for TV manufacturers (CUSTOM)
│   │   └── Wraps: omega-tv-brain
│   │
│   └── omega-constellation/      # Server coordination (CUSTOM)
│       └── Uses: omega-core, RuVector-Postgres
│
├── services/
│   ├── constellation-server/     # Server binary
│   └── federation-worker/        # Pattern aggregation
│
└── deploy/                       # Docker, K8s, Helm
```

---

## Component Mapping

### TV Omega Brain (Using Existing Crates)

```rust
// omega-tv-brain/src/lib.rs

use omega_agentdb::{AgentDB, AgentDBConfig, ReflexionEpisode, Skill, VectorResult};
use omega_memory::{CosmicMemory, Memory, MemoryTier, MemoryContent};
use omega_loops::{LoopEngine, LoopType, CycleInput, CycleOutput};
use omega_persistence::PersistenceManager;
use omega_runtime::RuntimeOrchestrator;
use omega_core::{Intelligence, Architecture, Paradigm};

/// TV Brain integrating all omega-* crates
pub struct OmegaTVBrain {
    /// SIMD-accelerated vector DB (omega-agentdb)
    /// - 13-41x SIMD acceleration
    /// - HNSW indexing
    /// - ReflexionEpisode, CausalEdge, Skill storage
    agentdb: AgentDB,

    /// 12-tier cosmic memory (omega-memory)
    /// - Instant (1ms) to Omega (universe-age)
    /// - Automatic consolidation
    /// - Semantic retrieval
    memory: CosmicMemory,

    /// 7 temporal loops (omega-loops)
    /// - Reflexive (100ms) to Transcendent (10y)
    /// - Multi-scale learning
    loops: LoopEngine,

    /// SQLite persistence (omega-persistence)
    persistence: PersistenceManager,

    /// Production orchestration (omega-runtime)
    runtime: RuntimeOrchestrator,

    /// Sync client for constellation
    sync: SyncClient,

    /// Configuration
    config: TVBrainConfig,
}

pub struct TVBrainConfig {
    pub dimension: usize,           // 384 (MiniLM embedding size)
    pub hnsw_m: usize,              // 32
    pub hnsw_ef: usize,             // 100
    pub max_patterns: usize,        // 10,000
    pub sync_interval_secs: u64,    // 300-900
    pub storage_path: PathBuf,      // /data/omega/
    pub constellation_url: String,
}

impl OmegaTVBrain {
    pub async fn init(config: TVBrainConfig) -> Result<Self> {
        // Initialize AgentDB with 384-dim embeddings
        let agentdb = AgentDB::new(AgentDBConfig {
            dimension: config.dimension,  // 384 for MiniLM
            hnsw_m: config.hnsw_m,
            hnsw_ef: config.hnsw_ef,
            cache_size: config.max_patterns,
        }).await?;

        // Initialize cosmic memory
        let memory = CosmicMemory::new().await?;

        // Initialize temporal loops
        let mut loops = LoopEngine::new();
        loops.initialize().await?;

        // Initialize SQLite persistence
        let persistence = PersistenceManager::new(&config.storage_path).await?;

        // Initialize runtime orchestrator
        let runtime = RuntimeOrchestrator::new().await?;

        // Initialize sync client
        let sync = SyncClient::new(&config.constellation_url)?;

        Ok(Self { agentdb, memory, loops, persistence, runtime, sync, config })
    }

    /// Get recommendations (<15ms)
    pub async fn recommend(&self, context: &ViewContext) -> Vec<Recommendation> {
        // 1. Create embedding from context (mock for now, real ONNX later)
        let query_embedding = self.embed_context(context);

        // 2. Search AgentDB vectors (<1ms with SIMD)
        let results = self.agentdb
            .vector_search(&query_embedding, 50)
            .await
            .unwrap_or_default();

        // 3. Filter by context using memory recall
        let memories = self.memory
            .recall(&Query::semantic(&context.to_string()), &[MemoryTier::Semantic])
            .await
            .unwrap_or_default();

        // 4. Rank and return recommendations
        self.rank_results(results, memories, context)
    }

    /// Record viewing event
    pub async fn observe(&mut self, event: ViewingEvent) {
        // 1. Create embedding
        let embedding = self.embed_event(&event);

        // 2. Store in AgentDB as ReflexionEpisode
        let episode = ReflexionEpisode {
            id: None,
            session_id: event.session_id.clone(),
            task: format!("watched_{}", event.content_id),
            input: serde_json::to_value(&event).unwrap(),
            output: serde_json::json!({"watch_pct": event.watch_percentage}),
            reward: event.watch_percentage as f64,
            success: event.watch_percentage > 0.7,
            critique: String::new(),
            latency_ms: 0,
            tokens: 0,
            timestamp: chrono::Utc::now(),
            embedding: Some(embedding.clone()),
        };
        self.agentdb.reflexion_store(episode).await.ok();

        // 3. Store in cosmic memory (Episodic tier)
        let memory = Memory::new(
            MemoryTier::Episodic,
            MemoryContent::MultiModal {
                text: Some(format!("Watched {} for {}%", event.content_id, event.watch_percentage * 100.0)),
                embedding: embedding.clone(),
                metadata: serde_json::to_value(&event).unwrap(),
            },
            embedding,
            event.watch_percentage as f64, // importance
        );
        self.memory.store(memory).await.ok();

        // 4. Execute Reflexive loop (100ms feedback)
        let input = CycleInput {
            data: std::collections::HashMap::from([
                ("event".to_string(), serde_json::to_value(&event).unwrap()),
            ]),
            context: "Process viewing event".to_string(),
            objectives: vec!["Update recommendations".to_string()],
        };
        self.loops.execute_cycle(LoopType::Reflexive, input).await.ok();
    }

    /// Sync with constellation
    pub async fn sync(&mut self) -> Result<SyncResult> {
        // Get high-quality patterns from AgentDB
        let skills = self.agentdb.skill_list(100).await?;
        let high_quality: Vec<_> = skills.into_iter()
            .filter(|s| s.success_rate >= 0.7)
            .collect();

        // Create delta
        let delta = self.sync.prepare_delta(&high_quality)?;

        // Send to constellation, receive global patterns
        let global = self.sync.sync(delta).await?;

        // Apply global patterns
        for pattern in global.patterns {
            self.agentdb.skill_store(Skill {
                id: None,
                name: pattern.name,
                description: pattern.description,
                embedding: pattern.embedding,
                usage_count: pattern.usage_count,
                success_rate: pattern.success_rate,
                created_at: chrono::Utc::now(),
            }).await.ok();
        }

        Ok(SyncResult {
            patterns_pushed: high_quality.len(),
            patterns_received: global.patterns.len(),
        })
    }

    fn embed_context(&self, context: &ViewContext) -> Vec<f32> {
        // TODO: Real ONNX inference
        // For now, deterministic pseudo-embedding
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&context.to_string(), &mut hasher);
        let seed = std::hash::Hasher::finish(&hasher);
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        (0..self.config.dimension).map(|_| rng.gen::<f32>() * 2.0 - 1.0).collect()
    }
}
```

---

## Memory Tier Mapping for TV

| Omega Memory Tier | TV Recommendation Usage |
|-------------------|------------------------|
| **Instant (1ms)** | Current viewing session buffer |
| **Session (min)** | Recent recommendations shown |
| **Episodic (hrs)** | Today's viewing history |
| **Semantic (days)** | Learned genre preferences |
| **Collective** | Global patterns from constellation |
| **Evolutionary** | Long-term taste evolution |

---

## Temporal Loops for TV

| Loop | Timescale | TV Application |
|------|-----------|----------------|
| **Reflexive** | 100ms | Real-time viewing feedback |
| **Reactive** | 5s | UI interaction response |
| **Adaptive** | 30min | Session-based learning |
| **Deliberative** | 24h | Daily preference update |
| **Evolutionary** | 7d | Weekly pattern consolidation |
| **Transformative** | 1y | Taste profile evolution |
| **Transcendent** | 10y | Long-term preference shifts |

---

## AgentDB Configuration for TV

```rust
// 384-dim for MiniLM embeddings (vs default 4096)
let config = AgentDBConfig {
    dimension: 384,        // MiniLM embedding size
    hnsw_m: 32,            // Graph connectivity
    hnsw_ef: 100,          // Search accuracy
    cache_size: 10_000,    // Max patterns per TV
};
```

**AgentDB Features Used:**
- `vector_store()` / `vector_search()` - SIMD-accelerated similarity
- `reflexion_store()` / `reflexion_retrieve()` - Learning episodes
- `skill_store()` / `skill_list()` - Learned patterns
- `causal_record()` - Action→outcome tracking

---

## Sync Protocol (Custom)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        SYNC FLOW                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  TV (omega-tv-brain)                        Constellation                   │
│  ─────────────────────                      ─────────────                   │
│                                                                              │
│  AgentDB.skill_list(quality≥0.7)                                            │
│           │                                                                  │
│           ▼                                                                  │
│  omega-tv-sync.prepare_delta()                                              │
│           │ ~1KB compressed                                                 │
│           ▼                                                                  │
│  ─────────────────────────────────────────────►  RuVector-Postgres         │
│                                                         │                    │
│                                                  Federated Averaging         │
│                                                         │                    │
│  ◄─────────────────────────────────────────────  ~5KB global patterns      │
│           │                                                                  │
│           ▼                                                                  │
│  AgentDB.skill_store() for each pattern                                     │
│  CosmicMemory.store(Collective tier)                                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Workspace Dependencies

```toml
# Cargo.toml (workspace root)

[workspace]
resolver = "2"
members = [
    "crates/omega-tv-brain",
    "crates/omega-tv-sync",
    "crates/omega-tv-sdk",
    "crates/omega-constellation",
    "services/constellation-server",
    "services/federation-worker",
]

[workspace.dependencies]
# === EXISTING OMEGA CRATES (from crates.io) ===
omega-core = "1.0"
omega-agentdb = "1.0"
omega-memory = "1.0"
omega-loops = "1.0"
omega-runtime = "1.0"
omega-persistence = "1.0"
omega-meta-sona = "1.0"

# === STANDARD DEPENDENCIES ===
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
rand = "0.8"

# === NETWORK ===
axum = "0.7"
reqwest = { version = "0.11", features = ["json"] }
zstd = "0.13"

# === CONSTELLATION ===
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres"] }
```

---

## What We Build vs What We Use

### USE (from crates.io):
| Crate | Key APIs |
|-------|----------|
| `omega-agentdb` | `AgentDB::new()`, `vector_store()`, `vector_search()`, `reflexion_store()`, `skill_store()` |
| `omega-memory` | `CosmicMemory::new()`, `store()`, `recall()`, `MemoryTier::*` |
| `omega-loops` | `LoopEngine::new()`, `execute_cycle()`, `LoopType::*` |
| `omega-runtime` | `RuntimeOrchestrator::new()`, health monitoring |
| `omega-persistence` | `PersistenceManager::new()`, SQLite ACID |
| `omega-core` | `Memory`, `MemoryTier`, `LoopType`, `CycleInput`, `CycleOutput` |

### BUILD (custom):
| Crate | Purpose |
|-------|---------|
| `omega-tv-brain` | Thin integration layer for TV |
| `omega-tv-sync` | Delta protocol (push ~1KB, pull ~5KB) |
| `omega-tv-sdk` | C FFI for TV manufacturers |
| `omega-constellation` | Server with RuVector-Postgres |

---

## Summary: 90% Less Code to Write

| Before | After |
|--------|-------|
| Custom VectorStore (500 lines) | `omega-agentdb` (0 lines) |
| Custom AgentMemory (400 lines) | `omega-memory` (0 lines) |
| Custom LoopEngine (300 lines) | `omega-loops` (0 lines) |
| Custom persistence (200 lines) | `omega-persistence` (0 lines) |
| Custom types (600 lines) | `omega-core` (0 lines) |
| **Total: ~2000 lines** | **Total: ~200 lines (integration only)** |

**Result: Focus on TV-specific integration and sync protocol, not reinventing the wheel.**
