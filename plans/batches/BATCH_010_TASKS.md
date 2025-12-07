# BATCH_010 Action List - Compilation Fixes & Production Hardening

**Generated**: 2025-12-06
**Methodology**: SPARC Phase 4 (Refinement) + Phase 5 (Completion)
**Analysis Source**: 9-agent Claude-Flow swarm analysis of repository state post-BATCH_009
**Priority**: P0/P1 tasks blocking compilation and production deployment

---

## Executive Summary

After comprehensive analysis of all 9 crates, infrastructure, and SPARC compliance, this batch focuses on:
1. **Fixing 45+ compilation errors** blocking cargo build
2. **Completing MCP server** SPARC requirements
3. **Wiring critical integrations** (Kafka, service routes)
4. **Production infrastructure** (CI/CD, monitoring, alerts)

**Total Tasks**: 12
**Estimated Effort**: 40-50 hours
**Dependencies**: None (BATCH_009 complete)

---

## TASK-001: Fix SQLx Offline Mode for All Crates

**Priority**: P0 - BLOCKING
**Crate**: workspace
**Effort**: 2 hours

### Problem
45+ compilation errors across auth, sync, playback, ingestion, discovery crates due to missing SQLx offline query cache. All `sqlx::query!` and `sqlx::query_as!` macros fail without `DATABASE_URL` or prepared queries.

### Affected Files
- `crates/auth/src/mfa/mod.rs` (4 errors)
- `crates/auth/src/storage.rs` (multiple errors)
- `crates/auth/src/admin/handlers.rs` (multiple errors)
- `crates/sync/src/repository.rs` (12 errors)
- `crates/playback/src/progress.rs` (7 errors)
- `crates/ingestion/src/*.rs` (5 errors)
- `crates/discovery/src/search/mod.rs` (errors)

### Implementation
1. Start PostgreSQL with docker-compose
2. Run migrations: `cargo sqlx migrate run`
3. Generate query cache: `cargo sqlx prepare --workspace`
4. Commit `.sqlx/*.json` files to repository
5. Verify CI/CD uses `SQLX_OFFLINE=true`

### Acceptance Criteria
- [ ] All SQLx query errors resolved
- [ ] `.sqlx/` directory contains JSON files for all crates
- [ ] `SQLX_OFFLINE=true cargo check --workspace` succeeds

---

## TASK-002: Fix Type Mismatches Across Crates

**Priority**: P0 - BLOCKING
**Crate**: sync, ingestion, auth
**Effort**: 2 hours

### Problem
Multiple type mismatch errors preventing compilation.

### Errors to Fix

#### 2.1 HLCTimestamp Type Conversion (sync/repository.rs:186)
```rust
// Current (broken):
HLCTimestamp::from_components(physical, logical) // physical: u64, logical: u32

// Fix:
HLCTimestamp::from_components(
    physical.try_into().expect("physical timestamp overflow"),
    logical.try_into().expect("logical counter overflow")
)
```

#### 2.2 String Type Mismatch (sync/publisher.rs:369)
```rust
// Current (broken):
message.payload.get_content_id().unwrap_or("unknown")

// Fix:
message.payload.get_content_id().unwrap_or_else(|| "unknown".to_string())
```

#### 2.3 CommandType Enum Variant (sync/command_router.rs:81)
```rust
// Current: CommandType::Cast { target_device_id }
// Defined: CommandType::CastTo { target_device_id, content_id }

// Fix: Update command_router.rs to use CastTo variant
```

#### 2.4 Redis TTL Type (ingestion/webhooks/deduplication.rs)
```rust
// Current: set_ex(..., ttl) where ttl: usize
// Fix: set_ex(..., ttl as u64)
```

#### 2.5 Qdrant ListValue Conversion (ingestion/qdrant.rs)
```rust
// Fix Qdrant Value::List construction - use proper conversion methods
```

#### 2.6 f64/f32 Mismatches (ingestion/repository.rs)
```rust
// Cast quality_score fields appropriately
quality_score as f32 // or use f64 consistently
```

### Acceptance Criteria
- [ ] All type mismatch errors resolved
- [ ] `cargo check --package media-gateway-sync` succeeds
- [ ] `cargo check --package media-gateway-ingestion` succeeds

---

## TASK-003: Fix totp-rs API Breaking Change

**Priority**: P0 - BLOCKING
**Crate**: auth
**Effort**: 30 minutes

### Problem
totp-rs v5.7.0 changed API. `Secret::generate_secret()` no longer exists.

### File
`crates/auth/src/mfa/totp.rs:26`

### Implementation
```rust
// Current (broken):
let secret = Secret::generate_secret();

// Fix - Option 1 (recommended):
let secret = Secret::default(); // Generates random secret

// Fix - Option 2:
use rand::Rng;
let bytes: [u8; 20] = rand::thread_rng().gen();
let secret = Secret::Raw(bytes.to_vec());
```

Also fix test at line 139.

### Acceptance Criteria
- [ ] MFA TOTP secret generation works
- [ ] All MFA tests pass
- [ ] `cargo test --package media-gateway-auth mfa` succeeds

