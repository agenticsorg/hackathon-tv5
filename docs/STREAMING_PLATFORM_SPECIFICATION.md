# Media Gateway - Streaming Platform Interaction Specification

**Document Version:** 1.0.0
**Date:** 2025-12-06
**Status:** Draft Specification
**Phase:** SPARC Specification

---

## Executive Summary

This specification defines the comprehensive streaming platform interaction patterns for the Media Gateway system. Based on extensive research of 10+ major streaming platforms (Netflix, Prime Video, Disney+, Hulu, Apple TV+, YouTube, Crave, HBO Max, Peacock, Paramount+), this document establishes functional requirements for authentication, content discovery, multistreaming coordination, and output management.

**Key Finding:** Direct API integration is not viable for 8 out of 10 platforms. The Media Gateway architecture leverages third-party aggregator APIs and deep linking as the primary integration strategy.

---

## 1. Streaming Platform Categories

### 1.1 Video Streaming Platforms

#### 1.1.1 Subscription Video-on-Demand (SVOD)

**Tier 1: Major Global Platforms (No Public APIs)**

| Platform | Region | API Status | Integration Method |
|----------|--------|------------|-------------------|
| Netflix | Global | Partner API only | Aggregator + Deep Link |
| Prime Video | Global | Partner API only | Aggregator + Deep Link |
| Disney+ | Global | None | Aggregator + Deep Link |
| Hulu | US only | None | Aggregator + Deep Link |
| Apple TV+ | Global | Partner API only | Aggregator + Deep Link |
| HBO Max | Global | None | Aggregator + Deep Link |
| Peacock | US only | None | Aggregator + Deep Link |
| Paramount+ | Global | None | Aggregator + Deep Link |

**Tier 2: Regional Platforms**

| Platform | Region | API Status | Integration Method |
|----------|--------|------------|-------------------|
| Crave | Canada | None | Aggregator + Deep Link |
| BBC iPlayer | UK | None | Aggregator + Deep Link |

**Functional Requirements:**

```yaml
FR-1.1.1: SVOD Platform Integration
  requirement: System MUST support content discovery for 8+ SVOD platforms
  method: Third-party aggregator APIs (JustWatch, Watchmode, Streaming Availability)
  rationale: No direct API access available for major platforms
  priority: CRITICAL

FR-1.1.2: Deep Link Support
  requirement: System MUST support iOS Universal Links and Android App Links
  platforms:
    - Netflix (deep linking supported)
    - Prime Video (deep linking supported)
    - Disney+ (deep linking supported)
    - All major SVOD platforms
  priority: CRITICAL
```

#### 1.1.2 Ad-Supported Video-on-Demand (AVOD)

| Platform | API Status | Content Access |
|----------|-----------|----------------|
| YouTube | Public API available | Full API access |
| Pluto TV | Limited | Aggregator recommended |
| Tubi | Limited | Aggregator recommended |
| Freevee | Limited | Aggregator recommended |

**Functional Requirements:**

```yaml
FR-1.1.3: YouTube API Integration
  requirement: System MUST integrate YouTube Data API v3 with OAuth 2.0
  authentication: OAuth 2.0 with Authorization Code + PKCE
  scopes:
    - youtube.readonly (view account data)
    - youtube.force-ssl (manage account)
  device_support: Device Authorization Grant (RFC 8628) for TV/CLI
  priority: HIGH

FR-1.1.4: AVOD Aggregation
  requirement: System SHOULD aggregate free streaming platforms
  platforms: Pluto TV, Tubi, Freevee, Crackle
  method: Aggregator APIs
  priority: MEDIUM
```

### 1.2 Audio Streaming Platforms

**Functional Requirements:**

```yaml
FR-1.2.1: Audio Platform Support (Future)
  requirement: System MAY support audio streaming platforms
  platforms:
    - Spotify (public API available)
    - Apple Music (MusicKit API)
    - Podcast platforms
  priority: LOW (Phase 2)
  notes: Focus on video platforms for initial release
```

### 1.3 Social Media Platforms

**Functional Requirements:**

```yaml
FR-1.3.1: Social Media Integration (Future)
  requirement: System MAY support social video platforms
  platforms:
    - YouTube (primary video platform)
    - Twitch (streaming API available)
    - Instagram (limited API)
    - TikTok (limited API)
  priority: LOW (Phase 2)
```

### 1.4 Custom RTMP Endpoints

**Functional Requirements:**

