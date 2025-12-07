# Streaming Platform Research Summary

**Research Date:** 2025-12-06
**Phase:** SPARC Specification
**Status:** Complete

---

## Executive Summary

Research into 10+ major streaming platforms reveals a critical finding: **direct API integration is not viable for 80% of platforms**. The Media Gateway architecture must be designed around third-party aggregator APIs and deep linking as the primary integration strategy.

---

## Key Findings

### 1. API Availability Landscape

**No Public APIs Available:**
- Netflix (Partner API only - requires content partnership)
- Prime Video (Partner API only)
- Disney+ (No API)
- Hulu (No API)
- HBO Max (No API)
- Peacock (No API)
- Paramount+ (No API)
- Crave (No API - Canada only)

**Public API Available:**
- YouTube (Full public API with OAuth 2.0)

**Conclusion:** Only 1 out of 10 platforms offers public API access. Direct platform integration is not a viable strategy.

---

## 2. Integration Strategy

### Recommended Approach: Aggregator APIs + Deep Linking

**Primary Integration:**
1. **Third-Party Aggregator APIs** for content metadata and availability
   - Streaming Availability API (60+ countries, 150+ platforms)
   - Watchmode API (200+ services, 50+ countries)
   - International Showtimes API (100+ markets)

2. **Deep Linking** for content access
   - iOS Universal Links (`.well-known/apple-app-site-association`)
   - Android App Links (`.well-known/assetlinks.json`)
   - Direct handoff to platform apps for viewing

3. **YouTube Direct Integration** (only platform with public API)
   - OAuth 2.0 with PKCE
   - Device Authorization Grant for TV/CLI
   - Full API access for search, recommendations, user data

---

## 3. Authentication Patterns

### OAuth 2.0 Standard Flows

**For Web/Mobile Applications:**
```
Authorization Code + PKCE (RFC 7636)
- Generate code_verifier and code_challenge
- User authorizes via platform
- Exchange authorization code for tokens
- PKCE prevents code interception attacks
```

**For TV/CLI Applications:**
```
Device Authorization Grant (RFC 8628)
- Request device_code and user_code
- Display user_code on screen
- User authorizes on phone/browser
- Device polls for token completion
```

**Security Requirements (RFC 9700 - January 2025):**
- Short-lived access tokens (15-60 minutes)
- Refresh token rotation
- No deprecated flows (Implicit Grant, ROPC)
- Scope and audience restriction
- Sender-constrained tokens for high-security scenarios

---

## 4. Platform Interaction Patterns

### Content Discovery Flow

```
1. User searches in Media Gateway
   ↓
2. Query aggregator API (Streaming Availability, Watchmode)
   ↓
3. Receive content metadata + platform availability
   ↓
4. Display results with "Watch on [Platform]" buttons
   ↓
5. User selects platform
   ↓
6. Deep link opens platform app
   (e.g., netflix://title/12345)
   ↓
7. User watches content in platform app
```

**Key Insight:** Media Gateway is a **discovery layer**, not a streaming/transcoding engine. Users watch content on platform apps, not within Media Gateway.

---

## 5. Metadata Standards

### Multiple Identifier Systems

**EIDR (Entertainment Identifier Registry):**
- Non-proprietary unique identifier
- Hierarchical relationships (series → season → episode)
- Industry standard but not universally adopted

**Gracenote/TMS IDs:**
- Rich metadata for search/discovery (themes, moods, keywords)
- 85+ countries, major platforms
- Proprietary (Nielsen-owned)

**TMDb IDs:**
- Community-built database
- Free API with rate limits
- Widely used by third-party tools

**IMDb IDs:**
- User-familiar ratings
- Extensive cast/crew data
- No official API (use aggregators)

**Implementation Strategy:** Support all identifier types, cross-reference to enrich metadata.

---

## 6. Regional Content Handling

### Geographic Rights Management

**Challenges:**
- Country-specific catalogs (Netflix US ≠ Netflix UK)
- Licensing agreements vary by territory
- Time-limited availability (expiry dates)
- Service availability (Hulu US-only, Crave Canada-only)

**Solution:**
- IP-based geolocation with manual override
- Query aggregator APIs with country code
- Filter results to user's region
- Display pricing in local currency
- Track expiry dates (Unix timestamps from APIs)

---

## 7. Privacy and Compliance

### Critical Regulatory Requirements

**GDPR (EU) and CCPA (California):**
- Explicit consent before data collection
- User rights: access, deletion, opt-out, portability
- Minimal data collection
- Transparent privacy policy
- Easy opt-out (single-click, not multi-step)

**VPPA (Video Privacy Protection Act) - 2025 Enforcement:**
- Requires explicit consent for video viewing data
- California AG active enforcement ($530,000 settlement Oct 2025)
- Applies to embedded video players
- Caution with Meta Pixel and social tracking

