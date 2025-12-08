# Exogenesis Omega - System Architecture

## Overview

**Exogenesis Omega** is a distributed viewer intelligence system for TV recommendations at hyperscale:
- **400 million total users**
- **10 million concurrent viewers**
- **Sub-10ms inference latency** (local on each TV)
- **Full privacy** (viewing data never leaves the TV)

## Core Principle: The TV IS the Edge

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         TRADITIONAL vs OMEGA                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  TRADITIONAL EDGE:          EXOGENESIS OMEGA:                                │
│  - 50-300 CDN POPs          - 400,000,000 Edge Nodes                        │
│  - Cloud inference          - TV-local inference                             │
│  - 50-500ms latency         - <10ms latency                                  │
│  - $2-5M/month              - $530K/month                                    │
│  - Data to cloud            - Data stays on TV                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## System Components

### 1. Omega Brain (On Each TV)

The Omega Brain is a ~200MB intelligent agent running on each TV:

```
┌─────────────────────────────────────────────────────────────────┐
│                    OMEGA BRAIN (~200MB)                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  RuVector Embedded (50MB)                                 │   │
│  │  - HNSW index with SIMD (ARM NEON)                        │   │
│  │  - 10,000 vectors per user                                │   │
│  │  - Cosine similarity search                               │   │
│  │  - <1ms search latency                                    │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  ONNX Runtime (100MB)                                     │   │
│  │  - MiniLM 4-bit quantized model                           │   │
│  │  - Text → 384-dim embeddings                              │   │
│  │  - <10ms inference                                        │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  AgentDB Memory (50MB)                                    │   │
│  │  ├── ReflexionMemory - Episode storage                    │   │
│  │  ├── SkillLibrary - Learned patterns                      │   │
│  │  ├── ReasoningBank - Consolidated approaches              │   │
│  │  └── CausalMemoryGraph - Action→outcome tracking          │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Sync Agent (5MB)                                         │   │
│  │  - Delta-only protocol (~1KB push, ~5KB pull)             │   │
│  │  - Interval: 5-15 minutes                                 │   │
│  │  - Offline-capable                                        │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### Omega Brain Rust Structure

```rust
// omega-brain/src/lib.rs

pub struct OmegaBrain {
    /// Local vector database
    pub vectors: RuVector,

    /// ONNX inference engine
    pub onnx: OnnxRuntime,

    /// Memory systems
    pub memory: AgentMemory,

    /// Sync coordinator
    pub sync: SyncAgent,

    /// Configuration
    pub config: BrainConfig,
}

pub struct BrainConfig {
    pub dimensions: usize,           // 384
    pub max_patterns: usize,         // 10,000
    pub sync_interval_secs: u64,     // 300-900 (5-15 min)
    pub model_path: PathBuf,         // /data/omega/model.onnx
    pub storage_path: PathBuf,       // /data/omega/brain.db
}

impl OmegaBrain {
    /// Initialize brain on TV boot
    pub async fn init(config: BrainConfig) -> Result<Self>;

    /// Get recommendations (LOCAL, <15ms)
    pub async fn recommend(&self, context: ViewContext) -> Vec<Recommendation>;

    /// Record viewing event (LOCAL)
    pub async fn observe(&mut self, event: ViewingEvent);

    /// Sync with constellation (NETWORK, every 5-15 min)
    pub async fn sync(&mut self) -> Result<SyncResult>;
}
```

---

### 2. Omega Constellation (Central Coordination)

The Constellation handles ONLY pattern aggregation and distribution - NO inference:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        OMEGA CONSTELLATION                                   │
│                     (100 Shards, 1000 Servers)                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                      LOAD BALANCER (Anycast)                            │ │
│  │                  Geo-routes TVs to nearest shard                        │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                         │
│           ┌────────────────────────┼────────────────────────┐               │
│           │                        │                        │               │
│           ▼                        ▼                        ▼               │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐         │
│  │   SHARD 1       │    │   SHARD 2       │    │   SHARD 100     │         │
│  │  (4M users)     │    │  (4M users)     │    │  (4M users)     │         │
│  │                 │    │                 │    │                 │         │
│  │ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │         │
│  │ │ RuVector-PG │ │    │ │ RuVector-PG │ │    │ │ RuVector-PG │ │         │
│  │ │  Primary    │ │    │ │  Primary    │ │    │ │  Primary    │ │         │
│  │ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │         │
│  │       │         │    │       │         │    │       │         │         │
│  │ ┌─────┴─────┐   │    │ ┌─────┴─────┐   │    │ ┌─────┴─────┐   │         │
│  │ │ Replica 1 │   │    │ │ Replica 1 │   │    │ │ Replica 1 │   │         │
│  │ │ Replica 2 │   │    │ │ Replica 2 │   │    │ │ Replica 2 │   │         │
│  │ └───────────┘   │    │ └───────────┘   │    │ └───────────┘   │         │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘         │
│           │                        │                        │               │
│           └────────────────────────┼────────────────────────┘               │
│                                    │                                         │
│                                    ▼                                         │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                    FEDERATION COORDINATOR                               │ │
│  │  - Aggregates patterns across shards                                    │ │
│  │  - Quality-weighted federated averaging                                 │ │
│  │  - Trend detection                                                      │ │
│  │  - New content embedding distribution                                   │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### RuVector-Postgres Cluster

```yaml
# docker-compose.constellation.yml