---

## TASK-004: Fix Rate Limiter ServiceResponse Type

**Priority**: P0 - BLOCKING
**Crate**: auth
**Effort**: 1 hour

### Problem
actix-web middleware type constraint mismatch in rate limiter.

### File
`crates/auth/src/middleware/rate_limit.rs:299`

### Error
```
expected `ServiceResponse<B>`, found `ServiceResponse<BoxBody>`
```

### Implementation
```rust
// Current (broken):
return Ok(req.into_response(response));

// Fix:
use actix_web::body::EitherBody;
let response = response.map_into_boxed_body();
let (req, _) = req.into_parts();
return Ok(ServiceResponse::new(req, response).map_into_right_body());
```

### Acceptance Criteria
- [ ] Rate limiter middleware compiles
- [ ] Rate limiting integration tests pass

---

## TASK-005: Fix AuditLogger get_logs() Signature

**Priority**: P0 - BLOCKING
**Crate**: auth, core
**Effort**: 1 hour

### Problem
`AuditLogger::get_logs()` called with 3 arguments but function expects `AuditLogFilter` struct.

### Files
- `crates/auth/src/admin/handlers.rs:45`
- `crates/core/src/audit/logger.rs`

### Implementation
```rust
// Current (broken):
audit_logger.get_logs(
    query.user_id.as_deref(),
    query.limit,
    query.action.as_deref(),
)

// Fix:
use media_gateway_core::audit::AuditLogFilter;
let filter = AuditLogFilter {
    user_id: query.user_id,
    action: query.action,
    limit: query.limit,
    offset: query.offset,
    start_date: query.start_date,
    end_date: query.end_date,
};
audit_logger.get_logs(filter).await
```

### Acceptance Criteria
- [ ] Admin audit log endpoint works
- [ ] Filter parameters properly passed
- [ ] Integration tests pass

---

## TASK-006: Add Missing HybridSearchService Field

**Priority**: P0 - BLOCKING
**Crate**: discovery
**Effort**: 30 minutes

### Problem
`activity_producer` field used but not declared in struct definition.

### File
`crates/discovery/src/search/mod.rs:33-43`

### Implementation
```rust
pub struct HybridSearchService {
    // ... existing fields ...
    activity_producer: Option<Arc<KafkaActivityProducer>>, // ADD THIS
}
```

### Acceptance Criteria
- [ ] HybridSearchService compiles
- [ ] Activity tracking works when producer configured

---

## TASK-007: Fix Redis Never Type Fallback Warnings

**Priority**: P1 - HIGH (Rust 2024 breaking)
**Crate**: auth, sync
**Effort**: 2 hours

### Problem
26+ warnings about "never type fallback" in Redis operations. Will become hard errors in Rust 2024 edition.

### Files
- `crates/auth/src/storage.rs` (26 occurrences)
- `crates/sync/src/*.rs`

### Implementation
Add explicit type annotations to all Redis async operations:
```rust
// Current:
conn.expire(&key, ttl).await?;

// Fix:
conn.expire::<_, ()>(&key, ttl).await?;

// Or for operations returning values:
let result: Option<String> = conn.get(&key).await?;
```

### Acceptance Criteria
- [ ] Zero "never type fallback" warnings
- [ ] All Redis operations have explicit type annotations

---

## TASK-008: Implement MCP Server list_devices Tool

**Priority**: P1 - SPARC REQUIREMENT
**Crate**: mcp-server
**Effort**: 4 hours

### Problem
SPARC specification requires `list_devices` tool but it's not implemented.

### File
Create `crates/mcp-server/src/tools/list_devices.rs`

### Implementation
```rust
pub async fn list_devices(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<DeviceInfo>, McpError> {
    let devices = sqlx::query_as!(
        DeviceInfo,
        r#"
        SELECT device_id, device_type, platform, capabilities,
               last_seen, is_online
        FROM user_devices
        WHERE user_id = $1
        ORDER BY last_seen DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(devices)
}
```

Register in tools.rs and update protocol handlers.

### Acceptance Criteria
- [ ] `list_devices` tool registered and functional
- [ ] Returns user's devices with online status
- [ ] Integration with sync service database

---

## TASK-009: Add MCP Server STDIO Transport

**Priority**: P1 - SPARC REQUIREMENT
**Crate**: mcp-server
**Effort**: 3 hours

### Problem
STDIO transport required for Claude Desktop integration but not implemented.

### File
Create `crates/mcp-server/src/transport/stdio.rs`

### Implementation
```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub async fn run_stdio_server(handler: Arc<McpHandler>) -> Result<()> {
    let stdin = BufReader::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();
    let mut lines = stdin.lines();

    while let Some(line) = lines.next_line().await? {
        let request: JsonRpcRequest = serde_json::from_str(&line)?;
        let response = handler.handle_request(request).await;
        let output = serde_json::to_string(&response)?;
        stdout.write_all(output.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }
    Ok(())
}
```

### Acceptance Criteria
- [ ] STDIO transport functional
- [ ] Can be used with Claude Desktop
- [ ] Main binary supports `--stdio` flag

