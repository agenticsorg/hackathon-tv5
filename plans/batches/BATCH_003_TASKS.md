# BATCH_003_TASKS.md - Media Gateway Action List

**Generated:** 2025-12-06
**Batch:** 003
**Previous Batches:** BATCH_001 (12 tasks), BATCH_002 (12 tasks) - All completed
**Analysis Method:** 9-agent Claude-Flow swarm parallel analysis
**SPARC Phase:** Refinement (Implementation)

---

## Action List

### TASK-001: Wire HybridSearchService to Redis Cache Layer

**File:** `/workspaces/media-gateway/crates/discovery/src/search/mod.rs` (modify existing)

**Description:** The HybridSearchService struct has NO `cache: RedisCache` field despite BATCH_002 implementing a complete Redis caching layer in `cache.rs`. The search pipeline doesn't call `cache.get_search_results()` or `cache.cache_search_results()`. Every search hits Qdrant+Postgres even for identical queries. Wire the cache module into the main search orchestrator with SHA256 query hashing for cache keys.

**Dependencies:** BATCH_002 TASK-001 (Redis cache.rs - completed)

**Acceptance Criteria:**
- HybridSearchService includes `cache: RedisCache` field initialized from config
- `search()` method checks cache before executing vector/keyword search
- Cache hits return in <10ms vs 200-400ms for cache misses
- Cache key uses SHA256 hash of query + filters + user_id
- Integration tests verify cache hit/miss behavior with 30min TTL

---

### TASK-002: Implement Offline-First Sync Queue with Persistence

**File:** `/workspaces/media-gateway/crates/sync/src/sync/queue.rs` (new file)

**Description:** The sync service has no persistent queue for operations performed while offline. Currently, watchlist and progress updates are only published to PubNub in real-time. When a device goes offline, local changes are never reconciled when connectivity is restored. Implement SQLite-backed operation queue that stores pending CRDT operations and replays them on reconnection.

**Dependencies:** BATCH_002 TASK-003 (PubNub Publishing - completed)

**Acceptance Criteria:**
- `OfflineSyncQueue` struct with `enqueue()`, `dequeue()`, `peek()` operations
- SQLite persistence for pending operations survives app restart
- Queue replays operations in order on `reconnect()` trigger
- Conflict detection when replaying (merge with remote state via CRDT)
- Integration test: enqueue 10 ops offline, reconnect, verify all synced within 500ms

---

### TASK-003: Implement Request Timeout and Retry Logic for MCP Tools

**File:** `/workspaces/media-gateway/apps/mcp-server/src/tools/` (modify all tool files)

**Description:** All MCP tool fetches have no timeout - if discovery/content service is slow, the <150ms MCP latency SLA fails. Add configurable request timeouts (default 100ms), retry logic with exponential backoff (max 2 retries, 50ms base delay), and fallback strategies when downstream services fail.

**Dependencies:** None

**Acceptance Criteria:**
- All `fetch()` calls wrapped with `AbortController` timeout (100ms default)
- Retry utility with exponential backoff: 50ms → 100ms → give up
- Timeout/retry metrics logged via structured logging
- Fallback returns cached stale data or graceful error with retry-after hint
- Tool tests verify timeout triggers after 100ms, retry succeeds on transient failure

---

### TASK-004: Wire SONA HTTP Endpoints to Recommendation Business Logic

**File:** `/workspaces/media-gateway/crates/sona/src/server.rs` (lines 58-158)

**Description:** All 5 SONA HTTP endpoints return hardcoded mock responses without calling actual business logic. `POST /recommendations` returns empty array instead of calling `GenerateRecommendations::execute()`. Wire each endpoint to its corresponding use case: recommendations → GenerateRecommendations, personalization/score → LoRA inference, profile/update → UpdateUserLoRA, lora/train → training pipeline.

**Dependencies:** BATCH_001 TASK-002, TASK-003 (SONA DB layers - completed), BATCH_002 TASK-002 (LoRA storage - completed)

**Acceptance Criteria:**
- `POST /recommendations` calls `GenerateRecommendations::execute()` with real UserProfile
- `POST /personalization/score` computes score using loaded LoRA adapter
- `POST /profile/update` triggers `BuildUserPreferenceVector::execute()`
- `POST /lora/train` queues actual LoRA training job
- All endpoints return real data, not hardcoded mocks

---

### TASK-005: Implement PostgreSQL Upsert for CanonicalContent

**File:** `/workspaces/media-gateway/crates/ingestion/src/repository.rs` (lines 54-77)

**Description:** The `PostgresContentRepository::upsert()` method is a stub that returns a new UUID without persisting CanonicalContent to the database. Despite the repository pattern from BATCH_001, actual INSERT/ON CONFLICT UPDATE SQL with field mapping is missing. Without this, all ingested content is lost after ingestion cycle.

**Dependencies:** BATCH_001 TASK-004 (Repository pattern - completed)

