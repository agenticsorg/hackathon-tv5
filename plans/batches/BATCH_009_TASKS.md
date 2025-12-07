# BATCH_009: Media Gateway Action List

**Generated**: 2025-12-06
**Analysis Method**: 9-Agent Claude-Flow Swarm Analysis
**Previous Batches**: BATCH_001 through BATCH_008 Completed (96 tasks total)
**Focus**: Compilation Fixes, SQLx Preparation, Infrastructure Scaffolding, MCP Server Bootstrap

---

## Executive Summary

Following comprehensive analysis of the repository after BATCH_001-008 implementation, this batch focuses on:
1. **Critical Compilation Fixes** - Fix blocking errors in API, SONA, and Sync crates
2. **SQLx Offline Mode** - Generate prepared queries for compile-time verification
3. **Infrastructure Scaffolding** - Create Kubernetes manifests and Terraform modules
4. **MCP Server Bootstrap** - Initialize Model Context Protocol server crate
5. **Audit Logger Completion** - Complete unfinished query implementation in Core

---

## Completed Tasks Inventory (BATCH_001-008)

**Auth Crate (100%)**: OAuth, PKCE, Device Auth, Token Families, MFA, Backup Codes, API Keys, Rate Limiting, Session Management, RBAC, Scopes, User Registration, Email Verification, Password Reset, Profile Management, Admin APIs, Parental Controls, Rate Limit Config API

**Discovery Crate (90%)**: Hybrid Search, Vector Search, Keyword Search, Intent Parsing, Autocomplete, Faceted Search, Spell Correction, Redis Caching, Search Analytics, Personalization, Catalog CRUD, Ranking Config API, Embedding Service

**SONA Crate (85%)**: LoRA Adapters, Collaborative Filtering (ALS), Content-Based, Graph Recommendations, A/B Testing, Diversity Filter, Cold Start, Context-Aware, ONNX Inference

**Ingestion Crate (95%)**: Platform Normalizers (8), Entity Resolution, Embedding Generation, Qdrant Indexing, Webhooks, Pipeline, Repository, Quality Scoring, Freshness Decay, Expiration Notifications

**Sync Crate (85%)**: CRDT (HLC, LWW, OR-Set), PubNub Integration, Offline Queue, Device Management, Watchlist Sync, Progress Sync, WebSocket Broadcasting

**Playback Crate (83%)**: Session Management, Continue Watching, Progress Tracking, Resume Position, Kafka Events

**Core Crate (95%)**: Database Pool, Config, Observability, Metrics, Health Checks, Retry, Pagination, Graceful Shutdown, Circuit Breaker, OpenTelemetry, Audit Logging (partial), User Activity Events, Prometheus Endpoint

**Infrastructure (72%)**: Docker Compose (all services + Kafka), Dockerfiles (7), Health Dashboard, Migration CLI, GitHub Workflows

**Testing (75%)**: Integration Test Framework, Per-crate tests, E2E Auth Flow Tests

---

## Critical Compilation Errors Summary

| Crate | Error Count | Root Cause |
|-------|-------------|------------|
| API | 5 | HeaderMap type mismatch (actix-http vs http) at playback.rs |
| SONA | 15+ | SQLx query macros require DATABASE_URL at graph.rs |
| Sync | 3+ | Missing rusqlite dependency, SQLx macros at repository.rs |
| Core | 0 (9 warnings) | Audit logger query_logs() incomplete |

---

## Task List

### TASK-001: Fix API Crate HeaderMap Type Mismatch
**Priority**: P0-Critical
**Complexity**: Low
**Estimated LOC**: 30-50
**Crate**: `api`

**Description**:
The API crate has 5 compilation errors at `crates/api/src/routes/playback.rs` lines 25, 49, 73, 98, 122. The error is a mismatched types between `actix_http::header::HeaderMap` and `http::header::HeaderMap`. The playback proxy needs to convert headers when forwarding to the playback service.

**Acceptance Criteria**:
- [ ] Convert `actix_http::HeaderMap` to `reqwest::header::HeaderMap` properly
- [ ] Use `.iter()` and manual header conversion instead of direct assignment
- [ ] Verify all 5 proxy methods compile without errors
- [ ] Add unit test for header conversion utility
- [ ] Ensure API crate compiles with `cargo check --package media-gateway-api`

