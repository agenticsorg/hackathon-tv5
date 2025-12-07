# Media Gateway - Architecture Context

**Document Version:** 1.0.0
**Generated:** 2025-12-06
**SPARC Phase:** All Phases Consolidated
**Purpose:** Comprehensive architecture context for the entire swarm implementation

---

## Executive Summary

Media Gateway is a production-ready, AI-native entertainment discovery platform that solves the "45-minute decision problem" by providing unified search across 150+ streaming platforms with sub-500ms latency, SONA-powered personalization, and real-time cross-device synchronization.

### Key Metrics

| Metric | Target | Technology |
|--------|--------|-----------|
| Search Latency (p95) | <500ms | Rust + Qdrant + PostgreSQL |
| SONA Personalization | <5ms | Rust + ONNX Runtime |
| Cross-Device Sync | <100ms | Rust + PubNub + CRDT |
| Platform Coverage | 150+ platforms, 60+ countries | Aggregator APIs |
| Token Efficiency | 85% reduction vs HTML scraping | ARW Protocol |
| Infrastructure Cost | <$4,000/month | GCP GKE Autopilot |
| System Availability | 99.9% | Multi-zone deployment |

---

## Technology Stack Summary

### Languages & Distribution
- **Rust 80%**: All performance-critical services (Discovery, SONA, Sync, Auth, Ingestion, Playback)
- **TypeScript 20%**: MCP Server, API Gateway, CLI tools, Web frontend

### Core Technologies
```yaml
runtime:
  rust_edition: "2021"
  rust_version: "1.75+"
  node_version: "18+"

http_frameworks:
  rust: "actix-web 4.x"
  typescript: "fastify 4.x"

async_runtime:
  rust: "tokio 1.x"

databases:
  primary: "PostgreSQL 15 (Cloud SQL HA)"
  cache: "Redis 7 (Memorystore)"
  vector: "Qdrant (self-hosted on GKE)"
  graph: "Ruvector (SQLite/PostgreSQL)"

messaging:
  realtime: "PubNub"
  events: "Cloud Pub/Sub (Kafka compatible)"

observability:
  metrics: "Prometheus + Cloud Monitoring"
  logging: "Cloud Logging (structured JSON)"
  tracing: "OpenTelemetry + Cloud Trace"
```

---

## Service Boundaries & Ports

### Core Services (Tier 1 - 99.9% SLA)

| Service | Port | Language | Scaling | Responsibility |
|---------|------|----------|---------|----------------|
| **Discovery Service** | 8081 | Rust | 3-20 replicas | Natural language search, content lookup, availability filtering |
| **SONA Engine** | 8082 | Rust | 2-10 replicas | AI-powered personalization, semantic recommendations |
| **Sync Service** | 8083 | Rust | 2-5 replicas | Real-time cross-device state sync via PubNub |
| **Auth Service** | 8084 | Rust | 2-10 replicas | OAuth 2.0 + PKCE, JWT tokens, device authorization |
| **MCP Server** | 3000 | TypeScript | 2-10 replicas | Model Context Protocol for AI agents |
| **API Gateway** | 8080 | TypeScript | 3-20 replicas | Rate limiting, auth validation, request routing |

### Supporting Services (Tier 2 - 99.5% SLA)

| Service | Port | Language | Scaling | Responsibility |
|---------|------|----------|---------|----------------|
| **Ingestion Service** | 8085 | Rust | 1-5 replicas | Platform data fetching, normalization, entity resolution |
| **Playback Service** | 8086 | Rust | 2-6 replicas | Device management, deep link generation |

---

## Data Models Summary

### Core Domain Models

#### CanonicalContent
```rust
struct CanonicalContent {
    // Primary identification
    id: Uuid,                           // Internal unique identifier
    content_type: ContentType,          // Movie, Series, Episode, Short

    // Core metadata
    title: String,                      // Primary title (localized)
    original_title: String,             // Original language title
    overview: String,                   // Plot summary
    release_date: NaiveDate,            // Original release

    // External identifiers
    external_ids: ExternalIds,          // EIDR, IMDb, TMDb, etc.

    // Classification
    genres: Vec<Genre>,                 // Comedy, Drama, Sci-Fi
    themes: Vec<String>,                // revenge, redemption, etc.
    moods: Vec<String>,                 // dark, uplifting, intense

    // People
    credits: Credits,                   // Cast and crew

    // Metrics
    popularity_score: f32,              // 0.0-1.0
    average_rating: f32,                // 0.0-10.0
    vote_count: i32,

    // Platform availability
    availability: Vec<PlatformAvailability>,

    // Temporal
    last_updated: DateTime<Utc>,
}

enum ContentType {
    Movie,
    Series,
    Episode,
    Short,
    Documentary,
    Special,
}
```

