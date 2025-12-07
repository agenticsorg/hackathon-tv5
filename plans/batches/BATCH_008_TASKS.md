# BATCH_008: Media Gateway Action List

**Generated**: 2025-12-06
**Analysis Method**: 9-Agent Claude-Flow Swarm Analysis
**Previous Batches**: BATCH_001 through BATCH_007 Completed (82 tasks total)
**Focus**: Production Hardening, Kafka Integration, Advanced Features, E2E Testing

---

## Executive Summary

Following comprehensive analysis of the repository after BATCH_001-007 implementation, this batch focuses on:
1. **Production Hardening** - Missing Dockerfiles, Kafka deployment, service hardening
2. **Webhook Completion** - Finish webhook integration system with full pipeline integration
3. **Advanced Search** - Semantic search improvements, search result ranking
4. **E2E Testing** - Complete integration test framework with real service tests
5. **Operational Tooling** - CLI tools, migration scripts, monitoring dashboards

---

## Completed Tasks Inventory (BATCH_001-007)

**Auth Crate**: OAuth (Google, GitHub, Apple), PKCE, Device Authorization, Token Families, MFA (TOTP), Backup Codes, API Keys, Rate Limiting, Session Management, RBAC, Scopes, User Registration, Email Verification, Password Reset, Profile Management, Admin APIs, Parental Controls

**Discovery Crate**: Hybrid Search, Vector Search, Keyword Search, Intent Parsing, Autocomplete, Faceted Search, Spell Correction, Redis Caching, Search Analytics, Personalization, Catalog CRUD API

**SONA Crate**: LoRA Adapters, Collaborative Filtering (ALS), Content-Based Filtering, Graph Recommendations, A/B Testing, Diversity Filter, Cold Start, Context-Aware Filtering, ONNX Inference

**Ingestion Crate**: Platform Normalizers (8 platforms), Entity Resolution, Embedding Generation, Qdrant Indexing, Webhooks (partial), Pipeline, Repository, Quality Scoring

**Sync Crate**: CRDT (HLC, LWW, OR-Set), PubNub Integration, Offline Queue, PostgreSQL Persistence, Device Management, Watchlist Sync, Progress Sync, WebSocket Broadcasting

**Playback Crate**: Session Management, Continue Watching, Progress Tracking, Resume Position, Kafka Events (partial)

**Core Crate**: Database Pool, Config Loader, Observability, Metrics, Health Checks, Retry Utility, Pagination, Graceful Shutdown, Circuit Breaker, OpenTelemetry Tracing, Audit Logging

**Infrastructure**: Docker Compose (all services), Jaeger, Dockerfiles (api, discovery, sona, auth, sync, ingestion, playback)

**Testing**: Integration Test Framework (tests/ workspace member), per-crate integration tests

---

## Task List

### TASK-001: Add Kafka Service to Docker Compose
**Priority**: P0-Critical
**Complexity**: Medium
**Estimated LOC**: 80-100
**Files**: `docker-compose.yml`, `docker-compose.kafka.yml`

**Description**:
Kafka is referenced in multiple crates (ingestion, playback, discovery) for event streaming but is not deployed in docker-compose.yml. The `rdkafka` dependency is already in workspace but services can't connect.

**Acceptance Criteria**:
- [ ] Add Zookeeper service (required by Kafka)
- [ ] Add Kafka broker service with health check
- [ ] Configure `KAFKA_BOOTSTRAP_SERVERS` env var for all services
- [ ] Create topics: `content-ingested`, `content-updated`, `playback-events`, `user-activity`
- [ ] Add `scripts/kafka-setup.sh` for topic creation
- [ ] Verify services connect on startup
- [ ] Document Kafka configuration in README

**Dependencies**: None

---

### TASK-002: Complete Webhook Pipeline Integration
**Priority**: P0-Critical
**Complexity**: Medium
**Estimated LOC**: 200-250
**Crate**: `ingestion`

**Description**:
Webhook handlers exist but have TODO comments indicating incomplete pipeline integration. The Netflix handler notes "TODO: Integrate with ingestion pipeline" and queue metrics are not tracked.

**Acceptance Criteria**:
- [ ] Connect `NetflixWebhookHandler` to `IngestionPipeline::ingest_content()`
- [ ] Implement configurable platform list in webhook queue (currently hardcoded)
- [ ] Track `processing_count` and `total_processed` metrics
- [ ] Implement `WebhookHandler::process()` for generic handler
- [ ] Add error recovery for failed webhook processing
- [ ] Emit Kafka events on successful webhook processing
- [ ] Integration test with mock webhook payloads

