# SPARC Architecture — Part 1: High-Level System Overview

**Document Version:** 1.0.0
**SPARC Phase:** Architecture
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [System Context Diagram](#2-system-context-diagram)
3. [Container Architecture (C4 Model)](#3-container-architecture-c4-model)
4. [Component Overview](#4-component-overview)
5. [Technology Stack](#5-technology-stack)
6. [Key Architectural Decisions](#6-key-architectural-decisions)
7. [Cross-Cutting Concerns](#7-cross-cutting-concerns)

---

## 1. Executive Summary

### 1.1 Architecture Vision

The Media Gateway architecture is a **microservices-based, event-driven platform** designed to deliver sub-500ms content discovery across 150+ streaming platforms while maintaining 99.9% availability under $4,000/month operational cost. The system prioritizes:

- **Performance:** Sub-100ms SONA personalization, <500ms end-to-end search
- **Scalability:** 100K concurrent users → 1M users with horizontal scaling
- **Privacy:** On-device CRDT state, differential privacy, federated learning
- **Interoperability:** MCP server for AI agents, ARW manifest compliance

### 1.2 Architectural Style

**Primary Pattern:** Microservices with event-driven coordination
**Communication:** Synchronous (gRPC/REST) + Asynchronous (PubNub/Event Bus)
**Deployment:** Cloud-native on GCP (GKE Autopilot + Cloud Run)
**Data Management:** Polyglot persistence (PostgreSQL, Redis, Qdrant, SQLite)

### 1.3 Quality Attribute Trade-offs

| Quality Attribute | Priority | Approach |
|-------------------|----------|----------|
| **Performance** | Critical | Rust for hot paths, aggressive caching, vector search |
| **Availability** | Critical | Multi-zone GKE, circuit breakers, graceful degradation |
| **Scalability** | High | Horizontal pod autoscaling, read replicas, CDN |
| **Security** | Critical | Zero-trust, OAuth 2.0 + PKCE, no credential storage |
| **Cost** | High | Autopilot GKE, Cloud Run scale-to-zero, preemptible nodes |
| **Time-to-Market** | Medium | Leverage existing OSS (Ruvector, PubNub, hackathon-tv5) |

---

## 2. System Context Diagram

### 2.1 External Actors and Systems

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          MEDIA GATEWAY SYSTEM CONTEXT                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  EXTERNAL ACTORS (Human)                                                    │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐ │
│  │End Users     │   │Developers    │   │Operations    │   │Admins        │ │
│  │(Consumers)   │   │(AI Agents)   │   │(SRE/DevOps)  │   │(Platform)    │ │
│  └──────┬───────┘   └──────┬───────┘   └──────┬───────┘   └──────┬───────┘ │
│         │                  │                  │                  │          │
│         ▼                  ▼                  ▼                  ▼          │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                      │   │
│  │                    MEDIA GATEWAY PLATFORM                            │   │
│  │                                                                      │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │   │
│  │  │ Web App      │  │ MCP Server   │  │ Admin        │              │   │
│  │  │ (Next.js)    │  │ (AI Agents)  │  │ Dashboard    │              │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘              │   │
│  │                                                                      │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │   │
│  │  │ Discovery    │  │ SONA         │  │ Sync         │              │   │
│  │  │ Engine       │  │ Intelligence │  │ Engine       │              │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘              │   │
│  │                                                                      │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │   │
│  │  │ PostgreSQL   │  │ Qdrant       │  │ Redis        │              │   │
│  │  │ (Metadata)   │  │ (Vectors)    │  │ (Cache)      │              │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘              │   │
│  │                                                                      │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│         │                  │                  │                  │          │
│         ▼                  ▼                  ▼                  ▼          │
│  EXTERNAL SYSTEMS (APIs)                                                    │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐ │
│  │YouTube API   │   │Streaming     │   │JustWatch     │   │PubNub        │ │
│  │(Direct)      │   │Availability  │   │API           │   │(Real-time)   │ │
│  └──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘ │
│                                                                              │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐ │
│  │TMDb API      │   │Watchmode     │   │GCP Services  │   │Auth0/OAuth   │ │
│  │(Metadata)    │   │API           │   │(Infra)       │   │(Identity)    │ │
│  └──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘ │
│                                                                              │
│  CLIENT APPLICATIONS                                                         │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐ │
│  │Mobile Apps   │   │Smart TV Apps │   │CLI Tool      │   │Claude/GPT-4  │ │
│  │(iOS/Android) │   │(Roku, LG, ..)│   │(Node.js)     │   │(MCP Client)  │ │
│  └──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 System Boundary

**IN SCOPE:**
- Content metadata aggregation and normalization
- Natural language search and recommendations
- Real-time cross-device synchronization
- MCP protocol for AI agent integration
- Deep linking to streaming platforms

**OUT OF SCOPE:**
- Video streaming or transcoding
- DRM or content protection
- Subscription billing for streaming services
- Social features (Phase 2+)
- Content hosting or CDN

---

## 3. Container Architecture (C4 Model)

### 3.1 Container Diagram

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
│  │  │ TypeScript │  │ TypeScript │  │ TypeScript │  │ Commander  │       │ │
│  │  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘       │ │
│  └────────┼───────────────┼───────────────┼───────────────┼──────────────┘ │
│           │               │               │               │                 │
│           │ HTTPS/WSS     │ HTTPS/WSS     │ HTTPS/WSS     │ HTTPS           │
│           ▼               ▼               ▼               ▼                 │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         API GATEWAY LAYER                               │ │
│  │  ┌──────────────────────────────────────────────────────────────────┐  │ │
│  │  │ Cloud Load Balancer (Global)                                      │  │ │
│  │  │ - HTTPS/2 termination, TLS 1.3                                   │  │ │
│  │  │ - DDoS protection, Cloud Armor rules                             │  │ │
│  │  │ - Request routing, SSL offloading                                │  │ │
│  │  └──────────────────────────────────────────────────────────────────┘  │ │
│  │                              │                                          │ │
│  │                              ▼                                          │ │
│  │  ┌──────────────────────────────────────────────────────────────────┐  │ │
│  │  │ API Gateway Service (Fastify on Cloud Run)                        │  │ │
│  │  │ - Rate limiting: 100 req/min per user                            │  │ │
│  │  │ - Auth validation: JWT verification                              │  │ │
│  │  │ - Request routing: /api/*, /mcp/*, /admin/*                      │  │ │
│  │  │ - Response caching: 5s-5min TTL                                  │  │ │
│  │  │ - Observability: OpenTelemetry traces                            │  │ │
│  │  └──────────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│           │               │               │               │                 │
│           ▼               ▼               ▼               ▼                 │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                      APPLICATION LAYER (GKE Autopilot)                  │ │
│  │                                                                          │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │ │
│  │  │ MCP Server      │  │ Discovery       │  │ Auth Service    │         │ │
│  │  │ TypeScript      │  │ Service         │  │ Rust            │         │ │
│  │  │ MCP SDK         │  │ Rust            │  │ OAuth 2.0+PKCE  │         │ │
│  │  │ 2-10 replicas   │  │ 3-20 replicas   │  │ 2-10 replicas   │         │ │
│  │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘         │ │
│  │           │                    │                    │                   │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │ │
│  │  │ Recommendation  │  │ Sync Service    │  │ Ingestion       │         │ │
│  │  │ Service (SONA)  │  │ Rust            │  │ Service         │         │ │
│  │  │ Rust            │  │ PubNub SDK      │  │ Rust            │         │ │
│  │  │ 2-10 replicas   │  │ 2-5 replicas    │  │ 1-5 replicas    │         │ │
│  │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘         │ │
│  │           │                    │                    │                   │ │
│  │           ▼                    ▼                    ▼                   │ │
│  │  ┌──────────────────────────────────────────────────────────────────┐  │ │
│  │  │                   Internal gRPC Mesh                              │  │ │
│  │  │                   (Istio service mesh, mTLS)                      │  │ │
│  │  └──────────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│           │               │               │               │                 │
│           ▼               ▼               ▼               ▼                 │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         DATA LAYER                                      │ │
│  │                                                                          │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │ │
│  │  │ Cloud SQL       │  │ Qdrant Vector   │  │ Memorystore     │         │ │
│  │  │ (PostgreSQL)    │  │ Database        │  │ (Redis)         │         │ │
│  │  │ - Metadata      │  │ - 768-dim       │  │ - Sessions      │         │ │
│  │  │ - Users         │  │   embeddings    │  │ - Cache         │         │ │
│  │  │ - Watchlists    │  │ - HNSW index    │  │ - Rate limits   │         │ │
│  │  │ - Audit logs    │  │ - 20M vectors   │  │ - 30s-5min TTL  │         │ │
│  │  │ 1 primary +     │  │ 1 cluster       │  │ 1 instance      │         │ │
│  │  │ 3 read replicas │  │ 3 nodes         │  │ (HA mode)       │         │ │
│  │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘         │ │
│  │           │                    │                    │                   │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │ │
│  │  │ Ruvector Store  │  │ Cloud Storage   │  │ Pub/Sub         │         │ │
│  │  │ (SQLite)        │  │ (GCS)           │  │ (Event Bus)     │         │ │
│  │  │ - Graph DB      │  │ - Backups       │  │ - Async jobs    │         │ │
│  │  │ - Knowledge     │  │ - Static assets │  │ - Ingestion     │         │ │
│  │  │   graph         │  │ - Logs archive  │  │   triggers      │         │ │
│  │  │ - 100M nodes    │  │ Multi-region    │  │ - Audit events  │         │ │
│  │  │ Embedded        │  │ Nearline class  │  │ 1M msg/day      │         │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘         │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                      EXTERNAL INTEGRATIONS                              │ │
│  │                                                                          │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │ │
│  │  │ PubNub Network  │  │ Platform APIs   │  │ Monitoring      │         │ │
│  │  │ - Real-time     │  │ - YouTube       │  │ - Prometheus    │         │ │
│  │  │   sync          │  │ - Streaming     │  │ - Grafana       │         │ │
│  │  │ - <100ms        │  │   Availability  │  │ - Cloud Trace   │         │ │
│  │  │   latency       │  │ - JustWatch     │  │ - Error         │         │ │
│  │  │ - 1M messages   │  │ - Watchmode     │  │   Reporting     │         │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘         │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Communication Patterns

| Pattern | Use Case | Technology | Latency |
|---------|----------|------------|---------|
| **Synchronous Request-Response** | Client → API Gateway → Services | REST/HTTP2, gRPC | <50ms p95 |
| **Asynchronous Messaging** | Cross-device sync, real-time updates | PubNub | <100ms p95 |
| **Event-Driven** | Ingestion pipeline, audit logging | Cloud Pub/Sub | <500ms p95 |
| **Streaming** | Real-time recommendations | Server-Sent Events (SSE) | <200ms p95 |

---

## 4. Component Overview

### 4.1 Core Services

#### Discovery Service (Tier 1)

**Responsibility:** Natural language search, content lookup, availability filtering

**Technology:**
- Language: Rust 1.75+
- Framework: Actix-web
- Dependencies: Qdrant client, PostgreSQL client, Redis client

**Key Operations:**
```
POST /api/search
  ├─> Parse NL query (GPT-4o-mini)
  ├─> Generate query embedding (768-dim)
  ├─> Vector search (Qdrant HNSW)
  ├─> Availability filter (user subscriptions)
  ├─> Result ranking (SONA personalization)
  └─> Return JSON (entity IDs + metadata)

GET /api/content/{id}
  ├─> Lookup in Redis cache (5min TTL)
  ├─> Fallback to PostgreSQL
  └─> Return canonical content
```

**Scaling:** 3-20 replicas, CPU-based autoscaling (>70%)

**SLO:** 99.9% availability, <300ms p95 latency

---

#### Recommendation Service (SONA Intelligence) (Tier 1)

**Responsibility:** Personalized recommendations, contextual suggestions

**Technology:**
- Language: Rust
- ML Framework: ONNX Runtime (inference)
- Memory: Two-Tier LoRA adapters (in-memory)

**Key Operations:**
```
POST /api/recommendations
  ├─> Load user LoRA adapter (cached)
  ├─> Retrieve candidate pool (top 100 by popularity)
  ├─> Score with SONA model (sub-5ms)
  ├─> Apply context filters (time, device, mood)
  ├─> Rank and return top N
  └─> Async: Update LoRA with implicit feedback
```

**SONA Architecture:**
- **Base Model:** Frozen foundation model (100M params)
- **User LoRA:** Per-user adapter (256K params, ~1MB)
- **Global LoRA:** Shared trending patterns (1M params, ~4MB)
- **EWC++:** Continual learning, catastrophic forgetting prevention

**Scaling:** 2-10 replicas, memory-bound (4GB per replica)

**SLO:** 99.9% availability, <100ms p95 latency (including LoRA load)

---

#### Sync Service (Tier 1)

**Responsibility:** Real-time cross-device state synchronization

**Technology:**
- Language: Rust
- Protocol: CRDT (OR-Set for watchlists, LWW-Register for progress)
- Transport: PubNub (Rust SDK)

**Key Operations:**
```
WebSocket /sync
  ├─> Authenticate (JWT)
  ├─> Subscribe to user channel: user.{userId}.sync
  ├─> Receive CRDT operations
  ├─> Merge with local state
  ├─> Broadcast to other devices (PubNub)
  └─> Async persist to PostgreSQL

POST /api/sync/watchlist
  ├─> Validate CRDT operation
  ├─> Optimistic local update
  ├─> Publish to PubNub (<100ms)
  └─> Return 202 Accepted
```

**Conflict Resolution:** Add-wins bias for watchlist items

**Scaling:** 2-5 replicas, WebSocket connection pooling

**SLO:** 99.5% availability, <100ms sync latency

---

#### Auth Service (Tier 1)

**Responsibility:** User authentication, OAuth 2.0 flows, token management

**Technology:**
- Language: Rust
- Framework: Actix-web
- Auth: OAuth 2.0 + PKCE, Device Authorization Grant (RFC 8628)
- Tokens: JWT (RS256), 15min access, 7d refresh

**Key Operations:**
```
POST /auth/login
  ├─> OAuth 2.0 PKCE flow (Google, GitHub)
  ├─> Create session (Redis, 7d TTL)
  ├─> Issue JWT access token (15min)
  ├─> Issue refresh token (7d, httpOnly cookie)
  └─> Return user profile

POST /auth/device
  ├─> Generate device code (8 chars)
  ├─> Store in Redis (15min TTL)
  ├─> Return code + verification URL
  └─> Poll endpoint: GET /auth/device/poll

POST /auth/refresh
  ├─> Validate refresh token
  ├─> Rotate tokens
  └─> Return new access token
```

**Security:**
- Zero credential storage (OAuth only)
- PKCE for web/mobile (prevents authorization code interception)
- Device grant for TV/CLI (user-friendly pairing)
- Refresh token rotation (mitigates token theft)

**Scaling:** 2-10 replicas, session state in Redis (shared)

**SLO:** 99.9% availability, <50ms p95 latency

---

#### Ingestion Service (Tier 2)

**Responsibility:** Platform data ingestion, normalization, entity resolution

**Technology:**
- Language: Rust
- Scheduler: Kubernetes CronJob (hourly)
- Queue: Cloud Pub/Sub (for async processing)

**Key Operations:**
```
CRON: Every 1 hour
  ├─> Fetch from platform APIs (parallel)
  │   ├─> YouTube Data API (10K/day quota)
  │   ├─> Streaming Availability API (100/min)
  │   └─> JustWatch API (1000/hour)
  ├─> Normalize to CanonicalContent
  ├─> Entity resolution (deduplication)
  ├─> Generate embeddings (batch, 500/s)
  ├─> Upsert to PostgreSQL + Qdrant
  └─> Publish events to Pub/Sub (for downstream)

Fallback: Manual trigger via Admin Dashboard
```

**Rate Limiting:**
- Multi-key rotation (5 YouTube keys)
- Exponential backoff on 429 errors
- Circuit breaker pattern (3 failures → open 60s)

**Scaling:** 1-5 replicas (CronJob concurrency limit)

**SLO:** 99.5% availability, <1h data freshness

---

#### MCP Server (Tier 1)

**Responsibility:** Model Context Protocol for AI agent integration

**Technology:**
- Language: TypeScript
- Framework: MCP SDK (@anthropic-ai/mcp)
- Transport: STDIO (Claude Desktop), SSE (web)

**MCP Tools Exposed:**
```typescript
[
  "semantic_search",           // Natural language content search
  "get_recommendations",       // Personalized recommendations
  "get_content_details",       // Full content metadata
  "list_user_watchlist",       // User's saved content
  "add_to_watchlist",          // Add content (OAuth required)
  "remove_from_watchlist",     // Remove content (OAuth required)
  "check_availability",        // Platform availability by region
  "get_streaming_links",       // Deep links to platforms
  "list_user_devices",         // Registered devices
  "send_to_device"             // Cross-device handoff (OAuth required)
]
```

**ARW Manifest:** `/.well-known/arw-manifest.json`
- Version: 0.1
- Profile: ARW-1
- 10 declared actions
- OAuth-protected write operations

**Token Efficiency:** 85% reduction vs HTML scraping (850 tokens vs 5,600 tokens)

**Scaling:** 2-10 replicas, stateless

**SLO:** 99.9% availability, <50ms MCP overhead (excludes downstream calls)

---

### 4.2 Supporting Services

#### Admin Dashboard (Tier 2)

- **Technology:** Next.js, React, Tailwind CSS
- **Deployment:** Cloud Run (scale-to-zero)
- **Features:** User management, ingestion triggers, metrics dashboards
- **Auth:** Admin-only JWT scopes

#### CLI Tool (Tier 2)

- **Technology:** TypeScript, Commander.js
- **Deployment:** npm package (`npx media-gateway`)
- **Features:** Search, recommendations, watchlist management, device pairing
- **Auth:** Device Authorization Grant flow

---

## 5. Technology Stack

### 5.1 Programming Languages

| Language | Use Cases | Rationale |
|----------|-----------|-----------|
| **Rust** | Discovery, Recommendation, Sync, Auth, Ingestion | Performance-critical paths, memory safety, concurrency |
| **TypeScript** | MCP Server, API Gateway, Web App, CLI | Ecosystem maturity, rapid development, MCP SDK availability |
| **Python** | ML training pipelines (offline) | Scikit-learn, PyTorch for SONA model training |

**Rust Coverage:** 80% of backend services (all hot paths)

---

### 5.2 Frameworks and Libraries

#### Backend

| Framework | Service | Version |
|-----------|---------|---------|
| Actix-web | Discovery, Recommendation, Auth | 4.x |
| Fastify | API Gateway | 4.x |
| MCP SDK | MCP Server | Latest |
| Tokio | Async runtime (all Rust services) | 1.35+ |

#### Frontend

| Framework | Application | Version |
|-----------|-------------|---------|
| Next.js | Web App, Admin Dashboard | 14.x |
| React Native | Mobile Apps (iOS/Android) | 0.73+ |
| React TV | Smart TV Apps | Latest |

---

### 5.3 Databases and Storage

| Database | Purpose | Technology | Persistence |
|----------|---------|------------|-------------|
| **Canonical Metadata** | Content, users, watchlists, audit logs | Cloud SQL (PostgreSQL 15) | Durable (multi-zone, automated backups) |
| **Vector Search** | 768-dim embeddings, HNSW index | Qdrant (self-hosted on GKE) | Durable (persistent volumes) |
| **Cache** | Sessions, rate limits, hot data | Memorystore (Redis 7.x) | Ephemeral (HA mode, no persistence) |
| **Knowledge Graph** | Entity relationships, GNN embeddings | Ruvector (SQLite-based, embedded) | Durable (per-service local) |
| **Object Storage** | Backups, logs, static assets | Cloud Storage (Nearline class) | Durable (multi-region) |

**Data Residency:** All user data in `us-central1` (Iowa) for GDPR/CCPA compliance

---

### 5.4 Message Queue and Real-time

| System | Purpose | Protocol | Latency |
|--------|---------|----------|---------|
| **PubNub** | Cross-device sync, real-time updates | WebSocket, MQTT | <100ms p95 |
| **Cloud Pub/Sub** | Async job queuing, audit events | gRPC | <500ms p95 |

**PubNub Quota:** 1M messages/day (Free tier → $49/mo at 10M)

---

### 5.5 Infrastructure (GCP)

| Service | Purpose | Configuration |
|---------|---------|---------------|
| **GKE Autopilot** | Container orchestration | Multi-zone `us-central1`, 3-50 nodes autoscaling |
| **Cloud Run** | Serverless containers | API Gateway, Admin Dashboard (scale-to-zero) |
| **Cloud Load Balancer** | Global HTTP(S) LB | Multi-region, SSL termination, Cloud Armor |
| **Cloud SQL** | Managed PostgreSQL | db-n1-standard-2, 1 primary + 3 read replicas |
| **Memorystore** | Managed Redis | 5GB HA instance |
| **Cloud Storage** | Object storage | Nearline class (backups), Standard (static assets) |
| **Cloud Monitoring** | Observability | Metrics, logs, traces (OpenTelemetry) |

**Cost Optimization:**
- Preemptible nodes for non-Tier 1 workloads (60-91% discount)
- Committed use discounts (37% on compute)
- Cloud Run scale-to-zero for low-traffic services

---

## 6. Key Architectural Decisions

### 6.1 Microservices vs Monolith

**Decision:** Microservices architecture

**Rationale:**
1. **Independent Scaling:** Recommendation service (memory-bound) vs Discovery service (CPU-bound) have different scaling profiles
2. **Technology Flexibility:** Rust for performance, TypeScript for rapid development
3. **Team Autonomy:** 51 platform normalizers can evolve independently
4. **Fault Isolation:** MCP server failure doesn't impact core search

**Trade-offs:**
- ✅ Horizontal scalability, polyglot support, independent deployments
- ❌ Operational complexity (service mesh, distributed tracing required)
- Mitigation: GKE Autopilot (managed control plane), Istio (service mesh)

---

### 6.2 Database Strategy (Polyglot Persistence)

**Decision:** Multiple specialized databases instead of single SQL database

**Rationale:**

| Database | Optimized For | Alternative Considered |
|----------|---------------|------------------------|
| PostgreSQL | ACID transactions, relational data | Single SQL (not optimized for vectors) |
| Qdrant | Vector similarity search (HNSW) | PostgreSQL pgvector (10x slower) |
| Redis | Sub-millisecond cache, sessions | Memcached (less feature-rich) |
| Ruvector | Graph traversal, GNN embeddings | Neo4j (higher cost, operational overhead) |

**Trade-offs:**
- ✅ Each workload gets optimal database (10x performance improvement)
- ❌ No cross-database transactions, eventual consistency
- Mitigation: Saga pattern for distributed transactions, CRDT for sync

---

### 6.3 Cloud Provider (GCP)

**Decision:** Google Cloud Platform (GCP)

**Rationale:**
1. **Vertex AI Integration:** SONA model training/deployment
2. **GKE Autopilot:** Zero node management, auto-scaling, cost-efficient
3. **BigQuery:** Future analytics on user behavior (GDPR-compliant)
4. **Committed Use Discounts:** 37% savings on 1-year commit
5. **Proximity to PubNub PoPs:** <50ms latency to real-time infrastructure

**Alternatives Considered:**
- AWS: More mature (EKS), but higher cost for equivalent services
- Azure: Less strong AI/ML ecosystem vs Vertex AI

**Multi-Cloud Readiness:**
- Infrastructure as Code (Terraform)
- Kubernetes-native (portable across clouds)
- No GCP-exclusive APIs in application layer

---

### 6.4 Programming Language (Rust for Backend)

**Decision:** Rust for all performance-critical services

**Rationale:**
1. **Performance:** 2-5x faster than Node.js for CPU-bound tasks
2. **Memory Safety:** No garbage collection pauses, predictable latency
3. **Concurrency:** Tokio async runtime handles 10K+ concurrent connections per instance
4. **Ecosystem:** ONNX Runtime (Rust bindings), gRPC, PostgreSQL drivers mature

**Benchmarks:**
- Rust Actix-web: 15K RPS @ p95 <50ms
- Node.js Fastify: 6K RPS @ p95 <50ms (same hardware)

**Trade-offs:**
- ✅ Performance, memory efficiency, zero-cost abstractions
- ❌ Steeper learning curve, slower initial development
- Mitigation: TypeScript for non-critical paths (MCP server, Admin Dashboard)

---

### 6.5 Real-time Sync (PubNub)

**Decision:** PubNub for cross-device synchronization

**Rationale:**
1. **Latency:** <100ms global message delivery (vs WebSocket self-hosted ~200ms)
2. **Scale:** Managed infrastructure, 1M concurrent connections
3. **SDKs:** Native support for all platforms (Web, iOS, Android, Rust)
4. **Presence:** Online/offline detection built-in
5. **Cost:** Free tier covers MVP, $49/mo at 10M messages (vs $500/mo self-hosted)

**Alternatives Considered:**
- Self-hosted WebSocket (Socket.io): Lower cost at scale, but operational burden
- Firebase Realtime DB: Vendor lock-in, less control over data residency

**Data Residency:** PubNub data centers in US for GDPR compliance

---

### 6.6 Vector Database (Qdrant)

**Decision:** Self-hosted Qdrant on GKE

**Rationale:**
1. **Performance:** HNSW index 150x faster than linear scan
2. **Scale:** 20M vectors @ 768-dim in <60GB memory
3. **Cost:** $0/month (self-hosted) vs $500/month (Pinecone equivalent)
4. **Control:** Full control over index parameters, data residency

**Deployment:**
- 3-node cluster on GKE (n1-highmem-4: 4 vCPU, 26GB RAM)
- Persistent SSD volumes (100GB per node)
- Horizontal scaling via sharding

**Alternatives Considered:**
- Pinecone: Managed, but $500/mo for 20M vectors
- Weaviate: Similar to Qdrant, less mature Rust client

---

## 7. Cross-Cutting Concerns

### 7.1 Observability Strategy

#### Metrics (Prometheus + Grafana)

**Service-Level Metrics:**
```
# Request latency histogram
http_request_duration_seconds{service="discovery",endpoint="/api/search"}

# Request rate counter
http_requests_total{service="discovery",status="200"}

# Active connections gauge
active_connections{service="sync",protocol="websocket"}

# Resource utilization
container_memory_usage_bytes{service="recommendation",pod="sona-abc123"}
```

**SLO Dashboards:**
- **User-Facing:** Search latency (p50/p95/p99), recommendation CTR, sync latency
- **System Health:** Pod crash rate, error rate, saturation (CPU/memory)
- **Business KPIs:** DAU, search volume, platform coverage

**Alerting:**
- PagerDuty integration for Tier 1 services
- Slack notifications for Tier 2 services
- Alert on SLO violations (error budget burn rate)

---

#### Logging (Cloud Logging)

**Log Levels:**
- `ERROR`: Service failures, unrecoverable errors → PagerDuty
- `WARN`: Degraded performance, fallback activations → Slack
- `INFO`: Request/response, state transitions → Cloud Logging (retained 30d)
- `DEBUG`: Detailed diagnostics → Local only (not in production)

**Structured Logging (JSON):**
```json
{
  "timestamp": "2025-12-06T21:30:00.000Z",
  "level": "INFO",
  "service": "discovery",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "span_id": "00f067aa0ba902b7",
  "message": "Search request processed",
  "user_id": "user-abc123",
  "query": "scary movies like Stranger Things",
  "results_count": 25,
  "latency_ms": 287
}
```

---

#### Tracing (Cloud Trace + OpenTelemetry)

**Distributed Traces:**
```
Search Request Trace
├─ API Gateway: 12ms
├─ Discovery Service: 275ms
│  ├─ NL parsing (GPT-4o-mini): 85ms
│  ├─ Embedding generation: 25ms
│  ├─ Qdrant vector search: 45ms
│  ├─ PostgreSQL availability filter: 30ms
│  └─ SONA ranking: 90ms
└─ Response serialization: 8ms

Total: 295ms (within 500ms SLO)
```

**Sampling:**
- 100% of errors
- 10% of successful requests (head-based sampling)
- 100% of requests >1s latency (tail-based sampling)

---

### 7.2 Security Approach

#### Zero-Trust Architecture

**Principles:**
1. **No Implicit Trust:** All service-to-service calls require mTLS
2. **Least Privilege:** Services have minimal IAM permissions
3. **Defense in Depth:** Multiple security layers (LB → API Gateway → Service Mesh)

**Implementation:**
- Istio service mesh for mTLS between pods
- Workload Identity for GKE → GCP API auth
- Cloud Armor for DDoS protection at LB layer

---

#### Data Protection

| Data Type | At Rest | In Transit | Retention |
|-----------|---------|------------|-----------|
| **User credentials** | NEVER STORED | OAuth only | N/A |
| **Session tokens** | Redis (encrypted) | TLS 1.3 | 7 days |
| **Watchlist data** | PostgreSQL AES-256 | TLS 1.3 | User lifetime |
| **Search queries** | Hashed (SHA-256) | TLS 1.3 | 30 days (analytics only) |
| **Audit logs** | PostgreSQL AES-256 | TLS 1.3 | 2 years (compliance) |

**Differential Privacy:**
- ε=1.0, δ=1e-5 guarantee on aggregate analytics
- Laplace noise added to query counts before export

---

#### Authentication Flows

**Web/Mobile (PKCE):**
```
1. Client generates code_verifier (random 128-bit)
2. Client sends code_challenge = SHA256(code_verifier)
3. Auth server returns authorization code
4. Client exchanges code + code_verifier for tokens
5. Auth server validates code_verifier against code_challenge
```

**TV/CLI (Device Grant):**
```
1. Device requests device code
2. User visits verification URL on phone
3. User enters code, approves device
4. Device polls /auth/device/poll (every 5s)
5. On approval, device receives access + refresh tokens
```

---

### 7.3 Scalability Patterns

#### Horizontal Pod Autoscaling (HPA)

**Discovery Service:**
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: discovery-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: discovery
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: "1000"
```

**Scaling Triggers:**
- CPU >70% → add pod
- Requests >1000 RPS per pod → add pod
- Cool-down: 5 minutes (prevent thrashing)

---

#### Caching Strategy (Multi-Tier)

**Layer 1: Client-Side (Browser/App)**
- Static assets: 1 year (immutable)
- API responses: 5 seconds (search results)

**Layer 2: API Gateway (Cloud Run)**
- Hot queries: 30 seconds TTL
- Content details: 5 minutes TTL

**Layer 3: Redis (Memorystore)**
- User sessions: 7 days TTL
- Content metadata: 5 minutes TTL
- Rate limit counters: 1 minute TTL

**Layer 4: Database Query Cache**
- PostgreSQL: Enabled (automatic)

**Cache Invalidation:**
- Event-driven (Pub/Sub) on content updates
- Lazy invalidation (check `last_updated` timestamp)

---

#### Database Scaling

**Read Replicas (PostgreSQL):**
- 1 primary (writes only)
- 3 read replicas (reads only)
- Connection pooling (PgBouncer): min=10, max=100 per service

**Sharding Strategy (Future):**
- Hash-based sharding on `user_id` for user-specific tables
- Geographic sharding for multi-region (Phase 2)

---

### 7.4 Reliability Patterns

#### Circuit Breaker (External APIs)

**State Machine:**
```
CLOSED (normal)
  ├─> 3 consecutive failures → OPEN (fail fast)
  │   ├─> Wait 60 seconds
  │   └─> Transition to HALF_OPEN
  └─> HALF_OPEN
      ├─> 1 success → CLOSED
      └─> 1 failure → OPEN (back to 60s wait)
```

**Example (YouTube API):**
```rust
let circuit_breaker = CircuitBreaker::new(
    failure_threshold: 3,
    timeout: Duration::from_secs(60),
);

match circuit_breaker.call(|| youtube_api.search(query)) {
    Ok(results) => Ok(results),
    Err(CircuitBreakerError::Open) => {
        // Fallback to cached data or alternative API
        fallback_search(query)
    }
}
```

---

#### Graceful Degradation

**4-Level Strategy:**

| Level | Condition | Response |
|-------|-----------|----------|
| **L0: Full Service** | All systems healthy | All features enabled |
| **L1: Reduced Quality** | High load, some replicas down | Return cached results (5min old) |
| **L2: Core Only** | Database read replicas down | Search works, no recommendations |
| **L3: Read-Only** | Primary database down | Cached reads only, no writes |
| **L4: Emergency** | Complete outage | Static error page with status link |

**Automatic Detection:**
- Health checks every 10s
- Degrade automatically when SLO violated
- Restore when health checks pass for 2 minutes

---

#### Retry Logic

**Exponential Backoff:**
```rust
let retry_policy = RetryPolicy::exponential(
    initial_delay: Duration::from_millis(100),
    max_delay: Duration::from_secs(10),
    max_attempts: 5,
);

retry_policy.retry(|| {
    external_api.call()
})
```

**Idempotency:**
- All write operations have idempotency keys
- Deduplication window: 24 hours (Redis-backed)

---

## Summary

This high-level architecture establishes:

1. **Container-based microservices** on GKE Autopilot with polyglot persistence (PostgreSQL, Qdrant, Redis, Ruvector)
2. **Rust-first backend** for performance-critical paths (80% coverage), TypeScript for rapid development
3. **Multi-tier caching** and horizontal scaling to support 100K→1M concurrent users
4. **Zero-trust security** with OAuth 2.0 + PKCE, no credential storage, differential privacy
5. **Comprehensive observability** (metrics, logs, traces) with SLO-based alerting
6. **Resilience patterns** (circuit breakers, graceful degradation, retry logic) for 99.9% availability

**Next Phase:** Architecture Part 2 will detail service-level designs, API specifications, and deployment architecture.

---

**Document Status:** Complete
**Review Required:** Architecture team, Security team
**Next Document:** [SPARC_ARCHITECTURE_PART_2.md](./SPARC_ARCHITECTURE_PART_2.md) - Service Design and API Specifications

---

END OF PART 1
