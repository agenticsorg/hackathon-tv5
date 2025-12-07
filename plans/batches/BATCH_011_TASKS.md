# BATCH_011 Action List - Compilation Resolution & SPARC Phase 5 Enablement

**Generated**: 2025-12-07
**Methodology**: SPARC Phase 4 (Refinement) → Phase 5 (Completion) Transition
**Analysis Source**: 9-agent Claude-Flow swarm analysis of repository state post-BATCH_010
**Priority**: P0/P1 tasks blocking workspace compilation and production readiness

---

## Executive Summary

After comprehensive analysis of all 9 crates, infrastructure state, and SPARC completion requirements, this batch focuses on:
1. **Resolving 87 remaining compilation errors** blocking `cargo check --workspace`
2. **Converting final SQLx macros** to runtime queries (78 macros in 3 files)
3. **Fixing critical type and trait issues** in playback, ingestion, and sona crates
4. **Completing infrastructure gaps** for production deployment
5. **Enabling SPARC Phase 5** integration validation

**Total Tasks**: 12
**Estimated Effort**: 35-45 hours
**Dependencies**: BATCH_010 complete

---

## TASK-001: Convert Sona Graph SQLx Macros to Runtime Queries

**Priority**: P0 - BLOCKING
**Crate**: sona
**Effort**: 3 hours

### Problem
22 SQLx macro errors in `crates/sona/src/graph.rs` preventing compilation. All `sqlx::query!` and `sqlx::query_as!` macros fail without database connection or cached queries.

### File
`crates/sona/src/graph.rs` - lines 81-367 (8 queries)

### Implementation
Convert all compile-time SQLx macros to runtime queries using `sqlx::query()` and `sqlx::query_as()`:

```rust
// Current (broken):
let rows = sqlx::query!(
    r#"SELECT DISTINCT content_id FROM watch_progress WHERE user_id = $1"#,
    user_id, limit as i64
)

// Fix:
let rows = sqlx::query(
    r#"SELECT DISTINCT content_id FROM watch_progress WHERE user_id = $1 LIMIT $2"#
)
.bind(user_id)
.bind(limit as i64)
.fetch_all(&self.pool)
.await?;
```

### Affected Queries (8 total)
1. `get_user_watched_content` (line 81)
2. `find_similar_by_genre` (line 148)
3. `find_similar_by_cast` (line 182)
4. `find_similar_by_director` (line 219)
5. `find_similar_by_themes` (line 245)
6. `get_collaborative_recommendations` (line 315)
7. `get_trending_content` (line 355)
8. `get_personalized_feed` (line 400+)

### Acceptance Criteria
- [ ] All 8 queries converted to runtime `sqlx::query()` or `sqlx::query_as()`
- [ ] Row mapping handled via `.try_get()` or manual struct construction
- [ ] `cargo check --package media-gateway-sona` succeeds
- [ ] Graph recommendation functions return correct types

---

## TASK-002: Convert Playback SQLx Macros to Runtime Queries

**Priority**: P0 - BLOCKING
**Crate**: playback
**Effort**: 2 hours

### Problem
8 SQLx macro errors plus `FromRow` trait bound failures for `ProgressRecord` struct.

### Files
- `crates/playback/src/cleanup.rs`
- `crates/playback/src/continue_watching.rs`
- `crates/playback/src/progress.rs`

### Implementation

#### 2.1 Add FromRow Derive to ProgressRecord
```rust
// Add to ProgressRecord struct definition:
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct ProgressRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub position_seconds: i64,
    pub duration_seconds: i64,
    pub completed: bool,
    pub updated_at: DateTime<Utc>,
}
```

#### 2.2 Convert Remaining Macros
Convert `sqlx::query_as!` to `sqlx::query_as::<_, ProgressRecord>()` pattern.

### Acceptance Criteria
- [ ] `ProgressRecord` derives `sqlx::FromRow`
- [ ] All query macros converted to runtime queries
- [ ] `cargo check --package media-gateway-playback` succeeds
- [ ] Continue watching functionality compiles

---

## TASK-003: Fix ORT (ONNX Runtime) Import Errors

**Priority**: P0 - BLOCKING
**Crate**: sona
**Effort**: 1 hour

### Problem
Unresolved imports for `ort` crate: `Session`, `Value`, `GraphOptimizationLevel`, `SessionBuilder`.

### File
`crates/sona/src/embeddings.rs` (or neural inference module)

### Error
```
error[E0432]: unresolved imports `ort::Session`, `ort::Value`, `ort::GraphOptimizationLevel`, `ort::SessionBuilder`
```

