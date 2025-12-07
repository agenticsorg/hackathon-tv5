# SPARC Specification — Part 4 of 4

## Media Gateway: Unified Cross-Platform TV Discovery Engine

**Document Version:** 1.0.0
**SPARC Phase:** Specification
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents — Part 4

15. [Service Expectations](#15-service-expectations)
16. [Agent Orchestration Goals](#16-agent-orchestration-goals)
17. [Authentication Constraints](#17-authentication-constraints)
18. [Error Cases](#18-error-cases)
19. [Performance Requirements](#19-performance-requirements)
20. [Constraints and Assumptions](#20-constraints-and-assumptions)
21. [Non-Functional Requirements](#21-non-functional-requirements)
22. [Appendix: Glossary](#22-appendix-glossary)

---

## 15. Service Expectations

### 15.1 Core Services Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    CORE SERVICES ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      API GATEWAY (Cloud Run)                     │   │
│  │  • Rate limiting • Authentication • Request routing             │   │
│  └───────────────────────────────┬─────────────────────────────────┘   │
│                                  │                                       │
│  ┌───────────────────────────────┼───────────────────────────────────┐  │
│  │                   SERVICE MESH (GKE Autopilot)                    │  │
│  │                                                                    │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │  │
│  │  │   Search    │  │ Recommend   │  │    Auth     │               │  │
│  │  │   Service   │  │  Service    │  │   Service   │               │  │
│  │  │             │  │             │  │             │               │  │
│  │  │ • SONA      │  │ • ML models │  │ • OAuth 2.0 │               │  │
│  │  │ • Ruvector  │  │ • LoRA      │  │ • PKCE      │               │  │
│  │  │ • Intent    │  │ • Context   │  │ • Device    │               │  │
│  │  │   parsing   │  │   aware     │  │   grant     │               │  │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘               │  │
│  │         │                │                │                       │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │  │
│  │  │  Ingestion  │  │   Device    │  │    Sync     │               │  │
│  │  │   Service   │  │   Service   │  │   Service   │               │  │
│  │  │             │  │             │  │             │               │  │
│  │  │ • Platform  │  │ • Registry  │  │ • PubNub    │               │  │
│  │  │   fetchers  │  │ • Heartbeat │  │ • CRDT      │               │  │
│  │  │ • Entity    │  │ • Remote    │  │ • Conflict  │               │  │
│  │  │   resolver  │  │   control   │  │   resolve   │               │  │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘               │  │
│  │         │                │                │                       │  │
│  │         └────────────────┼────────────────┘                       │  │
│  │                          │                                        │  │
│  └──────────────────────────┼────────────────────────────────────────┘  │
│                             │                                            │
│  ┌──────────────────────────┼────────────────────────────────────────┐  │
│  │                    DATA LAYER                                      │  │
│  │                                                                    │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │  │
│  │  │ PostgreSQL  │  │   Valkey    │  │   PubNub    │               │  │
│  │  │ + pg_vector │  │   Cache     │  │   Realtime  │               │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘               │  │
│  │                                                                    │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 15.2 Service Contracts

#### 15.2.1 Search Service

```rust
/// Search Service Contract
pub trait SearchService: Send + Sync {
    /// Execute natural language search
    async fn search(
        &self,
        request: SearchRequest,
    ) -> Result<SearchResponse, ServiceError>;

    /// Get content details by entity ID
    async fn get_content(
        &self,
        entity_id: &str,
        options: GetContentOptions,
    ) -> Result<Content, ServiceError>;

    /// Get trending content
    async fn get_trending(
        &self,
        region: &str,
        limit: usize,
    ) -> Result<Vec<TrendingContent>, ServiceError>;
}

pub struct SearchRequest {
    pub query: String,
    pub user_id: Option<String>,
    pub filters: SearchFilters,
    pub limit: usize,
    pub offset: usize,
}

pub struct SearchFilters {
    pub content_type: Option<ContentType>,
    pub genres: Option<Vec<String>>,
    pub platforms: Option<Vec<String>>,
    pub release_year_range: Option<(u16, u16)>,
    pub rating_min: Option<f32>,
    pub region: String,
}

// SLOs
// - Latency: p50 < 100ms, p95 < 500ms, p99 < 1s
// - Throughput: 1000 RPS
// - Availability: 99.9%
```

#### 15.2.2 Recommendation Service

```rust
/// Recommendation Service Contract
pub trait RecommendationService: Send + Sync {
    /// Get personalized recommendations
    async fn recommend(
        &self,
        request: RecommendRequest,
    ) -> Result<Vec<Recommendation>, ServiceError>;

    /// Get similar content
    async fn similar(
        &self,
        entity_id: &str,
        limit: usize,
    ) -> Result<Vec<SimilarContent>, ServiceError>;

    /// Update user preferences (for SONA learning)
    async fn update_preferences(
        &self,
        user_id: &str,
        event: PreferenceEvent,
    ) -> Result<(), ServiceError>;
}

pub struct RecommendRequest {
    pub user_id: String,
    pub context: Option<String>,      // "family movie night"
    pub mood: Option<String>,          // "relaxing"
    pub age_appropriate: Option<Vec<u8>>,
    pub exclude_watched: bool,
    pub platforms: Option<Vec<String>>,
    pub limit: usize,
}

pub enum PreferenceEvent {
    Watched { entity_id: String, completed: bool, rating: Option<f32> },
    Watchlisted { entity_id: String },
    Dismissed { entity_id: String, reason: Option<String> },
    SearchClicked { entity_id: String, query: String },
}

// SLOs
// - SONA personalization latency: < 5ms
// - Cold-start: meaningful recommendations within 3 interactions
// - Precision@10: >= 0.31
// - NDCG@10: >= 0.63
```

#### 15.2.3 Device Service

```rust
/// Device Service Contract
pub trait DeviceService: Send + Sync {
    /// Register a new device
    async fn register(
        &self,
        user_id: &str,
        device: DeviceRegistration,
    ) -> Result<Device, ServiceError>;

    /// List user's devices
    async fn list_devices(
        &self,
        user_id: &str,
        status_filter: Option<DeviceStatus>,
    ) -> Result<Vec<Device>, ServiceError>;

    /// Send command to device
    async fn send_command(
        &self,
        user_id: &str,
        command: DeviceCommand,
    ) -> Result<CommandAck, ServiceError>;

    /// Update device status (heartbeat)
    async fn heartbeat(
        &self,
        device_id: &str,
        status: DeviceHeartbeat,
    ) -> Result<(), ServiceError>;
}

pub struct DeviceCommand {
    pub command_type: CommandType,
    pub target_device_id: String,
    pub params: CommandParams,
    pub request_id: String,
}

pub enum CommandType {
    Cast { entity_id: String, platform: String },
    Play,
    Pause,
    Seek { seconds: u32 },
    Stop,
    Volume { level: u8 },
}

// SLOs
// - Remote control latency: p50 < 50ms, p99 < 100ms
// - Device presence accuracy: > 99%
// - Command success rate: > 99.5%
```

### 15.3 Service Lifecycle Management

```yaml
lifecycle:
  startup:
    sequence:
      1. Load configuration from Secret Manager
      2. Initialize database connections (with retry)
      3. Connect to cache (Valkey)
      4. Register with service mesh
      5. Start health check endpoint
      6. Begin accepting traffic

    health_check:
      endpoint: /health
      liveness: /health/live
      readiness: /health/ready

    graceful_start:
      initial_delay: 10s
      traffic_ramp: 30s

  shutdown:
    sequence:
      1. Stop accepting new requests
      2. Complete in-flight requests (30s timeout)
      3. Flush metrics and logs
      4. Close database connections
      5. Deregister from service mesh
      6. Exit

    graceful_period: 30s
    force_kill_after: 60s

  rolling_update:
    max_surge: 25%
    max_unavailable: 0
    min_ready_seconds: 30

  scaling:
    min_replicas: 2
    max_replicas: 100
    target_cpu: 70%
    target_memory: 80%
    scale_down_stabilization: 300s
```

### 15.4 Inter-Service Communication

```yaml
communication:
  sync:
    protocol: gRPC
    format: Protocol Buffers
    tls: mutual TLS (mTLS)
    timeout_default: 5s
    retry:
      max_attempts: 3
      backoff: exponential (100ms base, 2x multiplier)
      retryable_codes: [UNAVAILABLE, RESOURCE_EXHAUSTED]

  async:
    protocol: Google Pub/Sub
    format: JSON (with schema validation)
    topics:
      - content.ingested
      - content.updated
      - availability.changed
      - user.preference.updated
      - device.status.changed

  realtime:
    protocol: PubNub
    use_cases:
      - cross-device sync
      - remote control
      - notifications
```

---

## 16. Agent Orchestration Goals

### 16.1 AI Agent Integration Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    AI AGENT INTEGRATION                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    EXTERNAL AI AGENTS                            │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐            │   │
│  │  │ Claude  │  │  GPT-4  │  │ Gemini  │  │ Custom  │            │   │
│  │  │ Desktop │  │         │  │         │  │ Agents  │            │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘            │   │
│  │       │            │            │            │                   │   │
│  └───────┼────────────┼────────────┼────────────┼───────────────────┘   │
│          │            │            │            │                       │
│          ▼            ▼            ▼            ▼                       │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    MCP GATEWAY                                   │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │   │
│  │  │   STDIO     │  │    SSE      │  │    ARW      │             │   │
│  │  │  Transport  │  │  Transport  │  │  Discovery  │             │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘             │   │
│  │                                                                  │   │
│  │  ┌──────────────────────────────────────────────────────────┐   │   │
│  │  │                    MCP TOOLS (10)                         │   │   │
│  │  │  • semantic_search    • get_recommendations              │   │   │
│  │  │  • get_content        • list_devices                     │   │   │
│  │  │  • discover_content   • get_device_status                │   │   │
│  │  │  • initiate_playback  • update_preferences               │   │   │
│  │  │  • control_playback   • get_genres                       │   │   │
│  │  └──────────────────────────────────────────────────────────┘   │   │
│  │                                                                  │   │
│  └──────────────────────────────┬───────────────────────────────────┘   │
│                                 │                                        │
│  ┌──────────────────────────────┼────────────────────────────────────┐  │
│  │                    INTERNAL AGENTS                                 │  │
│  │                                                                    │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │  │
│  │  │  Ingestion  │  │ Recommender │  │  Anomaly    │               │  │
│  │  │    Agent    │  │    Agent    │  │  Detection  │               │  │
│  │  │             │  │             │  │    Agent    │               │  │
│  │  │ • Catalog   │  │ • Cold-start│  │ • Fraud     │               │  │
│  │  │   refresh   │  │   solving   │  │   detection │               │  │
│  │  │ • Entity    │  │ • Diversity │  │ • Abuse     │               │  │
│  │  │   resolution│  │   injection │  │   prevention│               │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘               │  │
│  │                                                                    │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 16.2 Autonomous Operation Modes

```yaml
autonomous_modes:
  catalog_maintenance:
    description: "Continuous catalog updates without human intervention"
    triggers:
      - scheduled (every 6 hours)
      - webhook (platform notifications)
      - anomaly (missing content detected)
    actions:
      - Fetch delta from aggregator APIs
      - Resolve entities
      - Update embeddings
      - Refresh availability
      - Publish update events
    guardrails:
      - max_updates_per_run: 10000
      - require_confirmation_for_deletions: true
      - alert_on_major_changes: > 5% catalog change

  recommendation_optimization:
    description: "Self-improving recommendation models"
    triggers:
      - scheduled (nightly)
      - accumulated_feedback: > 1000 events
    actions:
      - Analyze click-through rates
      - Adjust SONA weights
      - Retrain LoRA adapters
      - A/B test new strategies
    guardrails:
      - max_model_drift: 10%
      - require_validation_before_deploy: true
      - rollback_on_quality_drop: true

  cache_optimization:
    description: "Adaptive caching based on access patterns"
    triggers:
      - scheduled (hourly)
      - cache_hit_rate < 70%
    actions:
      - Analyze access patterns
      - Adjust TTLs
      - Pre-warm popular content
      - Evict stale entries
    guardrails:
      - max_cache_size: 16GB
      - min_ttl: 1 minute
      - max_ttl: 7 days
```

### 16.3 Human-in-the-Loop Scenarios

```yaml
human_in_the_loop:
  content_moderation:
    trigger: "Content flagged by ML model"
    workflow:
      1. AI flags content for review
      2. Notification to moderation queue
      3. Human reviews within SLA (4 hours)
      4. Human approves/rejects/escalates
      5. AI learns from decision
    escalation:
      - After 4 hours: notify on-call
      - After 8 hours: auto-approve if low-risk

  ambiguous_search:
    trigger: "Query intent unclear (confidence < 0.7)"
    workflow:
      1. AI presents clarifying options to user
      2. User selects or provides more context
      3. AI refines search with user input
    example:
      query: "matrix"
      clarification: "Did you mean: 1) The Matrix (1999 film), 2) The Matrix Resurrections, 3) Matrix-related content?"

  rights_verification:
    trigger: "Content availability uncertain"
    workflow:
      1. AI detects potential rights conflict
      2. Queue for manual verification
      3. Human verifies with platform
      4. Update availability database
    sla: 24 hours

  new_platform_onboarding:
    trigger: "New streaming platform detected"
    workflow:
      1. AI identifies new platform
      2. Creates normalizer template
      3. Human reviews API contracts
      4. Human approves integration
      5. AI activates and monitors
```

### 16.4 Multi-Agent Coordination

```yaml
multi_agent_coordination:
  topology: hierarchical

  agents:
    orchestrator:
      role: "Coordinate all agents, manage priorities"
      capabilities:
        - task_assignment
        - progress_monitoring
        - conflict_resolution
        - resource_allocation

    researchers:
      count: 3
      role: "Gather information from external sources"
      specializations:
        - platform_apis
        - metadata_enrichment
        - trend_analysis

    processors:
      count: 5
      role: "Transform and index data"
      specializations:
        - embedding_generation
        - entity_resolution
        - graph_construction
        - availability_tracking
        - quality_assurance

    responders:
      count: 10
      role: "Handle user queries"
      specializations:
        - search
        - recommendations
        - device_control

  communication:
    protocol: "Message queue (Pub/Sub)"
    patterns:
      - request_response (sync operations)
      - fire_and_forget (async updates)
      - broadcast (coordination signals)

  memory_namespace: "aqe/*"
  coordination_hooks:
    pre_task: "npx claude-flow@alpha hooks pre-task"
    post_task: "npx claude-flow@alpha hooks post-task"
```

---

## 17. Authentication Constraints

### 17.1 User Authentication

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    AUTHENTICATION FLOWS                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  WEB/MOBILE: OAuth 2.0 + PKCE (RFC 7636)                               │
│  ─────────────────────────────────────────                              │
│                                                                          │
│  1. App generates code_verifier (random 43-128 chars)                  │
│  2. App computes code_challenge = BASE64URL(SHA256(code_verifier))     │
│  3. App redirects to /authorize with:                                   │
│     - client_id                                                         │
│     - redirect_uri                                                      │
│     - response_type=code                                                │
│     - scope=read:content write:preferences                             │
│     - code_challenge                                                    │
│     - code_challenge_method=S256                                       │
│     - state (CSRF protection)                                          │
│  4. User authenticates (email/password, social login)                  │
│  5. Server redirects with authorization code                           │
│  6. App exchanges code for tokens (with code_verifier)                 │
│  7. Server validates code_verifier, returns tokens                     │
│                                                                          │
│  TV/CLI: Device Authorization Grant (RFC 8628)                         │
│  ───────────────────────────────────────────────                        │
│                                                                          │
│  1. Device requests device code from /device/code                      │
│  2. Server returns:                                                     │
│     - device_code (for polling)                                        │
│     - user_code (for display)                                          │
│     - verification_uri                                                  │
│     - expires_in (15 minutes)                                          │
│     - interval (5 seconds)                                             │
│  3. Device displays user_code and verification_uri                     │
│  4. User visits URL on phone/computer, enters code                     │
│  5. User authenticates and approves                                    │
│  6. Device polls /token with device_code                               │
│  7. After approval, server returns access_token and refresh_token      │
│                                                                          │
│  Token Specifications:                                                  │
│  ─────────────────────                                                  │
│  • Access token: JWT, 1 hour expiry                                    │
│  • Refresh token: Opaque, 30 day expiry, rotate on use                │
│  • ID token: JWT, user profile claims                                  │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 17.2 Service-to-Service Authentication

```yaml
service_auth:
  internal:
    method: Workload Identity (GCP)
    mechanism:
      - Services run with Kubernetes service accounts
      - Service accounts mapped to GCP IAM service accounts
      - mTLS for all internal communication
    verification:
      - Verify JWT from metadata server
      - Check audience claim matches target service

  external:
    method: API Key + OAuth (for partners)
    api_keys:
      - Stored in Secret Manager
      - Scoped to specific endpoints
      - Rate limited per key
    oauth:
      - Client credentials grant
      - Machine-to-machine communication
```

### 17.3 Authorization Model

```yaml
authorization:
  model: RBAC + Resource-level permissions

  roles:
    anonymous:
      permissions:
        - search:read (limited, rate-throttled)
        - trending:read
        - genres:read

    free_user:
      permissions:
        - search:read
        - recommendations:read (limited)
        - watchlist:read, watchlist:write
        - preferences:read, preferences:write
        - devices:read, devices:write (max 2)

    premium_user:
      permissions:
        - all free_user permissions
        - recommendations:read (unlimited)
        - devices:read, devices:write (max 10)
        - priority:search (faster, higher limits)
        - history:read, history:export

    admin:
      permissions:
        - all permissions
        - users:manage
        - content:moderate
        - system:configure

  resource_permissions:
    watchlist:
      owner: read, write, delete
      shared_with: read
      public: none

    device:
      owner: full control
      household: read, limited control
      public: none

  scopes:
    - read:content
    - read:preferences
    - write:preferences
    - read:devices
    - write:devices
    - write:playback
    - read:history
    - offline_access
```

### 17.4 Platform Token Management

```yaml
platform_tokens:
  youtube:
    oauth_scopes:
      - youtube.readonly
    refresh_strategy: automatic (before expiry)
    storage: encrypted in user profile
    revocation_url: https://myaccount.google.com/permissions

  aggregator_apis:
    streaming_availability:
      auth: API key (header)
      rotation: manual (on compromise)
      storage: Secret Manager
    watchmode:
      auth: API key (query param)
      storage: Secret Manager

  security:
    encryption: AES-256-GCM
    key_derivation: Argon2id (user password)
    at_rest: Cloud SQL encryption
    in_transit: TLS 1.3

  revocation:
    immediate: Token deleted from store
    propagation: PubNub notification (< 5 seconds)
    verification: Real-time token validation
```

---

## 18. Error Cases

### 18.1 Error Taxonomy

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    ERROR TAXONOMY                                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  CLIENT ERRORS (4xx)                                                    │
│  ───────────────────                                                    │
│  400 Bad Request        Invalid input, malformed JSON                  │
│  401 Unauthorized       Missing or invalid authentication               │
│  403 Forbidden          Valid auth but insufficient permissions         │
│  404 Not Found          Resource doesn't exist                          │
│  409 Conflict           State conflict (e.g., concurrent edit)         │
│  422 Unprocessable      Validation failed                               │
│  429 Too Many Requests  Rate limit exceeded                             │
│                                                                          │
│  SERVER ERRORS (5xx)                                                    │
│  ───────────────────                                                    │
│  500 Internal Error     Unexpected server error                         │
│  502 Bad Gateway        Upstream service unavailable                    │
│  503 Service Unavailable Server overloaded or in maintenance          │
│  504 Gateway Timeout    Upstream service timeout                        │
│                                                                          │
│  DOMAIN ERRORS                                                          │
│  ─────────────                                                          │
│  CONTENT_NOT_FOUND      Content ID doesn't exist                       │
│  PLATFORM_UNAVAILABLE   Platform not available in region               │
│  DEVICE_OFFLINE         Target device not reachable                    │
│  SYNC_CONFLICT          CRDT merge conflict (should auto-resolve)      │
│  QUOTA_EXCEEDED         Platform API quota exhausted                   │
│  EMBEDDING_FAILED       Failed to generate embedding                   │
│  INTENT_UNCLEAR         Natural language query not understood          │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 18.2 Error Response Format

```typescript
interface ErrorResponse {
  error: {
    code: string;           // e.g., "CONTENT_NOT_FOUND"
    message: string;        // Human-readable message
    details?: {
      field?: string;       // For validation errors
      reason?: string;      // Additional context
      suggestion?: string;  // How to fix
    };
    request_id: string;     // For debugging
    timestamp: string;      // ISO 8601
  };
}

// Example
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "Rate limit exceeded. Please retry after 60 seconds.",
    "details": {
      "limit": 100,
      "window": "15 minutes",
      "retry_after": 60,
      "suggestion": "Consider upgrading to premium for higher limits."
    },
    "request_id": "req-abc123",
    "timestamp": "2025-12-06T10:30:00Z"
  }
}
```

### 18.3 Graceful Degradation Patterns

```yaml
degradation:
  search_service_down:
    detection: Health check fails for > 30s
    response:
      - Return cached trending content
      - Show "Search temporarily unavailable" message
      - Offer category browsing as alternative
    recovery: Automatic when health check passes

  recommendation_service_down:
    detection: gRPC errors > 50% in 1 minute
    response:
      - Fall back to popularity-based recommendations
      - Disable personalization temporarily
      - Log degradation event
    recovery: Gradual traffic shift back

  aggregator_api_quota_exceeded:
    detection: 429 response from API
    response:
      - Serve from cache (even if slightly stale)
      - Reduce refresh frequency
      - Rotate to backup API key if available
    recovery: Reset at quota window boundary

  pubnub_connection_lost:
    detection: No heartbeat for > 60s
    response:
      - Queue state changes locally
      - Continue with optimistic UI updates
      - Show "Syncing..." indicator
    recovery: Replay queued changes on reconnect

  database_connection_pool_exhausted:
    detection: Connection timeout > 5s
    response:
      - Reject new requests with 503
      - Prioritize read operations
      - Shed load on non-critical endpoints
    recovery: Increase pool size, investigate leak
```

### 18.4 Retry Strategies

```yaml
retry:
  default:
    max_attempts: 3
    initial_delay: 100ms
    max_delay: 10s
    multiplier: 2.0
    jitter: 0.1
    retryable_codes: [408, 429, 500, 502, 503, 504]

  aggregator_apis:
    max_attempts: 5
    initial_delay: 1s
    max_delay: 60s
    multiplier: 2.0
    jitter: 0.2
    respect_retry_after: true

  database:
    max_attempts: 3
    initial_delay: 50ms
    max_delay: 1s
    multiplier: 2.0
    retryable: [connection_timeout, deadlock]

  pubnub:
    max_attempts: infinite
    initial_delay: 1s
    max_delay: 30s
    multiplier: 1.5
    jitter: 0.3
    reconnect_on_network_change: true
```

---

## 19. Performance Requirements

### 19.1 Latency Targets

| Operation | p50 | p95 | p99 | SLO |
|-----------|-----|-----|-----|-----|
| Search (simple) | 50ms | 200ms | 500ms | 99.9% |
| Search (complex NL) | 100ms | 500ms | 1s | 99.5% |
| Content details | 20ms | 100ms | 200ms | 99.9% |
| Recommendations | 30ms | 150ms | 300ms | 99.9% |
| SONA personalization | 2ms | 5ms | 10ms | 99.99% |
| Device list | 10ms | 50ms | 100ms | 99.9% |
| Remote control | 25ms | 75ms | 150ms | 99.9% |
| Watch progress sync | 50ms | 100ms | 200ms | 99.9% |
| Watchlist update | 50ms | 100ms | 200ms | 99.9% |

### 19.2 Throughput Requirements

```yaml
throughput:
  api_gateway:
    requests_per_second: 15,000
    concurrent_connections: 50,000

  search_service:
    queries_per_second: 1,000
    with_personalization: 500/s

  recommendation_service:
    requests_per_second: 8,000

  ingestion_pipeline:
    records_per_second: 1,000
    batch_size: 100

  pubnub:
    messages_per_second: 10,000 (user sync)
    presence_updates: 5,000/s

  database:
    read_qps: 50,000
    write_qps: 5,000
```

### 19.3 Resource Utilization Bounds

```yaml
resource_limits:
  cpu:
    target_utilization: 70%
    alert_threshold: 85%
    critical_threshold: 95%

  memory:
    target_utilization: 80%
    alert_threshold: 90%
    critical_threshold: 95%

  database_connections:
    pool_size: 100 per instance
    max_overflow: 20
    timeout: 5s

  cache:
    max_size: 16GB
    eviction_policy: LRU
    target_hit_rate: 80%

  network:
    max_bandwidth: 10 Gbps
    alert_threshold: 80%
```

### 19.4 Scalability Targets

```yaml
scalability:
  concurrent_users:
    launch: 100,000
    year_1: 1,000,000
    year_3: 10,000,000

  content_catalog:
    launch: 500,000 titles
    year_1: 1,000,000 titles
    year_3: 5,000,000 titles

  devices_per_user:
    average: 3
    max: 10

  watchlist_size:
    average: 50 items
    max: 1,000 items

  search_index:
    embeddings: 1M vectors @ 768 dimensions
    graph_nodes: 10M
    graph_edges: 100M
```

---

## 20. Constraints and Assumptions

### 20.1 Technical Constraints

```yaml
technical_constraints:
  platform_api_limitations:
    - "80% of major streaming platforms have NO public APIs"
    - "Must rely on third-party aggregators for catalog data"
    - "Cannot verify real-time availability for most platforms"
    - "Deep linking is only guaranteed method of playback"

  rate_limits:
    streaming_availability: 100/minute
    watchmode: 1,000/day
    youtube_data: 10,000 units/day
    tmdb: 40 requests/10 seconds

  technology_stack:
    primary_language: Rust (core services)
    secondary_language: TypeScript (CLI, web)
    database: PostgreSQL 15+, pg_vector extension
    cache: Valkey 7.2+ (Redis-compatible)
    cloud: Google Cloud Platform only

  compatibility:
    minimum_node_version: 18.0.0
    minimum_rust_version: 1.75.0
    supported_browsers: "Last 2 versions of Chrome, Firefox, Safari, Edge"
    supported_mobile: "iOS 15+, Android 11+"
```

### 20.2 Business Constraints

```yaml
business_constraints:
  budget:
    infrastructure_monthly: $3,850
    api_costs_monthly: $548
    total_monthly: ~$4,400

  timeline:
    mvp: 8 weeks
    beta: 12 weeks
    production: 16 weeks

  team:
    backend_engineers: 2-3
    frontend_engineers: 1-2
    ml_engineer: 1
    devops: 1 (part-time)

  scope:
    in_scope:
      - Content discovery and search
      - Personalized recommendations
      - Cross-device sync
      - MCP/AI agent integration
    out_of_scope:
      - Video streaming
      - Content hosting
      - Social features (Phase 2)
      - Advertising (Phase 3)
```

### 20.3 Regulatory Constraints

```yaml
regulatory:
  privacy:
    gdpr:
      - Right to access (within 30 days)
      - Right to erasure (within 72 hours)
      - Right to data portability
      - Consent management
      - Data minimization
    ccpa:
      - Do not sell my data
      - Right to know
      - Right to delete
    vppa:
      - Video viewing data retention: max 90 days
      - Opt-in for sharing viewing data

  accessibility:
    wcag_level: "AA"
    requirements:
      - Screen reader compatibility
      - Keyboard navigation
      - Color contrast ratios
      - Focus indicators

  content_ratings:
    compliance:
      - MPAA (US)
      - BBFC (UK)
      - FSK (Germany)
      - PEGI (Europe)
```

### 20.4 Assumptions

```yaml
assumptions:
  environmental:
    - "Users have stable internet connection (3+ Mbps)"
    - "Users have access to app stores (iOS/Android)"
    - "Smart TVs support modern web standards or native apps"
    - "GCP regions available in target markets"

  dependency:
    - "Aggregator APIs will remain available and affordable"
    - "PubNub will maintain 99.999% availability SLA"
    - "OpenAI embedding API will remain stable"
    - "Streaming platforms will not block deep links"

  user_behavior:
    - "Average user has 3-5 streaming subscriptions"
    - "Users search 2-5 times per session"
    - "Peak usage: 7-10 PM local time"
    - "50% of users will use multiple devices"

  infrastructure:
    - "GKE Autopilot will auto-scale appropriately"
    - "Cloud SQL will handle connection pooling"
    - "CDN will cache static assets globally"
    - "Valkey cluster will maintain consistency"
```

---

## 21. Non-Functional Requirements

### 21.1 Availability

```yaml
availability:
  targets:
    tier_1_services:  # Search, Recommendations, Auth
      slo: 99.9%
      monthly_downtime: <43 minutes
      error_budget: 0.1%

    tier_2_services:  # Device, Sync, Ingestion
      slo: 99.5%
      monthly_downtime: <3.6 hours
      error_budget: 0.5%

    tier_3_services:  # Analytics, Admin
      slo: 99.0%
      monthly_downtime: <7.2 hours
      error_budget: 1.0%

  disaster_recovery:
    rto: 4 hours (Recovery Time Objective)
    rpo: 1 hour (Recovery Point Objective)
    backup_frequency: every 6 hours
    backup_retention: 30 days
    multi_region: us-central1 (primary), us-east1 (failover)
```

### 21.2 Scalability

```yaml
scalability:
  horizontal:
    api_gateway: 2-100 instances
    search_service: 3-50 instances
    recommendation_service: 3-30 instances
    ingestion_service: 2-20 instances

  vertical:
    database: n2-standard-4 → n2-standard-32 (Cloud SQL)
    cache: 4GB → 64GB (Memorystore)

  auto_scaling:
    metric: CPU utilization
    target: 70%
    scale_up: +25% instances
    scale_down: -10% instances (stabilization: 5 min)

  data_scaling:
    partitioning:
      - content: by release_year
      - availability: by region
      - user_data: by user_id hash
    sharding:
      - future consideration for >10M users
```

### 21.3 Maintainability

```yaml
maintainability:
  code_quality:
    test_coverage: ≥80%
    linting: enforced (clippy for Rust, ESLint for TS)
    formatting: enforced (rustfmt, Prettier)
    complexity:
      cyclomatic_max: 10
      file_lines_max: 500
      function_lines_max: 50

  documentation:
    api: OpenAPI 3.0 spec
    code: rustdoc / TSDoc
    architecture: ADRs (Architecture Decision Records)
    runbooks: for all Tier 1 services

  deployment:
    strategy: GitOps (ArgoCD)
    frequency: daily (can do multiple)
    rollback: <5 minutes
    feature_flags: LaunchDarkly or similar

  monitoring:
    dashboards: Grafana
    alerts: PagerDuty
    logs: Cloud Logging
    traces: Cloud Trace
```

### 21.4 Observability

```yaml
observability:
  metrics:
    collection: Prometheus
    retention: 30 days
    key_metrics:
      - request_rate (by endpoint)
      - error_rate (by type)
      - latency_percentiles (p50, p95, p99)
      - saturation (CPU, memory, connections)

  logging:
    format: structured JSON
    levels: ERROR, WARN, INFO, DEBUG
    fields:
      - timestamp
      - request_id
      - user_id (hashed)
      - service_name
      - message
    retention: 30 days (hot), 1 year (cold)

  tracing:
    protocol: OpenTelemetry
    sampling: 1% (normal), 100% (errors)
    propagation: W3C Trace Context
    visualization: Cloud Trace

  alerting:
    channels:
      - PagerDuty (critical)
      - Slack (warning)
      - Email (informational)
    escalation:
      - P1: immediate page
      - P2: page after 15 min
      - P3: Slack notification
      - P4: next business day
```

### 21.5 Security

```yaml
security:
  authentication:
    - OAuth 2.0 + PKCE for all clients
    - Device Authorization Grant for TV/CLI
    - mTLS for service-to-service

  encryption:
    at_rest: AES-256
    in_transit: TLS 1.3
    secrets: Google Secret Manager

  network:
    - VPC with private subnets
    - Cloud Armor WAF
    - DDoS protection
    - Egress controls

  compliance:
    - SOC 2 Type II (target)
    - GDPR compliant
    - CCPA compliant
    - VPPA compliant

  auditing:
    - All authentication events
    - All data access events
    - All configuration changes
    - Retention: 1 year

  penetration_testing:
    frequency: quarterly
    scope: full application
    vendor: third-party
```

---

## 22. Appendix: Glossary

| Term | Definition |
|------|------------|
| **ARW** | Agent-Ready Web - Protocol for AI agent discovery and interaction |
| **CRDT** | Conflict-free Replicated Data Type - Data structures that enable automatic conflict resolution |
| **EIDR** | Entertainment Identifier Registry - Universal content identifier standard |
| **HLC** | Hybrid Logical Clock - Timestamp mechanism combining physical and logical time |
| **LWW** | Last-Writer-Wins - CRDT conflict resolution strategy |
| **MCP** | Model Context Protocol - Standard for AI agent communication |
| **OR-Set** | Observed-Remove Set - CRDT for set operations |
| **PKCE** | Proof Key for Code Exchange - OAuth 2.0 extension for public clients |
| **PubNub** | Real-time messaging infrastructure |
| **Ruvector** | Vector database + hypergraph storage component |
| **SONA** | Self-Optimizing Neural Architecture - Personalization engine |
| **SPARC** | Specification, Pseudocode, Architecture, Refinement, Completion |
| **SSE** | Server-Sent Events - HTTP streaming protocol |
| **VPPA** | Video Privacy Protection Act - US law governing video viewing data |

---

## Document Summary

This SPARC Specification defines the complete functional and non-functional requirements for evolving hackathon-tv5 into a production-ready Media Gateway engine.

**Key Deliverables:**
1. Unified cross-platform content discovery
2. SONA-powered personalized recommendations
3. Real-time cross-device sync via PubNub
4. MCP server for AI agent integration
5. ARW protocol for efficient agent discovery
6. Privacy-first architecture with CRDT-based sync

**Technology Stack:**
- Core: Rust microservices
- CLI/Web: TypeScript
- Database: PostgreSQL + pg_vector
- Cache: Valkey
- Real-time: PubNub
- Cloud: GCP (GKE Autopilot, Cloud Run, Cloud SQL)

**Performance Targets:**
- Search latency: <500ms (p95)
- Personalization: <5ms
- Cross-device sync: <100ms
- Availability: 99.9% (Tier 1)

**Cost Target:** ~$4,400/month

---

**End of SPARC Specification Phase**

**Next Steps:**
1. Stakeholder review and approval
2. SPARC Pseudocode Phase (algorithm design)
3. SPARC Architecture Phase (system design)
4. SPARC Refinement Phase (TDD implementation)
5. SPARC Completion Phase (integration and deployment)
