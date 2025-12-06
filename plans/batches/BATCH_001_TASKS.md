# BATCH_001_TASKS.md
## First Actionable Task Batch - Media Gateway Implementation

**Generated**: 2025-12-06
**Source**: 9-Agent Swarm Analysis of ACTION_LIST.md + Codebase
**Priority**: Phase 1 - Critical Foundation Tasks

---

## TASK-001: Implement OpenAI Embedding Generation

**Title**: Replace mock embedding with OpenAI text-embedding-3-small API
**Module**: `crates/discovery/src/search/vector.rs`
**Lines**: 134-138

**Description**:
The `generate_embedding()` function currently returns `vec![0.0; self.dimension]` (mock zeros). Replace with actual OpenAI API call using `reqwest` client to `https://api.openai.com/v1/embeddings` with model `text-embedding-3-small` (768 dimensions).

**Implementation**:
1. Add `OPENAI_API_KEY` environment variable handling
2. Create async POST request with JSON body `{"input": query, "model": "text-embedding-3-small"}`
3. Parse response `data[0].embedding` array
4. Add timeout (5s) and retry logic (3 attempts with exponential backoff)
5. Return `Vec<f32>` with 768 dimensions

**Dependencies**: None
**Acceptance**: Semantic search returns non-zero similarity scores for related queries

---

## TASK-002: Implement SONA Collaborative Filtering Database Layer

**Title**: Add SQLx persistence for collaborative filtering in SONA
**Module**: `crates/sona/src/collaborative.rs`
**Lines**: 50-61

**Description**:
Replace simulated `find_similar_users()` and `get_user_preferences()` functions that return `Vec::new()` and `HashMap::new()` with actual PostgreSQL queries using SQLx.

**Implementation**:
1. Add `sqlx::PgPool` to `CollaborativeEngine` struct
2. Implement `find_similar_users()`: Query `user_interactions` table, compute cosine similarity via SQL, return top-k users
3. Implement `get_user_preferences()`: Query `user_content_ratings` table, aggregate by content_id
4. Add connection pool initialization in constructor

**Dependencies**: TASK-005 (shared database pool setup)
**Acceptance**: Similar users query returns actual user IDs with similarity scores

---

## TASK-003: Implement SONA Content-Based Filtering Database Layer

**Title**: Add SQLx persistence for content-based recommendations
**Module**: `crates/sona/src/content_based.rs`
**Lines**: 38-50

**Description**:
Replace simulated content similarity functions with actual PostgreSQL + Qdrant vector queries.

**Implementation**:
1. Add `sqlx::PgPool` and `qdrant_client::QdrantClient` to `ContentBasedEngine`
2. Implement `get_content_features()`: Query `content_metadata` table for genre, cast, keywords
3. Implement `compute_similarity()`: Use Qdrant vector search for semantic similarity
4. Add batch query optimization for multiple content items

**Dependencies**: TASK-001 (embeddings), TASK-005 (database pool)
**Acceptance**: Content recommendations based on actual content metadata and vector similarity

---

## TASK-004: Implement Ingestion Pipeline Database Persistence

**Title**: Add repository pattern for ingestion database operations
**Module**: `crates/ingestion/src/pipeline.rs`
**Lines**: 290, 313, 329-341

**Description**:
Replace 7 TODO comments with actual database persistence using the repository pattern.

**Implementation**:
1. Create `ContentRepository` trait with `insert()`, `update_availability()`, `find_expiring()` methods
2. Implement `PostgresContentRepository` with SQLx
3. At line 290: Call `repo.insert(&content_item)` after transformation
4. At line 313: Call `repo.update_availability(content_id, &availability)`
5. At lines 329-341: Call `repo.find_expiring_within(Duration::days(7))`
6. Add transaction support for batch operations

**Dependencies**: TASK-005 (database pool)
**Acceptance**: Content items persisted to database after ingestion; expiration queries return valid results

---

## TASK-005: Create Shared Database Connection Pool Module

**Title**: Extract shared SQLx PostgreSQL connection pool
**Module**: `crates/core/src/database.rs` (new file)

