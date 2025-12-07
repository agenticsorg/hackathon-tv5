# SPARC Architecture Phase - Part 4: GCP Infrastructure Architecture

**Version:** 1.0.0
**Phase:** SPARC Architecture
**Date:** 2025-12-06
**Status:** Complete

---

## Executive Summary

This document defines the complete GCP infrastructure architecture for the Media Gateway platform, designed to support 100K-1M users with sub-500ms latency, 99.9% availability, and <$4,000/month operational cost.

### Key Infrastructure Components

- **Compute**: GKE Autopilot (Rust services) + Cloud Run (TypeScript services) + Cloud Functions (event handlers)
- **Network**: Multi-zone VPC + Cloud CDN + L7 Load Balancer + Cloud Armor
- **Storage**: Cloud SQL (PostgreSQL HA) + Memorystore (Redis) + Cloud Storage (multi-region)
- **Container**: Artifact Registry with vulnerability scanning
- **Observability**: Cloud Monitoring + Cloud Logging + Cloud Trace
- **Security**: Workload Identity + Binary Authorization + Secret Manager

### Monthly Cost Breakdown (100K Users)

| Service | Monthly Cost |
|---------|-------------|
| GKE Autopilot | $800-$1,200 |
| Cloud Run | $150-$300 |
| Cloud SQL HA | $600-$800 |
| Memorystore Redis | $200-$250 |
| Cloud Storage | $100-$150 |
| Cloud CDN | $80-$120 |
| Load Balancer | $150-$200 |
| Cloud Functions | $50-$100 |
| Monitoring/Logging | $100-$150 |
| Pub/Sub | $40-$60 |
| **TOTAL** | **$2,270-$3,330** |

**Budget Reserve**: $670-$1,730

---

## 1. Compute Architecture

### 1.1 GKE Autopilot - Core Services

**Services Deployed**:
- Discovery Engine (Rust, 2-6 pods)
- SONA Engine (Rust, 2-4 pods)  
- Ruvector Service (Rust, 2-4 pods)
- Auth Service (Rust, 2-4 pods)

**Configuration**:
- Region: us-central1 (multi-zone)
- Release Channel: REGULAR
- Workload Identity: Enabled
- Binary Authorization: Enabled
- Network Policies: Deny-by-default

**Auto-scaling**: CPU/memory-based HPA with custom metrics (SONA latency, Ruvector QPS)

**Cost**: $800-$1,200/month

### 1.2 Cloud Run - Serverless Workloads

**Services Deployed**:
- MCP Server (Node.js, min=0, max=10)
- API Gateway (Express, min=1, max=20)
- Web App (Next.js, min=2, max=15)

**Configuration**:
- Runtime: nodejs20
- CPU Boost: Enabled (fast cold starts)
- Concurrency: 80-100 req/container
- Timeout: 60-300s

**Cost**: $150-$300/month

### 1.3 Cloud Functions - Event Handlers

**Functions**:
- Platform Ingestion Trigger (Pub/Sub)
- Watchlist Sync Handler (Pub/Sub)
- Embedding Generation Job (HTTP/Scheduler)

**Cost**: $50-$100/month

---

## 2. Network Architecture

### 2.1 VPC Design

```
us-central1 VPC
├── GKE Pods Subnet: 10.0.0.0/20 (4,096 IPs)
├── GKE Services Subnet: 10.1.0.0/20 (4,096 IPs)
├── Cloud Run Subnet: 10.2.0.0/24 (256 IPs)
├── Private Services: 10.3.0.0/24 (Cloud SQL, Memorystore)
└── Management: 10.4.0.0/28 (Bastion, CI/CD)
```

**Firewall Rules**:
- Allow health checks from GCP ranges
- Deny default SSH (port 22)
- Allow internal GKE traffic
- Restrict egress to specific ports

### 2.2 Load Balancer Configuration

**Type**: External HTTPS Load Balancer (L7)