#### PlatformAvailability
```rust
struct PlatformAvailability {
    platform: Platform,                 // Netflix, Prime, etc.
    region: Region,                     // ISO 3166-1 alpha-2
    availability_type: AvailabilityType, // Subscription, Rental, Purchase, Free

    // Pricing
    price: Option<Money>,               // For rentals/purchases

    // Deep linking
    deep_link: Url,                     // Platform-specific link
    web_fallback: Url,                  // HTTPS fallback

    // Temporal
    available_from: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,

    // Quality
    video_quality: HashSet<VideoQuality>, // SD, HD, UHD, HDR
    audio_tracks: Vec<AudioTrack>,
    subtitle_tracks: Vec<SubtitleTrack>,
}

enum Platform {
    Netflix,
    PrimeVideo,
    DisneyPlus,
    Hulu,
    AppleTvPlus,
    HboMax,
    Peacock,
    ParamountPlus,
    YouTube,
    // ... 150+ total platforms
}
```

#### UserProfile
```rust
struct UserProfile {
    user_id: Uuid,
    external_auth_id: String,           // OAuth provider ID

    created_at: DateTime<Utc>,
    last_active: DateTime<Utc>,

    preferences: UserPreferences,
    privacy_settings: PrivacySettings,
    devices: Vec<Device>,
}

struct UserPreferences {
    favorite_genres: HashSet<Genre>,
    disliked_genres: HashSet<Genre>,
    preferred_languages: Vec<LanguageCode>,
    subscribed_platforms: HashSet<Platform>,
    max_content_rating: Option<ContentRating>,
    preferred_video_quality: VideoQuality,
    autoplay_next: bool,
}
```

---

## API Contracts Summary

### REST API Endpoints

```yaml
/api/v1:
  /content:
    GET /movies             # List movies
    GET /tv                 # List TV shows
    GET /trending           # Trending content
    GET /:id                # Content details

  /search:
    GET /semantic           # Natural language search
    GET /faceted            # Filtered search
    GET /autocomplete       # Search suggestions

  /discover:
    GET /movies             # Discover movies
    GET /tv                 # Discover TV
    GET /popular            # Popular content

  /recommendations:
    GET /for-you            # Personalized recommendations
    GET /similar/:id        # Similar content
    GET /trending           # Trending recommendations

  /user:
    GET /profile            # User profile
    GET /watchlist          # User watchlist
    GET /history            # Watch history
    PATCH /preferences      # Update preferences

  /platforms:
    GET /                   # List platforms
    GET /:platform_id/catalog # Platform catalog

  /auth:
    POST /login             # OAuth login
    POST /logout            # Logout
    POST /refresh           # Refresh token
    POST /device/code       # Device authorization
```

### MCP Protocol Tools

```typescript
// 10+ MCP tools exposed
const mcpTools = [
  'semantic_search',        // Natural language content search
  'get_recommendations',    // Personalized suggestions
  'check_availability',     // Platform availability check
  'get_content_details',    // Full content metadata
  'list_devices',          // User's registered devices
  'initiate_playback',     // Start content on device
  'control_playback',      // Play/pause/seek controls
  'update_preferences',    // Modify user preferences
  'list_platforms',        // Available platforms
  'get_trending',          // Trending content
];
```

### gRPC Service Contracts

```protobuf
// Discovery Service
service DiscoveryService {
  rpc Search(SearchRequest) returns (SearchResponse);
  rpc GetContent(GetContentRequest) returns (ContentResponse);
  rpc GetAvailability(AvailabilityRequest) returns (AvailabilityResponse);
}

// SONA Engine
service SonaService {
  rpc GetRecommendations(RecommendationRequest) returns (RecommendationResponse);
  rpc ScoreContent(ScoreRequest) returns (ScoreResponse);
  rpc UpdateUserProfile(ProfileUpdateRequest) returns (Empty);
}

// Sync Service
service SyncService {
  rpc SyncWatchlist(WatchlistSyncRequest) returns (SyncResponse);
  rpc SyncProgress(ProgressSyncRequest) returns (SyncResponse);
  rpc GetDevices(GetDevicesRequest) returns (DevicesResponse);
}
```