**Acceptance Criteria:**
- `upsert()` executes INSERT...ON CONFLICT UPDATE with all CanonicalContent fields
- Maps external_ids, genres, availability, credits to appropriate columns/JSON
- Returns existing content_id on conflict (match by EIDR, IMDB, or title+year)
- Transaction support for batch operations (10 items per transaction)
- Integration test verifies content survives service restart

---

### TASK-006: Implement Qdrant Vector Indexing After Content Ingestion

**File:** `/workspaces/media-gateway/crates/ingestion/src/qdrant.rs` (new file)

**Description:** No integration with Qdrant vector database exists. Embeddings are generated in `embedding.rs` and stored in PostgreSQL but never indexed in Qdrant for semantic search. Create QdrantClient module with batch upsert operations that indexes content embeddings after successful ingestion.

**Dependencies:** BATCH_001 TASK-001 (OpenAI embeddings - completed)

**Acceptance Criteria:**
- `QdrantClient` struct with connection pooling and health check
- `upsert_batch()` method indexes up to 100 vectors per call
- Vectors include metadata payload (content_id, title, genres, platform)
- Called from `IngestionPipeline::process_batch()` after DB persistence
- Integration test verifies vectors retrievable via similarity search

---

### TASK-007: Add Rate Limiting Middleware to Auth Endpoints

**File:** `/workspaces/media-gateway/crates/auth/src/middleware/rate_limit.rs` (new file)

**Description:** The auth service has `AuthError::RateLimitExceeded` error variant but no actual rate limiting middleware. All critical auth endpoints (`/auth/authorize`, `/auth/token`, `/auth/revoke`, `/auth/device`) are unprotected against brute force attacks. Implement Redis-backed rate limiting with per-endpoint and per-client-id limits.

**Dependencies:** BATCH_001 TASK-006 (Redis migration - completed)

**Acceptance Criteria:**
- Rate limit middleware using Redis sliding window algorithm
- Default limits: 10 req/min for `/token`, 5 req/min for `/device`
- Per-client_id tracking (not just IP)
- Returns 429 with Retry-After header when exceeded
- Bypass mechanism for internal service-to-service calls

---

### TASK-008: Implement Device Authorization Approval Endpoint (RFC 8628)

**File:** `/workspaces/media-gateway/crates/auth/src/server.rs` (add new endpoint)

**Description:** The device code polling endpoint (`/auth/device/poll`) exists but there is NO endpoint to actually approve a device code. RFC 8628 device authorization grant flow requires a separate approval flow where users login on phone/browser and approve the TV device. Implement `POST /auth/device/approve` endpoint.

**Dependencies:** BATCH_001 TASK-006 (Redis auth state - completed)

**Acceptance Criteria:**
- `POST /auth/device/approve` accepts user_code and JWT auth token
- Validates user_code matches pending device in Redis
- Transitions device state from `Pending` to `Approved` with user_id binding
- Polling endpoint returns tokens once approved
- Integration test: initiate device flow → approve → poll returns tokens

---

### TASK-009: Implement Playback-to-Sync Service Integration

**File:** `/workspaces/media-gateway/crates/playback/src/main.rs` (lines 108-118)

**Description:** The `update_position` endpoint updates position in Playback service Redis but does NOT synchronize with Sync Service's `ProgressSync`. Users starting on phone cannot resume on TV. After updating local position, call Sync Service to publish cross-device update via PubNub.

**Dependencies:** BATCH_002 TASK-003 (PubNub Publishing - completed), BATCH_002 TASK-010 (Command Router - completed)

**Acceptance Criteria:**
- `update_position` calls Sync Service `/api/v1/sync/progress` after local update
- HTTP client with 50ms timeout and retry (non-blocking, fire-and-forget OK)
- Position updates propagate to other devices within 100ms
- Integration test: update on device A, verify received on device B via PubNub

---

### TASK-010: Extract User Authentication Context in Discovery Endpoints

**File:** `/workspaces/media-gateway/crates/discovery/src/server.rs` (line 77)

**Description:** Hybrid search endpoint sets `user_id: None` with TODO "Extract from auth context". No JWT parsing occurs, so all requests are anonymous. Without user_id extraction, personalization (SONA integration) and per-user rate limiting cannot work. Extract user_id from Authorization header JWT claims.

**Dependencies:** BATCH_001 TASK-006 (Auth Redis - completed)

**Acceptance Criteria:**
- Parse `Authorization: Bearer <token>` header in search handlers
- Validate JWT signature using shared secret from config
- Extract `sub` claim as user_id, pass to HybridSearchService
- Anonymous requests allowed but logged as user_id: None
- Integration test: search with valid JWT returns personalized results

---

### TASK-011: Implement Exponential Backoff Retry Utility in Core Crate

**File:** `/workspaces/media-gateway/crates/core/src/retry.rs` (new file)

**Description:** Error module has `is_retryable()` method but NO utility to execute retries. Services independently implement backoff logic, creating inconsistencies. Create centralized retry utility with configurable policy (max retries, base delay, jitter) and async executor for transient failures.