**SSL**: Google-managed certificates for api.mediagateway.io, app.mediagateway.io, mcp.mediagateway.io

**Backends**:
- API Gateway (Cloud Run)
- Web App (Cloud Run) with CDN
- MCP Server (Cloud Run, no CDN)

**Cloud Armor**: Rate limiting (1000 req/min per IP), DDoS protection

**Cost**: $150-$200/month

### 2.3 Cloud CDN

**Cache Policies**:
- Static assets: 24h TTL
- API responses: Origin headers
- Content metadata: 1h TTL

**Egress**: 1TB/month

**Cost**: $80-$120/month

---

## 3. Storage Architecture

### 3.1 Cloud SQL (PostgreSQL 15)

**Configuration**:
- Instance: db-custom-2-7680 (2 vCPU, 7.68 GB RAM)
- High Availability: Regional (multi-zone)
- Replication: Synchronous to standby
- Backups: Daily at 3 AM UTC, 7-day retention
- PITR: Enabled (7-day transaction log)

**Read Replicas**:
- us-central1 (same specs, for read scaling)
- us-east1 (cross-region, for disaster recovery)

**Extensions**:
- pgvector (for embeddings)
- pg_stat_statements (for query analysis)

**Cost**: $600-$800/month

### 3.2 Memorystore (Redis 7.0)

**Configuration**:
- Tier: STANDARD_HA (multi-zone)
- Memory: 6 GB
- Eviction: allkeys-lru

**Usage Patterns**:
- Session cache (1h TTL)
- Content cache (5min TTL)
- API response cache (1min TTL)
- Rate limit counters (1min TTL)
- Embedding cache (24h TTL)

**Cost**: $200-$250/month

### 3.3 Cloud Storage

**Buckets**:
- media-gateway-assets-prod (multi-region, STANDARD → NEARLINE → COLDLINE lifecycle)
- media-gateway-backups-prod (NEARLINE)
- media-gateway-embeddings-prod (us-central1, STANDARD)

**CDN Backend**: Enabled for assets bucket

**Cost**: $100-$150/month

---

## 4. Container Architecture

### 4.1 Artifact Registry

**Repositories**:
- production (immutable tags)
- staging (14-day retention)
- development (7-day retention)

**Vulnerability Scanning**: Enabled on push, block CRITICAL/HIGH

**Binary Authorization**: Require signed images + vulnerability scan

### 4.2 Multi-stage Builds

**Rust Services** (distroless base):
- Unoptimized: 1.2GB → Optimized: 45MB (96% reduction)

**Node.js Services** (alpine base):
- Unoptimized: 800MB → Optimized: 120MB (85% reduction)

---

## 5. Auto-scaling Configuration

### 5.1 Horizontal Pod Autoscaler

**Discovery Engine**:
- Min: 2, Max: 6
- Target: 70% CPU, 80% memory, 1000 req/s

**SONA Engine**:
- Min: 2, Max: 4
- Target: 5ms inference latency

**Ruvector Service**:
- Min: 2, Max: 4
- Target: 1000 QPS

### 5.2 Cloud Run Autoscaling

**MCP Server**: min=0 (scale-to-zero), max=10
**API Gateway**: min=1 (always warm), max=20
**Web App**: min=2 (multi-region), max=15

### 5.3 Development Scale-to-Zero

- Dev cluster scales to 0 after 30min idle
- Weekend shutdown (Friday 10PM → Monday 8AM)
- **Savings**: $400-$600/month

---

## 6. High Availability & Disaster Recovery

### 6.1 Multi-Zone Deployment

**GKE**: Regional cluster across us-central1-a/b/c
**Cloud SQL**: Regional HA with synchronous replication
**Memorystore**: Standard HA tier

**SLA**: 99.9% composite availability (43.8 min/month downtime)

### 6.2 Regional Failover

**Primary**: us-central1
**Secondary**: us-east1

**DR Strategy**:
- Cloud SQL read replica in us-east1
- Promote to primary manually in disaster
- RTO: <30 minutes, RPO: <5 minutes