**Files to Modify**:
- `crates/ingestion/src/webhooks/handlers/netflix.rs`
- `crates/ingestion/src/webhooks/queue.rs`
- `crates/ingestion/src/webhooks/api.rs`

**Dependencies**: TASK-001 (Kafka)

---

### TASK-003: Implement Password Reset Email Sending
**Priority**: P0-Critical
**Complexity**: Low
**Estimated LOC**: 50-80
**Crate**: `auth`

**Description**:
Password reset flow exists but email sending has TODO comments. `password_reset_handlers.rs` line 43 has "TODO: Send password reset email" and line 90 needs password changed notification.

**Acceptance Criteria**:
- [ ] Wire `EmailService::send_password_reset_email()` in forgot password handler
- [ ] Wire `EmailService::send_password_changed_notification()` after reset
- [ ] Use existing email templates from `email/templates.rs`
- [ ] Add rate limiting: 3 reset emails per hour per email address
- [ ] Integration test with console email provider

**Files to Modify**:
- `crates/auth/src/password_reset_handlers.rs`

**Dependencies**: None (email service exists)

---

### TASK-004: Implement Real Embedding Service for Discovery
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 150-200
**Crate**: `discovery`

**Description**:
Vector search has TODO at line 57: "Generate query embedding (TODO: implement embedding service)". Current implementation uses stub. Need to connect to existing OpenAI embedding or local model.

**Acceptance Criteria**:
- [ ] Create `EmbeddingClient` that calls OpenAI API or local model
- [ ] Support `text-embedding-3-small` (768 dims) and `text-embedding-3-large` (1536 dims)
- [ ] Add caching layer for frequent queries (Redis, 24h TTL)
- [ ] Fallback to keyword search on embedding failure
- [ ] Batch embedding support for multiple queries
- [ ] Configuration via `EMBEDDING_PROVIDER` env var (openai, local)
- [ ] Unit tests with mocked API responses

**Files to Create/Modify**:
- `crates/discovery/src/embedding.rs` (expand)
- `crates/discovery/src/search/vector.rs` (modify)

**Dependencies**: None

---

### TASK-005: Implement Content Freshness Score Decay
**Priority**: P1-High
**Complexity**: Low
**Estimated LOC**: 80-100
**Crate**: `ingestion`

**Description**:
Quality scoring exists but freshness decay is not implemented. Old content should have decaying quality scores to prioritize recent updates in search ranking.

**Acceptance Criteria**:
- [ ] Add `last_updated_at` timestamp tracking
- [ ] Implement decay function: `score * exp(-decay_rate * days_since_update)`
- [ ] Configurable decay rate (default: 0.01 per day)
- [ ] Maximum decay cap (minimum 50% of original score)
- [ ] Background job to recalculate scores weekly
- [ ] Integrate with search ranking boost

**Files to Create/Modify**:
- `crates/ingestion/src/quality/scorer.rs`
- `crates/ingestion/src/quality/mod.rs`

**Dependencies**: None

---

### TASK-006: Create Migration CLI Tool
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 250-300
**Files**: `tools/mg-migrate/`

**Description**:
No CLI tool exists for running database migrations. Migrations in `migrations/` folder need to be applied manually. Create a migration CLI tool for operations teams.

**Acceptance Criteria**:
- [ ] Create `tools/mg-migrate` Rust binary
- [ ] Commands: `up`, `down`, `status`, `create <name>`
- [ ] Support `--dry-run` flag to preview changes
- [ ] Connect to database via `DATABASE_URL` env var
- [ ] Version tracking in `schema_migrations` table
- [ ] Integration with CI/CD (exit codes)
- [ ] Color output for status
- [ ] Add to workspace Cargo.toml

**Files to Create**:
- `tools/mg-migrate/Cargo.toml`
- `tools/mg-migrate/src/main.rs`
- `tools/mg-migrate/src/commands.rs`

**Dependencies**: None

---

### TASK-007: Implement E2E Auth Flow Test
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 200-250
**Crate**: `tests`

**Description**:
Integration test framework exists but auth crate dependency is disabled ("Temporarily disabled due to compilation errors"). E2E auth flow test needs complete user journey.

**Acceptance Criteria**:
- [ ] Fix auth crate compilation issues (Argon2 API changes)
- [ ] Enable `media-gateway-auth` dependency in tests/Cargo.toml
- [ ] Implement full E2E test: register → verify email → login → refresh → logout
- [ ] Test MFA enrollment and verification flow
- [ ] Test password reset flow
- [ ] Test OAuth login simulation (mocked providers)
- [ ] Test admin user management endpoints
- [ ] CI integration with test database

