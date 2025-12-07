# SPARC Phase 3: Master Architecture Document

**Document Version:** 1.0.0
**Phase:** SPARC Architecture (Phase 3)
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Part 1: High-Level System Overview](#part-1-high-level-system-overview)
3. [Part 2: Microservices Architecture](#part-2-microservices-architecture)
4. [Part 3: Integration Architecture](#part-3-integration-architecture)
5. [Part 4: Deployment, DevOps, and Operations](#part-4-deployment-devops-and-operations)
6. [API Architecture](#api-architecture)
7. [Security Architecture](#security-architecture)
8. [Data Architecture](#data-architecture)
9. [GCP Infrastructure Architecture](#gcp-infrastructure-architecture)
10. [Architecture Index and Quick Reference](#architecture-index-and-quick-reference)

---

# Executive Summary

This master document consolidates all Phase 3 SPARC Architecture documentation for the Media Gateway platform. The architecture defines a microservices-based, event-driven platform designed for:

- **Performance:** Sub-100ms SONA personalization, <500ms end-to-end search
- **Scalability:** 100K-1M concurrent users with horizontal scaling
- **Availability:** 99.9% SLA with multi-zone deployment
- **Cost:** <$4,000/month operational budget
- **Security:** Defense-in-depth with OAuth 2.0 + PKCE, mTLS, and compliance

### Technology Stack Overview

| Layer | Technology |
|-------|------------|
| **Languages** | Rust (80%), TypeScript (20%), Python (ML) |
| **API Framework** | Actix-web (Rust), Fastify (Node.js) |
| **Database** | PostgreSQL 15, Redis 7, Qdrant |
| **Real-time** | PubNub |
| **Infrastructure** | GCP, GKE Autopilot, Cloud Run |
| **CI/CD** | GitHub Actions, ArgoCD |
| **Observability** | Cloud Monitoring, Prometheus, Grafana |

### Monthly Cost Summary (100K Users)

| Service | Monthly Cost |
|---------|-------------|
| GKE Autopilot | $800-$1,200 |
| Cloud Run | $150-$300 |
| Cloud SQL HA | $600-$800 |
| Memorystore Redis | $200-$250 |
| Cloud Storage | $100-$150 |
| Cloud CDN | $80-$120 |
| Load Balancer | $150-$200 |
| Monitoring/Logging | $100-$150 |
| **TOTAL** | **$2,270-$3,330** |

---

# Part 1: High-Level System Overview

## 1.1 Architecture Vision

The Media Gateway architecture is a **microservices-based, event-driven platform** designed to deliver sub-500ms content discovery across 150+ streaming platforms while maintaining 99.9% availability under $4,000/month operational cost.

### Architectural Style

- **Primary Pattern:** Microservices with event-driven coordination
- **Communication:** Synchronous (gRPC/REST) + Asynchronous (PubNub/Event Bus)
- **Deployment:** Cloud-native on GCP (GKE Autopilot + Cloud Run)
- **Data Management:** Polyglot persistence (PostgreSQL, Redis, Qdrant, SQLite)

### Quality Attribute Trade-offs

| Quality Attribute | Priority | Approach |
|-------------------|----------|----------|
| **Performance** | Critical | Rust for hot paths, aggressive caching, vector search |
| **Availability** | Critical | Multi-zone GKE, circuit breakers, graceful degradation |
| **Scalability** | High | Horizontal pod autoscaling, read replicas, CDN |
| **Security** | Critical | Zero-trust, OAuth 2.0 + PKCE, no credential storage |
| **Cost** | High | Autopilot GKE, Cloud Run scale-to-zero, preemptible nodes |

## 1.2 System Context Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          MEDIA GATEWAY SYSTEM CONTEXT                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  EXTERNAL ACTORS (Human)                                                    │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐ │
│  │End Users     │   │Developers    │   │Operations    │   │Admins        │ │
│  │(Consumers)   │   │(AI Agents)   │   │(SRE/DevOps)  │   │(Platform)    │ │
│  └──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘ │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    MEDIA GATEWAY PLATFORM                            │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │   │
│  │  │ Web App      │  │ MCP Server   │  │ Admin        │              │   │
│  │  │ (Next.js)    │  │ (AI Agents)  │  │ Dashboard    │              │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘              │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │   │
│  │  │ Discovery    │  │ SONA         │  │ Sync         │              │   │
│  │  │ Engine       │  │ Intelligence │  │ Engine       │              │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘              │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │   │
│  │  │ PostgreSQL   │  │ Qdrant       │  │ Redis        │              │   │
│  │  │ (Metadata)   │  │ (Vectors)    │  │ (Cache)      │              │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│  EXTERNAL SYSTEMS (APIs)                                                    │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐ │
│  │YouTube API   │   │Streaming     │   │JustWatch     │   │PubNub        │ │
│  │(Direct)      │   │Availability  │   │API           │   │(Real-time)   │ │
│  └──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘ │
│                                                                              │
│  CLIENT APPLICATIONS                                                         │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐ │
│  │Mobile Apps   │   │Smart TV Apps │   │CLI Tool      │   │Claude/GPT-4  │ │
│  │(iOS/Android) │   │(Roku, LG, ..)│   │(Node.js)     │   │(MCP Client)  │ │
│  └──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 1.3 Container Architecture (C4 Model)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      CONTAINER ARCHITECTURE (C4 LEVEL 2)                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         CLIENT LAYER                                    │ │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐       │ │
│  │  │ Web App    │  │ Mobile App │  │ TV App     │  │ CLI        │       │ │
│  │  │ Next.js    │  │ React Nat. │  │ React TV   │  │ TypeScript │       │ │
│  │  └────────────┘  └────────────┘  └────────────┘  └────────────┘       │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         API GATEWAY LAYER                               │ │
│  │  ┌──────────────────────────────────────────────────────────────────┐  │ │
│  │  │ Cloud Load Balancer (Global)                                      │  │ │
│  │  │ - HTTPS/2 termination, TLS 1.3, DDoS protection, Cloud Armor     │  │ │
│  │  └──────────────────────────────────────────────────────────────────┘  │ │
│  │  ┌──────────────────────────────────────────────────────────────────┐  │ │
│  │  │ API Gateway Service (Fastify on Cloud Run)                        │  │ │
│  │  │ - Rate limiting, Auth validation, Request routing, Caching       │  │ │
│  │  └──────────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                      APPLICATION LAYER (GKE Autopilot)                  │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │ │
│  │  │ MCP Server      │  │ Discovery       │  │ Auth Service    │         │ │
│  │  │ TypeScript      │  │ Service (Rust)  │  │ Rust            │         │ │
│  │  │ 2-10 replicas   │  │ 3-20 replicas   │  │ 2-10 replicas   │         │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘         │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │ │
│  │  │ Recommendation  │  │ Sync Service    │  │ Ingestion       │         │ │
│  │  │ Service (SONA)  │  │ Rust            │  │ Service         │         │ │
│  │  │ 2-10 replicas   │  │ 2-5 replicas    │  │ 1-5 replicas    │         │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘         │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         DATA LAYER                                      │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │ │
│  │  │ Cloud SQL       │  │ Qdrant Vector   │  │ Memorystore     │         │ │
│  │  │ (PostgreSQL)    │  │ Database        │  │ (Redis)         │         │ │
│  │  │ 1 primary +     │  │ 768-dim         │  │ Sessions        │         │ │
│  │  │ 3 read replicas │  │ embeddings      │  │ Cache           │         │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘         │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │ │
│  │  │ Ruvector Store  │  │ Cloud Storage   │  │ Pub/Sub         │         │ │
│  │  │ (SQLite)        │  │ (GCS)           │  │ (Event Bus)     │         │ │
│  │  │ Graph DB        │  │ Backups         │  │ Async jobs      │         │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘         │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 1.4 Communication Patterns

| Pattern | Use Case | Technology | Latency |
|---------|----------|------------|---------|
| **Synchronous Request-Response** | Client → API Gateway → Services | REST/HTTP2, gRPC | <50ms p95 |
| **Asynchronous Messaging** | Cross-device sync, real-time updates | PubNub | <100ms p95 |
| **Event-Driven** | Ingestion pipeline, audit logging | Cloud Pub/Sub | <500ms p95 |
| **Streaming** | Real-time recommendations | Server-Sent Events (SSE) | <200ms p95 |

## 1.5 Core Services Overview

### Discovery Service (Tier 1)
- **Responsibility:** Natural language search, content lookup, availability filtering
- **Technology:** Rust 1.75+, Actix-web
- **Scaling:** 3-20 replicas, CPU-based autoscaling (>70%)
- **SLO:** 99.9% availability, <300ms p95 latency

### Recommendation Service (SONA Intelligence) (Tier 1)
- **Responsibility:** Personalized recommendations, contextual suggestions
- **Technology:** Rust, ONNX Runtime (inference)
- **Scaling:** 2-10 replicas, memory-bound (4GB per replica)
- **SLO:** 99.9% availability, <100ms p95 latency

### Sync Service (Tier 1)
- **Responsibility:** Real-time cross-device state synchronization
- **Technology:** Rust, CRDT (OR-Set for watchlists, LWW-Register for progress)
- **Scaling:** 2-5 replicas, WebSocket connection pooling
- **SLO:** 99.5% availability, <100ms sync latency

### Auth Service (Tier 1)
- **Responsibility:** User authentication, OAuth 2.0 flows, token management
- **Technology:** Rust, OAuth 2.0 + PKCE, Device Authorization Grant
- **Scaling:** 2-10 replicas, session state in Redis
- **SLO:** 99.9% availability, <50ms p95 latency

### MCP Server (Tier 1)
- **Responsibility:** Model Context Protocol for AI agent integration
- **Technology:** TypeScript, MCP SDK (@anthropic-ai/mcp)
- **Transport:** STDIO (Claude Desktop), SSE (web)
- **SLO:** 99.9% availability, <50ms MCP overhead

## 1.6 Key Architectural Decisions

### ADR-001: Microservices vs Monolith
**Decision:** Microservices architecture
- ✅ Horizontal scalability, polyglot support, independent deployments
- ❌ Operational complexity (mitigated by GKE Autopilot, Istio)

### ADR-002: Database Strategy (Polyglot Persistence)
**Decision:** Multiple specialized databases
- PostgreSQL: ACID transactions, relational data
- Qdrant: Vector similarity search (HNSW) - 150x faster than pgvector
- Redis: Sub-millisecond cache, sessions

### ADR-003: Cloud Provider (GCP)
**Decision:** Google Cloud Platform
- Vertex AI Integration, GKE Autopilot, BigQuery analytics
- 37% savings on 1-year committed use

### ADR-004: Programming Language (Rust)
**Decision:** Rust for all performance-critical services (80% coverage)
- 2-5x faster than Node.js for CPU-bound tasks
- Benchmarks: Rust Actix-web 15K RPS @ p95 <50ms

### ADR-005: Real-time Sync (PubNub)
**Decision:** PubNub for cross-device synchronization
- <100ms global message delivery
- $49/mo at 10M messages (vs $500/mo self-hosted)

### ADR-006: Vector Database (Qdrant)
**Decision:** Self-hosted Qdrant on GKE
- $0/month self-hosted vs $500/month Pinecone equivalent
- 20M vectors @ 768-dim in <60GB memory

---

# Part 2: Microservices Architecture

## 2.1 Design Principles

1. **Bounded Contexts**: Each service owns its domain and data
2. **API-First**: Services communicate via well-defined REST/gRPC interfaces
3. **Eventual Consistency**: Accept eventual consistency for non-critical paths
4. **Fail-Safe**: Services degrade gracefully under load
5. **Stateless Compute**: Application logic is stateless; state lives in databases
6. **Observability**: Every service emits metrics, logs, and traces

## 2.2 Service Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                     SERVICE DEPENDENCIES                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  MCP Service ────────┬──────────────────────────────────────────┐   │
│                      │                                           │   │
│  API Gateway ────────┼──────────────────────────────────────┐    │   │
│                      │                                       │    │   │
│                      ▼                                       ▼    ▼   │
│  Search Service ──▶ Content Service ◀─── Recommendation Service   │
│       │                  │                       │                 │
│       ▼                  ▼                       ▼                 │
│  Ruvector          PostgreSQL              AgentDB/Ruvector       │
│                                                                      │
│  Playback Service ──▶ Sync Service ──▶ PubNub                      │
│       │                  │                                          │
│       ▼                  ▼                                          │
│  Device Registry    CRDT State Store                               │
│                                                                      │
│  Auth Service ───────▶ All Services (JWT validation)               │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.3 Service Definitions

### API Gateway Service
- **Technology:** Kong Gateway 3.x
- **Deployment:** Kubernetes (DaemonSet), 1 per node
- **Scaling:** Auto (CPU > 70% or requests/sec > 1000), 2-10 instances

### Content Service
- **Technology:** Rust (Actix-web)
- **Deployment:** Kubernetes (Deployment), 2-6 instances
- **Data Ownership:** content, external_ids, platform_availability tables
- **Background Jobs:** catalog_refresh (6h), availability_sync (1h)

### Search Service
- **Technology:** Rust (Actix-web)
- **Deployment:** Kubernetes (Deployment), 2-8 instances
- **Algorithm:** Hybrid search (vector + keyword + graph) with RRF fusion
- **Data Ownership:** content_embeddings (Ruvector), query_cache (Redis)

### Recommendation Service (SONA)
- **Technology:** Rust (Actix-web) + PyTorch inference
- **Deployment:** Kubernetes (StatefulSet), 2-4 model-loaded instances
- **ML Architecture:** Base Model (100M params) + Per-user LoRA (256K params)
- **Training:** Triggered every 10 interactions per user

### Sync Service
- **Technology:** Rust (Tokio) + WebSocket
- **Deployment:** Kubernetes (StatefulSet), 2-4 instances
- **CRDT Types:** LWW-Register (playback), OR-Set (watchlist), HLC timestamps
- **Real-time:** PubNub integration (<100ms latency)

### Playback Service
- **Technology:** Rust (Actix-web)
- **Deployment:** Kubernetes (Deployment), 2-4 instances
- **Features:** Device management, deep link generation, platform routing

### Auth Service
- **Technology:** Rust (Actix-web + OAuth2/OIDC)
- **Deployment:** Kubernetes (Deployment), 2-4 instances
- **JWT:** RS256 algorithm, 1-hour access tokens, 7-day refresh tokens

### MCP Service
- **Technology:** TypeScript (Node.js 18+)
- **Deployment:** Kubernetes (Deployment), 2-4 instances
- **Transports:** STDIO (Claude Desktop), SSE (web)
- **Tools:** 10+ MCP tools (semantic_search, get_recommendations, etc.)

## 2.4 Inter-Service Communication

### Communication Patterns
```yaml
patterns:
  synchronous:
    protocol: gRPC (HTTP/2)
    use_cases: Service-to-service calls (<50ms)

  asynchronous:
    protocol: Kafka (event streaming)
    topics: content.ingested, content.updated, user.interaction

  real_time:
    protocol: PubNub
    channels: user.{user_id}.sync, device.{device_id}.presence
```

### Service Mesh (Istio)
- Mutual TLS (mTLS) between services
- Traffic management (retries: 3 attempts, 2s timeout)
- Circuit breaker (5 consecutive errors → 30s ejection)

## 2.5 Database Schema Organization

```
postgresql (primary data store)
├── content_schema
│   ├── content, external_ids, platform_availability
│   ├── content_images, credits, genres
├── user_schema
│   ├── users, user_preferences, devices, sessions
├── sync_schema
│   ├── sync_operations, playback_sessions, watchlists
└── auth_schema
    ├── oauth_clients, authorization_codes
    ├── device_codes, refresh_tokens
```

## 2.6 Caching Strategy (Valkey/Redis)

| Cache Layer | TTL | Keys |
|-------------|-----|------|
| L1 Gateway | 30s | rate_limit:{user_id}, trending_searches |
| L2 Service | 1-24h | content:{id}, user_profile:{user_id}, search:{query_hash} |
| L3 Embedding | 7d | embedding:{content_id}, external_id:{type}:{value} |

---

# Part 3: Integration Architecture

## 3.1 Integration Principles

1. **Adapter Pattern**: All external integrations isolated behind uniform interfaces
2. **Circuit Breaker**: Automatic fault isolation and recovery
3. **Rate Limiting**: Respect external API limits with multi-key rotation
4. **Caching**: Aggressive caching to reduce API calls and cost
5. **Fallback Chains**: Multiple data sources with automatic failover
6. **Idempotency**: All operations safe to retry

## 3.2 Integration Catalog

| Integration Type | Count | Purpose | Latency SLO | Availability SLO |
|-----------------|-------|---------|-------------|------------------|
| Streaming Platforms | 150+ | Content catalog & availability | <2s | 99.5% |
| Metadata Providers | 5 | Enriched metadata | <1s | 99.9% |
| Real-time Sync | 1 (PubNub) | Cross-device state sync | <100ms | 99.99% |
| AI/ML Services | 3 | Embeddings, LoRA, inference | <500ms | 99.9% |
| Webhooks | 2-way | Event notifications | <5s | 99.5% |

## 3.3 Streaming Platform Integrations

### Platform Adapter Interface
```typescript
interface PlatformAdapter {
  readonly platformId: string;
  readonly platformName: string;
  readonly supportedRegions: string[];

  getCatalog(region: string, options?: CatalogOptions): Promise<CatalogResponse>;
  searchContent(query: string, region: string): Promise<SearchResult[]>;
  getContentDetails(platformContentId: string, region: string): Promise<ContentDetails>;
  checkAvailability(contentId: string, region: string): Promise<AvailabilityInfo>;
  generateDeepLink(contentId: string, platform: 'ios' | 'android' | 'web'): DeepLink;
}
```

### Netflix Integration
- **Primary Source:** Streaming Availability API
- **Fallback Sources:** Watchmode API, JustWatch API
- **Cache TTL:** Catalog 6h, Content details 24h, Availability 1h
- **Circuit Breaker:** 5 failures → 60s timeout

### YouTube Direct Integration
- **Auth Method:** OAuth 2.0 + PKCE
- **Daily Quota:** 10,000 units (5 API keys with rotation)
- **Rate Limiting:** 10 req/s, 600 req/min, 20 burst

## 3.4 PubNub Integration Architecture

### Channel Architecture
```yaml
channel_naming:
  user_sync: "user.{user_id}.sync"
  device_status: "user.{user_id}.devices"
  watchlist: "user.{user_id}.watchlist"
  playback: "user.{user_id}.playback"

message_types:
  - watchlist_update
  - device_handoff
  - playback_control
  - watch_progress
```

### Presence Configuration
- Timeout: 300 seconds
- Heartbeat interval: 10 seconds
- Device metadata: type, name, capabilities, current_activity

### History Configuration
- user_sync: 24h retention
- watchlist: 7d retention
- playback: no storage (ephemeral)

## 3.5 Event-Driven Integration (Kafka)

### Topics Configuration
```yaml
topics:
  content_ingested:
    partitions: 12
    retention_ms: 604800000  # 7 days

  user_interaction:
    partitions: 24
    retention_ms: 2592000000  # 30 days

  availability_changed:
    partitions: 12
    retention_ms: 2592000000
```

### Consumer Groups
- **content-indexer-v1**: Updates Ruvector index, PostgreSQL
- **recommendation-updater-v1**: Updates user profiles, triggers LoRA training
- **availability-monitor-v1**: Checks watchlists, sends expiry notifications

## 3.6 AI/ML Integrations

### Embedding Service
- **Model:** sentence-transformers/all-MiniLM-L6-v2 (384-dim)
- **Deployment:** Cloud Run
- **Rate Limit:** 1000 req/min
- **Cache TTL:** 90 days, 95%+ hit rate target

### LoRA Training Service
- **Base Model:** recommendation_transformer_v1
- **LoRA Rank:** 8, Alpha: 16
- **Training:** 5 epochs, 16 batch size
- **Artifact:** ~10KB per user

---

# Part 4: Deployment, DevOps, and Operations

## 4.1 CI/CD Pipeline Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           CI/CD PIPELINE FLOW                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌────────────┐ │
│  │  Code   │──▶│  Build   │──▶│   Test   │──▶│ Security │──▶│   Deploy   │ │
│  │  Push   │   │  Stage   │   │  Stage   │   │   Scan   │   │   Stage    │ │
│  └─────────┘   └──────────┘   └──────────┘   └──────────┘   └────────────┘ │
│                                                                              │
│  Total Pipeline Duration: ~25 minutes                                       │
│  - Rust Build: 5m | Node Build: 2m | Docker: 3m                            │
│  - Unit Tests: 4m | Integration: 6m | Security: 2m                         │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Pipeline Stages
1. **Build:** Container build, dependency caching (Cargo, npm)
2. **Test:** Unit tests, integration tests, E2E tests (>80% coverage)
3. **Security:** SAST (cargo-audit, npm audit), container scanning (Trivy)
4. **Deploy:** Staging (automatic), Production (canary with manual approval)

## 4.2 Environment Strategy

| Environment | Purpose | Infra | Monthly Cost |
|-------------|---------|-------|--------------|
| **Local** | Developer testing | Docker Compose | - |
| **Feature** | PR validation | GKE (ephemeral) | Variable |
| **Development** | Integration | GKE (shared), 1 zone | ~$150 |
| **Staging** | Pre-production | GKE (prod-like), 2 zones | ~$400 |
| **Production** | Live users | GKE (HA), 3 zones | ~$2,500 |

## 4.3 Deployment Strategies

| Service | Strategy | Rationale |
|---------|----------|-----------|
| API Gateway | Blue-Green | Zero-downtime critical |
| MCP Server | Rolling | Stateless, quick restart |
| Discovery Service | Canary | Search quality validation |
| SONA Engine | Canary | Recommendation impact |
| Auth Service | Blue-Green | Security critical |

### Canary Deployment Phases
1. **Phase 1 (5%):** 15 minutes, monitor error rate/latency
2. **Phase 2 (25%):** 30 minutes, add business metrics (CTR)
3. **Phase 3 (50%):** 1 hour, full observability
4. **Phase 4 (100%):** Keep old version for 1 hour fast rollback

### Rollback Triggers (Automatic)
- Error rate > 5% for 5 minutes
- P99 latency > 3x baseline for 10 minutes
- Pod crash loop (>3 restarts in 5 minutes)
- Total MTTR target: <10 minutes

## 4.4 Infrastructure as Code

### Repository Structure
```
infrastructure/
├── terraform/
│   ├── modules/
│   │   ├── gke-cluster/
│   │   ├── cloud-sql/
│   │   ├── memorystore/
│   │   └── networking/
│   └── environments/
│       ├── dev/
│       ├── staging/
│       └── production/
├── kubernetes/
│   ├── base/ (Kustomize)
│   └── overlays/
└── helm/
    └── media-gateway/
```

### GitOps with ArgoCD
- Development: Auto-sync enabled
- Staging: Auto-sync enabled
- Production: Manual sync required (approval gate)

## 4.5 Observability Stack

### Three Pillars
| Pillar | Technology | Purpose |
|--------|-----------|---------|
| **Metrics** | Cloud Monitoring + Prometheus + Grafana | Service-level metrics, dashboards |
| **Logs** | Cloud Logging (structured JSON) | 30d hot, 90d warm, 1yr cold |
| **Traces** | Cloud Trace + OpenTelemetry | Distributed tracing, 10% sampling |

### Key SLIs
| Service | Metric | SLO Target |
|---------|--------|------------|
| API Gateway | Request latency p99 | <500ms |
| Discovery | Search latency p95 | <400ms |
| SONA | Recommendation latency | <200ms |
| Sync Service | Sync latency p99 | <100ms |

### Alert Severity Levels
- **P1 (Critical):** Page immediately, 15 min response
- **P2 (High):** Page during business hours, 1 hour response
- **P3 (Medium):** Ticket, 24 hour response
- **P4 (Low):** Backlog, 1 week response

## 4.6 Disaster Recovery

### RTO/RPO Targets
| Component | RTO | RPO | Strategy |
|-----------|-----|-----|----------|
| API Gateway | 5 min | N/A | Multi-zone, auto-failover |
| PostgreSQL | 1 hour | 5 min | Point-in-time recovery |
| Redis | 15 min | 5 min | Failover replica |
| Qdrant | 4 hours | 24 hours | Daily snapshots |

### Backup Strategy
- **PostgreSQL:** Continuous WAL + daily automated (7d) + weekly (4w) + monthly (12m)
- **Redis:** RDB snapshots every 15 minutes, AOF 1s fsync
- **Qdrant:** Daily collection snapshots, 7-day retention

---

# API Architecture

## 5.1 API Design Principles

1. **Protocol Diversity**: REST, MCP, GraphQL, WebSocket
2. **AI-First Design**: ARW protocol (85% token reduction)
3. **Production-Grade**: Circuit breakers, rate limiting, observability

## 5.2 API Gateway Design

### Kong Configuration
```yaml
plugins:
  - request-id (X-Request-ID)
  - correlation-id (X-Correlation-ID)
  - ai-agent-detection (AI-* headers)
  - rate-limiting (Redis policy)
  - circuit-breaker (5 failures → 30s break)
```

### Circuit Breaker Configuration
| Service | Request Threshold | Timeout | Error % |
|---------|------------------|---------|---------|
| content-service | 20 | 3s | 50% |
| search-service | 10 | 2s | 40% |
| recommendation-service | 15 | 5s | 50% |

## 5.3 REST API Design

### Resource Hierarchy
```
/api/v1/
├── /content (/movies, /tv, /trending)
├── /search (/semantic, /faceted, /autocomplete)
├── /discover (/movies, /tv, /popular)
├── /recommendations (/for-you, /similar, /trending)
├── /platforms (/{platform_id}/catalog)
├── /user (/profile, /watchlist, /history, /preferences)
└── /genres (/movies, /tv)
```

### Rate Limiting Tiers
| Tier | Per Second | Per Minute | Per Hour |
|------|------------|------------|----------|
| Anonymous | 5 | 100 | 1000 |
| Free User | 10 | 200 | 2000 |
| Pro User | 50 | 1000 | 10000 |
| Enterprise | 200 | 5000 | 50000 |

## 5.4 MCP Protocol Design

### MCP Tools
```typescript
const mcpTools = [
  'semantic_search',      // Natural language search
  'get_recommendations',  // Personalized recommendations
  'check_availability',   // Platform availability
  'get_content_details',  // Full content metadata
  'list_devices',        // Registered devices
];
```

### Transports
- **STDIO:** JSON-RPC 2.0 over stdin/stdout (Claude Desktop)
- **SSE:** Event stream endpoint (/mcp/events) + POST (/mcp/tools/call)

## 5.5 GraphQL Schema

```graphql
type Query {
  movie(id: ID!): Movie
  tvShow(id: ID!): TVShow
  search(query: String!, filters: SearchFilters, limit: Int): SearchResults!
  recommendations(input: RecommendationInput!): RecommendationResults!
}

type Subscription {
  contentUpdated(contentId: ID!): Content!
  availabilityChanged(contentId: ID!, region: String!): [PlatformAvailability!]!
}
```

### Query Complexity Limits
- Max depth: 7
- Max complexity: 1000
- Search: limit × 50 cost
- Recommendations: limit × 100 cost (AI inference expensive)

## 5.6 Real-time API (WebSocket)

### Socket.IO Configuration
- Redis adapter for horizontal scaling
- Authentication middleware (JWT)
- Rate limiting (100 events/60s per user)
- Reconnection: 10 attempts, exponential backoff, max 30s

### Event Types
```typescript
enum RealtimeEvent {
  CONTENT_UPDATED = 'content:updated',
  WATCHLIST_UPDATED = 'watchlist:updated',
  RECOMMENDATIONS_READY = 'recommendations:ready',
  STATE_SYNCED = 'state:synced',
}
```

---

# Security Architecture

## 6.1 Defense in Depth

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    SECURITY LAYERS (DEFENSE IN DEPTH)                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Layer 7: Compliance & Audit (GDPR, CCPA, VPPA)                         │
│  Layer 6: Application Security (Zod validation, OWASP Top 10)           │
│  Layer 5: Authentication & Authorization (OAuth 2.0 + PKCE, RBAC)       │
│  Layer 4: Data Protection (AES-256-GCM, TLS 1.3)                        │
│  Layer 3: Network Security (Cloud Armor WAF, VPC firewall)              │
│  Layer 2: Infrastructure Security (Workload Identity, Binary Auth)      │
│  Layer 1: Physical Security (GCP SOC 2, ISO 27001)                      │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## 6.2 Security Principles

1. **Zero Trust:** Never trust, always verify
2. **Least Privilege:** Minimal permissions by default
3. **Privacy by Design:** User data protection built-in
4. **Fail Secure:** Deny access on error
5. **Defense in Depth:** Multiple security layers

## 6.3 Identity and Access Management

### User Authentication (OAuth 2.0 + PKCE)
- **Providers:** Google, GitHub
- **PKCE:** S256 code challenge method
- **Security:** State parameter (CSRF), HTTPS-only redirects

### Service-to-Service Authentication (mTLS)
- **Certificate Authority:** Google CAS
- **Algorithm:** ECDSA P-256
- **Validity:** 90 days, auto-rotation at 60 days

### API Key Management
- **Format:** `mg_user_<base62(128bit)>`, `mg_svc_<base62(128bit)>`
- **Storage:** SHA-256 hash with per-key salt
- **Rate Limits:** User 1000/15min, Service 10000/15min

## 6.4 Authorization (RBAC)

### Role Hierarchy
```
admin
├── moderator → premium_user → basic_user → guest
└── service_account → ingestion_service, mcp_server, sona_engine
```

### Permission Format
`<resource>:<action>:<scope>` (e.g., "content:read:*")

### OAuth Scopes
- Read: `read:content`, `read:watchlist`, `read:preferences`
- Write: `write:watchlist`, `write:preferences`
- Special: `playback:control` (requires consent), `admin:full`

## 6.5 Data Protection

### Encryption at Rest
- **Key Management:** Google Cloud KMS (HSM-backed)
- **Algorithm:** AES-256-GCM
- **Rotation:** Automatic 90 days

### Encryption in Transit
- **TLS Version:** 1.3 minimum
- **Cipher Suites:** TLS_AES_256_GCM_SHA384, TLS_CHACHA20_POLY1305_SHA256
- **HSTS:** max-age 1 year, includeSubdomains, preload

### Secrets Management
- **Provider:** Google Secret Manager
- **Access Control:** Workload Identity + IAM (least privilege)
- **Versioning:** Last 10 versions retained

## 6.6 Network Security

### Cloud Armor (WAF + DDoS)
- Block known malicious IPs
- Rate limit: 1000 req/60s per IP
- SQL injection detection and blocking

### VPC Firewall
- Deny-by-default (priority 65534)
- Allow HTTPS ingress (port 443)
- Allow GCP health check ranges

### Private Service Endpoints
- PostgreSQL: Private IP via VPC peering, SSL required
- Redis: VPC-native Private IP, transit encryption
- Cloud KMS/Secret Manager: Private Service Connect

## 6.7 Token Security

### JWT Configuration
- **Algorithm:** RS256 (asymmetric)
- **Access Token:** 1 hour expiry
- **Refresh Token:** 7 days, rotation on every use

### Token Storage (Client-Side)
| Platform | Access Token | Refresh Token |
|----------|-------------|---------------|
| Web | Memory only | httpOnly cookie (secure, sameSite:strict) |
| Mobile | Memory | Keychain/EncryptedSharedPreferences |
| CLI | Memory | ~/.config/ with 0600 permissions |

### Token Revocation
- Storage: Redis SET with TTL
- Triggers: Logout, password change, admin action, suspicious activity
- Verification: Check Redis before JWT validation

## 6.8 Audit and Compliance

### Audit Logging
- **Events:** authentication, data_access, security_events
- **Format:** Structured JSON
- **Storage:** Cloud Logging (90d hot), Cloud Storage (2yr cold)

### GDPR Compliance
- Right to access: GET /api/gdpr/data-export (30-day SLA)
- Right to erasure: DELETE /api/gdpr/delete-account
- Right to portability: Machine-readable JSON export

### CCPA Compliance
- Right to know, delete (45-day SLA)
- Right to opt-out (no personal data sale)

### VPPA Compliance
- Explicit opt-in consent for watch history
- 90-day retention, then anonymized
- No sharing without explicit consent

## 6.9 Security SLOs

| Metric | Target |
|--------|--------|
| Authentication latency | <200ms p95 |
| Authorization latency | <10ms p95 |
| Security incident MTTD | <15 minutes |
| Security incident MTTR | <4 hours |
| Vulnerability patching | <24 hours (critical) |

---

# Data Architecture

## 7.1 Multi-Database Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                   DATA STORAGE STRATEGY                           │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  PostgreSQL (Primary Store)                                      │
│  ├── Use Case: OLTP, structured data, ACID transactions          │
│  ├── Data: Content metadata, users, interactions, auth          │
│  ├── Size: 500 GB (20M content items)                            │
│  └── Query Load: 50K QPS (80% reads, 20% writes)                 │
│                                                                   │
│  Redis/Valkey (Cache + Sessions)                                 │
│  ├── Use Case: L2 cache, session store, rate limiting           │
│  ├── Data: Hot content, search results, user preferences        │
│  ├── Size: 8 GB in-memory                                        │
│  └── Query Load: 200K QPS                                        │
│                                                                   │
│  Qdrant (Vector Database)                                        │
│  ├── Use Case: Semantic search, similarity matching              │
│  ├── Data: 768-dim embeddings (content, user prefs)             │
│  ├── Size: 80 GB (20M vectors)                                   │
│  └── Query Load: 5K QPS (vector searches)                        │
│                                                                   │
│  PubNub (Real-time Messaging)                                    │
│  ├── Use Case: Cross-device sync, presence, CRDT propagation    │
│  ├── Latency: <50ms p95                                          │
│  └── Throughput: 1M messages/day                                 │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

## 7.2 Database Sizing

| Database | Size | Records | QPS |
|----------|------|---------|-----|
| PostgreSQL | 500 GB | 20M content | 50K |
| Redis | 8 GB | Hot data | 200K |
| Qdrant | 80 GB | 20M vectors | 5K |
| PubNub | N/A | 1M msg/day | N/A |

---

# GCP Infrastructure Architecture

## 8.1 Compute Architecture

### GKE Autopilot - Core Services
- **Services:** Discovery Engine, SONA Engine, Ruvector, Auth Service (Rust)
- **Region:** us-central1 (multi-zone)
- **Auto-scaling:** CPU/memory-based HPA with custom metrics
- **Cost:** $800-$1,200/month

### Cloud Run - Serverless Workloads
- **Services:** MCP Server (Node.js), API Gateway, Web App
- **Configuration:** CPU Boost, 80-100 req/container concurrency
- **Cost:** $150-$300/month

### Cloud Functions - Event Handlers
- **Functions:** Platform ingestion, watchlist sync, embedding generation
- **Cost:** $50-$100/month

## 8.2 Network Architecture

### VPC Design
```
us-central1 VPC
├── GKE Pods Subnet: 10.0.0.0/20 (4,096 IPs)
├── GKE Services Subnet: 10.1.0.0/20 (4,096 IPs)
├── Cloud Run Subnet: 10.2.0.0/24 (256 IPs)
├── Private Services: 10.3.0.0/24 (Cloud SQL, Memorystore)
└── Management: 10.4.0.0/28 (Bastion, CI/CD)
```

### Load Balancer
- **Type:** External HTTPS (L7)
- **SSL:** Google-managed certificates
- **Cloud Armor:** 1000 req/min per IP rate limiting

## 8.3 Storage Architecture

### Cloud SQL (PostgreSQL 15)
- **Instance:** db-custom-2-7680 (2 vCPU, 7.68 GB RAM)
- **HA:** Regional (multi-zone), synchronous replication
- **Backups:** Daily at 3 AM UTC, 7-day PITR
- **Extensions:** pgvector, pg_stat_statements
- **Cost:** $600-$800/month

### Memorystore (Redis 7.0)
- **Tier:** STANDARD_HA (multi-zone)
- **Memory:** 6 GB
- **Eviction:** allkeys-lru
- **Cost:** $200-$250/month

### Cloud Storage
- **Buckets:** assets-prod, backups-prod, embeddings-prod
- **Lifecycle:** STANDARD → NEARLINE → COLDLINE
- **Cost:** $100-$150/month

## 8.4 High Availability & Disaster Recovery

### Multi-Zone Deployment
- GKE: Regional across us-central1-a/b/c
- Cloud SQL: Regional HA
- Memorystore: Standard HA tier
- **SLA:** 99.9% composite availability

### Regional Failover
- Primary: us-central1
- Secondary: us-east1 (read replica)
- **RTO:** <30 minutes, **RPO:** <5 minutes

## 8.5 Cost Optimization

### Strategies
- Committed Use Discounts: 37% savings ($2,976/year)
- Spot Instances: 30% for batch workloads ($150-$200/month)
- Resource Right-Sizing: Monthly review ($90/month savings)
- Dev Scale-to-Zero: Weekend shutdown ($400-$600/month)

---

# Architecture Index and Quick Reference

## 9.1 Document Inventory

| Document | Size | Description |
|----------|------|-------------|
| SPARC_ARCHITECTURE_PART_1.md | 46KB | System context, containers, ADRs |
| SPARC_ARCHITECTURE_PART_2.md | 60KB | All 8 core services with APIs |
| SPARC_ARCHITECTURE_PART_3.md | 59KB | Platform APIs, PubNub, webhooks |
| SPARC_ARCHITECTURE_PART_4.md | 38KB | CI/CD, deployment, observability |
| SPARC_ARCHITECTURE_API.md | 53KB | REST, MCP, GraphQL, WebSocket |
| SPARC_ARCHITECTURE_SECURITY.md | 26KB | AuthN/AuthZ, encryption, compliance |
| SPARC_ARCHITECTURE_DATA.md | 4KB | Database schemas, caching |
| SPARC_ARCHITECTURE_INFRASTRUCTURE.md | 10KB | GCP, GKE, networking |

**Total:** ~295KB (~9,200 lines)

## 9.2 Service Quick Reference

| Service | Language | Port | Scaling |
|---------|----------|------|---------|
| API Gateway | TypeScript | 8080 | 3-20 replicas |
| MCP Server | TypeScript | 3000 | 2-10 replicas |
| Discovery Service | Rust | 8081 | 3-15 replicas |
| SONA Engine | Rust | 8082 | 2-10 replicas |
| Sync Service | Rust | 8083 | 3-12 replicas |
| Auth Service | Rust | 8084 | 3-10 replicas |
| Ingestion Service | Rust | 8085 | 2-8 replicas |

## 9.3 Performance Targets

| Metric | Target |
|--------|--------|
| Search latency (p95) | <400ms |
| SONA personalization | <5ms |
| Sync latency | <100ms |
| API availability | 99.9% |
| Monthly infra cost | <$4,000 |

## 9.4 Security Checklist (Pre-Deployment)

- [x] OAuth 2.0 + PKCE architecture designed
- [x] mTLS certificate strategy defined
- [x] RBAC model and policies documented
- [x] Encryption at rest/transit specifications
- [x] Cloud Armor WAF rules designed
- [x] Input validation schemas defined
- [x] JWT token lifecycle documented
- [x] Audit logging design complete
- [x] Compliance requirements mapped (GDPR, CCPA, VPPA)
- [x] Security monitoring plan created
- [x] Incident response procedures documented
- [x] Secrets management strategy defined

## 9.5 Implementation Priority

### Phase 1: Foundation (Week 1-2)
1. OAuth 2.0 + PKCE authentication
2. JWT token generation and validation
3. Basic RBAC implementation
4. TLS 1.3 configuration

### Phase 2: Data Protection (Week 3-4)
1. Cloud KMS integration
2. Database column encryption
3. Secrets Manager setup
4. Key rotation automation

### Phase 3: Network Security (Week 5-6)
1. Cloud Armor deployment
2. VPC firewall rules
3. Private Service Connect
4. mTLS for internal services

### Phase 4: Application Security (Week 7-8)
1. Input validation (Zod)
2. OWASP mitigations
3. Dependency scanning
4. Security testing

### Phase 5: Monitoring & Compliance (Week 9-10)
1. Audit logging
2. Security metrics
3. Incident response runbooks
4. GDPR/CCPA compliance

---

## Summary

This Phase 3 Master Architecture document consolidates all SPARC Architecture phase documentation, providing:

1. **System Overview:** Microservices-based, event-driven platform on GCP
2. **8 Core Services:** API Gateway, MCP, Discovery, SONA, Sync, Auth, Playback, Ingestion
3. **Integration Layer:** 150+ platform adapters, PubNub real-time, Kafka events, AI/ML pipeline
4. **Deployment Strategy:** GitOps with ArgoCD, canary releases, <10min MTTR
5. **Security Architecture:** 7-layer defense-in-depth, OAuth 2.0 + PKCE, GDPR/CCPA compliant
6. **Data Architecture:** PostgreSQL + Redis + Qdrant polyglot persistence
7. **GCP Infrastructure:** GKE Autopilot + Cloud Run, $2,270-$3,330/month

**Document Status:** Complete
**Next Phase:** SPARC Refinement (TDD Implementation)
**Review Required:** Architecture team, Security team, DevOps team

---

END OF PHASE 3 MASTER ARCHITECTURE DOCUMENT