```yaml
FR-1.4.1: Custom RTMP Support
  requirement: System SHALL NOT implement multistreaming/RTMP in v1.0
  rationale: Media Gateway is content discovery, not streaming output
  priority: OUT OF SCOPE
  notes: RTMP multistreaming is a separate product category
```

---

## 2. Platform Interaction Patterns

### 2.1 OAuth 2.0 Authentication Flows

#### 2.1.1 Authorization Code + PKCE (Web/Mobile)

**Flow Specification:**

```yaml
OAuth_Flow_1: Authorization Code with PKCE
  grant_type: authorization_code
  pkce: REQUIRED
  platforms:
    - YouTube (OAuth 2.0 documented)
    - Internal Media Gateway authentication

  steps:
    1_client_initiates:
      action: Generate code_verifier (43-128 chars, cryptographically random)
      generate: code_challenge = BASE64URL(SHA256(code_verifier))
      method: code_challenge_method = S256

    2_authorization_request:
      endpoint: https://accounts.google.com/o/oauth2/v2/auth
      parameters:
        - client_id
        - redirect_uri
        - response_type: code
        - scope: youtube.readonly youtube.force-ssl
        - state: RANDOM_STATE_TOKEN
        - code_challenge: <generated>
        - code_challenge_method: S256

    3_user_consent:
      action: User authenticates and grants permissions
      redirect: redirect_uri?code=AUTH_CODE&state=STATE_TOKEN

    4_token_exchange:
      endpoint: https://oauth2.googleapis.com/token
      method: POST
      body:
        - client_id
        - client_secret: OPTIONAL for public clients
        - code: AUTH_CODE
        - code_verifier: <original_verifier>
        - grant_type: authorization_code
        - redirect_uri

      response:
        - access_token: SHORT_LIVED (15-60 minutes)
        - refresh_token: LONG_LIVED (rotate on use)
        - expires_in: 3600
        - token_type: Bearer
        - scope: <granted_scopes>

FR-2.1.1: OAuth PKCE Implementation
  requirement: System MUST implement PKCE for all OAuth flows
  standard: RFC 7636
  security: Prevents authorization code interception attacks
  priority: CRITICAL
```

#### 2.1.2 Device Authorization Grant (TV/CLI)

**Flow Specification:**

```yaml
OAuth_Flow_2: Device Authorization Grant (RFC 8628)
  use_case: Input-constrained devices (smart TVs, streaming sticks, CLI)
  platforms:
    - YouTube TV integration
    - Media Gateway CLI authentication

  steps:
    1_device_code_request:
      endpoint: https://oauth2.googleapis.com/device/code
      method: POST
      body:
        - client_id
        - scope: youtube.readonly

      response:
        - device_code: OPAQUE_CODE (device polls with this)
        - user_code: SHORT_CODE (user enters on phone/browser)
        - verification_url: https://www.google.com/device
        - verification_url_complete: OPTIONAL (pre-filled URL with code)
        - expires_in: 1800 (30 minutes)
        - interval: 5 (polling interval in seconds)

    2_user_authorization:
      display: Show user_code and verification_url on TV/CLI
      action: User navigates to URL on phone/computer
      flow: User enters user_code and authenticates

    3_device_polling:
      endpoint: https://oauth2.googleapis.com/token
      method: POST
      interval: Every 5 seconds (or as specified)
      body:
        - client_id
        - device_code
        - grant_type: urn:ietf:params:oauth:grant-type:device_code

      responses:
        pending:
          error: authorization_pending
          action: Continue polling

        slow_down:
          error: slow_down
          action: Increase polling interval by 5 seconds

        success:
          access_token: <token>
          refresh_token: <token>
          expires_in: 3600
          token_type: Bearer

        denied:
          error: access_denied
          action: User denied request

FR-2.1.2: Device Grant Implementation
  requirement: System MUST support Device Authorization Grant for CLI
  standard: RFC 8628
  platforms: YouTube integration, internal auth
  priority: HIGH
```

### 2.2 API Key Management

**Functional Requirements:**

```yaml
FR-2.2.1: API Key Storage
  requirement: System MUST store API keys securely
  storage: Google Cloud Secret Manager
  encryption: AES-256 at rest, TLS 1.3 in transit
  rotation: Automatic key rotation every 90 days
  priority: CRITICAL

FR-2.2.2: API Key Types
  youtube_api_key:
    purpose: General API access (non-user-specific)
    storage: Secret Manager
    usage: Server-side requests only

  aggregator_api_keys:
    providers:
      - JustWatch (if API key required)
      - Watchmode API
      - Streaming Availability API
    storage: Secret Manager
    rotation: Per provider policy

  third_party_tokens:
    oauth_tokens:
      storage: Encrypted database (Cloud SQL)
      per_user: true
      refresh_strategy: Automatic before expiration

FR-2.2.3: Netflix Backlot API (Partner-Only)
  requirement: System SHOULD NOT attempt Netflix Backlot integration
  rationale: Requires content partnership, not applicable for discovery platform
  priority: OUT OF SCOPE
```

