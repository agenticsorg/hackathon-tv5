# Media Gateway Implementation Action List

**Generated**: 2025-12-06
**Based On**: SPARC Master Documents + 9-Agent Codebase Analysis
**Status**: Phase 2 - Implementation Gaps Identified

---

## CRITICAL PATH ACTIONS (Blocking Production)

### ACTION-001: Implement Embedding Generation Service
- **Module**: `crates/discovery/src/search/vector.rs`
- **Lines**: 44, 135
- **Current State**: Returns `vec![0.0; dimension]` (mock zeros)
- **Required**: Integration with OpenAI text-embedding-3-small API
- **Files to Modify**:
  - `crates/discovery/src/search/vector.rs` - Replace mock with real API call
  - `crates/discovery/src/embedding.rs` - Complete batch embedding implementation
- **Constraints**: Must support 768-dim vectors, batch size 32, <25ms per embedding
- **Dependencies**: None
- **Effort**: 8 hours

### ACTION-002: Implement Graph Search Algorithm
- **Module**: `crates/discovery/src/search/`
- **Current State**: Only vector and keyword search exist; graph search entirely missing
- **Required**: Graph traversal for content relationships (30% weight in RRF)
- **Files to Create**:
  - `crates/discovery/src/search/graph.rs` - GraphSearch struct with Neo4j/Ruvector integration
- **Files to Modify**:
  - `crates/discovery/src/search/mod.rs` - Add graph_results to HybridSearchService
- **Constraints**: O(d * b^h) complexity, max 2 hops, integrate with RRF K=60
- **Dependencies**: ACTION-001 (embeddings for similarity)
- **Effort**: 16 hours

### ACTION-003: Implement SONA Database Persistence Layer
- **Module**: `crates/sona/src/`
- **Current State**: All `get_*_candidates` functions return `Vec::new()` (empty)
- **Required**: SQLx queries for user profiles, viewing history, LoRA adapters
- **Files to Create**:
  - `crates/sona/src/repository/mod.rs` - Repository trait definitions
  - `crates/sona/src/repository/user_profile.rs` - UserProfile CRUD
  - `crates/sona/src/repository/lora_adapter.rs` - LoRA storage/retrieval
  - `crates/sona/src/repository/viewing_history.rs` - Viewing events
- **Files to Modify**:
  - `crates/sona/src/collaborative.rs` - Replace simulated data
  - `crates/sona/src/content_based.rs` - Replace simulated data
  - `crates/sona/src/context.rs` - Replace simulated data
  - `crates/sona/src/recommendation.rs` - Wire up real candidates
- **Constraints**: Use sqlx with PostgreSQL, connection pooling, prepared statements
- **Dependencies**: None
- **Effort**: 20 hours

### ACTION-004: Implement Ingestion Database Persistence
- **Module**: `crates/ingestion/src/pipeline.rs`
- **Lines**: 290, 313, 329-330, 339-341
- **Current State**: 7 TODO comments for database operations
- **Required**: Persist ingested content to PostgreSQL
- **Files to Create**:
  - `crates/ingestion/src/repository/mod.rs` - Repository pattern
  - `crates/ingestion/src/repository/content.rs` - Content CRUD
  - `crates/ingestion/src/repository/availability.rs` - Availability updates
- **Files to Modify**:
  - `crates/ingestion/src/pipeline.rs` - Replace TODOs with repository calls
  - `crates/ingestion/src/main.rs` - Initialize pipeline with DB connection
- **Constraints**: Batch inserts (100 items), upsert semantics, 500 items/sec target
- **Dependencies**: None
- **Effort**: 16 hours

### ACTION-005: Implement Playback Service
- **Module**: `crates/playback/src/`
- **Current State**: Only 34 lines, health check only
- **Required**: Session management, device handoff, deep link generation
- **Files to Create**:
  - `crates/playback/src/session.rs` - PlaybackSession struct and management
  - `crates/playback/src/device_handoff.rs` - Cross-device transfer protocol
  - `crates/playback/src/handlers.rs` - HTTP endpoint handlers
  - `crates/playback/src/repository.rs` - Session persistence
