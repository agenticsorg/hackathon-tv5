# BATCH_005_TASKS.md - Media Gateway Action List

**Generated:** 2025-12-06
**Batch:** 005
**Previous Batches:** BATCH_001 (12 tasks), BATCH_002 (12 tasks), BATCH_003 (12 tasks), BATCH_004 (12 tasks) - All completed
**Analysis Method:** 9-agent Claude-Flow swarm parallel analysis
**SPARC Phase:** Refinement (Implementation)

---

## Action List

### TASK-001: Implement Sync Service PostgreSQL Persistence Layer

**File:** `/workspaces/media-gateway/crates/sync/src/repository.rs` (new file)

**Description:** All sync state (watchlists, progress, devices) exists only in memory and is lost on service restart. The sync service has complete CRDT implementations but NO database persistence. User data, device registrations, and playback positions are ephemeral. This is a critical data loss vector.

**Dependencies:** BATCH_001 TASK-005 (Database pool - completed)

**Acceptance Criteria:**
- Create `SyncRepository` trait with CRUD operations for watchlists, progress, and devices
- Implement `PostgresSyncRepository` using SQLx with connection pooling
- Add database schema: `user_watchlists`, `user_progress`, `user_devices` tables
- Load CRDT state from database on service startup
- Persist CRDT state on every mutation (debounced for performance)
- Integration tests verify data survives service restart

---

### TASK-002: Implement Graph-Based Recommendations for SONA

**File:** `/workspaces/media-gateway/crates/sona/src/graph.rs` (new file), modify `recommendation.rs`

**Description:** The recommendation engine allocates 30% weight to graph-based recommendations but the implementation returns empty vectors. Content relationship graphs (similar genres, shared cast, user co-viewing patterns) are not utilized. This severely degrades recommendation quality.

**Dependencies:** BATCH_001 TASK-002, TASK-003 (SONA DB layers - completed)

**Acceptance Criteria:**
- Create `GraphRecommender` struct with PostgreSQL-backed graph queries
- Implement content-content similarity using shared attributes (genre, cast, director)
- Implement user-user collaborative graph ("users who liked X also liked Y")
- Query returns ranked content IDs with graph affinity scores
- Replace stub in `generate_graph_based()` with actual implementation
- Performance: graph queries complete in <100ms for 1000-node traversal

---

### TASK-003: Implement ONNX Runtime Integration for SONA Embeddings

**File:** `/workspaces/media-gateway/crates/sona/src/inference.rs` (new file), modify `lora.rs`

**Description:** The `ort` (ONNX Runtime) dependency is declared but NEVER USED. All embedding operations return dummy `vec![0.0; 512]` vectors. Without real model inference, personalization and content-based filtering cannot function. The LoRA adapters train but inference is mocked.

**Dependencies:** BATCH_002 TASK-002 (LoRA storage - completed)

**Acceptance Criteria:**
- Create `ONNXInference` struct wrapping `ort::Session`
- Load pre-trained base embedding model from configurable path
- Implement `generate_embedding(text: &str) -> Vec<f32>` with actual inference
- Support batch inference for efficiency (`generate_embeddings_batch`)
- Apply LoRA adapter weights during inference
- Model loading time <2s, inference latency <50ms per item

---

### TASK-004: Wire Rate Limiting Middleware to Auth Server

**File:** `/workspaces/media-gateway/crates/auth/src/main.rs` (modify), `server.rs` (modify)

**Description:** BATCH_003 TASK-007 implemented comprehensive rate limiting middleware (576 lines) but it is NOT applied to the auth server. Critical authentication endpoints (`/auth/token`, `/auth/authorize`) are unprotected against brute force attacks. The middleware exists but is never instantiated.

**Dependencies:** BATCH_003 TASK-007 (Rate limiting middleware - completed)

**Acceptance Criteria:**
- Add `TokenFamilyManager` initialization in `main.rs` (currently missing, causes compile error)
- Apply `RateLimitMiddleware` to `HttpServer::new()` App builder
- Configure per-endpoint limits: 10 req/min for `/token`, 5 req/min for `/device`
- Add Redis client initialization for rate limit state
- Verify 429 responses with `Retry-After` header when limits exceeded
- Integration tests confirm rate limiting is active

