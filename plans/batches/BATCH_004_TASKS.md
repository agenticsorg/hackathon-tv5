# BATCH_004_TASKS.md - Media Gateway Action List

**Generated:** 2025-12-06
**Batch:** 004
**Previous Batches:** BATCH_001 (12 tasks), BATCH_002 (12 tasks), BATCH_003 (12 tasks) - All completed
**Analysis Method:** 9-agent Claude-Flow swarm parallel analysis
**SPARC Phase:** Refinement (Implementation)

---

## Action List

### TASK-001: Implement Query Spell Correction and Expansion

**File:** `/workspaces/media-gateway/crates/discovery/src/search/query_processor.rs` (new file)

**Description:** The search pipeline has NO query preprocessing layer. Misspelled queries like "sci fi movis 2023" go directly to vector/keyword search without correction. The IntentParser immediately calls GPT API without spell correction or synonym expansion, wasting API calls for correctable typos (30% of queries).

**Dependencies:** None

**Acceptance Criteria:**
- `QueryProcessor` struct with Levenshtein distance spell checker (max edit distance: 2)
- Dictionary built from top 10,000 movie/TV titles + genre keywords
- Synonym expansion: "sci-fi" → ["science fiction", "scifi", "sf"]
- Query rewriting: "movis about aliens" → "movies about aliens" + expanded variants
- Integration into `IntentParser::parse()` BEFORE GPT call (reduces API calls by 20-30%)
- Performance: spell check completes in <5ms for 99th percentile

---

### TASK-002: Implement Autocomplete and Query Suggestions Endpoint

**File:** `/workspaces/media-gateway/crates/discovery/src/search/autocomplete.rs` (new file), modify `server.rs`

**Description:** No `/api/v1/discovery/suggest` endpoint exists. Users typing "dark k" get no autocomplete suggestions for "dark knight", "dark crystal", etc. Modern search UX requires typeahead after 2 characters. The keyword search index (Tantivy) is only used for full queries, not prefix matching.

**Dependencies:** None

**Acceptance Criteria:**
- `AutocompleteService` with Trie data structure for O(k) prefix matching
- Pre-built index from top 50,000 titles, 10,000 actor/director names, genre combinations
- `GET /api/v1/discovery/suggest?q={prefix}&limit=10` endpoint
- Response: `{"suggestions": [{"text": "dark knight", "type": "title", "popularity": 0.95}]}`
- Performance: <20ms latency for 95th percentile
- Integration with Redis cache (TTL: 1 hour)

---

### TASK-003: Implement Faceted Search and Aggregations

**File:** `/workspaces/media-gateway/crates/discovery/src/search/facets.rs` (new file), modify `search/mod.rs`

**Description:** The `SearchResponse` struct returns results but NO facet aggregations. Users can't see "25 action movies, 12 comedies, 8 thrillers" after searching "movies 2023". The `SearchFilters` struct exists but doesn't compute counts. Faceted navigation is industry standard for discovery UX.

**Dependencies:** None

**Acceptance Criteria:**
- `FacetService` computes aggregations over search results
- Facet dimensions: genres, platforms, release_year (bucketed), rating (bucketed)
- Extend `SearchResponse` with `facets: HashMap<String, Vec<FacetCount>>`
- Facet computation integrated into `HybridSearchService::execute_search()`
- Performance: facet computation adds <30ms to search latency
- Integration test: search "action" returns facets showing genre distribution

---

### TASK-004: Implement A/B Testing Framework for SONA Recommendations

**File:** `/workspaces/media-gateway/crates/sona/src/ab_testing.rs` (new file)

**Description:** No infrastructure exists for A/B testing different recommendation algorithms, diversity thresholds, or LoRA configurations. Production optimization requires experimentation to measure impact on CTR, watch time, and retention. SPARC spec requires production-readiness.

**Dependencies:** BATCH_002 TASK-002 (LoRA storage - completed)

**Acceptance Criteria:**
- Experiment configuration system (variant definitions, traffic allocation)
- Consistent user-to-variant assignment (deterministic hash by user_id)
- `experiment_variant` field added to Recommendation type
- Metrics aggregation endpoint `/api/v1/experiments/{id}/metrics`
- Support multiple concurrent experiments with PostgreSQL persistence
- Database schema for experiments and variant assignments
- Integration tests with actual variant assignment

---

### TASK-005: Implement Refresh Token Rotation with Family Tracking

**File:** `/workspaces/media-gateway/crates/auth/src/server.rs` (lines 211-253)

**Description:** The `refresh_access_token` endpoint generates new tokens but does NOT implement refresh token rotation security best practice. No family tracking to detect token theft/replay attacks. OWASP recommends rotation with family tracking - when a revoked token is reused, ALL tokens in that family should be invalidated.

**Dependencies:** BATCH_001 TASK-006 (Redis auth - completed)