**Files to Modify**:
- `crates/api/src/routes/playback.rs`

**Dependencies**: None

---

### TASK-002: Generate SQLx Prepared Queries for Offline Mode
**Priority**: P0-Critical
**Complexity**: Medium
**Estimated LOC**: 0 (configuration)
**Crates**: `sona`, `sync`, `auth`, `core`

**Description**:
Multiple crates use `sqlx::query!` macros which require DATABASE_URL at compile time. For CI/CD and offline development, we need to run `cargo sqlx prepare` to generate `.sqlx/` cache files. This affects SONA (graph.rs), Sync (repository.rs), and potentially Auth/Core crates.

**Acceptance Criteria**:
- [ ] Create development database with all migrations applied
- [ ] Run `cargo sqlx prepare --workspace` to generate query cache
- [ ] Commit `.sqlx/` directory to repository
- [ ] Update CI/CD workflow to use offline mode
- [ ] Verify `cargo check` succeeds without DATABASE_URL
- [ ] Add documentation for SQLx workflow in CONTRIBUTING.md

**Commands to Execute**:
```bash
export DATABASE_URL="postgres://user:pass@localhost:5432/media_gateway"
cargo sqlx database create
cargo sqlx migrate run
cargo sqlx prepare --workspace
```

**Files to Create**:
- `.sqlx/*.json` (generated query cache files)
- Update `.github/workflows/ci-cd.yaml`

**Dependencies**: TASK-001 (fix compile errors first)

---

### TASK-003: Add Rusqlite Dependency to Sync Crate
**Priority**: P0-Critical
**Complexity**: Low
**Estimated LOC**: 20-40
**Crate**: `sync`

**Description**:
The Sync crate's `sync/queue.rs` imports `rusqlite` for offline sync queue persistence, but the dependency is missing from `Cargo.toml`. Error at line 9: `unresolved import rusqlite`. Also missing `SyncMessage` type at line 531.

**Acceptance Criteria**:
- [ ] Add `rusqlite = { version = "0.31", features = ["bundled"] }` to sync Cargo.toml
- [ ] Verify `SyncMessage` type is properly imported from `crate::pubnub`
- [ ] Fix any remaining compile errors in queue.rs
- [ ] Ensure `cargo check --package media-gateway-sync` succeeds
- [ ] Add unit test for offline queue persistence

**Files to Modify**:
- `crates/sync/Cargo.toml`
- `crates/sync/src/sync/queue.rs`

**Dependencies**: None

---

### TASK-004: Complete Core Audit Logger Query Implementation
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 100-150
**Crate**: `core`

**Description**:
The audit logger's `query_logs()` method at `crates/core/src/audit/logger.rs:150-190` has incomplete implementation. The filter parameters (start_date, end_date, action, limit, offset) are declared but not used in the SQL query. Currently generates 6 unused variable warnings.

**Acceptance Criteria**:
- [ ] Build dynamic SQL query with proper WHERE clauses
- [ ] Apply start_date and end_date filters using SQL BETWEEN
- [ ] Apply action filter using SQL LIKE or exact match
- [ ] Implement pagination with LIMIT and OFFSET
- [ ] Use sqlx `query_as` with dynamic binding
- [ ] Remove all unused variable warnings
- [ ] Add integration test for filtered audit log queries

**Files to Modify**:
- `crates/core/src/audit/logger.rs`

**Dependencies**: TASK-002 (SQLx prepare)

---

### TASK-005: Create Kubernetes Manifest Scaffolding
**Priority**: P1-High
**Complexity**: High
**Estimated LOC**: 500-700
**Files**: `k8s/`

**Description**:
SPARC Completion Master specifies GKE Autopilot deployment but no Kubernetes manifests exist. Create base manifests for all 7 microservices following the architecture specification.

**Acceptance Criteria**:
- [ ] Create `k8s/` directory structure:
  - `k8s/base/` - Base Kustomize resources
  - `k8s/overlays/dev/` - Development environment
  - `k8s/overlays/staging/` - Staging environment
  - `k8s/overlays/prod/` - Production environment