- **Files to Modify**:
  - `crates/playback/src/main.rs` - Full server with routes
  - `crates/playback/src/lib.rs` - Module exports
- **Constraints**: 5-second command expiry, device capability validation
- **Dependencies**: ACTION-003 (user context from SONA)
- **Effort**: 20 hours

### ACTION-006: Replace Auth In-Memory Storage with Redis
- **Module**: `crates/auth/src/server.rs`
- **Lines**: 31-35
- **Current State**: HashMap for PKCE sessions, auth codes, device codes
- **Required**: Redis-backed storage for multi-instance deployment
- **Files to Create**:
  - `crates/auth/src/storage/mod.rs` - Storage trait
  - `crates/auth/src/storage/redis.rs` - Redis implementation
- **Files to Modify**:
  - `crates/auth/src/server.rs` - Use storage trait instead of HashMap
- **Constraints**: 10-minute PKCE expiry, 15-minute device code expiry
- **Dependencies**: None
- **Effort**: 4 hours

### ACTION-007: Complete PubNub Subscribe Implementation
- **Module**: `crates/sync/src/pubnub.rs`
- **Lines**: 104-108
- **Current State**: Placeholder that only logs
- **Required**: Actual WebSocket/SSE subscription for real-time sync
- **Files to Modify**:
  - `crates/sync/src/pubnub.rs` - Implement real subscription
  - `crates/sync/src/websocket.rs` - Complete WebSocket handler
- **Constraints**: <100ms message delivery, 300s heartbeat TTL
- **Dependencies**: None
- **Effort**: 4 hours

---

## HIGH PRIORITY ACTIONS (Required for MVP)

### ACTION-008: Wire Discovery Service Routes
- **Module**: `crates/discovery/src/main.rs`
- **Current State**: Only health check endpoint
- **Required**: Import and use server.rs routes
- **Files to Modify**:
  - `crates/discovery/src/main.rs` - Use `server::start_server`
- **Dependencies**: ACTION-001, ACTION-002
- **Effort**: 2 hours

### ACTION-009: Wire SONA Service Routes
- **Module**: `crates/sona/src/main.rs`
- **Current State**: Only health check endpoint
- **Required**: Import and use server.rs routes
- **Files to Modify**:
  - `crates/sona/src/main.rs` - Use `server::start_server`
- **Dependencies**: ACTION-003
- **Effort**: 2 hours

### ACTION-010: Implement User Preference Scoring in Search
- **Module**: `crates/discovery/src/search/mod.rs`
- **Line**: 107 (TODO comment)
- **Current State**: Search results not personalized
- **Required**: Integration with SONA for user preference scoring
- **Files to Modify**:
  - `crates/discovery/src/search/mod.rs` - Add SONA client call
- **Constraints**: <5ms added latency
- **Dependencies**: ACTION-003
- **Effort**: 4 hours

### ACTION-011: Add num_cpus Dependency to Discovery
- **Module**: `crates/discovery/Cargo.toml`
- **Current State**: Missing dependency used in server.rs line 229
- **Required**: Add num_cpus crate
- **Files to Modify**:
  - `crates/discovery/Cargo.toml` - Add `num_cpus = "1.16"`
- **Dependencies**: None
- **Effort**: 0.5 hours

### ACTION-012: Implement Intent Parser GPT Integration
- **Module**: `crates/discovery/src/intent.rs`
- **Line**: 70 (TODO)
- **Current State**: Only fallback pattern matching, no GPT call
- **Required**: GPT-4o-mini integration for NL intent parsing
- **Files to Modify**:
  - `crates/discovery/src/intent.rs` - Implement OpenAI API call
- **Constraints**: <85ms latency, implement caching
- **Dependencies**: None
- **Effort**: 6 hours