**Files to Modify**:
- `tests/Cargo.toml`
- `tests/src/auth_tests.rs`
- `crates/auth/src/user/password.rs` (fix Argon2 API)

**Dependencies**: None

---

### TASK-008: Add Prometheus Metrics Endpoint
**Priority**: P1-High
**Complexity**: Low
**Estimated LOC**: 100-150
**Crate**: `api`

**Description**:
Prometheus dependency exists and metrics are collected but no `/metrics` endpoint exposes them. Metrics are internal only. Add scrape endpoint for monitoring.

**Acceptance Criteria**:
- [ ] Add `GET /metrics` endpoint exposing Prometheus format
- [ ] Include all registered metrics (HTTP, cache, DB, connections)
- [ ] Basic auth protection for metrics endpoint (optional)
- [ ] Configure Prometheus scrape in docker-compose
- [ ] Add Grafana dashboard JSON template
- [ ] Document metric names and labels

**Files to Create/Modify**:
- `crates/api/src/routes.rs`
- `docker-compose.yml` (add prometheus service optional)
- `grafana/dashboards/api-gateway.json`

**Dependencies**: None

---

### TASK-009: Implement User Activity Event Stream
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 200-250
**Crate**: `playback`, `discovery`, `auth`

**Description**:
User activity (searches, views, ratings) should stream to Kafka for analytics and recommendations. Currently only playback events are partially implemented.

**Acceptance Criteria**:
- [ ] Define `UserActivityEvent` schema (user_id, event_type, content_id, timestamp, metadata)
- [ ] Emit events from discovery search (query, results_count, clicked_items)
- [ ] Emit events from playback (start, pause, complete, abandon)
- [ ] Emit events from auth (login, logout, profile_update)
- [ ] Create `user-activity` Kafka topic
- [ ] Consumer example for SONA recommendations
- [ ] Event deduplication by event_id

**Files to Create/Modify**:
- `crates/core/src/events/user_activity.rs` (new)
- `crates/discovery/src/search/mod.rs`
- `crates/playback/src/events.rs`
- `crates/auth/src/handlers.rs`

**Dependencies**: TASK-001 (Kafka)

---

### TASK-010: Implement Search Result Ranking Tuning API
**Priority**: P2-Medium
**Complexity**: Medium
**Estimated LOC**: 200-250
**Crate**: `discovery`

**Description**:
Search ranking uses fixed weights. Admins need ability to tune ranking factors (vector similarity, keyword match, quality score, freshness) without code changes.

**Acceptance Criteria**:
- [ ] Create `RankingConfig` struct with adjustable weights
- [ ] `GET /api/v1/admin/search/ranking` - Get current config
- [ ] `PUT /api/v1/admin/search/ranking` - Update weights
- [ ] Store config in Redis with versioning
- [ ] A/B testing support (multiple ranking configs)
- [ ] Validate weights sum to 1.0
- [ ] Audit log ranking config changes
- [ ] Admin-only authentication

**Files to Create/Modify**:
- `crates/discovery/src/search/ranking.rs` (new)
- `crates/discovery/src/server/handlers/` (admin routes)

**Dependencies**: None

---

### TASK-011: Add Service Health Dashboard
**Priority**: P2-Medium
**Complexity**: Medium
**Estimated LOC**: 150-200
**Files**: `apps/health-dashboard/`

**Description**:
Health aggregator endpoint exists but no visual dashboard. Create simple HTML/JS dashboard that polls health endpoints and displays status.

**Acceptance Criteria**:
- [ ] Create static health dashboard (HTML + vanilla JS)
- [ ] Poll `/health/aggregate` every 10 seconds
- [ ] Display service status with color indicators (green/yellow/red)
- [ ] Show response times for each service
- [ ] Show dependency health (PostgreSQL, Redis, Qdrant, Kafka)
- [ ] Last check timestamp
- [ ] Serve from API gateway at `/dashboard/health`

**Files to Create**:
- `apps/health-dashboard/index.html`
- `apps/health-dashboard/styles.css`
- `apps/health-dashboard/app.js`
- `crates/api/src/routes.rs` (static file serving)

**Dependencies**: None

---

### TASK-012: Implement Content Expiration Notifications
**Priority**: P2-Medium
**Complexity**: Medium
**Estimated LOC**: 150-200
**Crate**: `ingestion`

**Description**:
Content has `availability_end` dates but no notification system. Platform admins and users should be notified before content expires.

**Acceptance Criteria**:
- [ ] Create `ExpirationNotificationJob` scheduled task
- [ ] Query content expiring in next 7 days, 3 days, 1 day
- [ ] Emit Kafka event `content-expiring` with content details
- [ ] API endpoint to get expiring content list
- [ ] Optional email notification to subscribed users (future)
- [ ] Configurable notification windows
- [ ] Track notification sent status to avoid duplicates

