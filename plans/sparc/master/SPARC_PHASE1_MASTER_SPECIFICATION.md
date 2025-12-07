# SPARC Phase 1: Master Specification

## Media Gateway: Unified Cross-Platform TV Discovery Engine

**Document Version:** 1.0.0
**SPARC Phase:** Specification (Phase 1 - Complete)
**Date:** 2025-12-06
**Status:** Master Document - Consolidated

---

## Document Overview

This master document consolidates the complete SPARC Phase 1 Specification for the Media Gateway project. It combines the content from Parts 1-4 into a single comprehensive reference.

### Source Documents
- SPARC_SPECIFICATION_PART_1.md (Sections 1-6)
- SPARC_SPECIFICATION_PART_2.md (Sections 7-10)
- SPARC_SPECIFICATION_PART_3.md (Sections 11-14)
- SPARC_SPECIFICATION_PART_4.md (Sections 15-22)

---

## Master Table of Contents

### Part 1: Foundation
1. [Executive Summary](#1-executive-summary)
2. [Problem Space Definition](#2-problem-space-definition)
3. [System Goals and Objectives](#3-system-goals-and-objectives)
4. [System Boundaries](#4-system-boundaries)
5. [Stakeholder Analysis](#5-stakeholder-analysis)
6. [User Flows](#6-user-flows)

### Part 2: Data & Integration
7. [Ingestion Behavior Specifications](#7-ingestion-behavior-specifications)
8. [Metadata Requirements](#8-metadata-requirements)
9. [Streaming Platform Interaction Patterns](#9-streaming-platform-interaction-patterns)
10. [MCP Connector Role](#10-mcp-connector-role)

### Part 3: Infrastructure & Sync
11. [Ruvector Simulation and Storage Responsibilities](#11-ruvector-simulation-and-storage-responsibilities)
12. [PubNub Real-Time Sync Behavior](#12-pubnub-real-time-sync-behavior)
13. [Device Interactions](#13-device-interactions)
14. [CLI Behavior Specifications](#14-cli-behavior-specifications)

### Part 4: Operations & Requirements
15. [Service Expectations](#15-service-expectations)
16. [Agent Orchestration Goals](#16-agent-orchestration-goals)
17. [Authentication Constraints](#17-authentication-constraints)
18. [Error Cases](#18-error-cases)
19. [Performance Requirements](#19-performance-requirements)
20. [Constraints and Assumptions](#20-constraints-and-assumptions)
21. [Non-Functional Requirements](#21-non-functional-requirements)
22. [Appendix: Glossary](#22-appendix-glossary)

---

# PART 1: FOUNDATION

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
│  │ TOKEN EFFICIENCY: 850 tokens (vs. 5,600 tokens HTML scraping)    │  │
│  │ LATENCY: 340ms total (MCP overhead: 15ms)                        │  │
│  │                                                                   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
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
│  │ 2. Request device   │              │                     │          │
│  │    code from API    │              │                     │          │
│  │ 3. Display:         │              │                     │          │
│  │    ┌─────────────┐  │              │                     │          │
│  │    │ Code: ABCD- │  │              │                     │          │
│  │    │      1234   │  │              │                     │          │
│  │    │ Go to:      │  │              │                     │          │
│  │    │ mg.app/link │  │              │                     │          │
│  │    └─────────────┘  │              │                     │          │
│  │                     │              │ 4. User opens       │          │
│  │                     │              │    mg.app/link      │          │
│  │                     │              │ 5. Enter code:      │          │
│  │                     │              │    ABCD-1234        │          │
│  │                     │              │ 6. Confirm          │          │
│  │ 7. Poll succeeds    │◀─────────────│ 7. Authorization    │          │
│  │    (access_token)   │              │    granted          │          │
│  │ 8. App loads        │              │ 8. "TV connected!"  │          │
│  │    personalized     │              │                     │          │
│  │    content          │              │                     │          │
│  └─────────────────────┘              └─────────────────────┘          │
│                                                                          │
│  SECURITY:                                                               │
│  - Device code expires in 15 minutes                                    │
│  - Polling interval: 5 seconds                                          │
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
│  ADD TO WATCHLIST                                                       │
│  ────────────────                                                       │
│  1. User taps "+" on content card                                       │
│  2. Client generates OR-Set add operation                               │
│  3. Optimistic local update (instant UI feedback)                       │
│  4. PubNub publish to user.{userId}.sync                               │
│  5. Other devices receive → merge OR-Set                                │
│  6. Backend persists to PostgreSQL (async)                              │
│                                                                          │
│  CONFLICT RESOLUTION (CRDT)                                             │
│  ─────────────────────────                                              │
│  Scenario: User adds item on phone while removing on tablet             │
│           (both offline, sync when reconnected)                         │
│                                                                          │
│  Phone:  add("Movie A", tag: "aaa", t: 10:00)                          │
│  Tablet: remove("Movie A", tag: "bbb", t: 10:01)                       │
│                                                                          │
│  Merge Result: OR-Set semantics                                         │
│  - Add-wins bias: concurrent add/remove → add wins                     │
│  Final State: "Movie A" IN watchlist                                    │
│                                                                          │
│  SYNC LATENCY TARGETS:                                                  │
│  - Local update: <10ms                                                  │
│  - Cross-device sync: <100ms (PubNub)                                  │
│  - Backend persist: <500ms (async, non-blocking)                       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

# PART 2: DATA & INTEGRATION

---

## 7. Ingestion Behavior Specifications

### 7.1 Data Ingestion Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    INGESTION PIPELINE ARCHITECTURE                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  EXTERNAL SOURCES                         INGESTION LAYER               │
│  ─────────────────                        ───────────────               │
│                                                                          │
│  ┌─────────────┐     ┌─────────────────────────────────────────────┐   │
│  │ Streaming   │────▶│  Platform Normalizer (51 micro-repos)       │   │
│  │ Availability│     │  ├── Netflix Normalizer                     │   │
│  │ API         │     │  ├── Prime Normalizer                       │   │
│  └─────────────┘     │  ├── Disney+ Normalizer                     │   │
│                      │  └── ... (48 more)                          │   │
│  ┌─────────────┐     └──────────────┬──────────────────────────────┘   │
│  │ Watchmode   │────▶               │                                   │
│  │ API         │                    ▼                                   │
│  └─────────────┘     ┌─────────────────────────────────────────────┐   │
│                      │  Entity Resolver                            │   │
│  ┌─────────────┐     │  ├── EIDR Matching                          │   │
│  │ YouTube     │────▶│  ├── Fuzzy Title Matching                   │   │
│  │ Data API    │     │  ├── Cross-Platform Deduplication           │   │
│  └─────────────┘     │  └── Canonical Entity Creation              │   │
│                      └──────────────┬──────────────────────────────┘   │
│  ┌─────────────┐                    │                                   │
│  │ TMDb API    │────▶               ▼                                   │
│  └─────────────┘     ┌─────────────────────────────────────────────┐   │
│                      │  Kafka Event Stream                         │   │
│  ┌─────────────┐     │  ├── content.ingested                       │   │
│  │ Gracenote/  │────▶│  ├── content.updated                        │   │
│  │ TMS         │     │  ├── availability.changed                   │   │
│  └─────────────┘     │  └── metadata.enriched                      │   │
│                      └──────────────┬──────────────────────────────┘   │
│                                     │                                   │
│                                     ▼                                   │
│                      ┌─────────────────────────────────────────────┐   │
│                      │  Storage Layer                               │   │
│                      │  ├── PostgreSQL (canonical data)            │   │
│                      │  ├── Ruvector (embeddings + graph)          │   │
│                      │  └── Valkey (cache)                         │   │
│                      └─────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 7.2 Platform Normalizer Specification

```rust
/// Standard interface for all platform normalizers
pub trait PlatformNormalizer: Send + Sync {
    /// Platform identifier (e.g., "netflix", "prime_video")
    fn platform_id(&self) -> &'static str;

    /// Fetch catalog updates since last sync
    async fn fetch_catalog_delta(
        &self,
        since: DateTime<Utc>,
        region: &str,
    ) -> Result<Vec<RawContent>, NormalizerError>;

    /// Transform raw platform data to canonical schema
    fn normalize(&self, raw: RawContent) -> Result<CanonicalContent, NormalizerError>;

    /// Generate deep link for content
    fn generate_deep_link(&self, content_id: &str) -> DeepLinkResult;

    /// Platform-specific rate limit configuration
    fn rate_limit_config(&self) -> RateLimitConfig;
}
```

### 7.3 Ingestion Scheduling

| Data Category | Refresh Frequency | Trigger | Priority |
|--------------|-------------------|---------|----------|
| Catalog (new content) | Every 6 hours | Scheduled | High |
| Availability changes | Every 1 hour | Scheduled + Webhook | Critical |
| Expiring content | Every 15 minutes | Scheduled | Critical |
| Metadata enrichment | Every 24 hours | Scheduled | Medium |
| Trending/Popular | Every 30 minutes | Scheduled | Medium |
| User preferences | Real-time | Event-driven | High |

### 7.4 Entity Resolution Pipeline

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    ENTITY RESOLUTION PIPELINE                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Stage 1: EIDR Matching (Exact)                                         │
│  ─────────────────────────────                                          │
│  If EIDR present → direct match to canonical entity                    │
│  Confidence: 100%                                                        │
│                                                                          │
│  Stage 2: External ID Matching                                          │
│  ───────────────────────────                                            │
│  Match by: IMDb ID → TMDb ID → Gracenote TMS ID                        │
│  Confidence: 99%+ (ID collision rare)                                   │
│                                                                          │
│  Stage 3: Fuzzy Title + Year Matching                                   │
│  ─────────────────────────────────                                      │
│  Algorithm: Levenshtein distance + year proximity                       │
│  Threshold: ≥0.85 similarity AND year ±1                               │
│  Confidence: 90-98%                                                     │
│                                                                          │
│  Stage 4: Embedding Similarity                                          │
│  ───────────────────────────                                            │
│  Vector cosine similarity on title + description embeddings            │
│  Threshold: ≥0.92 similarity                                           │
│  Confidence: 85-95%                                                     │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 8. Metadata Requirements

### 8.1 Core Metadata Fields

| Field | Type | Required | Source Priority | Description |
|-------|------|----------|-----------------|-------------|
| `entity_id` | String | Yes | EIDR > generated | Universal content identifier |
| `content_type` | Enum | Yes | Platform | Movie, Series, Episode, Season |
| `titles` | Map<Locale, String> | Yes | Gracenote > TMDb > Platform | Localized titles |
| `original_title` | String | Yes | EIDR > TMDb | Original language title |
| `descriptions` | Map<Locale, String> | No | Gracenote > TMDb | Localized synopses |
| `release_date` | Date | No | EIDR > TMDb | Original release date |
| `release_year` | Integer | Yes | Derived | Year of release |
| `runtime_minutes` | Integer | No | Platform average | Duration in minutes |
| `genres` | Array<GenreId> | Yes | Unified taxonomy | Genre classifications |
| `content_ratings` | Map<Region, Rating> | Yes | Platform | Age ratings by region |
| `credits` | Array<Credit> | No | TMDb > Gracenote | Cast and crew |

### 8.2 Availability Metadata

| Field | Type | Required | Update Frequency | Description |
|-------|------|----------|------------------|-------------|
| `platform_id` | String | Yes | Static | Platform identifier |
| `region` | String | Yes | Per-entry | ISO 3166-1 alpha-3 |
| `availability_type` | Enum | Yes | Hourly | Subscription/Rent/Buy/Free |
| `price` | Price | Conditional | Hourly | Required for Rent/Buy |
| `quality_tiers` | Array<Quality> | No | Daily | SD/HD/4K/HDR/Atmos |
| `deep_link` | URL | Yes | Weekly | Platform-specific link |
| `expires_at` | DateTime | No | Hourly | Content removal date |

### 8.3 Identifier Standards

```
EIDR (Entertainment Identifier Registry)
────────────────────────────────────────
Format: 10.5240/XXXX-XXXX-XXXX-XXXX-X-X
Example: 10.5240/7B2F-ED3B-4F25-4A59-R

IDENTIFIER CROSS-REFERENCE (Example: "The Matrix")
─────────────────────────────────────────────────
EIDR               │ 10.5240/7B2F-ED3B-4F25-4A59-R │ ISO 26324
IMDb               │ tt0133093                      │ IMDb.com
TMDb (Movie)       │ 603                            │ TMDb.org
Gracenote TMS      │ MV000057872000                 │ Gracenote
Netflix            │ 20557937                       │ Platform
Prime Video        │ B00FZL4WEK                     │ Platform
```

---

## 9. Streaming Platform Interaction Patterns

### 9.1 Platform Integration Matrix

| Platform | API | Integration | Deep Link |
|----------|-----|-------------|-----------|
| Netflix | ❌ No | Aggregator | netflix://title/X |
| Prime Video | ❌ No | Aggregator | aiv://detail?asin= |
| Disney+ | ❌ No | Aggregator | disneyplus:// |
| Hulu | ❌ No | Aggregator | hulu://watch/ |
| HBO Max | ❌ No | Aggregator | hbomax://content/ |
| Apple TV+ | ❌ No | Aggregator | tvapp:// |
| YouTube | ✅ Yes | Direct OAuth | youtube://watch?v= |
| Crunchyroll | ⚠️ Ltd | Aggregator+API | crunchyroll:// |

### 9.2 Aggregator API Integration

**Streaming Availability API:**
- Rate limits: 100/minute
- Coverage: 60+ countries, 150+ services
- Content: 500,000+ titles

**Watchmode API:**
- Rate limits: 1,000/day
- Coverage: 50+ countries, 200+ services
- Content: 1M+ titles

---

## 10. MCP Connector Role

### 10.1 MCP Server Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    MCP SERVER ARCHITECTURE                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                     AI AGENT (Claude, GPT, etc.)                   │  │
│  └───────────────────────────────┬───────────────────────────────────┘  │
│                                  │                                       │
│                    ┌─────────────┴─────────────┐                        │
│                    │   MCP Transport Layer     │                        │
│                    │  ┌─────────┐  ┌─────────┐│                        │
│                    │  │  STDIO  │  │   SSE   ││                        │
│                    │  └─────────┘  └─────────┘│                        │
│                    └─────────────┬─────────────┘                        │
│                                  │                                       │
│  ┌───────────────────────────────┴───────────────────────────────────┐  │
│  │                     MCP SERVER                                     │  │
│  │                                                                    │  │
│  │  CAPABILITIES                                                      │  │
│  │  ─────────────                                                     │  │
│  │  TOOLS (10)         │ RESOURCES (3)     │ PROMPTS (2)             │  │
│  │  • semantic_search  │ • hackathon://    │ • discovery_assistant   │  │
│  │  • get_content      │   config          │ • recommendation_guide  │  │
│  │  • discover_content │ • hackathon://    │                         │  │
│  │  • get_recommend.   │   tracks          │                         │  │
│  │  • initiate_play    │ • media://        │                         │  │
│  │  • control_playback │   trending        │                         │  │
│  │  • get_genres       │                   │                         │  │
│  │  • update_prefs     │                   │                         │  │
│  │  • list_devices     │                   │                         │  │
│  │  • get_device_stat  │                   │                         │  │
│  │                                                                    │  │
│  │  MIDDLEWARE                                                        │  │
│  │  ──────────                                                        │  │
│  │  • Authentication (OAuth 2.0 validation)                          │  │
│  │  • Rate Limiting (100-1000 req/15min)                             │  │
│  │  • Input Validation (Zod schemas)                                  │  │
│  │  • Error Handling (JSON-RPC 2.0)                                  │  │
│  │                                                                    │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 10.2 Key MCP Tools

| Tool | Description | Auth Required |
|------|-------------|---------------|
| `semantic_search` | Natural language content search | No |
| `get_content_details` | Full content metadata | No |
| `get_recommendations` | Personalized suggestions | Yes |
| `list_devices` | User's registered devices | Yes |
| `initiate_playback` | Start content on device | Yes |
| `control_playback` | Play/pause/seek controls | Yes |
| `update_preferences` | Modify user preferences | Yes |

### 10.3 ARW Manifest

```json
{
  "$schema": "https://arw.agentics.org/schemas/manifest-v1.json",
  "version": "1.0.0",
  "site": {
    "name": "Media Gateway",
    "description": "Unified cross-platform TV and movie discovery engine"
  },
  "capabilities": {
    "mcp": {
      "version": "2024-11-05",
      "transports": ["stdio", "sse"]
    },
    "semantic_search": {
      "endpoint": "/api/search",
      "supports_natural_language": true
    },
    "actions": [
      { "id": "search", "oauth_scopes": ["read:content"] },
      { "id": "recommend", "oauth_scopes": ["read:content", "read:preferences"] },
      { "id": "playback", "oauth_scopes": ["read:content", "write:playback"], "requires_user_consent": true }
    ]
  },
  "rate_limits": {
    "unauthenticated": 10,
    "authenticated": 1000,
    "window_seconds": 900
  }
}
```

---

# PART 3: INFRASTRUCTURE & SYNC

---

## 11. Ruvector Simulation and Storage Responsibilities

### 11.1 Ruvector Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    RUVECTOR ARCHITECTURE                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │  Vector Store   │  │ Hypergraph DB   │  │  GNN Layer      │         │
│  │  ─────────────  │  │  ─────────────  │  │  ─────────────  │         │
│  │  • 768-dim      │  │  • Multi-edge   │  │  • GraphSAGE    │         │
│  │    embeddings   │  │    relations    │  │  • 8-head attn  │         │
│  │  • HNSW index   │  │  • Platform     │  │  • +12.4% recall│         │
│  │  • 61μs p50     │  │    hyperedges   │  │  • 3.8ms fwd    │         │
│  │  • 8.2x faster  │  │  • Genre graph  │  │                 │         │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘         │
│           │                   │                    │                    │
│           └───────────────────┼────────────────────┘                    │
│                               │                                         │
│                               ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                   UNIFIED QUERY ENGINE                           │   │
│  │  • Hybrid search (vector + graph + text)                        │   │
│  │  • SONA-aware ranking                                           │   │
│  │  • Result fusion and deduplication                              │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  STORAGE BACKENDS:                                                      │
│  Production:  PostgreSQL + pg_vector + Valkey cluster                  │
│  Development: SQLite + in-memory HNSW                                  │
│  Testing:     Mock backends with deterministic responses               │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 11.2 Vector Storage Specifications

```yaml
embeddings:
  model: "text-embedding-3-small"  # OpenAI
  dimensions: 768
  normalization: true  # L2 normalize for cosine similarity

index:
  type: HNSW
  params:
    M: 16
    efConstruction: 200
    efSearch: 100

  performance:
    p50_latency: 61μs
    p95_latency: 150μs
    p99_latency: 500μs
    throughput: 16,000 QPS
```

### 11.3 Data Persistence Matrix

| Data Type | Primary Storage | Cache | Retention | Backup |
|-----------|----------------|-------|-----------|--------|
| Content metadata | PostgreSQL | Valkey (24h) | Permanent | Daily |
| Embeddings | pg_vector | Valkey (7d) | Permanent | Daily |
| Graph edges | PostgreSQL | Valkey (1h) | Permanent | Daily |
| User preferences | PostgreSQL + device | Valkey (1h) | 7 years | Real-time |
| Watch history | PostgreSQL | Valkey (24h) | 90 days (VPPA) | Daily |
| Session state | Valkey | N/A | 24 hours | None |

---

## 12. PubNub Real-Time Sync Behavior

### 12.1 Channel Architecture

```
CHANNEL HIERARCHY:
──────────────────

user.{userId}
├── .sync           # Watchlist, preferences, watch progress
├── .devices        # Device presence, heartbeat, control
└── .notifications  # Alerts, recommendations, expiring content

global
├── .trending       # Top 100 content (hourly updates)
└── .announcements  # System-wide messages

region.{regionCode}
└── .updates        # Regional availability changes
```

### 12.2 CRDT Specifications

**Hybrid Logical Clock (HLC):**
- 48-bit Unix timestamp (ms)
- 16-bit logical counter
- Tie-breaker: lexicographic device_id comparison

**LWW-Register (Last-Writer-Wins):**
- Used for: Watch Progress, User Preferences
- Conflict resolution: Latest timestamp wins

**OR-Set (Observed-Remove Set):**
- Used for: Watchlist
- Add-wins semantics: concurrent add/remove → add wins

### 12.3 Sync Latency Requirements

| Operation | Target p50 | Target p99 | SLO |
|-----------|-----------|-----------|-----|
| Watch progress sync | 50ms | 100ms | 99.9% |
| Watchlist update | 50ms | 100ms | 99.9% |
| Remote control command | 25ms | 75ms | 99.9% |
| Device presence update | 50ms | 150ms | 99.5% |

---

## 13. Device Interactions

### 13.1 Device Capabilities Schema

```typescript
interface DeviceCapabilities {
  // Video capabilities
  supports_4k: boolean;
  supports_hdr: boolean;
  hdr_formats: ("hdr10" | "dolby_vision" | "hdr10plus" | "hlg")[];
  max_resolution: "sd" | "hd" | "fhd" | "4k" | "8k";

  // Audio capabilities
  supports_dolby_atmos: boolean;
  audio_channels: "stereo" | "5.1" | "7.1" | "atmos";

  // Interaction capabilities
  has_remote_control: boolean;
  has_voice_input: boolean;
  has_touch_screen: boolean;

  // Platform integrations
  supported_platforms: string[];
  can_cast_to: string[];
  can_receive_cast: boolean;
}
```

### 13.2 Remote Control Protocol

**Supported Commands:**
- `cast` - Open content on target device
- `play` - Resume playback
- `pause` - Pause playback
- `stop` - Stop and close player
- `seek` - Jump to specific timestamp
- `volume` - Adjust volume

**Latency Target:** <100ms end-to-end (typical: 50-75ms)

---

## 14. CLI Behavior Specifications

### 14.1 Command Structure

```
media-gateway
├── init              # Initialize configuration
├── search <query>    # Content discovery
├── recommend         # Personalized recommendations
├── info <entity_id>  # Content details
├── watchlist         # Watchlist management
│   ├── list
│   ├── add <id>
│   ├── remove <id>
│   └── sync
├── devices           # Device management
│   ├── list
│   ├── rename <id>
│   └── remove <id>
├── cast <entity_id>  # Send to device
├── mcp               # MCP server mode
│   ├── start
│   └── --transport [stdio|sse]
├── config            # Configuration
├── auth              # Authentication
│   ├── login
│   ├── logout
│   └── status
└── help
```

### 14.2 Exit Codes

| Code | Name | Description |
|------|------|-------------|
| 0 | SUCCESS | Command completed successfully |
| 1 | GENERAL_ERROR | Generic error |
| 2 | INVALID_USAGE | Invalid command or arguments |
| 3 | NOT_AUTHENTICATED | Authentication required |
| 4 | NOT_FOUND | Resource not found |
| 5 | NETWORK_ERROR | Unable to reach API |
| 7 | RATE_LIMITED | API rate limit exceeded |
| 130 | INTERRUPTED | User cancelled (Ctrl+C) |

---

# PART 4: OPERATIONS & REQUIREMENTS

---

## 15. Service Expectations

### 15.1 Core Services

| Service | Responsibility | SLO |
|---------|----------------|-----|
| Search Service | SONA-powered content search | p95 < 500ms, 99.9% |
| Recommendation Service | Personalized suggestions | <5ms SONA, 99.9% |
| Device Service | Device registry & control | p99 < 100ms, 99.9% |
| Ingestion Service | Platform data fetching | High throughput |
| Sync Service | PubNub CRDT operations | <100ms, 99.9% |
| Auth Service | OAuth 2.0 + PKCE | 99.99% |

### 15.2 Service Lifecycle

```yaml
lifecycle:
  startup:
    sequence:
      1. Load configuration from Secret Manager
      2. Initialize database connections (with retry)
      3. Connect to cache (Valkey)
      4. Register with service mesh
      5. Start health check endpoint
      6. Begin accepting traffic

  shutdown:
    graceful_period: 30s
    force_kill_after: 60s

  scaling:
    min_replicas: 2
    max_replicas: 100
    target_cpu: 70%
```

---

## 16. Agent Orchestration Goals

### 16.1 Autonomous Operation Modes

**Catalog Maintenance:**
- Trigger: Every 6 hours or webhook
- Actions: Fetch delta, resolve entities, update embeddings
- Guardrails: Max 10,000 updates/run, alert on >5% catalog change

**Recommendation Optimization:**
- Trigger: Nightly or >1,000 feedback events
- Actions: Analyze CTR, adjust SONA weights, retrain LoRA
- Guardrails: Max 10% model drift, validation before deploy

### 16.2 Multi-Agent Coordination

```yaml
multi_agent_coordination:
  topology: hierarchical

  agents:
    orchestrator: Coordinate all agents, manage priorities
    researchers (3): Gather information from external sources
    processors (5): Transform and index data
    responders (10): Handle user queries

  communication:
    protocol: "Message queue (Pub/Sub)"
    patterns:
      - request_response (sync)
      - fire_and_forget (async)
      - broadcast (coordination)

  memory_namespace: "aqe/*"
```

---

## 17. Authentication Constraints

### 17.1 Authentication Flows

**Web/Mobile:** OAuth 2.0 + PKCE (RFC 7636)
- code_verifier/code_challenge for public clients
- Access token: JWT, 1 hour expiry
- Refresh token: Opaque, 30 day expiry, rotate on use

**TV/CLI:** Device Authorization Grant (RFC 8628)
- Device code expires in 15 minutes
- Polling interval: 5 seconds
- User authenticates on phone/computer

### 17.2 Authorization Model

| Role | Permissions |
|------|-------------|
| anonymous | search:read (limited), trending:read |
| free_user | search, recommendations (limited), watchlist, devices (max 2) |
| premium_user | unlimited recommendations, devices (max 10), history export |
| admin | all permissions + user management |

---

## 18. Error Cases

### 18.1 Error Taxonomy

**Client Errors (4xx):**
- 400 Bad Request, 401 Unauthorized, 403 Forbidden
- 404 Not Found, 422 Unprocessable, 429 Rate Limited

**Server Errors (5xx):**
- 500 Internal, 502 Bad Gateway, 503 Unavailable, 504 Timeout

**Domain Errors:**
- CONTENT_NOT_FOUND, PLATFORM_UNAVAILABLE, DEVICE_OFFLINE
- SYNC_CONFLICT, QUOTA_EXCEEDED, INTENT_UNCLEAR

### 18.2 Graceful Degradation

| Failure | Response |
|---------|----------|
| Search service down | Return cached trending, show "temporarily unavailable" |
| Recommendation service down | Fall back to popularity-based |
| Aggregator quota exceeded | Serve from cache, reduce refresh frequency |
| PubNub connection lost | Queue locally, replay on reconnect |

---

## 19. Performance Requirements

### 19.1 Latency Targets

| Operation | p50 | p95 | p99 |
|-----------|-----|-----|-----|
| Search (simple) | 50ms | 200ms | 500ms |
| Search (complex NL) | 100ms | 500ms | 1s |
| Content details | 20ms | 100ms | 200ms |
| Recommendations | 30ms | 150ms | 300ms |
| SONA personalization | 2ms | 5ms | 10ms |
| Remote control | 25ms | 75ms | 150ms |

### 19.2 Scalability Targets

| Metric | Launch | Year 1 | Year 3 |
|--------|--------|--------|--------|
| Concurrent users | 100K | 1M | 10M |
| Content catalog | 500K | 1M | 5M |
| Devices per user | avg: 3 | max: 10 | - |

---

## 20. Constraints and Assumptions

### 20.1 Technical Constraints

- 80% of streaming platforms have NO public APIs
- Rate limits: SA 100/min, Watchmode 1000/day, YouTube 10K/day
- Primary: Rust, Secondary: TypeScript
- Cloud: Google Cloud Platform only
- Minimum: Node 18+, Rust 1.75+

### 20.2 Business Constraints

- **Budget:** ~$4,400/month infrastructure
- **Timeline:** MVP 8 weeks, Beta 12 weeks, Production 16 weeks
- **Team:** 2-3 backend, 1-2 frontend, 1 ML, 0.5 DevOps

### 20.3 Regulatory Constraints

- **GDPR:** Right to access (30d), erasure (72h), portability
- **CCPA:** Do not sell, right to know, right to delete
- **VPPA:** Video viewing data retention max 90 days
- **WCAG:** AA accessibility compliance

---

## 21. Non-Functional Requirements

### 21.1 Availability

| Tier | Services | SLO | Monthly Downtime |
|------|----------|-----|------------------|
| Tier 1 | Search, Recommendations, Auth | 99.9% | <43 minutes |
| Tier 2 | Device, Sync, Ingestion | 99.5% | <3.6 hours |
| Tier 3 | Analytics, Admin | 99.0% | <7.2 hours |

**Disaster Recovery:**
- RTO: 4 hours
- RPO: 1 hour
- Multi-region: us-central1 (primary), us-east1 (failover)

### 21.2 Security

- OAuth 2.0 + PKCE for all clients
- mTLS for service-to-service
- Encryption: AES-256 at rest, TLS 1.3 in transit
- VPC with private subnets, Cloud Armor WAF
- Quarterly penetration testing

### 21.3 Observability

- **Metrics:** Prometheus, 30-day retention
- **Logging:** Structured JSON, Cloud Logging
- **Tracing:** OpenTelemetry, 1% sampling (100% on errors)
- **Alerting:** PagerDuty (critical), Slack (warning)

---

## 22. Appendix: Glossary

| Term | Definition |
|------|------------|
| **ARW** | Agent-Ready Web - Protocol for AI agent discovery and interaction |
| **CRDT** | Conflict-free Replicated Data Type - Data structures that enable automatic conflict resolution |
| **EIDR** | Entertainment Identifier Registry - Universal content identifier standard |
| **HLC** | Hybrid Logical Clock - Timestamp mechanism combining physical and logical time |
| **LWW** | Last-Writer-Wins - CRDT conflict resolution strategy |
| **MCP** | Model Context Protocol - Standard for AI agent communication |
| **OR-Set** | Observed-Remove Set - CRDT for set operations |
| **PKCE** | Proof Key for Code Exchange - OAuth 2.0 extension for public clients |
| **PubNub** | Real-time messaging infrastructure |
| **Ruvector** | Vector database + hypergraph storage component |
| **SONA** | Self-Optimizing Neural Architecture - Personalization engine |
| **SPARC** | Specification, Pseudocode, Architecture, Refinement, Completion |
| **SSE** | Server-Sent Events - HTTP streaming protocol |
| **VPPA** | Video Privacy Protection Act - US law governing video viewing data |

---

## Document Summary

This SPARC Phase 1 Master Specification consolidates all requirements for evolving hackathon-tv5 into a production-ready Media Gateway engine.

### Key Deliverables
1. Unified cross-platform content discovery
2. SONA-powered personalized recommendations
3. Real-time cross-device sync via PubNub
4. MCP server for AI agent integration
5. ARW protocol for efficient agent discovery
6. Privacy-first architecture with CRDT-based sync

### Technology Stack
- **Core:** Rust microservices
- **CLI/Web:** TypeScript
- **Database:** PostgreSQL + pg_vector
- **Cache:** Valkey
- **Real-time:** PubNub
- **Cloud:** GCP (GKE Autopilot, Cloud Run, Cloud SQL)

### Performance Targets
- Search latency: <500ms (p95)
- Personalization: <5ms
- Cross-device sync: <100ms
- Availability: 99.9% (Tier 1)

### Cost Target
~$4,400/month

---

**End of SPARC Phase 1: Master Specification**

**Next Phases:**
1. SPARC Pseudocode Phase (algorithm design)
2. SPARC Architecture Phase (system design)
3. SPARC Refinement Phase (TDD implementation)
4. SPARC Completion Phase (integration and deployment)

---

*Document consolidated from SPARC_SPECIFICATION_PART_1.md through SPARC_SPECIFICATION_PART_4.md*
*Generated: 2025-12-06*
