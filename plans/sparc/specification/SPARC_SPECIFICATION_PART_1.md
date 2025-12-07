# SPARC Specification — Part 1 of 4

## Media Gateway: Unified Cross-Platform TV Discovery Engine

**Document Version:** 1.0.0
**SPARC Phase:** Specification
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents — Part 1

1. [Executive Summary](#1-executive-summary)
2. [Problem Space Definition](#2-problem-space-definition)
3. [System Goals and Objectives](#3-system-goals-and-objectives)
4. [System Boundaries](#4-system-boundaries)
5. [Stakeholder Analysis](#5-stakeholder-analysis)
6. [User Flows](#6-user-flows)

---

## 1. Executive Summary

### 1.1 Vision Statement

The Media Gateway is a production-ready, AI-native entertainment discovery platform that evolves the hackathon-tv5 prototype into a unified cross-platform TV discovery engine. The system solves the "45-minute decision problem"—billions of hours lost globally every day as users struggle to decide what to watch across fragmented streaming platforms.

### 1.2 Core Value Proposition

| Metric | Current State | Media Gateway Target |
|--------|---------------|---------------------|
| Decision Time | 45+ minutes/night | <5 minutes |
| Platform Coverage | Single platform | 150+ platforms, 60+ countries |
| Personalization | Basic recommendations | SONA-powered adaptive learning |
| AI Agent Token Efficiency | 100% (HTML scraping) | 15% (ARW protocol, 85% reduction) |
| Cross-Device Sync | None | <100ms real-time sync |

### 1.3 System Scope

The Media Gateway is a **content discovery layer**, NOT a streaming infrastructure. The system:

**DOES:**
- Aggregate metadata from 150+ streaming platforms
- Provide AI-powered natural language search
- Deliver personalized recommendations via SONA intelligence
- Synchronize user state across devices in real-time
- Expose functionality via MCP protocol for AI agent integration
- Deep-link users to native platform apps for content playback

**DOES NOT:**
- Host, stream, or transcode video/audio content
- Store user streaming credentials
- Bypass platform DRM or access controls
- Provide multistreaming or RTMP output
- Cache or proxy copyrighted content

### 1.4 Technology Foundation

The system builds upon two authoritative source repositories:

1. **hackathon-tv5** (`github.com/agenticsorg/hackathon-tv5`)
   - TypeScript CLI toolkit with MCP server
   - ARW (Agent-Ready Web) protocol specification
   - Media discovery reference application
   - AgentDB cognitive memory system
   - Agentic Flow multi-agent orchestration

2. **media-gateway-research** (`github.com/globalbusinessadvisors/media-gateway-research`)
   - Production architecture blueprints
   - SONA intelligence engine specification
   - Ruvector knowledge graph design
   - GCP deployment architecture
   - Security and compliance frameworks

---

## 2. Problem Space Definition

### 2.1 The Fragmentation Crisis

Modern entertainment consumption is characterized by severe fragmentation:

```
┌─────────────────────────────────────────────────────────────┐
│                    FRAGMENTATION MATRIX                      │
├─────────────────────────────────────────────────────────────┤
│  Platforms: 150+     │  Content: 500K+ titles               │
│  Countries: 190+     │  User Accounts: 3-7 per household    │
│  Price Points: Vary  │  Catalog Overlap: <15%               │
│  Exclusives: 40%+    │  Churn Rate: 40%/year                │
└─────────────────────────────────────────────────────────────┘
```

**Quantified User Pain:**
- Average time to decide what to watch: **45 minutes/night**
- Global hours lost daily: **Billions**
- User frustration with fragmentation: **78%** (industry surveys)
- Households with 4+ streaming subscriptions: **45%**

### 2.2 Root Cause Analysis

```
┌──────────────────────────────────────────────────────────────────┐
│                      ROOT CAUSE TREE                              │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  [Decision Fatigue] ─┬─> [Too Many Platforms]                    │
│                      ├─> [Inconsistent UIs]                      │
│                      ├─> [Siloed Recommendations]                │
│                      └─> [No Cross-Platform Memory]              │
│                                                                   │
│  [Discovery Failure] ─┬─> [Search Limited to Platform]           │
│                       ├─> [Recommendations Echo Chambers]        │
│                       ├─> [No Mood/Context Awareness]            │
│                       └─> [Missing Cross-Platform Context]       │
│                                                                   │
│  [Subscription Waste] ─┬─> [Duplicate Subscriptions]             │
│                        ├─> [Unknown Availability]                │
│                        └─> [Price Comparison Impossible]         │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

### 2.3 Critical Platform Constraint

**CRITICAL FINDING:** 80% of major streaming platforms offer NO public APIs.

| Platform | API Status | Integration Strategy |
|----------|------------|---------------------|
| YouTube | ✅ Full Public API | Direct OAuth 2.0 integration |
| Netflix | ❌ No Public API | Third-party aggregators + deep linking |
| Prime Video | ❌ No Public API | Third-party aggregators + deep linking |
| Disney+ | ❌ No Public API | Third-party aggregators + deep linking |
| Hulu | ❌ No Public API | Third-party aggregators + deep linking |
| HBO Max | ❌ No Public API | Third-party aggregators + deep linking |
| Apple TV+ | ❌ No Public API | Third-party aggregators + deep linking |
| Peacock | ❌ No Public API | Third-party aggregators + deep linking |
| Paramount+ | ❌ No Public API | Third-party aggregators + deep linking |

**Strategic Response:**
1. Primary: Third-party aggregator APIs (Streaming Availability, Watchmode, JustWatch)
2. Fallback: ARW manifest discovery for compliant platforms
3. Delivery: Deep linking to native apps (no credential storage)

### 2.4 AI Agent Integration Gap

Current web infrastructure forces AI agents to:
- Scrape HTML pages (token-inefficient)
- Parse unstructured content (error-prone)
- Lack action capabilities (read-only)
- Miss real-time updates (stale data)

**ARW Protocol Solution:**
```
Traditional HTML Scraping    →    ARW Manifest Discovery
─────────────────────────         ─────────────────────
100% token usage                  15% token usage (85% reduction)
Error-prone parsing               Structured JSON
No actions                        Declared capabilities
Slow discovery                    10x faster discovery
```

---

## 3. System Goals and Objectives

### 3.1 Primary Goals

#### G1: Unified Content Discovery
**Objective:** Enable users to search across all their streaming platforms with a single natural language query.

**Success Criteria:**
- Support 150+ streaming platforms across 60+ countries
- Natural language query parsing with 95%+ intent accuracy
- Results returned in <500ms (p95)
- Cross-platform deduplication with 99%+ accuracy

#### G2: Intelligent Personalization
**Objective:** Deliver recommendations that adapt in real-time to user preferences and context.

**Success Criteria:**
- SONA personalization latency <5ms
- Recommendation relevance: Precision@10 ≥ 0.31, NDCG@10 ≥ 0.63
- Cold-start problem solved within 3 interactions
- Context-aware (time of day, device, viewing history)

#### G3: Seamless Cross-Device Experience
**Objective:** Synchronize user state across all devices with imperceptible latency.

**Success Criteria:**
- State sync latency <100ms (PubNub)
- Support for web, mobile (iOS/Android), TV, CLI
- Offline-first with conflict resolution (CRDTs)
- Graceful handoff between devices

#### G4: AI Agent Interoperability
**Objective:** Enable AI agents to discover, understand, and act on media gateway capabilities.

**Success Criteria:**
- MCP server with 10+ tools exposed
- ARW manifest compliance
- 85% token reduction vs. HTML scraping
- OAuth-protected actions with user consent

#### G5: Privacy-First Architecture
**Objective:** Protect user data while enabling personalization.

**Success Criteria:**
- On-device preference storage (CRDT-synced)
- Differential privacy: (ε=1.0, δ=1e-5)-DP guarantee
- Federated learning (no raw data leaves device)
- GDPR/CCPA/VPPA compliance

### 3.2 Secondary Goals

#### G6: Operational Excellence
- 99.9% availability SLO for Tier 1 services
- <$4,000/month infrastructure cost
- 100K concurrent users at launch

#### G7: Developer Experience
- Comprehensive MCP tool documentation
- TypeScript/Rust SDK availability
- Interactive CLI for all operations
- <5 minute onboarding for developers

#### G8: Extensibility
- Plugin architecture for new platforms
- Custom recommendation models
- White-label capabilities
- Multi-tenant isolation

---

## 4. System Boundaries

### 4.1 System Context Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         EXTERNAL SYSTEMS                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐│
│  │   YouTube    │  │ Streaming    │  │  JustWatch   │  │  Watchmode   ││
│  │     API      │  │ Availability │  │     API      │  │     API      ││
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘│
│         │                 │                 │                 │         │
│         ▼                 ▼                 ▼                 ▼         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                                                                   │  │
│  │                      MEDIA GATEWAY SYSTEM                         │  │
│  │                                                                   │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │  │
│  │  │ MCP Server  │  │ Discovery   │  │   SONA      │               │  │
│  │  │             │  │  Engine     │  │ Intelligence│               │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘               │  │
│  │                                                                   │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │  │
│  │  │  Ruvector   │  │   PubNub    │  │    Auth     │               │  │
│  │  │  Storage    │  │    Sync     │  │   Service   │               │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘               │  │
│  │                                                                   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│         │                 │                 │                 │         │
│         ▼                 ▼                 ▼                 ▼         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐│
│  │   Web App    │  │ Mobile Apps  │  │   TV Apps    │  │  AI Agents   ││
│  │  (Next.js)   │  │ (iOS/Android)│  │ (Smart TVs)  │  │ (Claude,etc) ││
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘│
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4.2 In-Scope Components

| Component | Responsibility | Technology |
|-----------|----------------|------------|
| MCP Server | Protocol bridge for AI agents | TypeScript, MCP SDK |
| Discovery Engine | Content search and aggregation | Rust, SONA |
| Recommendation Engine | Personalized suggestions | Rust, ML models |
| Ruvector Storage | Vector search and knowledge graph | Rust, SQLite/PostgreSQL |
| PubNub Sync | Real-time cross-device sync | PubNub SDK |
| Auth Service | OAuth 2.0 + PKCE, platform tokens | Rust, JWT |
| CLI Tool | Developer and power-user interface | TypeScript, Commander.js |
| API Gateway | REST/GraphQL interface | Cloud Run, Express |
| Admin Dashboard | Operations and monitoring | Next.js, React |

### 4.3 Out-of-Scope Components

| Component | Reason | Alternative |
|-----------|--------|-------------|
| Video Streaming | Not a streaming infrastructure | Deep link to native apps |
| Content Hosting | Legal and licensing issues | Metadata only |
| DRM Handling | Platform responsibility | Pass-through to platforms |
| Subscription Management | Platform-specific | Link to platform account pages |
| Ad Serving | Outside core mission | Partner integrations if needed |
| Social Features | Phase 2 consideration | Watch parties via PubNub later |

### 4.4 Integration Boundaries

#### External API Dependencies

| Dependency | Purpose | Rate Limit | Fallback |
|------------|---------|------------|----------|
| Streaming Availability API | Platform catalog data | 100/min | Cache + Watchmode |
| Watchmode API | Secondary catalog source | 1000/day | Cache + JustWatch |
| YouTube Data API | Direct platform integration | 10,000/day | Multi-key rotation |
| TMDb API | Community metadata | 40/10s | Cache (7-day) |
| Gracenote/TMS | Premium metadata | Varies | TMDb fallback |
| PubNub | Real-time messaging | 1M msg/day | Queue + retry |

#### Internal Service Boundaries

```
┌─────────────────────────────────────────────────────────────────┐
│                    SERVICE BOUNDARY MAP                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Layer 1: Ingestion (Data In)                                   │
│  ├── Platform Normalizers (51 micro-repos)                      │
│  ├── Entity Resolver (deduplication)                            │
│  └── Rate Limit Manager                                         │
│                                                                  │
│  Layer 2: Intelligence (Processing)                             │
│  ├── SONA Engine (personalization)                              │
│  ├── Ruvector (vector search + GNN)                            │
│  └── Recommendation Pipeline                                    │
│                                                                  │
│  Layer 3: Consolidation (State)                                 │
│  ├── PostgreSQL (canonical data)                                │
│  ├── Valkey/Redis (cache + sessions)                           │
│  └── PubNub (real-time sync)                                   │
│                                                                  │
│  Layer 4: Applications (Interfaces)                             │
│  ├── MCP Server (AI agents)                                     │
│  ├── REST API (web/mobile)                                      │
│  ├── CLI (developers)                                           │
│  └── Admin Dashboard (operators)                                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 5. Stakeholder Analysis

### 5.1 Primary Stakeholders

#### End Users (Consumers)

**Persona: "Streaming Sarah"**
- Demographics: 25-45, tech-savvy, 3+ streaming subscriptions
- Pain Points: Decision fatigue, missing content, forgotten watchlists
- Goals: Quick discovery, personalized suggestions, seamless handoff
- Devices: Phone (primary), Smart TV (viewing), Laptop (browsing)

**Needs:**
1. Single search across all platforms
2. "What should I watch tonight?" natural language
3. Cross-device watchlist sync
4. Price/availability transparency
5. Privacy-respecting personalization

#### AI Agent Developers

**Persona: "Developer Dave"**
- Role: Building AI-powered entertainment assistants
- Pain Points: Token-heavy HTML scraping, no action capabilities
- Goals: Efficient discovery, structured data, actionable tools
- Tools: Claude, GPT-4, custom agents

**Needs:**
1. MCP server with comprehensive tools
2. ARW manifest for capability discovery
3. OAuth-protected actions
4. Low-latency, high-accuracy responses
5. SDK and documentation

#### Platform Operators

**Persona: "Operations Olivia"**
- Role: Running and maintaining the Media Gateway
- Pain Points: Scaling, reliability, cost management
- Goals: 99.9% uptime, cost efficiency, security
- Tools: GCP Console, Grafana, PagerDuty

**Needs:**
1. Comprehensive observability
2. Auto-scaling with cost controls
3. Security monitoring and alerting
4. Runbook automation
5. Disaster recovery procedures

### 5.2 Secondary Stakeholders

| Stakeholder | Interest | Engagement |
|-------------|----------|------------|
| Streaming Platforms | Traffic referral, user engagement | Partnership discussions |
| Aggregator APIs | API usage, data quality | Vendor management |
| Advertisers | User attention (future) | Phase 2 consideration |
| Regulators | Privacy compliance | GDPR/CCPA/VPPA adherence |
| Investors | Growth metrics, unit economics | Dashboard access |

### 5.3 Stakeholder Requirements Matrix

| Requirement | End User | AI Developer | Operator |
|-------------|----------|--------------|----------|
| Search Latency <500ms | Critical | Critical | High |
| 99.9% Availability | High | Critical | Critical |
| Privacy Compliance | Critical | Medium | Critical |
| MCP Tools | N/A | Critical | Low |
| Cost <$4K/month | N/A | N/A | Critical |
| Cross-Device Sync | Critical | Low | Medium |
| Documentation | Low | Critical | High |

---

## 6. User Flows

### 6.1 Primary User Flow: Natural Language Search

```
┌─────────────────────────────────────────────────────────────────────────┐
│              USER FLOW: NATURAL LANGUAGE SEARCH                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌───────┐ │
│  │ User │───▶│   App    │───▶│   API    │───▶│  SONA    │───▶│Results│ │
│  └──────┘    └──────────┘    └──────────┘    └──────────┘    └───────┘ │
│                                                                          │
│  Step 1: User enters natural language query                             │
│  ────────────────────────────────────────                               │
│  Input: "Something like Stranger Things but more scary"                 │
│  Device: Mobile app (iPhone 15)                                         │
│  Context: 9:30 PM, Saturday, user preferences loaded                    │
│                                                                          │
│  Step 2: Intent parsing (GPT-4o-mini)                                   │
│  ───────────────────────────────────                                    │
│  Parsed Intent:                                                          │
│  {                                                                       │
│    "mood": "scary",                                                      │
│    "reference": "Stranger Things",                                       │
│    "themes": ["horror", "supernatural", "thriller"],                    │
│    "intensity": "higher than reference"                                 │
│  }                                                                       │
│                                                                          │
│  Step 3: Multi-strategy search                                          │
│  ─────────────────────────────                                          │
│  a) Vector similarity: Embed "Stranger Things" → find neighbors        │
│  b) Genre filter: Horror + Supernatural                                 │
│  c) SONA personalization: Weight by user history                        │
│  d) Availability filter: User's subscribed platforms                    │
│                                                                          │
│  Step 4: Result ranking                                                  │
│  ─────────────────────                                                  │
│  score = base_match × theme_boost × preference_weight × popularity      │
│                                                                          │
│  Step 5: Response delivery (<500ms target)                              │
│  ──────────────────────────────────────                                 │
│  Results:                                                                │
│  1. "The Haunting of Hill House" (Netflix) - Match: 94%                │
│  2. "Midnight Mass" (Netflix) - Match: 91%                              │
│  3. "Dark" (Netflix) - Match: 88%                                       │
│  4. "Lovecraft Country" (HBO Max) - Match: 85%                          │
│                                                                          │
│  Step 6: User selects → Deep link to platform                           │
│  ───────────────────────────────────────────                            │
│  Action: Tap "The Haunting of Hill House"                               │
│  Deep Link: netflix://title/80189221                                    │
│  Fallback: https://netflix.com/title/80189221                           │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.2 User Flow: Cross-Device Handoff

```
┌─────────────────────────────────────────────────────────────────────────┐
│              USER FLOW: CROSS-DEVICE HANDOFF                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  SCENARIO: User discovers on phone, continues on TV                     │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ PHONE (Discovery)                              TV (Viewing)       │  │
│  ├──────────────────────────────────────────────────────────────────┤  │
│  │                                                                   │  │
│  │ 1. Search: "90s action movies"                                   │  │
│  │ 2. Browse results                                                │  │
│  │ 3. Add "The Matrix" to watchlist        ───PubNub (45ms)───▶     │  │
│  │                                                                   │  │
│  │                                         4. Watchlist updates     │  │
│  │                                         5. "Continue on TV?"     │  │
│  │ 6. Tap "Send to TV"                     ───PubNub (32ms)───▶     │  │
│  │                                                                   │  │
│  │                                         7. Deep link opens       │  │
│  │                                         8. Netflix launches      │  │
│  │                                         9. The Matrix plays      │  │
│  │                                                                   │  │
│  │ 10. Watch progress syncs                ◀───PubNub (67ms)───     │  │
│  │     (paused at 1:32:45)                                          │  │
│  │                                                                   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  SYNC MECHANISM: CRDT-based state (LWW-Register for progress)          │
│  LATENCY TARGET: <100ms for all sync operations                         │
│  CONFLICT RESOLUTION: Hybrid Logical Clock (HLC) timestamps            │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.3 User Flow: AI Agent Interaction (MCP)

```
┌─────────────────────────────────────────────────────────────────────────┐
│              USER FLOW: AI AGENT VIA MCP                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  SCENARIO: Claude helps user plan movie night                           │
│                                                                          │
│  User: "Plan a movie night for my family. Kids are 8 and 12."          │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ AI AGENT WORKFLOW                                                 │  │
│  ├──────────────────────────────────────────────────────────────────┤  │
│  │                                                                   │  │
│  │ Step 1: ARW Discovery (cached)                                   │  │
│  │ ───────────────────────────────                                  │  │
│  │ GET /.well-known/arw-manifest.json                               │  │
│  │ → Available tools: semantic_search, get_recommendations,         │  │
│  │   get_content_details, list_devices                              │  │
│  │                                                                   │  │
│  │ Step 2: MCP Tool Call - Get Recommendations                      │  │
│  │ ──────────────────────────────────────────                       │  │
│  │ {                                                                 │  │
│  │   "method": "tools/call",                                        │  │
│  │   "params": {                                                    │  │
│  │     "name": "get_recommendations",                               │  │
│  │     "arguments": {                                               │  │
│  │       "context": "family movie night",                           │  │
│  │       "age_appropriate": [8, 12],                                │  │
│  │       "limit": 5                                                 │  │
│  │     }                                                            │  │
│  │   }                                                              │  │
│  │ }                                                                 │  │
│  │                                                                   │  │
│  │ Step 3: Response Processing                                      │  │
│  │ ───────────────────────────                                      │  │
│  │ Results: [                                                        │  │
│  │   { title: "Encanto", platforms: ["Disney+"], rating: "PG" },   │  │
│  │   { title: "Luca", platforms: ["Disney+"], rating: "PG" },      │  │
│  │   { title: "The Mitchells vs. the Machines", platforms: ["Netflix"] }│
│  │ ]                                                                 │  │
│  │                                                                   │  │
│  │ Step 4: MCP Tool Call - Check Availability                       │  │
│  │ ─────────────────────────────────────────                        │  │
│  │ For each recommendation:                                         │  │
│  │ → get_content_details(entity_id) → availability in user's region│  │
│  │                                                                   │  │
│  │ Step 5: Agent Synthesizes Response                               │  │
│  │ ─────────────────────────────────                                │  │
│  │ "Here are 3 great options for your family movie night:          │  │
│  │  1. Encanto (Disney+) - Musical adventure, PG, 1h 49m           │  │
│  │  2. Luca (Disney+) - Italian seaside adventure, PG, 1h 35m      │  │
│  │  3. The Mitchells vs. Machines (Netflix) - Sci-fi comedy, PG   │  │
│  │                                                                   │  │
│  │  All are available on your subscribed platforms!"                │  │
│  │                                                                   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  TOKEN EFFICIENCY: 850 tokens (vs. 5,600 tokens with HTML scraping)    │
│  LATENCY: 340ms total (MCP overhead: 15ms)                              │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.4 User Flow: Device Authorization (TV/CLI)

```
┌─────────────────────────────────────────────────────────────────────────┐
│              USER FLOW: DEVICE AUTHORIZATION GRANT (RFC 8628)            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  SCENARIO: User logs into Media Gateway on Smart TV                    │
│                                                                          │
│  ┌─────────────────────┐              ┌─────────────────────┐          │
│  │     SMART TV        │              │    MOBILE PHONE     │          │
│  ├─────────────────────┤              ├─────────────────────┤          │
│  │                     │              │                     │          │
│  │ 1. Launch app       │              │                     │          │
│  │                     │              │                     │          │
│  │ 2. Request device   │              │                     │          │
│  │    code from API    │              │                     │          │
│  │    ────────────────▶│              │                     │          │
│  │                     │              │                     │          │
│  │ 3. Display:         │              │                     │          │
│  │    ┌─────────────┐  │              │                     │          │
│  │    │ Code: ABCD- │  │              │                     │          │
│  │    │      1234   │  │              │                     │          │
│  │    │             │  │              │                     │          │
│  │    │ Go to:      │  │              │                     │          │
│  │    │ mg.app/link │  │              │                     │          │
│  │    └─────────────┘  │              │                     │          │
│  │                     │              │ 4. User opens       │          │
│  │                     │              │    mg.app/link      │          │
│  │                     │              │                     │          │
│  │                     │              │ 5. Enter code:      │          │
│  │                     │              │    ABCD-1234        │          │
│  │                     │              │                     │          │
│  │                     │              │ 6. Confirm on       │          │
│  │                     │              │    existing session │          │
│  │                     │              │                     │          │
│  │ 7. Poll succeeds    │◀─────────────│ 7. Authorization    │          │
│  │    (access_token)   │              │    granted          │          │
│  │                     │              │                     │          │
│  │ 8. App loads        │              │ 8. "TV connected!"  │          │
│  │    personalized     │              │                     │          │
│  │    content          │              │                     │          │
│  │                     │              │                     │          │
│  └─────────────────────┘              └─────────────────────┘          │
│                                                                          │
│  SECURITY:                                                               │
│  - Device code expires in 15 minutes                                    │
│  - Polling interval: 5 seconds                                          │
│  - Max attempts: 180 (15 min / 5 sec)                                   │
│  - User must already be authenticated on phone                          │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.5 User Flow: Watchlist Management

```
┌─────────────────────────────────────────────────────────────────────────┐
│              USER FLOW: WATCHLIST MANAGEMENT                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  OPERATIONS: Add, Remove, Reorder, Categorize                           │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ ADD TO WATCHLIST                                                  │  │
│  ├──────────────────────────────────────────────────────────────────┤  │
│  │                                                                   │  │
│  │ 1. User taps "+" on content card                                 │  │
│  │ 2. Client generates OR-Set add operation:                        │  │
│  │    {                                                              │  │
│  │      "op": "add",                                                 │  │
│  │      "entity_id": "eidr:10.5240/1234-5678-90AB",                 │  │
│  │      "tag": "uuid-v4-unique",                                    │  │
│  │      "timestamp": "2025-12-06T21:30:00.000Z",                    │  │
│  │      "device_id": "phone-abc123"                                 │  │
│  │    }                                                              │  │
│  │ 3. Optimistic local update (instant UI feedback)                 │  │
│  │ 4. PubNub publish to user.{userId}.sync                          │  │
│  │ 5. Other devices receive → merge OR-Set                          │  │
│  │ 6. Backend persists to PostgreSQL (async)                        │  │
│  │                                                                   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ CONFLICT RESOLUTION (CRDT)                                        │  │
│  ├──────────────────────────────────────────────────────────────────┤  │
│  │                                                                   │  │
│  │ Scenario: User adds item on phone while removing on tablet       │  │
│  │           (both offline, sync when reconnected)                  │  │
│  │                                                                   │  │
│  │ Phone:  add("Movie A", tag: "aaa", t: 10:00)                     │  │
│  │ Tablet: remove("Movie A", tag: "bbb", t: 10:01)                  │  │
│  │                                                                   │  │
│  │ Merge Result: OR-Set semantics                                   │  │
│  │ - If add tag "aaa" not in remove set → item stays               │  │
│  │ - Add-wins bias: concurrent add/remove → add wins               │  │
│  │                                                                   │  │
│  │ Final State: "Movie A" IN watchlist                              │  │
│  │                                                                   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  SYNC LATENCY TARGETS:                                                  │
│  - Local update: <10ms                                                  │
│  - Cross-device sync: <100ms (PubNub)                                  │
│  - Backend persist: <500ms (async, non-blocking)                       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## End of Part 1

**Continue to:** [SPARC Specification — Part 2 of 4](./SPARC_SPECIFICATION_PART_2.md)

**Part 2 Contents:**
- Ingestion Behavior Specifications
- Metadata Requirements
- Streaming Platform Interaction Patterns
- MCP Connector Role and Tools