### ACTION-013: Implement Batch Embedding API
- **Module**: `crates/discovery/src/embedding.rs`
- **Line**: 85 (TODO)
- **Current State**: Batch API not implemented
- **Required**: Batch embedding for ingestion pipeline
- **Files to Modify**:
  - `crates/discovery/src/embedding.rs` - Add batch_generate method
- **Constraints**: 32 items per batch, O(n*d/p) complexity
- **Dependencies**: ACTION-001
- **Effort**: 4 hours

---

## INFRASTRUCTURE ACTIONS (Pre-Production)

### ACTION-014: Create Terraform Infrastructure
- **Module**: `infrastructure/terraform/`
- **Current State**: Entirely missing
- **Required**: GCP infrastructure provisioning
- **Files to Create**:
  - `infrastructure/terraform/main.tf` - Provider configuration
  - `infrastructure/terraform/gke.tf` - GKE cluster
  - `infrastructure/terraform/cloudsql.tf` - PostgreSQL instance
  - `infrastructure/terraform/memorystore.tf` - Redis instance
  - `infrastructure/terraform/vpc.tf` - VPC networking
  - `infrastructure/terraform/iam.tf` - Service accounts
  - `infrastructure/terraform/variables.tf` - Input variables
  - `infrastructure/terraform/outputs.tf` - Output values
- **Constraints**: Multi-zone HA, private networking, Workload Identity
- **Dependencies**: None
- **Effort**: 20 hours

### ACTION-015: Create Database StatefulSets
- **Module**: `infrastructure/k8s/databases/`
- **Current State**: Missing
- **Required**: In-cluster database deployments or managed service connections
- **Files to Create**:
  - `infrastructure/k8s/databases/qdrant.yaml` - Qdrant StatefulSet + PVC
  - `infrastructure/k8s/databases/cloudsql-proxy.yaml` - CloudSQL Proxy sidecar config
- **Constraints**: Persistent storage, backup-enabled
- **Dependencies**: ACTION-014
- **Effort**: 8 hours

### ACTION-016: Deploy Monitoring Stack
- **Module**: `infrastructure/k8s/monitoring/`
- **Current State**: Only example configs exist
- **Required**: Production observability
- **Files to Create**:
  - `infrastructure/k8s/monitoring/prometheus-operator.yaml`
  - `infrastructure/k8s/monitoring/grafana.yaml`
  - `infrastructure/k8s/monitoring/servicemonitors.yaml`
  - `infrastructure/k8s/monitoring/alerting-rules.yaml`
  - `infrastructure/k8s/monitoring/dashboards/` (7 dashboard JSONs)
- **Constraints**: 30-day metrics retention, 10% trace sampling
- **Dependencies**: ACTION-014
- **Effort**: 12 hours

### ACTION-017: Implement Backup Automation
- **Module**: `infrastructure/backup/`
- **Current State**: Missing
- **Required**: Automated database and K8s backups
- **Files to Create**:
  - `infrastructure/k8s/backup/velero.yaml` - Velero deployment
  - `infrastructure/backup/postgres-backup.yaml` - CronJob for pg_dump
  - `infrastructure/backup/qdrant-snapshot.yaml` - CronJob for Qdrant snapshots
- **Constraints**: Daily backups, 7-day retention, RPO <1 hour
- **Dependencies**: ACTION-014, ACTION-015
- **Effort**: 6 hours

### ACTION-018: Add ResourceQuotas and LimitRanges
- **Module**: `infrastructure/k8s/namespace.yaml`
- **Current State**: Only namespace definitions
- **Required**: Cost control and resource governance
- **Files to Modify**:
  - `infrastructure/k8s/namespace.yaml` - Add ResourceQuota and LimitRange
- **Dependencies**: None
- **Effort**: 2 hours

---

## TESTING ACTIONS (Quality Gates)

### ACTION-019: Add Integration Tests for Discovery
- **Module**: `crates/discovery/tests/`
- **Current State**: Unit tests only
- **Required**: Real database integration tests
- **Files to Create**:
  - `crates/discovery/tests/integration/mod.rs`
  - `crates/discovery/tests/integration/search_test.rs`
  - `crates/discovery/tests/integration/hybrid_search_test.rs`