### 2.3 Multistreaming Coordination

**Functional Requirements:**

```yaml
FR-2.3.1: Multistreaming NOT IN SCOPE
  requirement: Media Gateway SHALL NOT implement multistreaming
  rationale: This is a content discovery system, not streaming output
  platforms: N/A
  priority: OUT OF SCOPE

  alternatives: Users directed to dedicated multistreaming tools
    - Restream.io
    - StreamYard
    - OBS with RTMP outputs
```

### 2.4 Platform-Specific Metadata Requirements

**Functional Requirements:**

```yaml
FR-2.4.1: Unified Metadata Schema
  requirement: System MUST normalize platform-specific metadata

  standard_identifiers:
    - EIDR: Entertainment Identifier Registry (preferred)
    - Gracenote TMS ID: Rich metadata source
    - TMDb ID: Community database
    - IMDb ID: User-familiar ratings

  normalization_rules:
    input: Platform-specific API responses
    process: mg-metadata-normalizer service
    output: Unified MediaContent schema

  schema_fields:
    required:
      - id (internal UUID)
      - title
      - mediaType (movie|tv)
      - overview
      - releaseDate
      - posterPath
      - genreIds

    optional:
      - eidr_id
      - tmdb_id
      - imdb_id
      - gracenote_id
      - runtime
      - voteAverage
      - popularity
      - backdropPath

FR-2.4.2: Cross-Reference Identifier Mapping
  requirement: System MUST cross-reference multiple identifier systems

  mapping_service: mg-entity-resolver
  strategy:
    1: Query aggregator API for content
    2: Extract provided IDs (TMDb, IMDb)
    3: Query EIDR registry if available
    4: Store all ID mappings in metadata fabric
    5: Use mappings to enrich future queries

  priority: HIGH
```

---

## 3. Ingestion Behavior Specifications

### 3.1 Input Source Types

**Media Gateway is NOT a streaming ingestion system. This section clarifies scope.**

```yaml
FR-3.1.1: Content Metadata Ingestion (IN SCOPE)
  requirement: System MUST ingest content metadata from aggregator APIs

  sources:
    - Streaming Availability API (60+ countries)
    - Watchmode API (200+ services, 50+ countries)
    - International Showtimes API (100+ markets)
    - YouTube Data API v3 (direct)

  data_types:
    - Content titles, descriptions, cast
    - Genre classifications
    - Availability by platform and region
    - Deep link URLs
    - Pricing (subscription, rental, purchase)
    - Expiry dates for time-limited content

  frequency:
    - Real-time: YouTube API queries
    - Batch: Daily catalog updates from aggregators
    - Event-driven: Content availability changes via webhooks

  priority: CRITICAL

FR-3.1.2: Video/Audio Stream Ingestion (OUT OF SCOPE)
  requirement: System SHALL NOT ingest video/audio streams
  rationale: Media Gateway is discovery, not encoding/transcoding
  alternatives: Direct users to platform apps for viewing
  priority: OUT OF SCOPE
```

### 3.2 Encoding Requirements (Not Applicable)

```yaml
FR-3.2.1: Video Encoding NOT IN SCOPE
  requirement: Media Gateway does not encode video
  rationale: Users watch on platform apps, not Media Gateway
  priority: OUT OF SCOPE
```

### 3.3 Bitrate Adaptation (Not Applicable)

```yaml
FR-3.3.1: Bitrate Adaptation NOT IN SCOPE
  requirement: Media Gateway does not transcode or adapt streams
  priority: OUT OF SCOPE
```

### 3.4 Resolution Handling (Not Applicable)

```yaml
FR-3.4.1: Resolution Handling NOT IN SCOPE
  requirement: Media Gateway does not process video resolutions
  priority: OUT OF SCOPE
```

---

## 4. Output Management

### 4.1 Content Discovery Output

**Functional Requirements:**

