# SPARC Architecture Phase - Part 2: Microservices Architecture

**Version:** 1.0.0
**Phase:** SPARC Architecture
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [System Architecture Overview](#system-architecture-overview)
3. [Service Definitions](#service-definitions)
4. [Inter-Service Communication](#inter-service-communication)
5. [Data Architecture](#data-architecture)
6. [Deployment Architecture](#deployment-architecture)
7. [Security Architecture](#security-architecture)
8. [Observability Architecture](#observability-architecture)

---

## Executive Summary

This document defines the microservices architecture for the Media Gateway platform. The system is decomposed into 8 core services, each with clearly defined responsibilities, API contracts, and scaling characteristics.

### Design Principles

1. **Bounded Contexts**: Each service owns its domain and data
2. **API-First**: Services communicate via well-defined REST/gRPC interfaces
3. **Eventual Consistency**: Accept eventual consistency for non-critical paths
4. **Fail-Safe**: Services degrade gracefully under load
5. **Stateless Compute**: Application logic is stateless; state lives in databases
6. **Observability**: Every service emits metrics, logs, and traces

### Technology Stack

| Layer | Technology |
|-------|-----------|
| API Gateway | Kong / Envoy |
| Compute | Rust (services), TypeScript (MCP) |
| Data Store | PostgreSQL (primary), Valkey (cache) |
| Vector Search | Ruvector (custom) |
| Message Bus | PubNub (real-time), Kafka (events) |
| Orchestration | Kubernetes (GKE) |
| Observability | Prometheus, Grafana, Loki, Tempo |

---

## System Architecture Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          CLIENT TIER                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐    │
│  │ Web App  │  │  Mobile  │  │   TV     │  │   AI Agents      │    │
│  │(Next.js) │  │(iOS/And) │  │  Apps    │  │(Claude,MCP)      │    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────────────┘    │
└───────┼─────────────┼─────────────┼──────────────┼──────────────────┘
        │             │             │              │
        └─────────────┴─────────────┴──────────────┘
                              │
┌─────────────────────────────┼──────────────────────────────────────┐
│                       API GATEWAY TIER                              │
│             ┌───────────────┴────────────────┐                      │
│             │   API Gateway Service (Kong)   │                      │
│             │  ├─ Rate Limiting               │                      │
│             │  ├─ Authentication              │                      │
│             │  ├─ Request Routing             │                      │
│             │  └─ API Versioning              │                      │
│             └───────────────┬────────────────┘                      │
└─────────────────────────────┼──────────────────────────────────────┘
                              │
┌─────────────────────────────┼──────────────────────────────────────┐
│                      APPLICATION TIER                               │
│    ┌─────────────┬──────────┼────────┬───────────┬─────────────┐   │
│    │             │          │        │           │             │   │
│    ▼             ▼          ▼        ▼           ▼             ▼   │
│ ┌─────────┐ ┌─────────┐ ┌─────┐ ┌─────────┐ ┌──────┐ ┌──────────┐ │
│ │ Content │ │ Search  │ │Recom│ │  Sync   │ │Play- │ │   MCP    │ │
│ │ Service │ │ Service │ │Svc  │ │ Service │ │back  │ │ Service  │ │
│ └────┬────┘ └────┬────┘ └──┬──┘ └────┬────┘ └──┬───┘ └────┬─────┘ │
│      │           │          │         │         │          │       │
└──────┼───────────┼──────────┼─────────┼─────────┼──────────┼───────┘
       │           │          │         │         │          │
       │           │          │         │         │          │
┌──────┼───────────┼──────────┼─────────┼─────────┼──────────┼───────┐
│                       DATA / STORAGE TIER                           │
│   ┌──────┐    ┌──────────┐    ┌────────┐    ┌────────┐            │
│   │ PostgreSQL│  Ruvector   │  │ Valkey │    │ PubNub │            │
│   │ (Primary) │  (Vector)   │  │(Cache) │    │(Sync)  │            │
│   └──────────┘ └──────────┘    └────────┘    └────────┘            │
└─────────────────────────────────────────────────────────────────────┘

                    ┌────────────────────┐
                    │   Auth Service     │
                    │   (Cross-cutting)  │
                    └────────────────────┘
```

### Service Interaction Map

```
┌─────────────────────────────────────────────────────────────────────┐
│                     SERVICE DEPENDENCIES                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  MCP Service ────────┬──────────────────────────────────────────┐   │
│                      │                                           │   │
│  API Gateway ────────┼──────────────────────────────────────┐    │   │
│                      │                                       │    │   │
│                      ▼                                       ▼    ▼   │
│  Search Service ──▶ Content Service ◀─── Recommendation Service   │
│       │                  │                       │                 │
│       │                  │                       │                 │
│       ▼                  ▼                       ▼                 │
│  Ruvector          PostgreSQL              AgentDB/Ruvector       │
│                                                                      │
│  Playback Service ──▶ Sync Service ──▶ PubNub                      │
│       │                  │                                          │
│       │                  │                                          │
│       ▼                  ▼                                          │
│  Device Registry    CRDT State Store                               │
│                                                                      │
│  Auth Service ───────▶ All Services (JWT validation)               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Service Definitions

### 1. API Gateway Service

**Responsibility:** Entry point for all external requests; handles cross-cutting concerns.

#### Architecture

```yaml
service: api-gateway
technology: Kong Gateway 3.x
deployment: Kubernetes (DaemonSet)
instances: 1 per node

responsibilities:
  - Request routing to backend services
  - Rate limiting (per-user, per-IP)
  - Authentication (JWT validation)
  - API versioning (path-based: /v1/, /v2/)
  - Request/response transformation
  - CORS handling
  - API documentation serving

endpoints:
  public:
    - /v1/search
    - /v1/content/:id
    - /v1/recommendations
    - /v1/auth/*
    - /v1/playback/*
    - /v1/sync/*
    - /mcp/* (MCP protocol endpoints)

  admin:
    - /admin/plugins
    - /admin/services
    - /admin/routes

dependencies:
  internal:
    - content-service (HTTP)
    - search-service (HTTP)
    - recommendation-service (HTTP)
    - sync-service (WebSocket)
    - playback-service (HTTP)
    - auth-service (gRPC)
    - mcp-service (HTTP/SSE)

  external:
    - None (entry point)

data_ownership: None (stateless)
```

#### API Contract Summary

```typescript
// Rate Limiting Configuration
interface RateLimitConfig {
  anonymous: {
    requests_per_minute: 10
    burst_size: 5
  }
  authenticated: {
    tier_free: {
      requests_per_minute: 100
      burst_size: 20
    }
    tier_premium: {
      requests_per_minute: 1000
      burst_size: 100
    }
  }
}

// Request Routing
interface RouteConfig {
  path: "/v1/search"
  methods: ["GET", "POST"]
  upstream: "search-service.default.svc.cluster.local:8080"
  strip_path: true
  preserve_host: false
  plugins: ["jwt", "rate-limiting", "correlation-id"]
}
```

#### Scaling Characteristics

| Metric | Behavior |
|--------|----------|
| Horizontal Scaling | Auto (CPU > 70% or requests/sec > 1000) |
| Min Instances | 2 (HA) |
| Max Instances | 10 |
| Scaling Velocity | 30 seconds to add instance |
| Statelessness | Fully stateless |

---

### 2. Content Service

**Responsibility:** Content ingestion, metadata management, entity resolution, platform connectors.

#### Architecture

```yaml
service: content-service
technology: Rust (Actix-web)
deployment: Kubernetes (Deployment)
instances: 2-6 (auto-scaled)

responsibilities:
  - Ingest content from aggregator APIs
  - Normalize platform-specific schemas
  - Entity resolution (deduplication)
  - Metadata enrichment
  - Platform availability management
  - External ID mapping

api:
  rest:
    - GET /content/:id
    - GET /content/search (basic keyword)
    - POST /content/batch (bulk fetch)
    - GET /content/:id/availability
    - POST /internal/ingest (admin)

  grpc:
    - GetContent(content_id) -> ContentDetail
    - BatchGetContent(ids[]) -> ContentDetail[]
    - ResolveExternalId(id_type, id_value) -> ContentDetail

dependencies:
  internal:
    - auth-service (gRPC) - validate requests

  external:
    - Streaming Availability API (aggregator)
    - Watchmode API (aggregator)
    - YouTube Data API (direct)
    - TMDb API (metadata enrichment)
    - Gracenote/TMS API (premium metadata)

data_ownership:
  postgresql:
    - content (canonical content table)
    - external_ids (ID cross-reference)
    - platform_availability (where to watch)
    - content_images (posters, backdrops)
    - credits (cast and crew)

  valkey_cache:
    - content:{id} (TTL: 1 hour)
    - external_id:{type}:{value} (TTL: 24 hours)

background_jobs:
  - catalog_refresh (every 6 hours)
  - availability_sync (every 1 hour)
  - expiry_warnings (every 15 minutes)
  - metadata_enrichment (every 24 hours)
```

#### API Contract Summary

```rust
// GET /content/:id
struct GetContentResponse {
    id: Uuid,
    content_type: ContentType,
    title: String,
    original_title: String,
    overview: String,
    release_date: NaiveDate,
    genres: Vec<Genre>,
    ratings: HashMap<Region, ContentRating>,
    runtime_minutes: Option<u16>,
    availability: Vec<PlatformAvailability>,
    images: ContentImages,
    external_ids: ExternalIds,
}

// POST /internal/ingest
struct IngestRequest {
    platform: Platform,
    raw_content: serde_json::Value,
    region: String,
}

struct IngestResponse {
    content_id: Uuid,
    action: IngestAction, // Created | Updated | Merged
    entity_confidence: f32,
}
```

#### Data Schema

```sql
-- Core content table
CREATE TABLE content (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_type VARCHAR(20) NOT NULL,
    title TEXT NOT NULL,
    original_title TEXT,
    overview TEXT,
    release_date DATE NOT NULL,
    release_year INTEGER NOT NULL,
    runtime_minutes INTEGER,
    popularity_score REAL DEFAULT 0.5,
    average_rating REAL DEFAULT 0.0,
    vote_count INTEGER DEFAULT 0,
    last_updated TIMESTAMPTZ DEFAULT NOW(),

    -- Indexes
    INDEX idx_content_type (content_type),
    INDEX idx_release_year (release_year),
    INDEX idx_popularity (popularity_score DESC),
    INDEX idx_last_updated (last_updated)
);

-- External ID mappings
CREATE TABLE external_ids (
    content_id UUID REFERENCES content(id) ON DELETE CASCADE,
    id_type VARCHAR(50) NOT NULL, -- eidr, imdb, tmdb_movie, tmdb_tv, gracenote
    id_value VARCHAR(255) NOT NULL,

    PRIMARY KEY (id_type, id_value),
    INDEX idx_content_id (content_id)
);

-- Platform availability
CREATE TABLE platform_availability (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_id UUID REFERENCES content(id) ON DELETE CASCADE,
    platform VARCHAR(50) NOT NULL,
    region VARCHAR(3) NOT NULL,
    availability_type VARCHAR(20) NOT NULL,
    price_amount DECIMAL(10,2),
    price_currency VARCHAR(3),
    deep_link TEXT NOT NULL,
    web_url TEXT NOT NULL,
    available_from TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ,

    INDEX idx_content_platform (content_id, platform),
    INDEX idx_region (region),
    INDEX idx_expires_at (expires_at) WHERE expires_at IS NOT NULL
);
```

#### Scaling Characteristics

| Metric | Behavior |
|--------|----------|
| Horizontal Scaling | Auto (CPU > 60% or latency p95 > 200ms) |
| Min Instances | 2 |
| Max Instances | 6 |
| Database Connections | 10-50 per instance (pooled) |
| Cache Hit Ratio | Target >85% |
| External API Budget | Managed via rate limiter |

---

### 3. Search Service

**Responsibility:** Hybrid search (vector + keyword + graph), intent parsing, result ranking, caching.

#### Architecture

```yaml
service: search-service
technology: Rust (Actix-web)
deployment: Kubernetes (Deployment)
instances: 2-8 (auto-scaled)

responsibilities:
  - Parse natural language search queries
  - Execute hybrid search (vector + keyword + graph)
  - Reciprocal Rank Fusion (RRF) result merging
  - Result ranking and personalization
  - Search result caching
  - Query suggestion generation

api:
  rest:
    - POST /search
    - GET /search/suggestions
    - GET /search/trending

  grpc:
    - SearchContent(query, filters) -> SearchResults
    - VectorSearch(embedding, filters) -> ScoredContent[]
    - GraphSearch(seed_ids, filters) -> ScoredContent[]

dependencies:
  internal:
    - content-service (gRPC) - fetch content details
    - recommendation-service (gRPC) - user affinity scores
    - auth-service (gRPC) - user context

  external:
    - GPT-4o-mini (intent parsing)
    - None for search execution (self-contained)

data_ownership:
  ruvector:
    - content_embeddings (768-dim vectors)
    - query_cache (recent queries + results)

  valkey_cache:
    - search:{query_hash} (TTL: 30 minutes)
    - trending_searches (TTL: 5 minutes)
    - popular_queries (TTL: 1 hour)

background_jobs:
  - embedding_refresh (continuous, batched)
  - trending_aggregation (every 5 minutes)
  - cache_warmup (on deployment)
```

#### API Contract Summary

```rust
// POST /search
struct SearchRequest {
    query: String,                      // "movies like The Matrix"
    filters: Option<SearchFilters>,
    page: u32,                          // 1-indexed
    page_size: u32,                     // 1-100
    user_id: Option<Uuid>,              // For personalization
    region: String,                     // ISO 3166-1 alpha-3
}

struct SearchFilters {
    genres: Option<Vec<Genre>>,
    content_types: Option<Vec<ContentType>>,
    year_range: Option<(u16, u16)>,
    rating_range: Option<(f32, f32)>,
    platforms: Option<Vec<Platform>>,
}

struct SearchResponse {
    results: Vec<SearchResult>,
    total_count: usize,
    page: u32,
    page_size: u32,
    query_parsed: ParsedIntent,
    search_time_ms: u64,
}

struct SearchResult {
    content: ContentSummary,
    relevance_score: f32,
    match_reasons: Vec<String>,
    vector_similarity: f32,
    graph_score: f32,
    keyword_score: f32,
}
```

#### Search Algorithm

```
┌─────────────────────────────────────────────────────────────────┐
│                    HYBRID SEARCH FLOW                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. Parse Intent (GPT-4o-mini)                                  │
│     ├─ Extract referenced titles                               │
│     ├─ Detect mood/context                                     │
│     └─ Identify temporal constraints                           │
│                                                                  │
│  2. Generate Query Embedding                                    │
│     └─ Embed query text → 768-dim vector                       │
│                                                                  │
│  3. Parallel Search Execution                                   │
│     ├─ Vector Search (Ruvector HNSW)                           │
│     ├─ Keyword Search (PostgreSQL full-text)                   │
│     └─ Graph Search (if references found)                      │
│                                                                  │
│  4. Reciprocal Rank Fusion (RRF)                               │
│     └─ Merge results: score = 1/(k + rank) for each source    │
│                                                                  │
│  5. Personalization Boost                                       │
│     └─ Apply user affinity if user_id provided                │
│                                                                  │
│  6. Sort & Paginate                                             │
│     └─ Return top N results                                    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### Scaling Characteristics

| Metric | Behavior |
|--------|----------|
| Horizontal Scaling | Auto (CPU > 65% or search latency p95 > 400ms) |
| Min Instances | 2 |
| Max Instances | 8 |
| Vector Index | HNSW in-memory (replicated per instance) |
| Cache Hit Ratio | Target >70% |
| Search Latency | p95 <400ms, p99 <800ms |

---

### 4. Recommendation Service (SONA)

**Responsibility:** User embeddings, LoRA personalization, collaborative filtering, cold-start handling.

#### Architecture

```yaml
service: recommendation-service
technology: Rust (Actix-web) + PyTorch (inference)
deployment: Kubernetes (StatefulSet for model serving)
instances: 2-4 (model-loaded instances)

responsibilities:
  - Build and update user preference vectors
  - Train per-user LoRA adapters
  - Generate personalized recommendations
  - Collaborative filtering
  - Cold-start user handling
  - Diversity filtering (MMR)

api:
  rest:
    - POST /recommendations
    - POST /recommendations/cold-start
    - GET /user/:id/profile
    - POST /user/:id/feedback (interaction tracking)

  grpc:
    - GetRecommendations(user_id, context) -> Recommendation[]
    - CalculateAffinity(user_id, content_id) -> float
    - UpdateUserProfile(user_id, viewing_event) -> void

dependencies:
  internal:
    - content-service (gRPC) - fetch content metadata
    - search-service (gRPC) - graph-based candidates
    - auth-service (gRPC) - user authentication

  external:
    - None (self-contained ML models)

data_ownership:
  agentdb:
    - user_profiles (preference vectors, LoRA adapters)
    - viewing_history (watch events, ratings)
    - genre_affinities (per-user genre scores)

  ruvector:
    - user_embeddings (512-dim vectors)
    - collaborative_matrix (user-user similarity)

  valkey_cache:
    - user_profile:{user_id} (TTL: 15 minutes)
    - recommendations:{user_id}:{context_hash} (TTL: 1 hour)

background_jobs:
  - lora_training (triggered every 10 interactions)
  - collaborative_matrix_update (every 6 hours)
  - cold_start_model_refresh (daily)
```

#### API Contract Summary

```rust
// POST /recommendations
struct RecommendationRequest {
    user_id: Uuid,
    context: Option<RecommendationContext>,
    limit: usize,                       // Max 20
    exclude_watched: bool,
    diversity_threshold: f32,           // 0.0-1.0
}

struct RecommendationContext {
    mood: Option<String>,               // "relaxing", "intense"
    time_of_day: Option<String>,        // "morning", "evening"
    device_type: Option<DeviceType>,    // "tv", "mobile"
    viewing_with: Option<Vec<String>>,  // ["family", "partner"]
}

struct RecommendationResponse {
    recommendations: Vec<Recommendation>,
    generated_at: DateTime<Utc>,
    ttl_seconds: u32,
}

struct Recommendation {
    content: ContentSummary,
    confidence_score: f32,
    recommendation_type: RecommendationType, // Collaborative | ContentBased | Graph
    based_on: Vec<String>,                   // ["similar to The Matrix", "high rating"]
    explanation: String,
}

// POST /user/:id/feedback
struct FeedbackEvent {
    content_id: Uuid,
    event_type: EventType,              // Watched | Rated | Dismissed | Added
    completion_rate: Option<f32>,       // 0.0-1.0
    rating: Option<u8>,                 // 1-5
    timestamp: DateTime<Utc>,
}
```

#### SONA Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                  SONA PERSONALIZATION ENGINE                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  User Profile                                                    │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Preference Vector (512-dim)                             │  │
│  │  Genre Affinities (sparse vector)                        │  │
│  │  Temporal Patterns (24hr, 7day, 4season)                 │  │
│  │  LoRA Adapter (rank-8 low-rank weights)                  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
│  Recommendation Pipeline                                        │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  1. Candidate Generation (parallel)                      │  │
│  │     ├─ Collaborative (similar users)                     │  │
│  │     ├─ Content-based (similar content)                   │  │
│  │     ├─ Graph-based (connections)                         │  │
│  │     └─ Context-aware (time/device/mood)                  │  │
│  │                                                           │  │
│  │  2. LoRA Personalization                                 │  │
│  │     └─ Score refinement via user-specific adapter        │  │
│  │                                                           │  │
│  │  3. Diversity Filter (MMR)                               │  │
│  │     └─ Maximize relevance, minimize similarity           │  │
│  │                                                           │  │
│  │  4. Explanation Generation                               │  │
│  │     └─ Why this recommendation?                          │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### Scaling Characteristics

| Metric | Behavior |
|--------|----------|
| Horizontal Scaling | Auto (CPU > 70% or latency p95 > 150ms) |
| Min Instances | 2 |
| Max Instances | 4 |
| Model Memory | ~2GB per instance (shared base + user LoRAs) |
| Inference Latency | <5ms per user (LoRA forward pass) |
| Training Frequency | Every 10 interactions per user |

---

### 5. Sync Service

**Responsibility:** CRDT operations (LWW, OR-Set), PubNub integration, device presence, offline reconciliation.

#### Architecture

```yaml
service: sync-service
technology: Rust (Tokio) + WebSocket
deployment: Kubernetes (StatefulSet)
instances: 2-4 (sticky sessions via PubNub channels)

responsibilities:
  - Real-time state synchronization across devices
  - CRDT-based conflict resolution
  - Device presence tracking
  - Offline queue management
  - Watchlist sync
  - Playback state sync

api:
  websocket:
    - /sync/ws (bidirectional state sync)

  rest:
    - GET /sync/state/:user_id
    - POST /sync/merge (manual merge trigger)
    - GET /sync/devices/:user_id

  grpc:
    - SyncState(user_id, device_id, operations[]) -> MergeResult
    - GetDevices(user_id) -> Device[]
    - BroadcastUpdate(user_id, state_delta) -> void

dependencies:
  internal:
    - auth-service (gRPC) - user/device authentication
    - playback-service (gRPC) - playback state updates

  external:
    - PubNub (real-time messaging)

data_ownership:
  postgresql:
    - user_state (canonical synced state)
    - device_registry (connected devices)
    - sync_operations (CRDT operation log)

  pubnub:
    - user.{user_id}.sync (real-time channel)
    - device.{device_id}.presence (online/offline)

crdt_types:
  - LWW-Register (last-write-wins) for playback position
  - OR-Set (observed-remove set) for watchlist
  - Counter (G-Counter) for view counts
  - HLC (Hybrid Logical Clock) for timestamps
```

#### API Contract Summary

```rust
// WebSocket message format
enum SyncMessage {
    // Client → Server
    Subscribe { user_id: Uuid, device_id: Uuid },
    StateUpdate { operations: Vec<CRDTOperation> },
    Heartbeat,

    // Server → Client
    StateDelta { operations: Vec<CRDTOperation> },
    Acknowledge { operation_id: Uuid },
    DeviceEvent { device_id: Uuid, event: DeviceEvent },
}

struct CRDTOperation {
    operation_id: Uuid,
    operation_type: OperationType,
    timestamp: HybridLogicalClock,
    device_id: Uuid,
    payload: serde_json::Value,
}

enum OperationType {
    // Watchlist (OR-Set)
    WatchlistAdd { content_id: Uuid, tag: Uuid },
    WatchlistRemove { content_id: Uuid, tag: Uuid },

    // Playback position (LWW-Register)
    PlaybackUpdate { content_id: Uuid, position_ms: u64 },

    // Device presence
    DeviceOnline,
    DeviceOffline,
}

// GET /sync/state/:user_id
struct SyncStateResponse {
    user_id: Uuid,
    watchlist: Vec<WatchlistItem>,
    playback_positions: HashMap<Uuid, PlaybackPosition>,
    devices: Vec<DeviceInfo>,
    last_sync: DateTime<Utc>,
}
```

#### CRDT Merge Algorithm

```
┌─────────────────────────────────────────────────────────────────┐
│                  CRDT CONFLICT RESOLUTION                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  OR-Set (Watchlist)                                             │
│  ────────────────────                                           │
│  State: { added: {(content_id, tag)}, removed: {tag} }         │
│                                                                  │
│  Merge:                                                          │
│    added' = added1 ∪ added2                                     │
│    removed' = removed1 ∪ removed2                               │
│    result = {(c, t) ∈ added' | t ∉ removed'}                   │
│                                                                  │
│  Example:                                                        │
│    Device A: add("Movie", tag_a, t=10:00)                       │
│    Device B: remove("Movie", tag_b, t=10:01)                    │
│    Merged: "Movie" exists if tag_a ≠ tag_b (add-wins bias)     │
│                                                                  │
│  ─────────────────────────────────────────────────────────────  │
│                                                                  │
│  LWW-Register (Playback Position)                               │
│  ──────────────────────────────                                 │
│  State: { value, timestamp }                                    │
│                                                                  │
│  Merge:                                                          │
│    IF timestamp1 > timestamp2 THEN value1                       │
│    ELSE IF timestamp2 > timestamp1 THEN value2                  │
│    ELSE TieBreaker(device_id1, device_id2)                      │
│                                                                  │
│  Example:                                                        │
│    Phone: position=1234ms @ HLC(100, device_a)                  │
│    TV: position=5678ms @ HLC(101, device_b)                     │
│    Merged: 5678ms (later HLC wins)                              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### Scaling Characteristics

| Metric | Behavior |
|--------|----------|
| Horizontal Scaling | Auto (active connections > 1000 per instance) |
| Min Instances | 2 |
| Max Instances | 4 |
| Connections per Instance | 1000-2000 WebSocket connections |
| Sync Latency | <100ms (PubNub target <75ms) |
| PubNub Messages | 1M/day included (Free tier) |

---

### 6. Playback Service

**Responsibility:** Device management, remote control, deep link generation, platform routing.

#### Architecture

```yaml
service: playback-service
technology: Rust (Actix-web)
deployment: Kubernetes (Deployment)
instances: 2-4

responsibilities:
  - Device registration and management
  - Remote playback control
  - Deep link generation (platform-specific)
  - Playback session tracking
  - Platform routing (best availability)

api:
  rest:
    - POST /playback/initiate
    - POST /playback/control (play, pause, seek)
    - GET /playback/session/:id
    - GET /devices
    - POST /devices/register
    - DELETE /devices/:id

  grpc:
    - InitiatePlayback(user_id, content_id, device_id) -> DeepLink
    - ControlPlayback(session_id, command) -> void
    - GetDevices(user_id) -> Device[]

dependencies:
  internal:
    - content-service (gRPC) - fetch availability
    - sync-service (gRPC) - broadcast playback state
    - auth-service (gRPC) - device authorization

  external:
    - None (generates deep links, doesn't control platforms)

data_ownership:
  postgresql:
    - devices (registered devices)
    - playback_sessions (active/historical sessions)

  valkey_cache:
    - device:{device_id} (TTL: 1 hour)
    - session:{session_id} (TTL: duration + 1 hour)
```

#### API Contract Summary

```rust
// POST /playback/initiate
struct InitiatePlaybackRequest {
    content_id: Uuid,
    device_id: Uuid,
    preferred_platform: Option<Platform>,
    quality_preference: Option<VideoQuality>,
    resume: bool,                       // Resume from last position
}

struct InitiatePlaybackResponse {
    deep_link: String,                  // netflix://title/123
    web_fallback: String,               // https://netflix.com/title/123
    session_id: Uuid,
    platform: Platform,
    resume_position_ms: Option<u64>,
}

// POST /devices/register
struct RegisterDeviceRequest {
    device_name: String,
    device_type: DeviceType,
    capabilities: DeviceCapabilities,
    push_token: Option<String>,
}

struct DeviceCapabilities {
    supports_4k: bool,
    supports_hdr: bool,
    supports_dolby_atmos: bool,
    supported_platforms: Vec<Platform>,
}

struct Device {
    device_id: Uuid,
    user_id: Uuid,
    device_name: String,
    device_type: DeviceType,
    capabilities: DeviceCapabilities,
    last_seen: DateTime<Utc>,
    status: DeviceStatus,               // Online | Offline | Watching
}
```

#### Deep Link Generation

```rust
// Platform-specific deep link templates
static DEEP_LINK_TEMPLATES: phf::Map<&str, DeepLinkTemplate> = phf_map! {
    "netflix" => DeepLinkTemplate {
        ios: "nflx://www.netflix.com/title/{id}",
        android: "intent://www.netflix.com/title/{id}#Intent;scheme=nflx;package=com.netflix.mediaclient;end",
        web: "https://www.netflix.com/title/{id}",
    },
    "prime_video" => DeepLinkTemplate {
        ios: "aiv://aiv/detail?asin={id}",
        android: "intent://www.amazon.com/gp/video/detail/{id}#Intent;scheme=https;package=com.amazon.avod.thirdpartyclient;end",
        web: "https://www.amazon.com/gp/video/detail/{id}",
    },
    // ... additional platforms
};

fn generate_deep_link(
    platform: &Platform,
    content_id: &str,
    device_type: DeviceType,
) -> DeepLink {
    let template = DEEP_LINK_TEMPLATES.get(platform.as_str())?;
    let link = match device_type {
        DeviceType::IOS => template.ios.replace("{id}", content_id),
        DeviceType::Android => template.android.replace("{id}", content_id),
        DeviceType::Web | _ => template.web.replace("{id}", content_id),
    };
    DeepLink { link, fallback: template.web.replace("{id}", content_id) }
}
```

#### Scaling Characteristics

| Metric | Behavior |
|--------|----------|
| Horizontal Scaling | Auto (CPU > 60%) |
| Min Instances | 2 |
| Max Instances | 4 |
| Session Tracking | In-memory (per instance) + PostgreSQL |
| Device Registry | PostgreSQL + Valkey cache |

---

### 7. Auth Service

**Responsibility:** OAuth 2.0 + PKCE, Device Authorization Grant, token management, RBAC.

#### Architecture

```yaml
service: auth-service
technology: Rust (Actix-web + OAuth2/OIDC)
deployment: Kubernetes (Deployment)
instances: 2-4

responsibilities:
  - OAuth 2.0 + PKCE authorization code flow
  - Device Authorization Grant (RFC 8628)
  - JWT token issuance and validation
  - Token refresh
  - Role-based access control (RBAC)
  - User session management

api:
  rest:
    - GET /oauth/authorize
    - POST /oauth/token
    - POST /oauth/refresh
    - POST /oauth/revoke
    - GET /oauth/device (device code flow)
    - POST /oauth/device/token

  grpc:
    - ValidateToken(jwt) -> UserContext
    - CreateSession(user_id, device_id) -> SessionToken
    - RevokeSession(session_id) -> void
    - CheckPermission(user_id, resource, action) -> bool

dependencies:
  internal:
    - None (foundational service)

  external:
    - Google OAuth (optional SSO)
    - GitHub OAuth (optional SSO)

data_ownership:
  postgresql:
    - users (user accounts)
    - oauth_clients (registered clients)
    - authorization_codes (temporary codes)
    - device_codes (device flow codes)
    - sessions (active sessions)
    - refresh_tokens (long-lived tokens)

  valkey_cache:
    - jwt_blacklist:{jti} (revoked tokens, TTL: token expiry)
    - device_code:{code} (TTL: 15 minutes)
```

#### API Contract Summary

```rust
// GET /oauth/authorize
struct AuthorizeRequest {
    response_type: String,              // "code"
    client_id: String,
    redirect_uri: String,
    scope: String,
    state: String,
    code_challenge: String,             // PKCE
    code_challenge_method: String,      // "S256"
}

// POST /oauth/token
struct TokenRequest {
    grant_type: String,                 // "authorization_code" | "refresh_token" | "device_code"
    code: Option<String>,
    redirect_uri: Option<String>,
    code_verifier: Option<String>,      // PKCE
    refresh_token: Option<String>,
    device_code: Option<String>,
}

struct TokenResponse {
    access_token: String,               // JWT, 1 hour expiry
    token_type: String,                 // "Bearer"
    expires_in: u64,                    // 3600
    refresh_token: String,              // Long-lived, rotate on use
    scope: String,
}

// GET /oauth/device
struct DeviceCodeResponse {
    device_code: String,                // Server-side code
    user_code: String,                  // 8-char user-friendly code
    verification_uri: String,           // https://mg.app/device
    expires_in: u64,                    // 900 (15 minutes)
    interval: u64,                      // 5 (poll every 5 seconds)
}
```

#### JWT Structure

```json
{
  "header": {
    "alg": "RS256",
    "typ": "JWT",
    "kid": "auth-key-2025-01"
  },
  "payload": {
    "sub": "user-uuid",
    "iss": "https://api.mediagateway.io",
    "aud": ["media-gateway-api"],
    "exp": 1735689600,
    "iat": 1735686000,
    "jti": "token-uuid",
    "scope": "read:content read:preferences write:playback",
    "roles": ["user"],
    "device_id": "device-uuid"
  },
  "signature": "..."
}
```

#### Scaling Characteristics

| Metric | Behavior |
|--------|----------|
| Horizontal Scaling | Auto (CPU > 60%) |
| Min Instances | 2 |
| Max Instances | 4 |
| Token Validation | <5ms (local JWT verification) |
| Session Store | PostgreSQL + Valkey cache |

---

### 8. MCP Service

**Responsibility:** Tool registry, request handling, STDIO/SSE transports, ARW manifest.

#### Architecture

```yaml
service: mcp-service
technology: TypeScript (Node.js 18+)
deployment: Kubernetes (Deployment)
instances: 2-4

responsibilities:
  - MCP protocol implementation (stdio + SSE)
  - Tool registry (10+ tools)
  - Resource serving (config, trending)
  - Prompt serving (discovery assistant, recommendation guide)
  - ARW manifest serving
  - Request routing to backend services

api:
  stdio:
    - JSON-RPC 2.0 over stdin/stdout

  sse:
    - GET /mcp/sse (event stream)
    - POST /mcp/request (tool calls)

  rest:
    - GET /.well-known/arw-manifest.json
    - GET /mcp/tools
    - GET /mcp/resources
    - GET /mcp/prompts

dependencies:
  internal:
    - search-service (gRPC) - semantic_search tool
    - content-service (gRPC) - get_content_details tool
    - recommendation-service (gRPC) - get_recommendations tool
    - playback-service (gRPC) - initiate_playback tool
    - sync-service (gRPC) - list_devices tool
    - auth-service (gRPC) - validate tokens

  external:
    - None (orchestrates internal services)

data_ownership:
  None (stateless orchestrator)
```

#### MCP Tools

```typescript
// Tool registry
const MCP_TOOLS: Tool[] = [
  {
    name: "semantic_search",
    description: "Search for movies and TV shows using natural language",
    inputSchema: {
      type: "object",
      properties: {
        query: { type: "string" },
        filters: { type: "object" },
        limit: { type: "integer", default: 10 }
      },
      required: ["query"]
    }
  },
  {
    name: "get_content_details",
    description: "Get detailed information about specific content",
    inputSchema: {
      type: "object",
      properties: {
        entity_id: { type: "string" },
        region: { type: "string", default: "USA" },
        include: { type: "array", items: { type: "string" } }
      },
      required: ["entity_id"]
    }
  },
  {
    name: "get_recommendations",
    description: "Get personalized content recommendations",
    inputSchema: {
      type: "object",
      properties: {
        context: { type: "string" },
        mood: { type: "string" },
        age_appropriate: { type: "array", items: { type: "integer" } },
        limit: { type: "integer", default: 5 }
      }
    }
  },
  {
    name: "initiate_playback",
    description: "Initiate playback of content on a device",
    inputSchema: {
      type: "object",
      properties: {
        entity_id: { type: "string" },
        device_id: { type: "string" },
        platform: { type: "string" },
        resume: { type: "boolean", default: true }
      },
      required: ["entity_id", "device_id"]
    }
  },
  {
    name: "list_devices",
    description: "List all registered devices for the user",
    inputSchema: {
      type: "object",
      properties: {
        status_filter: {
          type: "string",
          enum: ["all", "online", "offline"],
          default: "all"
        }
      }
    }
  }
  // ... 5 more tools
];
```

#### ARW Manifest

```json
{
  "$schema": "https://arw.agentics.org/schemas/manifest-v1.json",
  "version": "1.0.0",
  "site": {
    "name": "Media Gateway",
    "description": "Unified cross-platform TV and movie discovery engine",
    "logo": "https://app.mediagateway.io/logo.svg"
  },
  "capabilities": {
    "mcp": {
      "version": "2024-11-05",
      "transports": ["stdio", "sse"],
      "tools_url": "/mcp/tools",
      "resources_url": "/mcp/resources"
    },
    "actions": [
      {
        "id": "search",
        "name": "Search Content",
        "oauth_scopes": ["read:content"]
      },
      {
        "id": "recommend",
        "name": "Get Recommendations",
        "oauth_scopes": ["read:content", "read:preferences"]
      },
      {
        "id": "playback",
        "name": "Control Playback",
        "oauth_scopes": ["write:playback"],
        "requires_user_consent": true
      }
    ]
  },
  "authentication": {
    "oauth2": {
      "authorization_endpoint": "/oauth/authorize",
      "token_endpoint": "/oauth/token",
      "scopes": [
        "read:content",
        "read:preferences",
        "write:preferences",
        "write:playback",
        "read:devices"
      ]
    }
  },
  "rate_limits": {
    "unauthenticated": 10,
    "authenticated": 1000,
    "window_seconds": 900
  }
}
```

#### Scaling Characteristics

| Metric | Behavior |
|--------|----------|
| Horizontal Scaling | Auto (CPU > 60%) |
| Min Instances | 2 |
| Max Instances | 4 |
| Request Latency | p95 <100ms (overhead only) |
| SSE Connections | 1000 per instance |

---

## Inter-Service Communication

### Communication Patterns

```yaml
patterns:
  synchronous:
    protocol: gRPC (HTTP/2)
    use_cases:
      - Service-to-service calls (content lookup, user validation)
      - Low latency requirements (<50ms)

    example: |
      search-service → content-service.GetContent(id)
      recommendation-service → search-service.GraphSearch(seeds)

  asynchronous:
    protocol: Kafka (event streaming)
    use_cases:
      - Non-blocking operations (ingestion, analytics)
      - High throughput (10K+ events/sec)

    topics:
      - content.ingested
      - content.updated
      - availability.changed
      - user.interaction

  real-time:
    protocol: PubNub (publish/subscribe)
    use_cases:
      - Cross-device sync (<100ms latency)
      - Presence tracking

    channels:
      - user.{user_id}.sync
      - device.{device_id}.presence
```

### Service Mesh

```yaml
service_mesh: Istio
features:
  - Mutual TLS (mTLS) between services
  - Traffic management (retries, timeouts, circuit breaking)
  - Observability (request tracing)
  - Canary deployments

configuration:
  retries:
    attempts: 3
    per_try_timeout: 2s
    retry_on: 5xx,reset,connect-failure

  circuit_breaker:
    consecutive_errors: 5
    interval: 30s
    base_ejection_time: 30s

  timeout:
    default: 10s
    search_service: 5s
    recommendation_service: 3s
```

---

## Data Architecture

### Database Schema Organization

```
postgresql (primary data store)
├── content_schema
│   ├── content
│   ├── external_ids
│   ├── platform_availability
│   ├── content_images
│   ├── credits
│   └── genres
│
├── user_schema
│   ├── users
│   ├── user_preferences
│   ├── devices
│   └── sessions
│
├── sync_schema
│   ├── sync_operations
│   ├── playback_sessions
│   └── watchlists
│
└── auth_schema
    ├── oauth_clients
    ├── authorization_codes
    ├── device_codes
    └── refresh_tokens
```

### Data Ownership Matrix

| Service | Writes To | Reads From |
|---------|-----------|------------|
| Content Service | content, external_ids, platform_availability | content, external_ids |
| Search Service | None (read-only) | content, external_ids (via content-service) |
| Recommendation Service | user_profiles (AgentDB) | content, user_profiles, viewing_history |
| Sync Service | sync_operations, devices | sync_operations, devices, watchlists |
| Playback Service | playback_sessions, devices | devices, playback_sessions, content |
| Auth Service | users, sessions, oauth_* tables | users, sessions |
| MCP Service | None (stateless) | None (delegates to other services) |

### Caching Strategy

```yaml
valkey (Redis-compatible cache):
  layers:
    L1_gateway:
      ttl: 30s
      keys:
        - "rate_limit:{user_id}"
        - "trending_searches"

    L2_service:
      ttl: 1-24 hours
      keys:
        - "content:{id}"  # 1 hour
        - "user_profile:{user_id}"  # 15 minutes
        - "search:{query_hash}"  # 30 minutes
        - "recommendations:{user_id}:{context}"  # 1 hour

    L3_embedding:
      ttl: 7 days
      keys:
        - "embedding:{content_id}"
        - "external_id:{type}:{value}"

  eviction_policy: allkeys-lru
  max_memory: 4GB per instance
  persistence: AOF (append-only file) for critical keys
```

---

## Deployment Architecture

### Kubernetes Resources

```yaml
namespace: media-gateway

deployments:
  api-gateway:
    type: DaemonSet
    replicas: 1 per node
    resources:
      requests: { cpu: "500m", memory: "512Mi" }
      limits: { cpu: "2000m", memory: "2Gi" }

  content-service:
    type: Deployment
    replicas: 2-6
    resources:
      requests: { cpu: "1000m", memory: "1Gi" }
      limits: { cpu: "4000m", memory: "4Gi" }

  search-service:
    type: Deployment
    replicas: 2-8
    resources:
      requests: { cpu: "2000m", memory: "4Gi" }  # HNSW index in-memory
      limits: { cpu: "8000m", memory: "16Gi" }

  recommendation-service:
    type: StatefulSet
    replicas: 2-4
    resources:
      requests: { cpu: "2000m", memory: "2Gi" }
      limits: { cpu: "4000m", memory: "8Gi" }

  sync-service:
    type: StatefulSet
    replicas: 2-4
    resources:
      requests: { cpu: "500m", memory: "512Mi" }
      limits: { cpu: "2000m", memory: "2Gi" }

  playback-service:
    type: Deployment
    replicas: 2-4
    resources:
      requests: { cpu: "250m", memory: "256Mi" }
      limits: { cpu: "1000m", memory: "1Gi" }

  auth-service:
    type: Deployment
    replicas: 2-4
    resources:
      requests: { cpu: "500m", memory: "512Mi" }
      limits: { cpu: "2000m", memory: "2Gi" }

  mcp-service:
    type: Deployment
    replicas: 2-4
    resources:
      requests: { cpu: "250m", memory: "256Mi" }
      limits: { cpu: "1000m", memory: "1Gi" }

autoscaling:
  metrics:
    - type: Resource
      resource:
        name: cpu
        target: { type: Utilization, averageUtilization: 70 }
    - type: Resource
      resource:
        name: memory
        target: { type: Utilization, averageUtilization: 80 }
    - type: Pods
      pods:
        metric:
          name: http_requests_per_second
        target: { type: AverageValue, averageValue: "1000" }
```

### GCP Infrastructure

```yaml
gcp_project: media-gateway-prod
region: us-central1

compute:
  gke_cluster:
    name: media-gateway-cluster
    node_pools:
      - name: general
        machine_type: n2-standard-4
        nodes: 3-10
      - name: search-optimized
        machine_type: n2-highmem-8  # For HNSW index
        nodes: 2-4
      - name: recommendation
        machine_type: n2-standard-8
        nodes: 2-4

databases:
  cloud_sql:
    instance: media-gateway-postgres
    tier: db-custom-4-16384  # 4 vCPU, 16GB RAM
    storage: 500GB SSD
    backup: automated daily

  memorystore_redis:
    instance: media-gateway-valkey
    tier: standard  # HA
    memory: 10GB
    replicas: 1

storage:
  gcs_buckets:
    - name: media-gateway-images
      location: us-central1
      storage_class: STANDARD
    - name: media-gateway-backups
      location: us-central1
      storage_class: NEARLINE

networking:
  load_balancer:
    type: HTTP(S) Load Balancer
    backend_services:
      - api-gateway (port 80, 443)
      - mcp-service (port 3000 for SSE)
```

---

## Security Architecture

### Defense in Depth

```
┌─────────────────────────────────────────────────────────────────┐
│                      SECURITY LAYERS                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Layer 1: Edge (Cloud Armor)                                    │
│  ├─ DDoS protection                                             │
│  ├─ Rate limiting (global)                                      │
│  └─ IP allowlist/blocklist                                      │
│                                                                  │
│  Layer 2: API Gateway                                           │
│  ├─ JWT validation                                              │
│  ├─ OAuth 2.0 + PKCE                                            │
│  ├─ Rate limiting (per-user)                                    │
│  └─ Request sanitization                                        │
│                                                                  │
│  Layer 3: Service Mesh (Istio)                                  │
│  ├─ mTLS (mutual TLS)                                           │
│  ├─ Service-to-service authz                                    │
│  └─ Network policies                                            │
│                                                                  │
│  Layer 4: Application                                           │
│  ├─ Input validation (Zod schemas)                              │
│  ├─ SQL injection prevention (parameterized queries)            │
│  ├─ XSS prevention (output encoding)                            │
│  └─ CSRF tokens (web clients)                                   │
│                                                                  │
│  Layer 5: Data                                                   │
│  ├─ Encryption at rest (AES-256)                                │
│  ├─ Encryption in transit (TLS 1.3)                             │
│  ├─ Database access controls (IAM)                              │
│  └─ PII masking in logs                                         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Secrets Management

```yaml
secrets_manager: Google Secret Manager

secrets:
  - name: postgres-credentials
    rotation: 90 days
    access: content-service, recommendation-service, sync-service

  - name: oauth-signing-key
    rotation: 180 days
    access: auth-service

  - name: pubnub-keys
    rotation: manual
    access: sync-service

  - name: aggregator-api-keys
    rotation: manual
    access: content-service

injection: Kubernetes secrets mounted as volumes
```

---

## Observability Architecture

### Metrics (Prometheus)

```yaml
metrics:
  service_level:
    - http_requests_total{service, method, status}
    - http_request_duration_seconds{service, endpoint}
    - grpc_server_handled_total{service, method, status}
    - grpc_server_handling_seconds{service, method}

  business:
    - search_queries_total{query_type}
    - recommendations_generated_total{user_id}
    - playback_initiated_total{platform}
    - sync_operations_total{operation_type}

  infrastructure:
    - container_cpu_usage_seconds_total
    - container_memory_working_set_bytes
    - node_cpu_utilization
    - node_memory_utilization

scrape_interval: 15s
retention: 30 days
```

### Logging (Loki)

```yaml
logging:
  format: JSON
  fields:
    - timestamp
    - level (DEBUG, INFO, WARN, ERROR)
    - service
    - trace_id
    - user_id (if applicable)
    - message
    - context (additional fields)

  aggregation: Grafana Loki
  retention: 7 days (INFO+), 30 days (WARN+)

  example: |
    {
      "timestamp": "2025-12-06T12:34:56.789Z",
      "level": "INFO",
      "service": "search-service",
      "trace_id": "abc123...",
      "user_id": "uuid",
      "message": "Hybrid search completed",
      "context": {
        "query": "movies like The Matrix",
        "results_count": 25,
        "duration_ms": 387
      }
    }
```

### Tracing (Tempo)

```yaml
tracing:
  protocol: OpenTelemetry (OTLP)
  backend: Grafana Tempo
  sampling_rate: 0.1  # 10% of requests

  spans:
    - http.request (API Gateway)
    - grpc.call (inter-service)
    - db.query (PostgreSQL)
    - cache.lookup (Valkey)
    - search.vector (Ruvector)
    - ml.inference (SONA)

  example_trace:
    - search_service.handle_request (400ms)
      ├─ intent_parser.parse (50ms)
      ├─ vector_search.execute (150ms)
      │  └─ ruvector.hnsw_search (145ms)
      ├─ keyword_search.execute (80ms)
      │  └─ postgresql.fulltext_search (75ms)
      ├─ graph_search.execute (100ms)
      └─ rrf_fusion.merge (20ms)
```

### Dashboards

```yaml
grafana_dashboards:
  - name: Service Overview
    panels:
      - Request rate (per service)
      - Latency percentiles (p50, p95, p99)
      - Error rate
      - CPU and memory usage

  - name: Search Performance
    panels:
      - Search queries/sec
      - Search latency breakdown (vector, keyword, graph)
      - Cache hit ratio
      - Result count distribution

  - name: Recommendation Quality
    panels:
      - Recommendations generated/sec
      - LoRA training frequency
      - Diversity score
      - User feedback (positive/negative)

  - name: Infrastructure
    panels:
      - Kubernetes pod status
      - Database connection pool
      - PubNub message rate
      - API Gateway throughput
```

---

## Cost Analysis

### Monthly Infrastructure Cost (100K Users)

| Component | Configuration | Monthly Cost |
|-----------|---------------|--------------|
| GKE Cluster | 3 n2-standard-4 nodes | $350 |
| Cloud SQL (PostgreSQL) | db-custom-4-16384 | $450 |
| Memorystore (Valkey) | 10GB standard | $200 |
| Cloud Storage | 500GB + bandwidth | $50 |
| PubNub | 1M messages/day | $0 (free tier) |
| External APIs | Aggregator APIs | $500 |
| Monitoring | Prometheus, Grafana Cloud | $100 |
| **Total** | | **~$1,650/month** |

### Scaling Projections

| Users | Monthly Cost | Notes |
|-------|--------------|-------|
| 10K | $800 | Minimum viable infrastructure |
| 100K | $1,650 | Base architecture (above) |
| 500K | $3,500 | +2 nodes, +database read replicas |
| 1M | $6,000 | +4 nodes, sharded database |

---

## End of Document

**Status:** Complete
**Next Phase:** SPARC Refinement (TDD implementation)
**Review Required:** Architecture team, DevOps, Security

---