- **Constraints**: Use testcontainers for PostgreSQL/Qdrant
- **Dependencies**: ACTION-001, ACTION-002
- **Effort**: 8 hours

### ACTION-020: Add Integration Tests for SONA
- **Module**: `crates/sona/tests/`
- **Current State**: Unit tests only
- **Required**: Real database integration tests
- **Files to Create**:
  - `crates/sona/tests/integration/mod.rs`
  - `crates/sona/tests/integration/recommendation_test.rs`
  - `crates/sona/tests/integration/lora_test.rs`
- **Constraints**: Use testcontainers for PostgreSQL
- **Dependencies**: ACTION-003
- **Effort**: 8 hours

### ACTION-021: Add E2E Test Suite
- **Module**: `tests/e2e/`
- **Current State**: Missing
- **Required**: Full user journey tests
- **Files to Create**:
  - `tests/e2e/Cargo.toml`
  - `tests/e2e/src/lib.rs`
  - `tests/e2e/src/search_flow.rs`
  - `tests/e2e/src/recommendation_flow.rs`
  - `tests/e2e/src/sync_flow.rs`
- **Constraints**: 10 critical paths, <5 min total runtime
- **Dependencies**: ACTION-008, ACTION-009, ACTION-005
- **Effort**: 12 hours

### ACTION-022: Add Performance Benchmarks
- **Module**: `benches/`
- **Current State**: Missing
- **Required**: Latency and throughput benchmarks
- **Files to Create**:
  - `benches/search_benchmark.rs` - Search latency (p50: 150ms, p95: 400ms)
  - `benches/recommendation_benchmark.rs` - SONA latency (p50: 2ms, p95: 5ms)
  - `benches/crdt_benchmark.rs` - CRDT merge throughput (10K ops/s)
- **Constraints**: Use criterion crate, CI integration
- **Dependencies**: ACTION-001, ACTION-003
- **Effort**: 6 hours

---

## REFINEMENT ACTIONS (Code Quality)

### ACTION-023: Extract Shared Similarity Utilities
- **Module**: `crates/common/`
- **Current State**: Cosine similarity duplicated in 3+ places
- **Required**: Shared utility crate
- **Files to Create**:
  - `crates/common/Cargo.toml`
  - `crates/common/src/lib.rs`
  - `crates/common/src/similarity.rs`
- **Files to Modify**:
  - `Cargo.toml` - Add common to workspace
  - `crates/discovery/Cargo.toml` - Add common dependency
  - `crates/sona/Cargo.toml` - Add common dependency
- **Dependencies**: None
- **Effort**: 2 hours

### ACTION-024: Create Centralized HTTP Client
- **Module**: `apps/mcp-server/src/`
- **Current State**: fetch() calls without timeout or retry
- **Required**: Shared HTTP client with timeout and retry
- **Files to Create**:
  - `apps/mcp-server/src/http-client.ts`
- **Files to Modify**:
  - All tool files - Use shared client
- **Constraints**: 5s timeout, 3 retries with exponential backoff
- **Dependencies**: None
- **Effort**: 2 hours

### ACTION-025: Add MCP Server Unit Tests
- **Module**: `apps/mcp-server/`
- **Current State**: No tests
- **Required**: Unit tests for all 7 tools
- **Files to Create**:
  - `apps/mcp-server/src/__tests__/tools/semantic_search.test.ts`
  - `apps/mcp-server/src/__tests__/tools/get_recommendations.test.ts`
  - (5 more tool tests)
- **Constraints**: >80% coverage, mock backend services
- **Dependencies**: None
- **Effort**: 8 hours

### ACTION-026: Implement Response Caching in MCP
- **Module**: `apps/mcp-server/src/`
- **Current State**: node-cache installed but unused
- **Required**: Cache backend responses
- **Files to Modify**:
  - `apps/mcp-server/src/tools/*.ts` - Add caching layer
  - `apps/mcp-server/src/resources/index.ts` - Cache resources
