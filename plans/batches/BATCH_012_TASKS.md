# BATCH_012_TASKS.md

**Generated**: 2025-12-07
**Status**: COMPLETE
**Prerequisites**: BATCH_011 complete (all library crates compile with zero errors)
**Completed**: 2025-12-07
**Focus**: Test Infrastructure, Integration Validation, Performance Framework, Security Hardening

---

## Overview

BATCH_012 focuses on production readiness through comprehensive testing infrastructure, integration validation, and security hardening. All tasks are derived from SPARC Phase 5 (Completion) requirements and address gaps identified in the post-BATCH_011 analysis.

**Priority Legend**:
- P0: Critical/Blocking
- P1: High Priority
- P2: Medium Priority

---

## TASK-001: Fix Test Compilation Errors in Auth Crate

**Priority**: P0-BLOCKING
**Estimated Effort**: 2 hours
**Dependencies**: None

### Description
Fix test compilation failures in the auth crate that prevent the test suite from running. Three specific issues need resolution: async/await mismatch in email manager initialization, private method access in PKCE tests, and missing `token_family_id` field in Claims test struct.

### Files to Modify
- `/workspaces/media-gateway/crates/auth/src/handlers.rs:347` - Fix async email manager initialization
- `/workspaces/media-gateway/crates/auth/src/oauth/pkce.rs` - Make `create_s256_challenge` method `pub(crate)`
- `/workspaces/media-gateway/crates/auth/src/middleware/auth.rs:195` - Add `token_family_id: None` to test Claims

### Acceptance Criteria
- [ ] `cargo test -p media-gateway-auth --lib` compiles without errors
- [ ] All existing auth tests pass (minimum 95% pass rate)
- [ ] No new warnings introduced

---

## TASK-002: Fix Test Compilation Errors in Sync Crate

**Priority**: P0-BLOCKING
**Estimated Effort**: 2 hours
**Dependencies**: None

### Description
Fix test compilation failures in the sync crate related to Actix actor trait conflicts and type annotation issues. The integration tests reference outdated WebSocket patterns that need updating.

### Files to Modify
- `/workspaces/media-gateway/crates/sync/src/websocket.rs` - Fix Actor trait conflict
- `/workspaces/media-gateway/crates/sync/tests/integration_websocket_broadcaster_test.rs` - Update test patterns
- `/workspaces/media-gateway/crates/sync/src/ws/registry.rs` - Fix Handler trait implementation

### Acceptance Criteria
- [ ] `cargo test -p media-gateway-sync --lib` compiles without errors
- [ ] WebSocket broadcaster tests execute successfully
- [ ] No regression in existing sync functionality

---

## TASK-003: Fix Test Compilation Errors in Playback Crate

**Priority**: P0-BLOCKING
**Estimated Effort**: 1.5 hours
**Dependencies**: None

### Description
Fix playback crate test compilation errors including missing SQLx query cache for integration tests and Deserialize trait issues in continue_watching tests.

### Files to Modify
- `/workspaces/media-gateway/crates/playback/tests/continue_watching_integration_test.rs` - Fix query cache usage
- `/workspaces/media-gateway/crates/playback/src/continue_watching.rs:59` - Move `MockContentMetadataProvider` to test module

### Acceptance Criteria
- [ ] `cargo test -p media-gateway-playback --lib` compiles without errors
- [ ] Integration tests can run with `SQLX_OFFLINE=true`
- [ ] Mock provider moved out of production code

---

## TASK-004: Fix Rust 2024 Compatibility Warnings

**Priority**: P1-HIGH
**Estimated Effort**: 2 hours
**Dependencies**: None

### Description
Add explicit type annotations to Redis operations across multiple crates to fix "never type fallback" warnings that will become errors in Rust 2024 edition.