---

### TASK-005: Implement Google OAuth Provider

**File:** `/workspaces/media-gateway/crates/auth/src/oauth/providers/google.rs` (new file)

**Description:** The OAuth infrastructure exists with `OAuthConfig` and `OAuthProvider` traits, but NO actual provider implementations exist. The main.rs initializes with an empty HashMap of providers. Users cannot authenticate via social login despite the framework being in place.

**Dependencies:** BATCH_001 TASK-006 (Redis auth storage - completed)

**Acceptance Criteria:**
- Create `GoogleOAuthProvider` implementing `OAuthProvider` trait
- Implement authorization URL generation with PKCE and state parameter
- Implement token exchange at `https://oauth2.googleapis.com/token`
- Implement user profile retrieval from Google API
- Add `GET /auth/oauth/google/authorize` and `GET /auth/oauth/google/callback` endpoints
- Configuration via `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET` env vars
- Integration tests with mock Google responses

---

### TASK-006: Implement User Personalization in Discovery Search

**File:** `/workspaces/media-gateway/crates/discovery/src/search/personalization.rs` (new file), modify `mod.rs`

**Description:** The search pipeline has a TODO stub for personalization. User ID is extracted from JWT (BATCH_003 TASK-010) but NEVER used for scoring. Search results are identical for all users regardless of preferences. The `ranked_results` variable just returns unmodified `merged_results`.

**Dependencies:** BATCH_003 TASK-010 (Auth context extraction - completed), BATCH_004 TASK-004 (A/B testing - completed)

**Acceptance Criteria:**
- Create `PersonalizationService` that calls SONA `/personalization/score` endpoint
- Fetch user preference vector on search requests with valid user_id
- Apply personalization boost to relevance scores (configurable weight)
- Rerank results based on user profile affinity
- Support A/B testing of personalization algorithms via experiment variant
- Performance: personalization adds <50ms to search latency
- Cache user preferences in Redis (TTL: 5 minutes)

---

### TASK-007: Implement Intent Parser Redis Caching

**File:** `/workspaces/media-gateway/crates/discovery/src/intent.rs` (modify lines 70-75)

**Description:** Every search query makes a GPT-4o-mini API call for intent parsing. The code has a TODO comment "implement caching" but no cache integration exists. Repeated identical queries waste API calls ($0.15/1M tokens) and add 100-500ms latency. The RedisCache is initialized but never used in IntentParser.

**Dependencies:** BATCH_002 TASK-001 (Redis cache - completed)

**Acceptance Criteria:**
- Add `cache: Arc<RedisCache>` field to `IntentParser` struct
- Generate cache key from SHA256 hash of normalized query
- Check cache before calling `parse_with_gpt()`
- Cache parsed intents with TTL from config (`intent_ttl_sec`: 600 seconds)
- Log cache hit/miss metrics
- Performance: cache hit returns in <5ms vs 100-500ms for GPT call
- Unit tests verify cache behavior and TTL expiration

---

### TASK-008: Add Missing Platform Normalizers (Hulu, Apple TV+, Paramount+, Peacock)

**File:** `/workspaces/media-gateway/crates/ingestion/src/normalizer/` (new files)

**Description:** SPARC specification lists 9 streaming platforms but only 5 have dedicated normalizers (Netflix, Prime, Disney+, HBO Max, Generic). Deep link support exists for all platforms but 4 normalizers are missing. Content from these platforms uses GenericNormalizer without platform-specific genre mapping or metadata extraction.

**Dependencies:** BATCH_004 TASK-006 (HBO Max normalizer pattern - completed)

**Acceptance Criteria:**
- Create `hulu.rs`, `apple_tv_plus.rs`, `paramount_plus.rs`, `peacock.rs`
- Each implements `PlatformNormalizer` trait following HBO Max pattern
- Platform-specific genre mapping (e.g., Hulu Originals, Apple TV+ Originals)
- Extract subscription tier information where applicable
- Integrate with existing deep link generation
- Add exports in `normalizer/mod.rs`
- Unit tests with 80%+ coverage for each normalizer