- [ ] Create Deployment + Service for each microservice:
  - API Gateway (8080)
  - Auth Service (8084)
  - Discovery Service (8081)
  - SONA Engine (8082)
  - Sync Service (8083)
  - Ingestion Service (8085)
  - Playback Service (8086)
- [ ] Create ConfigMaps for environment configuration
- [ ] Create Secrets templates for sensitive data
- [ ] Create Horizontal Pod Autoscalers (HPA)
- [ ] Create PodDisruptionBudgets (PDB)
- [ ] Create NetworkPolicies for service mesh
- [ ] Validate manifests with `kubectl --dry-run`

**Files to Create**:
- `k8s/base/kustomization.yaml`
- `k8s/base/namespace.yaml`
- `k8s/base/api-gateway/deployment.yaml`
- `k8s/base/api-gateway/service.yaml`
- `k8s/base/auth-service/...`
- `k8s/base/discovery-service/...`
- `k8s/base/sona-service/...`
- `k8s/base/sync-service/...`
- `k8s/base/ingestion-service/...`
- `k8s/base/playback-service/...`
- `k8s/overlays/*/kustomization.yaml`

**Dependencies**: None

---

### TASK-006: Create Terraform GCP Infrastructure Module
**Priority**: P1-High
**Complexity**: High
**Estimated LOC**: 600-800
**Files**: `terraform/`

**Description**:
SPARC Completion Master specifies GCP infrastructure (GKE Autopilot, Cloud SQL, Memorystore, Secret Manager) but no Terraform exists. Create modular Terraform configuration.

**Acceptance Criteria**:
- [ ] Create `terraform/` directory structure:
  - `terraform/modules/` - Reusable modules
  - `terraform/environments/dev/`
  - `terraform/environments/staging/`
  - `terraform/environments/prod/`
- [ ] Create VPC module with:
  - Custom VPC network
  - Private Google Access enabled
  - Cloud NAT for egress
- [ ] Create GKE Autopilot module with:
  - Autopilot cluster
  - Workload Identity
  - Private cluster configuration
- [ ] Create Cloud SQL module with:
  - PostgreSQL 15 instance
  - Private IP connectivity
  - Automated backups
- [ ] Create Memorystore module with:
  - Redis 7 instance
  - Private service access
- [ ] Create Secret Manager configuration
- [ ] Create Cloud Armor security policy
- [ ] Validate with `terraform validate`

**Files to Create**:
- `terraform/modules/vpc/main.tf`
- `terraform/modules/gke/main.tf`
- `terraform/modules/cloudsql/main.tf`
- `terraform/modules/memorystore/main.tf`
- `terraform/environments/*/main.tf`
- `terraform/environments/*/variables.tf`
- `terraform/environments/*/outputs.tf`

**Dependencies**: None

---

### TASK-007: Bootstrap MCP Server Crate
**Priority**: P1-High
**Complexity**: High
**Estimated LOC**: 400-500
**Crate**: `mcp-server` (new)

**Description**:
SPARC Architecture specifies MCP Server for AI-assisted content discovery (port 3000) but no implementation exists. Create the crate scaffold with Model Context Protocol foundation.

**Acceptance Criteria**:
- [ ] Create `crates/mcp-server/` workspace member
- [ ] Add to workspace Cargo.toml
- [ ] Implement MCP protocol types:
  - `Tool` definitions for search, recommend, sync
  - `Resource` definitions for content, user preferences
  - `Prompt` definitions for discovery flows
- [ ] Create JSON-RPC server using axum
- [ ] Implement tool handlers:
  - `search_content` - Calls Discovery service
  - `get_recommendations` - Calls SONA service
  - `sync_watchlist` - Calls Sync service
- [ ] Add health check endpoint
- [ ] Create Dockerfile for MCP server
- [ ] Integration test with mock MCP client

**Files to Create**:
- `crates/mcp-server/Cargo.toml`
- `crates/mcp-server/src/lib.rs`
- `crates/mcp-server/src/main.rs`
- `crates/mcp-server/src/protocol.rs`
- `crates/mcp-server/src/tools.rs`
- `crates/mcp-server/src/resources.rs`
- `crates/mcp-server/src/handlers.rs`
- `docker/mcp-server.Dockerfile`

**Dependencies**: TASK-001, TASK-002, TASK-003 (crates must compile)

---