services:
  ruvector-shard-1-primary:
    image: ruvnet/ruvector-postgres:latest
    environment:
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      RUVECTOR_DIMENSIONS: 384
      RUVECTOR_METRIC: cosine
      RUVECTOR_HNSW_M: 32
      RUVECTOR_HNSW_EF_CONSTRUCTION: 200
      RUVECTOR_GNN_ENABLED: "true"
      RUVECTOR_GNN_LEARNING_RATE: 0.001
      RUVECTOR_RAFT_ENABLED: "true"
      RUVECTOR_RAFT_NODE_ID: 1
      RUVECTOR_RAFT_PEERS: ruvector-shard-1-replica1:5433,ruvector-shard-1-replica2:5434
      RUVECTOR_COMPRESSION: adaptive  # f32→f16→PQ8→PQ4 based on query frequency
    volumes:
      - shard1_data:/var/lib/postgresql/data
    deploy:
      resources:
        limits:
          memory: 64G
          cpus: '16'
```

---

### 3. Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           DATA FLOW                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  USER WATCHES CONTENT                                                        │
│           │                                                                  │
│           ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ 1. LOCAL OBSERVATION (Omega Brain)                                   │    │
│  │    event = { media: "Thriller X", watchTime: 95%, time: "evening" }  │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│           │                                                                  │
│           ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ 2. LOCAL EMBEDDING (ONNX, <10ms)                                     │    │
│  │    vector = onnx.embed(event) → Float32[384]                         │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│           │                                                                  │
│           ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ 3. LOCAL STORAGE (RuVector, <1ms)                                    │    │
│  │    brain.vectors.insert(event_id, vector, metadata)                  │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│           │                                                                  │
│           ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ 4. LOCAL RECOMMENDATION (when user asks)                             │    │
│  │    context = current_time + mood + recent_watches                    │    │
│  │    query_vec = onnx.embed(context)                                   │    │
│  │    similar = brain.vectors.search(query_vec, k=50)                   │    │
│  │    recommendations = rank_by_success_rate(similar)                   │    │
│  │    TOTAL: <15ms                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│           │                                                                  │
│           │ (Every 5-15 minutes)                                            │
│           ▼                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ 5. DELTA SYNC TO CONSTELLATION                                       │    │
│  │    - Push: High-quality patterns (~100 vectors, ~1KB compressed)     │    │
│  │    - Pull: Global trends + new content embeddings (~5KB)             │    │
│  │    - NO raw viewing data sent (only aggregated patterns)             │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 4. Technology Stack (All Rust)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        RUST TECHNOLOGY STACK                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  TV OMEGA BRAIN                           CONSTELLATION                      │
│  ─────────────────                        ─────────────                      │
│                                                                              │
│  omega-brain (Rust)                       constellation (Rust)               │
│  ├── ruvector (embedded)                  ├── ruvector-postgres (Docker)     │
│  │   └── hnsw-rs                          │   ├── tokio (async runtime)      │
│  │   └── simd (ARM NEON)                  │   ├── raft-rs (consensus)        │
│  ├── onnxruntime-rs                       │   └── gnn (self-learning)        │
│  │   └── 4-bit quantized                  ├── axum (HTTP API)                │
│  ├── agentdb-core                         ├── tonic (gRPC)                   │
│  │   ├── reflexion-memory                 └── federation-coordinator         │
│  │   ├── skill-library                                                       │
│  │   └── reasoning-bank                   SHARED CRATES                      │
│  └── sync-agent                           ──────────────                     │
│      ├── quinn (QUIC)                     omega-protocol                     │
│      └── delta-sync                       ├── Delta sync messages            │
│                                           ├── Pattern serialization          │
│                                           └── Compression (zstd)             │
│                                                                              │
│  OPTIONAL (TypeScript)                                                       │
│  ─────────────────────                                                       │
│  admin-dashboard                                                             │
│  ├── Next.js                                                                 │
│  ├── React                                                                   │
│  └── TailwindCSS                                                             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 5. Crate Structure

```
exogenesis-omega/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── omega-brain/              # TV-side intelligence
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Main brain logic
│   │       ├── vectors.rs        # RuVector wrapper
│   │       ├── inference.rs      # ONNX runtime
│   │       ├── memory.rs         # AgentDB memory
│   │       └── recommend.rs      # Recommendation engine
│   │
│   ├── omega-sync/               # Sync protocol
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── delta.rs          # Delta encoding
│   │       ├── protocol.rs       # Wire protocol
│   │       └── compression.rs    # zstd compression
│   │
│   ├── omega-constellation/      # Server-side coordination
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── shard.rs          # Shard management
│   │       ├── federation.rs     # Pattern federation
│   │       ├── api.rs            # HTTP/gRPC endpoints
│   │       └── storage.rs        # RuVector-Postgres client
│   │
│   ├── omega-protocol/           # Shared types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── messages.rs       # Protocol messages
│   │       ├── patterns.rs       # Pattern types
│   │       └── events.rs         # Viewing events
│   │
│   └── omega-sdk/                # TV manufacturer SDK
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── ffi.rs            # C FFI for TV platforms
│           └── wasm.rs           # WebAssembly target
│
├── services/
│   ├── constellation-server/     # Main server binary
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   │
│   └── federation-worker/        # Pattern aggregation worker
│       ├── Cargo.toml
│       └── src/main.rs
│
├── tools/
│   ├── omega-cli/                # Management CLI
│   └── brain-simulator/          # Testing tool
│
└── admin/                        # TypeScript dashboard (optional)
    ├── package.json
    └── src/