**DNS Failover**: Health checks with 60s TTL

### 6.3 Backup Strategy

**Cloud SQL**:
- Automated backups: Daily, 7-day retention
- Point-in-time recovery: 7 days
- Manual backups before schema changes

**Memorystore**: Daily snapshots, 7-day retention

**Monthly restore testing** to staging environment

---

## 7. Cost Optimization

### 7.1 Committed Use Discounts

- 1-year compute commitment: 37% savings ($88/month)
- 1-year Cloud SQL commitment: 20% savings ($160/month)
- **Total annual savings**: $2,976

### 7.2 Spot Instances

- GKE Autopilot: 30% spot for batch workloads
- **Savings**: $150-$200/month

### 7.3 Resource Right-Sizing

**Monthly review** using GKE Cost Optimization Insights:
- Discovery Engine: -25% CPU/memory → $30/month
- SONA Engine: -19% memory → $20/month
- Ruvector Service: -17% CPU, -12.5% memory → $40/month
- **Total savings**: $90/month

### 7.4 Cost Allocation Tags

**Labels**: environment (prod/staging/dev), service, cost_center, team

**Tracking**: Monthly cost reports by service and environment

---

## 8. Infrastructure as Code

### 8.1 Terraform Structure

```
infrastructure/
├── environments/production/staging/development
├── modules/gke-autopilot/cloud-run/cloud-sql/memorystore/networking/security
├── global/artifact-registry/iam/dns
└── README.md
```

### 8.2 CI/CD Pipeline

**Terraform Plan**: Automated on PR (with plan comment)
**Terraform Apply**: Manual approval required
**Workload Identity**: No service account keys

---

## 9. Observability

### 9.1 Monitoring

- **Cloud Monitoring**: Custom metrics (SONA latency, Ruvector QPS, cache hit ratio)
- **Prometheus**: Scraped from GKE pods (30s interval)
- **Dashboards**: Grafana for service health

### 9.2 Logging

- **Cloud Logging**: 30-day retention (default), 400-day (audit)
- **Log Sinks**: Error logs → Pub/Sub alerts, Audit logs → BigQuery

### 9.3 Tracing

- **Cloud Trace**: 10% sampling (Discovery Engine), 100% (MCP Server)
- **W3C Trace Context**: Propagated across services

**Cost**: $100-$150/month

---

## 10. Security

### 10.1 Workload Identity

- GKE pods use GCP service accounts (no keys)
- Cloud Run services use service accounts
- Least-privilege IAM roles

### 10.2 Binary Authorization

- Require signed images from CI/CD
- Block images with HIGH/CRITICAL vulnerabilities
- Allowlist for distroless base images

### 10.3 Secrets Management

- Secret Manager for API keys, credentials
- Cloud SQL passwords auto-rotated
- Encrypted at rest with CMEK

### 10.4 Network Security

- VPC firewall rules (deny-by-default)
- Cloud Armor for DDoS/rate limiting
- Private Service Access for Cloud SQL/Memorystore

---

## Summary

This GCP Infrastructure Architecture delivers:

✅ **Scalability**: 100K → 1M users with minimal changes
✅ **Availability**: 99.9% SLA with multi-zone deployment
✅ **Performance**: Sub-500ms latency with CDN + caching
✅ **Cost**: $2,270-$3,330/month (within $4,000 budget)
✅ **Security**: Workload Identity, Binary Authorization, Cloud Armor
✅ **Automation**: Terraform IaC with CI/CD pipelines

**Next Steps**: SPARC Completion phase - Implementation roadmap and deployment plan

---

**Document Status:** Complete
**Related Documents**: 
- SPARC_ARCHITECTURE_PART_1.md (System Overview)
- SPARC_ARCHITECTURE_PART_2.md (Microservices Architecture)
- SPARC_ARCHITECTURE_PART_3.md (Integration Architecture)

---

END OF GCP INFRASTRUCTURE ARCHITECTURE
