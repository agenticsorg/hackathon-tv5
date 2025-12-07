# SPARC Refinement — Part 2: Acceptance Criteria

**Document Version:** 1.0.0
**SPARC Phase:** Refinement
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents

1. [Acceptance Criteria Framework](#1-acceptance-criteria-framework)
2. [Auth Service Acceptance Criteria](#2-auth-service-acceptance-criteria)
3. [Content Service Acceptance Criteria](#3-content-service-acceptance-criteria)
4. [Search Service Acceptance Criteria](#4-search-service-acceptance-criteria)
5. [SONA Recommendation Service Acceptance Criteria](#5-sona-recommendation-service-acceptance-criteria)
6. [Sync Service Acceptance Criteria](#6-sync-service-acceptance-criteria)
7. [Playback Service Acceptance Criteria](#7-playback-service-acceptance-criteria)
8. [MCP Service Acceptance Criteria](#8-mcp-service-acceptance-criteria)
9. [API Gateway Acceptance Criteria](#9-api-gateway-acceptance-criteria)
10. [Integration Acceptance Criteria](#10-integration-acceptance-criteria)

---

## 1. Acceptance Criteria Framework

### 1.1 Acceptance Criteria Structure

All acceptance criteria follow the **Given-When-Then** (GWT) format with measurable success metrics:

```gherkin
Feature: [Feature Name]

Scenario: [Scenario Description]
  Given [initial context/preconditions]
  And [additional preconditions]
  When [action/event occurs]
  And [additional actions]
  Then [expected outcome]
  And [additional outcomes]

  # Success Metrics
  Metrics:
    - [Quantifiable metric]: [target value]
    - [Performance metric]: [latency target]

  # Edge Cases
  Edge Cases:
    - [Edge case description]
    - [Expected behavior]

  # Failure Scenarios
  Failures:
    - [Failure condition]: [expected error handling]
```

### 1.2 Performance Metrics Definitions

| Metric | Definition | Target |
|--------|------------|--------|
| **p50 Latency** | 50th percentile response time | <100ms |
| **p95 Latency** | 95th percentile response time | <500ms |
| **p99 Latency** | 99th percentile response time | <1000ms |
| **Availability** | Uptime percentage | 99.9% (Tier 1), 99.5% (Tier 2) |
| **Throughput** | Requests per second | 1000+ RPS per service |
| **Error Rate** | Percentage of failed requests | <0.1% |
| **Accuracy** | Correctness of results | >95% |

### 1.3 Test Coverage Requirements

| Test Type | Coverage Target | Tools |
|-----------|----------------|-------|
| **Unit Tests** | 80%+ line coverage | Jest, Rust `cargo test` |
| **Integration Tests** | All API endpoints | Supertest, reqwest |
| **E2E Tests** | Critical user flows | Playwright, Cypress |
| **Performance Tests** | All Tier 1 services | k6, wrk |
| **Security Tests** | OWASP Top 10 | ZAP, Burp Suite |

---

## 2. Auth Service Acceptance Criteria

### 2.1 OAuth 2.0 + PKCE Flow (Web/Mobile)

```gherkin
Feature: OAuth 2.0 Authorization with PKCE

Scenario: Successful OAuth login with Google
  Given a user with a valid Google account
  And the user is on the Media Gateway login page
  When the user clicks "Sign in with Google"
  And the client generates a code_verifier (128-bit random)
  And the client sends code_challenge = SHA256(code_verifier)
  And the user completes Google OAuth consent flow
  And Google redirects with authorization code
  And the client exchanges code + code_verifier for tokens
  Then the auth service validates code_verifier against code_challenge
  And the auth service issues JWT access token (15min expiry)
  And the auth service issues refresh token (7d expiry, httpOnly cookie)
  And the user is redirected to dashboard with session established
  And the session is stored in Redis with 7-day TTL

  Metrics:
    - OAuth flow completion time: <3 seconds
    - Token validation latency: <10ms
    - Session creation latency: <50ms

  Edge Cases:
    - User cancels OAuth consent → redirect to login with error message
    - Authorization code already used → return 400 "Code already redeemed"
    - code_verifier mismatch → return 401 "Invalid code verifier"
    - Expired authorization code (>10min) → return 401 "Code expired"

  Failures:
    - Google OAuth down: Display "Authentication unavailable, try again"
    - Redis unavailable: Return 503 "Service temporarily unavailable"
    - Invalid state parameter: Return 400 "Invalid state, possible CSRF"
```

### 2.2 Device Authorization Grant (TV/CLI)

```gherkin
Feature: Device Authorization Grant (RFC 8628)

Scenario: Successful TV device pairing
  Given a user with an existing Media Gateway account
  And the user launches the TV app (not logged in)
  When the TV app requests a device code from POST /auth/device
  Then the auth service generates 8-character device code (uppercase, no ambiguous chars)
  And the auth service stores code in Redis with 15-minute TTL
  And the auth service returns device_code, user_code, verification_uri
  And the TV displays:
    """
    Code: ABCD-1234
    Go to: mg.app/link
    Expires in: 15 minutes
    """
  And the TV starts polling GET /auth/device/poll every 5 seconds

  When the user opens mg.app/link on their phone
  And the user enters code "ABCD-1234"
  And the user confirms "Authorize Media Gateway TV?"
  Then the auth service marks device code as approved
  And the next poll request receives access + refresh tokens
  And the TV app stores tokens securely (keychain/secure storage)
  And the TV app loads user's personalized content
  And the device is registered in user's device list

  Metrics:
    - Device code generation: <50ms
    - Polling interval: 5 seconds ±0.5s
    - Total flow completion: <15 minutes (user-dependent)
    - Token delivery on approval: <100ms

  Edge Cases:
    - User enters invalid code → "Invalid code, please try again"
    - Code expires during entry → "Code expired, generate new code on TV"
    - Polling before approval → 428 "Authorization pending"
    - Polling after rejection → 403 "Authorization denied"
    - Max 180 poll attempts (15min / 5s) → "Timeout, generate new code"

  Failures:
    - Redis down during code generation: Return 503
    - User already has 10 devices: Return 429 "Device limit reached"
    - Network interruption during polling: Continue polling with exponential backoff
```

### 2.3 JWT Validation Performance

```gherkin
Feature: JWT Access Token Validation

Scenario: High-throughput token validation
  Given 1000 concurrent requests to protected endpoints
  And each request includes valid JWT in Authorization header
  When the API Gateway validates each JWT signature
  And the API Gateway checks token expiry (15min window)
  And the API Gateway extracts user_id and scopes
  Then all 1000 requests are validated successfully
  And validation latency p95 <10ms
  And no requests are incorrectly rejected
  And expired tokens (>15min) are rejected with 401

  Metrics:
    - Validation latency p50: <5ms
    - Validation latency p95: <10ms
    - Validation latency p99: <15ms
    - Throughput: 10,000+ validations/second per instance

  Edge Cases:
    - Token issued in future (clock skew) → allow ±30s tolerance
    - Token with invalid signature → reject immediately (no DB lookup)
    - Token with missing required claims → reject with 400
    - Token revoked (user logout) → check Redis revocation list (<5ms)

  Failures:
    - Malformed JWT: Return 400 "Invalid token format"
    - Expired token: Return 401 "Token expired, refresh required"
    - Invalid signature: Return 401 "Invalid token signature"
    - Public key rotation in progress: Accept both old and new keys (1h overlap)
```

### 2.4 Token Refresh Rotation

```gherkin
Feature: Refresh Token Rotation

Scenario: Secure token refresh with rotation
  Given a user with expired access token (>15min)
  And a valid refresh token (httpOnly cookie, <7d old)
  When the client sends POST /auth/refresh with refresh token
  Then the auth service validates refresh token signature
  And the auth service checks refresh token not in revocation list
  And the auth service issues NEW access token (15min expiry)
  And the auth service issues NEW refresh token (7d expiry)
  And the auth service adds OLD refresh token to revocation list
  And the auth service returns new tokens to client
  And the client updates stored tokens

  Metrics:
    - Refresh flow latency: <50ms p95
    - Revocation list lookup: <5ms (Redis)
    - Token generation: <10ms

  Edge Cases:
    - Refresh token already used (replay attack) → revoke entire token family
    - Refresh token expired (>7d) → require re-authentication
    - User changed password → revoke all refresh tokens
    - User logged out → add all tokens to revocation list

  Failures:
    - Refresh token in revocation list: Return 401 "Token revoked"
    - Redis unavailable: Allow refresh but skip revocation (fail-open)
    - Refresh token missing: Return 400 "Refresh token required"
```

### 2.5 Rate Limiting Enforcement

```gherkin
Feature: Authentication Rate Limiting

Scenario: Rate limit enforcement per tier
  Given a Free tier user (limit: 100 req/min)
  And a Pro tier user (limit: 1000 req/min)
  When the Free user makes 101 requests in 1 minute
  And the Pro user makes 1001 requests in 1 minute
  Then the Free user's 101st request receives 429 "Rate limit exceeded"
  And the response includes Retry-After: 45 (seconds until reset)
  And the Pro user's 1001st request receives 429
  And subsequent requests within same minute are also rate-limited
  And after 60 seconds, rate limits reset for both users

  Metrics:
    - Rate limit check latency: <2ms (Redis counter)
    - Counter increment atomicity: 100%
    - False negatives (allowed over-limit): 0%
    - False positives (blocked under-limit): 0%

  Edge Cases:
    - User upgrades tier mid-minute → apply new limit immediately
    - User makes burst of 50 requests in 1 second → count correctly
    - Distributed requests across multiple API Gateway instances → Redis atomic counters
    - Clock drift between servers → use Redis server time

  Failures:
    - Redis unavailable: Fail-open (no rate limiting) + alert
    - Network partition: Each API Gateway instance tracks local limits (degraded mode)
```

---

## 3. Content Service Acceptance Criteria

### 3.1 Content Ingestion Performance

```gherkin
Feature: High-Throughput Content Ingestion

Scenario: Ingest 1000 items per minute from external APIs
  Given ingestion service running hourly CronJob
  And rate limits: YouTube (10K/day), Streaming Availability (100/min), JustWatch (1000/hour)
  When the ingestion job starts at :00 minute
  And the service fetches from YouTube API (parallel, 5 API keys)
  And the service fetches from Streaming Availability API (parallel)
  And the service fetches from JustWatch API (parallel)
  Then the service processes 1000+ content items in 60 seconds
  And all items are normalized to CanonicalContent schema
  And entity resolution deduplicates cross-platform matches (>95% accuracy)
  And embeddings are generated for all items (768-dim vectors)
  And all items are upserted to PostgreSQL + Qdrant
  And ingestion completes without rate limit violations

  Metrics:
    - Ingestion throughput: 1000+ items/minute
    - Normalization latency: <50ms per item
    - Entity resolution accuracy: >95% (measured against manual labels)
    - Embedding generation: 500 items/second (batch)
    - Database upsert: <100ms per batch (100 items)

  Edge Cases:
    - API returns 429 rate limit → exponential backoff (1s, 2s, 4s, 8s, 16s)
    - API returns malformed data → skip item, log error, continue
    - Duplicate content across APIs → keep most complete record (field count)
    - Missing required fields → populate defaults, mark as incomplete

  Failures:
    - API completely unavailable: Skip API, log warning, continue with others
    - Database write failure: Retry 3x with backoff, then fail job (manual intervention)
    - Embedding service timeout: Skip embedding, mark for retry, continue
    - Circuit breaker opens (3 consecutive API failures): Stop calling that API for 60s
```

### 3.2 Entity Resolution Accuracy

```gherkin
Feature: Cross-Platform Entity Resolution

Scenario: Deduplicate "The Matrix" across platforms
  Given "The Matrix (1999)" exists in YouTube, Netflix, Prime Video
  And YouTube record has EIDR: 10.5240/1234-5678-90AB
  And Netflix record has title: "The Matrix", year: 1999, no EIDR
  And Prime Video record has title: "Matrix", year: 1999, IMDB: tt0133093
  When the entity resolver processes all three records
  Then the resolver matches Netflix record to YouTube via fuzzy title + exact year
  And the resolver matches Prime Video via IMDB lookup → EIDR mapping
  And the resolver creates single CanonicalContent entity
  And the entity has merged availability: [YouTube, Netflix, Prime Video]
  And the entity preserves best metadata from each source
  And the entity has primary identifier: EIDR 10.5240/1234-5678-90AB

  Metrics:
    - Entity resolution accuracy: >95% (precision and recall)
    - Resolution latency: <100ms per entity
    - False positives (incorrect matches): <1%
    - False negatives (missed matches): <5%

  Edge Cases:
    - Title variations: "Matrix" vs "The Matrix" → normalize titles
    - Year mismatches (±1 year): "1999" vs "2000" → allow 1-year tolerance
    - Remakes: "The Matrix (1999)" vs "The Matrix Resurrections (2021)" → separate entities
    - Director's cuts: Same EIDR → merge as single entity with variant metadata

  Failures:
    - No matching identifiers: Create separate entity (manual review queue)
    - Conflicting metadata: Prefer record with most complete data
    - EIDR lookup service down: Fall back to title + year + director matching
```

### 3.3 Metadata Normalization

```gherkin
Feature: Cross-Platform Metadata Normalization

Scenario: Normalize metadata from heterogeneous sources
  Given raw content from YouTube API:
    """json
    {
      "title": "Inception",
      "publishedAt": "2010-07-16T00:00:00Z",
      "contentDetails": { "duration": "PT2H28M" },
      "snippet": { "categoryId": "1" }
    }
    """
  And raw content from Netflix (via Streaming Availability):
    """json
    {
      "name": "Inception",
      "releaseDate": "2010-07-16",
      "runtime": 148,
      "genres": ["Sci-Fi", "Thriller"]
    }
    """
  When the normalizer processes both records
  Then the normalized CanonicalContent contains:
    """json
    {
      "entity_id": "eidr:10.5240/ABCD-EFGH-IJKL",
      "title": "Inception",
      "release_date": "2010-07-16",
      "runtime_minutes": 148,
      "genres": ["science_fiction", "thriller"],
      "platforms": [
        { "name": "YouTube", "availability": "rent", "price_usd": 3.99 },
        { "name": "Netflix", "availability": "subscription", "tier": "Standard" }
      ],
      "content_type": "movie",
      "metadata_completeness": 0.92
    }
    """
  And all genre names are normalized to lowercase with underscores
  And all dates are ISO 8601 format
  And all runtime values are in minutes (integer)

  Metrics:
    - Normalization latency: <50ms per record
    - Schema compliance: 100% (all records valid CanonicalContent)
    - Metadata completeness: >90% average

  Edge Cases:
    - Missing runtime: Estimate from duration range (e.g., "movie" → 90-120min)
    - Missing genre: Infer from content description (ML classifier, >80% accuracy)
    - Missing release date: Extract from title "(2010)" pattern
    - Conflicting values: Prefer most recent source (timestamp-based)

  Failures:
    - Required field missing (title): Skip record, log error
    - Invalid data type: Attempt coercion, else use null
    - Enum value not in schema: Map to "other" category, log for review
```

### 3.4 Image Caching and CDN Delivery

```gherkin
Feature: Content Image Caching

Scenario: Cache and serve images via CDN
  Given a content entity with poster_url from external source
    "https://platform.example.com/images/movie-123-poster.jpg"
  When the ingestion service downloads the image
  And the service validates image (JPEG/PNG, <10MB, >200px width)
  And the service uploads to Cloud Storage bucket (Standard class)
  And the service generates CDN URL: "https://cdn.mediagateway.io/images/eidr-123/poster.jpg"
  And the service updates CanonicalContent with CDN URL
  Then subsequent content requests return CDN URL
  And CDN serves image with cache headers: "max-age=31536000" (1 year)
  And image is served from edge location <50ms from user
  And original platform URL is not exposed to clients

  Metrics:
    - Image download latency: <500ms p95
    - CDN upload latency: <200ms p95
    - CDN hit rate: >95% after 24 hours
    - Image serving latency: <50ms p95 (from CDN edge)

  Edge Cases:
    - Image URL returns 404 → use placeholder image, mark for manual review
    - Image exceeds 10MB → skip caching, log warning
    - Image is < 200px width → skip caching, likely low quality
    - Image format is WebP/AVIF → convert to JPEG for compatibility

  Failures:
    - Cloud Storage upload fails: Retry 3x, then use original URL (fallback)
    - CDN unavailable: Serve directly from Cloud Storage (degraded)
    - Invalid image format: Use placeholder image
```

---

## 4. Search Service Acceptance Criteria

### 4.1 Search Latency Requirements

```gherkin
Feature: Sub-500ms Search Response

Scenario: Natural language search with hybrid strategy
  Given a user searches "scary movies like Stranger Things"
  And the search index contains 20M content entities
  And the user's subscription platforms: [Netflix, Hulu, Disney+]
  When the search service receives POST /api/search
  Then the service parses intent with GPT-4o-mini (<100ms)
  And the service generates query embedding 768-dim (<30ms)
  And the service performs vector search in Qdrant HNSW index (<50ms)
  And the service filters by user platforms (<20ms, PostgreSQL indexed)
  And the service ranks with SONA personalization (<100ms)
  And the service returns top 25 results
  And total latency p95 <400ms

  Metrics:
    - End-to-end latency p50: <250ms
    - End-to-end latency p95: <400ms
    - End-to-end latency p99: <500ms
    - Throughput: 1000+ searches/second (cluster-wide)

  Edge Cases:
    - Typo in query: "scry movies" → auto-correct to "scary movies"
    - Ambiguous query: "Matrix" → return all franchise entries
    - No results on user platforms → return "Available on [other platforms]"
    - Empty query → return trending/popular content

  Failures:
    - Qdrant unavailable: Fall back to PostgreSQL full-text search (slower, ~800ms)
    - SONA ranking timeout: Use default popularity ranking
    - Embedding service timeout: Use keyword-only search
    - All backends down: Return 503 with cached popular content
```

### 4.2 Hybrid Search Quality

```gherkin
Feature: Hybrid Vector + Keyword + Graph Search

Scenario: Combine multiple search strategies
  Given a query "Christopher Nolan time travel movies"
  When the search service executes:
    - Vector search: Embed query → find semantic neighbors
    - Keyword search: Match "Christopher Nolan" (director field)
    - Graph search: Traverse "directed_by" edges to related entities
  Then the service combines results with weighted scoring:
    - Vector similarity: 0.5 weight
    - Keyword exact match: 0.3 weight
    - Graph relationship strength: 0.2 weight
  And the service applies MMR (Maximal Marginal Relevance) for diversity
  And the top result is "Tenet (2020)" (Christopher Nolan, time inversion)
  And results include "Interstellar (2014)" and "Inception (2010)"
  And all results are from user's available platforms

  Metrics:
    - Precision@10: ≥0.31 (industry benchmark)
    - NDCG@10: ≥0.63 (normalized discounted cumulative gain)
    - Result diversity (intra-list similarity): <0.7
    - User click-through rate: >15% on top 3 results

  Edge Cases:
    - Query matches genre but not director → prioritize genre
    - Query matches title exactly → return exact match as #1
    - Query has conflicting signals → use context (time of day, device)

  Failures:
    - Vector search returns no results: Fall back to keyword only
    - Graph traversal times out: Skip graph component
    - Keyword search fails: Rely on vector + graph
```

### 4.3 Filter Application Correctness

```gherkin
Feature: Multi-Dimensional Filtering

Scenario: Apply genre, year, and platform filters
  Given a search for "action movies"
  And filters:
    - Genres: ["action", "thriller"]
    - Year range: 2015-2023
    - Platforms: ["Netflix", "HBO Max"]
    - Rating: PG-13 or R
  When the search service applies filters
  Then all returned results satisfy:
    - Genre contains "action" OR "thriller"
    - Release year between 2015-2023 (inclusive)
    - Available on Netflix OR HBO Max (in user's region)
    - Rating is "PG-13" OR "R"
  And results are ranked by relevance within filtered set
  And filter counts are returned: { "action": 127, "thriller": 89, "Netflix": 56 }

  Metrics:
    - Filter application latency: <20ms (indexed columns)
    - Filter accuracy: 100% (no false positives/negatives)
    - Filter count calculation: <10ms

  Edge Cases:
    - No results match all filters → relax least important filter
    - Filter values not in database → return empty results + suggestion
    - Conflicting filters (e.g., year: 2020-2025, but also "classic") → prioritize explicit

  Failures:
    - Invalid filter format: Return 400 with clear error message
    - Filter on unindexed column: Reject with "Filter not supported"
    - Too many filter combinations: Limit to 5 simultaneous filters
```

### 4.4 Result Relevance Explanations

```gherkin
Feature: Explainable Search Results

Scenario: Provide relevance explanation for each result
  Given a search "movies like Parasite"
  And the top result is "Snowpiercer (2013)"
  When the client requests explanation (include_explanation=true)
  Then the response includes:
    """json
    {
      "entity_id": "eidr:10.5240/SNOW-PIER-0001",
      "title": "Snowpiercer",
      "relevance_score": 0.87,
      "explanation": {
        "primary_reason": "Same director (Bong Joon-ho)",
        "secondary_reasons": [
          "Similar themes: class struggle, social commentary",
          "Genre overlap: thriller, drama",
          "High user rating correlation (r=0.82)"
        ],
        "score_breakdown": {
          "semantic_similarity": 0.75,
          "director_match": 0.95,
          "genre_overlap": 0.60,
          "user_preference": 0.88
        }
      }
    }
    ```
  And explanations are human-readable (no technical jargon)
  And explanations are accurate (manual review >90% agreement)

  Metrics:
    - Explanation generation latency: <50ms
    - Explanation accuracy (user agreement): >90%
    - Explanation clarity (readability score): >80

  Edge Cases:
    - Multiple strong signals → list top 3 reasons
    - Weak match (score <0.5) → explain why included (e.g., "Popular choice")
    - Personalized result → include "Based on your history" disclaimer

  Failures:
    - Explanation generation fails: Return result without explanation
    - Score breakdown unavailable: Provide primary reason only
```

---

## 5. SONA Recommendation Service Acceptance Criteria

### 5.1 Personalization Latency

```gherkin
Feature: Sub-5ms Personalization Inference

Scenario: Real-time personalized recommendations
  Given a user with 50+ hours of viewing history
  And a user-specific LoRA adapter (256K params, ~1MB)
  And a candidate pool of 100 popular content items
  When the recommendation service receives POST /api/recommendations
  Then the service loads user LoRA from cache (<1ms, in-memory)
  And the service scores all 100 candidates with SONA model
  And the service applies context filters (time: 9 PM, device: TV)
  And the service ranks by personalized score
  And the service returns top 10 recommendations
  And total inference latency <5ms (p95)

  Metrics:
    - Inference latency p50: <3ms
    - Inference latency p95: <5ms
    - Inference latency p99: <10ms
    - Throughput: 10,000+ inferences/second per instance

  Edge Cases:
    - User LoRA not in cache → load from disk (<20ms, SSD)
    - User LoRA doesn't exist (new user) → use global LoRA only
    - Candidate pool < 10 items → include items from similar users
    - All candidates scored <0.3 → fall back to popularity ranking

  Failures:
    - SONA model file missing: Use fallback popularity-based ranking
    - LoRA load fails: Use global LoRA only (generic recommendations)
    - Inference timeout (>100ms): Return cached recommendations
```

### 5.2 Cold-Start Problem Handling

```gherkin
Feature: New User Cold-Start Recommendations

Scenario: Personalize within 3 interactions
  Given a new user with no viewing history
  When the user completes onboarding questionnaire:
    - Favorite genres: [Sci-Fi, Thriller]
    - Favorite movie: "Inception"
    - Preferred language: English
  Then the system generates initial LoRA adapter
  And the system uses "Inception" embedding as seed
  And the system recommends semantically similar content
  And the user watches "Interstellar" (50% completion)

  When the system updates LoRA with implicit feedback (watch time)
  And the user searches "Christopher Nolan movies"
  Then the system recommends "Tenet", "Dunkirk", "The Prestige"
  And recommendations reflect both explicit (genre) and implicit (watch) signals
  And recommendation relevance p@10 >0.25 (vs >0.31 for established users)

  Metrics:
    - Cold-start precision@10: >0.25 (vs 0.31 normal)
    - Interactions to reach 0.31 precision: ≤3
    - Onboarding questionnaire completion rate: >80%

  Edge Cases:
    - User skips questionnaire → use global popularity trends
    - User selects 10+ favorite genres → normalize to top 5 by weight
    - User's favorite movie not in database → use closest match

  Failures:
    - Questionnaire data invalid: Use genre-based recommendations
    - Initial LoRA creation fails: Defer to global LoRA, retry async
```

### 5.3 LoRA Incremental Training

```gherkin
Feature: Incremental LoRA Adaptation

Scenario: Adapt user LoRA with new viewing data
  Given a user with existing LoRA adapter
  And the user watches "The Matrix" (100% completion, 5-star rating)
  When the recommendation service receives implicit feedback event
  Then the service queues LoRA update job (async)
  And the job runs within 5 minutes
  And the job fine-tunes user LoRA with new data point
  And the job applies EWC++ regularization (prevent catastrophic forgetting)
  And the job validates updated LoRA (sanity checks)
  And the job swaps in-memory LoRA atomically (no downtime)
  And subsequent recommendations reflect "The Matrix" preference

  Metrics:
    - Feedback processing latency: <5 minutes p95
    - LoRA update frequency: Max 1 update per 10 minutes per user
    - Catastrophic forgetting metric: <5% degradation on old preferences
    - Recommendation shift after update: 20-40% (significant but not complete)

  Edge Cases:
    - Multiple feedback events within 10 minutes → batch into single update
    - User rates content 1-star → apply negative reinforcement
    - User watches 5% then abandons → weak negative signal (low weight)
    - User re-watches content → increase preference weight

  Failures:
    - LoRA update fails validation: Rollback to previous version
    - Training diverges (loss > 2x baseline): Abort, alert, manual review
    - Atomic swap fails: Retry, else use stale LoRA (no user impact)
```

### 5.4 Recommendation Diversity (MMR)

```gherkin
Feature: Maximal Marginal Relevance for Diversity

Scenario: Ensure diverse recommendations, not echo chamber
  Given a user who primarily watches Marvel superhero movies
  And the base SONA scores heavily favor more Marvel content
  When the recommendation service applies MMR algorithm
  Then the service selects top recommendation (highest score)
  And for each subsequent recommendation:
    - Maximize: Relevance score (SONA output)
    - Minimize: Similarity to already-selected items
    - Lambda parameter: 0.7 (balance relevance vs diversity)
  And the final top 10 includes:
    - 6 superhero movies (high relevance)
    - 4 non-superhero action/sci-fi (diversity)
  And average intra-list cosine similarity <0.7

  Metrics:
    - Intra-list similarity: <0.7 (cosine distance of embeddings)
    - User engagement with diverse items: >10%
    - MMR computation latency: <10ms

  Edge Cases:
    - All candidates very similar → relax diversity constraint (min similarity 0.6)
    - User history extremely narrow → increase diversity weight (lambda 0.5)
    - Explicit user request "more like X" → disable MMR for this query

  Failures:
    - MMR computation timeout: Return non-diversified list
    - Similarity matrix unavailable: Use genre-based diversity fallback
```

---

## 6. Sync Service Acceptance Criteria

### 6.1 Cross-Device Sync Latency

```gherkin
Feature: Sub-100ms Cross-Device Synchronization

Scenario: Add to watchlist, sync across devices
  Given a user with 3 active devices: Phone, Tablet, TV
  And all devices connected to WebSocket /sync
  And all devices subscribed to PubNub channel: user.{userId}.sync
  When the user adds "Inception" to watchlist on Phone
  Then Phone generates CRDT OR-Set add operation
  And Phone applies operation locally (optimistic update, <10ms UI feedback)
  And Phone publishes operation to PubNub
  And Tablet receives operation via PubNub (<100ms)
  And TV receives operation via PubNub (<100ms)
  And Tablet merges operation with local CRDT state
  And TV merges operation with local CRDT state
  And all 3 devices show "Inception" in watchlist
  And total cross-device sync latency p95 <100ms

  Metrics:
    - Local update latency: <10ms (optimistic)
    - PubNub publish latency: <50ms p95
    - PubNub delivery latency: <100ms p95 (global)
    - CRDT merge latency: <5ms

  Edge Cases:
    - Device offline when operation occurs → sync when reconnected
    - Concurrent adds from 2 devices → both operations preserved (OR-Set)
    - Concurrent add on Phone, remove on Tablet → add wins (add-bias)
    - PubNub message lost → client detects gap, requests full state

  Failures:
    - PubNub unavailable: Queue operations locally, sync when restored
    - WebSocket disconnected: Reconnect with exponential backoff
    - CRDT merge conflict (shouldn't occur with OR-Set): Log error, manual review
```

### 6.2 CRDT Conflict Resolution

```gherkin
Feature: Conflict-Free Replicated Data Type Merge

Scenario: Concurrent watchlist modifications resolve correctly
  Given Phone and Tablet both offline
  And both have watchlist: ["Movie A", "Movie B"]
  When Phone adds "Movie C" at 10:00:00
  And Tablet removes "Movie B" at 10:00:05
  And both devices reconnect at 10:01:00
  Then both devices exchange CRDT operations
  And Phone receives remove("Movie B") operation
  And Tablet receives add("Movie C") operation
  And both devices merge using OR-Set semantics:
    - add("Movie C", tag="phone-uuid-1", timestamp=10:00:00)
    - remove("Movie B", tag="tablet-uuid-2", timestamp=10:00:05)
  And final state on both devices: ["Movie A", "Movie C"]
  And no manual conflict resolution required

  Metrics:
    - Merge correctness: 100% (no data loss)
    - Merge latency: <5ms for typical operations
    - Convergence time after reconnect: <1 second

  Edge Cases:
    - Concurrent add same item from 2 devices → deduplicate by entity_id
    - Add with tag "uuid-1", remove with same tag → remove wins
    - Add with tag "uuid-1", remove with different tag → item stays (add-bias)
    - 1000+ operations queued during offline → process in batches

  Failures:
    - CRDT state corrupted: Reset to server canonical state (data loss warning)
    - Merge produces invalid state: Reject operation, request full sync
    - Timestamp clock skew >1 minute: Use Hybrid Logical Clock (HLC)
```

### 6.3 Offline Queue Reconciliation

```gherkin
Feature: Offline Operation Queue

Scenario: Reconcile operations after extended offline period
  Given a user's Phone goes offline at 9:00 AM
  And the user performs 25 operations while offline:
    - Add 10 movies to watchlist
    - Remove 3 movies from watchlist
    - Update watch progress for 5 movies
    - Mark 7 movies as "want to watch"
  When the Phone reconnects at 11:00 AM (2 hours later)
  Then the Phone sends queued operations to sync service
  And the sync service applies operations in timestamp order
  And the sync service detects conflicts:
    - Movie X removed on Phone, but also removed on Tablet → no-op
    - Movie Y added on Phone, already exists on server → deduplicate
  And the sync service broadcasts net changes to other devices
  And other devices update UI with changes
  And Phone clears operation queue after confirmation

  Metrics:
    - Queue size limit: 1000 operations (else warn user)
    - Reconciliation time for 100 operations: <5 seconds
    - Conflict detection accuracy: 100%

  Edge Cases:
    - Queue exceeds 1000 operations → prompt user "Sync required"
    - Server state changed significantly → show "Review changes" UI
    - Operation references deleted entity → skip operation, log warning

  Failures:
    - Server rejects operation: Mark as failed, allow user to retry/discard
    - Network interruption during reconciliation: Resume from last confirmed operation
    - Queue corrupted: Prompt user to reset (data loss)
```

### 6.4 Device Presence Tracking

```gherkin
Feature: Real-Time Device Online/Offline Status

Scenario: Track device presence accurately
  Given a user with 3 devices: Phone, Tablet, TV
  When Phone connects to WebSocket /sync
  Then sync service subscribes to PubNub presence channel
  And sync service marks Phone as "online" in presence set (Redis)
  And other devices receive presence event: { device: "Phone", status: "online" }
  And UI shows green indicator next to "Phone"

  When Phone loses network connection
  Then PubNub detects timeout (60 seconds of inactivity)
  And PubNub sends presence event: { device: "Phone", status: "offline" }
  And sync service marks Phone as "offline" in Redis
  And other devices update UI to show gray indicator

  Metrics:
    - Presence detection latency: <60 seconds
    - Presence accuracy: >99% (no false online/offline)
    - Presence update delivery: <5 seconds after change

  Edge Cases:
    - Device switches networks (WiFi → cellular) → brief offline, then online
    - Device suspends/resumes → heartbeat timeout, then reconnect
    - Device uninstalls app → remains "offline" indefinitely (cleanup after 30d)
    - User has >10 devices → show "X other devices" summary

  Failures:
    - PubNub presence unavailable: No presence updates, show stale status
    - WebSocket flapping (connect/disconnect rapidly) → debounce (5s window)
    - Presence set in Redis grows unbounded → expire after 30 days inactive
```

---

## 7. Playback Service Acceptance Criteria

### 7.1 Deep Link Generation

```gherkin
Feature: Platform-Specific Deep Link Generation

Scenario: Generate deep links for all supported platforms
  Given a content entity "The Matrix" available on:
    - Netflix: netflix://title/60001164
    - Prime Video: primevideo://detail/0PDOILW8PHEHK6L3NVF6IHSG5X
    - YouTube: youtube://watch?v=vKQi3bBA1y8
  When the user selects "Watch on Netflix" from iPhone app
  Then playback service generates deep link: "netflix://title/60001164"
  And playback service includes fallback: "https://netflix.com/title/60001164"
  And playback service tracks click event for analytics
  And client attempts deep link (opens Netflix app if installed)
  And if Netflix app not installed, client opens fallback URL in Safari

  Metrics:
    - Deep link generation latency: <10ms
    - Deep link success rate: >90% (app opens)
    - Fallback success rate: 100% (browser opens)
    - Platform coverage: 150+ platforms

  Edge Cases:
    - Platform not installed → fallback to web URL
    - Platform requires authentication → redirect to platform login
    - Platform region-locked → show "Not available in your region"
    - Deep link format changed by platform → detect failure, alert for update

  Failures:
    - Deep link malformed: Use fallback URL
    - Deep link database missing entry: Use generic platform search URL
    - Platform discontinued: Show "No longer available"
```

### 7.2 Remote Playback Commands

```gherkin
Feature: Cross-Device Playback Control

Scenario: Send playback command from phone to TV
  Given user's Phone and TV both authenticated
  And TV is streaming "Inception" on Netflix
  And Phone shows "Now Playing: Inception on Living Room TV"
  When user taps "Pause" on Phone
  Then Phone sends command to playback service: POST /api/playback/pause
  And playback service validates user owns both devices
  And playback service publishes command to PubNub: user.{userId}.playback.{tvId}
  And TV receives command via WebSocket (<50ms)
  And TV pauses Netflix playback (native API if available, else simulate button press)
  And TV acknowledges command back to playback service
  And Phone updates UI to show "Paused"

  Metrics:
    - Command delivery latency: <50ms p95
    - Command acknowledgment latency: <100ms p95
    - Command success rate: >95%

  Edge Cases:
    - TV offline → queue command, execute when reconnected
    - TV playing on different platform → show error "Platform not supported"
    - User sends rapid commands (spam) → debounce (max 1 per second)
    - Multiple users in household → only device owner can control

  Failures:
    - Platform doesn't support remote control: Show "Not supported on [platform]"
    - Command times out (no acknowledgment): Retry 2x, then fail
    - TV unregistered during command: Return 404 "Device not found"
```

### 7.3 Device Status Real-Time Updates

```gherkin
Feature: Real-Time Playback Status Synchronization

Scenario: Sync playback status across devices
  Given user is watching "The Matrix" on TV
  And TV reports playback status every 10 seconds:
    - Current position: 1:32:45
    - State: playing
    - Platform: Netflix
    - Quality: 4K HDR
  When TV sends status update to playback service
  Then playback service updates LWW-Register CRDT (Last-Write-Wins)
  And playback service publishes update to PubNub
  And Phone receives update (<100ms)
  And Phone updates "Continue Watching" card:
    "The Matrix - 1h 32m of 2h 16m (68%)"
  And Phone shows progress bar at correct position

  Metrics:
    - Status update frequency: Every 10 seconds during playback
    - Status delivery latency: <100ms p95
    - Position accuracy: ±5 seconds

  Edge Cases:
    - User switches devices mid-stream → new device picks up last position
    - Playback ends (position = duration) → mark as "watched", remove from "continue"
    - User seeks backward → update position immediately (not batched)
    - Multiple devices playing same content → show most recent (LWW)

  Failures:
    - Status update lost (network) → next update corrects position
    - Position goes backward unexpectedly → ignore (likely stale update)
    - Status updates too frequent (spam) → throttle to max 1 per second
```

---

## 8. MCP Service Acceptance Criteria

### 8.1 MCP Tool Functionality

```gherkin
Feature: Model Context Protocol Tool Execution

Scenario: Semantic search via MCP
  Given Claude Desktop connected to MCP server via STDIO
  And user asks Claude: "Find sci-fi movies like Interstellar"
  When Claude calls MCP tool: semantic_search
    """json
    {
      "query": "sci-fi movies like Interstellar",
      "limit": 5,
      "filters": { "content_type": "movie" }
    }
    ```
  Then MCP server forwards request to discovery service
  And discovery service returns results (<400ms)
  And MCP server formats results as MCP response:
    ```json
    {
      "content": [
        {
          "type": "text",
          "text": "Found 5 sci-fi movies similar to Interstellar:\n1. Arrival (2016) - Available on Paramount+\n2. Contact (1997) - Available on HBO Max\n..."
        }
      ],
      "isError": false
    }
    ```
  And Claude presents results to user with natural language
  And total MCP overhead <50ms (excluding discovery service time)

  Metrics:
    - MCP tool execution latency: <50ms overhead
    - Tool success rate: >99%
    - Tool availability: 10+ tools exposed

  Edge Cases:
    - Tool called with invalid parameters → return MCP error with clear message
    - Tool times out (>30s) → return partial results + timeout notice
    - Tool requires auth but user not authenticated → return auth URL

  Failures:
    - Discovery service down: Return cached popular results + notice
    - MCP transport error: Retry 2x with exponential backoff
    - Tool result too large (>10MB): Truncate with "Show more" link
```

### 8.2 STDIO Transport Compatibility

```gherkin
Feature: STDIO Transport for Claude Desktop

Scenario: MCP server via STDIO in Claude Desktop
  Given Claude Desktop configured with MCP server:
    ```json
    {
      "mcpServers": {
        "media-gateway": {
          "command": "npx",
          "args": ["media-gateway-mcp", "start"],
          "env": { "API_KEY": "user-api-key" }
        }
      }
    }
    ```
  When Claude Desktop starts MCP server as subprocess
  Then MCP server initializes (<2 seconds)
  And MCP server listens on STDIN for JSON-RPC messages
  And MCP server writes responses to STDOUT
  And MCP server writes logs to STDERR (not visible to Claude)
  And Claude Desktop discovers 10+ available tools
  And tools appear in Claude's capability list
  And user can invoke tools via natural language

  Metrics:
    - Server initialization time: <2 seconds
    - STDIO message latency: <10ms
    - No spurious output to STDOUT (breaks protocol)

  Edge Cases:
    - Server crashes → Claude detects, shows error, offers restart
    - Server hangs → Claude times out after 30s, kills process
    - Multiple clients (shouldn't happen with STDIO): N/A (1:1 transport)

  Failures:
    - Invalid JSON-RPC: Log to STDERR, return MCP error response
    - STDIN closed unexpectedly: Shutdown gracefully
    - STDOUT buffer full: Flush immediately, don't block
```

### 8.3 SSE Transport for Web Clients

```gherkin
Feature: Server-Sent Events Transport for Web

Scenario: MCP server via SSE for web app
  Given web app establishes SSE connection:
    GET /mcp/sse
    Headers:
      Authorization: Bearer {jwt}
      Accept: text/event-stream
  When MCP server accepts connection
  Then server sends initial event:
    ```
    event: connected
    data: {"session_id": "sess-abc123", "tools": [...]}
    ```
  And client sends tool requests via POST /mcp/sse/{session_id}/request
  And server sends responses via SSE events:
    ```
    event: tool_response
    data: {"request_id": "req-xyz", "result": {...}}
    ```
  And connection stays open for bidirectional streaming

  Metrics:
    - SSE connection establishment: <200ms
    - Event delivery latency: <50ms
    - Connection stability: >99% uptime during session
    - Concurrent SSE connections: 10,000+ per instance

  Edge Cases:
    - Client closes connection → server cleans up session
    - Server sends event while client reconnecting → queue events (5 min max)
    - Multiple tabs from same user → separate sessions

  Failures:
    - SSE connection dropped: Client auto-reconnects with exponential backoff
    - Event larger than SSE limit (2MB): Split into multiple events
    - Session expired: Send "session_expired" event, require re-auth
```

### 8.4 ARW Manifest Validation

```gherkin
Feature: Agent-Ready Web Manifest Compliance

Scenario: ARW manifest at /.well-known/arw-manifest.json
  Given an AI agent discovers Media Gateway
  When the agent fetches GET /.well-known/arw-manifest.json
  Then the server returns ARW manifest:
    ```json
    {
      "arwVersion": "0.1",
      "profile": "ARW-1",
      "name": "Media Gateway",
      "description": "Unified cross-platform TV discovery engine",
      "capabilities": {
        "actions": [
          {
            "name": "semantic_search",
            "description": "Search for content using natural language",
            "inputSchema": { "type": "object", "properties": {...} },
            "auth": { "type": "none" }
          },
          {
            "name": "add_to_watchlist",
            "description": "Add content to user's watchlist",
            "inputSchema": { "type": "object", "properties": {...} },
            "auth": { "type": "oauth2", "scopes": ["watchlist:write"] }
          }
        ]
      },
      "endpoints": {
        "mcp": "https://api.mediagateway.io/mcp",
        "oauth": "https://auth.mediagateway.io/oauth/authorize"
      }
    }
    ```
  And the manifest validates against ARW 0.1 schema
  And all actions have complete input schemas
  And auth requirements are clearly declared

  Metrics:
    - Manifest fetch latency: <50ms (CDN cached)
    - Manifest size: <50KB
    - Schema validation: 100% compliant

  Edge Cases:
    - Manifest requested from agent in China → CDN serves from nearest edge
    - Manifest version negotiation → support ARW 0.1 and future versions

  Failures:
    - Manifest file missing: Return 404 (breaks ARW discovery)
    - Manifest invalid JSON: Return 500, fix immediately
    - Manifest out of sync with actual capabilities: Alert, manual review
```

---

## 9. API Gateway Acceptance Criteria

### 9.1 Rate Limiting Enforcement

```gherkin
Feature: Tiered Rate Limiting

Scenario: Enforce rate limits per user tier
  Given Free tier: 100 req/min, Pro tier: 1000 req/min, Enterprise: 10,000 req/min
  And a Free tier user makes 50 requests in 30 seconds
  When the user makes request #51
  Then the API Gateway increments Redis counter: rate_limit:user:{userId}:min:{timestamp_min}
  And the counter value is 51 (<100 limit)
  And the request is allowed
  And response headers include:
    - X-RateLimit-Limit: 100
    - X-RateLimit-Remaining: 49
    - X-RateLimit-Reset: 1670350800 (Unix timestamp)

  When the same user makes 50 more requests (total 101)
  Then request #101 receives 429 Too Many Requests
  And response includes:
    - Retry-After: 30 (seconds)
    - Error message: "Rate limit exceeded. Upgrade to Pro for higher limits."

  Metrics:
    - Rate limit check latency: <2ms (Redis lookup)
    - Rate limit accuracy: 100% (no false positives/negatives)
    - Counter atomicity: 100% (Redis INCR command)

  Edge Cases:
    - User upgrades tier mid-minute → apply new limit immediately
    - API Gateway instance failure → other instances enforce limits (shared Redis)
    - Redis key expiry race condition → TTL set atomically with first INCR

  Failures:
    - Redis unavailable: Fail-open (allow requests) + alert SRE
    - Clock skew between API Gateway instances: Use Redis TIME command
```

### 9.2 Authentication Validation

```gherkin
Feature: JWT Authentication Validation

Scenario: Validate JWT on protected endpoints
  Given a request to POST /api/search
  And the request includes header: Authorization: Bearer {jwt}
  When the API Gateway validates JWT
  Then the gateway verifies signature with public key (cached, <5ms)
  And the gateway checks expiry: exp > now
  And the gateway checks not-before: nbf <= now
  And the gateway checks issuer: iss == "https://auth.mediagateway.io"
  And the gateway extracts user_id from sub claim
  And the gateway checks token not in revocation list (Redis, <5ms)
  And the gateway forwards request to backend with header: X-User-ID: {user_id}

  Metrics:
    - JWT validation latency p50: <5ms
    - JWT validation latency p95: <10ms
    - Validation success rate: 100% for valid tokens
    - Public key cache hit rate: >99%

  Edge Cases:
    - JWT expired by 1 second → reject (no tolerance)
    - JWT issued in future (nbf > now) → reject, possible clock skew attack
    - JWT with unknown key ID (kid) → fetch public key, cache for 1 hour
    - Token in revocation list (user logged out) → reject with 401

  Failures:
    - Public key fetch fails: Use cached key if available, else reject request
    - Redis revocation list unavailable: Fail-open (allow request) + alert
    - Malformed JWT: Return 400 "Invalid token format"
```

### 9.3 Request Routing Correctness

```gherkin
Feature: Intelligent Request Routing

Scenario: Route requests to correct backend service
  Given API Gateway receives requests:
    - POST /api/search → Discovery Service
    - POST /api/recommendations → SONA Recommendation Service
    - POST /auth/login → Auth Service
    - GET /mcp/tools → MCP Server
    - POST /api/sync/watchlist → Sync Service
  When the API Gateway inspects request path and method
  Then the gateway routes each request to correct service (via service mesh)
  And the gateway includes tracing headers: X-Trace-ID, X-Span-ID
  And the gateway sets timeout: 5s for /auth/*, 30s for /api/*
  And the gateway retries failed requests: 2x with 100ms delay (idempotent only)

  Metrics:
    - Routing latency: <5ms (in-memory path matching)
    - Routing accuracy: 100% (no misrouted requests)
    - Timeout enforcement: 100% (no hung requests)

  Edge Cases:
    - Request path not in routing table → return 404 "Endpoint not found"
    - Backend service unavailable → return 503 "Service unavailable"
    - Backend service slow (near timeout) → log warning, don't retry

  Failures:
    - All replicas of service down: Return 503 with retry-after header
    - Service mesh partition: Route to last-known-healthy instance
    - Invalid routing configuration: Fail-safe to 503, alert immediately
```

### 9.4 Error Response Standardization

```gherkin
Feature: Standardized Error Responses

Scenario: Return consistent error format across all endpoints
  Given a request to POST /api/search with invalid parameters
  When the Discovery Service returns validation error
  Then the API Gateway transforms response to standard format:
    ```json
    {
      "error": {
        "code": "INVALID_PARAMETER",
        "message": "Query parameter is required",
        "details": {
          "field": "query",
          "reason": "missing_required_field"
        },
        "request_id": "req-abc123",
        "timestamp": "2025-12-06T21:30:00.000Z"
      }
    }
    ```
  And the response status code is 400
  And the response includes header: X-Request-ID: req-abc123
  And the error is logged with full context (user, endpoint, parameters)

  Metrics:
    - Error response latency: <10ms (transformation overhead)
    - Error format compliance: 100%
    - Error logging success rate: >99%

  Edge Cases:
    - Backend returns non-JSON error → wrap in standard format
    - Backend returns 5xx error → add "try again" suggestion
    - Backend timeout → return 504 "Gateway timeout" with request_id

  Failures:
    - Error transformation fails: Return raw backend error (degraded)
    - Logging pipeline down: Return error response, log locally
```

---

## 10. Integration Acceptance Criteria

### 10.1 End-to-End User Flow

```gherkin
Feature: Complete User Journey

Scenario: New user discovers, saves, and watches content
  Given a new user visits https://mediagateway.io

  # Step 1: Authentication
  When the user clicks "Sign in with Google"
  Then OAuth 2.0 + PKCE flow completes (<3s)
  And user is redirected to dashboard

  # Step 2: Search
  When the user searches "scary movies like Stranger Things"
  Then search results return in <500ms
  And results include "The Haunting of Hill House" (top result)

  # Step 3: Add to Watchlist
  When the user clicks "Add to Watchlist" on "The Haunting of Hill House"
  Then watchlist updates locally (<10ms)
  And CRDT operation syncs to other devices (<100ms)

  # Step 4: Cross-Device Handoff
  When the user opens Media Gateway on Smart TV
  Then watchlist includes "The Haunting of Hill House"
  And TV shows "Continue on TV" prompt

  # Step 5: Playback
  When the user selects "Watch on Netflix"
  Then deep link opens Netflix app on TV
  And content starts playing
  And playback status syncs to Phone (<100ms)

  # Step 6: Recommendations
  When the user returns to dashboard after watching
  Then SONA recommendations reflect horror preference
  And recommendations include similar shows

  Metrics:
    - Total flow completion time: <5 minutes (user-dependent)
    - Zero errors encountered: 100% success rate
    - All latency SLOs met: 100%

  Edge Cases:
    - User has no Netflix subscription → show alternative platforms
    - TV not paired → prompt to pair device
    - Network interruption during flow → graceful recovery

  Failures:
    - Any service down → graceful degradation with clear messaging
    - Authentication fails → retry with different provider
```

### 10.2 Multi-Platform Consistency

```gherkin
Feature: Cross-Platform Data Consistency

Scenario: Ensure data consistency across all platforms
  Given a user with accounts on Web, iOS, Android, TV, CLI
  When the user performs actions on each platform:
    - Web: Add "Movie A" to watchlist
    - iOS: Remove "Movie B" from watchlist
    - Android: Mark "Movie C" as watched
    - TV: Update watch progress for "Movie D" to 50%
    - CLI: Rate "Movie E" 5 stars
  Then within 2 minutes, all platforms reflect all changes:
    - Watchlist includes "Movie A", excludes "Movie B"
    - "Movie C" marked as watched
    - "Movie D" shows 50% progress
    - "Movie E" shows 5-star rating
  And CRDT merge resolves all operations correctly
  And no data is lost or corrupted

  Metrics:
    - Cross-platform sync time: <2 minutes
    - Data consistency: 100% (eventual consistency guaranteed)
    - Conflict resolution accuracy: 100%

  Edge Cases:
    - Platform offline during changes → sync when reconnected
    - Conflicting operations → CRDT semantics resolve deterministically
    - User performs 100+ operations → batch sync efficiently

  Failures:
    - Sync service down: Queue operations, sync when restored
    - CRDT state divergence: Reset to canonical server state (alert user)
```

### 10.3 Performance Under Load

```gherkin
Feature: System Performance at Scale

Scenario: Handle 100K concurrent users
  Given 100,000 authenticated users active simultaneously
  And users distributed: 40% web, 30% mobile, 20% TV, 10% CLI
  And user actions: 60% search, 25% browse, 10% watchlist, 5% playback
  When the system handles load for 1 hour
  Then all services meet SLOs:
    - Search latency p95 <500ms: ✓
    - Recommendation latency p95 <100ms: ✓
    - Sync latency p95 <100ms: ✓
    - Auth validation p95 <10ms: ✓
  And auto-scaling maintains performance:
    - Discovery Service: 3 → 15 replicas
    - Recommendation Service: 2 → 8 replicas
    - Sync Service: 2 → 5 replicas
  And infrastructure cost stays <$4,000/month

  Metrics:
    - Concurrent users: 100,000
    - Requests per second: 10,000+
    - Error rate: <0.1%
    - P95 latency for all services: Within SLO

  Edge Cases:
    - Traffic spike (2x normal) → scale to 200K users smoothly
    - Service failure (1 replica down) → auto-replace within 60s
    - Database read replica lag → route reads to primary temporarily

  Failures:
    - Complete region failure: Fail over to backup region (RTO: 5 minutes)
    - DDoS attack: Cloud Armor blocks, rate limiting protects backend
```

### 10.4 Data Privacy Compliance

```gherkin
Feature: GDPR/CCPA Compliance

Scenario: User exercises "Right to be Forgotten"
  Given a user submits GDPR data deletion request
  When the system processes request (within 30 days)
  Then the system deletes all user data:
    - User account (PostgreSQL)
    - Session tokens (Redis)
    - Watchlist data (PostgreSQL + CRDT)
    - Viewing history (PostgreSQL)
    - Recommendation model (user LoRA deleted)
    - Audit logs anonymized (user_id → hash)
  And the system retains only:
    - Anonymized aggregate analytics (ε-DP guarantee)
    - Legal compliance audit logs (2-year retention)
  And the system confirms deletion to user via email
  And subsequent login attempts fail with "Account not found"

  Metrics:
    - Data deletion completeness: 100%
    - Deletion confirmation time: <30 days
    - Re-identification risk: <1e-5 (ε=1.0, δ=1e-5 DP)

  Edge Cases:
    - User has active sessions → revoke all tokens before deletion
    - User data in backups → flag for deletion on next backup cycle
    - User data in logs → anonymize retroactively

  Failures:
    - Deletion fails for one data store: Retry, escalate to manual if fails 3x
    - User requests export before deletion: Provide data in JSON format
```

---

## Summary

This document defines **comprehensive, measurable acceptance criteria** for all Media Gateway components. Each criterion follows the Given-When-Then format with:

✅ **Success Metrics**: Quantifiable performance targets
✅ **Edge Cases**: Boundary conditions and corner cases
✅ **Failure Scenarios**: Error handling and graceful degradation

### Key Highlights

| Component | Critical Criteria | Target Metrics |
|-----------|------------------|----------------|
| **Auth Service** | OAuth 2.0 + PKCE flow | <3s completion, <10ms JWT validation |
| **Content Service** | Ingestion throughput | 1000 items/min, >95% entity resolution |
| **Search Service** | Hybrid search latency | p95 <400ms, precision@10 ≥0.31 |
| **SONA Recommendations** | Personalization | <5ms inference, cold-start in 3 interactions |
| **Sync Service** | Cross-device sync | <100ms latency, 100% CRDT correctness |
| **Playback Service** | Deep linking | >90% success rate, <50ms command delivery |
| **MCP Service** | Tool execution | 10+ tools, <50ms overhead, ARW compliant |
| **API Gateway** | Rate limiting | <2ms check, 100% accuracy, tiered limits |
| **Integration** | End-to-end flow | <5 min user journey, 100K concurrent users |

### Test Coverage Matrix

| Test Level | Coverage | Tools | Priority |
|------------|----------|-------|----------|
| **Unit Tests** | 80%+ | Jest, Cargo | High |
| **Integration Tests** | All APIs | Supertest | Critical |
| **E2E Tests** | User flows | Playwright | Critical |
| **Performance Tests** | Tier 1 services | k6 | High |
| **Security Tests** | OWASP Top 10 | ZAP | Critical |

---

**Document Status:** Complete
**Review Required:** Engineering team, QA team, Product team
**Next Phase:** Implementation with Test-Driven Development (TDD)

---

END OF PART 2