**Design Principle:** Privacy-first architecture with differential privacy for ML training.

---

## 8. Deep Linking Reliability

### Implementation Challenges

**Known Issues:**
- Email/marketing link wrappers break Universal Links
- Inconsistent platform support across devices
- OS version differences affect behavior
- Hosting platforms may silently rewrite manifest files

**Mitigation Strategies:**
- Use direct links in-app (avoid marketing wrappers)
- Test each platform individually
- Provide manual "Open in App" buttons as fallback
- Monitor deep link success rates
- Clear user messaging on failures

---

## 9. Technology Stack Recommendations

### Core Technologies from Research

**Foundation:**
- hackathon-tv5 toolkit (ARW specification, MCP server, 17+ tools)
- ARW Protocol (85% token reduction for AI agents)

**Intelligence:**
- SONA (Self-Optimizing Neural Architecture)
  - Two-Tier LoRA for personalization (~10KB per user)
  - 39 attention mechanisms (Graph, Hyperbolic, Transformer)
  - Tiny Dancer semantic routing (<5ms latency)
  - ReasoningBank for pattern reuse

**Data Layer:**
- Ruvector (Hypergraph + Vector + GNN)
- PubNub (real-time cross-device sync)

**Security:**
- E2B Sandboxes (Firecracker microVM isolation for agent code)
- OAuth 2.0 with PKCE
- Google Cloud Secret Manager

**Infrastructure:**
- GKE Autopilot (Kubernetes for microservices)
- Cloud Run (serverless API gateway)
- Cloud SQL (PostgreSQL)
- Memorystore (Valkey/Redis caching)

---

## 10. Implementation Priorities

### Phase 1: Foundation (Critical)

**Must-Have Features:**
1. Aggregator API integration (Streaming Availability or Watchmode)
2. Unified metadata schema (TMDb/IMDb IDs minimum)
3. Deep link support (iOS Universal Links, Android App Links)
4. OAuth 2.0 PKCE implementation
5. Multi-region content availability
6. GDPR/CCPA/VPPA compliance (consent, data rights)
7. Unified search API
8. Platform availability display
9. Token security (short-lived, rotation)
10. API rate limiting

### Phase 2: Enhancement (High Priority)

**Should-Have Features:**
1. YouTube API integration (OAuth 2.0)
2. Device Authorization Grant (TV/CLI)
3. Hybrid recommendation engine (collaborative + content-based + GNN)
4. SONA real-time personalization
5. PubNub watchlist sync
6. Content expiry tracking
7. Aggregator API caching (Redis)
8. MFA for user accounts

### Out of Scope

**NOT in Media Gateway:**
- Video/audio stream ingestion
- Video encoding/transcoding
- Bitrate adaptation
- Multistreaming/RTMP output
- Resolution processing

**Rationale:** Media Gateway is content discovery, not streaming infrastructure. Users watch on platform apps via deep links.

---

## 11. Cost Estimates

### Third-Party API Costs

**Streaming Availability API:**
- Pay-per-request model
- Estimated: $200-500/month (depends on traffic)

**Watchmode API:**
- Tiered pricing
- Estimated: $300-600/month

**YouTube API:**
- Free tier: 10,000 quota units/day
- Estimated: Free for moderate usage, $100-200/month if exceeding quota

**Total Aggregator Costs:** $500-1,300/month

### Infrastructure Costs (GCP)

See main architecture document for full breakdown.
**Estimated Total:** $2,400-3,650/month for production deployment.

---

## 12. Key Architectural Decisions

### Decision 1: Aggregator-First Strategy

**Decision:** Use third-party aggregator APIs as primary integration method.

**Rationale:**
- 80% of platforms have no public API
- Aggregators provide unified access to 150+ platforms
- Deep linking handles content access
- Lower maintenance burden than scraping

**Alternatives Rejected:**
- Web scraping (violates TOS, fragile)
- Direct API integration (not available)
- Platform partnerships (not viable for discovery platform)

### Decision 2: Privacy-Safe Personalization

**Decision:** Three-tier data architecture (on-device, federated, server).

**Rationale:**
- GDPR/CCPA require minimal data collection
- VPPA restricts video viewing data
- Users increasingly privacy-conscious
- Differential privacy enables ML without individual tracking

**Implementation:**
- Detailed history stays on-device
- SONA LoRA adapters for personalization (~10KB per user)
- Server only sees anonymized aggregate patterns

### Decision 3: Deep Linking Over Embedding

**Decision:** Direct users to platform apps via deep links.

**Rationale:**
- No platform APIs for embedded playback
- DRM restrictions prevent embedding
- Better user experience (native app features)
- Lower legal/licensing risk

