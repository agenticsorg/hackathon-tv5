# SPARC Specification — Part 3 of 4

## Media Gateway: Unified Cross-Platform TV Discovery Engine

**Document Version:** 1.0.0
**SPARC Phase:** Specification
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents — Part 3

11. [Ruvector Simulation and Storage Responsibilities](#11-ruvector-simulation-and-storage-responsibilities)
12. [PubNub Real-Time Sync Behavior](#12-pubnub-real-time-sync-behavior)
13. [Device Interactions](#13-device-interactions)
14. [CLI Behavior Specifications](#14-cli-behavior-specifications)

---

## 11. Ruvector Simulation and Storage Responsibilities

### 11.1 Ruvector Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    RUVECTOR ARCHITECTURE                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                     RUVECTOR CORE                                  │  │
│  │                                                                    │  │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐   │  │
│  │  │  Vector Store   │  │ Hypergraph DB   │  │  GNN Layer      │   │  │
│  │  │  ─────────────  │  │  ─────────────  │  │  ─────────────  │   │  │
│  │  │  • 768-dim      │  │  • Multi-edge   │  │  • GraphSAGE    │   │  │
│  │  │    embeddings   │  │    relations    │  │  • 8-head attn  │   │  │
│  │  │  • HNSW index   │  │  • Platform     │  │  • GNN-Attention│   │  │
│  │  │  • 61μs p50     │  │    hyperedges   │  │  • +12.4% recall│   │  │
│  │  │  • 8.2x faster  │  │  • Genre graph  │  │  • 3.8ms fwd    │   │  │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘   │  │
│  │           │                   │                    │              │  │
│  │           └───────────────────┼────────────────────┘              │  │
│  │                               │                                   │  │
│  │                               ▼                                   │  │
│  │  ┌─────────────────────────────────────────────────────────────┐  │  │
│  │  │                   UNIFIED QUERY ENGINE                       │  │  │
│  │  │  • Hybrid search (vector + graph + text)                    │  │  │
│  │  │  • SONA-aware ranking                                       │  │  │
│  │  │  • Result fusion and deduplication                          │  │  │
│  │  └─────────────────────────────────────────────────────────────┘  │  │
│  │                                                                    │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  STORAGE BACKENDS:                                                      │
│  ─────────────────                                                      │
│  Production:  PostgreSQL + pg_vector + dedicated Valkey cluster        │
│  Development: SQLite + in-memory HNSW                                  │
│  Testing:     Mock backends with deterministic responses               │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 11.2 Vector Storage Specifications

#### 11.2.1 Embedding Configuration

```yaml
embeddings:
  model: "text-embedding-3-small"  # OpenAI
  dimensions: 768
  normalization: true  # L2 normalize for cosine similarity

  content_embedding:
    input_template: "{title} | {description} | Genres: {genres} | Year: {year}"
    max_tokens: 8191
    cache_ttl: 7 days

  query_embedding:
    input_template: "{query}"
    max_tokens: 512
    cache_ttl: 5 minutes

index:
  type: HNSW
  params:
    M: 16                # Number of bi-directional links
    efConstruction: 200  # Index-time accuracy/speed tradeoff
    efSearch: 100        # Query-time accuracy/speed tradeoff

  performance:
    p50_latency: 61μs
    p95_latency: 150μs
    p99_latency: 500μs
    throughput: 16,000 QPS
```

#### 11.2.2 Vector Operations

```rust
/// Vector search interface
pub trait VectorStore: Send + Sync {
    /// Insert or update an embedding
    async fn upsert(
        &self,
        entity_id: &str,
        embedding: &[f32],
        metadata: &Metadata,
    ) -> Result<(), VectorError>;

    /// Find k nearest neighbors
    async fn search(
        &self,
        query_embedding: &[f32],
        k: usize,
        filter: Option<&Filter>,
    ) -> Result<Vec<SearchResult>, VectorError>;

    /// Batch upsert for ingestion
    async fn batch_upsert(
        &self,
        items: Vec<(String, Vec<f32>, Metadata)>,
    ) -> Result<BatchResult, VectorError>;

    /// Delete by entity ID
    async fn delete(&self, entity_id: &str) -> Result<(), VectorError>;
}

pub struct SearchResult {
    pub entity_id: String,
    pub score: f32,        // Cosine similarity (0-1)
    pub metadata: Metadata,
}

pub struct Filter {
    pub content_type: Option<ContentType>,
    pub genres: Option<Vec<String>>,
    pub release_year_range: Option<(u16, u16)>,
    pub platforms: Option<Vec<String>>,
    pub region: Option<String>,
}
```

### 11.3 Hypergraph Storage

#### 11.3.1 Graph Schema

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    HYPERGRAPH SCHEMA                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  NODES:                                                                  │
│  ──────                                                                 │
│  Content     { entity_id, title, type, release_year, ... }             │
│  Person      { person_id, name, role, ... }                            │
│  Genre       { genre_id, name, parent_id }                             │
│  Platform    { platform_id, name, region }                             │
│  User        { user_id, preferences_vector }                           │
│                                                                          │
│  EDGES (Binary):                                                        │
│  ───────────────                                                        │
│  ACTED_IN    (Person) ──────────────▶ (Content)                        │
│  DIRECTED    (Person) ──────────────▶ (Content)                        │
│  SIMILAR_TO  (Content) ─────────────▶ (Content)                        │
│  SEQUEL_OF   (Content) ─────────────▶ (Content)                        │
│  WATCHED     (User) ────────────────▶ (Content)                        │
│  WATCHLISTED (User) ────────────────▶ (Content)                        │
│                                                                          │
│  HYPEREDGES (N-ary):                                                    │
│  ───────────────────                                                    │
│  AVAILABLE_ON: (Content, Platform, Region, AvailabilityType, Price)   │
│  BELONGS_TO:   (Content, Genre, Confidence)                            │
│  CAST:         (Content, [Person], [Role])                             │
│                                                                          │
│  HYPEREDGE EXAMPLE - "Available On":                                    │
│  ───────────────────────────────────                                    │
│  {                                                                       │
│    hyperedge_type: "AVAILABLE_ON",                                      │
│    nodes: [                                                              │
│      { type: "Content", id: "eidr:10.5240/..." },                       │
│      { type: "Platform", id: "netflix" },                               │
│      { type: "Region", id: "USA" }                                      │
│    ],                                                                    │
│    properties: {                                                         │
│      availability_type: "subscription",                                 │
│      quality: ["4K", "HDR"],                                            │
│      added_at: "2025-01-15",                                            │
│      expires_at: null                                                   │
│    }                                                                     │
│  }                                                                       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 11.3.2 Graph Query Interface

```rust
/// Hypergraph query interface
pub trait HypergraphStore: Send + Sync {
    /// Traverse graph from starting node
    async fn traverse(
        &self,
        start: &NodeId,
        edge_types: &[EdgeType],
        max_depth: usize,
        limit: usize,
    ) -> Result<Vec<PathResult>, GraphError>;

    /// Find shortest path between nodes
    async fn shortest_path(
        &self,
        from: &NodeId,
        to: &NodeId,
        edge_types: &[EdgeType],
    ) -> Result<Option<Path>, GraphError>;

    /// Get all content available on platforms
    async fn content_by_availability(
        &self,
        platforms: &[String],
        region: &str,
        filter: Option<&Filter>,
    ) -> Result<Vec<Content>, GraphError>;

    /// Find similar content via graph structure
    async fn graph_similar(
        &self,
        entity_id: &str,
        similarity_types: &[SimilarityType],
        limit: usize,
    ) -> Result<Vec<SimilarResult>, GraphError>;
}

pub enum SimilarityType {
    SharedGenres,
    SharedCast,
    SameDirector,
    SequelPrequel,
    Franchise,
    UserCoWatched,
    VectorSimilar,
}
```

### 11.4 Simulation Modes

#### 11.4.1 Mode Configuration

```yaml
simulation_modes:
  development:
    description: "Fast iteration with synthetic data"
    vector_backend: "memory"
    graph_backend: "sqlite"
    data_source: "fixtures/dev-catalog.json"
    features:
      - mock_embeddings  # Skip OpenAI calls
      - synthetic_availability
      - fast_refresh (1 minute)
    performance:
      max_content: 1000
      response_latency: "real"  # No artificial delays

  staging:
    description: "Production-like with subset of real data"
    vector_backend: "postgres+pgvector"
    graph_backend: "postgres"
    data_source: "aggregator_apis"
    features:
      - real_embeddings
      - real_availability
      - rate_limited_apis
    performance:
      max_content: 50000
      api_rate_limit: 0.5x  # Half of production limits

  production:
    description: "Full production deployment"
    vector_backend: "postgres+pgvector+valkey"
    graph_backend: "postgres"
    data_source: "aggregator_apis"
    features:
      - real_embeddings
      - real_availability
      - multi_key_rotation
      - circuit_breakers
    performance:
      max_content: unlimited
      api_rate_limit: 1.0x

  testing:
    description: "Deterministic responses for CI/CD"
    vector_backend: "mock"
    graph_backend: "mock"
    data_source: "fixtures/test-catalog.json"
    features:
      - deterministic_responses
      - fixed_timestamps
      - no_external_calls
    performance:
      max_content: 100
      response_latency: "instant"
```

#### 11.4.2 Mock Data Generation

```rust
/// Generate synthetic content for testing
pub fn generate_mock_catalog(count: usize, seed: u64) -> Vec<MockContent> {
    let mut rng = StdRng::seed_from_u64(seed);

    (0..count)
        .map(|i| MockContent {
            entity_id: format!("mock:content:{:06}", i),
            title: TITLE_TEMPLATES[i % TITLE_TEMPLATES.len()].to_string(),
            content_type: if rng.gen_bool(0.7) { Movie } else { Series },
            release_year: rng.gen_range(1980..2025),
            genres: random_genres(&mut rng, 1..4),
            rating: rng.gen_range(5.0..9.5),
            embedding: random_embedding(&mut rng, 768),
            availability: random_availability(&mut rng, PLATFORMS),
        })
        .collect()
}

/// Deterministic embedding for testing
pub fn mock_embedding(text: &str) -> Vec<f32> {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash = hasher.finalize();

    // Convert hash to 768 floats (deterministic)
    (0..768)
        .map(|i| {
            let byte = hash[i % 32] as f32;
            (byte / 255.0) * 2.0 - 1.0  // Normalize to [-1, 1]
        })
        .collect()
}
```

### 11.5 Storage Responsibilities

#### 11.5.1 Data Persistence Matrix

| Data Type | Primary Storage | Cache | Retention | Backup |
|-----------|----------------|-------|-----------|--------|
| Content metadata | PostgreSQL | Valkey (24h) | Permanent | Daily |
| Embeddings | pg_vector | Valkey (7d) | Permanent | Daily |
| Graph edges | PostgreSQL | Valkey (1h) | Permanent | Daily |
| User preferences | PostgreSQL + device | Valkey (1h) | 7 years | Real-time |
| Watch history | PostgreSQL | Valkey (24h) | 90 days (VPPA) | Daily |
| Search analytics | BigQuery | None | 2 years | Continuous |
| Session state | Valkey | N/A | 24 hours | None |

#### 11.5.2 Cache Strategy

```yaml
cache:
  layers:
    L1:  # Application memory
      type: "lru"
      size: "256MB"
      ttl: "5 minutes"
      targets:
        - parsed_intents
        - frequent_embeddings

    L2:  # Valkey cluster
      type: "valkey"
      size: "16GB"
      targets:
        - content_metadata (ttl: 24h)
        - embeddings (ttl: 7d)
        - availability (ttl: 1h)
        - user_sessions (ttl: 24h)
        - search_results (ttl: 5m)

    L3:  # CDN edge
      type: "cloud_cdn"
      targets:
        - images (ttl: 30d)
        - static_assets (ttl: 365d)

  invalidation:
    strategies:
      - ttl_expiry
      - event_driven (availability.changed, metadata.updated)
      - manual_purge (admin action)

  performance:
    hit_rate_target: 80%+
    miss_penalty: <100ms
    warm_up_on_deploy: true
```

### 11.6 SONA Integration

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    SONA INTEGRATION PIPELINE                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  INPUT                           SONA ENGINE                 OUTPUT     │
│  ─────                           ───────────                 ──────     │
│                                                                          │
│  ┌──────────────┐               ┌──────────────┐                        │
│  │ User Query   │──────────────▶│ Intent       │                        │
│  │              │               │ Parser       │                        │
│  └──────────────┘               └──────┬───────┘                        │
│                                        │                                 │
│  ┌──────────────┐               ┌──────▼───────┐                        │
│  │ User Profile │──────────────▶│ Two-Tier     │                        │
│  │ (LoRA, 10KB) │               │ LoRA         │                        │
│  └──────────────┘               │ Adaptation   │                        │
│                                 └──────┬───────┘                        │
│  ┌──────────────┐                      │                                 │
│  │ Context      │──────────────┐      │                                 │
│  │ (time, device│              │      │                                 │
│  │  mood)       │              ▼      ▼                                 │
│  └──────────────┘        ┌─────────────────┐      ┌──────────────┐     │
│                          │ 39 Attention    │─────▶│ Personalized │     │
│  ┌──────────────┐        │ Mechanisms      │      │ Rankings     │     │
│  │ Ruvector     │───────▶│                 │      │              │     │
│  │ Results      │        │ - Self-attn     │      │ [Content 1]  │     │
│  └──────────────┘        │ - Cross-attn    │      │ [Content 2]  │     │
│                          │ - Graph-attn    │      │ [Content 3]  │     │
│                          │ - Temporal-attn │      │ ...          │     │
│                          └─────────────────┘      └──────────────┘     │
│                                                                          │
│  PERFORMANCE CHARACTERISTICS:                                           │
│  ─────────────────────────────                                         │
│  • LoRA adapter size: 10KB per user                                    │
│  • Inference latency: <5ms                                             │
│  • Context window: 2048 tokens                                         │
│  • Precision@10: 0.31                                                  │
│  • NDCG@10: 0.63                                                       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 12. PubNub Real-Time Sync Behavior

### 12.1 Channel Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    PUBNUB CHANNEL ARCHITECTURE                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  CHANNEL HIERARCHY:                                                     │
│  ─────────────────                                                      │
│                                                                          │
│  user.{userId}                                                          │
│  ├── .sync           # Watchlist, preferences, watch progress           │
│  ├── .devices        # Device presence, heartbeat, control              │
│  └── .notifications  # Alerts, recommendations, expiring content        │
│                                                                          │
│  global                                                                 │
│  ├── .trending       # Top 100 content (hourly updates)                │
│  └── .announcements  # System-wide messages                             │
│                                                                          │
│  region.{regionCode}                                                    │
│  └── .updates        # Regional availability changes                    │
│                                                                          │
│  platform.{platformId}                                                  │
│  └── .catalog        # Per-platform catalog updates                     │
│                                                                          │
│  CHANNEL ACCESS PATTERNS:                                               │
│  ────────────────────────                                               │
│  • User subscribes to: user.{userId}.*, global.trending                │
│  • Device subscribes to: user.{userId}.devices                         │
│  • Admin subscribes to: global.*, region.*, platform.*                 │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 12.2 Message Types and Payloads

#### 12.2.1 Sync Messages (CRDT-based)

```typescript
// Watch Progress Update (LWW-Register CRDT)
interface WatchProgressMessage {
  type: "watch_progress";
  payload: {
    entity_id: string;
    progress_seconds: number;
    total_seconds: number;
    completed: boolean;
    timestamp: HybridLogicalClock;
    device_id: string;
  };
}

// Watchlist Update (OR-Set CRDT)
interface WatchlistMessage {
  type: "watchlist_update";
  payload: {
    operation: "add" | "remove";
    entity_id: string;
    tag: string;  // Unique UUID for add operation
    timestamp: HybridLogicalClock;
    device_id: string;
    metadata?: {
      added_reason?: string;  // "search", "recommendation", "manual"
      priority?: number;
    };
  };
}

// Preference Update (LWW-Map CRDT)
interface PreferenceMessage {
  type: "preference_update";
  payload: {
    key: string;  // "favorite_genres", "preferred_platforms", etc.
    value: any;
    timestamp: HybridLogicalClock;
    device_id: string;
  };
}
```

#### 12.2.2 Device Messages

```typescript
// Device Heartbeat
interface DeviceHeartbeatMessage {
  type: "device_heartbeat";
  payload: {
    device_id: string;
    device_type: "phone" | "tablet" | "tv" | "web" | "cli";
    status: "idle" | "browsing" | "watching";
    current_content?: {
      entity_id: string;
      title: string;
      progress_percent: number;
    };
    capabilities: {
      supports_4k: boolean;
      supports_hdr: boolean;
      supports_dolby_atmos: boolean;
    };
    app_version: string;
  };
}

// Remote Control Command
interface RemoteControlMessage {
  type: "remote_control";
  payload: {
    command: "play" | "pause" | "seek" | "stop" | "cast";
    target_device_id: string;
    params?: {
      seek_to_seconds?: number;
      content_id?: string;
      platform?: string;
    };
    request_id: string;
    source_device_id: string;
  };
}

// Remote Control Acknowledgment
interface RemoteControlAckMessage {
  type: "remote_control_ack";
  payload: {
    request_id: string;
    success: boolean;
    error?: string;
    device_id: string;
  };
}
```

#### 12.2.3 Notification Messages

```typescript
// Content Expiring Alert
interface ExpiringContentMessage {
  type: "content_expiring";
  payload: {
    entity_id: string;
    title: string;
    platform: string;
    expires_at: string;  // ISO 8601
    days_remaining: number;
    in_watchlist: boolean;
    deep_link: string;
  };
}

// New Recommendation
interface RecommendationMessage {
  type: "new_recommendation";
  payload: {
    entity_id: string;
    title: string;
    reason: string;  // "Because you watched X", "New release in your favorite genre"
    confidence: number;
    poster_url: string;
    platforms: string[];
  };
}
```

### 12.3 CRDT Specifications

#### 12.3.1 Hybrid Logical Clock (HLC)

```typescript
interface HybridLogicalClock {
  physical_time: number;  // 48-bit Unix timestamp (ms)
  logical_counter: number;  // 16-bit counter
}

function compareHLC(a: HybridLogicalClock, b: HybridLogicalClock): number {
  if (a.physical_time !== b.physical_time) {
    return a.physical_time - b.physical_time;
  }
  return a.logical_counter - b.logical_counter;
}

function incrementHLC(current: HybridLogicalClock, wallClock: number): HybridLogicalClock {
  const physical = Math.max(current.physical_time, wallClock);
  const logical = physical === current.physical_time
    ? current.logical_counter + 1
    : 0;
  return { physical_time: physical, logical_counter: logical };
}
```

#### 12.3.2 LWW-Register (Last-Writer-Wins)

```typescript
interface LWWRegister<T> {
  value: T;
  timestamp: HybridLogicalClock;
  device_id: string;
}

function mergeLWW<T>(
  local: LWWRegister<T>,
  remote: LWWRegister<T>
): LWWRegister<T> {
  const cmp = compareHLC(local.timestamp, remote.timestamp);

  if (cmp > 0) {
    return local;  // Local is newer
  } else if (cmp < 0) {
    return remote;  // Remote is newer
  } else {
    // Tie-breaker: lexicographic device_id comparison
    return local.device_id > remote.device_id ? local : remote;
  }
}

// Usage: Watch Progress
type WatchProgress = LWWRegister<{
  entity_id: string;
  progress_seconds: number;
  completed: boolean;
}>;
```

#### 12.3.3 OR-Set (Observed-Remove Set)

```typescript
interface ORSetEntry<T> {
  value: T;
  tag: string;  // Unique UUID for this add operation
  timestamp: HybridLogicalClock;
}

interface ORSet<T> {
  added: Map<string, ORSetEntry<T>>;    // tag -> entry
  removed: Set<string>;                  // removed tags
}

function addToORSet<T>(
  set: ORSet<T>,
  value: T,
  clock: HybridLogicalClock
): ORSet<T> {
  const tag = crypto.randomUUID();
  return {
    added: new Map([...set.added, [tag, { value, tag, timestamp: clock }]]),
    removed: set.removed
  };
}

function removeFromORSet<T>(
  set: ORSet<T>,
  value: T
): ORSet<T> {
  const tagsToRemove = [...set.added.entries()]
    .filter(([_, entry]) => deepEqual(entry.value, value))
    .map(([tag, _]) => tag);

  return {
    added: set.added,
    removed: new Set([...set.removed, ...tagsToRemove])
  };
}

function mergeORSet<T>(local: ORSet<T>, remote: ORSet<T>): ORSet<T> {
  // Union of all adds
  const mergedAdded = new Map([...local.added, ...remote.added]);

  // Union of all removes
  const mergedRemoved = new Set([...local.removed, ...remote.removed]);

  // Remove entries whose tags are in the removed set
  const finalAdded = new Map(
    [...mergedAdded.entries()].filter(([tag, _]) => !mergedRemoved.has(tag))
  );

  return { added: finalAdded, removed: mergedRemoved };
}

function getORSetValues<T>(set: ORSet<T>): T[] {
  return [...set.added.values()]
    .filter(entry => !set.removed.has(entry.tag))
    .map(entry => entry.value);
}

// Usage: Watchlist
type Watchlist = ORSet<{
  entity_id: string;
  added_at: string;
  priority: number;
}>;
```

### 12.4 Presence Management

```yaml
presence:
  heartbeat_interval: 30 seconds
  timeout: 60 seconds

  device_states:
    idle:
      description: "App open, not actively browsing"
      transition_to_offline: 5 minutes inactivity

    browsing:
      description: "Actively searching/browsing content"
      transition_to_idle: 2 minutes inactivity

    watching:
      description: "Content playback in progress"
      keep_alive: true
      sync_interval: 30 seconds (progress updates)

    offline:
      description: "App closed or network disconnected"
      grace_period: 60 seconds before presence leave

  events:
    join:
      - Update device registry
      - Sync latest state from server
      - Subscribe to user channels

    leave:
      - Persist unsync'd state to server
      - Unsubscribe from channels
      - Update last_seen timestamp

    timeout:
      - Mark device as offline
      - Emit device_offline event to other devices
```

### 12.5 Sync Latency Requirements

| Operation | Target p50 | Target p99 | SLO |
|-----------|-----------|-----------|-----|
| Watch progress sync | 50ms | 100ms | 99.9% |
| Watchlist update | 50ms | 100ms | 99.9% |
| Remote control command | 25ms | 75ms | 99.9% |
| Device presence update | 50ms | 150ms | 99.5% |
| Content expiring notification | 1s | 5s | 99% |
| Trending update | 5s | 30s | 95% |

### 12.6 Offline Handling

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    OFFLINE SYNC FLOW                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  DEVICE GOES OFFLINE:                                                   │
│  ────────────────────                                                   │
│  1. PubNub connection lost (or app backgrounded)                        │
│  2. Local state continues to accumulate changes                         │
│  3. Changes stored in local IndexedDB/SQLite                            │
│  4. Each change tagged with HLC timestamp                               │
│                                                                          │
│  WHILE OFFLINE:                                                         │
│  ─────────────                                                          │
│  • User can browse cached content                                       │
│  • User can modify watchlist (stored locally)                           │
│  • Watch progress recorded locally                                      │
│  • UI indicates "offline mode"                                          │
│                                                                          │
│  DEVICE COMES ONLINE:                                                   │
│  ─────────────────────                                                  │
│  1. Reconnect to PubNub                                                 │
│  2. Fetch messages from history (last 24h)                              │
│  3. Merge remote state with local state using CRDTs                    │
│  4. Publish local changes that weren't sync'd                          │
│  5. Resolve any conflicts (CRDT merge = automatic)                     │
│  6. Update UI with merged state                                         │
│                                                                          │
│  CONFLICT EXAMPLE:                                                      │
│  ─────────────────                                                      │
│  Phone (offline): Add "Movie A" to watchlist at t=10                   │
│  Tablet (online): Add "Movie B" to watchlist at t=11                   │
│  Phone comes online at t=15                                             │
│                                                                          │
│  Result: OR-Set merge → Watchlist contains both "Movie A" AND "Movie B"│
│  (Add-wins semantics: no data loss)                                    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 13. Device Interactions

### 13.1 Device Registration Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    DEVICE REGISTRATION FLOW                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  NEW DEVICE SETUP:                                                      │
│  ─────────────────                                                      │
│                                                                          │
│  1. User opens Media Gateway app on new device                          │
│  2. App generates device fingerprint:                                   │
│     • Device type (phone, tablet, tv, web, cli)                        │
│     • Platform (iOS, Android, webOS, Tizen, etc.)                      │
│     • Unique device ID (stored in secure storage)                      │
│     • Capabilities (4K, HDR, Dolby Atmos)                              │
│                                                                          │
│  3. Device registration request:                                        │
│     POST /api/devices/register                                          │
│     {                                                                    │
│       "device_id": "d-abc123",                                         │
│       "device_type": "tv",                                              │
│       "platform": "webos",                                              │
│       "name": "Living Room TV",                                         │
│       "capabilities": {                                                 │
│         "supports_4k": true,                                            │
│         "supports_hdr": true,                                           │
│         "supports_dolby_atmos": true                                    │
│       },                                                                 │
│       "app_version": "1.2.0"                                            │
│     }                                                                    │
│                                                                          │
│  4. Server responds with:                                               │
│     • Device confirmation                                               │
│     • PubNub tokens (read/write to user channels)                      │
│     • Initial sync state                                                │
│                                                                          │
│  5. Device subscribes to:                                               │
│     • user.{userId}.sync                                                │
│     • user.{userId}.devices                                             │
│     • user.{userId}.notifications                                       │
│     • global.trending                                                   │
│                                                                          │
│  6. Device publishes presence join:                                     │
│     Channel: user.{userId}.devices                                      │
│     Message: { type: "device_join", device_id: "d-abc123", ... }       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 13.2 Device Capabilities Schema

```typescript
interface DeviceCapabilities {
  // Video capabilities
  supports_4k: boolean;
  supports_hdr: boolean;
  hdr_formats: ("hdr10" | "dolby_vision" | "hdr10plus" | "hlg")[];
  max_resolution: "sd" | "hd" | "fhd" | "4k" | "8k";

  // Audio capabilities
  supports_dolby_atmos: boolean;
  supports_dts_x: boolean;
  audio_channels: "stereo" | "5.1" | "7.1" | "atmos";

  // Interaction capabilities
  has_remote_control: boolean;
  has_voice_input: boolean;
  has_touch_screen: boolean;
  has_keyboard: boolean;

  // Platform integrations
  supported_platforms: string[];  // Deep link support
  can_cast_to: string[];  // Chromecast, AirPlay, etc.
  can_receive_cast: boolean;

  // Network
  connection_type: "wifi" | "ethernet" | "cellular";
  bandwidth_tier: "low" | "medium" | "high" | "unlimited";
}

// Platform-specific capability presets
const DEVICE_PRESETS: Record<string, Partial<DeviceCapabilities>> = {
  "apple_tv_4k": {
    supports_4k: true,
    supports_hdr: true,
    hdr_formats: ["dolby_vision", "hdr10"],
    supports_dolby_atmos: true,
    can_receive_cast: true,  // AirPlay
  },
  "fire_tv_stick_4k": {
    supports_4k: true,
    supports_hdr: true,
    hdr_formats: ["dolby_vision", "hdr10plus", "hdr10"],
    supports_dolby_atmos: true,
  },
  "chromecast_4k": {
    supports_4k: true,
    supports_hdr: true,
    hdr_formats: ["dolby_vision", "hdr10"],
    supports_dolby_atmos: true,
    can_receive_cast: true,
  },
  "iphone_15_pro": {
    supports_4k: true,
    supports_hdr: true,
    hdr_formats: ["dolby_vision"],
    has_touch_screen: true,
  },
  "web_browser": {
    supports_4k: false,  // Varies
    supports_hdr: false,
    has_keyboard: true,
    has_touch_screen: false,  // Varies
  },
};
```

### 13.3 Remote Control Protocol

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    REMOTE CONTROL PROTOCOL                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  COMMAND FLOW:                                                          │
│  ────────────                                                           │
│                                                                          │
│  ┌──────────────┐         ┌──────────────┐         ┌──────────────┐    │
│  │    Phone     │         │    PubNub    │         │      TV      │    │
│  └──────┬───────┘         └──────┬───────┘         └──────┬───────┘    │
│         │                        │                        │             │
│         │  1. User taps "Send    │                        │             │
│         │     to TV" button      │                        │             │
│         │                        │                        │             │
│         │  2. Publish command    │                        │             │
│         │────────────────────────▶│                        │             │
│         │  {                      │                        │             │
│         │    type: "remote_ctrl", │                        │             │
│         │    command: "cast",     │                        │             │
│         │    target: "d-tv123",   │                        │             │
│         │    content_id: "...",   │                        │             │
│         │    request_id: "r-1"    │                        │             │
│         │  }                      │                        │             │
│         │                        │                        │             │
│         │                        │  3. Deliver to TV      │             │
│         │                        │────────────────────────▶│             │
│         │                        │                        │             │
│         │                        │                        │  4. TV opens│
│         │                        │                        │     deep    │
│         │                        │                        │     link    │
│         │                        │                        │             │
│         │                        │  5. ACK from TV        │             │
│         │                        │◀────────────────────────│             │
│         │                        │  {                      │             │
│         │                        │    type: "remote_ack",  │             │
│         │                        │    request_id: "r-1",   │             │
│         │                        │    success: true        │             │
│         │                        │  }                      │             │
│         │                        │                        │             │
│         │  6. ACK to phone       │                        │             │
│         │◀────────────────────────│                        │             │
│         │                        │                        │             │
│         │  7. Show "Sent to TV!" │                        │             │
│         │                        │                        │             │
│                                                                          │
│  LATENCY TARGET: <100ms end-to-end (typical: 50-75ms)                  │
│                                                                          │
│  SUPPORTED COMMANDS:                                                    │
│  ───────────────────                                                    │
│  • cast      - Open content on target device                           │
│  • play      - Resume playback                                          │
│  • pause     - Pause playback                                           │
│  • stop      - Stop and close player                                    │
│  • seek      - Jump to specific timestamp                               │
│  • volume    - Adjust volume (if supported)                             │
│  • next      - Skip to next episode                                     │
│  • previous  - Go to previous episode                                   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 13.4 Device Health Monitoring

```yaml
health_monitoring:
  heartbeat:
    interval: 30 seconds
    timeout: 60 seconds
    missed_threshold: 2  # Mark offline after 2 missed heartbeats

  metrics_collected:
    - connection_quality: "excellent" | "good" | "poor" | "offline"
    - last_activity: timestamp
    - current_state: "idle" | "browsing" | "watching"
    - app_version: string
    - errors_last_hour: number

  alerts:
    device_offline:
      condition: "missed heartbeats >= 2"
      action: "update presence, notify other devices"

    device_error:
      condition: "errors_last_hour > 10"
      action: "log to monitoring, consider device degraded"

    version_outdated:
      condition: "app_version < min_supported_version"
      action: "show update prompt on device"

  dashboard_metrics:
    - total_registered_devices
    - devices_online_now
    - devices_by_type
    - average_session_duration
    - most_active_device_type
```

---

## 14. CLI Behavior Specifications

### 14.1 Command Structure

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    CLI COMMAND STRUCTURE                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  COMMAND HIERARCHY:                                                     │
│  ─────────────────                                                      │
│                                                                          │
│  media-gateway                                                          │
│  ├── init              # Initialize configuration                       │
│  │   ├── --force       # Overwrite existing config                     │
│  │   └── --profile     # Create named profile                          │
│  │                                                                       │
│  ├── search            # Content discovery                              │
│  │   ├── <query>       # Natural language search                       │
│  │   ├── --type        # movie, series, any                            │
│  │   ├── --genre       # Filter by genre                               │
│  │   ├── --year        # Filter by year/range                          │
│  │   ├── --platform    # Filter by platform                            │
│  │   ├── --limit       # Max results (default: 10)                     │
│  │   └── --json        # Output as JSON                                │
│  │                                                                       │
│  ├── recommend         # Personalized recommendations                  │
│  │   ├── --mood        # e.g., "relaxing", "exciting"                  │
│  │   ├── --context     # e.g., "family", "date night"                  │
│  │   └── --limit       # Max results                                   │
│  │                                                                       │
│  ├── info              # Content details                                │
│  │   ├── <entity_id>   # Show full metadata                            │
│  │   ├── --availability # Show platform availability                   │
│  │   └── --similar     # Show similar content                          │
│  │                                                                       │
│  ├── watchlist         # Watchlist management                          │
│  │   ├── list          # Show watchlist                                │
│  │   ├── add <id>      # Add to watchlist                              │
│  │   ├── remove <id>   # Remove from watchlist                         │
│  │   └── sync          # Force sync with cloud                         │
│  │                                                                       │
│  ├── devices           # Device management                              │
│  │   ├── list          # Show registered devices                       │
│  │   ├── rename <id>   # Rename device                                 │
│  │   └── remove <id>   # Unregister device                             │
│  │                                                                       │
│  ├── cast              # Send to device                                 │
│  │   ├── <entity_id>   # Content to cast                               │
│  │   └── --device      # Target device                                 │
│  │                                                                       │
│  ├── mcp               # MCP server mode                                │
│  │   ├── start         # Start MCP server                              │
│  │   ├── --transport   # stdio | sse                                   │
│  │   └── --port        # Port for SSE (default: 3000)                  │
│  │                                                                       │
│  ├── config            # Configuration                                  │
│  │   ├── show          # Show current config                           │
│  │   ├── set <key>     # Set config value                              │
│  │   ├── get <key>     # Get config value                              │
│  │   └── reset         # Reset to defaults                             │
│  │                                                                       │
│  ├── auth              # Authentication                                 │
│  │   ├── login         # Interactive login                             │
│  │   ├── logout        # Clear credentials                             │
│  │   ├── status        # Show auth status                              │
│  │   └── token         # Show/refresh access token                     │
│  │                                                                       │
│  ├── status            # System status                                  │
│  │   ├── --services    # Backend service health                        │
│  │   └── --sync        # Sync status                                   │
│  │                                                                       │
│  └── help              # Help and documentation                        │
│      └── <command>     # Help for specific command                     │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 14.2 Interactive vs Non-Interactive Modes

```yaml
modes:
  interactive:
    detection:
      - tty attached
      - stdin is terminal
      - not in CI environment (CI=true not set)

    features:
      - colored output (chalk)
      - progress spinners (ora)
      - interactive prompts (enquirer)
      - ASCII art banner
      - table formatting
      - fuzzy search in lists

    examples:
      - "media-gateway search"  # Prompts for query
      - "media-gateway devices remove"  # Shows device picker

  non_interactive:
    detection:
      - no tty
      - piped output
      - CI environment
      - --no-interactive flag
      - --json flag

    features:
      - plain text output
      - no prompts (fails if required input missing)
      - machine-parseable (JSON with --json)
      - exit codes for scripting
      - no colors (unless --color=always)

    examples:
      - "media-gateway search 'action movies' --json | jq '.results[0]'"
      - "echo 'matrix' | media-gateway search"
```

### 14.3 Configuration File

```toml
# ~/.config/media-gateway/config.toml

[profile.default]
api_endpoint = "https://api.mediagateway.io"
region = "USA"
language = "en-US"

[profile.default.preferences]
default_platforms = ["netflix", "prime_video", "disney_plus"]
default_content_type = "any"
results_per_page = 10
enable_adult_content = false

[profile.default.display]
color = "auto"  # auto, always, never
table_format = "rounded"  # ascii, rounded, minimal, none
date_format = "relative"  # relative, iso, local
show_platform_icons = true

[profile.default.cache]
enabled = true
directory = "~/.cache/media-gateway"
max_size_mb = 100
ttl_hours = 24

[mcp]
default_transport = "stdio"
sse_port = 3000
log_level = "info"

# Additional profiles
[profile.work]
region = "GBR"
default_platforms = ["bbc_iplayer", "all4"]
```

### 14.4 Output Formats

#### 14.4.1 Search Results (Interactive)

```
┌─────────────────────────────────────────────────────────────────────────┐
│  🔍 Search: "sci-fi movies like arrival"                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. Interstellar (2014)                                    ⭐ 8.6       │
│     Drama, Sci-Fi • 2h 49m                                              │
│     A team of explorers travel through a wormhole in space...          │
│     📺 Netflix, Prime Video                                             │
│                                                                          │
│  2. Annihilation (2018)                                    ⭐ 6.8       │
│     Horror, Mystery, Sci-Fi • 1h 55m                                   │
│     A biologist signs up for a dangerous expedition...                 │
│     📺 Netflix, Paramount+                                              │
│                                                                          │
│  3. Contact (1997)                                         ⭐ 7.5       │
│     Drama, Mystery, Sci-Fi • 2h 30m                                    │
│     Dr. Ellie Arroway finds conclusive evidence of alien life...      │
│     📺 HBO Max, Prime Video (rent)                                     │
│                                                                          │
│  [1-3] Select • [n] Next page • [q] Quit • [?] Help                    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 14.4.2 Search Results (JSON)

```json
{
  "query": "sci-fi movies like arrival",
  "parsed_intent": {
    "reference_title": "Arrival",
    "genres": ["sci-fi"],
    "content_type": "movie",
    "mood": "thought-provoking"
  },
  "results": [
    {
      "entity_id": "eidr:10.5240/0D36-19B5-A28B-C53F-J",
      "title": "Interstellar",
      "original_title": "Interstellar",
      "release_year": 2014,
      "content_type": "movie",
      "runtime_minutes": 169,
      "genres": ["Drama", "Sci-Fi"],
      "rating": 8.6,
      "match_score": 0.94,
      "description": "A team of explorers travel through a wormhole in space...",
      "availability": [
        {
          "platform": "netflix",
          "type": "subscription",
          "deep_link": "netflix://title/70305903"
        },
        {
          "platform": "prime_video",
          "type": "subscription",
          "deep_link": "aiv://detail?asin=B00TU9UFTS"
        }
      ],
      "poster_url": "https://image.tmdb.org/t/p/w500/..."
    }
  ],
  "total_results": 47,
  "page": 1,
  "per_page": 10
}
```

### 14.5 Exit Codes

```yaml
exit_codes:
  0:
    name: SUCCESS
    description: Command completed successfully

  1:
    name: GENERAL_ERROR
    description: Generic error

  2:
    name: INVALID_USAGE
    description: Invalid command or arguments

  3:
    name: NOT_AUTHENTICATED
    description: Authentication required

  4:
    name: NOT_FOUND
    description: Resource not found (content, device, etc.)

  5:
    name: NETWORK_ERROR
    description: Unable to reach API

  6:
    name: PERMISSION_DENIED
    description: Insufficient permissions

  7:
    name: RATE_LIMITED
    description: API rate limit exceeded

  10:
    name: CONFIG_ERROR
    description: Configuration file invalid or missing

  130:
    name: INTERRUPTED
    description: User cancelled (Ctrl+C)
```

### 14.6 Device Authorization (CLI)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    CLI DEVICE AUTHORIZATION                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  $ media-gateway auth login                                             │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                                                                  │   │
│  │   Media Gateway CLI Login                                       │   │
│  │                                                                  │   │
│  │   To authenticate, open this URL in your browser:               │   │
│  │                                                                  │   │
│  │   https://app.mediagateway.io/cli-auth                          │   │
│  │                                                                  │   │
│  │   Then enter this code:                                         │   │
│  │                                                                  │   │
│  │            ┌─────────────────────────┐                          │   │
│  │            │      ABCD-1234          │                          │   │
│  │            └─────────────────────────┘                          │   │
│  │                                                                  │   │
│  │   This code expires in 15 minutes.                              │   │
│  │                                                                  │   │
│  │   Press Ctrl+C to cancel.                                       │   │
│  │                                                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ⠋ Waiting for authorization...                                        │
│                                                                          │
│  ────────────────────────────────────────────────────────────────────── │
│                                                                          │
│  ✓ Successfully authenticated as user@example.com                       │
│                                                                          │
│  Device registered: CLI on macOS (d-cli-abc123)                         │
│  Profile: default                                                        │
│                                                                          │
│  You can now use all media-gateway commands.                            │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## End of Part 3

**Continue to:** [SPARC Specification — Part 4 of 4](./SPARC_SPECIFICATION_PART_4.md)

**Part 4 Contents:**
- Service Expectations
- Agent Orchestration Goals
- Authentication Constraints
- Error Cases
- Performance Requirements
- Constraints and Assumptions
- Non-Functional Requirements