### Files to Modify
- `/workspaces/media-gateway/crates/playback/src/session.rs:173,277,393` - Add type annotations to Redis commands
- `/workspaces/media-gateway/crates/auth/src/storage.rs` - Fix Redis operation types
- `/workspaces/media-gateway/crates/auth/src/email/service.rs` - Fix Redis operation types
- `/workspaces/media-gateway/crates/auth/src/session.rs` - Fix Redis operation types

### Code Pattern
```rust
// Before (deprecated)
conn.set_ex(&key, value, TTL)?;

// After (Rust 2024 compatible)
conn.set_ex::<_, _, ()>(&key, value, TTL)?;
```

### Acceptance Criteria
- [ ] Zero "never type fallback" warnings across workspace
- [ ] All Redis operations have explicit type annotations
- [ ] `cargo check --workspace` shows no future incompatibility warnings for never type

---

## TASK-005: Migrate Qdrant Client to Non-Deprecated API

**Priority**: P1-HIGH
**Estimated Effort**: 4 hours
**Dependencies**: None

### Description
Update all qdrant-client usage from deprecated `QdrantClient` API to new `Qdrant` client API. This affects SONA collaborative filtering, content-based recommendations, and ingestion indexing.

### Files to Modify
- `/workspaces/media-gateway/crates/sona/src/collaborative.rs` - Update client initialization and method calls
- `/workspaces/media-gateway/crates/sona/src/content_based.rs` - Update search operations
- `/workspaces/media-gateway/crates/ingestion/src/qdrant.rs` - Update upsert and search methods

### Migration Pattern
```rust
// Before (deprecated)
let client = QdrantClient::from_url(url).build()?;
client.search_points(&SearchPoints { ... }).await?;

// After (recommended)
let client = Qdrant::from_url(url).build()?;
client.search_points(SearchPoints { ... }).await?;
```

### Acceptance Criteria
- [ ] Zero deprecation warnings for qdrant-client
- [ ] All vector search operations functional
- [ ] Integration tests pass with Qdrant 1.16+

---

## TASK-006: Implement E2E Integration Test Framework

**Priority**: P1-HIGH
**Estimated Effort**: 8 hours
**Dependencies**: TASK-001, TASK-002, TASK-003

### Description
Create a comprehensive end-to-end integration test framework using testcontainers for PostgreSQL, Redis, and Qdrant. Implement service-to-service contract tests as defined in SPARC Completion Phase.

### Files to Create
- `/workspaces/media-gateway/tests/src/e2e/mod.rs` - E2E test module
- `/workspaces/media-gateway/tests/src/e2e/auth_flow.rs` - Complete auth flow tests
- `/workspaces/media-gateway/tests/src/e2e/search_pipeline.rs` - Search → SONA integration
- `/workspaces/media-gateway/tests/src/e2e/sync_realtime.rs` - WebSocket sync tests
- `/workspaces/media-gateway/tests/src/containers.rs` - Testcontainers setup

### Files to Modify
- `/workspaces/media-gateway/tests/Cargo.toml` - Add testcontainers dependency
- `/workspaces/media-gateway/tests/src/lib.rs` - Export e2e module

### Test Coverage Targets
- Authentication: OAuth PKCE flow, token refresh, session management
- Search: Natural language → vector search → personalization
- Sync: CRDT operations, conflict resolution, cross-device sync
- Playback: Progress tracking, continue watching, deep links

### Acceptance Criteria
- [ ] E2E test framework compiles and runs
- [ ] Minimum 20 integration tests implemented
- [ ] Tests use real databases via testcontainers
- [ ] All tests pass in CI environment

---

## TASK-007: Create Performance Testing Framework with k6

**Priority**: P1-HIGH
**Estimated Effort**: 6 hours
**Dependencies**: None

### Description
Implement the missing performance testing framework using k6. Create load test scripts for baseline, stress, spike, and soak testing as defined in SPARC Completion Phase.