**Trade-offs:**
- Less seamless than in-app playback
- Dependent on platform app quality
- Deep link reliability issues

---

## 13. Risk Assessment

### High Risks

**Risk 1: Aggregator API Changes**
- **Impact:** Critical (core functionality)
- **Likelihood:** Medium
- **Mitigation:** Support multiple aggregators, monitor deprecation notices

**Risk 2: Privacy Regulation Changes**
- **Impact:** High (legal compliance)
- **Likelihood:** High (20+ US states enacting laws)
- **Mitigation:** Design for strictest requirements (GDPR), regular legal review

**Risk 3: Deep Link Reliability**
- **Impact:** Medium (user experience)
- **Likelihood:** High (known platform issues)
- **Mitigation:** Extensive testing, fallback options, user education

### Medium Risks

**Risk 4: Metadata Fragmentation**
- **Impact:** Medium (data quality)
- **Likelihood:** High (multiple ID systems)
- **Mitigation:** Cross-reference multiple identifiers, fuzzy matching

**Risk 5: Regional Content Gaps**
- **Impact:** Medium (feature completeness)
- **Likelihood:** Medium (aggregator coverage varies)
- **Mitigation:** Tiered regional support, clear messaging to users

---

## 14. Next Steps

### Immediate Actions (Week 1)

1. **Select primary aggregator API**
   - Evaluate Streaming Availability API vs. Watchmode
   - Sign up for trial/sandbox access
   - Test API coverage and data quality

2. **Design unified metadata schema**
   - Define MediaContent type
   - Map aggregator responses to schema
   - Plan TMDb/IMDb ID storage

3. **Prototype deep linking**
   - Implement Universal Links (iOS)
   - Implement App Links (Android)
   - Test with Netflix, Prime, Disney+

### Short-Term (Weeks 2-4)

4. **Implement OAuth 2.0 PKCE**
   - Set up authorization server
   - Implement token management
   - Add refresh token rotation

5. **Build search API**
   - Create unified search endpoint
   - Integrate aggregator API
   - Implement platform availability display

6. **Privacy compliance foundation**
   - Draft privacy policy
   - Implement consent management UI
   - Add data deletion endpoints

### Medium-Term (Months 2-3)

7. **YouTube API integration**
   - OAuth 2.0 flow for YouTube
   - Device Grant for TV/CLI
   - Search and recommendation endpoints

8. **Recommendation engine**
   - Collaborative filtering baseline
   - SONA integration for personalization
   - A/B testing framework

9. **Cross-device sync**
   - PubNub channel setup
   - CRDT implementation
   - Watchlist synchronization

---

## 15. Research Artifacts

### Documentation Generated

1. **STREAMING_PLATFORM_SPECIFICATION.md** - Full functional requirements
2. **STREAMING_PLATFORM_RESEARCH_SUMMARY.md** - This document

### Source Materials Analyzed

1. `/tmp/media-gateway-research/research/streaming-platform-research.md` (1,700+ lines)
2. `/tmp/media-gateway-research/research/FINAL_ARCHITECTURE_BLUEPRINT.md`
3. `/tmp/hackathon-tv5/apps/media-discovery/src/lib/tmdb.ts` (TMDB API integration example)
4. `/tmp/hackathon-tv5/apps/media-discovery/src/types/media.ts` (Type definitions)

### Platforms Researched

- Netflix
- Amazon Prime Video
- Disney+
- Hulu
- Apple TV+
- YouTube / YouTube TV
- Crave (Canada)
- HBO Max
- Peacock
- Paramount+

### APIs Evaluated

- Streaming Availability API
- Watchmode API
- International Showtimes API
- YouTube Data API v3
- TMDB API

---

## Conclusion

The Media Gateway platform requires a **fundamentally different architecture** than traditional streaming services. Rather than attempting direct integration with platforms (which is largely impossible), the system acts as an **intelligent discovery layer** that:

1. Aggregates content metadata from third-party APIs
2. Provides unified search and recommendations
3. Handles user privacy with a three-tier data architecture
4. Seamlessly hands off to platform apps via deep linking

This approach aligns with the hackathon-tv5 ARW (Agent-Ready Web) philosophy: optimize for **discovery and orchestration**, not content delivery. The result is a system that solves the "45-minute decision problem" without requiring platform partnerships or complex licensing negotiations.

**Strategic Advantage:** By focusing on discovery rather than streaming, Media Gateway avoids the technical, legal, and financial complexity of content delivery while providing maximum value to users navigating the fragmented streaming landscape.

---

**Research Conducted By:** Research and Analysis Agent
**For:** Media Gateway SPARC Specification Phase
**Date:** 2025-12-06

---

END OF RESEARCH SUMMARY