### Implementation
```rust
// Option 1: Update imports for ort v2.x API
use ort::{
    session::{Session, SessionBuilder},
    tensor::Value,
    GraphOptimizationLevel,
};

// Option 2: If ort not needed, feature-gate the module
#[cfg(feature = "onnx")]
mod neural_embeddings { ... }
```

Also update `Cargo.toml` if version mismatch:
```toml
ort = { version = "2.0", optional = true }
```

### Acceptance Criteria
- [ ] ORT imports resolved or feature-gated
- [ ] No unresolved import errors in sona crate
- [ ] Neural embedding functionality compiles (or is properly gated)

---

## TASK-004: Fix Ingestion Type Mismatches and Borrow Errors

**Priority**: P0 - BLOCKING
**Crate**: ingestion
**Effort**: 2 hours

### Problem
5 compilation errors including moved value borrow and type mismatches.

### Files
- `crates/ingestion/src/webhooks/processor.rs`
- `crates/ingestion/src/repository.rs`
- `crates/ingestion/src/qdrant.rs`

### Errors
1. `borrow of moved value: raw_items` - Double use of moved Vec
2. `mismatched types` (2x) - Type conversion issues
3. `type annotations needed` (2x) - Ambiguous generics

### Implementation

#### 4.1 Fix Borrow After Move
```rust
// Current (broken):
let items = process_items(raw_items); // raw_items moved
log_items(&raw_items); // ERROR: borrowed after move

// Fix:
let items = process_items(raw_items.clone());
log_items(&raw_items);
// Or restructure to avoid clone
```

#### 4.2 Add Type Annotations
```rust
// Current (broken):
let result = conn.get(&key).await?;

// Fix:
let result: Option<String> = conn.get(&key).await?;
```

### Acceptance Criteria
- [ ] All borrow checker errors resolved
- [ ] Type annotations added where needed
- [ ] `cargo check --package media-gateway-ingestion` succeeds

---

## TASK-005: Fix totp-rs Secret::default API

**Priority**: P0 - BLOCKING
**Crate**: auth
**Effort**: 30 minutes

### Problem
BATCH_010 TASK-003 specified fix but error persists: `no variant or associated item named 'default' found for enum 'totp_rs::Secret'`

### File
`crates/auth/src/mfa/totp.rs:26`

### Implementation
```rust
// Current (broken):
let secret = Secret::default();

// Fix for totp-rs v5.7.0:
use rand::Rng;
let bytes: [u8; 20] = rand::thread_rng().gen();
let secret = Secret::Raw(bytes.to_vec());

// Or use generate method if available:
let secret = Secret::generate_secret();
```

Verify correct API by checking `totp-rs` documentation for installed version.

### Acceptance Criteria
- [ ] MFA TOTP secret generation compiles
- [ ] Secret creation produces valid TOTP secrets
- [ ] MFA tests pass

---

## TASK-006: Fix Recommendation Struct Missing Field

**Priority**: P0 - BLOCKING
**Crate**: sona
**Effort**: 30 minutes

### Problem
3 errors: `missing field 'experiment_variant' in initializer of 'Recommendation'`

### File
`crates/sona/src/recommendations.rs` or `crates/sona/src/ab_testing.rs`

### Implementation
Add missing field to all `Recommendation` struct initializations:

```rust
Recommendation {
    content_id: id,
    score: score,
    reason: reason,
    experiment_variant: None, // Add this field
}
```

Or if experiment tracking is required:
```rust
experiment_variant: Some(ExperimentVariant {
    experiment_id: active_experiment.id,
    variant_name: assigned_variant.name.clone(),
}),
```

### Acceptance Criteria
- [ ] All `Recommendation` initializers include `experiment_variant`
- [ ] A/B testing integration compiles
- [ ] Recommendation queries return complete structs

---

## TASK-007: Fix PgRow try_get Method Errors

**Priority**: P0 - BLOCKING
**Crate**: sync, sona
**Effort**: 1.5 hours

### Problem
10 errors: `no method named 'try_get' found for struct 'PgRow'`

### Files
- `crates/sync/src/repository.rs`
- `crates/sona/src/graph.rs`
- `crates/sona/src/recommendations.rs`

### Implementation
Import the `Row` trait to enable `try_get`:

```rust
// Add import:
use sqlx::Row;

// Then try_get works:
let id: Uuid = row.try_get("id")?;
let name: String = row.try_get("name")?;
```