```

---

### 6. API Specifications

#### 6.1 TV ↔ Constellation Sync API (gRPC)

```protobuf
// omega-protocol/proto/sync.proto

syntax = "proto3";
package omega.sync;

service OmegaSync {
  // Push local patterns to constellation
  rpc PushPatterns(PushRequest) returns (PushResponse);

  // Pull global patterns from constellation
  rpc PullPatterns(PullRequest) returns (PullResponse);

  // Bidirectional stream for real-time sync
  rpc StreamSync(stream SyncMessage) returns (stream SyncMessage);
}

message PushRequest {
  string device_id = 1;
  bytes patterns_delta = 2;          // Compressed delta (~1KB)
  uint64 local_version = 3;
  repeated PatternQuality qualities = 4;
}

message PatternQuality {
  string pattern_id = 1;
  float success_rate = 2;            // 0.0-1.0
  uint32 sample_count = 3;
}

message PullRequest {
  string device_id = 1;
  uint64 last_sync_version = 2;
  repeated string content_ids = 3;   // New content to get embeddings for
}

message PullResponse {
  bytes global_patterns = 1;         // Compressed (~5KB)
  uint64 global_version = 2;
  repeated ContentEmbedding new_content = 3;
  repeated TrendSignal trends = 4;
}

message ContentEmbedding {
  string content_id = 1;
  bytes embedding = 2;               // Float16[384] = 768 bytes
  ContentMetadata metadata = 3;
}

message TrendSignal {
  string content_id = 1;
  float trending_score = 2;
  string region = 3;
}
```

#### 6.2 Constellation Management API (REST)

```yaml
# openapi.yaml

openapi: 3.0.3
info:
  title: Omega Constellation API
  version: 1.0.0

paths:
  /api/v1/health:
    get:
      summary: Health check
      responses:
        200:
          description: Healthy

  /api/v1/shards:
    get:
      summary: List all shards
      responses:
        200:
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/Shard'

  /api/v1/shards/{shard_id}/stats:
    get:
      summary: Get shard statistics
      parameters:
        - name: shard_id
          in: path
          required: true
          schema:
            type: integer
      responses:
        200:
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ShardStats'

  /api/v1/content:
    post:
      summary: Add new content embedding
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/NewContent'
      responses:
        201:
          description: Content added

  /api/v1/federation/trigger:
    post:
      summary: Trigger federation round
      responses:
        202:
          description: Federation started

components:
  schemas:
    Shard:
      type: object
      properties:
        id: { type: integer }
        region: { type: string }
        devices: { type: integer }
        status: { type: string, enum: [healthy, degraded, offline] }

    ShardStats:
      type: object
      properties:
        shard_id: { type: integer }
        total_devices: { type: integer }
        active_devices: { type: integer }
        patterns_stored: { type: integer }
        sync_requests_per_min: { type: integer }
        avg_sync_latency_ms: { type: number }

    NewContent:
      type: object
      required: [content_id, title, embedding]
      properties:
        content_id: { type: string }
        title: { type: string }
        embedding: { type: array, items: { type: number } }
        metadata:
          type: object
          properties:
            genre: { type: string }
            year: { type: integer }
            duration_min: { type: integer }