**Description**:
Multiple crates (SONA, ingestion, auth) need database access. Create a shared connection pool module to avoid duplication.

**Implementation**:
1. Add `database.rs` to `crates/core/src/`
2. Export in `lib.rs`: `pub mod database;`
3. Implement `DatabasePool` struct wrapping `sqlx::PgPool`
4. Add `new(database_url: &str)` constructor with connection options (max_connections: 20, idle_timeout: 10min)
5. Add health check method `is_healthy() -> bool`
6. Add `Cargo.toml` dependency: `sqlx = { workspace = true }`

**Dependencies**: None
**Acceptance**: `cargo build` succeeds; pool connects to PostgreSQL

---

## TASK-006: Migrate Auth Storage from HashMap to Redis

**Title**: Replace in-memory HashMaps with Redis for auth state
**Module**: `crates/auth/src/server.rs`
**Lines**: 32-34

**Description**:
Replace `Arc<RwLock<HashMap<...>>>` for PKCE sessions, auth codes, and device codes with Redis-backed storage with appropriate TTLs.

**Implementation**:
1. Add Redis client to `AuthServer` struct
2. Replace `pkce_sessions` HashMap: `SET pkce:{id} {data} EX 600` (10 min TTL)
3. Replace `auth_codes` HashMap: `SET authcode:{code} {data} EX 300` (5 min TTL)
4. Replace `device_codes` HashMap: `SET devicecode:{code} {data} EX 900` (15 min TTL)
5. Add JSON serialization for stored values
6. Update all read/write operations to use Redis GET/SET

**Dependencies**: None
**Acceptance**: Auth flows work with Redis; tokens expire correctly

---

## TASK-007: Implement PubNub Subscribe with Message Callbacks

**Title**: Complete PubNub real-time subscription
**Module**: `crates/sync/src/pubnub.rs`
**Lines**: 104-109

**Description**:
Replace placeholder `subscribe()` that only logs with actual PubNub subscription and message handling.

**Implementation**:
1. Use `pubnub` crate's async subscription API
2. Create `MessageHandler` callback trait
3. Implement subscription loop with `pubnub.subscribe(channels).await`
4. Parse incoming messages and dispatch to handlers
5. Add reconnection logic on connection drops
6. Add unsubscribe support for cleanup

**Dependencies**: None
**Acceptance**: Messages published to PubNub channels are received by subscribers

---

## TASK-008: Wire Discovery Service Routes to Server

**Title**: Connect discovery endpoints to Actix server
**Module**: `crates/discovery/src/main.rs`

**Description**:
The `main.rs` only has a health check endpoint. The `server.rs` has `configure_routes()` but it's not being used.

**Implementation**:
1. Import `server::configure_routes` in main.rs
2. Add `.configure(configure_routes)` to Actix App builder
3. Initialize required services (VectorSearch, etc.) in main
4. Pass services via app_data to routes
5. Test `/api/v1/search` endpoint responds

**Dependencies**: TASK-001 (embeddings for search)
**Acceptance**: `curl http://localhost:8080/api/v1/search?q=test` returns JSON response

---

## TASK-009: Implement Playback Session Management

**Title**: Add complete session lifecycle to playback service
**Module**: `crates/playback/src/main.rs`

**Description**:
Currently only 34 lines with health check. Needs full session management for video playback.

**Implementation**:
1. Add `SessionManager` struct with create/get/update/delete operations
2. Create POST `/sessions` endpoint for session creation
3. Create GET `/sessions/{id}` endpoint for session retrieval
4. Create PATCH `/sessions/{id}/position` for progress updates
5. Add Redis storage for session state with 24h TTL
6. Add WebSocket endpoint for real-time position sync

**Dependencies**: None
**Acceptance**: Session CRUD operations work; position updates persist

---

## TASK-010: Add num_cpus Dependency for Thread Pool Sizing

**Title**: Add num_cpus for dynamic worker thread configuration
**Module**: `crates/api/Cargo.toml`, `crates/discovery/Cargo.toml`