### Acceptance Criteria
- [ ] `sqlx::Row` trait imported in all files using `try_get`
- [ ] All row field extractions compile
- [ ] No `try_get` method not found errors

---

## TASK-008: Fix Auth Crate Miscellaneous Errors

**Priority**: P0 - BLOCKING
**Crate**: auth
**Effort**: 1 hour

### Problem
Multiple remaining errors in auth crate including HttpResponse method access.

### Errors
1. `attempted to take value of method 'status' on type 'HttpResponse<()>'`
2. `attempted to take value of method 'headers' on type 'HttpResponse<()>'`

### Files
- `crates/auth/src/middleware/rate_limit.rs`
- `crates/auth/src/handlers.rs`

### Implementation
```rust
// Current (broken):
response.status  // Missing parentheses

// Fix:
response.status()
response.headers()
```

### Acceptance Criteria
- [ ] All method calls use proper syntax
- [ ] `cargo check --package media-gateway-auth` succeeds

---

## TASK-009: Create Kubernetes Ingress Manifest

**Priority**: P1 - INFRASTRUCTURE
**Crate**: infrastructure
**Effort**: 2 hours

### Problem
No Kubernetes ingress manifest exists for routing external traffic to services.

### File
Create `kubernetes/ingress.yaml`

### Implementation
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: media-gateway-ingress
  namespace: media-gateway
  annotations:
    kubernetes.io/ingress.class: "gce"
    kubernetes.io/ingress.global-static-ip-name: "media-gateway-ip"
    networking.gke.io/managed-certificates: "media-gateway-cert"
    kubernetes.io/ingress.allow-http: "false"