**Acceptance Criteria:**
- Add `token_family_id` to JWT claims (UUID generated at initial authorization)
- Store token family chain in Redis: `token_family:{family_id}` → Set of active JTIs
- On refresh: verify old token is in family, revoke ALL family tokens if reuse detected
- Integration test: attempt to reuse old refresh token → all tokens invalidated
- Latency impact: <5ms for Redis family lookup

---

### TASK-006: Implement HBO Max Platform Normalizer

**File:** `/workspaces/media-gateway/crates/ingestion/src/normalizer/hbo_max.rs` (new file)

**Description:** HBO Max has deep link generation in `deep_link.rs` but NO dedicated normalizer. Only generic fallback available via `GenericNormalizer`. Missing HBO-specific genre mappings (HBO Originals, Prestige Drama), subscription tier handling (ad-supported vs ad-free), and metadata extraction.

**Dependencies:** BATCH_001 TASK-004 (Repository pattern - completed)

**Acceptance Criteria:**
- Create `hbo_max.rs` implementing `PlatformNormalizer` trait
- HBO-specific genre mapping (Prestige Drama, HBO Originals, Max Originals)
- Handle HBO Max-specific availability (ad-supported vs ad-free tiers)
- Extract series/episode relationships and Max Originals badge
- Integrate with existing deep link generation
- Add to module exports in `normalizer/mod.rs`
- Unit tests with 80%+ coverage

---

### TASK-007: Implement Availability Sync Pipeline

**File:** `/workspaces/media-gateway/crates/ingestion/src/pipeline.rs` (lines 350-377)

**Description:** The `sync_availability` function runs every hour but contains only TODO stubs. Availability data extraction from `RawContent`, incremental updates, region-specific pricing, and content expiration updates are completely missing. Critical for accurate availability and expiring content notifications.

**Dependencies:** BATCH_003 TASK-005 (PostgreSQL upsert - completed)

**Acceptance Criteria:**
- Implement availability extraction logic in `sync_availability` (lines 366-377)
- Normalize each raw item to extract `AvailabilityInfo` struct
- Implement `update_availability` method in `repository.rs` (lines 354-366)
- Update database: regions, subscription_required, prices, available_until
- Emit `AvailabilityChangedEvent` via Kafka when availability changes
- Integration tests with real database verifying availability updates

---

### TASK-008: Implement Delta Sync for Offline Queue Operations

**File:** `/workspaces/media-gateway/crates/sync/src/sync/queue.rs` (lines 405-433)

**Description:** The `publish_operation` method in `OfflineSyncQueue` contains only placeholder comments. No actual conversion from `SyncOperation` to publisher messages, no delta encoding for partial state updates, and no compression. Full-state sync wastes bandwidth (70-90% reducible with deltas).

**Dependencies:** BATCH_003 TASK-002 (Offline sync queue - completed), BATCH_002 TASK-003 (PubNub publishing - completed)

**Acceptance Criteria:**
- Implement actual `SyncOperation` to `SyncMessage` conversion
- Add delta encoding for progress updates (only send changed fields)
- Implement batch compression for multiple operations
- Configurable delta sync strategy (full vs incremental)
- Metrics for bandwidth saved via delta sync
- Integration tests with actual publisher (not placeholder)

---

### TASK-009: Implement Reusable Pagination Utilities

**File:** `/workspaces/media-gateway/crates/core/src/pagination.rs` (new file)

**Description:** Search models have `limit`/`offset` fields but no reusable abstraction. Each crate reimplements pagination logic. No cursor-based pagination support, no page calculation utilities, and no standardized pagination response wrapper with HATEOAS links.

**Dependencies:** None

**Acceptance Criteria:**
- Create `Pagination` struct with offset/limit and cursor variants
- Add `PaginationParams` trait for request parsing
- Add `PaginatedResponse<T>` wrapper with metadata (total, has_more, links)
- Implement cursor encoding/decoding (base64 or opaque tokens)
- Helper methods: `next_page()`, `prev_page()`, `total_pages()`
- Integration with SearchQuery and SearchResult models
- Comprehensive unit tests for edge cases

---

### TASK-010: Implement Graceful Shutdown Coordinator

**File:** `/workspaces/media-gateway/crates/core/src/shutdown.rs` (new file)

**Description:** No centralized shutdown signal handling. Services don't coordinate graceful shutdown, active requests may be interrupted mid-flight, no drain period for completing in-progress work. Risk of data corruption during deployments and lost in-flight requests during rolling updates.

**Dependencies:** BATCH_002 TASK-007 (Observability - completed)

**Acceptance Criteria:**
- Create `ShutdownCoordinator` with signal handling (SIGTERM, SIGINT)
- Implement graceful shutdown phases: drain → stop accepting → wait → force
- Add `ShutdownHandle` for tasks to register for notification
- Support configurable grace periods per phase
- Integration with Tokio runtime and task cancellation
- Hook for flushing metrics/logs before exit
- Example integration with Actix-web server

---

### TASK-011: Implement Resume Position Calculation and Watch History

**File:** `/workspaces/media-gateway/crates/playback/src/session.rs` (lines 206-266)