```yaml
FR-4.1.1: Unified Search API
  requirement: System MUST provide unified search across platforms

  endpoint: GET /api/search

  input_parameters:
    - query: string (required)
    - mediaType: movie|tv|all (default: all)
    - genres: number[] (genre IDs)
    - yearRange: {min, max}
    - ratingMin: number (0-10)
    - page: number (pagination)

  output_schema:
    success: boolean
    results: SearchResult[]
      - content: MediaContent
      - relevanceScore: number
      - matchReasons: string[]
      - similarityScore: number
    totalPages: number
    totalResults: number

  priority: CRITICAL

FR-4.1.2: Platform Availability Display
  requirement: System MUST show which platforms have content

  output_format:
    contentId: UUID
    title: string
    availability:
      - platform: Netflix
        region: US
        type: subscription
        deepLink: netflix://title/12345
        expiresAt: 2025-06-15T00:00:00Z

      - platform: Prime Video
        region: US
        type: rental
        price: 3.99
        currency: USD
        deepLink: primevideo://detail/67890

  priority: CRITICAL
```

### 4.2 Deep Linking Output

**Functional Requirements:**

```yaml
FR-4.2.1: iOS Universal Links
  requirement: System MUST support iOS Universal Links

  configuration:
    file: .well-known/apple-app-site-association
    location: https://media-gateway.example.com/.well-known/
    format: JSON (no .json extension)

    content:
      applinks:
        apps: []
        details:
          - appID: TEAM_ID.com.example.medigateway
            paths:
              - /watch/*
              - /content/*
              - /movie/*
              - /tv/*

    behavior:
      app_installed: Open Media Gateway app directly
      app_not_installed: Fallback to web app or App Store

  handoff_to_platform:
    user_selects: "Watch on Netflix"
    action: Open netflix://title/12345
    fallback: https://netflix.com/title/12345

  priority: HIGH

FR-4.2.2: Android App Links
  requirement: System MUST support Android App Links

  configuration:
    file: .well-known/assetlinks.json
    location: https://media-gateway.example.com/.well-known/
    format: JSON

    content:
      - relation: ["delegate_permission/common.handle_all_urls"]
        target:
          namespace: android_app
          package_name: com.example.medigateway
          sha256_cert_fingerprints: [CERT_HASH]

    manifest_intent_filter:
      - action: android.intent.action.VIEW
        category: android.intent.category.DEFAULT
        category: android.intent.category.BROWSABLE
        data:
          scheme: https
          host: media-gateway.example.com
          pathPrefix: /watch

  priority: HIGH

FR-4.2.3: Deep Link Reliability
  requirement: System SHOULD implement fallback strategies

  challenges:
    - Email/marketing link wrappers break Universal Links
    - Inconsistent platform support
    - OS versions affect behavior

  mitigation:
    - Use direct links in-app (not wrapped)
    - Test each platform individually
    - Provide manual "Open in App" buttons
    - Monitor deep link success rates
    - Clear user messaging on failures

  priority: MEDIUM
```

### 4.3 Recommendation Output

**Functional Requirements:**

```yaml
FR-4.3.1: Hybrid Recommendation Engine
  requirement: System MUST provide personalized recommendations

  algorithms:
    - Collaborative filtering (user-user, item-item)
    - Content-based filtering (genre, cast, themes)
    - Graph Neural Networks (GraphSAGE on Ruvector hypergraph)
    - SONA Intelligence (Two-Tier LoRA personalization)

  input:
    - User watch history (on-device, privacy-safe)
    - Genre preferences
    - Liked/disliked content
    - Aggregated patterns (differential privacy)

  output:
    contentId: UUID
    score: number (0-1 relevance)
    reasons: string[]
      - "Based on your love of sci-fi"
      - "Similar to The Matrix"
      - "Trending in your region"
    basedOn:
      type: similar|genre|history|trending
      references: [contentId1, contentId2]

  priority: CRITICAL

FR-4.3.2: Real-Time Personalization
  requirement: System SHOULD adapt recommendations in real-time

  mechanism: SONA Two-Tier LoRA
    - Per-user LoRA adapters (~10KB each)
    - Runtime adaptation without model retraining
    - EWC++ prevents catastrophic forgetting

  latency_target: <100ms for recommendation generation

  priority: HIGH
```

### 4.4 Cross-Platform Synchronization Output

**Functional Requirements:**

