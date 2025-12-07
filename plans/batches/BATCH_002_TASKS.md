# BATCH_002_TASKS.md - Media Gateway Action List

**Generated:** 2025-12-06
**Batch:** 002
**Previous Batch:** BATCH_001_TASKS.md (12 tasks completed)
**Analysis Method:** 9-agent Claude-Flow swarm parallel analysis
**SPARC Phase:** Refinement (Implementation)

---

## Action List

### TASK-001: Implement Redis Caching Layer for Search Results and Intent Parsing

**File:** `/workspaces/media-gateway/crates/discovery/src/cache.rs` (new file)

**Description:** Create Redis caching module to cache search results (30min TTL), embeddings (1hr TTL), and parsed intents (10min TTL) as specified in CacheConfig. Integrate with HybridSearchService and IntentParser to reduce OpenAI API calls and improve <500ms latency target. The cache infrastructure is configured but marked as TODO in intent.rs:70.

**Dependencies:** None (uses existing Redis from docker-compose)

**Acceptance Criteria:**
- cache.rs implements RedisCache with get/set/delete operations
- IntentParser checks cache before GPT-4o-mini calls
- HybridSearchService caches SearchResponse by query hash
- Integration tests show cache hit/miss behavior with TTL expiration
- Latency improvements measurable (cache hit <10ms vs API call >1000ms)

---

### TASK-002: LoRA Model Persistence and Loading Infrastructure

**File:** `/workspaces/media-gateway/crates/sona/src/lora_storage.rs` (new file)

**Description:** Implement SQLx-based LoRA adapter persistence layer to save/load the two-tier LoRA weights (base_layer_weights, user_layer_weights) to PostgreSQL. BATCH_001 covered database queries for collaborative/content filtering but did NOT cover LoRA model serialization. This enables <5ms inference by pre-loading trained adapters.

**Dependencies:** None (uses existing PostgreSQL)

**Acceptance Criteria:**
- Can serialize UserLoRAAdapter to PostgreSQL BYTEA column
- Retrieve by user_id in <2ms
- Deserialize ndarray matrices correctly
- Unit tests verify round-trip serialization preserves weights within 0.001 epsilon

---

### TASK-003: Integrate PubNub Publishing with Sync Managers

**File:** `/workspaces/media-gateway/crates/sync/src/sync/publisher.rs` (new file), modify `sync/watchlist.rs`, `sync/progress.rs`

**Description:** Create SyncPublisher trait that wraps WatchlistSync and ProgressSync to automatically publish CRDT updates (WatchlistUpdate, ProgressUpdate) to PubNub channels when add/remove/update operations are called. Currently WebSocket messages have TODO comments for PubNub broadcast integration.

**Dependencies:** BATCH_001 TASK-007 (PubNub Subscribe)

**Acceptance Criteria:**
- WatchlistSync and ProgressSync changes automatically publish to PubNub user.{userId}.sync channel
- Integration tests verify messages received on subscribed clients within 100ms
- CRDT operations trigger PubNub publish, not just local state updates

---

### TASK-004: Implement Response Caching Middleware with Redis

**File:** `/workspaces/media-gateway/crates/api/src/middleware/cache.rs` (new file)

**Description:** Create Redis-backed response caching middleware that intercepts responses for cacheable routes (GET requests to content/search endpoints), stores them in Redis with appropriate TTLs (5min for content, 1h for sessions), and returns cached responses with proper Cache-Control and ETag headers.

**Dependencies:** None (uses existing Redis)