**Dependencies:** BATCH_002 TASK-006 (Config loader - completed)

**Acceptance Criteria:**
- `RetryPolicy` struct with max_retries, base_delay_ms, max_delay_ms, jitter
- `retry_with_backoff<F, T, E>()` generic async function
- Exponential backoff: delay = base * 2^attempt + random_jitter
- Respects `is_retryable()` from error types
- Unit tests verify retry count, delay progression, and jitter bounds

---

### TASK-012: Implement Kafka Event Publishing for Playback State Changes

**File:** `/workspaces/media-gateway/crates/playback/src/events.rs` (new file)

**Description:** Session updates (create, update_position, delete) do NOT publish events to analytics/event stream. BATCH_002 TASK-008 implemented Kafka for Ingestion, but Playback doesn't emit events. SONA needs playback events (`playback.session.created`, `playback.position.updated`, `playback.session.ended`) to train recommendations from watch history.

**Dependencies:** BATCH_002 TASK-008 (Kafka events infrastructure - completed)

**Acceptance Criteria:**
- `PlaybackEventProducer` trait with `KafkaPlaybackProducer` implementation
- Event types: SessionCreatedEvent, PositionUpdatedEvent, SessionEndedEvent
- Events published at end of create_session, update_position, end_session
- Include user_id, content_id, device_id, position_seconds, timestamp
- Unit tests with mock producer verify event emission

---

## Summary

| Task ID | Title | Files | Dependencies |
|---------|-------|-------|--------------|
| TASK-001 | Wire Search to Redis Cache | discovery/src/search/mod.rs | B2-T001 |
| TASK-002 | Offline-First Sync Queue | sync/src/sync/queue.rs | B2-T003 |
| TASK-003 | MCP Tool Timeouts & Retries | apps/mcp-server/src/tools/ | None |
| TASK-004 | Wire SONA Endpoints | sona/src/server.rs | B1-T002, B1-T003, B2-T002 |
| TASK-005 | PostgreSQL Content Upsert | ingestion/src/repository.rs | B1-T004 |
| TASK-006 | Qdrant Vector Indexing | ingestion/src/qdrant.rs | B1-T001 |
| TASK-007 | Auth Rate Limiting | auth/src/middleware/rate_limit.rs | B1-T006 |
| TASK-008 | Device Approval Endpoint | auth/src/server.rs | B1-T006 |
| TASK-009 | Playback-Sync Integration | playback/src/main.rs | B2-T003, B2-T010 |
| TASK-010 | Auth Context Extraction | discovery/src/server.rs | B1-T006 |
| TASK-011 | Retry Utility Module | core/src/retry.rs | B2-T006 |
| TASK-012 | Playback Kafka Events | playback/src/events.rs | B2-T008 |

**Total Tasks:** 12
**Independent Tasks (can start immediately):** 5 (TASK-003, 005, 006, 007, 011)
**Tasks with Dependencies:** 7

---

## Execution Order Recommendation

**Phase 1 (Parallel - No Dependencies):**
- TASK-003: MCP Tool Timeouts (critical for <150ms SLA)
- TASK-005: PostgreSQL Content Upsert (critical for data persistence)
- TASK-006: Qdrant Vector Indexing (enables semantic search)
- TASK-007: Auth Rate Limiting (security hardening)
- TASK-011: Retry Utility Module (enables resilience patterns)

**Phase 2 (After Phase 1):**
- TASK-001: Wire Search to Redis Cache (improves latency)
- TASK-002: Offline-First Sync Queue (enables offline support)
- TASK-008: Device Approval Endpoint (completes RFC 8628)
- TASK-010: Auth Context Extraction (enables personalization)

**Phase 3 (After Phase 2):**
- TASK-004: Wire SONA Endpoints (requires auth context for user_id)
- TASK-009: Playback-Sync Integration (requires sync infrastructure)
- TASK-012: Playback Kafka Events (feeds SONA recommendations)

---

## Agent Contributions

| Agent | Focus Area | Gaps Identified | Tasks Selected |
|-------|------------|-----------------|----------------|
| Agent 1 | API Gateway | 13 gaps | - |
| Agent 2 | Discovery Service | 11 gaps | TASK-001, TASK-010 |
| Agent 3 | SONA Engine | 11 gaps | TASK-004 |
| Agent 4 | Sync Service | 9 gaps | TASK-002 |
| Agent 5 | Auth Service | 10 gaps | TASK-007, TASK-008 |
| Agent 6 | Ingestion Service | 15 gaps | TASK-005, TASK-006 |
| Agent 7 | Core Crate | 12 gaps | TASK-011 |
| Agent 8 | Playback Service | 10 gaps | TASK-009, TASK-012 |
| Agent 9 | MCP Server | 10 gaps | TASK-003 |

**Total Gaps Analyzed:** 101 across 9 services
**Tasks Selected:** 12 highest-priority, non-duplicating tasks

---

*Generated by 9-agent Claude-Flow swarm analysis*