```yaml
FR-4.4.1: Watchlist Sync
  requirement: System MUST sync watchlists across devices

  technology: PubNub real-time messaging

  channel_topology:
    - user.{userId}.sync (watchlist, preferences)
    - user.{userId}.devices (device presence)

  data_synchronized:
    - Watchlist additions/removals
    - Watch progress (per-device tracking)
    - Preference updates (genre likes/dislikes)
    - Search history (optional, user-controlled)

  CRDT_strategy:
    type: Conflict-free Replicated Data Type
    implementation: mg-sync-engine
    benefits: Eventually consistent without conflicts

  priority: HIGH

FR-4.4.2: Privacy-Safe Sync
  requirement: System MUST protect user privacy during sync

  data_tiers:
    tier_1_on_device:
      - Detailed viewing history
      - Exact timestamps
      - Watch progress percentages
      - Never leaves device

    tier_2_federated:
      - Model updates (differential privacy)
      - Aggregated with other users
      - No individual identification

    tier_3_server:
      - Anonymized patterns only
      - Genre popularity trends
      - No user-specific data

  compliance: GDPR, CCPA, VPPA

  priority: CRITICAL
```

---

## 5. Regional Content Handling

### 5.1 Geographic Rights Management

**Functional Requirements:**

```yaml
FR-5.1.1: Multi-Region Content Availability
  requirement: System MUST handle country-specific catalogs

  detection:
    method: IP-based geolocation
    fallback: User manual region selection
    library: MaxMind GeoIP2 or similar

  catalog_filtering:
    - Query aggregator API with country code
    - Filter results to user's region
    - Display "Not available in your region" when applicable
    - Show pricing in local currency

  supported_regions:
    tier_1: US, UK, Canada, Australia, Germany, France, Japan
    tier_2: 40+ additional countries via Watchmode API

  priority: CRITICAL

FR-5.1.2: Content Expiry Tracking
  requirement: System MUST track time-limited content availability

  data_source: Streaming Availability API expiry dates (Unix timestamps)

  notifications:
    - "Leaving Netflix on June 15"
    - "Added to Hulu today"
    - "Coming to Disney+ next week"

  storage:
    table: content_availability
    fields:
      - contentId UUID
      - platform VARCHAR
      - region VARCHAR
      - availableFrom TIMESTAMP
      - expiresAt TIMESTAMP (nullable)
      - type ENUM(subscription, rental, purchase, free)

  priority: MEDIUM
```

### 5.2 Licensing Metadata

**Functional Requirements:**

```yaml
FR-5.2.1: Rights Aggregation
  requirement: System SHOULD aggregate licensing information

  data_points:
    - Which platforms have content per region
    - Pricing variations by market
    - Windowing (different release dates)
    - Service availability (not all platforms in all countries)

  use_cases:
    - Show users best value for content
    - "Available on 3 platforms in your region"
    - Price comparison across platforms

  priority: MEDIUM

FR-5.2.2: Platform Coverage by Region
  requirement: System MUST track platform regional availability

  examples:
    - Hulu: US only
    - Crave: Canada only
    - BBC iPlayer: UK only
    - Netflix: Global (varied catalogs)

  implementation:
    service: mg-rights-engine
    data: Platform → Region mapping
    updates: Monthly review of platform expansion

  priority: HIGH
```

---

## 6. Privacy and Compliance

### 6.1 GDPR/CCPA Compliance

**Functional Requirements:**

```yaml
FR-6.1.1: Explicit Consent Management
  requirement: System MUST obtain explicit consent before data collection

  consent_types:
    - Analytics cookies (optional)
    - Personalization (optional, but recommended for best experience)
    - Cross-device sync (optional)
    - Watch history storage (optional, on-device by default)

  consent_UI:
    - Clear, plain language explanations
    - Granular controls (not all-or-nothing)
    - Easy to change preferences later
    - No dark patterns

  platform: OneTrust or CookieYes consent management

  priority: CRITICAL

FR-6.1.2: User Data Rights
  requirement: System MUST support user data rights

  rights:
    right_to_access:
      endpoint: GET /api/user/data
      response: JSON export of all user data
      timeline: Within 30 days (GDPR requirement)

    right_to_deletion:
      endpoint: DELETE /api/user/account
      action: Permanent deletion of all user data
      exceptions: Legal hold data only
      timeline: Within 30 days

    right_to_opt_out:
      endpoint: POST /api/user/opt-out
      action: Stop data collection, delete existing data
      process: Minimal steps (single click, not multi-step cookie preferences)
      timeline: Immediate

    right_to_portability:
      endpoint: GET /api/user/export
      format: JSON (machine-readable)
      includes: Watchlist, preferences, history

  priority: CRITICAL

FR-6.1.3: Privacy Policy Transparency
  requirement: System MUST provide clear privacy policy

  disclosures:
    - What data is collected (specific fields)
    - Why data is collected (specific purposes)
    - How long data is retained (specific timelines)
    - Who data is shared with (specific third parties)
    - User rights and how to exercise them

  language: Plain language, not legalese
  accessibility: Linked from all pages, easy to find
  updates: Notify users of material changes

  priority: CRITICAL
```