- **Constraints**: 30s TTL for search, 5min for content details
- **Dependencies**: None
- **Effort**: 3 hours

---

## LOCAL DEVELOPMENT ACTIONS

### ACTION-027: Create Docker Compose for Local Dev
- **Module**: `docker-compose.yml`
- **Current State**: Missing
- **Required**: Full stack local development
- **Files to Create**:
  - `docker-compose.yml` - All services + databases
  - `docker-compose.override.yml` - Dev overrides
  - `.env.example` - Environment template
- **Constraints**: Hot reload, volume mounts for code
- **Dependencies**: None
- **Effort**: 4 hours

### ACTION-028: Create Database Seed Scripts
- **Module**: `scripts/seed/`
- **Current State**: Missing
- **Required**: Sample data for development
- **Files to Create**:
  - `scripts/seed/content.sql` - 1000 sample content items
  - `scripts/seed/users.sql` - 10 test users
  - `scripts/seed/seed.sh` - Orchestration script
- **Dependencies**: None
- **Effort**: 3 hours

---

## EXECUTION ORDER (Dependency-Sorted)

### Phase 1: Core Functionality (Week 1-2)
1. ACTION-001 (Embeddings) - Unblocks vector search
2. ACTION-003 (SONA DB) - Unblocks recommendations
3. ACTION-004 (Ingestion DB) - Unblocks content pipeline
4. ACTION-006 (Auth Redis) - Unblocks multi-instance auth
5. ACTION-007 (PubNub) - Unblocks real-time sync

### Phase 2: Service Integration (Week 2-3)
6. ACTION-002 (Graph Search) - Requires ACTION-001
7. ACTION-005 (Playback) - Requires ACTION-003
8. ACTION-008 (Discovery Routes) - Requires ACTION-001, ACTION-002
9. ACTION-009 (SONA Routes) - Requires ACTION-003
10. ACTION-010 (Personalization) - Requires ACTION-003
11. ACTION-011 (num_cpus) - No dependencies
12. ACTION-012 (Intent Parser) - No dependencies
13. ACTION-013 (Batch Embeddings) - Requires ACTION-001

### Phase 3: Infrastructure (Week 3-4)
14. ACTION-014 (Terraform) - No dependencies
15. ACTION-015 (DB StatefulSets) - Requires ACTION-014
16. ACTION-016 (Monitoring) - Requires ACTION-014
17. ACTION-017 (Backups) - Requires ACTION-014, ACTION-015
18. ACTION-018 (Resource Quotas) - No dependencies

### Phase 4: Testing & Quality (Week 4-5)
19. ACTION-019 (Discovery Integration Tests) - Requires ACTION-001, ACTION-002
20. ACTION-020 (SONA Integration Tests) - Requires ACTION-003
21. ACTION-021 (E2E Tests) - Requires Phase 2 complete
22. ACTION-022 (Benchmarks) - Requires ACTION-001, ACTION-003
23. ACTION-023 (Shared Utilities) - No dependencies
24. ACTION-024 (HTTP Client) - No dependencies
25. ACTION-025 (MCP Tests) - No dependencies
26. ACTION-026 (MCP Caching) - No dependencies

### Phase 5: Developer Experience (Week 5)
27. ACTION-027 (Docker Compose) - No dependencies
28. ACTION-028 (Seed Scripts) - No dependencies

---

## SUMMARY

| Priority | Actions | Total Effort |
|----------|---------|--------------|
| CRITICAL | 7 | 88 hours |
| HIGH | 6 | 18.5 hours |
| INFRASTRUCTURE | 5 | 48 hours |
| TESTING | 4 | 34 hours |
| REFINEMENT | 4 | 15 hours |
| LOCAL DEV | 2 | 7 hours |
| **TOTAL** | **28** | **210.5 hours** |

---

**Document Generated By**: 9-Agent Claude-Flow Analysis Swarm
**SPARC Compliance**: Verified against all 5 master documents
**Next Step**: Execute Phase 1 actions with implementation swarm