spec:
  rules:
    - host: api.media-gateway.io
      http:
        paths:
          - path: /api/v1/auth/*
            pathType: ImplementationSpecific
            backend:
              service:
                name: auth-service
                port:
                  number: 8084
          - path: /api/v1/search/*
            pathType: ImplementationSpecific
            backend:
              service:
                name: discovery-service
                port:
                  number: 8081
          - path: /api/v1/recommendations/*
            pathType: ImplementationSpecific
            backend:
              service:
                name: sona-service
                port:
                  number: 8082
          - path: /api/v1/sync/*
            pathType: ImplementationSpecific
            backend:
              service:
                name: sync-service
                port:
                  number: 8083
          - path: /*
            pathType: ImplementationSpecific
            backend:
              service:
                name: api-gateway
                port:
                  number: 8080
    - host: mcp.media-gateway.io
      http:
        paths:
          - path: /*
            pathType: ImplementationSpecific
            backend:
              service:
                name: mcp-server
                port:
                  number: 3000
```

### Acceptance Criteria
- [ ] Ingress manifest created with all service routes
- [ ] TLS termination configured via managed certificate
- [ ] Health check paths configured
- [ ] `kubectl apply -f kubernetes/ingress.yaml` validates

---

## TASK-010: Create Terraform tfvars for Staging/Production

**Priority**: P1 - INFRASTRUCTURE
**Crate**: infrastructure
**Effort**: 2 hours

### Problem
Terraform environments exist but lack `.tfvars` files for actual deployment values.

### Files
Create:
- `terraform/environments/staging/terraform.tfvars`
- `terraform/environments/prod/terraform.tfvars`

### Implementation

#### staging/terraform.tfvars
```hcl
project_id         = "media-gateway-staging"
region             = "us-central1"
environment        = "staging"
gke_node_count     = 3
gke_machine_type   = "e2-standard-4"
cloudsql_tier      = "db-custom-2-4096"
redis_memory_gb    = 4
enable_ha          = false
```

#### prod/terraform.tfvars
```hcl
project_id         = "media-gateway-prod"
region             = "us-central1"
environment        = "production"
gke_node_count     = 6
gke_machine_type   = "e2-standard-8"
cloudsql_tier      = "db-custom-4-8192"
redis_memory_gb    = 8
enable_ha          = true
enable_backups     = true
backup_retention   = 30
```

### Acceptance Criteria
- [ ] Both tfvars files created with environment-appropriate values
- [ ] `terraform plan -var-file=terraform.tfvars` succeeds in each environment
- [ ] Secrets referenced from Secret Manager (not hardcoded)

---

## TASK-011: Create Grafana Service Dashboards

**Priority**: P1 - MONITORING
**Crate**: infrastructure
**Effort**: 4 hours

### Problem
Only api-gateway dashboard exists. Missing dashboards for 7 other services.

### Files
Create in `config/grafana/dashboards/`:
- `auth-service.json`
- `discovery-service.json`
- `sona-engine.json`
- `sync-service.json`
- `playback-service.json`
- `ingestion-service.json`
- `mcp-server.json`

### Implementation
Each dashboard should include:
- Request rate panel
- Error rate panel (4xx, 5xx)
- Latency percentiles (p50, p95, p99)
- Active connections
- Database pool usage
- Redis cache hit rate
- Service-specific metrics

### Template Structure
```json
{
  "dashboard": {
    "title": "Service Name Dashboard",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [{"expr": "rate(http_requests_total{service=\"$service\"}[5m])"}]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [{"expr": "rate(http_requests_total{service=\"$service\",status=~\"5..\"}[5m])"}]
      },
      {
        "title": "Latency P95",
        "type": "graph",
        "targets": [{"expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket{service=\"$service\"}[5m]))"}]
      }
    ]
  }
}
```

### Acceptance Criteria
- [ ] 7 new dashboard JSON files created
- [ ] Each dashboard imports successfully into Grafana
- [ ] Metrics queries match service metric names
- [ ] Variables allow filtering by pod/instance

---

## TASK-012: Fix ndarray Least Squares Method Error

**Priority**: P1 - FEATURE BLOCKING
**Crate**: sona
**Effort**: 1 hour

### Problem
2 errors: `no method named 'least_squares' found for struct 'ArrayBase<S, D>'`

### File
`crates/sona/src/analysis.rs` or statistical module

### Implementation
The `least_squares` method requires `ndarray-linalg` crate with LAPACK backend:

```toml
# Cargo.toml
[dependencies]
ndarray = "0.15"
ndarray-linalg = { version = "0.16", features = ["openblas-system"] }
```

Or use alternative implementation:
```rust
// Using linregress crate instead
use linregress::{FormulaRegressionBuilder, RegressionDataBuilder};

let formula = "y ~ x1 + x2";
let data = RegressionDataBuilder::new().build_from(...)?;
let model = FormulaRegressionBuilder::new().data(&data).formula(formula).fit()?;
```

### Acceptance Criteria
- [ ] Linear regression functionality compiles
- [ ] Dependencies properly configured
- [ ] Statistical analysis features work

---

## Dependencies Graph

```
TASK-001 (Sona SQLx) ──┬──> TASK-006 (Recommendation field)
                       └──> TASK-007 (try_get import)

TASK-002 (Playback SQLx) ──> Independent

TASK-003 (ORT imports) ────> Independent

TASK-004 (Ingestion types) ─> Independent

TASK-005 (TOTP API) ───────> Independent

TASK-007 (Row trait) ──────> TASK-001, TASK-002

TASK-008 (Auth misc) ──────> Independent

TASK-009 (K8s Ingress) ────> Independent

TASK-010 (Terraform) ──────> Independent

TASK-011 (Grafana) ────────> Independent

TASK-012 (ndarray) ────────> TASK-001
```

---

## Verification Checklist

After completing all tasks:

```bash
# 1. Full compilation check (MUST PASS)
SQLX_OFFLINE=true cargo check --workspace

# 2. Run all tests
cargo test --workspace

# 3. Clippy lint check
cargo clippy --workspace -- -D warnings

# 4. Format check
cargo fmt --all -- --check

# 5. Kubernetes manifest validation
kubectl apply --dry-run=client -f kubernetes/

# 6. Terraform validation
cd terraform/environments/staging && terraform validate
cd terraform/environments/prod && terraform validate

# 7. Grafana dashboard validation
for f in config/grafana/dashboards/*.json; do jq . "$f" > /dev/null; done
```

---

## Completion Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Compilation Errors | 0 | 87 |
| SQLx Macros Remaining | 0 | 78 |
| Grafana Dashboards | 8 | 1 |
| Terraform Environments | 3 complete | 1 complete |
| K8s Ingress | 1 | 0 |

---

## Notes

- **SQLx Strategy**: Continue converting macros to runtime queries for offline compilation
- **ORT Dependency**: Consider feature-gating ONNX if not critical path
- **Infrastructure Priority**: Ingress and tfvars block production deployment
- **SPARC Phase 5**: Requires zero compilation errors before integration validation

---

**Next Batch**: BATCH_012 should focus on:
1. E2E integration tests across all services
2. Load testing framework implementation
3. Security penetration testing preparation
4. API documentation (OpenAPI/Swagger)
5. Deployment runbooks and rollback procedures
6. Service mesh configuration (Istio/Linkerd)