---

## Performance Targets

### Latency SLOs (p95 unless noted)

| Operation | Target | Maximum | Algorithm Complexity |
|-----------|--------|---------|---------------------|
| Content Lookup (by ID) | 20ms | 50ms | O(1) hash lookup |
| Search (simple keyword) | 200ms | 400ms | O(log n) vector search |
| Search (complex NL) | 350ms | 600ms | O(d + log n) embed + search |
| SONA Personalization | 2ms | 5ms | O(1) LoRA inference |
| Recommendation Generation | 100ms | 200ms | O(k log n) top-k search |
| Cross-Device Sync | 50ms | 100ms | O(m) CRDT merge |
| PubNub Message Delivery | 30ms | 75ms | Network latency |
| Entity Resolution | 100ms | 300ms | O(log n) fuzzy match |
| JWT Token Validation | 3ms | 10ms | O(1) signature verify |

### Throughput Targets

| Service | Current | Peak | Capacity |
|---------|---------|------|----------|
| API Gateway | 2,000 RPS | 5,000 RPS | 10,000 RPS |
| Discovery Service | 800 RPS | 2,000 RPS | 3,000 RPS |
| SONA Engine | 600 RPS | 1,500 RPS | 2,000 RPS |
| Sync Service | 4,000 msg/s | 10,000 msg/s | 20,000 msg/s |
| Database (PostgreSQL) | 10K QPS | 30K QPS | 50K QPS |
| Cache (Redis) | 50K QPS | 150K QPS | 200K QPS |

---

## Directory Structure

### Rust Workspace Layout

```
/workspaces/media-gateway/
├── Cargo.toml                  # Workspace root
├── src/
│   └── lib.rs                  # Root library module
│
├── crates/
│   ├── core/                   # Shared types, errors, utilities
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types/          # Domain types
│   │       ├── errors/         # Error types
│   │       └── utils/          # Shared utilities
│   │
│   ├── discovery/              # Discovery Service (Port 8081)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── routes/         # HTTP routes
│   │       ├── handlers/       # Request handlers
│   │       ├── search/         # Search algorithms
│   │       └── grpc/           # gRPC service
│   │
│   ├── sona/                   # SONA Recommendation Engine (Port 8082)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── engine/         # Personalization engine
│   │       ├── models/         # ML model loading
│   │       └── grpc/           # gRPC service
│   │
│   ├── sync/                   # Sync Service (Port 8083)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── crdt/           # CRDT implementations
│   │       ├── pubnub/         # PubNub integration
│   │       └── grpc/           # gRPC service
│   │
│   ├── auth/                   # Auth Service (Port 8084)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── oauth/          # OAuth 2.0 + PKCE
│   │       ├── jwt/            # JWT token management
│   │       └── grpc/           # gRPC service
│   │
│   ├── ingestion/              # Ingestion Service (Port 8085)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── normalizers/    # Platform normalizers
│   │       ├── resolvers/      # Entity resolution
│   │       └── pipeline/       # Ingestion pipeline
│   │
│   ├── playback/               # Playback Service (Port 8086)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── devices/        # Device management
│   │       └── deeplink/       # Deep link generation
│   │
│   ├── mcp/                    # MCP Server Wrapper (Port 3000)
│   │   ├── Cargo.toml          # TypeScript, but Rust can call
│   │   └── src/
│   │       └── lib.rs          # Rust-TypeScript bridge
│   │
│   └── api/                    # API Gateway (Port 8080)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── gateway/        # Gateway logic
│           ├── middleware/     # Rate limiting, auth
│           └── routing/        # Request routing
│
├── tests/                      # Integration tests
│   ├── integration/
│   ├── e2e/
│   └── performance/
│
└── docs/                       # Documentation
    ├── api/
    ├── architecture/
    └── deployment/
```

---

## Core Dependencies

### Rust Dependencies (Cargo.toml)