**Description**:
Multiple crates hardcode thread counts. Use `num_cpus` for optimal sizing.

**Implementation**:
1. Add `num_cpus = "1.16"` to workspace dependencies in root `Cargo.toml`
2. Add `num_cpus = { workspace = true }` to `crates/api/Cargo.toml`
3. Add `num_cpus = { workspace = true }` to `crates/discovery/Cargo.toml`
4. Replace hardcoded thread counts with `num_cpus::get()` calls
5. Document minimum (2) and maximum (32) thread bounds

**Dependencies**: None
**Acceptance**: `cargo build` succeeds; thread count matches CPU cores

---

## TASK-011: Extract Shared Cosine Similarity Utility

**Title**: Deduplicate cosine_similarity implementations
**Module**: `crates/core/src/math.rs` (new file)

**Description**:
12 instances of `cosine_similarity()` exist across discovery, SONA, and search crates. Extract to shared module.

**Implementation**:
1. Create `crates/core/src/math.rs` with:
   ```rust
   pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32
   pub fn normalize_vector(v: &mut [f32])
   pub fn dot_product(a: &[f32], b: &[f32]) -> f32
   ```
2. Export in `lib.rs`: `pub mod math;`
3. Replace all 12 instances with `media_gateway_core::math::cosine_similarity`
4. Add SIMD optimization hint for future enhancement

**Dependencies**: None
**Acceptance**: All crates compile; no duplicate cosine_similarity definitions

---

## TASK-012: Create Docker Compose for Local Development

**Title**: Add docker-compose.yml with all required services
**Module**: Root directory `docker-compose.yml`

**Description**:
No local development environment exists. Create Docker Compose with PostgreSQL, Redis, and Qdrant.

**Implementation**:
1. Create `docker-compose.yml` with services:
   - `postgres:16` on port 5432 with health check
   - `redis:7` on port 6379 with health check
   - `qdrant/qdrant:latest` on ports 6333/6334
2. Add volumes for data persistence
3. Add `.env.example` with required environment variables
4. Add `depends_on` with health conditions
5. Create `scripts/dev-setup.sh` for first-time setup

**Dependencies**: None
**Acceptance**: `docker-compose up -d` starts all services; health checks pass

---

## Execution Order

**Parallel Group 1** (No dependencies):
- TASK-005: Database pool
- TASK-006: Redis migration
- TASK-007: PubNub subscribe
- TASK-010: num_cpus dependency
- TASK-011: Cosine similarity utility
- TASK-012: Docker Compose

**Parallel Group 2** (Requires Group 1):
- TASK-001: Embeddings (after TASK-005)
- TASK-004: Ingestion persistence (after TASK-005)
- TASK-009: Playback sessions (after TASK-006)

**Parallel Group 3** (Requires Group 2):
- TASK-002: SONA collaborative (after TASK-001, TASK-005)
- TASK-003: SONA content-based (after TASK-001, TASK-005)
- TASK-008: Discovery routes (after TASK-001)

---

## Summary

| Task | Module | Priority | Dependencies |
|------|--------|----------|--------------|
| TASK-001 | discovery/vector.rs | Critical | None |
| TASK-002 | sona/collaborative.rs | High | TASK-005 |
| TASK-003 | sona/content_based.rs | High | TASK-001, TASK-005 |
| TASK-004 | ingestion/pipeline.rs | Critical | TASK-005 |
| TASK-005 | core/database.rs | Critical | None |
| TASK-006 | auth/server.rs | Critical | None |
| TASK-007 | sync/pubnub.rs | High | None |
| TASK-008 | discovery/main.rs | High | TASK-001 |
| TASK-009 | playback/main.rs | Medium | None |
| TASK-010 | Cargo.toml | Low | None |
| TASK-011 | core/math.rs | Medium | None |
| TASK-012 | docker-compose.yml | Critical | None |

**Total Tasks**: 12
**Critical**: 5 (TASK-001, 004, 005, 006, 012)
**High**: 4 (TASK-002, 003, 007, 008)
**Medium**: 2 (TASK-009, 011)
**Low**: 1 (TASK-010)