### Files to Create
- `/workspaces/media-gateway/tests/performance/k6/baseline.js` - 10K users, 1000 RPS, 30 min
- `/workspaces/media-gateway/tests/performance/k6/stress.js` - 20K users, 3500 RPS peak
- `/workspaces/media-gateway/tests/performance/k6/spike.js` - 100K users sudden load
- `/workspaces/media-gateway/tests/performance/k6/soak.js` - 24-hour sustained load
- `/workspaces/media-gateway/tests/performance/k6/config.js` - Shared configuration
- `/workspaces/media-gateway/tests/performance/README.md` - Usage documentation

### Performance Targets (from SPARC)
- Search API: <500ms p95 latency
- SONA personalization: <5ms p95 latency
- Cross-device sync: <100ms latency
- Auth endpoints: <50ms p95 latency

### Acceptance Criteria
- [ ] k6 scripts execute successfully
- [ ] Baseline test completes without errors
- [ ] Results exported to InfluxDB/Grafana format
- [ ] CI integration for performance regression detection

---

## TASK-008: Implement Production Kafka Event Producer

**Priority**: P1-HIGH
**Estimated Effort**: 6 hours
**Dependencies**: None

### Description
Replace the MockEventProducer in the ingestion crate with a production rdkafka implementation. Add delivery confirmation, error recovery, and metrics.

### Files to Modify
- `/workspaces/media-gateway/crates/ingestion/src/events.rs` - Implement real Kafka producer
- `/workspaces/media-gateway/crates/ingestion/Cargo.toml` - Add rdkafka dependency if missing

### Files to Create
- `/workspaces/media-gateway/crates/ingestion/src/events/kafka_producer.rs` - Production implementation
- `/workspaces/media-gateway/crates/ingestion/src/events/metrics.rs` - Producer metrics

### Implementation Requirements
- Use `rdkafka::FutureProducer` for async operations
- Implement delivery confirmation callback
- Add retry logic with exponential backoff
- Export Prometheus metrics for delivery success/failure

### Acceptance Criteria
- [ ] Real Kafka producer replaces mock implementation
- [ ] Events delivered to Kafka cluster successfully
- [ ] Metrics exported for monitoring
- [ ] Integration test validates end-to-end event flow

---

## TASK-009: Add Alertmanager Configuration

**Priority**: P1-HIGH
**Estimated Effort**: 3 hours
**Dependencies**: None

### Description
Create Alertmanager configuration to route alerts defined in Prometheus. Configure notification channels (Slack, PagerDuty placeholders) and alert grouping.

### Files to Create
- `/workspaces/media-gateway/config/alertmanager/alertmanager.yml` - Main configuration
- `/workspaces/media-gateway/config/alertmanager/templates/` - Notification templates

### Files to Modify
- `/workspaces/media-gateway/docker-compose.yml` - Add Alertmanager service
- `/workspaces/media-gateway/config/prometheus.yml` - Point to Alertmanager

### Alert Routing Rules
- P0 Critical: PagerDuty + Slack #incidents
- P1 High: Slack #alerts
- P2 Medium: Slack #monitoring (business hours)
- P3 Low: Email digest

### Acceptance Criteria
- [ ] Alertmanager starts and connects to Prometheus
- [ ] Test alert fires and routes correctly
- [ ] Notification templates render properly
- [ ] Silence and inhibition rules configured

---

## TASK-010: Consolidate Database Migrations

**Priority**: P2-MEDIUM
**Estimated Effort**: 4 hours
**Dependencies**: None

### Description
Consolidate fragmented migration directories into a single source of truth. Establish clear numbering, add down migrations for rollback capability.

### Current State
- `/workspaces/media-gateway/migrations/` - 13 files (004-019, gaps)
- `/workspaces/media-gateway/infrastructure/db/postgres/migrations/` - 6 files (001-003)
- `/workspaces/media-gateway/crates/discovery/migrations/` - 1 file

### Target State
- Single `/workspaces/media-gateway/migrations/` directory
- Sequential numbering (001-025+)
- Every up migration has corresponding down migration
- Clear service ownership comments