### TASK-008: Fix CI/CD Pipeline Configuration
**Priority**: P1-High
**Complexity**: Medium
**Estimated LOC**: 150-200
**Files**: `.github/workflows/`

**Description**:
The main CI/CD pipeline (`ci-cd.yaml`) may reference non-existent paths or have configuration issues based on analysis. Need to update for current crate structure and add SQLx offline mode support.

**Acceptance Criteria**:
- [ ] Update `ci-cd.yaml` for current workspace structure
- [ ] Add SQLx offline mode configuration (use .sqlx cache)
- [ ] Add matrix build for all crates
- [ ] Add Docker build jobs for all 7 services + MCP
- [ ] Add Kubernetes manifest validation step
- [ ] Add Terraform validation step (format, validate)
- [ ] Add integration test job with test containers
- [ ] Ensure pipeline passes with all fixes from BATCH_009

**Files to Modify**:
- `.github/workflows/ci-cd.yaml`

**Dependencies**: TASK-001, TASK-002, TASK-003, TASK-005, TASK-006, TASK-007

---

### TASK-009: Add Playback Deep Linking Support
**Priority**: P2-Medium
**Complexity**: Medium
**Estimated LOC**: 150-200
**Crate**: `playback`

**Description**:
Playback crate is 83% complete but missing deep linking for content playback URLs. Need to generate platform-specific deep links (netflix://, spotify://, etc.) for seamless playback handoff.

**Acceptance Criteria**:
- [ ] Create `deep_link.rs` module in playback crate
- [ ] Support deep link generation for:
  - Netflix (netflix://title/{id})
  - Spotify (spotify://track/{id}, spotify://album/{id})
  - Apple Music (music://itunes.apple.com/...)
  - Hulu (hulu://watch/{id})
  - Disney+ (disneyplus://content/{id})
  - HBO Max (hbomax://content/{id})
  - Prime Video (primevideo://detail?id={id})
- [ ] Add web fallback URLs for unsupported apps
- [ ] Add device capability detection for deep link support
- [ ] Integration test for all platform deep links

**Files to Create/Modify**:
- `crates/playback/src/deep_link.rs` (new)
- `crates/playback/src/lib.rs` (add export)

**Dependencies**: None

---

### TASK-010: Create Development Environment Setup Script
**Priority**: P2-Medium
**Complexity**: Low
**Estimated LOC**: 150-200
**Files**: `scripts/`

**Description**:
Create comprehensive development environment setup script that initializes database, runs migrations, seeds test data, and starts all services for local development.

**Acceptance Criteria**:
- [ ] Create `scripts/dev-setup.sh` that:
  - Checks prerequisites (Docker, Rust, cargo-sqlx)
  - Starts Docker Compose services
  - Waits for service health
  - Runs database migrations
  - Seeds test data (users, content, preferences)
  - Generates SQLx prepared queries
- [ ] Create `scripts/dev-teardown.sh` for cleanup
- [ ] Create `scripts/seed-data.sql` with test fixtures
- [ ] Add Makefile with common targets
- [ ] Update README with development setup instructions

**Files to Create**:
- `scripts/dev-setup.sh`
- `scripts/dev-teardown.sh`
- `scripts/seed-data.sql`
- `Makefile`

**Dependencies**: TASK-002

---

### TASK-011: Implement SONA ExperimentRepository
**Priority**: P2-Medium
**Complexity**: Medium
**Estimated LOC**: 200-250
**Crate**: `sona`

**Description**:
The A/B testing module references `ExperimentRepository` for persisting experiments but the repository is not implemented. Currently experiments are stored in-memory only.

**Acceptance Criteria**:
- [ ] Create `experiment_repository.rs` module
- [ ] Implement `ExperimentRepository` trait with:
  - `create_experiment()` - Persist new experiment
  - `get_experiment()` - Retrieve by ID
  - `list_experiments()` - List with pagination
  - `update_experiment()` - Update status/config
  - `delete_experiment()` - Soft delete
  - `record_assignment()` - Track user assignments
  - `record_metric()` - Record conversion metrics
- [ ] Implement PostgreSQL storage backend
- [ ] Add database migration for experiments table
- [ ] Integration test with real database

**Files to Create/Modify**:
- `crates/sona/src/experiment_repository.rs` (new)
- `crates/sona/src/ab_testing.rs` (integrate repository)
- `crates/sona/src/lib.rs` (add export)
- `migrations/015_create_experiments.sql` (new)

**Dependencies**: TASK-002

---

### TASK-012: Add Prometheus Service Discovery for Grafana
**Priority**: P2-Medium
**Complexity**: Low
**Estimated LOC**: 100-150
**Files**: `docker-compose.yml`, `prometheus/`, `grafana/`

**Description**:
Prometheus metrics endpoint was added in BATCH_008 but full monitoring stack integration is incomplete. Need Prometheus service discovery config and Grafana datasource setup.

**Acceptance Criteria**:
- [ ] Add Prometheus service to docker-compose.yml
- [ ] Create `prometheus/prometheus.yml` with:
  - Scrape configs for all services
  - Service discovery for Docker
  - Relabeling rules
- [ ] Add Grafana service to docker-compose.yml
- [ ] Create `grafana/provisioning/datasources/prometheus.yml`
- [ ] Import dashboard from `grafana/dashboards/api-gateway.json`
- [ ] Verify metrics flow from services to Grafana
- [ ] Document monitoring setup in docs/

**Files to Create/Modify**:
- `docker-compose.yml`
- `prometheus/prometheus.yml`
- `grafana/provisioning/datasources/prometheus.yml`
- `grafana/provisioning/dashboards/dashboard.yml`

**Dependencies**: None

---

## Implementation Order

The recommended implementation sequence based on dependencies and criticality:

1. **TASK-001**: Fix API HeaderMap (unblocks compilation)
2. **TASK-003**: Add rusqlite dependency (unblocks Sync compilation)
3. **TASK-002**: Generate SQLx prepared queries (unblocks offline CI)
4. **TASK-004**: Complete Audit Logger (cleans up warnings)
5. **TASK-005**: Kubernetes Manifests (infrastructure foundation)
6. **TASK-006**: Terraform Modules (infrastructure foundation)
7. **TASK-007**: MCP Server Bootstrap (new capability)
8. **TASK-008**: Fix CI/CD Pipeline (enables continuous integration)
9. **TASK-009**: Playback Deep Linking (feature completion)
10. **TASK-010**: Dev Setup Script (developer experience)
11. **TASK-011**: SONA ExperimentRepository (feature completion)
12. **TASK-012**: Prometheus/Grafana Setup (observability)

---

## Verification Checklist

For each completed task, verify:

- [ ] All acceptance criteria met
- [ ] `cargo check --workspace` succeeds without errors
- [ ] `cargo test --workspace` passes
- [ ] No new compilation warnings introduced
- [ ] Documentation updated where applicable
- [ ] SPARC methodology followed (TDD where appropriate)
- [ ] Code review completed
- [ ] Integration tests pass

---

## Dependency Graph

```
TASK-001 (API Fix) ───┐
TASK-003 (Rusqlite) ──┼──→ TASK-002 (SQLx Prepare) ──→ TASK-004 (Audit Logger)
                      │                              ├──→ TASK-011 (ExperimentRepo)
                      │                              └──→ TASK-010 (Dev Setup)
                      │
                      └──→ TASK-007 (MCP Server)

TASK-005 (K8s) ──────┐
TASK-006 (Terraform) ┼──→ TASK-008 (CI/CD Fix)
TASK-007 (MCP) ──────┘

TASK-009 (Deep Links) ─── (independent)
TASK-012 (Prometheus) ─── (independent)
```

---

## Notes

- **No duplication**: All tasks are new work not covered in BATCH_001-008
- **SPARC aligned**: Each task follows Specification → Implementation → Verification
- **Priority justified**: P0 tasks fix broken compilation, P1 tasks enable deployment, P2 tasks complete features
- **Incremental**: Tasks can be parallelized by different teams/agents
- **Total Tasks**: 12
- **Critical (P0)**: 3 (TASK-001, TASK-002, TASK-003)
- **High (P1)**: 5 (TASK-004, TASK-005, TASK-006, TASK-007, TASK-008)
- **Medium (P2)**: 4 (TASK-009, TASK-010, TASK-011, TASK-012)

---

*Generated by BATCH_009 Analysis Swarm*
