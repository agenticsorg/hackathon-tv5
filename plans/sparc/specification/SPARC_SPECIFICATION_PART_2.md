# SPARC Specification — Part 2 of 4

## Media Gateway: Unified Cross-Platform TV Discovery Engine

**Document Version:** 1.0.0
**SPARC Phase:** Specification
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents — Part 2

7. [Ingestion Behavior Specifications](#7-ingestion-behavior-specifications)
8. [Metadata Requirements](#8-metadata-requirements)
9. [Streaming Platform Interaction Patterns](#9-streaming-platform-interaction-patterns)
10. [MCP Connector Role](#10-mcp-connector-role)

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
│                      │  ├── ... (48 more)                          │   │
│  ┌─────────────┐     │  └── Custom Platform Template               │   │
│  │ Watchmode   │────▶│                                             │   │
│  │ API         │     └──────────────┬──────────────────────────────┘   │
│  └─────────────┘                    │                                   │
│                                     ▼                                   │
│  ┌─────────────┐     ┌─────────────────────────────────────────────┐   │
│  │ YouTube     │────▶│  Entity Resolver                            │   │
│  │ Data API    │     │  ├── EIDR Matching                          │   │
│  └─────────────┘     │  ├── Fuzzy Title Matching                   │   │
│                      │  ├── Cross-Platform Deduplication           │   │
│  ┌─────────────┐     │  └── Canonical Entity Creation              │   │
│  │ TMDb        │────▶│                                             │   │
│  │ API         │     └──────────────┬──────────────────────────────┘   │
│  └─────────────┘                    │                                   │
│                                     ▼                                   │
│  ┌─────────────┐     ┌─────────────────────────────────────────────┐   │
│  │ Gracenote/  │────▶│  Kafka Event Stream                         │   │
│  │ TMS         │     │  ├── content.ingested                       │   │
│  └─────────────┘     │  ├── content.updated                        │   │
│                      │  ├── availability.changed                   │   │
│                      │  └── metadata.enriched                      │   │
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

#### 7.2.1 Normalizer Interface (Rust Trait)

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

#### 7.2.2 Canonical Content Schema

```rust
pub struct CanonicalContent {
    /// Primary identifier (EIDR preferred)
    pub entity_id: EntityId,

    /// Content type
    pub content_type: ContentType, // Movie, Series, Episode, Season

    /// Localized titles
    pub titles: HashMap<Locale, String>,

    /// Localized descriptions
    pub descriptions: HashMap<Locale, String>,

    /// Release information
    pub release_date: Option<NaiveDate>,
    pub release_year: u16,

    /// Runtime in minutes
    pub runtime_minutes: Option<u16>,

    /// Genre classifications
    pub genres: Vec<GenreId>,

    /// Content ratings by region
    pub ratings: HashMap<Region, ContentRating>,

    /// Cast and crew
    pub credits: Vec<Credit>,

    /// External identifiers
    pub external_ids: ExternalIds,

    /// Platform availability
    pub availability: Vec<PlatformAvailability>,

    /// Poster and backdrop images
    pub images: ContentImages,

    /// Embedding vector (768 dimensions)
    pub embedding: Option<Vec<f32>>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

pub struct ExternalIds {
    pub eidr: Option<String>,       // "10.5240/1234-5678-90AB-CDEF-O"
    pub imdb: Option<String>,        // "tt1234567"
    pub tmdb_movie: Option<u32>,     // 12345
    pub tmdb_tv: Option<u32>,        // 67890
    pub gracenote_tms: Option<String>, // "MV012345678901"
}

pub struct PlatformAvailability {
    pub platform_id: String,
    pub region: String,              // ISO 3166-1 alpha-3
    pub availability_type: AvailabilityType, // Subscription, Rent, Buy, Free
    pub price: Option<Price>,
    pub quality_tiers: Vec<QualityTier>, // SD, HD, 4K, HDR
    pub deep_link: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub added_at: DateTime<Utc>,
}
```

### 7.3 Ingestion Scheduling

#### 7.3.1 Refresh Frequencies

| Data Category | Refresh Frequency | Trigger | Priority |
|--------------|-------------------|---------|----------|
| Catalog (new content) | Every 6 hours | Scheduled | High |
| Availability changes | Every 1 hour | Scheduled + Webhook | Critical |
| Expiring content | Every 15 minutes | Scheduled | Critical |
| Metadata enrichment | Every 24 hours | Scheduled | Medium |
| Trending/Popular | Every 30 minutes | Scheduled | Medium |
| User preferences | Real-time | Event-driven | High |

#### 7.3.2 Rate Limiting Strategy

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    RATE LIMITING CONFIGURATION                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  API                  │ Limit          │ Strategy           │ Keys     │
│  ─────────────────────┼────────────────┼────────────────────┼───────── │
│  Streaming Avail.     │ 100/min        │ Token bucket       │ 3        │
│  Watchmode            │ 1000/day       │ Sliding window     │ 2        │
│  YouTube Data         │ 10,000/day     │ Quota units        │ 5+       │
│  TMDb                 │ 40/10sec       │ Fixed window       │ 1        │
│  Gracenote/TMS        │ Contract-based │ Token bucket       │ 1        │
│  JustWatch            │ 1000/hour      │ Sliding window     │ 3        │
│                                                                          │
│  MULTI-KEY ROTATION:                                                    │
│  ────────────────────                                                   │
│  For high-volume APIs (YouTube, aggregators), maintain multiple API    │
│  keys with automatic rotation when approaching limits.                  │
│                                                                          │
│  CIRCUIT BREAKER:                                                       │
│  ────────────────                                                       │
│  - Failure threshold: 5 consecutive errors                             │
│  - Open duration: 30 seconds                                            │
│  - Half-open: 3 test requests before closing                           │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 7.4 Entity Resolution

#### 7.4.1 Deduplication Pipeline

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
│  Confidence: 90-98% (manual review queue for edge cases)               │
│                                                                          │
│  Stage 4: Embedding Similarity                                          │
│  ───────────────────────────                                            │
│  Vector cosine similarity on title + description embeddings            │
│  Threshold: ≥0.92 similarity                                           │
│  Confidence: 85-95% (requires human validation)                        │
│                                                                          │
│  OUTPUT:                                                                 │
│  ───────                                                                │
│  - Canonical entity ID assignment                                       │
│  - Confidence score                                                      │
│  - Merge log for audit trail                                            │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 7.4.2 Conflict Resolution Rules

| Conflict Type | Resolution Strategy |
|--------------|---------------------|
| Title mismatch | Prefer EIDR-registered title, store variants |
| Runtime differs | Average across sources, flag >5min variance |
| Release date differs | Prefer earliest reliable source |
| Genre mismatch | Union of genres across platforms |
| Rating mismatch | Store all regional ratings |
| Description differs | Prefer longest, store platform-specific |

### 7.5 Data Quality Monitoring

```yaml
# Data quality SLOs
data_quality:
  completeness:
    required_fields: ["entity_id", "titles.en", "content_type", "release_year"]
    target: 99.9%

  accuracy:
    eidr_match_rate:
      target: 95%+
    deduplication_precision:
      target: 99%+

  freshness:
    catalog_lag:
      target: <6 hours
    availability_lag:
      target: <1 hour
    expiry_warning:
      target: 7 days advance notice

  alerts:
    - metric: completeness_rate
      threshold: <99%
      severity: warning
    - metric: ingestion_failures
      threshold: >10/hour
      severity: critical
```

---

## 8. Metadata Requirements

### 8.1 Metadata Schema

#### 8.1.1 Core Metadata Fields

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
| `keywords` | Array<String> | No | TMDb | Searchable tags |
| `languages` | Array<Locale> | No | Platform | Available audio languages |
| `subtitles` | Array<Locale> | No | Platform | Available subtitle languages |

#### 8.1.2 Availability Metadata

| Field | Type | Required | Update Frequency | Description |
|-------|------|----------|------------------|-------------|
| `platform_id` | String | Yes | Static | Platform identifier |
| `region` | String | Yes | Per-entry | ISO 3166-1 alpha-3 |
| `availability_type` | Enum | Yes | Hourly | Subscription/Rent/Buy/Free |
| `price` | Price | Conditional | Hourly | Required for Rent/Buy |
| `quality_tiers` | Array<Quality> | No | Daily | SD/HD/4K/HDR/Atmos |
| `deep_link` | URL | Yes | Weekly | Platform-specific link |
| `web_url` | URL | Yes | Weekly | Fallback web URL |
| `expires_at` | DateTime | No | Hourly | Content removal date |
| `added_at` | DateTime | Yes | Once | First availability |

#### 8.1.3 Enhanced Metadata (SONA Input)

| Field | Type | Source | Purpose |
|-------|------|--------|---------|
| `mood_tags` | Array<MoodTag> | ML-derived | Emotional classification |
| `themes` | Array<ThemeTag> | Gracenote + ML | Thematic elements |
| `pacing` | Enum | ML-derived | Slow/Medium/Fast |
| `visual_style` | Array<StyleTag> | ML-derived | Cinematography style |
| `complexity` | Float [0-1] | ML-derived | Narrative complexity |
| `family_friendly` | Float [0-1] | ML-derived | Family suitability score |
| `binge_score` | Float [0-1] | ML-derived | Series binge potential |
| `similar_to` | Array<EntityId> | Graph | Related content |

### 8.2 Identifier Standards

#### 8.2.1 EIDR (Entertainment Identifier Registry)

```
Format: 10.5240/XXXX-XXXX-XXXX-XXXX-X-X
Example: 10.5240/7B2F-ED3B-4F25-4A59-R

Structure:
- Prefix: "10.5240" (EIDR DOI prefix)
- Content ID: 4 groups of 4 hex characters
- Suffix: Check digit and type indicator

Usage:
- Primary cross-platform identifier
- Required for canonical entity creation
- Used for licensing and rights management
```

#### 8.2.2 Identifier Cross-Reference Table

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    IDENTIFIER CROSS-REFERENCE                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Example: "The Matrix" (1999)                                           │
│                                                                          │
│  Identifier Type    │ Value                          │ Authority        │
│  ───────────────────┼────────────────────────────────┼───────────────── │
│  EIDR               │ 10.5240/7B2F-ED3B-4F25-4A59-R │ ISO 26324        │
│  IMDb               │ tt0133093                      │ IMDb.com         │
│  TMDb (Movie)       │ 603                            │ TMDb.org         │
│  Gracenote TMS      │ MV000057872000                 │ Gracenote        │
│  Netflix            │ 20557937                       │ Platform         │
│  Prime Video        │ B00FZL4WEK                     │ Platform         │
│  YouTube            │ UCsn8WvsDf_wQe... (channel)    │ Platform         │
│                                                                          │
│  RESOLUTION ORDER:                                                      │
│  1. EIDR (authoritative, ISO standard)                                 │
│  2. Gracenote TMS (rich metadata)                                      │
│  3. IMDb (widely recognized)                                           │
│  4. TMDb (community-sourced, free)                                     │
│  5. Platform-specific (last resort)                                    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.3 Genre Taxonomy

#### 8.3.1 Unified Genre Hierarchy

```yaml
genres:
  action:
    id: "genre:action"
    display_name:
      en: "Action"
      es: "Acción"
    children:
      - "genre:action:martial_arts"
      - "genre:action:spy"
      - "genre:action:superhero"
    platform_mappings:
      netflix: [28, 1365]
      prime: ["action", "action_adventure"]
      disney: ["action"]

  comedy:
    id: "genre:comedy"
    display_name:
      en: "Comedy"
    children:
      - "genre:comedy:romantic"
      - "genre:comedy:dark"
      - "genre:comedy:slapstick"
    platform_mappings:
      netflix: [6548, 869]
      prime: ["comedy"]

  # ... 20+ top-level genres with ~100 sub-genres
```

#### 8.3.2 Platform Genre Mapping

| Unified Genre | Netflix Codes | Prime Categories | TMDb IDs |
|--------------|---------------|------------------|----------|
| Action | 28, 1365 | action | 28 |
| Comedy | 6548, 869 | comedy | 35 |
| Drama | 5763, 2150 | drama | 18 |
| Horror | 8711, 10944 | horror | 27 |
| Sci-Fi | 7537, 3916 | sci-fi | 878 |
| Documentary | 6839, 7534 | documentary | 99 |
| Animation | 7424, 10045 | animation | 16 |
| Thriller | 8933, 10499 | thriller | 53 |

### 8.4 Image Requirements

```yaml
images:
  poster:
    required: true
    sizes:
      - name: "thumbnail"
        width: 92
        height: 138
      - name: "small"
        width: 185
        height: 278
      - name: "medium"
        width: 342
        height: 513
      - name: "large"
        width: 500
        height: 750
      - name: "original"
        width: variable
        height: variable
    format: ["webp", "jpg"]
    fallback: "/images/no-poster.svg"

  backdrop:
    required: false
    sizes:
      - name: "small"
        width: 300
        height: 169
      - name: "medium"
        width: 780
        height: 439
      - name: "large"
        width: 1280
        height: 720
      - name: "original"
        width: variable
        height: variable
    format: ["webp", "jpg"]

  logo:
    required: false
    sizes:
      - name: "small"
        width: 92
      - name: "medium"
        width: 185
      - name: "large"
        width: 500
```

### 8.5 Localization Requirements

#### 8.5.1 Supported Locales

| Priority | Locales | Coverage |
|----------|---------|----------|
| Tier 1 | en-US, en-GB, es-ES, es-MX, fr-FR, de-DE, pt-BR, ja-JP | Launch |
| Tier 2 | it-IT, ko-KR, zh-CN, zh-TW, nl-NL, pl-PL, sv-SE | Phase 2 |
| Tier 3 | All ISO 639-1 codes | On-demand |

#### 8.5.2 Localization Fallback Chain

```
User Locale: "es-MX"

Fallback Chain:
1. es-MX (exact match)
2. es-ES (language match)
3. es (language only)
4. en-US (default)

Applied to:
- Titles
- Descriptions
- Genre names
- Content ratings
- UI strings
```

---

## 9. Streaming Platform Interaction Patterns

### 9.1 Platform Integration Matrix

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    PLATFORM INTEGRATION MATRIX                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Platform      │ API    │ Integration    │ Auth    │ Deep Link          │
│  ──────────────┼────────┼────────────────┼─────────┼──────────────────  │
│  Netflix       │ ❌ No  │ Aggregator     │ N/A     │ netflix://title/X  │
│  Prime Video   │ ❌ No  │ Aggregator     │ N/A     │ aiv://detail?asin= │
│  Disney+       │ ❌ No  │ Aggregator     │ N/A     │ disneyplus://      │
│  Hulu          │ ❌ No  │ Aggregator     │ N/A     │ hulu://watch/      │
│  HBO Max       │ ❌ No  │ Aggregator     │ N/A     │ hbomax://content/  │
│  Apple TV+     │ ❌ No  │ Aggregator     │ N/A     │ tvapp://          │
│  Peacock       │ ❌ No  │ Aggregator     │ N/A     │ peacock://watch/   │
│  Paramount+    │ ❌ No  │ Aggregator     │ N/A     │ paramountplus://   │
│  YouTube       │ ✅ Yes │ Direct OAuth   │ OAuth2  │ youtube://watch?v= │
│  Twitch        │ ✅ Yes │ Direct OAuth   │ OAuth2  │ twitch://stream/   │
│  Crunchyroll   │ ⚠️ Ltd│ Aggregator+API │ OAuth2  │ crunchyroll://     │
│  Tubi          │ ❌ No  │ Aggregator     │ N/A     │ tubi://watch/      │
│  Pluto TV      │ ❌ No  │ Aggregator     │ N/A     │ plutotv://         │
│                                                                          │
│  Legend:                                                                 │
│  ✅ Full API available                                                   │
│  ⚠️ Limited API (read-only or restricted)                               │
│  ❌ No public API                                                        │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 9.2 Aggregator API Integration

#### 9.2.1 Streaming Availability API

```yaml
provider: streaming-availability
base_url: https://streaming-availability.p.rapidapi.com
authentication:
  type: api_key
  header: X-RapidAPI-Key

rate_limits:
  requests_per_minute: 100
  requests_per_month: 500000

endpoints:
  shows_search:
    path: /shows/search/filters
    params:
      - country: required, string (ISO 3166-1 alpha-2)
      - catalogs: optional, array of service IDs
      - show_type: optional, enum [movie, series]
      - genres: optional, array
      - output_language: optional, string
    response:
      - shows: array of content objects
      - hasMore: boolean
      - nextCursor: string

  show_details:
    path: /shows/{id}
    params:
      - country: required
    response:
      - Full content metadata
      - streamingOptions by country

coverage:
  countries: 60+
  services: 150+
  content: 500,000+ titles
```

#### 9.2.2 Watchmode API

```yaml
provider: watchmode
base_url: https://api.watchmode.com/v1
authentication:
  type: api_key
  query_param: apiKey

rate_limits:
  requests_per_day: 1000
  burst: 50/minute

endpoints:
  search:
    path: /search/
    params:
      - search_field: title, people, etc.
      - search_value: query string
      - types: movie, tv_series, etc.

  title_details:
    path: /title/{id}/details/
    params:
      - append_to_response: sources,genres

  sources:
    path: /title/{id}/sources/
    params:
      - regions: comma-separated ISO codes

  list_titles:
    path: /list-titles/
    params:
      - source_ids: platform filter
      - regions: geographic filter

coverage:
  services: 200+
  countries: 50+
  content: 1M+ titles
```

### 9.3 Deep Linking Specification

#### 9.3.1 iOS Universal Links

```json
// /.well-known/apple-app-site-association
{
  "applinks": {
    "apps": [],
    "details": [
      {
        "appID": "TEAMID.com.mediagateway.app",
        "paths": [
          "/movie/*",
          "/tv/*",
          "/watch/*",
          "/open/*"
        ]
      }
    ]
  }
}
```

#### 9.3.2 Android App Links

```json
// /.well-known/assetlinks.json
[
  {
    "relation": ["delegate_permission/common.handle_all_urls"],
    "target": {
      "namespace": "android_app",
      "package_name": "com.mediagateway.app",
      "sha256_cert_fingerprints": [
        "AA:BB:CC:DD:..."
      ]
    }
  }
]
```

#### 9.3.3 Platform Deep Link Patterns

```typescript
interface DeepLinkConfig {
  platform: string;
  ios: {
    scheme: string;
    appStoreId: string;
    universalLink?: string;
  };
  android: {
    scheme: string;
    packageName: string;
    playStoreUrl: string;
  };
  web: {
    baseUrl: string;
    pathTemplate: string;
  };
}

const deepLinks: Record<string, DeepLinkConfig> = {
  netflix: {
    platform: "netflix",
    ios: {
      scheme: "nflx://www.netflix.com/title/{id}",
      appStoreId: "363590051",
      universalLink: "https://www.netflix.com/title/{id}"
    },
    android: {
      scheme: "intent://www.netflix.com/title/{id}#Intent;scheme=nflx;package=com.netflix.mediaclient;end",
      packageName: "com.netflix.mediaclient",
      playStoreUrl: "https://play.google.com/store/apps/details?id=com.netflix.mediaclient"
    },
    web: {
      baseUrl: "https://www.netflix.com",
      pathTemplate: "/title/{id}"
    }
  },

  prime_video: {
    platform: "prime_video",
    ios: {
      scheme: "aiv://aiv/detail?asin={id}",
      appStoreId: "545519333"
    },
    android: {
      scheme: "intent://www.amazon.com/gp/video/detail/{id}#Intent;scheme=https;package=com.amazon.avod.thirdpartyclient;end",
      packageName: "com.amazon.avod.thirdpartyclient",
      playStoreUrl: "https://play.google.com/store/apps/details?id=com.amazon.avod.thirdpartyclient"
    },
    web: {
      baseUrl: "https://www.amazon.com",
      pathTemplate: "/gp/video/detail/{id}"
    }
  },

  // ... additional platforms
};
```

### 9.4 OAuth Flow for YouTube Integration

```
┌─────────────────────────────────────────────────────────────────────────┐
│              YOUTUBE OAUTH 2.0 + PKCE FLOW                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────┐         ┌──────────┐         ┌──────────────────────┐    │
│  │   User   │         │   App    │         │   Google OAuth       │    │
│  └────┬─────┘         └────┬─────┘         └──────────┬───────────┘    │
│       │                    │                          │                │
│       │  1. Tap "Connect   │                          │                │
│       │     YouTube"       │                          │                │
│       │───────────────────▶│                          │                │
│       │                    │                          │                │
│       │                    │  2. Generate:            │                │
│       │                    │     - code_verifier      │                │
│       │                    │     - code_challenge     │                │
│       │                    │     - state              │                │
│       │                    │                          │                │
│       │                    │  3. Redirect to Google   │                │
│       │◀───────────────────│  /authorize?             │                │
│       │                    │  client_id=XXX&          │                │
│       │                    │  redirect_uri=XXX&       │                │
│       │                    │  scope=youtube.readonly& │                │
│       │                    │  code_challenge=XXX&     │                │
│       │                    │  code_challenge_method=  │                │
│       │                    │  S256&state=XXX          │                │
│       │                    │                          │                │
│       │  4. User logs in   │                          │                │
│       │     and consents   │                          │                │
│       │─────────────────────────────────────────────▶│                │
│       │                    │                          │                │
│       │  5. Redirect back  │                          │                │
│       │     with auth code │                          │                │
│       │◀─────────────────────────────────────────────│                │
│       │                    │                          │                │
│       │                    │  6. Exchange code for    │                │
│       │                    │     tokens (with         │                │
│       │                    │     code_verifier)       │                │
│       │                    │─────────────────────────▶│                │
│       │                    │                          │                │
│       │                    │  7. Return:              │                │
│       │                    │     - access_token       │                │
│       │                    │     - refresh_token      │                │
│       │                    │◀─────────────────────────│                │
│       │                    │                          │                │
│       │  8. "YouTube       │                          │                │
│       │     Connected!"    │                          │                │
│       │◀───────────────────│                          │                │
│       │                    │                          │                │
└─────────────────────────────────────────────────────────────────────────┘

Required Scopes:
- youtube.readonly: Read user's subscriptions, watch history
- youtube.force-ssl: API access over HTTPS

Token Lifecycle:
- Access token: 1 hour expiry
- Refresh token: Long-lived, rotate on use
- Storage: Secure keychain/keystore, encrypted at rest
```

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
│  │  ┌─────────────────────────────────────────────────────────────┐  │  │
│  │  │                    CAPABILITIES                              │  │  │
│  │  ├─────────────────────────────────────────────────────────────┤  │  │
│  │  │  TOOLS (10)         │ RESOURCES (3)     │ PROMPTS (2)       │  │  │
│  │  │  ─────────────────  │ ────────────────  │ ────────────────  │  │  │
│  │  │  • semantic_search  │ • hackathon://    │ • discovery_      │  │  │
│  │  │  • get_content      │   config          │   assistant       │  │  │
│  │  │  • discover_content │ • hackathon://    │ • recommendation_ │  │  │
│  │  │  • get_recommend.   │   tracks          │   guide           │  │  │
│  │  │  • initiate_play    │ • media://        │                   │  │  │
│  │  │  • control_playback │   trending        │                   │  │  │
│  │  │  • get_genres       │                   │                   │  │  │
│  │  │  • update_prefs     │                   │                   │  │  │
│  │  │  • list_devices     │                   │                   │  │  │
│  │  │  • get_device_stat  │                   │                   │  │  │
│  │  └─────────────────────────────────────────────────────────────┘  │  │
│  │                                                                    │  │
│  │  ┌─────────────────────────────────────────────────────────────┐  │  │
│  │  │                    MIDDLEWARE                                │  │  │
│  │  ├─────────────────────────────────────────────────────────────┤  │  │
│  │  │  • Authentication (OAuth 2.0 validation)                    │  │  │
│  │  │  • Rate Limiting (100-1000 req/15min)                       │  │  │
│  │  │  • Input Validation (Zod schemas)                           │  │  │
│  │  │  • Error Handling (JSON-RPC 2.0)                            │  │  │
│  │  │  • Logging (structured, trace IDs)                          │  │  │
│  │  └─────────────────────────────────────────────────────────────┘  │  │
│  │                                                                    │  │
│  └───────────────────────────────┬───────────────────────────────────┘  │
│                                  │                                       │
│  ┌───────────────────────────────┴───────────────────────────────────┐  │
│  │                     BACKEND SERVICES                               │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐              │  │
│  │  │ SONA    │  │Ruvector │  │ PubNub  │  │  Auth   │              │  │
│  │  │ Engine  │  │ Search  │  │  Sync   │  │ Service │              │  │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘              │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 10.2 MCP Tool Definitions

#### 10.2.1 `semantic_search`

```typescript
{
  name: "semantic_search",
  description: "Search for movies and TV shows using natural language queries. Supports mood-based, reference-based, and descriptive searches.",
  inputSchema: {
    type: "object",
    properties: {
      query: {
        type: "string",
        description: "Natural language search query (e.g., 'something like Stranger Things but more scary')"
      },
      filters: {
        type: "object",
        properties: {
          content_type: {
            type: "string",
            enum: ["movie", "series", "any"],
            default: "any"
          },
          genres: {
            type: "array",
            items: { type: "string" }
          },
          release_year_min: { type: "integer" },
          release_year_max: { type: "integer" },
          platforms: {
            type: "array",
            items: { type: "string" },
            description: "Filter to specific streaming platforms"
          },
          region: {
            type: "string",
            description: "ISO 3166-1 alpha-3 region code"
          }
        }
      },
      limit: {
        type: "integer",
        minimum: 1,
        maximum: 50,
        default: 10
      }
    },
    required: ["query"]
  },
  outputSchema: {
    type: "object",
    properties: {
      results: {
        type: "array",
        items: {
          type: "object",
          properties: {
            entity_id: { type: "string" },
            title: { type: "string" },
            content_type: { type: "string" },
            release_year: { type: "integer" },
            description: { type: "string" },
            genres: { type: "array", items: { type: "string" } },
            rating: { type: "number" },
            match_score: { type: "number" },
            availability: {
              type: "array",
              items: {
                type: "object",
                properties: {
                  platform: { type: "string" },
                  type: { type: "string" },
                  deep_link: { type: "string" }
                }
              }
            },
            poster_url: { type: "string" }
          }
        }
      },
      total_count: { type: "integer" },
      query_parsed: {
        type: "object",
        description: "Parsed intent from natural language query"
      }
    }
  }
}
```

#### 10.2.2 `get_content_details`

```typescript
{
  name: "get_content_details",
  description: "Get detailed information about a specific movie or TV show including full metadata, cast, availability, and related content.",
  inputSchema: {
    type: "object",
    properties: {
      entity_id: {
        type: "string",
        description: "The unique entity identifier (EIDR or internal ID)"
      },
      region: {
        type: "string",
        description: "Region for availability info (ISO 3166-1 alpha-3)",
        default: "USA"
      },
      include: {
        type: "array",
        items: {
          type: "string",
          enum: ["credits", "similar", "availability", "images", "videos"]
        },
        default: ["availability"]
      }
    },
    required: ["entity_id"]
  }
}
```

#### 10.2.3 `get_recommendations`

```typescript
{
  name: "get_recommendations",
  description: "Get personalized content recommendations based on user preferences, viewing history, and context.",
  inputSchema: {
    type: "object",
    properties: {
      context: {
        type: "string",
        description: "Viewing context (e.g., 'family movie night', 'date night', 'background viewing')"
      },
      mood: {
        type: "string",
        description: "Desired mood (e.g., 'uplifting', 'thrilling', 'relaxing')"
      },
      age_appropriate: {
        type: "array",
        items: { type: "integer" },
        description: "Ages of viewers for content filtering"
      },
      exclude_watched: {
        type: "boolean",
        default: true
      },
      platforms: {
        type: "array",
        items: { type: "string" },
        description: "Limit to specific platforms"
      },
      limit: {
        type: "integer",
        minimum: 1,
        maximum: 20,
        default: 5
      }
    }
  }
}
```

#### 10.2.4 `list_devices`

```typescript
{
  name: "list_devices",
  description: "List all devices registered to the user's account with their current status.",
  inputSchema: {
    type: "object",
    properties: {
      status_filter: {
        type: "string",
        enum: ["all", "online", "offline"],
        default: "all"
      }
    }
  },
  outputSchema: {
    type: "object",
    properties: {
      devices: {
        type: "array",
        items: {
          type: "object",
          properties: {
            device_id: { type: "string" },
            name: { type: "string" },
            type: { type: "string", enum: ["phone", "tablet", "tv", "web", "cli"] },
            status: { type: "string", enum: ["online", "offline", "watching"] },
            last_seen: { type: "string", format: "date-time" },
            capabilities: {
              type: "object",
              properties: {
                supports_4k: { type: "boolean" },
                supports_hdr: { type: "boolean" },
                supports_dolby_atmos: { type: "boolean" }
              }
            },
            current_activity: {
              type: "object",
              nullable: true,
              properties: {
                content_id: { type: "string" },
                title: { type: "string" },
                progress_percent: { type: "number" }
              }
            }
          }
        }
      }
    }
  }
}
```

#### 10.2.5 `initiate_playback`

```typescript
{
  name: "initiate_playback",
  description: "Initiate playback of content on a specific device. Requires user authorization for the target device.",
  inputSchema: {
    type: "object",
    properties: {
      entity_id: {
        type: "string",
        description: "Content to play"
      },
      device_id: {
        type: "string",
        description: "Target device ID"
      },
      platform: {
        type: "string",
        description: "Preferred streaming platform"
      },
      resume: {
        type: "boolean",
        default: true,
        description: "Resume from last position if available"
      }
    },
    required: ["entity_id", "device_id"]
  }
}
```

### 10.3 MCP Transport Specifications

#### 10.3.1 STDIO Transport

```typescript
// Configuration for Claude Desktop / Cursor integration
{
  "mcpServers": {
    "media-gateway": {
      "command": "npx",
      "args": ["@media-gateway/mcp-server"],
      "env": {
        "MG_API_KEY": "${MG_API_KEY}",
        "MG_USER_ID": "${MG_USER_ID}"
      }
    }
  }
}

// Message format (line-delimited JSON)
// Request:
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"semantic_search","arguments":{"query":"sci-fi movies like Arrival"}}}

// Response:
{"jsonrpc":"2.0","id":1,"result":{"results":[...]}}
```

#### 10.3.2 SSE Transport

```typescript
// SSE Server Configuration
const sseConfig = {
  port: 3000,
  cors: {
    origin: ["https://app.mediagateway.io"],
    credentials: true
  },
  security: {
    helmet: true,
    rateLimitWindow: 15 * 60 * 1000, // 15 minutes
    maxRequestsPerWindow: 100
  }
};

// Connection endpoint: GET /mcp/sse
// Event stream format:
// event: message
// data: {"jsonrpc":"2.0",...}

// Request endpoint: POST /mcp/request
// Content-Type: application/json
// Body: {"jsonrpc":"2.0","method":"tools/call",...}
```

### 10.4 ARW Manifest

```json
// /.well-known/arw-manifest.json
{
  "$schema": "https://arw.agentics.org/schemas/manifest-v1.json",
  "version": "1.0.0",
  "site": {
    "name": "Media Gateway",
    "description": "Unified cross-platform TV and movie discovery engine",
    "logo": "https://app.mediagateway.io/logo.svg"
  },
  "capabilities": {
    "mcp": {
      "version": "2024-11-05",
      "transports": ["stdio", "sse"],
      "tools_url": "/api/mcp/tools",
      "resources_url": "/api/mcp/resources"
    },
    "semantic_search": {
      "endpoint": "/api/search",
      "method": "POST",
      "supports_natural_language": true,
      "supports_filters": true
    },
    "actions": [
      {
        "id": "search",
        "name": "Search Content",
        "description": "Search for movies and TV shows",
        "oauth_scopes": ["read:content"]
      },
      {
        "id": "recommend",
        "name": "Get Recommendations",
        "description": "Get personalized recommendations",
        "oauth_scopes": ["read:content", "read:preferences"]
      },
      {
        "id": "playback",
        "name": "Control Playback",
        "description": "Initiate playback on connected devices",
        "oauth_scopes": ["read:content", "write:playback"],
        "requires_user_consent": true
      }
    ]
  },
  "authentication": {
    "oauth2": {
      "authorization_endpoint": "/oauth/authorize",
      "token_endpoint": "/oauth/token",
      "scopes": [
        "read:content",
        "read:preferences",
        "write:preferences",
        "write:playback",
        "read:devices"
      ]
    }
  },
  "rate_limits": {
    "unauthenticated": 10,
    "authenticated": 1000,
    "window_seconds": 900
  }
}
```

### 10.5 Error Handling

```typescript
// MCP Error Codes (JSON-RPC 2.0 + MCP extensions)
const MCP_ERRORS = {
  // Standard JSON-RPC errors
  PARSE_ERROR: { code: -32700, message: "Parse error" },
  INVALID_REQUEST: { code: -32600, message: "Invalid Request" },
  METHOD_NOT_FOUND: { code: -32601, message: "Method not found" },
  INVALID_PARAMS: { code: -32602, message: "Invalid params" },
  INTERNAL_ERROR: { code: -32603, message: "Internal error" },

  // MCP-specific errors
  TOOL_NOT_FOUND: { code: -32001, message: "Tool not found" },
  RESOURCE_NOT_FOUND: { code: -32002, message: "Resource not found" },
  UNAUTHORIZED: { code: -32003, message: "Unauthorized" },
  RATE_LIMITED: { code: -32004, message: "Rate limit exceeded" },

  // Media Gateway errors
  CONTENT_NOT_FOUND: { code: -33001, message: "Content not found" },
  DEVICE_OFFLINE: { code: -33002, message: "Device offline" },
  PLAYBACK_FAILED: { code: -33003, message: "Playback initiation failed" },
  PLATFORM_UNAVAILABLE: { code: -33004, message: "Platform unavailable in region" }
};

// Error response format
interface MCPError {
  jsonrpc: "2.0";
  id: string | number | null;
  error: {
    code: number;
    message: string;
    data?: {
      details?: string;
      retry_after?: number;  // For rate limiting
      suggestion?: string;   // Actionable guidance
    };
  };
}
```

---

## End of Part 2

**Continue to:** [SPARC Specification — Part 3 of 4](./SPARC_SPECIFICATION_PART_3.md)

**Part 3 Contents:**
- Ruvector Simulation and Storage Responsibilities
- PubNub Real-Time Sync Behavior
- Device Interactions
- CLI Behavior Specifications