```toml
[workspace]
members = [
    "crates/core",
    "crates/discovery",
    "crates/sona",
    "crates/sync",
    "crates/auth",
    "crates/ingestion",
    "crates/playback",
    "crates/api",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
authors = ["Media Gateway Team"]
license = "MIT"

[workspace.dependencies]
# HTTP & Web
actix-web = "4"
actix-rt = "2"
actix-cors = "0.7"

# Async runtime
tokio = { version = "1", features = ["full"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }

# Cache
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# HTTP client
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# Identifiers
uuid = { version = "1", features = ["v4", "serde"] }

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
opentelemetry = "0.21"
opentelemetry-otlp = "0.14"

# Error handling
thiserror = "1"
anyhow = "1"

# gRPC
tonic = "0.10"
prost = "0.12"

# Security
jsonwebtoken = "9"
bcrypt = "0.15"
argon2 = "0.5"

# Configuration
config = "0.13"
dotenvy = "0.15"

# Vector operations
qdrant-client = "1.7"

# Testing
mockall = "0.12"
wiremock = "0.5"
```

---

## Database Schemas

### PostgreSQL Schema Organization

```sql
-- Content Schema
CREATE SCHEMA IF NOT EXISTS content;

CREATE TABLE content.canonical_content (
    id UUID PRIMARY KEY,
    content_type VARCHAR(50) NOT NULL,
    title VARCHAR(500) NOT NULL,
    original_title VARCHAR(500),
    overview TEXT,
    release_date DATE,
    popularity_score FLOAT DEFAULT 0.5,
    average_rating FLOAT DEFAULT 0.0,
    vote_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE content.platform_availability (
    id UUID PRIMARY KEY,
    content_id UUID REFERENCES content.canonical_content(id),
    platform VARCHAR(100) NOT NULL,
    region VARCHAR(2) NOT NULL,
    availability_type VARCHAR(50) NOT NULL,
    deep_link TEXT NOT NULL,
    available_from TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- User Schema
CREATE SCHEMA IF NOT EXISTS users;

CREATE TABLE users.profiles (
    id UUID PRIMARY KEY,
    external_auth_id VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_active TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE users.preferences (
    user_id UUID PRIMARY KEY REFERENCES users.profiles(id),
    favorite_genres JSONB DEFAULT '[]',
    disliked_genres JSONB DEFAULT '[]',
    preferred_languages JSONB DEFAULT '["en"]',
    subscribed_platforms JSONB DEFAULT '[]',
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Sync Schema
CREATE SCHEMA IF NOT EXISTS sync;

CREATE TABLE sync.watchlists (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users.profiles(id),
    content_id UUID REFERENCES content.canonical_content(id),
    added_at TIMESTAMPTZ DEFAULT NOW(),
    device_id VARCHAR(255),
    hlc_timestamp BIGINT NOT NULL
);

CREATE TABLE sync.playback_positions (
    user_id UUID REFERENCES users.profiles(id),
    content_id UUID REFERENCES content.canonical_content(id),
    position_seconds INTEGER NOT NULL,
    total_duration_seconds INTEGER,
    last_updated TIMESTAMPTZ DEFAULT NOW(),
    device_id VARCHAR(255),
    hlc_timestamp BIGINT NOT NULL,
    PRIMARY KEY (user_id, content_id)
);
```

### Indexes

```sql
-- Performance indexes
CREATE INDEX idx_content_platform ON content.platform_availability(platform, content_id);
CREATE INDEX idx_content_region ON content.platform_availability(region, platform);
CREATE INDEX idx_content_expires ON content.platform_availability(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_users_external_auth ON users.profiles(external_auth_id);
CREATE INDEX idx_watchlist_user ON sync.watchlists(user_id, added_at DESC);
CREATE INDEX idx_playback_user ON sync.playback_positions(user_id);
```

---

## Deployment Architecture

### GCP Infrastructure

```yaml
region: us-central1
availability_zones: [us-central1-a, us-central1-b, us-central1-c]

compute:
  gke_autopilot:
    cluster_name: media-gateway-prod
    namespace: media-gateway
    node_pool: autopilot-managed
    min_nodes: 2
    max_nodes: 50

databases:
  cloud_sql:
    instance_type: db-custom-2-7680
    ha_mode: regional
    backup: daily_at_3am_utc
    pitr: 7_days

  memorystore_redis:
    tier: standard_ha
    memory: 6GB
    eviction_policy: allkeys-lru

network:
  vpc: custom
  subnets:
    - pods: 10.0.0.0/20
    - services: 10.1.0.0/20
    - cloud_run: 10.2.0.0/24

load_balancer:
  type: L7_HTTPS
  ssl: google_managed
  cloud_armor: enabled
```