### Files to Modify
- Consolidate all migrations to `/workspaces/media-gateway/migrations/`
- `/workspaces/media-gateway/scripts/run-migrations.sh` - Update paths
- `/workspaces/media-gateway/.github/workflows/ci-cd.yaml` - Update migration paths

### Acceptance Criteria
- [ ] Single migrations directory with all migrations
- [ ] Sequential numbering without gaps
- [ ] Down migrations exist for all schema changes
- [ ] Migration tests pass in CI

---

## TASK-011: Implement JWT Secret Security Hardening

**Priority**: P1-HIGH
**Estimated Effort**: 2 hours
**Dependencies**: None

### Description
Remove hardcoded JWT secret fallback in API gateway middleware. Implement fail-fast behavior when JWT_SECRET environment variable is not set in production.

### Files to Modify
- `/workspaces/media-gateway/crates/api/src/middleware/auth.rs:135` - Remove default secret

### Current Code (INSECURE)
```rust
let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
```

### Target Code (SECURE)
```rust
let secret = std::env::var("JWT_SECRET")
    .expect("JWT_SECRET must be set - cannot start with default secret");
```

### Additional Changes
- Add startup validation in `main.rs`
- Log warning in development mode
- Document required environment variables

### Acceptance Criteria
- [ ] Service fails to start without JWT_SECRET in production
- [ ] Clear error message indicates missing configuration
- [ ] Development mode allows override with explicit flag
- [ ] Documentation updated with security requirements

---

## TASK-012: Clean Up Unused Code and Warnings

**Priority**: P2-MEDIUM
**Estimated Effort**: 3 hours
**Dependencies**: None

### Description
Remove unused imports, dead code, and fix remaining compiler warnings across the workspace. Delete legacy files like `user_old.rs`.

### Files to Delete
- `/workspaces/media-gateway/crates/auth/src/user_old.rs` (6,685 lines of dead code)

### Warnings to Fix (by category)
- **Unused imports** (65+ instances): Run `cargo fix --allow-dirty --workspace`
- **Dead code** (14 instances): Remove or implement
- **Naming conventions** (4 instances): Fix or add `#[allow]` attributes

### Commands
```bash
# Auto-fix unused imports
cargo fix --allow-dirty --workspace

# Check remaining warnings
cargo clippy --workspace -- -D warnings
```

### Acceptance Criteria
- [ ] `cargo clippy --workspace` produces zero warnings
- [ ] Dead code files removed
- [ ] All remaining `#[allow]` attributes documented with justification

---

## Verification Checklist

After completing all BATCH_012 tasks:

```bash
# 1. Full workspace compilation
SQLX_OFFLINE=true cargo check --workspace

# 2. Test suite execution
SQLX_OFFLINE=true cargo test --workspace

# 3. Clippy warnings
cargo clippy --workspace -- -D warnings

# 4. Format check
cargo fmt --all -- --check

# 5. E2E tests (requires Docker)
docker-compose up -d postgres redis qdrant
cargo test --package tests --test e2e

# 6. Performance baseline
cd tests/performance && k6 run k6/baseline.js
```

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Test compilation errors | 0 | `cargo test --workspace --no-run` |
| Clippy warnings | 0 | `cargo clippy --workspace` |
| Integration tests | 20+ new tests | Test count |
| E2E test pass rate | 100% | CI results |
| Performance baseline | Established | k6 output |
| Security hardening | JWT secure | Code review |

---

## Next Batch Preview (BATCH_013)

Based on SPARC Completion Phase requirements:
- Load testing execution and optimization
- Security penetration testing preparation
- API documentation (OpenAPI 3.1)
- Service mesh configuration (Istio/Linkerd)
- Disaster recovery testing
- Production deployment runbooks

---

**Document Version**: 1.0
**Generated By**: 9-Agent Claude-Flow Swarm Analysis
**Analysis Sources**: SPARC Master Documents, Batch 001-011 Task Files, Crate Source Code, Infrastructure Configuration
