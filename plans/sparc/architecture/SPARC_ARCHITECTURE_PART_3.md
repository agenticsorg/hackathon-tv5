# SPARC Architecture Phase - Part 3: Integration Architecture

**Version:** 1.0.0
**Phase:** SPARC Architecture
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents

1. [Integration Architecture Overview](#1-integration-architecture-overview)
2. [Streaming Platform Integrations](#2-streaming-platform-integrations)
3. [Metadata Provider Integrations](#3-metadata-provider-integrations)
4. [PubNub Integration Architecture](#4-pubnub-integration-architecture)
5. [AI/ML Integrations](#5-aiml-integrations)
6. [Webhook Architecture](#6-webhook-architecture)
7. [Event-Driven Integration](#7-event-driven-integration)
8. [Third-Party Service Abstraction](#8-third-party-service-abstraction)
9. [Integration Testing Strategy](#9-integration-testing-strategy)

---

## 1. Integration Architecture Overview

### 1.1 Integration Principles

The Media Gateway integration architecture follows these core principles:

1. **Adapter Pattern**: All external integrations isolated behind uniform interfaces
2. **Circuit Breaker**: Automatic fault isolation and recovery
3. **Rate Limiting**: Respect external API limits with multi-key rotation
4. **Caching**: Aggressive caching to reduce API calls and cost
5. **Fallback Chains**: Multiple data sources with automatic failover
6. **Idempotency**: All operations safe to retry
7. **Observable**: Comprehensive metrics and distributed tracing

### 1.2 Integration Layers

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    INTEGRATION ARCHITECTURE LAYERS                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Layer 1: Application Layer                                             │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  MCP Server  │  REST API  │  GraphQL  │  CLI  │  Admin Dashboard │  │
│  └────────────────────────┬─────────────────────────────────────────┘  │
│                           │                                              │
│  Layer 2: Service Abstraction                                           │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  Discovery Service  │  Recommendation Service  │  Sync Service    │  │
│  │  ┌────────────┐     │  ┌──────────────┐       │  ┌────────────┐  │  │
│  │  │ Circuit    │     │  │ Rate Limiter │       │  │ Cache      │  │  │
│  │  │ Breaker    │     │  │              │       │  │ Manager    │  │  │
│  │  └────────────┘     │  └──────────────┘       │  └────────────┘  │  │
│  └────────────────────────┬─────────────────────────────────────────┘  │
│                           │                                              │
│  Layer 3: Integration Adapters (Uniform Interface)                     │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  Platform    │  Metadata   │  PubNub     │  ML        │ Webhook  │  │
│  │  Adapter     │  Adapter    │  Adapter    │  Adapter   │ Adapter  │  │
│  └────────────────────────┬─────────────────────────────────────────┘  │
│                           │                                              │
│  Layer 4: External Systems                                              │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  Netflix  │  TMDb   │  PubNub  │  Embedding  │  Platform        │  │
│  │  Prime    │  IMDb   │  Service │  Service    │  Webhooks        │  │
│  │  Disney+  │  JustWatch      │  LoRA Service   │                  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.3 Integration Catalog

| Integration Type | Count | Purpose | Latency SLO | Availability SLO |
|-----------------|-------|---------|-------------|------------------|
| Streaming Platforms | 150+ | Content catalog & availability | <2s | 99.5% |
| Metadata Providers | 5 | Enriched metadata | <1s | 99.9% |
| Real-time Sync | 1 (PubNub) | Cross-device state sync | <100ms | 99.99% |
| AI/ML Services | 3 | Embeddings, LoRA, inference | <500ms | 99.9% |
| Webhooks | 2-way | Event notifications | <5s | 99.5% |
| Event Bus | 1 (Kafka) | Internal events | <50ms | 99.9% |

---

## 2. Streaming Platform Integrations

### 2.1 Platform Adapter Interface

All streaming platform integrations implement a common interface for consistency:

```typescript
interface PlatformAdapter {
  // Platform identification
  readonly platformId: string;
  readonly platformName: string;
  readonly supportedRegions: string[];

  // Catalog operations
  getCatalog(region: string, options?: CatalogOptions): Promise<CatalogResponse>;
  searchContent(query: string, region: string): Promise<SearchResult[]>;
  getContentDetails(platformContentId: string, region: string): Promise<ContentDetails>;

  // Availability operations
  checkAvailability(contentId: string, region: string): Promise<AvailabilityInfo>;
  getExpiringContent(region: string, daysAhead: number): Promise<ExpiringContent[]>;

  // Deep linking
  generateDeepLink(contentId: string, platform: 'ios' | 'android' | 'web'): DeepLink;

  // Rate limiting and health
  getRateLimitStatus(): RateLimitStatus;
  healthCheck(): Promise<HealthStatus>;
}
```

### 2.2 Netflix Integration Pattern

Netflix has no public API, so we use aggregator services with fallback chain:

```yaml
netflix_integration:
  primary_source: streaming_availability_api
  fallback_sources:
    - watchmode_api
    - justwatch_api

  data_flow:
    step_1_catalog_fetch:
      endpoint: "https://streaming-availability.p.rapidapi.com/shows/search/filters"
      method: POST
      body:
        country: "US"
        catalogs: ["netflix"]
        show_type: "all"
        output_language: "en"
      rate_limit: 100 req/min
      cache_ttl: 6 hours

    step_2_normalization:
      input: RawNetflixContent
      output: CanonicalContent
      mapping:
        - platform_id: "shows[].id"
        - title: "shows[].title"
        - overview: "shows[].overview"
        - release_year: "shows[].firstAirYear || shows[].year"
        - genres: "shows[].genres[].name"
        - rating: "shows[].rating / 20"  # Convert 0-100 to 0-5
        - images:
            poster: "shows[].imageSet.verticalPoster.w480"
            backdrop: "shows[].imageSet.horizontalPoster.w1440"

    step_3_entity_resolution:
      strategy: fuzzy_match
      identifiers:
        - imdb_id: "shows[].imdbId"
        - tmdb_id: "shows[].tmdbId"
      fallback: title_year_matching

    step_4_deep_link_generation:
      ios:
        scheme: "nflx://www.netflix.com/title/{netflix_id}"
        fallback: "https://www.netflix.com/title/{netflix_id}"
      android:
        intent: "intent://www.netflix.com/title/{netflix_id}#Intent;scheme=nflx;package=com.netflix.mediaclient;end"
      web:
        url: "https://www.netflix.com/title/{netflix_id}"

  circuit_breaker:
    failure_threshold: 5
    timeout_seconds: 30
    half_open_requests: 3
    reset_timeout: 60 seconds

  cache_strategy:
    catalog_cache: 6 hours
    content_details: 24 hours
    availability: 1 hour
    expiring_content: 15 minutes
```

### 2.3 YouTube Direct Integration

YouTube provides a full API via OAuth 2.0:

```yaml
youtube_integration:
  auth_method: oauth2_pkce
  scopes:
    - youtube.readonly
    - youtube.force-ssl

  api_endpoints:
    search:
      url: "https://www.googleapis.com/youtube/v3/search"
      quota_cost: 100 units
      params:
        - part: snippet
        - type: video
        - maxResults: 50
        - relevanceLanguage: en
        - safeSearch: moderate

    video_details:
      url: "https://www.googleapis.com/youtube/v3/videos"
      quota_cost: 1 unit
      params:
        - part: snippet,contentDetails,statistics
        - id: "{video_id}"

    channels:
      url: "https://www.googleapis.com/youtube/v3/channels"
      quota_cost: 1 unit
      params:
        - part: snippet,contentDetails,statistics
        - id: "{channel_id}"

  quota_management:
    daily_quota: 10000 units
    api_keys: 5  # Multi-key rotation
    rotation_strategy: round_robin
    quota_refresh: daily at 00:00 UTC
    alert_threshold: 80%

  rate_limiting:
    requests_per_second: 10
    requests_per_minute: 600
    burst_allowance: 20

  oauth_flow:
    authorization_url: "https://accounts.google.com/o/oauth2/v2/auth"
    token_url: "https://oauth2.googleapis.com/token"
    pkce:
      code_challenge_method: S256
      code_verifier_length: 128
    token_storage:
      location: secure_keystore
      encryption: AES-256-GCM
      refresh_strategy: automatic

  error_handling:
    quota_exceeded:
      action: switch_to_next_api_key
      fallback: return_cached_results
    auth_failure:
      action: trigger_reauth_flow
      user_notification: required
    rate_limit:
      action: exponential_backoff
      max_retries: 3
```

### 2.4 Prime Video Integration Pattern

```yaml
prime_video_integration:
  primary_source: streaming_availability_api
  fallback_sources:
    - watchmode_api

  data_normalization:
    platform_id: "prime_video"
    content_id_prefix: "amzn1.dv.gti."

    genre_mapping:
      "Action & Adventure": ["action", "adventure"]
      "Comedy": ["comedy"]
      "Drama": ["drama"]
      "Science Fiction": ["sci-fi"]
      "Documentary": ["documentary"]
      "Kids": ["family", "animation"]

  deep_linking:
    ios:
      scheme: "aiv://aiv/detail?asin={asin}"
      app_store_id: "545519333"
    android:
      intent: "intent://www.amazon.com/gp/video/detail/{asin}#Intent;scheme=https;package=com.amazon.avod.thirdpartyclient;end"
      package: "com.amazon.avod.thirdpartyclient"
    web:
      url: "https://www.amazon.com/gp/video/detail/{asin}"

  availability_tracking:
    check_frequency: 1 hour
    expiry_notifications: 7 days advance
    prime_member_detection: via_user_preferences
```

### 2.5 Multi-Key Rotation System

```typescript
class ApiKeyRotator {
  private keys: ApiKey[];
  private currentIndex: number = 0;
  private quotaTracking: Map<string, QuotaStatus>;

  constructor(keys: ApiKey[]) {
    this.keys = keys;
    this.quotaTracking = new Map();
  }

  async getNextAvailableKey(): Promise<ApiKey> {
    const startIndex = this.currentIndex;

    do {
      const key = this.keys[this.currentIndex];
      const quota = this.quotaTracking.get(key.id);

      // Check if key has available quota
      if (!quota || quota.remaining > quota.threshold) {
        this.currentIndex = (this.currentIndex + 1) % this.keys.length;
        return key;
      }

      // Try next key
      this.currentIndex = (this.currentIndex + 1) % this.keys.length;
    } while (this.currentIndex !== startIndex);

    // All keys exhausted
    throw new QuotaExhaustedError('All API keys have exceeded quota');
  }

  updateQuota(keyId: string, remaining: number, limit: number, resetAt: Date): void {
    this.quotaTracking.set(keyId, {
      remaining,
      limit,
      resetAt,
      threshold: limit * 0.2  // Alert at 20% remaining
    });
  }

  async waitForQuotaReset(): Promise<void> {
    // Find earliest reset time
    let earliestReset: Date | null = null;

    for (const quota of this.quotaTracking.values()) {
      if (!earliestReset || quota.resetAt < earliestReset) {
        earliestReset = quota.resetAt;
      }
    }

    if (earliestReset) {
      const waitTime = earliestReset.getTime() - Date.now();
      if (waitTime > 0) {
        await sleep(waitTime);
      }
    }
  }
}
```

---

## 3. Metadata Provider Integrations

### 3.1 TMDb Integration

```yaml
tmdb_integration:
  base_url: "https://api.themoviedb.org/3"
  auth_method: api_key

  endpoints:
    search_movie:
      path: "/search/movie"
      rate_limit: 40 requests / 10 seconds
      cache_ttl: 24 hours
      params:
        - query: required
        - year: optional
        - language: optional (default: en-US)
      response_mapping:
        - id: "results[].id"
        - imdb_id: requires_secondary_call
        - title: "results[].title"
        - overview: "results[].overview"
        - release_date: "results[].release_date"
        - poster_path: "results[].poster_path"
        - backdrop_path: "results[].backdrop_path"
        - genre_ids: "results[].genre_ids"

    movie_details:
      path: "/movie/{movie_id}"
      rate_limit: 40 requests / 10 seconds
      cache_ttl: 7 days
      params:
        - append_to_response: "credits,keywords,external_ids,videos"
      response_mapping:
        - imdb_id: "imdb_id"
        - runtime: "runtime"
        - genres: "genres[].name"
        - credits: "credits.cast[0:20]"
        - keywords: "keywords.keywords[].name"
        - trailers: "videos.results[?type='Trailer']"

    tv_details:
      path: "/tv/{tv_id}"
      similar_to: movie_details

  image_configuration:
    base_url: "https://image.tmdb.org/t/p/"
    poster_sizes:
      - w92
      - w154
      - w185
      - w342
      - w500
      - w780
      - original
    backdrop_sizes:
      - w300
      - w780
      - w1280
      - original

  caching_strategy:
    cache_layer: redis
    key_prefix: "tmdb:"
    ttl_by_endpoint:
      search: 24h
      details: 7d
      images: 30d
      configuration: 7d
    cache_invalidation:
      strategy: ttl_based
      manual_purge: supported

  error_handling:
    rate_limit_exceeded:
      action: exponential_backoff
      max_wait: 30s
      retry_after_header: respect
    not_found:
      action: try_fallback_provider
      fallback: imdb
    service_unavailable:
      action: return_cached_stale
      stale_ttl: 14d
```

### 3.2 IMDb Integration

```yaml
imdb_integration:
  # IMDb has no official public API
  source: web_scraping (fallback only)
  method: structured_data_extraction

  data_sources:
    primary: tmdb_external_ids
    fallback: web_scraping

  scraping_strategy:
    user_agent: "MediaGateway/1.0 (compatible; bot)"
    rate_limit: 1 request / 2 seconds
    cache_ttl: 30 days

    structured_data:
      format: json-ld
      schema_type: "Movie" | "TVSeries"
      extraction_fields:
        - name: "name"
        - description: "description"
        - rating: "aggregateRating.ratingValue"
        - rating_count: "aggregateRating.ratingCount"
        - duration: "duration"
        - genre: "genre"
        - actors: "actor[].name"
        - directors: "director[].name"

  legal_compliance:
    robots_txt: respect
    rate_limiting: strict
    caching: aggressive
    attribution: required
```

### 3.3 JustWatch Integration

```yaml
justwatch_integration:
  base_url: "https://apis.justwatch.com/content"
  auth_method: none (public endpoints)

  endpoints:
    search:
      path: "/titles/{locale}/popular"
      rate_limit: 100 requests / hour
      cache_ttl: 6 hours
      params:
        - content_types: movie,show
        - page: pagination
        - page_size: max 50

    title_details:
      path: "/titles/{title_id}/locale/{locale}"
      rate_limit: 100 requests / hour
      cache_ttl: 12 hours

    providers:
      path: "/providers/locale/{locale}"
      rate_limit: 10 requests / hour
      cache_ttl: 7 days

  data_enrichment:
    # JustWatch excels at streaming availability
    primary_use: availability_tracking
    secondary_use: pricing_information

    availability_mapping:
      - provider_id: netflix
        monetization_types: [flatrate]
      - provider_id: prime
        monetization_types: [flatrate, rent, buy]
      - provider_id: disney
        monetization_types: [flatrate]

  regional_support:
    supported_locales: 60+
    locale_format: "en_US"
    fallback_locale: "en_US"
```

### 3.4 Metadata Aggregation Strategy

```typescript
class MetadataAggregator {
  private providers: MetadataProvider[];

  async aggregateMetadata(contentId: string): Promise<EnrichedMetadata> {
    // Parallel fetch from all providers
    const results = await Promise.allSettled([
      this.tmdb.getDetails(contentId),
      this.imdb.getDetails(contentId),
      this.justwatch.getDetails(contentId),
      this.gracenote.getDetails(contentId)
    ]);

    // Merge strategy with priority
    const merged: EnrichedMetadata = {
      // Title: EIDR > Gracenote > TMDb > IMDb
      title: this.selectBestValue([
        results[3]?.value?.title,  // Gracenote
        results[0]?.value?.title,  // TMDb
        results[1]?.value?.title   // IMDb
      ]),

      // Description: Longest non-marketing description
      description: this.selectLongestNonMarketing([
        results[3]?.value?.description,
        results[0]?.value?.description,
        results[1]?.value?.description
      ]),

      // Runtime: Average of sources (within 5min variance)
      runtime: this.averageRuntimes([
        results[3]?.value?.runtime,
        results[0]?.value?.runtime,
        results[1]?.value?.runtime
      ]),

      // Genres: Union across all sources
      genres: this.unionGenres([
        results[3]?.value?.genres,
        results[0]?.value?.genres,
        results[1]?.value?.genres
      ]),

      // Rating: Weighted average (IMDb highest weight)
      rating: this.weightedAverageRating([
        { value: results[1]?.value?.rating, weight: 0.5 },  // IMDb
        { value: results[0]?.value?.rating, weight: 0.3 },  // TMDb
        { value: results[3]?.value?.rating, weight: 0.2 }   // Gracenote
      ]),

      // Availability: JustWatch is authoritative
      availability: results[2]?.value?.availability || [],

      // Images: Highest quality available
      images: this.selectBestImages([
        results[3]?.value?.images,
        results[0]?.value?.images
      ])
    };

    return merged;
  }

  private selectBestValue(values: (string | undefined)[]): string | undefined {
    return values.find(v => v !== undefined && v.length > 0);
  }

  private selectLongestNonMarketing(descriptions: (string | undefined)[]): string | undefined {
    const valid = descriptions.filter(d =>
      d && d.length > 50 && !this.isMarketingCopy(d)
    );
    return valid.sort((a, b) => b!.length - a!.length)[0];
  }

  private isMarketingCopy(text: string): boolean {
    const marketingPatterns = [
      /don't miss/i,
      /available now/i,
      /streaming exclusively/i,
      /watch now/i
    ];
    return marketingPatterns.some(pattern => pattern.test(text));
  }
}
```

---

## 4. PubNub Integration Architecture

### 4.1 Channel Architecture

```yaml
pubnub_integration:
  account_type: pro_tier
  monthly_transactions: 1M messages

  channel_naming_conventions:
    user_sync: "user.{user_id}.sync"
    device_status: "user.{user_id}.devices"
    watchlist: "user.{user_id}.watchlist"
    playback: "user.{user_id}.playback"
    notifications: "user.{user_id}.notifications"

    # Wildcard subscriptions
    user_all: "user.{user_id}.*"

  channel_configuration:
    user_sync:
      purpose: Cross-device state synchronization
      message_types:
        - watchlist_update
        - preference_change
        - watch_progress
        - device_handoff
      retention: 24 hours
      max_message_size: 32 KB

    device_status:
      purpose: Device presence and capabilities
      message_types:
        - device_online
        - device_offline
        - device_capabilities_update
      retention: 1 hour
      presence: enabled

    playback:
      purpose: Real-time playback control
      message_types:
        - play_command
        - pause_command
        - seek_command
        - quality_change
      retention: none (ephemeral)
      latency_slo: <100ms
```

### 4.2 Message Payload Standards

```typescript
// Base message structure
interface PubNubMessage {
  message_id: string;        // UUID for deduplication
  timestamp: number;         // Unix timestamp (milliseconds)
  version: string;           // Message schema version (e.g., "1.0.0")
  type: MessageType;         // Message type discriminator
  user_id: string;           // User identifier
  device_id: string;         // Originating device
  payload: unknown;          // Type-specific payload
  hlc?: HybridLogicalClock;  // For CRDT conflict resolution
}

// Watchlist update message
interface WatchlistUpdateMessage extends PubNubMessage {
  type: 'watchlist_update';
  payload: {
    operation: 'add' | 'remove' | 'reorder';
    content_id: string;
    position?: number;
    metadata?: {
      added_at: number;
      source: 'manual' | 'recommendation' | 'continue_watching';
    };
  };
}

// Device handoff message
interface DeviceHandoffMessage extends PubNubMessage {
  type: 'device_handoff';
  payload: {
    content_id: string;
    platform: string;
    progress_seconds: number;
    target_device_id: string;
    deep_link: string;
  };
}

// Playback control message
interface PlaybackControlMessage extends PubNubMessage {
  type: 'playback_control';
  payload: {
    action: 'play' | 'pause' | 'seek' | 'stop';
    content_id: string;
    position_seconds?: number;
    quality_tier?: 'auto' | 'sd' | 'hd' | '4k';
  };
}
```

### 4.3 Presence Configuration

```yaml
presence_configuration:
  user_devices:
    channel: "user.{user_id}.devices"
    timeout: 300 seconds
    announce_max: true
    interval: 10 seconds

    metadata:
      - device_type: "phone" | "tablet" | "tv" | "web" | "cli"
      - device_name: string
      - capabilities:
          supports_4k: boolean
          supports_hdr: boolean
          supports_atmos: boolean
      - current_activity:
          watching: content_id | null
          progress: seconds | null

  presence_events:
    join:
      action: broadcast_to_user_devices
      payload:
        - device_id
        - device_type
        - capabilities

    leave:
      action: broadcast_to_user_devices
      timeout: 300s
      grace_period: 30s

    timeout:
      action: mark_offline
      cleanup: remove_from_active_list

    state_change:
      action: broadcast_to_user_devices
      debounce: 5s
```

### 4.4 History and Persistence

```yaml
history_configuration:
  # PubNub Storage & Playback
  storage_ttl:
    user_sync: 24 hours
    device_status: 1 hour
    watchlist: 7 days
    playback: none (no storage)
    notifications: 30 days

  message_retrieval:
    max_count: 100 messages
    reverse: true (newest first)
    include_meta: true
    include_uuid: true

  persistence_strategy:
    pubnub_storage: short_term_sync
    postgresql: long_term_persistence

    sync_flow:
      1_message_published:
        - publish_to_pubnub
        - trigger_async_db_write

      2_client_offline_recovery:
        - fetch_from_pubnub_history (last 24h)
        - fetch_from_postgresql (older than 24h)
        - merge_and_deduplicate

      3_conflict_resolution:
        - apply_hlc_ordering
        - apply_crdt_merge_rules
```

### 4.5 SDK Integration

```typescript
class PubNubSyncClient {
  private pubnub: PubNub;
  private userId: string;
  private deviceId: string;
  private messageHandlers: Map<MessageType, Handler>;

  constructor(config: PubNubConfig) {
    this.pubnub = new PubNub({
      publishKey: config.publishKey,
      subscribeKey: config.subscribeKey,
      userId: config.userId,
      uuid: config.deviceId,

      // Presence configuration
      heartbeatInterval: 10,
      presenceTimeout: 300,

      // Restore on reconnection
      restore: true,

      // SSL required
      ssl: true
    });

    this.userId = config.userId;
    this.deviceId = config.deviceId;
    this.messageHandlers = new Map();

    this.setupListeners();
  }

  private setupListeners(): void {
    this.pubnub.addListener({
      message: (event) => this.handleMessage(event),
      presence: (event) => this.handlePresence(event),
      status: (event) => this.handleStatus(event)
    });
  }

  async subscribe(): Promise<void> {
    const channels = [
      `user.${this.userId}.sync`,
      `user.${this.userId}.watchlist`,
      `user.${this.userId}.notifications`
    ];

    const channelGroups = [
      `user.${this.userId}.all`
    ];

    this.pubnub.subscribe({
      channels,
      channelGroups,
      withPresence: true
    });

    // Set initial presence state
    await this.setPresenceState({
      device_id: this.deviceId,
      device_type: this.getDeviceType(),
      capabilities: this.getDeviceCapabilities()
    });
  }

  async publishWatchlistUpdate(update: WatchlistUpdate): Promise<void> {
    const message: WatchlistUpdateMessage = {
      message_id: uuid(),
      timestamp: Date.now(),
      version: '1.0.0',
      type: 'watchlist_update',
      user_id: this.userId,
      device_id: this.deviceId,
      payload: update,
      hlc: this.getHLC()
    };

    await this.pubnub.publish({
      channel: `user.${this.userId}.watchlist`,
      message,
      storeInHistory: true,
      ttl: 7 * 24  // 7 days
    });
  }

  async fetchHistory(channel: string, options?: HistoryOptions): Promise<Message[]> {
    const result = await this.pubnub.history({
      channel,
      count: options?.count || 100,
      reverse: true,
      includeMeta: true,
      includeUUID: true,
      start: options?.start,
      end: options?.end
    });

    return result.messages.map(m => ({
      message: m.entry,
      timetoken: m.timetoken,
      uuid: m.uuid,
      meta: m.meta
    }));
  }

  private handleMessage(event: PubNubMessageEvent): void {
    const message = event.message as PubNubMessage;

    // Deduplicate (ignore own messages)
    if (message.device_id === this.deviceId) {
      return;
    }

    // Route to appropriate handler
    const handler = this.messageHandlers.get(message.type);
    if (handler) {
      handler(message);
    }
  }

  private handlePresence(event: PubNubPresenceEvent): void {
    if (event.action === 'join') {
      console.log(`Device ${event.uuid} joined`);
    } else if (event.action === 'leave' || event.action === 'timeout') {
      console.log(`Device ${event.uuid} left`);
    }
  }

  private getHLC(): HybridLogicalClock {
    // Implement HLC for CRDT conflict resolution
    return {
      logical: this.logicalClock++,
      physical: Date.now(),
      node_id: this.deviceId
    };
  }
}
```

---

## 5. AI/ML Integrations

### 5.1 Embedding Service Architecture

```yaml
embedding_service:
  model: sentence-transformers/all-MiniLM-L6-v2
  embedding_dimension: 384
  deployment: cloud_run

  api_specification:
    endpoint: POST /v1/embeddings/generate
    authentication: bearer_token
    rate_limit: 1000 requests / minute

    request_schema:
      inputs:
        type: array
        items:
          type: string
          max_length: 512 tokens
        max_items: 32  # Batch size

      options:
        normalize: boolean (default: true)
        pooling: "mean" | "cls" (default: "mean")

    response_schema:
      embeddings:
        type: array
        items:
          type: array
          items: float
          length: 384

      metadata:
        model_version: string
        processing_time_ms: number
        token_count: number

  performance_targets:
    latency_p50: <50ms
    latency_p95: <200ms
    latency_p99: <500ms
    throughput: 500 embeddings/second

  caching_strategy:
    cache_layer: redis
    key_format: "emb:{model_version}:{sha256(input)}"
    ttl: 90 days
    cache_hit_target: >95%

  batch_processing:
    enabled: true
    max_batch_size: 32
    batch_timeout: 100ms
    dynamic_batching: true
```

### 5.2 LoRA Training Service

```yaml
lora_training_service:
  base_model: recommendation_transformer_v1
  lora_rank: 8
  lora_alpha: 16
  deployment: gpu_cloud_run

  training_pipeline:
    input:
      user_id: string
      viewing_events: array (min 10, max 100)
      training_mode: "incremental" | "full_retrain"

    steps:
      1_data_preparation:
        - filter_low_engagement (threshold: 0.3)
        - apply_temporal_decay (rate: 0.95)
        - generate_embeddings (cached)
        - prepare_training_pairs

      2_lora_adaptation:
        optimizer: adamw
        learning_rate: 0.001
        epochs: 5
        batch_size: 16
        loss_function: binary_cross_entropy

      3_validation:
        holdout_split: 0.2
        metrics:
          - precision_at_k (k=5)
          - ndcg_at_k (k=10)
          - user_cold_start_performance

      4_deployment:
        artifact: user_lora_weights.safetensors
        storage: gcs://media-gateway/user-models/{user_id}/
        version_tracking: enabled

    output:
      model_artifact: safetensors
      metrics: json
      training_log: jsonl

  inference_api:
    endpoint: POST /v1/lora/inference
    request:
      user_id: string
      candidate_content_ids: array (max 100)
    response:
      scores: array<float>
      latency_ms: number

  performance_targets:
    training_latency: <10 seconds
    inference_latency_p95: <5ms per candidate
    model_size: ~10KB per user
    concurrent_trainings: 100
```

### 5.3 ML Model Versioning

```yaml
model_versioning:
  strategy: semantic_versioning

  model_registry:
    location: gcs://media-gateway/models/
    metadata_store: postgresql

    model_metadata:
      - model_id: uuid
      - name: string
      - version: semver
      - created_at: timestamp
      - trained_on: dataset_version
      - metrics: json
      - deployment_status: "staging" | "production" | "archived"
      - sha256_hash: string

  deployment_strategy:
    canary_rollout:
      initial_traffic: 5%
      increment_percentage: 10%
      increment_interval: 1 hour
      success_criteria:
        - error_rate <1%
        - latency_p95 <500ms
        - user_engagement_delta >-5%

    rollback_conditions:
      - error_rate >5%
      - latency_p99 >2000ms
      - user_negative_feedback >10%

    blue_green:
      enabled: true
      traffic_split: weighted
      monitoring_period: 24 hours

  a_b_testing:
    framework: google_optimize
    experiments:
      - embedding_model_comparison
      - lora_rank_tuning
      - recommendation_strategy

    statistical_confidence: 95%
    minimum_sample_size: 1000 users per variant
```

---

## 6. Webhook Architecture

### 6.1 Inbound Webhooks (Platform Callbacks)

```yaml
inbound_webhooks:
  purpose: Receive real-time updates from platforms

  endpoints:
    platform_catalog_update:
      path: /webhooks/platform/{platform_id}/catalog
      method: POST
      authentication: hmac_sha256

      payload_schema:
        event_type: "content_added" | "content_removed" | "availability_changed"
        platform_id: string
        content_id: string
        region: string
        timestamp: iso8601
        changes: json

      processing:
        - verify_signature
        - validate_payload
        - publish_to_kafka (topic: platform.updates)
        - acknowledge_receipt (200 OK)

      retry_policy:
        max_retries: 3
        backoff: exponential (1s, 2s, 4s)

    youtube_subscription_notification:
      path: /webhooks/youtube/pubsubhubbub
      method: POST
      authentication: google_pubsubhubbub

      payload_schema:
        hub_challenge: string (for verification)
        feed: atom_xml

      processing:
        - verify_subscription
        - parse_atom_feed
        - extract_video_updates
        - trigger_content_sync

  signature_verification:
    algorithm: hmac_sha256
    secret_storage: google_secret_manager
    header: X-Webhook-Signature

    verification_flow:
      1_extract_signature: from_header
      2_compute_expected: hmac_sha256(secret, body)
      3_compare: constant_time_comparison
      4_reject_if_mismatch: 401 Unauthorized

  idempotency:
    key_header: X-Idempotency-Key
    storage: redis
    ttl: 24 hours

    deduplication:
      - check_idempotency_key
      - if_exists: return_cached_response (200 OK)
      - if_new: process_and_cache_response
```

### 6.2 Outbound Webhooks (Notifications)

```yaml
outbound_webhooks:
  purpose: Notify external systems of events

  webhook_subscriptions:
    storage: postgresql
    schema:
      - subscription_id: uuid
      - user_id: uuid
      - url: url
      - events: array<event_type>
      - secret: encrypted_string
      - created_at: timestamp
      - status: "active" | "paused" | "failed"
      - failure_count: integer

  event_types:
    - content.watchlist.added
    - content.watchlist.removed
    - content.availability.expiring_soon
    - content.recommendation.ready
    - user.watch_progress.updated

  delivery_mechanism:
    endpoint: POST {subscriber_url}
    method: POST
    headers:
      Content-Type: application/json
      X-Webhook-Signature: hmac_sha256(secret, body)
      X-Webhook-Event: event_type
      X-Webhook-ID: event_id
      X-Webhook-Timestamp: unix_timestamp

    body_schema:
      event_id: uuid
      event_type: string
      timestamp: iso8601
      user_id: uuid
      data: json

  retry_policy:
    max_attempts: 5
    backoff_strategy: exponential
    backoff_multiplier: 2
    initial_delay: 5 seconds
    max_delay: 1 hour

    retry_schedule:
      - attempt_1: immediate
      - attempt_2: +5s
      - attempt_3: +10s
      - attempt_4: +20s
      - attempt_5: +40s

    failure_handling:
      after_5_failures:
        - mark_subscription_as_failed
        - send_email_notification_to_subscriber
        - pause_webhook_delivery

      recovery:
        - manual_reactivation_required
        - test_endpoint_before_resume

  monitoring:
    metrics:
      - delivery_success_rate
      - delivery_latency_p95
      - failure_rate_by_subscriber
      - retry_rate

    alerts:
      - delivery_failure_rate >10% (warning)
      - delivery_failure_rate >25% (critical)
      - avg_delivery_latency >5s (warning)
```

### 6.3 Webhook Security

```typescript
class WebhookSecurityManager {
  async verifyInboundWebhook(
    signature: string,
    body: string,
    platformId: string
  ): Promise<boolean> {
    const secret = await this.getSecret(platformId);
    const expectedSignature = this.computeHMAC(secret, body);

    // Constant-time comparison to prevent timing attacks
    return this.constantTimeCompare(signature, expectedSignature);
  }

  async signOutboundWebhook(
    body: string,
    subscriptionSecret: string
  ): Promise<string> {
    return this.computeHMAC(subscriptionSecret, body);
  }

  private computeHMAC(secret: string, data: string): string {
    return crypto
      .createHmac('sha256', secret)
      .update(data)
      .digest('hex');
  }

  private constantTimeCompare(a: string, b: string): boolean {
    if (a.length !== b.length) {
      return false;
    }

    let mismatch = 0;
    for (let i = 0; i < a.length; i++) {
      mismatch |= a.charCodeAt(i) ^ b.charCodeAt(i);
    }

    return mismatch === 0;
  }

  async checkIdempotency(key: string): Promise<{
    exists: boolean;
    response?: WebhookResponse;
  }> {
    const cached = await this.redis.get(`idempotency:${key}`);

    if (cached) {
      return {
        exists: true,
        response: JSON.parse(cached)
      };
    }

    return { exists: false };
  }

  async cacheIdempotentResponse(
    key: string,
    response: WebhookResponse,
    ttl: number = 86400
  ): Promise<void> {
    await this.redis.setex(
      `idempotency:${key}`,
      ttl,
      JSON.stringify(response)
    );
  }
}
```

---

## 7. Event-Driven Integration

### 7.1 Event Bus Architecture (Kafka)

```yaml
kafka_configuration:
  cluster: confluent_cloud
  region: us-central1

  topics:
    content_ingested:
      partitions: 12
      replication_factor: 3
      retention_ms: 604800000  # 7 days
      compression: snappy

      schema:
        event_type: "content.ingested"
        content_id: string
        platform_id: string
        region: string
        ingestion_timestamp: iso8601
        canonical_content: json

    content_updated:
      partitions: 12
      replication_factor: 3
      retention_ms: 604800000

      schema:
        event_type: "content.updated"
        content_id: string
        changed_fields: array<string>
        previous_values: json
        new_values: json
        timestamp: iso8601

    availability_changed:
      partitions: 12
      replication_factor: 3
      retention_ms: 2592000000  # 30 days

      schema:
        event_type: "availability.changed"
        content_id: string
        platform_id: string
        region: string
        change_type: "added" | "removed" | "price_changed" | "expiring"
        availability: json
        effective_date: iso8601

    user_interaction:
      partitions: 24
      replication_factor: 3
      retention_ms: 2592000000  # 30 days

      schema:
        event_type: "user.interaction"
        user_id: string
        interaction_type: "view" | "search" | "click" | "watch" | "rate"
        content_id: string (nullable)
        metadata: json
        timestamp: iso8601

  producers:
    configuration:
      acks: all
      retries: 3
      compression_type: snappy
      max_in_flight_requests: 5
      idempotence: true

    error_handling:
      serialization_error: log_and_discard
      network_error: retry_with_backoff
      quota_exceeded: throttle

  consumers:
    content_indexer:
      group_id: content-indexer-v1
      topics: [content_ingested, content_updated]
      offset_reset: earliest
      enable_auto_commit: false
      max_poll_records: 500

      processing:
        - deserialize_event
        - update_ruvector_index
        - update_postgresql
        - commit_offset

    recommendation_updater:
      group_id: recommendation-updater-v1
      topics: [user_interaction, content_updated]
      offset_reset: latest
      enable_auto_commit: true

      processing:
        - deserialize_event
        - update_user_profile
        - trigger_lora_training (if threshold met)
        - update_trending_scores

    availability_monitor:
      group_id: availability-monitor-v1
      topics: [availability_changed]
      offset_reset: earliest

      processing:
        - deserialize_event
        - check_user_watchlists
        - send_expiry_notifications
        - update_cache
```

### 7.2 Event Schema Registry

```yaml
schema_registry:
  provider: confluent_schema_registry
  compatibility_mode: backward

  schemas:
    content_event_v1:
      type: avro
      schema:
        type: record
        name: ContentEvent
        namespace: com.mediagateway.events
        fields:
          - name: event_id
            type: string
          - name: event_type
            type: string
          - name: timestamp
            type: long
            logicalType: timestamp-millis
          - name: content_id
            type: string
          - name: payload
            type:
              type: record
              name: ContentPayload
              fields:
                - name: canonical_content
                  type: string  # JSON encoded

    user_event_v1:
      type: avro
      schema:
        type: record
        name: UserEvent
        namespace: com.mediagateway.events
        fields:
          - name: event_id
            type: string
          - name: user_id
            type: string
          - name: event_type
            type: string
          - name: timestamp
            type: long
            logicalType: timestamp-millis
          - name: metadata
            type: map
            values: string

  evolution_strategy:
    - add_optional_fields: allowed
    - remove_fields: prohibited
    - rename_fields: prohibited (breaking change)
    - change_field_type: prohibited (breaking change)

  version_migration:
    - deploy_new_schema_version
    - update_producers (rolling)
    - wait_for_consumer_lag_zero
    - update_consumers (rolling)
    - deprecate_old_schema (after 30 days)
```

### 7.3 Consumer Patterns

```typescript
class KafkaConsumerGroup {
  private consumer: Kafka.Consumer;
  private handlers: Map<string, EventHandler>;

  async start(): Promise<void> {
    await this.consumer.subscribe({
      topics: ['content_ingested', 'content_updated'],
      fromBeginning: false
    });

    await this.consumer.run({
      partitionsConsumedConcurrently: 3,

      eachMessage: async ({ topic, partition, message }) => {
        const event = this.deserializeEvent(message);

        try {
          // Process event
          await this.processEvent(event);

          // Manual commit for at-least-once processing
          await this.consumer.commitOffsets([{
            topic,
            partition,
            offset: (parseInt(message.offset) + 1).toString()
          }]);

        } catch (error) {
          // Send to DLQ and continue
          await this.sendToDLQ(event, error);
        }
      }
    });
  }

  private async processEvent(event: ContentEvent): Promise<void> {
    const handler = this.handlers.get(event.event_type);

    if (!handler) {
      console.warn(`No handler for event type: ${event.event_type}`);
      return;
    }

    await handler.handle(event);
  }

  private async sendToDLQ(event: ContentEvent, error: Error): Promise<void> {
    await this.dlqProducer.send({
      topic: 'content_events_dlq',
      messages: [{
        key: event.content_id,
        value: JSON.stringify({
          original_event: event,
          error: {
            message: error.message,
            stack: error.stack
          },
          failed_at: Date.now()
        })
      }]
    });
  }
}
```

### 7.4 Dead Letter Queue (DLQ) Strategy

```yaml
dlq_configuration:
  topics:
    content_events_dlq:
      retention_ms: 2592000000  # 30 days
      cleanup_policy: delete

    user_events_dlq:
      retention_ms: 2592000000
      cleanup_policy: delete

  processing:
    manual_review:
      frequency: daily
      dashboard: admin_ui
      actions:
        - retry_with_fixes
        - discard_permanently
        - update_schema

    automatic_retry:
      enabled: true
      conditions:
        - error_type: transient
        - retry_count: <3
      schedule:
        - retry_1: +1 hour
        - retry_2: +6 hours
        - retry_3: +24 hours

  alerting:
    dlq_size_threshold: 1000 messages
    dlq_age_threshold: 24 hours
    notification_channels:
      - slack: #media-gateway-alerts
      - pagerduty: on_call_engineer
```

---

## 8. Third-Party Service Abstraction

### 8.1 Adapter Pattern Implementation

```typescript
// Base adapter interface
interface ServiceAdapter<TRequest, TResponse> {
  readonly serviceName: string;
  readonly version: string;

  execute(request: TRequest): Promise<TResponse>;
  healthCheck(): Promise<HealthStatus>;
  getRateLimitStatus(): RateLimitStatus;
}

// Circuit breaker wrapper
class CircuitBreakerAdapter<TRequest, TResponse> implements ServiceAdapter<TRequest, TResponse> {
  private state: 'CLOSED' | 'OPEN' | 'HALF_OPEN' = 'CLOSED';
  private failureCount: number = 0;
  private successCount: number = 0;
  private lastFailureTime: number = 0;

  constructor(
    private wrapped: ServiceAdapter<TRequest, TResponse>,
    private config: CircuitBreakerConfig
  ) {}

  get serviceName(): string {
    return this.wrapped.serviceName;
  }

  get version(): string {
    return this.wrapped.version;
  }

  async execute(request: TRequest): Promise<TResponse> {
    // Check circuit state
    if (this.state === 'OPEN') {
      if (Date.now() - this.lastFailureTime > this.config.resetTimeout) {
        this.state = 'HALF_OPEN';
        this.successCount = 0;
      } else {
        throw new CircuitOpenError(`Circuit open for ${this.serviceName}`);
      }
    }

    try {
      const response = await this.wrapped.execute(request);
      this.onSuccess();
      return response;

    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  private onSuccess(): void {
    this.failureCount = 0;

    if (this.state === 'HALF_OPEN') {
      this.successCount++;

      if (this.successCount >= this.config.halfOpenSuccessThreshold) {
        this.state = 'CLOSED';
        this.successCount = 0;
      }
    }
  }

  private onFailure(): void {
    this.failureCount++;
    this.lastFailureTime = Date.now();

    if (this.failureCount >= this.config.failureThreshold) {
      this.state = 'OPEN';
    }
  }

  async healthCheck(): Promise<HealthStatus> {
    if (this.state === 'OPEN') {
      return {
        status: 'unhealthy',
        reason: 'Circuit breaker open',
        circuit_state: this.state
      };
    }

    return this.wrapped.healthCheck();
  }

  getRateLimitStatus(): RateLimitStatus {
    return this.wrapped.getRateLimitStatus();
  }
}

// Cache-aside adapter
class CachedAdapter<TRequest, TResponse> implements ServiceAdapter<TRequest, TResponse> {
  constructor(
    private wrapped: ServiceAdapter<TRequest, TResponse>,
    private cache: CacheClient,
    private ttl: number
  ) {}

  get serviceName(): string {
    return this.wrapped.serviceName;
  }

  get version(): string {
    return this.wrapped.version;
  }

  async execute(request: TRequest): Promise<TResponse> {
    const cacheKey = this.generateCacheKey(request);

    // Try cache first
    const cached = await this.cache.get<TResponse>(cacheKey);
    if (cached) {
      return cached;
    }

    // Cache miss - call wrapped service
    const response = await this.wrapped.execute(request);

    // Store in cache
    await this.cache.set(cacheKey, response, this.ttl);

    return response;
  }

  private generateCacheKey(request: TRequest): string {
    const serialized = JSON.stringify(request);
    const hash = crypto.createHash('sha256').update(serialized).digest('hex');
    return `${this.serviceName}:${this.version}:${hash}`;
  }

  async healthCheck(): Promise<HealthStatus> {
    return this.wrapped.healthCheck();
  }

  getRateLimitStatus(): RateLimitStatus {
    return this.wrapped.getRateLimitStatus();
  }
}
```

### 8.2 Fallback Strategies

```typescript
class FallbackChainAdapter<TRequest, TResponse> implements ServiceAdapter<TRequest, TResponse> {
  constructor(
    private adapters: ServiceAdapter<TRequest, TResponse>[],
    private strategy: FallbackStrategy
  ) {
    if (adapters.length === 0) {
      throw new Error('At least one adapter required');
    }
  }

  get serviceName(): string {
    return `FallbackChain(${this.adapters.map(a => a.serviceName).join(', ')})`;
  }

  get version(): string {
    return this.adapters[0].version;
  }

  async execute(request: TRequest): Promise<TResponse> {
    const errors: Error[] = [];

    for (let i = 0; i < this.adapters.length; i++) {
      const adapter = this.adapters[i];

      try {
        const response = await adapter.execute(request);

        // Log successful fallback
        if (i > 0) {
          console.warn(`Fallback to ${adapter.serviceName} succeeded after ${i} failures`);
        }

        return response;

      } catch (error) {
        errors.push(error as Error);
        console.error(`Adapter ${adapter.serviceName} failed:`, error);

        // Continue to next adapter
        if (i < this.adapters.length - 1) {
          continue;
        }
      }
    }

    // All adapters failed
    throw new AllAdaptersFailedError(
      `All ${this.adapters.length} adapters failed`,
      errors
    );
  }

  async healthCheck(): Promise<HealthStatus> {
    const results = await Promise.allSettled(
      this.adapters.map(a => a.healthCheck())
    );

    const healthy = results.filter(r =>
      r.status === 'fulfilled' && r.value.status === 'healthy'
    ).length;

    return {
      status: healthy > 0 ? 'healthy' : 'unhealthy',
      adapters_healthy: healthy,
      adapters_total: this.adapters.length,
      details: results
    };
  }

  getRateLimitStatus(): RateLimitStatus {
    // Return status of primary adapter
    return this.adapters[0].getRateLimitStatus();
  }
}
```

### 8.3 Health Monitoring

```yaml
health_monitoring:
  health_check_endpoints:
    streaming_availability:
      url: https://streaming-availability.p.rapidapi.com/health
      interval: 60 seconds
      timeout: 5 seconds
      expected_status: 200

    tmdb:
      url: https://api.themoviedb.org/3/configuration
      interval: 60 seconds
      timeout: 5 seconds
      expected_status: 200

    pubnub:
      url: https://ps.pndsn.com/time/0
      interval: 30 seconds
      timeout: 3 seconds
      expected_status: 200

    embedding_service:
      url: https://embedding-service.run.app/health
      interval: 30 seconds
      timeout: 5 seconds
      expected_status: 200

  health_aggregation:
    overall_health:
      calculation: weighted_average
      weights:
        streaming_availability: 0.3
        tmdb: 0.2
        pubnub: 0.3
        embedding_service: 0.2

      status_thresholds:
        healthy: all_critical_services_up
        degraded: any_critical_service_degraded
        unhealthy: any_critical_service_down

  alerting:
    service_down:
      severity: critical
      notification: immediate
      channels: [pagerduty, slack]

    service_degraded:
      severity: warning
      notification: 5 minute delay
      channels: [slack]

    high_error_rate:
      threshold: 5%
      window: 5 minutes
      severity: warning
      channels: [slack]
```

---

## 9. Integration Testing Strategy

### 9.1 Mock Services

```typescript
// Mock platform adapter for testing
class MockPlatformAdapter implements PlatformAdapter {
  private catalog: Map<string, ContentDetails> = new Map();
  private failureMode: 'none' | 'timeout' | 'rate_limit' | 'error' = 'none';

  constructor(
    public readonly platformId: string,
    public readonly platformName: string,
    private testData?: TestDataSet
  ) {
    if (testData) {
      this.loadTestData(testData);
    }
  }

  setFailureMode(mode: 'none' | 'timeout' | 'rate_limit' | 'error'): void {
    this.failureMode = mode;
  }

  async getCatalog(region: string, options?: CatalogOptions): Promise<CatalogResponse> {
    await this.simulateFailure();

    const items = Array.from(this.catalog.values())
      .filter(item => item.regions.includes(region))
      .slice(0, options?.limit || 100);

    return {
      items,
      total: items.length,
      page: options?.page || 1
    };
  }

  async searchContent(query: string, region: string): Promise<SearchResult[]> {
    await this.simulateFailure();

    return Array.from(this.catalog.values())
      .filter(item =>
        item.title.toLowerCase().includes(query.toLowerCase()) &&
        item.regions.includes(region)
      )
      .map(item => ({
        content: item,
        relevance: 0.9
      }));
  }

  async getContentDetails(platformContentId: string, region: string): Promise<ContentDetails> {
    await this.simulateFailure();

    const content = this.catalog.get(platformContentId);
    if (!content) {
      throw new NotFoundError(`Content not found: ${platformContentId}`);
    }

    return content;
  }

  generateDeepLink(contentId: string, platform: 'ios' | 'android' | 'web'): DeepLink {
    return {
      url: `mock://${this.platformId}/${contentId}`,
      fallback: `https://mock.${this.platformId}.com/${contentId}`
    };
  }

  private async simulateFailure(): Promise<void> {
    switch (this.failureMode) {
      case 'timeout':
        await new Promise(resolve => setTimeout(resolve, 30000));
        break;
      case 'rate_limit':
        throw new RateLimitError('Rate limit exceeded (mock)');
      case 'error':
        throw new Error('Mock service error');
      case 'none':
        // No failure
        break;
    }
  }

  private loadTestData(testData: TestDataSet): void {
    for (const item of testData.content) {
      this.catalog.set(item.platform_content_id, item);
    }
  }

  async healthCheck(): Promise<HealthStatus> {
    return {
      status: this.failureMode === 'none' ? 'healthy' : 'unhealthy',
      mode: this.failureMode
    };
  }

  getRateLimitStatus(): RateLimitStatus {
    return {
      remaining: this.failureMode === 'rate_limit' ? 0 : 100,
      limit: 100,
      reset_at: new Date(Date.now() + 3600000)
    };
  }
}
```

### 9.2 Contract Testing

```yaml
contract_testing:
  framework: pact

  consumer_contracts:
    media_gateway_api:
      provider: streaming_availability_api

      interactions:
        search_netflix_content:
          request:
            method: POST
            path: /shows/search/filters
            headers:
              X-RapidAPI-Key: "${API_KEY}"
            body:
              country: "US"
              catalogs: ["netflix"]

          response:
            status: 200
            headers:
              Content-Type: "application/json"
            body:
              shows:
                - id: "netflix:12345"
                  title: "Example Movie"
                  type: "movie"
                  year: 2023

        get_content_details:
          request:
            method: GET
            path: /shows/{id}
            path_params:
              id: "netflix:12345"

          response:
            status: 200
            body:
              id: "netflix:12345"
              title: "Example Movie"
              imdbId: "tt1234567"
              # ... additional fields

  provider_verification:
    # Provider must verify contracts
    streaming_availability_mock:
      pact_files: ./pacts/*.json
      provider_base_url: http://localhost:8080
      provider_states:
        - "content exists"
        - "content not found"

  ci_integration:
    publish_pacts: true
    broker_url: https://pact-broker.mediagateway.io
    can_i_deploy: true
```

### 9.3 Integration Test Environments

```yaml
test_environments:
  local:
    purpose: Developer testing
    components:
      - docker-compose setup
      - mock services
      - local postgres
      - local redis

    services:
      mock_streaming_api:
        image: wiremock/wiremock
        port: 8081
        mappings: ./test/mocks/streaming_api

      mock_tmdb:
        image: wiremock/wiremock
        port: 8082
        mappings: ./test/mocks/tmdb

      postgres:
        image: postgres:15
        port: 5432

      redis:
        image: redis:7
        port: 6379

  staging:
    purpose: Pre-production testing
    components:
      - real services with test API keys
      - staging database
      - staging cache
      - limited quota

    external_services:
      streaming_availability:
        use: real_api
        quota: 1000 requests/day
        api_key: staging_key

      tmdb:
        use: real_api
        quota: 5000 requests/day

      pubnub:
        use: real_service
        keyset: staging_keyset

  production:
    purpose: Live production
    components:
      - production services
      - production database
      - production cache
      - full quota

    safety:
      - gradual_rollout: canary
      - feature_flags: enabled
      - rollback_capability: automated
```

### 9.4 Load Testing

```yaml
load_testing:
  framework: k6

  test_scenarios:
    search_load:
      target: 1000 rps
      duration: 10 minutes
      script: |
        export default function() {
          http.post('https://api.mediagateway.io/v1/search', {
            query: 'action movies',
            limit: 20
          });
        }

      thresholds:
        http_req_duration: ['p(95)<500']
        http_req_failed: ['rate<0.01']

    recommendation_load:
      target: 500 rps
      duration: 10 minutes
      script: |
        export default function() {
          http.get('https://api.mediagateway.io/v1/recommendations', {
            headers: {
              'Authorization': `Bearer ${__ENV.TEST_TOKEN}`
            }
          });
        }

      thresholds:
        http_req_duration: ['p(95)<200']
        http_req_failed: ['rate<0.01']

    platform_catalog_sync:
      vus: 10
      duration: 30 minutes
      script: |
        export default function() {
          // Simulate catalog ingestion load
          for (let platform of platforms) {
            http.get(`https://api.mediagateway.io/v1/platforms/${platform}/sync`);
          }
          sleep(60);  // Run every minute
        }

  performance_budgets:
    api_latency_p95: 500ms
    api_latency_p99: 1000ms
    error_rate: <1%
    throughput: >1000 rps
```

---

## Summary

This integration architecture provides:

1. **Robust Platform Integration**: Adapter pattern with circuit breakers and fallback chains
2. **Metadata Enrichment**: Multi-source aggregation with conflict resolution
3. **Real-time Sync**: PubNub integration for <100ms cross-device synchronization
4. **AI/ML Pipeline**: Embedding service and LoRA training integration
5. **Event-Driven**: Kafka-based event bus for scalable event processing
6. **Webhook Support**: Bi-directional webhooks with security and idempotency
7. **Service Abstraction**: Uniform interfaces with health monitoring
8. **Comprehensive Testing**: Mock services, contract tests, and load testing

**Next Steps:**
- Implement adapter interfaces in Rust
- Set up Kafka topics and schema registry
- Configure PubNub channels and presence
- Deploy ML services to Cloud Run
- Establish monitoring and alerting

---

**Document Status:** Complete
**Review Required:** Platform team, ML team, DevOps team
**Related Documents:**
- SPARC Architecture Part 1 (System Design)
- SPARC Architecture Part 2 (Data Architecture)
- SPARC Pseudocode Part 1 (Data Structures)
- SPARC Pseudocode Part 2 (Search & SONA)

---

END OF PART 3