### 6.2 VPPA Compliance (Video Privacy Protection Act)

**Functional Requirements:**

```yaml
FR-6.2.1: Video Viewing Data Consent
  requirement: System MUST obtain consent for video viewing data

  scope:
    - Applies to watch history
    - Applies to embedded video players (if any)
    - Applies to social media tracking pixels

  consent_requirements:
    - Clear, informed consent (not buried in TOS)
    - Separate from general consent
    - User can opt-out easily
    - Applied to third-party sharing

  enforcement: California AG active enforcement (2025 settlements)

  priority: CRITICAL

FR-6.2.2: Third-Party Pixel Tracking
  requirement: System SHOULD minimize third-party tracking

  avoid:
    - Meta Pixel for video tracking (VPPA risk)
    - Embedded video players that share data

  alternatives:
    - First-party analytics only
    - Anonymized aggregate data
    - Differential privacy for ML training

  priority: HIGH
```

---

## 7. Security Architecture

### 7.1 OAuth 2.0 Security (RFC 9700 Best Practices)

**Functional Requirements:**

```yaml
FR-7.1.1: Token Security
  requirement: System MUST implement token security best practices

  access_tokens:
    lifetime: 15-60 minutes (short-lived)
    storage: Never in localStorage (XSS risk)
    transmission: HTTPS only, Authorization header
    constraints: Sender-constrained via mTLS or DPoP (high security scenarios)

  refresh_tokens:
    lifetime: 90 days (rotate on use)
    rotation: Issue new refresh token with each use
    revocation: Detect and revoke compromised tokens
    storage: Encrypted database, never client-side

  priority: CRITICAL

FR-7.1.2: Deprecated Flows (Do Not Use)
  requirement: System MUST NOT use deprecated OAuth flows

  forbidden:
    - Implicit Grant (officially deprecated)
    - Resource Owner Password Credentials (ROPC)

  reason: Security vulnerabilities, better alternatives exist

  priority: CRITICAL

FR-7.1.3: Scope and Audience Restriction
  requirement: System MUST restrict token privileges

  principle_of_least_privilege:
    - Request minimum required scopes
    - Audience restriction (single Resource Server per token)
    - Prevents token misuse across services

  example:
    scope: youtube.readonly (not youtube.force-ssl unless needed)
    audience: https://www.googleapis.com/auth/youtube

  priority: HIGH
```

### 7.2 Rate Limiting

**Functional Requirements:**

```yaml
FR-7.2.1: API Rate Limiting
  requirement: System MUST implement rate limiting

  limits:
    per_user: 100 requests/minute
    per_ip: 1000 requests/minute
    per_api_key: 10000 requests/day

  algorithms:
    - Token bucket
    - Sliding window
    - Exponential backoff for retries

  responses:
    status: 429 Too Many Requests
    headers:
      - X-RateLimit-Limit
      - X-RateLimit-Remaining
      - X-RateLimit-Reset
      - Retry-After

  abuse_prevention:
    - CAPTCHA on endpoints prone to abuse
    - Temporary IP bans for repeated violations

  priority: HIGH
```

### 7.3 Multi-Factor Authentication

**Functional Requirements:**

```yaml
FR-7.3.1: MFA for User Accounts
  requirement: System SHOULD support MFA for user accounts

  methods:
    - TOTP (Google Authenticator, Authy)
    - SMS (fallback, less secure)
    - Email (fallback)
    - Hardware keys (FIDO2/WebAuthn)

  enforcement:
    - Optional for regular users
    - Required for admin accounts
    - Required for payment/subscription changes

  priority: MEDIUM
```

---

## 8. Third-Party Integrations

### 8.1 Aggregator API Selection

**Functional Requirements:**

```yaml
FR-8.1.1: Primary Aggregator API
  requirement: System MUST integrate at least one aggregator API

  recommended: Streaming Availability API

  evaluation_criteria:
    - coverage: 60+ countries, 150+ platforms
    - metadata: Deep links, expiry dates, video qualities
    - pricing: Reasonable cost per request
    - reliability: Uptime SLA
    - data_freshness: Update frequency

  alternatives:
    - Watchmode API (200+ services, 50+ countries)
    - International Showtimes API (100+ markets)

  priority: CRITICAL

FR-8.1.2: Aggregator API Caching
  requirement: System MUST cache aggregator API responses

  strategy:
    cache_layer: Memorystore (Valkey/Redis)
    ttl_content_metadata: 24 hours (content doesn't change frequently)
    ttl_availability: 6 hours (availability changes more often)
    ttl_pricing: 1 hour (pricing can fluctuate)

  invalidation:
    - Webhook from aggregator (if available)
    - Manual purge via admin API
    - TTL expiration

  benefits:
    - Reduced API costs
    - Faster response times
    - Resilience to API downtime

  priority: HIGH
```