---

## Security Architecture

### Authentication Flow

```yaml
web_mobile:
  method: OAuth 2.0 + PKCE
  providers: [Google, GitHub]
  token_lifetime: 3600s  # 1 hour
  refresh_token: 604800s  # 7 days

tv_cli:
  method: Device Authorization Grant (RFC 8628)
  device_code_expiry: 900s  # 15 minutes
  polling_interval: 5s

jwt:
  algorithm: RS256
  issuer: "https://auth.media-gateway.io"
  audience: "media-gateway-api"
```

### Authorization (RBAC)

```yaml
roles:
  anonymous:
    permissions: ["search:read", "trending:read"]
    rate_limit: 10/min

  free_user:
    permissions: ["search", "recommendations_limited", "watchlist", "devices:2"]
    rate_limit: 60/min

  premium_user:
    permissions: ["all", "devices:10", "history_export"]
    rate_limit: 300/min

  admin:
    permissions: ["all", "user_management", "analytics"]
    rate_limit: 1000/min
```

---

## Observability

### Metrics Collection

```yaml
prometheus:
  scrape_interval: 15s
  retention: 30d

  metrics:
    http_requests_total: counter
    http_request_duration_seconds: histogram
    database_query_duration_seconds: histogram
    cache_hit_rate: gauge
    active_connections: gauge
    error_rate: counter
```

### Logging Standards

```json
{
  "timestamp": "2025-12-06T10:30:00.123Z",
  "level": "INFO",
  "service": "discovery-service",
  "version": "1.0.0",
  "trace_id": "abc123",
  "request_id": "req-456",
  "message": "Search query executed",
  "latency_ms": 156
}
```

### Alert Rules

```yaml
critical_alerts:
  - name: ServiceDown
    condition: up == 0 for 2m
    severity: P1

  - name: HighErrorRate
    condition: error_rate > 5% for 5m
    severity: P1

  - name: HighLatency
    condition: p95_latency > 1000ms for 10m
    severity: P2
```

---

## Success Metrics

### Business KPIs

| Metric | Target |
|--------|--------|
| Monthly Active Users (MAU) | 100K by M6, 500K by M12 |
| Day 1 Retention | ≥40% |
| Search Success Rate | ≥70% |
| Recommendation CTR | ≥15% |

### Technical KPIs

| Metric | Target |
|--------|--------|
| System Availability | ≥99.9% |
| API Gateway p95 Latency | <100ms |
| Search p95 Latency | <400ms |
| Cache Hit Rate | >90% |

### Cost KPIs

| Metric | Target |
|--------|--------|
| Total Infrastructure Cost | <$4,000/month at 100K users |
| Cost Per User | <$0.04/user/month |
| CPU Utilization | 50-70% |

---

## Development Standards

### Testing Requirements

```yaml
coverage:
  unit: ">80%"
  integration: ">70%"
  e2e_critical_paths: "100%"

test_pyramid:
  unit: "80%"
  integration: "15%"
  e2e: "5%"
```

### Code Quality

```yaml
rust:
  formatter: rustfmt
  linter: clippy
  max_complexity: 15
  max_file_lines: 500

typescript:
  formatter: prettier
  linter: eslint
  max_complexity: 15
  max_file_lines: 500
```

---

## Next Steps for Implementation

1. **Phase 1 (Week 1-2)**: Core infrastructure setup
   - Workspace creation
   - Database schemas
   - Basic service skeletons

2. **Phase 2 (Week 3-4)**: Core services
   - Discovery Service with search
   - Auth Service with OAuth
   - API Gateway with routing

3. **Phase 3 (Week 5-6)**: Personalization
   - SONA Engine implementation
   - Recommendation pipeline
   - User preference learning

4. **Phase 4 (Week 7-8)**: Real-time & Integration
   - Sync Service with CRDT
   - PubNub integration
   - Cross-device sync

5. **Phase 5 (Week 9-10)**: Production readiness
   - Load testing
   - Security hardening
   - Monitoring & alerting
   - Documentation

---

**Document Status**: Complete
**Generated by**: SPARC Context Parser Agent
**Ready for**: Full swarm implementation
