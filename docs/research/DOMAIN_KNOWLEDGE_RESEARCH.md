# Media Gateway Domain Knowledge Research
## Authoritative Domain Context for SPARC Specification Phase

**Research Source**: [media-gateway-research repository](https://github.com/globalbusinessadvisors/media-gateway-research)
**Analysis Date**: 2025-12-06
**Research Scope**: Complete architectural documentation and specifications

---

## 1. Media Gateway Domain Concepts and Terminology

### 1.1 Core Problem Statement

**The "45-Minute Decision Problem"**: The measurable time users waste deciding what to watch across fragmented streaming platforms. Media Gateway solves this by creating a unified cross-platform TV discovery system.

### 1.2 System Identity

**Official Definition**: "A Global Cross-Platform TV Discovery System"

**Key Characteristics**:
- 100% Rust implementation for reliability and speed
- 4-layer architecture supporting 51 independent micro-repositories
- Distributed across foundation, data ingestion, intelligence, consolidation, and application layers
- Agent-driven intelligence with SPARC methodology orchestration

### 1.3 Domain Terminology

**ARW (Agent-Ready Web)**: Protocol specification providing machine-readable structured data views instead of HTML, achieving 85% token reduction for AI agents and 10x faster discovery through pre-computed embeddings and relationship graphs.

**SONA (Self-Optimizing Neural Architecture)**: Intelligence engine with 39 dynamic attention mechanisms, Two-Tier LoRA for runtime adaptation, Tiny Dancer semantic routing, and ReasoningBank for persistent pattern storage.

**Two-Tier LoRA**: Low-rank adaptation technique enabling per-user personalization (~10KB per user) without full model retraining, using constraint W' = W + (α/r) × B @ A.

**Tiny Dancer**: FastGRNN-based semantic query classifier routing requests to specialized handlers (GNN similarity, vector search, mood matching, catalog freshness, availability checks) in <0.5ms.

**ReasoningBank**: Persistent storage of successful recommendation patterns including query embeddings, result IDs, user segments, and contextual metadata for pattern-based boosting.

**Ruvector**: Hybrid data engine combining hypergraph + vector + GNN capabilities with HNSW indexes for similarity search and Cypher-compatible query language.

**hackathon-tv5**: Agentics Foundation toolkit providing 17+ integrated tools, ARW protocol implementation, and MCP server capabilities across orchestration, data, and infrastructure layers.

**E2B Sandboxing**: Firecracker microVM-based secure execution environment with ~150ms startup times, providing hardware-level isolation for untrusted LLM-generated code.

**CRDT (Conflict-Free Replicated Data Types)**: Distributed data structures enabling offline-first synchronization with automatic conflict resolution—Last-Writer-Wins registers for watch progress, Observed-Remove Sets for watchlist management.

**Trust Scoring**: 5-component weighted confidence system (source reliability 25%, metadata accuracy 25%, availability confidence 20%, recommendation quality 15%, preference confidence 15%) with exponential time decay.

---

## 2. Streaming Platform Interaction Patterns

### 2.1 API Availability Landscape

**Critical Constraint**: 8 out of 10 major streaming platforms offer NO public APIs for third-party consumer applications.

**Public API Access**:
- **YouTube/YouTube TV**: Full public API via YouTube Data API v3 with OAuth 2.0 and Device Authorization Grant support

**Partner-Only APIs**:
- **Netflix**: Backlot API (content partners only) with OAuth 2.0 + JWT
- **Prime Video**: Video Central and Reporting APIs (Amazon Partner Network only)
- **Apple TV+**: Video Partner Program with XML feeds and subscription APIs

**No Public APIs**:
- Disney+, Hulu, Crave, HBO Max, Peacock, Paramount+ provide no developer access

### 2.2 Integration Strategies

**Path 1: Direct API Integration (YouTube Only)**
- OAuth 2.0 with Authorization Code + PKCE flow
- Device Authorization Grant (RFC 8628) for TV/CLI devices
- Rate limiting per API key and authenticated user
- Direct server-to-server integration

**Path 2: Partner Programs (Limited Access)**
- Manual business partnership vetting process
- Internal APIs with proprietary authentication
- Closed ecosystem requiring specific integrations

**Path 3: Third-Party Aggregators (Recommended for Most Platforms)**
- **Streaming Availability API**: 60+ countries, ISO 639-2 language codes
- **Watchmode API**: 200+ services, 51 countries, episode-level deep links
- **International Showtimes API**: 100+ markets with pricing data
- **Reelgood API**: 150+ services with commercial licensing

**Path 4: TV Everywhere/MVPD Integration**
- Adobe Pass Authentication for cable/satellite subscribers
- 4-6 week typical integration timeline

### 2.3 Platform-Specific Constraints

**Netflix**:
- No public consumer API
- Partner API requires "Fulfillment Partner: Admin" role
- OAuth 2.0 + JWT authentication
- Enterprise SSO via OIDC (Okta)

**Prime Video**:
- Login with Amazon (LWA) OAuth protocol
- Amazon Developer Console admin credentials required
- Quota-based rate limiting

**Apple TV+**:
- VideoSubscriberAccount framework for TV provider authentication
- XML catalog feeds and availability feeds
- Requires Xcode 8+, iOS 10.0+, tvOS 10.0+

**YouTube**:
- OAuth 2.0 Authorization Code Grant for web/mobile
- Device Authorization Grant for TVs
- API key required for all requests
- Rate limiting enforced per key and user

### 2.4 Data Access Model

**What Third-Party Integrations Can Access**:
- Content metadata (titles, descriptions, cast, genres, ratings)
- Platform availability with expiry dates
- Deep links to platform apps
- Regional pricing (subscriptions, rentals, purchases)
- Public popularity signals

**What Is Restricted**:
- User watch history (not available via third-party APIs)
- Personal viewing preferences
- Individual playback behavior
- Platform-specific recommendations

**Strategic Implication**: Media Gateway aggregates public metadata and availability data, using local encrypted storage for user behavioral data with federated learning for privacy-safe personalization.

---

## 3. Metadata Requirements and Standards

### 3.1 Industry Standard Identifiers

**EIDR (Entertainment Identifier Registry)**:
- Non-proprietary unique identifier for all content types
- Supports hierarchical relationships (Series → Season → Episode)
- Links to IMDb, Rotten Tomatoes, Common Sense Media
- Minimal registration requirements
- Recommended as primary cross-platform identifier

**Gracenote TMS IDs**:
- Proprietary metadata system owned by Nielsen
- Provides theme, genre, mood, keywords, Nielsen ratings
- Covers 85+ countries with 105,000+ unique titles (Q3 2025)
- Enhanced via "Content Connect Platform" for CTV advertising
- Can be combined with EIDR for richer metadata webs

**TMDb (The Movie Database)**:
- Community-built database with public API
- Widely used by third-party applications
- Often cross-referenced with EIDR and Gracenote

**Integration Strategy**: Use multiple identifier types for cross-referencing and validation—EIDR as primary, Gracenote for enrichment, TMDb for community data.

### 3.2 Metadata Schema Requirements

**Content Node Properties**:
- Title, original title, description, release date
- Duration, content rating (MPAA/TV), language codes (ISO 639-2)
- 1536-dimensional vector embedding (OpenAI text-embedding-3-large)
- Trust score components (0.0-1.0 scale)
- Entity resolution confidence scores

**Person Node Properties**:
- Name, roles (actor, director, creator), biography
- 768-dimensional embedding (sentence-transformers)
- Filmography relationships via ACTED_IN, DIRECTED edges

**Platform Availability Properties**:
- Service name, pricing model (subscription/rental/purchase)
- Quality tiers (SD/HD/4K), audio options (stereo/5.1/Atmos)
- Subtitle languages, deep link URLs
- Regional availability (ISO 3166-1 alpha-3)
- Temporal windows (Unix timestamps for expiry)
- Trust scores for availability confidence

**Genre and Mood Taxonomies**:
- Standardized genre classifications
- Mood descriptors for contextual recommendations
- Multi-label support (content can have multiple genres)

### 3.3 Data Quality Standards

**Completeness Requirements**:
- Minimum 90% metadata completeness for high trust scores
- All primary fields (title, description, release date) mandatory
- Cast information required for collaborative filtering

**Accuracy Validation**:
- Cross-source validation across multiple aggregators
- User correction tracking with penalty application
- Entity resolution confidence thresholds
- Field-level trust scoring

**Freshness Targets**:
- Daily catalog updates from aggregators
- Real-time availability status via platform deep links
- Exponential trust decay: trust(t) = original × (1 - 0.01 × days_since_verification)

---

## 4. Ingestion Behavior Specifications

### 4.1 MCP Connector Framework

**Standardized Rust Trait Implementation**:
```rust
// All platform integrations implement this trait
pub trait MCPConnector {
    fn tool_schema(&self) -> ToolSchema;
    async fn fetch_catalog(&self, cursor: Option<String>) -> Result<CatalogPage>;
    async fn search_content(&self, query: SearchQuery) -> Result<Vec<Content>>;
    async fn health_check(&self) -> Result<HealthStatus>;
}
```

**Connector Capabilities**:
- Tool schema definitions for agent discovery
- Async catalog fetching with cursor-based pagination
- Content search with filtering
- Health checks for service availability
- Configurable rate limiting and retry logic

### 4.2 Ingestion Pipeline Architecture

**Layer 1 (Data Ingestion)**: 20 micro-repositories handling:
- Platform-specific MCP connectors (Netflix, Prime, Disney+, YouTube, etc.)
- Metadata normalizers for schema standardization
- Entity resolution for deduplication across sources
- Authentication and identity management
- Device state synchronization

**Data Flow Pattern**:
1. External sources (JustWatch, Watchmode, TMDb APIs) → Ingestion connectors
2. Raw data → Metadata normalizers → Standardized schema
3. Normalized data → Entity resolver → Unified entities with EIDR/TMS IDs
4. Resolved entities → Ruvector hypergraph database
5. Change events → Kafka topics (`content.ingested`, `rights.availability.changed`)

### 4.3 Rate Limiting and Throttling

**Third-Party Aggregator Limits**:
- **YouTube Data API**: Quota-based per API key + per user
- **Streaming Availability API**: Tiered subscription limits
- **Watchmode API**: Tier-specific request quotas

**Internal Rate Limiting**:
- Token bucket algorithms per connector
- Configurable capacity and refill rates
- Backpressure handling via semaphore-based controls
- Batch ingestion processing 1000-item batches with progress tracking

### 4.4 Error Handling and Resilience

**Circuit Breaker Pattern**:
- Opens after threshold failures (e.g., 5 failures in 60 seconds)
- Half-open state for recovery testing
- Full-open rejects requests immediately to prevent cascade failures

**Retry Logic**:
- Exponential backoff with configurable maximum attempts
- Distinction between retryable (503, timeout) and non-retryable (401, 404) errors
- Jitter addition to prevent thundering herd

**Error Categories**:
- **Client Errors (4xx)**: Invalid input, unauthorized, not found
- **Server Errors (5xx)**: Service unavailable, timeout, internal error
- **Custom Errors**: Circuit breaker open, rate limit exceeded, invalid schema

### 4.5 Data Freshness Management

**Update Frequencies**:
- Aggregator catalog updates: Daily to weekly depending on provider
- Availability status: Real-time validation via deep link health checks
- User preference updates: Immediate local storage with federated sync

**Staleness Detection**:
- Timestamp tracking for last verification
- Exponential trust decay for unverified data
- Automatic re-validation triggers for low-confidence content

---

## 5. Device Interaction Patterns

### 5.1 Supported Device Platforms

**Smart TV Platforms**:
- **Samsung Tizen**: Native app with TV-specific UX
- **LG webOS**: webOS SDK integration
- **Roku**: External Control Protocol (ECP) for control
- **Fire TV**: Android Debug Bridge (ADB) integration
- **Apple TV**: AirPlay protocol for casting
- **Google TV**: Cast protocol for content delivery

**Mobile Platforms**:
- **iOS**: Native app with Universal Links
- **Android**: Native app with App Links

**Web Platforms**:
- **Web Browsers**: Next.js application with responsive design

**Command-Line Interfaces**:
- **CLI/TUI**: Bubble Tea (Go) and Commander.js (Node.js) implementations

### 5.2 Device Authorization Patterns

**OAuth 2.0 + PKCE (Web/Mobile)**:
- 15-minute access tokens
- 7-day refresh tokens
- Authorization Code Grant with Proof Key for Code Exchange
- Secure token storage: Keychain (macOS/iOS), Keystore (Android), Secret Service (Linux)

**Device Authorization Grant RFC 8628 (TV/CLI)**:
- Device requests device_code from authorization server
- User authorizes on secondary device (phone/computer) via QR code or URL
- Primary device polls for authorization completion
- No keyboard input required on TV

**Deep Linking Authentication**:
- **iOS Universal Links**: `apple-app-site-association` file at `.well-known/` endpoint
- **Android App Links**: `assetlinks.json` at `.well-known/` endpoint
- HTTPS + domain verification required
- Fallback to browser if app not installed

### 5.3 Cross-Device Synchronization

**PubNub Channel Organization**:
- **User Scope**: `user.{id}.sync`, `user.{id}.devices` for personal state
- **Global Scope**: `trending`, `announcements` for system-wide updates
- **Regional Scope**: Region-specific availability changes

**CRDT Synchronization Protocols**:
- **Last-Writer-Wins Registers**: Watch progress timestamps with HLC (Hybrid Logical Clocks)
- **Observed-Remove Sets**: Watchlist management with concurrent add/remove support
- Offline queuing with automatic conflict resolution on reconnection

**Sync Propagation Performance**:
- Sub-100ms via PubNub real-time messaging
- Presence channels for device online/offline status
- Command publishing to device-specific channels

### 5.4 Playback Control and Deep Linking

**Deep Link URL Patterns**:
- Netflix: `netflix.com/title/{id}` or `nflx://` scheme
- Prime Video: `primevideo.com/detail/{id}` or `aiv://` scheme
- Disney+: `disneyplus.com/video/{id}` or `disneyplus://` scheme
- YouTube: `youtube.com/watch?v={id}` or `youtube://` scheme

**Cross-Device Playback Commands**:
- Play/pause/stop controls via PubNub commands
- Seek position synchronization
- Volume and subtitle settings
- Queue management across devices

**Device State Polling**:
- TV devices poll for playback state updates
- Phones receive synchronization confirmations
- Status indicators update in real-time across all connected clients

### 5.5 Device Capability Detection

**Quality Tier Selection**:
- Automatic detection of 4K/HDR support
- Audio capability detection (Atmos, 5.1, stereo)
- Network bandwidth assessment for quality adaptation

**Interface Adaptation**:
- 10-foot UI for TV experiences
- Touch-optimized for mobile
- Keyboard/mouse for desktop web
- TUI for command-line interfaces

---

## 6. Performance Requirements and Benchmarks

### 6.1 Latency Targets

**Query and Recommendation Performance**:
- **Total query latency**: 430ms end-to-end
- **First results**: ~200ms (progressive rendering)
- **P50 recommendation latency**: 62ms
- **P99 recommendation latency**: 124ms
- **Sub-100ms target**: For interactive recommendation generation

**SONA Intelligence Engine**:
- **LoRA loading**: 5ms (P50) / 15ms (P99)
- **Embedding adaptation**: 3ms (P50) / 10ms (P99)
- **Semantic routing (Tiny Dancer)**: 0.5ms (P50) / 2ms (P99)
- **Full SONA pipeline**: 20ms (P50) / 50ms (P99)

**Multi-Agent Orchestration (SPARC)**:
- **Specification phase**: 50ms (requirements analysis)
- **Pseudocode phase**: 30ms (query planning)
- **Architecture phase**: 200ms (parallel agent execution)
- **Refinement phase**: 100ms (quality assurance)
- **Completion phase**: 50ms (response formatting)

**Database and Search Performance**:
- **Ruvector query routing**: Sub-millisecond via Tiny Dancer
- **Vector similarity search**: HNSW index optimized
- **Hypergraph Cypher queries**: Streaming results with async executors
- **Concurrent query support**: 10 parallel queries

### 6.2 Throughput Targets

**Recommendation Throughput**:
- 12,000 requests per second (with caching)
- Kubernetes HPA targeting 70% CPU, 80% memory utilization
- Auto-scaling from 3 to 20 replicas based on load

**Trust Validation Overhead**:
- Per-request trust scoring: <50ms
- Trust score calculations integrated into ranking pipeline

**Sync Propagation**:
- <100ms PubNub message delivery
- CRDT conflict resolution without blocking

### 6.3 Resource Efficiency Benchmarks

**SONA Per-User Overhead**:
- User adapter storage: ~5MB per active user
- FastGRNN model: 800KB total
- Pattern storage (ReasoningBank): 50KB average per user
- Total cache requirement: ~10GB for 1M active users

**E2B Sandbox Performance**:
- Startup time: ~150ms (sub-200ms target)
- Daily sandbox executions: ~3,800 across all agents
- Monthly compute hours: ~1,000 hours
- Estimated cost: $250-450/month

**Embedding Storage**:
- 1M movies × 1536-dim: ~6GB
- 500K people × 768-dim: ~1.5GB
- 10M episodes × 768-dim: ~30GB
- HNSW index overhead: ~2x raw data
- Total estimated: ~80GB for full catalog

### 6.4 Scalability Benchmarks

**Load Testing Targets**:
- 100K concurrent users for production readiness (Phase 5)
- <50ms P99 latency under load (optimization target)

**GKE Autopilot Scaling**:
- Layer 1-3 microservices: 15+ services
- Auto-scaling based on CPU/memory thresholds
- Spot VM integration for 60-80% cost savings on fault-tolerant workloads

**Database Scaling**:
- Cloud SQL PostgreSQL with regional HA
- Memorystore Valkey (10GB capacity) for distributed caching
- Ruvector with Raft consensus for distributed graph operations

### 6.5 Quality Metrics

**Recommendation Quality**:
- **Precision@10**: 0.26 baseline → 0.31 with SONA (+19%)
- **NDCG@10**: 0.54 baseline → 0.63 with SONA (+17%)
- **Cold-start accuracy**: 47% improvement after 5 interactions

**Trust Score Effectiveness**:
- Content below 0.6 trust filtered from recommendations
- Trust decay function: trust(t) = original × (1 - 0.01 × days_since_verification)
- Multi-component validation (5 weighted factors)

---

## 7. Security and Authentication Best Practices

### 7.1 OAuth 2.0 Security Standards (RFC 9700, January 2025)

**REQUIRED Modern OAuth Practices**:
- **Authorization Code + PKCE**: For ALL client types (web, mobile, native)
- **Short-lived access tokens**: 15-60 minutes maximum
- **Refresh token rotation**: New refresh token issued on each use
- **Sender-constrained tokens**: mTLS or DPoP for token binding
- **MFA for critical operations**: Multi-factor authentication required

**DEPRECATED (Do Not Use)**:
- **Implicit Grant**: Officially deprecated
- **Resource Owner Password Credentials**: Security vulnerability

### 7.2 Authentication Mechanisms

**OAuth 2.0 + PKCE (Web/Mobile)**:
- Authorization endpoint: `https://accounts.{provider}.com/o/oauth2/v2/auth`
- Token endpoint: `https://oauth2.{provider}.com/token`
- Code verifier generation: SHA256(random 32-byte string)
- Code challenge: Base64URL(SHA256(code_verifier))
- Access token TTL: 15 minutes
- Refresh token TTL: 7 days

**Device Authorization Grant RFC 8628 (TV/CLI)**:
- Device code endpoint: `https://oauth2.{provider}.com/device/code`
- User authorization via QR code or short URL
- Device polling with exponential backoff
- No keyboard input required for TV devices

**Service-to-Service Authentication**:
- Mutual TLS (mTLS) with certificate-based authentication
- Service identifiers in request headers
- Request ID tracking for observability

### 7.3 Token Storage and Management

**Secure Token Storage**:
- **macOS/iOS**: Keychain Services
- **Android**: Keystore system
- **Linux**: Secret Service API (GNOME Keyring, KWallet)
- **Windows**: Credential Manager

**Token Lifecycle Management**:
- Automatic refresh before expiration
- Refresh token rotation on each use
- Secure deletion on logout
- Token revocation on security events

### 7.4 Privacy Architecture

**Three-Tier Data Handling**:

**Tier 1 (On-Device)**:
- Full watch history encrypted with AES-256-GCM
- PBKDF2-derived keys from user credentials
- Local TensorFlow Lite models for personalization
- Biometric authentication (Face ID, Touch ID, fingerprint)
- No cloud transmission of raw behavioral data

**Tier 2 (Aggregated Server-Side)**:
- Anonymized preference clusters with k-anonymity (1000+ users minimum)
- Differential privacy (ε=1.0, δ=1e-5) for federated learning
- Gradient updates with Laplace noise injection
- Gradient clipping at norm threshold 1.0
- Secure aggregation (server never sees individual updates)

**Tier 3 (Public)**:
- Content metadata (titles, descriptions, cast)
- Aggregate popularity signals
- Public ratings and reviews
- Platform availability (no user association)

### 7.5 Privacy Regulation Compliance

**GDPR (EU)**:
- Explicit prior consent required for data collection
- Right to access personal data
- Right to deletion (Right to be Forgotten)
- Right to data portability
- Applies globally to EU-resident users

**CCPA (California)**:
- Right to Know what data is collected
- Right to Opt-Out of data sharing
- Right to Deletion of personal information
- Broad definition includes device IDs, IP addresses, cookies
- 2025 enforcement example: $530,000 settlement for ineffective opt-out mechanisms

**VPPA (Video Privacy Protection Act) - 2025 Resurgence**:
- Explicit consent required for video viewing data collection
- Applies to embedded video players and social pixels (Meta Pixel)
- Wave of litigation impacts publishers, advertisers, platforms
- Viewing behavior requires explicit user authorization

### 7.6 Security Implementation Patterns

**Rate Limiting**:
- Token bucket algorithms with configurable capacity
- Per-tool rate limit configurations
- 1,000 requests per minute per IP via Cloud Armor
- Graduated throttling under load

**DDoS Protection (Cloud Armor)**:
- XSS and SQL injection filtering
- Adaptive DDoS defense
- Geographic IP blocking if needed
- WAF rule customization

**Container Security**:
- Distroless container images (no shell interpreters)
- Non-root user execution
- Read-only root filesystems where possible
- Security scanning in CI/CD pipeline

**Workload Identity (GKE)**:
- Service account annotation binding Kubernetes to GCP identities
- Eliminates credential storage in containers
- Fine-grained IAM permissions per service
- Automatic credential rotation

---

## 8. Error Handling Patterns

### 8.1 Error Classification Framework

**Client Errors (4xx)**:
- **INVALID_INPUT**: Malformed requests, missing required fields
- **UNAUTHORIZED**: Missing or invalid authentication tokens
- **FORBIDDEN**: Valid token but insufficient permissions
- **NOT_FOUND**: Requested resource does not exist
- **RATE_LIMIT_EXCEEDED**: Client exceeded quota

**Server Errors (5xx)**:
- **SERVICE_UNAVAILABLE**: Backend service temporarily down
- **TIMEOUT**: Request exceeded configured deadline
- **INTERNAL_ERROR**: Unexpected server-side failure
- **DEPENDENCY_FAILURE**: Downstream service error

**Custom Error Codes**:
- **CIRCUIT_BREAKER_OPEN**: Circuit breaker preventing requests
- **TRUST_SCORE_TOO_LOW**: Content filtered due to low confidence
- **REGION_NOT_SUPPORTED**: Content not available in user region
- **PLATFORM_NOT_AVAILABLE**: Streaming service not accessible

### 8.2 Error Response Structure

**Standardized Error Format**:
```json
{
  "error": {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Recommendation service temporarily unavailable",
    "details": {
      "service": "recommendation-engine",
      "retry_after_seconds": 30
    },
    "retryable": true,
    "request_id": "req_abc123xyz"
  }
}
```

**Error Response Components**:
- **code**: Machine-readable error identifier
- **message**: Human-readable description
- **details**: Optional context object
- **retryable**: Boolean flag for client retry logic
- **request_id**: Trace ID for debugging

### 8.3 Retry and Resilience Patterns

**Exponential Backoff**:
- Initial delay: 100ms
- Maximum delay: 32 seconds
- Backoff multiplier: 2.0
- Jitter: ±25% randomization to prevent thundering herd
- Maximum retry attempts: 5

**Circuit Breaker States**:
- **Closed**: Normal operation, requests pass through
- **Open**: Threshold failures exceeded (e.g., 5 failures in 60s), requests rejected immediately
- **Half-Open**: Recovery testing, limited requests allowed

**Fallback Strategies**:
- Cached results for recommendation failures
- Degraded service mode with reduced features
- User notification for prolonged outages
- Graceful degradation without complete failure

### 8.4 Logging and Observability

**Structured Logging**:
- JSON-formatted log entries
- Trace ID propagation across services
- Log levels: DEBUG, INFO, WARN, ERROR, FATAL
- Centralized log aggregation in BigQuery

**Error Tracking**:
- Error rate metrics per service/endpoint
- Alert triggers: >1% error rate or P99 latency >1s
- Distributed tracing via Cloud Trace
- Request flow visualization

**Performance Monitoring**:
- Latency percentiles (P50, P90, P95, P99)
- Throughput metrics (requests per second)
- Resource utilization (CPU, memory, network)
- Custom business metrics (recommendation quality, trust scores)

---

## 9. Industry Standards Referenced

### 9.1 Authentication and Security Standards

**OAuth 2.0 Evolution**:
- **RFC 6749**: OAuth 2.0 Authorization Framework (original specification)
- **RFC 7636**: PKCE (Proof Key for Code Exchange) for public clients
- **RFC 8628**: Device Authorization Grant for browserless/input-constrained devices
- **RFC 9700**: OAuth 2.0 Security Best Current Practice (January 2025) - deprecates Implicit and Password Credentials grants

**OpenID Connect**:
- **OIDC Core**: Identity layer on top of OAuth 2.0
- Enterprise SSO integration (Netflix via Okta)
- ID tokens with user claims
- Discovery and dynamic registration

### 9.2 Content Identification Standards

**EIDR (Entertainment Identifier Registry)**:
- ISO 26324 standard for content identification
- Hierarchical relationship support
- Cross-platform identifier resolution
- Links to IMDb, Rotten Tomatoes, Common Sense Media

**Gracenote TMS**:
- Nielsen-owned proprietary metadata system
- 105,000+ unique titles across 85+ countries (Q3 2025)
- Theme, genre, mood, keyword taxonomies
- Content Connect Platform for CTV advertising

**TMDb (The Movie Database)**:
- Community-built open metadata database
- Public API for third-party applications
- Cross-referencing with EIDR and Gracenote

### 9.3 Geographic and Language Standards

**ISO 3166-1 Alpha-3**: Country codes for regional availability
**UN M49**: Regional codes for geographic grouping
**ISO 639-2**: Three-letter language codes for content metadata

### 9.4 Communication Protocols

**gRPC**:
- HTTP/2-based RPC framework with Protocol Buffers
- Streaming RPC for progressive result delivery
- Service definitions for discovery and orchestration
- Tonic implementation in Rust

**WebSocket**:
- Bidirectional real-time communication
- Device sync API for watch progress and preferences
- Presence channels for device status

**Model Context Protocol (MCP)**:
- Standardized agent-tool communication
- STDIO and SSE transport support
- Tool discovery and schema validation
- Request/response and streaming patterns

### 9.5 Data Formats and Serialization

**Protocol Buffers (Protobuf)**:
- Efficient binary serialization
- Strong typing with schema evolution
- gRPC service definitions
- Cross-language compatibility

**TOML**:
- Configuration file format for CLI profiles
- Human-readable with strong typing
- Environment variable override support

**JSON/YAML**:
- API request/response formats
- Configuration alternatives
- Output format options for CLI

### 9.6 Privacy and Compliance Standards

**GDPR (General Data Protection Regulation)**:
- EU regulation for data protection and privacy
- Explicit consent requirements
- Rights: access, deletion, portability
- Global applicability for EU residents

**CCPA (California Consumer Privacy Act)**:
- California state privacy law
- Rights to know, opt-out, deletion
- Broad definition of personal information
- 2025 enforcement with significant settlements

**VPPA (Video Privacy Protection Act)**:
- US federal law protecting video viewing data
- Explicit consent required for collection
- 2025 litigation wave for embedded players and pixels

**Differential Privacy**:
- Mathematical privacy guarantee (ε, δ parameters)
- Federated learning with ε=1.0, δ=1e-5
- Noise injection in gradient updates
- K-anonymity with 1000+ user minimum thresholds

---

## 10. Constraints and Assumptions

### 10.1 Platform API Constraints

**Critical Limitation**: "8 out of 10 platforms offer no public API for consumer apps"

**Implication**: Media Gateway CANNOT directly access:
- User watch history from streaming platforms
- Platform-specific recommendations
- Real-time availability status from platforms directly
- Individual user account data

**Workaround Strategy**:
- Aggregate public metadata from third-party APIs
- Store user behavioral data locally with encryption
- Use deep links for playback (not direct streaming)
- Federated learning for privacy-safe personalization without cloud transmission

### 10.2 Regional Licensing Constraints

**Content Availability Variability**:
- Licensing agreements differ by country/region
- Temporal windows for content availability (expiry dates)
- Platform-specific regional restrictions (BBC iPlayer UK-only, Crave Canada-only)

**Metadata Granularity**:
- Watchmode Tier 1: USA, UK, Canada, Australia, India, Germany, Belgium, Netherlands, New Zealand, Spain, Brazil
- Watchmode Tier 2: Additional 40+ countries with varying metadata completeness
- Regional pricing differences for subscriptions/rentals

**System Design Impact**:
- ISO 3166-1 alpha-3 country codes required
- Availability tracking with Unix timestamp expiry
- Trust score decay for unverified regional data

### 10.3 Third-Party Dependency Assumptions

**Aggregator API Assumptions**:
- Daily to weekly catalog updates (data freshness)
- Subscription-based pricing tiers with rate limits
- Coverage of 150+ streaming services across 60+ countries
- Deep link accuracy and maintenance

**Dependency Risk**:
- Firebase Dynamic Links deprecated (August 2025) → Migration to native platform deep linking
- Aggregator service availability and uptime
- API schema changes requiring adapter updates

**Mitigation**:
- Multiple aggregator integration for redundancy
- Circuit breaker patterns for service failures
- Cached fallback data for temporary outages

### 10.4 Performance Constraints

**Latency Budgets**:
- Total query latency target: <500ms end-to-end
- First result progressive rendering: <200ms
- SONA intelligence pipeline: <50ms (P99)
- Trust validation overhead: <50ms per request

**Resource Limitations**:
- Per-user SONA adapter: ~5MB cache overhead
- 10GB cache for 1M active users
- Kubernetes HPA scaling 3-20 replicas (cost constraints)

**Scalability Assumptions**:
- GKE Autopilot scaling for Layer 1-3 services
- Cloud Run auto-scaling for stateless components
- Memorystore Valkey (10GB) for distributed caching
- Estimated 100K concurrent users for production readiness

### 10.5 Privacy and Security Constraints

**User Data Restrictions**:
- MUST NOT transmit raw watch history to servers
- MUST implement differential privacy (ε=1.0, δ=1e-5) for federated learning
- MUST maintain k-anonymity with 1000+ user minimum thresholds
- MUST provide explicit opt-out mechanisms (CCPA/GDPR compliance)

**OAuth Security Requirements** (RFC 9700):
- MUST use Authorization Code + PKCE (NOT Implicit Grant)
- MUST implement refresh token rotation
- MUST use short-lived access tokens (15-60 minutes)
- SHOULD implement MFA for critical operations

**Data Retention Assumptions**:
- On-device encrypted storage with user-controlled deletion
- Server-side aggregated data with 30-day TTL for inactive users
- ReasoningBank pattern storage with relevance-based pruning

### 10.6 Development and Deployment Constraints

**Technology Stack Assumptions**:
- 100% Rust implementation for production services
- TypeScript for development tooling (hackathon-tv5)
- GCP as exclusive cloud provider
- Kubernetes (GKE Autopilot) for orchestration

**Repository Organization**:
- 51 independent micro-repositories
- Semantic versioning with Bill of Materials (BOM)
- Quarterly major releases, bi-weekly minor releases
- Two major version backward compatibility

**Cost Constraints**:
- Estimated monthly GCP infrastructure: $1,855–$2,605
- E2B sandbox compute: $250-450/month
- Total estimated: $2,400–$3,650/month
- Optimization via Spot VMs (60-80% savings), committed use discounts (37-55%)

### 10.7 Agent Orchestration Assumptions

**SPARC Methodology Requirements**:
- Five-phase execution: Specification → Pseudocode → Architecture → Refinement → Completion
- Nine specialized agents: RequirementsAnalyst, QueryPlanner, ContentSearcher, RecommendationBuilder, AvailabilityChecker, DeviceCoordinator, TrustScorer, QualityAssurer, ResponseFormatter

**Claude-Flow Integration**:
- MCP protocol for agent-tool communication
- Pre/post hooks for lifecycle management
- Session state management via AgentDB (Redis)
- Coordination memory namespace (`aqe/*`)

**E2B Sandbox Assumptions**:
- ~150ms Firecracker microVM startup
- ~3,800 daily sandbox executions
- ~1,000 monthly compute hours
- Hardware-level isolation for untrusted code

### 10.8 Data Architecture Assumptions

**Ruvector Capabilities**:
- Hypergraph + vector + GNN combined engine
- HNSW indexes for similarity search
- Cypher-compatible query language
- Raft consensus for distributed operations

**Embedding Dimensions**:
- Content nodes: 1536-dim (OpenAI text-embedding-3-large)
- Person nodes: 768-dim (sentence-transformers)
- User preferences: 512-dim (privacy-conscious)
- Classification: 256-dim

**Storage Estimates**:
- ~40GB embeddings for 1M movies, 500K people, 10M episodes
- ~2x HNSW index overhead
- Total ~80GB for full catalog

### 10.9 User Experience Assumptions

**Progressive Rendering**:
- Results display after 200ms initial delay
- Batch updates every 100ms or 5 items
- Scroll position locking to prevent jumping

**Cross-Platform Consistency**:
- Unified API contracts across web, mobile, TV, CLI
- CRDT synchronization for offline-first experiences
- Deep linking for platform-agnostic playback

**Authentication UX**:
- OAuth 2.0 + PKCE for web/mobile (seamless browser flow)
- Device Authorization Grant for TV/CLI (QR code pairing)
- Subscription declaration (no direct platform authentication)

### 10.10 Testing and Quality Assumptions

**Quality Metrics**:
- Precision@10: 0.31 target (baseline 0.26)
- NDCG@10: 0.63 target (baseline 0.54)
- Cold-start improvement: 47% after 5 interactions

**Load Testing**:
- 100K concurrent users for production readiness
- <50ms P99 latency under load
- 12,000 RPS recommendation throughput with caching

**Trust Thresholds**:
- Minimum 0.6 trust score for recommendation inclusion
- 90%+ metadata completeness for high trust
- Cross-source validation across multiple aggregators

---

## 11. Strategic Recommendations

Based on comprehensive analysis of the media-gateway-research repository, the following strategic recommendations establish the foundation for SPARC Specification:

### 11.1 Architecture Strategy

**Adopt 4-Layer Microservices Architecture**:
- Layer 1: Data ingestion with MCP connectors (20 micro-repos)
- Layer 2: Intelligence with SONA and multi-agent orchestration (8 repos)
- Layer 3: Consolidation with unified metadata and availability (5 repos)
- Layer 4: Applications across web, mobile, TV, CLI (6 repos)

**Use Ruvector as Central Data Engine**:
- Hypergraph for complex content relationships
- Vector embeddings for semantic search
- GNN layers for personalized recommendations
- HNSW indexes for similarity performance

### 11.2 Integration Strategy

**Prioritize Third-Party Aggregators Over Direct Platform APIs**:
- Streaming Availability API, Watchmode API, International Showtimes API
- Provides 150+ service coverage without partnership negotiations
- Pre-built metadata and deep linking
- Faster time-to-market than direct integrations

**Implement Standardized MCP Connector Framework**:
- Uniform Rust trait for all platform integrations
- Cursor-based pagination for catalog fetching
- Configurable rate limiting and retry logic
- Health check endpoints for monitoring

### 11.3 Intelligence Strategy

**Deploy SONA with Two-Tier LoRA**:
- Per-user adapters (~5MB) for runtime personalization
- FastGRNN semantic routing (<0.5ms)
- 39 dynamic attention mechanisms
- ReasoningBank for pattern storage and boosting

**Implement Hybrid Recommendation Engine**:
- Collaborative filtering (35% weight)
- Content-based similarity (25%)
- GraphSAGE GNN (30%)
- Context-aware filtering (10%)
- Reciprocal Rank Fusion with MMR diversity

### 11.4 Privacy and Security Strategy

**Adopt Three-Tier Privacy Architecture**:
- Tier 1: On-device encrypted storage (no cloud transmission)
- Tier 2: Federated learning with differential privacy (ε=1.0, δ=1e-5)
- Tier 3: Public metadata only

**Implement Modern OAuth 2.0 (RFC 9700)**:
- Authorization Code + PKCE for all clients
- Device Authorization Grant for TV/CLI
- Short-lived tokens with refresh rotation
- Secure platform-specific storage (Keychain, Keystore)

### 11.5 Performance Strategy

**Target Sub-500ms End-to-End Latency**:
- Progressive rendering after 200ms
- P99 recommendation latency <124ms
- SONA pipeline <50ms (P99)
- Trust validation <50ms overhead

**Optimize for 12,000 RPS Throughput**:
- Kubernetes HPA with 70% CPU / 80% memory targets
- Memorystore Valkey (10GB) for distributed caching
- Cloud Run auto-scaling for stateless components

### 11.6 Deployment Strategy

**Use GCP with Hybrid Containerization**:
- GKE Autopilot for stateful microservices (Layers 1-3)
- Cloud Run for stateless APIs (Layer 4)
- Cloud SQL PostgreSQL with regional HA
- Terraform IaC with environment isolation

**Implement Cost Optimization**:
- Spot VMs for fault-tolerant workloads (60-80% savings)
- Committed use discounts (37-55% reduction)
- Aggressive right-sizing based on telemetry
- Target $2,400–$3,650/month total infrastructure

### 11.7 Quality Strategy

**Establish Trust Scoring Framework**:
- 5-component weighted system
- 0.6 minimum threshold for recommendations
- Exponential time decay for unverified data
- Cross-source validation

**Define Quality Metrics**:
- Precision@10 ≥0.31
- NDCG@10 ≥0.63
- 47% cold-start improvement after 5 interactions
- 90%+ metadata completeness

### 11.8 Development Strategy

**Follow SPARC Methodology**:
- Specification → Pseudocode → Architecture → Refinement → Completion
- 9 specialized agents for orchestration
- Claude-Flow with MCP protocol
- Pre/post hooks for lifecycle management

**Maintain 51-Repository Organization**:
- Independent versioning and deployment
- Semantic versioning with BOM tracking
- Quarterly major releases, bi-weekly minor updates
- Two major version backward compatibility

---

## 12. Research Summary and Next Steps

### 12.1 Key Findings

This comprehensive domain knowledge research establishes Media Gateway as a **sophisticated, privacy-first, AI-driven TV discovery platform** solving the "45-minute decision problem" through:

1. **Unified Discovery**: Aggregating 150+ streaming services across 60+ countries via third-party APIs
2. **Intelligent Recommendations**: SONA neural architecture with 39 attention mechanisms and Two-Tier LoRA personalization
3. **Privacy-Safe Personalization**: Three-tier architecture with federated learning and differential privacy
4. **Cross-Platform Experience**: Web, mobile, TV, CLI with CRDT-based synchronization
5. **Production-Grade Infrastructure**: GCP deployment with Kubernetes, Cloud Run, and comprehensive observability

### 12.2 Authoritative Domain Context Established

**Domain Concepts**: ARW protocol, SONA intelligence, Ruvector hypergraph, Two-Tier LoRA, Tiny Dancer routing, ReasoningBank patterns, CRDT synchronization, trust scoring framework

**Technical Standards**: OAuth 2.0 RFC 9700, EIDR/Gracenote identifiers, ISO country/language codes, MCP protocol, gRPC/Protobuf, GDPR/CCPA/VPPA compliance

**Architecture Patterns**: 4-layer microservices, 51-repository organization, hybrid recommendation engine, three-tier privacy, event-driven Kafka integration

**Performance Targets**: <500ms latency, 12,000 RPS throughput, <50ms SONA pipeline, 0.31 Precision@10, 0.63 NDCG@10

**Constraints**: 8/10 platforms lack public APIs, regional licensing variability, privacy regulation compliance, $2,400–$3,650/month cost target

### 12.3 SPARC Specification Phase Readiness

This domain knowledge research provides the authoritative foundation for proceeding with the SPARC Specification phase. All critical domain concepts, technical requirements, industry standards, performance targets, and constraints have been extracted and documented from the media-gateway-research repository.

**Next Phase**: Use this domain knowledge to create detailed specifications for Media Gateway implementation, ensuring alignment with established architectural patterns, performance benchmarks, and regulatory requirements.

---

**Research Completion**: 2025-12-06
**Source Repository**: https://github.com/globalbusinessadvisors/media-gateway-research
**Documentation Files Analyzed**: 26 files across architecture, integration, deployment, and specifications
**Research Agent**: Domain Knowledge Research Specialist (SPARC Specification Phase)