### 8.2 Metadata Standards

**Functional Requirements:**

```yaml
FR-8.2.1: EIDR Integration
  requirement: System SHOULD support EIDR identifiers

  purpose: Entertainment Identifier Registry (industry standard)

  usage:
    - Cross-reference with aggregator APIs
    - Deduplicate content across platforms
    - Hierarchical relationships (series → season → episode)

  adoption: Expanding but not universal

  priority: MEDIUM

FR-8.2.2: Gracenote/TMS IDs
  requirement: System SHOULD support Gracenote TMS IDs

  purpose: Rich metadata for search/discovery

  data_provided:
    - Theme, genre, mood taxonomies
    - Keywords for semantic search
    - Similar title recommendations
    - Nielsen ratings

  coverage: 85+ countries, major platforms

  priority: MEDIUM

FR-8.2.3: TMDb/IMDb IDs
  requirement: System MUST support TMDb and IMDb IDs

  purpose: Community databases with broad coverage

  TMDb:
    - Community-built database
    - Free API with rate limits
    - Used by many aggregators

  IMDb:
    - User-familiar ratings
    - Extensive cast/crew data
    - No official API (use aggregators)

  priority: HIGH
```

---

## 9. Implementation Priorities

### 9.1 Phase 1: Foundation (Months 1-3)

```yaml
P1_Critical_Requirements:
  - FR-1.1.1: SVOD Platform Integration via aggregators
  - FR-1.1.2: Deep Link Support (iOS Universal Links, Android App Links)
  - FR-2.1.1: OAuth PKCE Implementation
  - FR-2.4.1: Unified Metadata Schema
  - FR-4.1.1: Unified Search API
  - FR-4.1.2: Platform Availability Display
  - FR-5.1.1: Multi-Region Content Availability
  - FR-6.1.1: Explicit Consent Management
  - FR-6.1.2: User Data Rights (access, deletion, opt-out)
  - FR-7.1.1: Token Security (short-lived, rotation)
  - FR-8.1.1: Primary Aggregator API Integration
  - FR-8.2.3: TMDb/IMDb ID Support

P1_High_Requirements:
  - FR-1.1.3: YouTube API Integration with OAuth 2.0
  - FR-2.1.2: Device Grant Implementation for CLI
  - FR-4.2.1: iOS Universal Links
  - FR-4.2.2: Android App Links
  - FR-4.3.1: Hybrid Recommendation Engine
  - FR-4.3.2: Real-Time Personalization (SONA)
  - FR-4.4.1: Watchlist Sync via PubNub
  - FR-5.1.2: Content Expiry Tracking
  - FR-6.2.1: VPPA Compliance
  - FR-7.1.3: Scope and Audience Restriction
  - FR-7.2.1: API Rate Limiting
  - FR-8.1.2: Aggregator API Caching
```

### 9.2 Phase 2: Enhancement (Months 4-6)

```yaml
P2_Medium_Requirements:
  - FR-1.1.4: AVOD Aggregation (Pluto, Tubi, Freevee)
  - FR-4.2.3: Deep Link Reliability improvements
  - FR-5.2.1: Rights Aggregation and pricing comparison
  - FR-5.2.2: Platform Coverage by Region tracking
  - FR-7.3.1: MFA for User Accounts
  - FR-8.2.1: EIDR Integration
  - FR-8.2.2: Gracenote/TMS IDs

P2_Low_Requirements:
  - FR-1.2.1: Audio Platform Support (Spotify, Apple Music)
  - FR-1.3.1: Social Media Integration (Twitch, etc.)
```

### 9.3 Out of Scope

```yaml
Out_Of_Scope:
  - FR-1.4.1: Custom RTMP Support (multistreaming)
  - FR-3.1.2: Video/Audio Stream Ingestion
  - FR-3.2.1: Video Encoding
  - FR-3.3.1: Bitrate Adaptation
  - FR-3.4.1: Resolution Handling
  - FR-2.2.3: Netflix Backlot API
  - FR-2.3.1: Multistreaming Coordination

Rationale:
  Media Gateway is a content discovery platform, not a streaming/transcoding engine.
  Users watch content on platform apps, accessed via deep links.
```