**Acceptance Criteria:**
- Middleware caches GET /api/v1/content/* responses in Redis with 5-minute TTL
- Cache-Control and ETag headers added to responses
- Cache hit/miss metrics logged via tracing
- Integration test demonstrates cache hit returns 304 Not Modified on ETag match

---

### TASK-005: Add Circuit Breaker State Persistence to Redis

**File:** `/workspaces/media-gateway/crates/api/src/circuit_breaker.rs` (modify existing)

**Description:** Extend CircuitBreakerManager to persist state (open/closed/half-open, failure counts) to Redis. Currently state is in-memory only, lost on gateway restarts and not shared across multiple gateway instances. SPARC specifies Redis for state management with high availability.

**Dependencies:** None (uses existing Redis)

**Acceptance Criteria:**
- Circuit breaker state persisted to Redis with keys like "circuit_breaker:{service}:state"
- State persists across server restarts (integration test verifies)
- Multiple gateway instances share the same circuit state
- Redis connection failures degrade gracefully to in-memory-only mode with warning logs

---

### TASK-006: Create Shared Configuration Loader Module

**File:** `/workspaces/media-gateway/crates/core/src/config.rs` (new file)

**Description:** Implement centralized configuration management module with environment variable parsing, validation, and type-safe configuration loading utilities. Each service currently has its own config.rs with duplicated patterns. This module should support common patterns like database URLs, service endpoints, timeouts, and feature flags.

**Dependencies:** None

**Acceptance Criteria:**
- Module exports ConfigLoader trait with from_env() and with_defaults() functions
- Includes validation for required fields
- Unit tests demonstrate loading from environment variables with proper error handling
- Supports .env file loading with environment variable precedence

---

### TASK-007: Implement Structured Logging and Tracing Initialization

**File:** `/workspaces/media-gateway/crates/core/src/observability.rs` (new file)

**Description:** Create shared observability module that initializes structured logging with tracing-subscriber and provides consistent log formatting across all services. Include support for JSON and pretty-print formats, configurable log levels per module, and environment-based configuration.

**Dependencies:** TASK-006 (shared config)

**Acceptance Criteria:**
- Module exports init_logging() function that sets up tracing subscriber with JSON formatting
- Supports RUST_LOG environment variable
- Includes span tracking for request correlation
- Integration tests verify log output format and filtering

---

### TASK-008: Implement Kafka Event Streaming for Content Lifecycle Events

**File:** `/workspaces/media-gateway/crates/ingestion/src/events.rs` (new file)

**Description:** Create Kafka event producer that publishes content lifecycle events (content.ingested, content.updated, availability.changed, metadata.enriched) to enable real-time downstream notifications. SPARC specification requires Kafka event streaming for ingestion events.

**Dependencies:** Kafka added to docker-compose (minor addition)

**Acceptance Criteria:**
- Kafka producer using rdkafka crate with async/await support
- Event types: ContentIngestedEvent, ContentUpdatedEvent, AvailabilityChangedEvent
- Events published at end of process_batch, sync_availability, and enrich_metadata in pipeline.rs
- Configuration from environment (KAFKA_BROKERS, KAFKA_TOPIC_PREFIX)
- Unit tests with mock Kafka producer showing successful event emission

---

### TASK-009: Context-Aware Candidate Generation Database Integration

**File:** `/workspaces/media-gateway/crates/sona/src/context.rs` (modify existing)

**Description:** Replace simulated methods (filter_by_time_of_day, filter_by_device, filter_by_mood) with real PostgreSQL queries using SQLx. Query content table filtered by temporal patterns (hourly_patterns vector), device_type compatibility flags, and mood-to-genre mappings. Currently all methods return empty vectors with "Simulated" comments.

**Dependencies:** BATCH_001 TASK-002, TASK-003 (SONA DB layers)

**Acceptance Criteria:**
- filter_by_time_of_day returns real content IDs from PostgreSQL based on user's hourly_patterns
- Integration test with seeded data verifies correct filtering for "evening" vs "morning" contexts
- No more "Simulated - in real implementation" comments in context.rs

---

### TASK-010: Build Remote Command Router with PubNub Targeting

**File:** `/workspaces/media-gateway/crates/sync/src/command_router.rs` (new file), modify `websocket.rs`

**Description:** Create CommandRouter that validates RemoteCommand against target DeviceInfo (using device.rs validation), publishes command to PubNub user.{userId}.devices channel with target_device_id filtering, and handles command TTL expiration (5s). WebSocket handler should route DeviceCommand messages through this router instead of inline TODO comments.

**Dependencies:** TASK-003 (PubNub Publishing)

**Acceptance Criteria:**
- Remote commands (Play, Pause, Seek, Cast) published to PubNub and delivered only to target device
- Expired commands (>5s TTL) rejected with CommandError::Expired
- Integration test confirms TV receives phone's Play command within 100ms

---

### TASK-011: Implement Prometheus Metrics Endpoints in All Services

**File:** `/workspaces/media-gateway/crates/core/src/metrics.rs` (new file), update all service `main.rs` files

**Description:** Create shared metrics module in core crate that provides HTTP request counters, latency histograms, and active connection gauges. The prometheus = "0.13" dependency exists in Cargo.toml but is unused. K8s manifests configure Prometheus scraping on /metrics endpoint (port 9090) for all services, but no services currently expose metrics.

**Dependencies:** TASK-007 (observability module)

**Acceptance Criteria:**
- All 7 services (api, discovery, sona, sync, auth, ingestion, mcp) expose /metrics endpoint on port 9090
- Metrics include http_requests_total, http_request_duration_seconds, and service-specific gauges
- Prometheus-formatted output compatible with K8s scrape annotations

---

### TASK-012: Add Production Readiness Health Checks to All Services

**File:** Update all service `main.rs` files (api, discovery, sona, sync, auth, ingestion)

**Description:** Enhance health endpoints to check actual service readiness. Current /health endpoints return static JSON. K8s liveness/readiness probes require dependency health checks: PostgreSQL connection pool status, Redis connectivity, Qdrant availability, and PubNub connection state. Return HTTP 200 only when all dependencies are healthy, 503 otherwise.

**Dependencies:** None

**Acceptance Criteria:**
- Health endpoints return 503 if database/Redis/Qdrant is unavailable
- K8s readiness probes correctly remove unhealthy pods from service rotation
- Health response includes detailed component status: `{"status": "healthy|degraded|unhealthy", "components": {...}}`

---

## Summary

| Task ID | Title | Files | Dependencies |
|---------|-------|-------|--------------|
| TASK-001 | Redis Caching for Search/Intent | discovery/src/cache.rs | None |
| TASK-002 | LoRA Model Persistence | sona/src/lora_storage.rs | None |
| TASK-003 | PubNub Publishing Integration | sync/src/sync/publisher.rs | B1-T007 |
| TASK-004 | Response Caching Middleware | api/src/middleware/cache.rs | None |
| TASK-005 | Circuit Breaker Redis Persistence | api/src/circuit_breaker.rs | None |
| TASK-006 | Shared Configuration Loader | core/src/config.rs | None |
| TASK-007 | Structured Logging/Tracing Init | core/src/observability.rs | TASK-006 |
| TASK-008 | Kafka Event Streaming | ingestion/src/events.rs | Kafka in docker-compose |
| TASK-009 | Context-Aware DB Integration | sona/src/context.rs | B1-T002, B1-T003 |
| TASK-010 | Remote Command Router | sync/src/command_router.rs | TASK-003 |
| TASK-011 | Prometheus Metrics Endpoints | core/src/metrics.rs + services | TASK-007 |
| TASK-012 | Production Health Checks | all service main.rs files | None |

**Total Tasks:** 12
**Independent Tasks (can start immediately):** 8
**Tasks with Dependencies:** 4

---

## Execution Order Recommendation

**Phase 1 (Parallel - No Dependencies):**
- TASK-001, TASK-002, TASK-004, TASK-005, TASK-006, TASK-012

**Phase 2 (After Phase 1):**
- TASK-003 (depends on B1-T007, already done)
- TASK-007 (depends on TASK-006)
- TASK-008 (requires Kafka in docker-compose)
- TASK-009 (depends on B1-T002, B1-T003, already done)

**Phase 3 (After Phase 2):**
- TASK-010 (depends on TASK-003)
- TASK-011 (depends on TASK-007)

---

*Generated by 9-agent Claude-Flow swarm analysis*