---

### TASK-009: Implement Entity Resolution Database Persistence

**File:** `/workspaces/media-gateway/crates/ingestion/src/entity_resolution.rs` (modify)

**Description:** Entity resolution indices (EIDR, IMDB, TMDB mappings) exist only in memory HashMaps. All resolution state is lost on restart, breaking cross-platform content deduplication. The comment explicitly states "In production, this would connect to the database" but no database integration exists.

**Dependencies:** BATCH_001 TASK-005 (Database pool - completed)

**Acceptance Criteria:**
- Add `sqlx::PgPool` to `EntityResolver` struct
- Create `entity_mappings` table with (external_id, id_type, entity_id) schema
- Load entity indices from PostgreSQL on startup
- Persist new mappings on resolution (with upsert semantics)
- Add cache layer with Redis for hot path lookups
- Performance: resolution lookups <5ms from cache, <20ms from database
- Integration tests verify mapping persistence across restarts

---

### TASK-010: Implement Metadata Enrichment Pipeline

**File:** `/workspaces/media-gateway/crates/ingestion/src/pipeline.rs` (modify lines 480-488)

**Description:** The `enrich_metadata` function runs on a 24-hour schedule but contains only TODO comments. It queries nothing, regenerates nothing, and updates nothing. Content embeddings never get refreshed, quality scores are never computed, and stale metadata accumulates indefinitely.

**Dependencies:** BATCH_001 TASK-001 (Embeddings - completed), BATCH_003 TASK-006 (Qdrant indexing - completed)

**Acceptance Criteria:**
- Query database for content with embeddings older than configurable threshold (default: 7 days)
- Batch regenerate embeddings using OpenAI API (100 items per batch)
- Update Qdrant vectors with fresh embeddings
- Compute and store quality scores based on metadata completeness
- Emit `metadata.enriched` Kafka events for downstream processing
- Progress logging and error handling for long-running batch operations
- Integration tests verify stale content detection and update flow

---

### TASK-011: Expose SONA and Playback Services via API Gateway

**File:** `/workspaces/media-gateway/crates/api/src/routes.rs` (modify)

**Description:** The API Gateway only proxies 5 routes to discovery service. SONA (13 endpoints) and Playback (5 endpoints) are completely inaccessible through the gateway. Clients cannot get recommendations or manage playback sessions without direct service access, breaking the gateway abstraction.

**Dependencies:** None

**Acceptance Criteria:**
- Add proxy routes for SONA service (port 8082):
  - `POST /api/v1/recommendations` → SONA `/recommendations`
  - `POST /api/v1/personalization/score` → SONA `/personalization/score`
  - `GET /api/v1/experiments/{id}/metrics` → SONA `/experiments/{id}/metrics`
- Add proxy routes for Playback service (port 8086):
  - `POST /api/v1/sessions` → Playback `/sessions`
  - `GET /api/v1/sessions/{id}` → Playback `/sessions/{id}`
  - `PATCH /api/v1/sessions/{id}/position` → Playback `/sessions/{id}/position`
- Add proxy routes for Sync service watchlist/history endpoints
- Update service health checks to include new proxied services
- Integration tests verify all routes respond correctly

---

### TASK-012: Add Application Services to Docker Compose

**File:** `/workspaces/media-gateway/docker-compose.yml` (modify)

**Description:** Docker Compose only defines infrastructure services (postgres, redis, qdrant). Zero application services are defined. Developers cannot run the full stack with `docker-compose up`. Each service must be started manually, hindering development velocity and onboarding.

**Dependencies:** None

**Acceptance Criteria:**
- Add service definitions for all 7 microservices:
  - `api-gateway` (port 8080)
  - `discovery` (port 8081)
  - `sona` (port 8082)
  - `auth` (port 8083)
  - `sync` (port 8084)
  - `ingestion` (port 8085)
  - `playback` (port 8086)
