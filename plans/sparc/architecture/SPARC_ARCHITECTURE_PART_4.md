# SPARC Architecture Phase - Part 4: Deployment, DevOps, and Operations

**Version:** 1.0.0
**Phase:** SPARC Architecture
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents

1. [CI/CD Pipeline Architecture](#cicd-pipeline-architecture)
2. [Environment Strategy](#environment-strategy)
3. [Deployment Strategies](#deployment-strategies)
4. [Infrastructure as Code](#infrastructure-as-code)
5. [Observability Stack](#observability-stack)
6. [Release Management](#release-management)
7. [Disaster Recovery](#disaster-recovery)
8. [Operations Runbooks](#operations-runbooks)

---

## 1. CI/CD Pipeline Architecture

### 1.1 Pipeline Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           CI/CD PIPELINE FLOW                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌────────────┐ │
│  │  Code   │──▶│  Build   │──▶│   Test   │──▶│ Security │──▶│   Deploy   │ │
│  │  Push   │   │  Stage   │   │  Stage   │   │   Scan   │   │   Stage    │ │
│  └─────────┘   └──────────┘   └──────────┘   └──────────┘   └────────────┘ │
│       │             │              │              │               │         │
│       ▼             ▼              ▼              ▼               ▼         │
│   ┌───────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌────────────┐ │
│   │GitHub │   │Container │   │Unit+Int  │   │Trivy+    │   │Staging/    │ │
│   │Actions│   │Build     │   │Tests     │   │Snyk Scan │   │Production  │ │
│   └───────┘   └──────────┘   └──────────┘   └──────────┘   └────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 GitHub Actions Workflow Structure

```yaml
# .github/workflows/ci-cd.yml (Conceptual)
Pipeline Stages:

  1. Trigger:
     - Push to main/develop branches
     - Pull request to main
     - Manual dispatch for hotfixes
     - Scheduled (nightly for security scans)

  2. Build Stage:
     - Checkout code
     - Setup Rust/Node.js toolchains
     - Dependency caching (Cargo, npm)
     - Compile services
     - Build Docker images
     - Push to Artifact Registry

  3. Test Stage:
     - Unit tests (cargo test, jest)
     - Integration tests (testcontainers)
     - E2E tests (Playwright)
     - Coverage report (>80% threshold)

  4. Security Stage:
     - SAST (cargo-audit, npm audit)
     - Container scanning (Trivy)
     - Dependency scanning (Snyk)
     - Secret detection (gitleaks)

  5. Deploy Stage:
     - Deploy to staging (automatic)
     - Smoke tests
     - Manual approval gate
     - Deploy to production (canary)
     - Progressive rollout
```

### 1.3 Build Pipeline Details

| Stage | Tool | Duration | Gate |
|-------|------|----------|------|
| Checkout | GitHub Actions | 10s | - |
| Cache Restore | actions/cache | 30s | - |
| Rust Build | cargo build --release | 5m | - |
| Node Build | npm run build | 2m | - |
| Docker Build | docker buildx | 3m | - |
| Unit Tests | cargo test / jest | 4m | Coverage >80% |
| Integration Tests | testcontainers | 6m | All pass |
| Security Scan | Trivy + Snyk | 2m | No critical CVEs |
| Push Images | Artifact Registry | 1m | - |
| **Total** | | **~25m** | |

### 1.4 Approval Gates

```
┌─────────────────────────────────────────────────────────────────┐
│                     DEPLOYMENT APPROVAL FLOW                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Development ──▶ Staging ──▶ [Manual Approval] ──▶ Production   │
│       │             │                                    │       │
│   Automatic     Automatic                            Canary      │
│    Deploy       Deploy                              10% ──▶ 50%  │
│                   │                                   ──▶ 100%   │
│              Smoke Tests                                         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

Approval Requirements:
  - Staging smoke tests pass
  - No critical security findings
  - At least 1 reviewer approval
  - SRE sign-off for production
```

---

## 2. Environment Strategy

### 2.1 Environment Overview

| Environment | Purpose | Infra | Data | Access |
|-------------|---------|-------|------|--------|
| **Local** | Developer testing | Docker Compose | Mock/seed | Developer |
| **Feature** | PR validation | GKE (ephemeral) | Anonymized subset | Developer |
| **Development** | Integration | GKE (shared) | Synthetic | Team |
| **Staging** | Pre-production | GKE (prod-like) | Anonymized prod | Team + QA |
| **Production** | Live users | GKE (HA) | Real | Ops only |

### 2.2 Environment Configuration

```
┌────────────────────────────────────────────────────────────────┐
│                    ENVIRONMENT TOPOLOGY                         │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌────────────────┐  │
│  │   DEVELOPMENT   │  │     STAGING     │  │   PRODUCTION   │  │
│  ├─────────────────┤  ├─────────────────┤  ├────────────────┤  │
│  │ GKE Autopilot   │  │ GKE Autopilot   │  │ GKE Autopilot  │  │
│  │ 1 zone          │  │ 2 zones         │  │ 3 zones        │  │
│  │ Spot instances  │  │ Spot + Standard │  │ Standard only  │  │
│  │                 │  │                 │  │                │  │
│  │ Cloud SQL       │  │ Cloud SQL       │  │ Cloud SQL HA   │  │
│  │ (db-f1-micro)   │  │ (db-g1-small)   │  │ (db-n2-std-4)  │  │
│  │                 │  │                 │  │                │  │
│  │ Memorystore     │  │ Memorystore     │  │ Memorystore HA │  │
│  │ (1 GB)          │  │ (2 GB)          │  │ (8 GB)         │  │
│  └─────────────────┘  └─────────────────┘  └────────────────┘  │
│                                                                 │
│  Monthly Cost:                                                  │
│  ~$150              ~$400               ~$2,500                 │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

### 2.3 Feature Environments (Ephemeral)

```
Feature Environment Strategy:

  Trigger: Pull request opened/updated

  Provisioning:
    - Namespace: pr-{pr_number}
    - Services: All microservices (minimal replicas)
    - Database: Shared dev database with PR-specific schema
    - Duration: Destroyed on PR close/merge

  Configuration:
    - Ingress: pr-{number}.dev.media-gateway.io
    - Resources: 50% of development limits
    - Scale: 1 replica per service

  Cost Control:
    - Auto-destroy after 24h of inactivity
    - Maximum 5 concurrent feature environments
    - Spot instances only
```

---

## 3. Deployment Strategies

### 3.1 Strategy by Service

| Service | Strategy | Rationale |
|---------|----------|-----------|
| API Gateway | Blue-Green | Zero-downtime critical |
| MCP Server | Rolling | Stateless, quick restart |
| Discovery Service | Canary | Search quality validation |
| SONA Engine | Canary | Recommendation impact |
| Sync Service | Rolling | CRDT ensures consistency |
| Auth Service | Blue-Green | Security critical |
| Ingestion Service | Rolling | Background processing |

### 3.2 Canary Deployment Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    CANARY DEPLOYMENT PHASES                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Phase 1: Canary (5%)                                           │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ [████░░░░░░░░░░░░░░░░] 5% traffic to new version           │ │
│  │  Duration: 15 minutes                                       │ │
│  │  Metrics: Error rate, latency p99, success rate            │ │
│  │  Rollback if: Error rate > 1% OR p99 > 2x baseline         │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              │                                   │
│                              ▼                                   │
│  Phase 2: Expand (25%)                                          │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ [█████████░░░░░░░░░░░] 25% traffic to new version          │ │
│  │  Duration: 30 minutes                                       │ │
│  │  Metrics: Same + business metrics (CTR, conversions)       │ │
│  │  Rollback if: Business metrics degraded > 5%               │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              │                                   │
│                              ▼                                   │
│  Phase 3: Expand (50%)                                          │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ [████████████░░░░░░░░] 50% traffic to new version          │ │
│  │  Duration: 1 hour                                           │ │
│  │  Metrics: Full observability suite                         │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              │                                   │
│                              ▼                                   │
│  Phase 4: Complete (100%)                                       │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ [████████████████████] 100% traffic to new version         │ │
│  │  Old version kept for 1 hour (fast rollback)               │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.3 Rollback Procedures

```
Automatic Rollback Triggers:
  - Error rate exceeds 5% for 5 minutes
  - P99 latency exceeds 3x baseline for 10 minutes
  - Pod crash loop detected (>3 restarts in 5 minutes)
  - Health check failures on >25% of pods

Manual Rollback Command:
  kubectl rollout undo deployment/{service} -n production

Rollback Time Targets:
  - Detection: <2 minutes (automated monitoring)
  - Decision: <3 minutes (PagerDuty alert)
  - Execution: <5 minutes (kubectl rollout)
  - Total MTTR: <10 minutes
```

---

## 4. Infrastructure as Code

### 4.1 Repository Structure

```
infrastructure/
├── terraform/
│   ├── modules/
│   │   ├── gke-cluster/           # GKE Autopilot cluster
│   │   ├── cloud-sql/             # PostgreSQL instances
│   │   ├── memorystore/           # Redis instances
│   │   ├── networking/            # VPC, subnets, firewall
│   │   ├── iam/                   # Service accounts, roles
│   │   └── monitoring/            # Alerting policies
│   ├── environments/
│   │   ├── dev/
│   │   │   ├── main.tf
│   │   │   ├── variables.tf
│   │   │   └── terraform.tfvars
│   │   ├── staging/
│   │   └── production/
│   └── shared/
│       ├── artifact-registry.tf
│       ├── dns.tf
│       └── secrets.tf
│
├── kubernetes/
│   ├── base/                      # Kustomize base
│   │   ├── namespace.yaml
│   │   ├── configmaps/
│   │   ├── secrets/
│   │   └── services/
│   │       ├── api-gateway/
│   │       ├── mcp-server/
│   │       ├── discovery-service/
│   │       ├── sona-engine/
│   │       ├── sync-service/
│   │       ├── auth-service/
│   │       └── ingestion-service/
│   └── overlays/
│       ├── development/
│       ├── staging/
│       └── production/
│
└── helm/
    └── media-gateway/
        ├── Chart.yaml
        ├── values.yaml
        ├── values-staging.yaml
        ├── values-production.yaml
        └── templates/
```

### 4.2 GitOps with ArgoCD

```
┌─────────────────────────────────────────────────────────────────┐
│                      GITOPS WORKFLOW                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────────┐  │
│  │   GitHub    │      │   ArgoCD    │      │      GKE        │  │
│  │  (Config)   │─────▶│ (Reconcile) │─────▶│   (Cluster)     │  │
│  └─────────────┘      └─────────────┘      └─────────────────┘  │
│        │                    │                      │            │
│        │                    │                      │            │
│   Push config         Detect drift           Apply changes      │
│   changes             (3 min sync)           (kubectl apply)    │
│                                                                  │
│  ArgoCD Applications:                                           │
│  ├── media-gateway-infra (Terraform state)                      │
│  ├── media-gateway-dev                                          │
│  ├── media-gateway-staging                                      │
│  └── media-gateway-production                                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

Sync Policies:
  - Development: Auto-sync enabled
  - Staging: Auto-sync enabled
  - Production: Manual sync required (approval gate)
```

### 4.3 Terraform Module Example

```hcl
# Conceptual module structure for GKE cluster

module "gke_cluster" {
  source = "./modules/gke-cluster"

  project_id   = var.project_id
  region       = var.region
  cluster_name = "media-gateway-${var.environment}"

  # Autopilot configuration
  autopilot_enabled = true

  # Networking
  network    = module.networking.vpc_name
  subnetwork = module.networking.subnet_name

  # Node configuration (Autopilot manages this)
  release_channel = "REGULAR"

  # Security
  workload_identity_enabled = true
  binary_authorization      = var.environment == "production"

  # Maintenance
  maintenance_window = {
    start_time = "03:00"
    end_time   = "07:00"
    recurrence = "FREQ=WEEKLY;BYDAY=SU"
  }
}
```

---

## 5. Observability Stack

### 5.1 Three Pillars Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    OBSERVABILITY STACK                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐   │
│  │     METRICS     │ │      LOGS       │ │     TRACES      │   │
│  ├─────────────────┤ ├─────────────────┤ ├─────────────────┤   │
│  │ Cloud Monitoring│ │  Cloud Logging  │ │   Cloud Trace   │   │
│  │ + Prometheus    │ │  (Structured)   │ │ + OpenTelemetry │   │
│  │                 │ │                 │ │                 │   │
│  │ Grafana         │ │ Log Explorer    │ │ Trace Explorer  │   │
│  │ Dashboards      │ │ Log Analytics   │ │ Latency Analysis│   │
│  └────────┬────────┘ └────────┬────────┘ └────────┬────────┘   │
│           │                   │                   │             │
│           └───────────────────┼───────────────────┘             │
│                               │                                  │
│                               ▼                                  │
│                    ┌─────────────────┐                          │
│                    │    ALERTING     │                          │
│                    ├─────────────────┤                          │
│                    │ Cloud Monitoring│                          │
│                    │ Alert Policies  │                          │
│                    │       +         │                          │
│                    │   PagerDuty     │                          │
│                    └─────────────────┘                          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 Key Metrics (SLIs)

| Service | Metric | SLO Target | Alert Threshold |
|---------|--------|------------|-----------------|
| API Gateway | Request latency p99 | <500ms | >750ms for 5m |
| API Gateway | Error rate | <0.1% | >1% for 5m |
| Discovery | Search latency p95 | <400ms | >600ms for 5m |
| SONA | Recommendation latency | <200ms | >400ms for 5m |
| Sync Service | Sync latency p99 | <100ms | >200ms for 5m |
| Auth Service | Token validation | <10ms | >50ms for 5m |
| All Services | Availability | 99.9% | <99.5% for 15m |

### 5.3 Logging Strategy

```
Structured Log Format (JSON):
{
  "timestamp": "2025-12-06T12:00:00.000Z",
  "level": "INFO|WARN|ERROR",
  "service": "discovery-service",
  "version": "1.2.3",
  "trace_id": "abc123...",
  "span_id": "def456...",
  "user_id": "usr_xxx" (hashed),
  "message": "Search completed",
  "context": {
    "query": "action movies",
    "results_count": 25,
    "latency_ms": 142
  }
}

Log Retention:
  - Hot (Cloud Logging): 30 days
  - Warm (Cloud Storage): 90 days
  - Cold (Archive): 1 year
  - Audit logs: 7 years
```

### 5.4 Alerting Policies

```
Alert Severity Levels:

  P1 (Critical) - Page immediately:
    - Service completely down
    - Error rate > 10%
    - Data corruption detected
    - Security breach indicators
    Response: 15 minutes

  P2 (High) - Page during business hours:
    - Degraded performance (>2x baseline)
    - Error rate > 5%
    - Approaching resource limits
    Response: 1 hour

  P3 (Medium) - Ticket:
    - Elevated error rate (>1%)
    - Non-critical service issues
    - Certificate expiring (7 days)
    Response: 24 hours

  P4 (Low) - Backlog:
    - Optimization opportunities
    - Non-urgent maintenance
    Response: 1 week
```

---

## 6. Release Management

### 6.1 Versioning Strategy

```
Semantic Versioning: MAJOR.MINOR.PATCH

  MAJOR: Breaking API changes
  MINOR: New features (backward compatible)
  PATCH: Bug fixes

Version Examples:
  - 1.0.0: Initial production release
  - 1.1.0: Added new recommendation algorithm
  - 1.1.1: Fixed search ranking bug
  - 2.0.0: GraphQL API v2 (breaking changes)

Container Tags:
  - :latest (development only)
  - :v1.2.3 (immutable, production)
  - :sha-abc1234 (commit-based)
```

### 6.2 Release Process

```
┌─────────────────────────────────────────────────────────────────┐
│                     RELEASE WORKFLOW                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. Create Release Branch                                       │
│     └── git checkout -b release/v1.2.0                          │
│                                                                  │
│  2. Version Bump                                                │
│     └── Update version in Cargo.toml, package.json              │
│                                                                  │
│  3. Changelog Generation                                        │
│     └── git-cliff --tag v1.2.0 > CHANGELOG.md                   │
│                                                                  │
│  4. Release PR                                                  │
│     └── PR: release/v1.2.0 → main                               │
│                                                                  │
│  5. Approvals                                                   │
│     ├── Code review (2 approvers)                               │
│     ├── QA sign-off                                             │
│     └── SRE sign-off                                            │
│                                                                  │
│  6. Merge & Tag                                                 │
│     ├── Merge to main                                           │
│     └── git tag v1.2.0                                          │
│                                                                  │
│  7. Automated Deployment                                        │
│     ├── CI/CD builds tagged images                              │
│     ├── Deploy to staging                                       │
│     ├── Smoke tests                                             │
│     └── Production canary                                       │
│                                                                  │
│  8. Release Notes                                               │
│     └── GitHub Release with changelog                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 6.3 Dependency Management

```
Dependency Update Strategy:

  Automated (Renovate):
    - Security patches: Auto-merge after CI passes
    - Minor updates: Weekly batch PR
    - Major updates: Monthly review

  Manual Review Required:
    - Rust async runtime changes
    - Database driver updates
    - Authentication library updates

  Lockfiles:
    - Cargo.lock: Committed
    - package-lock.json: Committed
    - Dependabot alerts: Enabled
```

---

## 7. Disaster Recovery

### 7.1 RTO/RPO Targets

| Component | RTO | RPO | Strategy |
|-----------|-----|-----|----------|
| API Gateway | 5 min | N/A | Multi-zone, auto-failover |
| Core Services | 15 min | N/A | Multi-zone replicas |
| PostgreSQL | 1 hour | 5 min | Point-in-time recovery |
| Redis | 15 min | 5 min | Failover replica |
| Qdrant | 4 hours | 24 hours | Daily snapshots |
| Object Storage | N/A | N/A | Multi-region replication |

### 7.2 Backup Strategy

```
┌─────────────────────────────────────────────────────────────────┐
│                      BACKUP ARCHITECTURE                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  PostgreSQL:                                                    │
│  ├── Continuous WAL archiving (Cloud Storage)                  │
│  ├── Daily automated backups (7-day retention)                 │
│  ├── Weekly backups (4-week retention)                         │
│  └── Monthly backups (12-month retention)                      │
│                                                                  │
│  Redis:                                                         │
│  ├── RDB snapshots every 15 minutes                            │
│  └── AOF persistence (1 second fsync)                          │
│                                                                  │
│  Qdrant:                                                        │
│  ├── Daily collection snapshots                                │
│  └── 7-day retention                                           │
│                                                                  │
│  Configuration:                                                 │
│  ├── Git (infrastructure repo)                                 │
│  └── Secret Manager (automatic versioning)                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 7.3 Disaster Recovery Procedures

```
Scenario 1: Single Zone Failure
  Impact: Temporary latency increase
  Action: Automatic - GKE redistributes pods
  Recovery Time: 2-5 minutes
  Data Loss: None

Scenario 2: Database Corruption
  Impact: Service degradation
  Action: Point-in-time recovery
  Recovery Time: 30-60 minutes
  Data Loss: Up to 5 minutes

Scenario 3: Region Failure
  Impact: Complete outage
  Action: Manual failover to backup region
  Recovery Time: 2-4 hours
  Data Loss: Up to 1 hour (async replication)

Scenario 4: Security Breach
  Impact: Potential data exposure
  Action: Incident response playbook
  Recovery Time: Variable
  Steps:
    1. Isolate affected systems
    2. Revoke compromised credentials
    3. Forensic analysis
    4. Restore from known-good backup
    5. Post-incident review
```

---

## 8. Operations Runbooks

### 8.1 On-Call Responsibilities

```
On-Call Rotation:
  - Primary: Responds to all pages
  - Secondary: Backup for escalation
  - Rotation: Weekly

Escalation Path:
  P1: Primary (5 min) → Secondary (10 min) → Engineering Lead (15 min)
  P2: Primary (15 min) → Secondary (30 min) → Engineering Lead (1 hour)

Tools:
  - PagerDuty: Alert routing
  - Slack: #incident-response channel
  - Runbook: Notion/Confluence
  - Status Page: statuspage.io
```

### 8.2 Common Runbook Procedures

```
Runbook: High Error Rate

Trigger: Error rate > 5% for 5 minutes

Steps:
  1. Check Cloud Monitoring dashboard
     - Identify affected service(s)
     - Check error distribution

  2. Review recent deployments
     - kubectl rollout history deployment/{service}
     - If recent deploy, consider rollback

  3. Check dependent services
     - Database connectivity
     - Redis availability
     - External API status

  4. Review logs
     - Cloud Logging → Error filter
     - Look for stack traces

  5. If database issue:
     - Check connection pool
     - Check slow query log
     - Scale read replicas if needed

  6. If memory issue:
     - Check pod memory usage
     - Restart affected pods
     - Consider HPA adjustment

  7. Document and notify
     - Update status page
     - Post in #incident-response
     - Create incident ticket
```

### 8.3 SLO Dashboard

```
┌─────────────────────────────────────────────────────────────────┐
│                     SLO STATUS DASHBOARD                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Service Availability (30-day rolling):                         │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ API Gateway    [████████████████████░] 99.95% (SLO: 99.9%) │ │
│  │ Discovery      [████████████████████░] 99.92% (SLO: 99.9%) │ │
│  │ SONA Engine    [████████████████████░] 99.89% (SLO: 99.5%) │ │
│  │ Sync Service   [████████████████████░] 99.97% (SLO: 99.9%) │ │
│  │ Auth Service   [████████████████████░] 99.99% (SLO: 99.9%) │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  Error Budget Remaining (30-day):                               │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ API Gateway    [████████████░░░░░░░░] 60% (26 min left)    │ │
│  │ Discovery      [██████████░░░░░░░░░░] 50% (21 min left)    │ │
│  │ SONA Engine    [████████████████░░░░] 80% (3.4 hr left)    │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  Recent Incidents:                                              │
│  • 2025-12-05: Search latency spike (P2, 15 min, resolved)     │
│  • 2025-12-01: Database failover (P1, 8 min, resolved)         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Cost Summary

### Monthly Infrastructure Costs (Production)

| Component | Service | Monthly Cost |
|-----------|---------|--------------|
| Compute | GKE Autopilot | $800 |
| Compute | Cloud Run | $200 |
| Database | Cloud SQL (HA) | $400 |
| Cache | Memorystore | $200 |
| Storage | Cloud Storage | $100 |
| Network | Load Balancer + CDN | $150 |
| Monitoring | Cloud Operations | $100 |
| Other | DNS, KMS, etc. | $50 |
| **Total** | | **~$2,000/month** |

### Cost Optimization Strategies

1. **Committed Use Discounts**: 30% savings on compute
2. **Spot Instances**: Dev/staging environments
3. **Auto-scaling**: Scale-to-zero for non-prod
4. **Right-sizing**: Regular resource optimization
5. **Reserved Capacity**: Database committed pricing

---

## Summary

The deployment and operations architecture provides:

- **Automated CI/CD**: 25-minute pipeline with security gates
- **Multi-environment**: Dev, staging, production with GitOps
- **Safe Deployments**: Canary releases with automatic rollback
- **Full Observability**: Metrics, logs, traces, and alerting
- **Disaster Recovery**: RTO <1 hour, RPO <5 minutes
- **Operational Excellence**: Runbooks, SLOs, and on-call procedures

---

**Document Status:** Complete
**Phase Complete:** SPARC Architecture Phase
**Next Phase:** SPARC Refinement (TDD Implementation)

---

END OF SPARC ARCHITECTURE PART 4