---

## TASK-010: Register Missing Discovery HTTP Routes

**Priority**: P1 - FEATURE INCOMPLETE
**Crate**: discovery
**Effort**: 2 hours

### Problem
Handler functions exist but routes not registered in server.

### File
`crates/discovery/src/server/mod.rs:41-49`

### Missing Routes
- `/api/v1/search` - Main search endpoint
- `/api/v1/search/autocomplete` - Autocomplete
- `/api/v1/analytics/*` - Analytics endpoints
- `/api/v1/admin/search/ranking/*` - Ranking admin
- `/api/v1/quality/*` - Quality reports

### Implementation
```rust
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health))
            .route("/search", web::post().to(handlers::search::execute))
            .route("/search/autocomplete", web::get().to(handlers::search::autocomplete))
            .service(
                web::scope("/analytics")
                    .route("/queries", web::get().to(handlers::analytics::get_queries))
                    .route("/popular", web::get().to(handlers::analytics::get_popular))
            )
            .service(
                web::scope("/admin/search/ranking")
                    .route("/config", web::get().to(handlers::ranking::get_config))
                    .route("/config", web::put().to(handlers::ranking::update_config))
            )
    );
    crate::catalog::configure_routes(cfg);
}
```

### Acceptance Criteria
- [ ] All search endpoints accessible
- [ ] Analytics endpoints functional
- [ ] Admin ranking endpoints secured

---

## TASK-011: Create Rust CI/CD Workflow

**Priority**: P1 - PRODUCTION REQUIRED
**Crate**: infrastructure
**Effort**: 3 hours

### Problem
No GitHub Actions workflow for Rust backend CI/CD.

### File
Create `.github/workflows/rust-ci.yml`

### Implementation
```yaml
name: Rust CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  SQLX_OFFLINE: true

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

      - name: Build
        run: cargo build --workspace --release

      - name: Test
        run: cargo test --workspace

  docker:
    needs: check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build Docker images
        run: |
          for service in api auth discovery sona sync ingestion playback mcp-server; do
            docker build -f docker/${service}.Dockerfile -t media-gateway-${service}:${{ github.sha }} .
          done
```

### Acceptance Criteria
- [ ] CI runs on all PRs
- [ ] Formatting, linting, build, test all pass
- [ ] Docker images built for all 8 services

---

## TASK-012: Create Prometheus Alert Rules

**Priority**: P1 - PRODUCTION REQUIRED
**Crate**: infrastructure
**Effort**: 2 hours

### Problem
No alerting configured for production monitoring.

### File
Create `config/prometheus/alerts.yml`

### Implementation
```yaml
groups:
  - name: media-gateway-alerts
    rules:
      - alert: ServiceDown
        expr: up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Service {{ $labels.job }} is down"

      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate on {{ $labels.job }}"

      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 0.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High latency on {{ $labels.job }}"

      - alert: DatabaseConnectionPoolExhausted
        expr: sqlx_pool_connections_idle == 0
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Database connection pool exhausted"

      - alert: RedisConnectionFailed
        expr: redis_up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Redis connection failed"
```

Update `config/prometheus.yml` to include:
```yaml
rule_files:
  - /etc/prometheus/alerts.yml
```

### Acceptance Criteria
- [ ] Alert rules loaded by Prometheus
- [ ] Alerts fire correctly in test scenarios
- [ ] AlertManager integration configured

---

## Dependencies Graph

```
TASK-001 (SQLx) ──┬──> TASK-002 (Types)
                  ├──> TASK-005 (AuditLogger)
                  ├──> TASK-006 (Discovery)
                  └──> TASK-008 (MCP list_devices)

TASK-003 (TOTP) ──────> Independent

TASK-004 (RateLimit) ─> Independent

TASK-007 (Redis) ─────> Independent

TASK-009 (STDIO) ─────> TASK-008 (MCP tools)

TASK-010 (Routes) ────> TASK-006 (Discovery fields)

TASK-011 (CI/CD) ─────> TASK-001 (SQLx offline)

TASK-012 (Alerts) ────> Independent
```

---

## Verification Checklist

After completing all tasks:

```bash
# 1. Full compilation check
SQLX_OFFLINE=true cargo check --workspace

# 2. Run all tests
cargo test --workspace

# 3. Clippy lint check
cargo clippy --workspace -- -D warnings

# 4. Format check
cargo fmt --all -- --check

# 5. Docker build verification
docker-compose build

# 6. Integration test
docker-compose up -d
./scripts/health-check.sh
```

---

## Notes

- **SQLx Offline Mode**: Commit `.sqlx/*.json` files after generation
- **Breaking Changes**: Tasks 1-6 must complete before any tests can run
- **SPARC Compliance**: Tasks 8-9 complete MCP server requirements
- **Production Path**: Tasks 11-12 enable deployment readiness

---

**Next Batch**: BATCH_011 should focus on:
1. Graph search implementation (discovery)
2. Quality score integration (discovery)
3. E2E integration tests
4. Load testing framework
5. Terraform staging/prod tfvars
6. Grafana dashboards for all services