```

---

### 7. Deployment Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        DEPLOYMENT TOPOLOGY                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  REGION: US-EAST (Primary)              REGION: US-WEST                     │
│  ─────────────────────────              ────────────────                    │
│                                                                              │
│  ┌─────────────────────────┐            ┌─────────────────────────┐         │
│  │ Kubernetes Cluster      │            │ Kubernetes Cluster      │         │
│  │                         │            │                         │         │
│  │ Shards 1-25            │◄──────────►│ Shards 26-50           │         │
│  │ (100M users)           │   Cross-    │ (100M users)           │         │
│  │                         │   Region    │                         │         │
│  │ 250 pods               │   Sync      │ 250 pods               │         │
│  └─────────────────────────┘            └─────────────────────────┘         │
│                                                                              │
│  REGION: EU-WEST                        REGION: APAC                        │
│  ─────────────────                      ────────────                        │
│                                                                              │
│  ┌─────────────────────────┐            ┌─────────────────────────┐         │
│  │ Kubernetes Cluster      │            │ Kubernetes Cluster      │         │
│  │                         │            │                         │         │
│  │ Shards 51-75           │◄──────────►│ Shards 76-100          │         │
│  │ (100M users)           │            │ (100M users)           │         │
│  │                         │            │                         │         │
│  │ 250 pods               │            │ 250 pods               │         │
│  └─────────────────────────┘            └─────────────────────────┘         │
│                                                                              │
│  GLOBAL SERVICES                                                             │
│  ───────────────                                                             │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Cloudflare Anycast (Load Balancing + DDoS Protection)               │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Federation Coordinator (Multi-region, Active-Active)                │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 8. Cost Analysis

| Component | Specification | Monthly Cost |
|-----------|---------------|--------------|
| **Compute** | 1000 pods × 16 vCPU, 64GB | $320,000 |
| **Storage** | 100 shards × 2TB NVMe | $50,000 |
| **Network** | ~720GB/hour egress | $100,000 |
| **Load Balancer** | Cloudflare Enterprise | $30,000 |
| **Monitoring** | Datadog/Prometheus | $30,000 |
| **TOTAL** | | **$530,000/month** |

**vs Traditional Cloud Inference: $2-5M/month (75-90% savings)**

---

### 9. Security Model

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        SECURITY ARCHITECTURE                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  DATA CLASSIFICATION                                                         │
│  ───────────────────                                                         │
│                                                                              │
│  ┌─────────────────────────┐    ┌─────────────────────────┐                │
│  │ STAYS ON TV (Private)   │    │ SENT TO CONSTELLATION   │                │
│  │ ─────────────────────── │    │ (Aggregated Only)       │                │
│  │                         │    │ ─────────────────────── │                │
│  │ • Raw viewing history   │    │ • Pattern embeddings    │                │
│  │ • Watch timestamps      │    │ • Success rates         │                │
│  │ • User preferences      │    │ • Sample counts         │                │
│  │ • Pause/seek behavior   │    │ • Device health         │                │
│  │ • Personal patterns     │    │ (NO personal data)      │                │
│  └─────────────────────────┘    └─────────────────────────┘                │
│                                                                              │
│  TRANSPORT SECURITY                                                          │
│  ──────────────────                                                          │
│                                                                              │
│  • TLS 1.3 for all connections                                              │
│  • QUIC for sync (0-RTT resumption)                                         │
│  • mTLS between constellation nodes                                         │
│  • Device attestation (hardware root of trust)                              │
│                                                                              │
│  ACCESS CONTROL                                                              │
│  ──────────────                                                              │
│                                                                              │
│  • Device certificates (per TV)                                             │
│  • Shard-level isolation                                                    │
│  • Admin API: OAuth2 + RBAC                                                 │
│  • Rate limiting: 1 sync/minute per device                                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 10. Monitoring & Observability

```rust
// Metrics exported by each component

// Omega Brain (TV)
gauge!("omega_brain_patterns_stored").set(count);
histogram!("omega_brain_inference_ms").record(latency);
counter!("omega_brain_recommendations_served").increment(1);
gauge!("omega_brain_memory_mb").set(usage);

// Constellation
gauge!("constellation_devices_active").set(count);
histogram!("constellation_sync_latency_ms").record(latency);
counter!("constellation_patterns_received").increment(batch_size);
gauge!("constellation_shard_health", "shard" => shard_id).set(health);
```

**Dashboards:**
- Real-time sync rates across regions
- Pattern quality distribution
- Device connectivity health
- Federation convergence metrics
- Cost per recommendation served

---

## Summary

| Aspect | Design Decision |
|--------|-----------------|
| **Edge Computing** | 400M TV Omega Brains |
| **Inference Location** | 100% local on TV |
| **Central Role** | Pattern sync only, no inference |
| **Language** | Rust (entire stack) |
| **Vector DB (TV)** | RuVector embedded |
| **Vector DB (Cloud)** | RuVector-Postgres |
| **Sync Protocol** | Delta-only, QUIC |
| **Privacy** | Viewing data never leaves TV |
| **Cost** | $530K/mo (90% savings) |
| **Latency** | <15ms (local) |