- Configure `depends_on` with health check conditions for infrastructure
- Add environment variable configuration from `.env` file
- Add volume mounts for local development with hot reload
- Create `docker-compose.dev.yml` override for development settings
- Update `scripts/dev-setup.sh` to run migrations before starting services
- Full stack starts with single `docker-compose up -d` command

---

## Summary

| Task ID | Title | Files | Dependencies |
|---------|-------|-------|--------------|
| TASK-001 | Sync PostgreSQL Persistence | sync/src/repository.rs | B1-T005 |
| TASK-002 | Graph-Based Recommendations | sona/src/graph.rs | B1-T002, B1-T003 |
| TASK-003 | ONNX Runtime Integration | sona/src/inference.rs | B2-T002 |
| TASK-004 | Wire Rate Limiting to Auth | auth/src/main.rs, server.rs | B3-T007 |
| TASK-005 | Google OAuth Provider | auth/src/oauth/providers/google.rs | B1-T006 |
| TASK-006 | Discovery Personalization | discovery/src/search/personalization.rs | B3-T010, B4-T004 |
| TASK-007 | Intent Parser Caching | discovery/src/intent.rs | B2-T001 |
| TASK-008 | Missing Platform Normalizers | ingestion/src/normalizer/*.rs | B4-T006 |
| TASK-009 | Entity Resolution Persistence | ingestion/src/entity_resolution.rs | B1-T005 |
| TASK-010 | Metadata Enrichment Pipeline | ingestion/src/pipeline.rs | B1-T001, B3-T006 |
| TASK-011 | API Gateway Service Exposure | api/src/routes.rs | None |
| TASK-012 | Docker Compose Services | docker-compose.yml | None |

**Total Tasks:** 12
**Independent Tasks (can start immediately):** 4 (TASK-004, 007, 011, 012)
**Tasks with Dependencies:** 8

---

## Execution Order Recommendation

**Phase 1 (Parallel - No Dependencies):**
- TASK-004: Wire Rate Limiting (security critical)
- TASK-007: Intent Parser Caching (cost reduction)
- TASK-011: API Gateway Service Exposure (accessibility)
- TASK-012: Docker Compose Services (developer experience)

**Phase 2 (After Phase 1):**
- TASK-001: Sync PostgreSQL Persistence (data durability)
- TASK-005: Google OAuth Provider (user authentication)
- TASK-008: Missing Platform Normalizers (content coverage)
- TASK-009: Entity Resolution Persistence (data integrity)

**Phase 3 (After Phase 2):**
- TASK-002: Graph-Based Recommendations (recommendation quality)
- TASK-003: ONNX Runtime Integration (ML inference)
- TASK-006: Discovery Personalization (search quality)
- TASK-010: Metadata Enrichment Pipeline (data freshness)

---

## Agent Contributions

| Agent | Focus Area | Critical Gaps Found | Tasks Selected |
|-------|------------|---------------------|----------------|
| Agent 1 | Ingestion Service | 5 gaps (normalizers, enrichment, entity resolution) | TASK-008, 009, 010 |
| Agent 2 | Discovery Service | 4 gaps (personalization, caching, graph) | TASK-006, 007 |
| Agent 3 | Auth Service | 4 gaps (OAuth providers, rate limiting, middleware wiring) | TASK-004, 005 |
| Agent 4 | Playback Service | 3 gaps (minor - mostly complete) | - |
| Agent 5 | Sync Service | 6 gaps (persistence, multi-tenancy, metrics) | TASK-001 |
| Agent 6 | SONA Engine | 5 gaps (graph recommendations, ONNX, training) | TASK-002, 003 |
| Agent 7 | Core/MCP | 0 core gaps, MCP pending | - |
| Agent 8 | Integration | 12 gaps (gateway routing, Docker) | TASK-011, 012 |

**Total Gaps Analyzed:** 39 across 8 focus areas
**Tasks Selected:** 12 highest-priority, non-duplicating tasks

---

*Generated by 9-agent Claude-Flow swarm analysis*