---

## 10. Testing and Validation

### 10.1 Integration Testing

**Test Requirements:**

```yaml
T-10.1.1: Aggregator API Testing
  test: Verify Streaming Availability API integration
  assertions:
    - Content search returns results
    - Platform availability is accurate
    - Deep links are valid
    - Regional filtering works
  frequency: Daily (automated)
  priority: CRITICAL

T-10.1.2: Deep Link Testing
  test: Verify Universal Links and App Links
  platforms:
    - iOS 16+ (Universal Links)
    - Android 12+ (App Links)
  scenarios:
    - App installed → Opens directly to content
    - App not installed → Fallback to web/store
    - Email link wrappers → Manual override needed
  frequency: Weekly (manual)
  priority: HIGH
```

### 10.2 Security Testing

**Test Requirements:**

```yaml
T-10.2.1: OAuth Flow Security
  test: Verify PKCE implementation
  attacks_to_prevent:
    - Authorization code interception
    - Token replay attacks
    - Cross-site request forgery (CSRF)
  tools: OWASP ZAP, Burp Suite
  frequency: Quarterly
  priority: CRITICAL

T-10.2.2: Privacy Compliance Audit
  test: Verify GDPR/CCPA/VPPA compliance
  checks:
    - Consent flows are clear
    - Data deletion works
    - Opt-out is single-click
    - Privacy policy is accurate
  auditor: Third-party legal review
  frequency: Annually
  priority: CRITICAL
```

---

## 11. Appendices

### Appendix A: Platform Deep Link Schemas

```yaml
Netflix: netflix://title/{id}
Prime Video: primevideo://detail/{id}
Disney+: disneyplus://content/{id}
Hulu: hulu://watch/{id}
Apple TV+: https://tv.apple.com/show/{id}
YouTube: youtube://watch?v={id}
HBO Max: hbomax://feature/{id}
Peacock: peacock://content/{id}
Paramount+: paramountplus://content/{id}
```

### Appendix B: Aggregator API Comparison

| Feature | Streaming Availability | Watchmode | International Showtimes |
|---------|----------------------|-----------|------------------------|
| Countries | 60+ | 50+ | 100+ |
| Platforms | 150+ | 200+ | 100+ |
| Deep Links | Yes | Yes | Yes |
| Expiry Dates | Yes (Unix timestamps) | Limited | Limited |
| Pricing | Yes | Yes | Yes |
| Episode-Level | Yes | Yes (Tier 1 countries) | Limited |
| API Cost | Pay-per-request | Tiered pricing | Enterprise |

### Appendix C: OAuth 2.0 Endpoints

**YouTube:**
```yaml
Authorization: https://accounts.google.com/o/oauth2/v2/auth
Token: https://oauth2.googleapis.com/token
Device Code: https://oauth2.googleapis.com/device/code
Revocation: https://oauth2.googleapis.com/revoke
```

**Media Gateway (Internal):**
```yaml
Authorization: https://api.media-gateway.example.com/oauth/authorize
Token: https://api.media-gateway.example.com/oauth/token
Device Code: https://api.media-gateway.example.com/oauth/device
Revocation: https://api.media-gateway.example.com/oauth/revoke
```

---

## 12. References

### Research Sources

1. **Streaming Platform Research:** `/tmp/media-gateway-research/research/streaming-platform-research.md`
2. **Architecture Blueprint:** `/tmp/media-gateway-research/research/FINAL_ARCHITECTURE_BLUEPRINT.md`
3. **hackathon-tv5 Integration:** `/tmp/hackathon-tv5` repository
4. **TMDB Implementation:** `/tmp/hackathon-tv5/apps/media-discovery/src/lib/tmdb.ts`

### Standards and RFCs

- RFC 6749: OAuth 2.0 Authorization Framework
- RFC 7636: Proof Key for Code Exchange (PKCE)
- RFC 8628: Device Authorization Grant
- RFC 9700: OAuth 2.0 Security Best Current Practice (January 2025)

### Privacy Regulations

- GDPR (General Data Protection Regulation)
- CCPA (California Consumer Privacy Act)
- VPPA (Video Privacy Protection Act)
- 20+ US State Privacy Laws

---

**Document Control**

- **Author:** Research and Analysis Agent
- **Reviewers:** [Pending]
- **Approval:** [Pending]
- **Next Review:** 2025-Q2

**Change Log**

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 1.0.0 | 2025-12-06 | Initial specification | Research Agent |

---

END OF SPECIFICATION