**Files to Create/Modify**:
- `crates/ingestion/src/notifications/expiration.rs` (new)
- `crates/ingestion/src/lib.rs`

**Dependencies**: TASK-001 (Kafka)

---

### TASK-013: Create API Rate Limiting Configuration UI
**Priority**: P2-Medium
**Complexity**: Medium
**Estimated LOC**: 200-250
**Crate**: `auth`

**Description**:
Rate limiting exists but limits are hardcoded. Admins need ability to configure rate limits per endpoint, per user tier, without code changes.

**Acceptance Criteria**:
- [ ] Create `RateLimitConfig` struct (endpoint, tier, requests_per_minute, burst)
- [ ] `GET /api/v1/admin/rate-limits` - List all configs
- [ ] `PUT /api/v1/admin/rate-limits/{endpoint}` - Update config
- [ ] Store in Redis with hot reload
- [ ] User tiers: anonymous, free, premium, enterprise
- [ ] Default fallback config
- [ ] Audit log config changes
- [ ] Admin-only authentication

**Files to Create/Modify**:
- `crates/auth/src/rate_limit_config.rs` (new)
- `crates/auth/src/admin/` (add routes)

**Dependencies**: None

---

### TASK-014: Implement Session Invalidation on Password Change
**Priority**: P2-Medium
**Complexity**: Low
**Estimated LOC**: 50-80
**Crate**: `auth`

**Description**:
Password reset flow exists but doesn't invalidate existing sessions. Security best practice requires all sessions to be terminated when password changes.

**Acceptance Criteria**:
- [ ] After password reset, invalidate all user sessions
- [ ] Invalidate all refresh tokens for user
- [ ] Keep current session optionally (with `keep_current: bool` param)
- [ ] Emit `sessions-invalidated` event
- [ ] Return count of invalidated sessions in response
- [ ] Integration test for session invalidation

**Files to Modify**:
- `crates/auth/src/password_reset.rs`
- `crates/auth/src/password_reset_handlers.rs`
- `crates/auth/src/session.rs`

**Dependencies**: None

---

---

## Implementation Order

The recommended implementation sequence based on dependencies and priority:

1. **TASK-001**: Kafka Docker (blocks event streaming tasks)
2. **TASK-003**: Password Reset Emails (quick fix)
3. **TASK-007**: E2E Auth Tests (enables test coverage)
4. **TASK-002**: Webhook Pipeline (completes ingestion)
5. **TASK-004**: Embedding Service (search quality)
6. **TASK-009**: Activity Events (analytics foundation)
7. **TASK-008**: Prometheus Endpoint (monitoring)
8. **TASK-006**: Migration CLI (operational tooling)
9. **TASK-005**: Freshness Decay (search ranking)
10. **TASK-014**: Session Invalidation (security)
11. **TASK-010**: Ranking Tuning API (search optimization)
12. **TASK-012**: Expiration Notifications (content lifecycle)
13. **TASK-013**: Rate Limit Config (operational flexibility)
14. **TASK-011**: Health Dashboard (monitoring UX)

---

## Verification Checklist

For each completed task, verify:

- [ ] All acceptance criteria met
- [ ] Unit tests with >80% coverage
- [ ] Integration tests where applicable
- [ ] No compilation warnings
- [ ] Documentation updated
- [ ] SPARC Refinement patterns followed (TDD)
- [ ] Security review for auth-related tasks
- [ ] Database migrations tested
- [ ] Docker builds succeed

---

## Dependency Graph

```
TASK-001 (Kafka)
    ├── TASK-002 (Webhook Pipeline)
    ├── TASK-009 (Activity Events)
    └── TASK-012 (Expiration Notifications)

TASK-007 (E2E Tests)
    └── requires fixing Argon2 API in auth crate
```

---

## Notes

- **No duplication**: All tasks are new work not covered in BATCH_001-007
- **SPARC aligned**: Each task follows Specification → Pseudocode → Architecture → Refinement → Completion
- **Priority justified**: P0 tasks complete broken functionality, P1 tasks enable production features, P2 tasks enhance operations
- **Incremental**: Tasks can be parallelized by different teams/agents
- **Total Tasks**: 14
- **Critical (P0)**: 3 (TASK-001, TASK-002, TASK-003)
- **High (P1)**: 6 (TASK-004, TASK-005, TASK-006, TASK-007, TASK-008, TASK-009)
- **Medium (P2)**: 5 (TASK-010, TASK-011, TASK-012, TASK-013, TASK-014)

---

*Generated by BATCH_008 Analysis Swarm*