**Description:** Position is updated but not analyzed for resume behavior. No "resume position" logic (ignore last 5%, don't resume if <30s watched). Watch history is lost after 24h Redis TTL. Users expect to resume where they left off, but not for nearly-complete or barely-started content.

**Dependencies:** BATCH_001 TASK-008 (Playback sessions - completed)

**Acceptance Criteria:**
- Add `calculate_resume_position()` function (None if <30s, None if >95% complete, else last position)
- Store watch history in PostgreSQL (user_id, content_id, resume_position, last_watched_at)
- On session creation, query watch history and return `resume_position_seconds: Option<u32>`
- Update watch history on session delete/update
- Integration test: Watch 50%, delete session, new session returns resume position

---

### TASK-012: Implement MCP Protocol 2024-11-05 Missing Methods

**File:** `/workspaces/media-gateway/apps/mcp-server/src/server.ts` (lines 14-43)

**Description:** Missing standard MCP protocol methods: `ping`, `notifications/initialized`, `notifications/cancelled`, `logging/setLevel`, `completion/complete`. The `ping` method is essential for SSE connection health checks, cancellation prevents wasted compute on abandoned requests.

**Dependencies:** BATCH_003 TASK-003 (MCP timeouts - completed)

**Acceptance Criteria:**
- Implement `ping` method returning `{}`
- Implement `notifications/initialized` acknowledgment
- Implement `notifications/cancelled` for request cancellation
- Implement `logging/setLevel` for dynamic log level adjustment
- Implement `completion/complete` for autocomplete suggestions
- Protocol compliance tests against MCP spec 2024-11-05
- Integration tests for each new method

---

## Summary

| Task ID | Title | Files | Dependencies |
|---------|-------|-------|--------------|
| TASK-001 | Query Spell Correction | discovery/src/search/query_processor.rs | None |
| TASK-002 | Autocomplete Suggestions | discovery/src/search/autocomplete.rs | None |
| TASK-003 | Faceted Search | discovery/src/search/facets.rs | None |
| TASK-004 | A/B Testing Framework | sona/src/ab_testing.rs | B2-T002 |
| TASK-005 | Refresh Token Rotation | auth/src/server.rs | B1-T006 |
| TASK-006 | HBO Max Normalizer | ingestion/src/normalizer/hbo_max.rs | B1-T004 |
| TASK-007 | Availability Sync Pipeline | ingestion/src/pipeline.rs | B3-T005 |
| TASK-008 | Delta Sync for Offline Queue | sync/src/sync/queue.rs | B3-T002, B2-T003 |
| TASK-009 | Pagination Utilities | core/src/pagination.rs | None |
| TASK-010 | Graceful Shutdown | core/src/shutdown.rs | B2-T007 |
| TASK-011 | Resume Position Logic | playback/src/session.rs | B1-T008 |
| TASK-012 | MCP Protocol Compliance | apps/mcp-server/src/server.ts | B3-T003 |

**Total Tasks:** 12
**Independent Tasks (can start immediately):** 5 (TASK-001, 002, 003, 009)
**Tasks with Dependencies:** 8

---

## Execution Order Recommendation

**Phase 1 (Parallel - No Dependencies):**
- TASK-001: Query Spell Correction (search quality)
- TASK-002: Autocomplete Suggestions (user experience)
- TASK-003: Faceted Search (discovery UX)
- TASK-009: Pagination Utilities (code reuse)

**Phase 2 (After Phase 1):**
- TASK-004: A/B Testing Framework (recommendation optimization)
- TASK-005: Refresh Token Rotation (security hardening)
- TASK-006: HBO Max Normalizer (platform coverage)
- TASK-010: Graceful Shutdown (production reliability)

**Phase 3 (After Phase 2):**
- TASK-007: Availability Sync Pipeline (data freshness)
- TASK-008: Delta Sync for Offline Queue (bandwidth optimization)
- TASK-011: Resume Position Logic (playback UX)
- TASK-012: MCP Protocol Compliance (spec adherence)

---

## Agent Contributions

| Agent | Focus Area | Gaps Identified | Tasks Selected |
|-------|------------|-----------------|----------------|
| Agent 1 | API Gateway | 4 gaps | - (lower priority than service gaps) |
| Agent 2 | Discovery Service | 4 gaps | TASK-001, TASK-002, TASK-003 |
| Agent 3 | SONA Engine | 4 gaps | TASK-004 |
| Agent 4 | Sync Service | 4 gaps | TASK-008 |
| Agent 5 | Auth Service | 4 gaps | TASK-005 |
| Agent 6 | Ingestion Service | 4 gaps | TASK-006, TASK-007 |
| Agent 7 | Core Crate | 4 gaps | TASK-009, TASK-010 |
| Agent 8 | Playback Service | 4 gaps | TASK-011 |
| Agent 9 | MCP Server | 4 gaps | TASK-012 |

**Total Gaps Analyzed:** 36 across 9 focus areas
**Tasks Selected:** 12 highest-priority, non-duplicating tasks

---

*Generated by 9-agent Claude-Flow swarm analysis*
