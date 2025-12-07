# HybridSearchService Cache Integration Architecture

## Request Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Client Request                                │
│                     SearchRequest {                                  │
│                       query: "action movies"                         │
│                       filters: {genres: ["action"]}                  │
│                       page: 1, page_size: 20                         │
│                       user_id: Some(uuid)                            │
│                     }                                                │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    HybridSearchService::search()                     │
│                                                                      │
│  Step 1: Generate Cache Key                                         │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ SHA256(JSON.serialize(request))                            │    │
│  │ → "search:a3f8e9d2c1b4..."                                 │    │
│  └────────────────────────────────────────────────────────────┘    │
│                               │                                      │
│                               ▼                                      │
│  Step 2: Check Cache                                                │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ cache.get::<SearchResponse>(&key)                          │    │
│  └────────────────────────────────────────────────────────────┘    │
│                               │                                      │
│                ┌──────────────┴──────────────┐                      │
│                ▼                              ▼                      │
│         ┌─────────────┐              ┌──────────────┐              │
│         │ CACHE HIT   │              │ CACHE MISS   │              │
│         │  <10ms      │              │  ~200ms      │              │
│         └──────┬──────┘              └──────┬───────┘              │
│                │                              │                      │
│                │                              ▼                      │
│                │               ┌──────────────────────────────┐    │
│                │               │ execute_search(request)      │    │
│                │               │                              │    │
│                │               │ 1. Parse Intent (GPT-4o)     │    │
│                │               │ 2. Vector Search (Qdrant)    │    │
│                │               │ 3. Keyword Search (Tantivy)  │    │
│                │               │ 4. Merge with RRF            │    │
│                │               │ 5. Personalize               │    │
│                │               │ 6. Paginate                  │    │
│                │               └──────────┬───────────────────┘    │
│                │                          │                         │
│                │                          ▼                         │
│                │               ┌──────────────────────────────┐    │
│                │               │ cache.set(key, response,     │    │
│                │               │           ttl=1800s)         │    │
│                │               └──────────┬───────────────────┘    │
│                │                          │                         │
│                └──────────────┬───────────┘                         │
│                               │                                      │
│                               ▼                                      │
│                    Return SearchResponse                             │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │   Client receives    │
                    │   SearchResponse     │
                    └──────────────────────┘
```

## Cache Key Generation

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SearchRequest Components                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  query:       "action movies"                                       │
│  filters:     {genres: ["action"], platforms: ["netflix"],          │
│                year_range: (2020, 2024)}                            │
│  page:        1                                                     │
│  page_size:   20                                                    │
│  user_id:     "550e8400-e29b-41d4-a716-446655440000"               │
│                                                                      │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
                    JSON Serialization
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│ {"query":"action movies","filters":{"genres":["action"],           │
│  "platforms":["netflix"],"year_range":[2020,2024]},                │
│  "page":1,"page_size":20,"user_id":"550e8400..."}                  │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
                        SHA256 Hash
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│ a3f8e9d2c1b4a6f5e8d7c9b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9e8d7c6b5a4f3 │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
                      Add Prefix
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│        Final Cache Key (72 characters total)                        │
│ search:a3f8e9d2c1b4a6f5e8d7c9b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9e8... │
└─────────────────────────────────────────────────────────────────────┘
```

## Performance Comparison

### Before Cache Integration
```
Request → Search Service → Parse Intent (50ms)
                        → Vector Search (100ms)
                        → Keyword Search (80ms)
                        → Merge Results (10ms)
                        → Personalize (5ms)
                        → Paginate (5ms)
                        ────────────────────────
Total: ~250ms per request (regardless of repetition)
```

### After Cache Integration
```
First Request (Cache Miss):
Request → Cache Check (2ms) → Full Search (250ms) → Cache Write (3ms)
Total: ~255ms (one-time overhead)

Subsequent Requests (Cache Hit):
Request → Cache Check (2ms) → Return Cached
Total: ~2-5ms (50-100x faster!)
```

## Data Flow

```
┌──────────────┐
│   Client     │
└──────┬───────┘
       │ SearchRequest
       ▼
┌──────────────────────────────┐
│  HybridSearchService         │
│  ┌────────────────────────┐  │
│  │ generate_cache_key()   │  │
│  └──────────┬─────────────┘  │
│             │ cache_key      │
│             ▼                 │
│  ┌────────────────────────┐  │
│  │ cache.get(key)         │◄─┼────┐
│  └──────────┬─────────────┘  │    │
│             │                 │    │
│       ┌─────┴─────┐          │    │
│       ▼           ▼          │    │
│   Hit(data)   Miss            │    │
│       │           │           │    │
│       │           ▼           │    │
│       │  ┌─────────────────┐ │    │
│       │  │ execute_search()│ │    │
│       │  └────────┬────────┘ │    │
│       │           │ response  │    │
│       │           ▼           │    │
│       │  ┌─────────────────┐ │    │
│       │  │ cache.set(key,  │─┼────┘
│       │  │   response, TTL)│ │ Write to Redis
│       │  └─────────────────┘ │
│       └─────┬─────┘           │
│             │                 │
│             ▼                 │
│        SearchResponse         │
└──────────┬───────────────────┘
           │
           ▼
    ┌─────────────┐
    │ Redis Cache │
    │             │
    │ ┌─────────┐ │
    │ │search:  │ │
    │ │ a3f8... │ │ TTL: 1800s
    │ │ ├─data  │ │
    │ └─────────┘ │
    └─────────────┘
```

## Cache Key Uniqueness

```
Same Query, Same Filters, Same Page, Same User:
  → SAME cache key → Cache Hit ✓

Different Page:
  query: "action movies", page: 1 → search:a3f8...
  query: "action movies", page: 2 → search:b7e2... (different key)

Different Filters:
  filters: {genres: ["action"]} → search:a3f8...
  filters: {genres: ["action", "thriller"]} → search:c9d4... (different key)

Different User:
  user_id: user_1 → search:a3f8...
  user_id: user_2 → search:e5f6... (different key for personalization)

Different Query:
  query: "action movies" → search:a3f8...
  query: "drama series" → search:f7g8... (different key)
```

## TTL Lifecycle

```
Time 0s:
  ┌─────────────────────────────┐
  │ Cache Write                 │
  │ TTL: 1800s (30 min)        │
  └─────────────────────────────┘

Time 0-1800s:
  ┌─────────────────────────────┐
  │ Cache Hit                   │
  │ Returns in <10ms            │
  │ TTL counting down...        │
  └─────────────────────────────┘

Time 1800s:
  ┌─────────────────────────────┐
  │ Cache Expired               │
  │ Redis removes key           │
  └─────────────────────────────┘

Time 1800s+:
  ┌─────────────────────────────┐
  │ Cache Miss                  │
  │ Full search executed        │
  │ Cache repopulated           │
  │ TTL reset to 1800s          │
  └─────────────────────────────┘
```

## Error Handling Flow

```
                    ┌─────────────────┐
                    │ search(request) │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │ cache.get(key)  │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
        ┌─────────┐    ┌─────────┐    ┌─────────┐
        │  Hit    │    │  Miss   │    │  Error  │
        └────┬────┘    └────┬────┘    └────┬────┘
             │              │              │
             │              ▼              │
             │    ┌──────────────────┐    │
             │    │ execute_search() │    │
             │    └────────┬─────────┘    │
             │             │              │
             │    ┌────────▼────────┐    │
             │    │ cache.set(key)  │    │
             │    └────────┬────────┘    │
             │             │              │
             │      ┌──────┴───────┐     │
             │      ▼              ▼     │
             │  Success       Error      │
             │      │              │     │
             │      │   ┌──────────┘     │
             │      │   │                │
             │      │   ▼                │
             │      │ Log Error          │
             │      │ Continue           │
             │      │                    │
             └──────┴────────────────────┘
                    │
             ┌──────▼─────┐
             │   Return   │
             │  Response  │
             └────────────┘

Key Points:
- Cache errors treated as cache misses
- Cache write failures logged, not thrown
- Service remains operational even if Redis fails
```

## Observability & Metrics

```
Tracing Spans:
  search(query="action movies", page=1)
    ├─ generate_cache_key() → cache_key="search:a3f8..."
    ├─ cache.get() → result=Miss
    └─ execute_search(query="action movies")
        ├─ intent_parser.parse() → 50ms
        ├─ vector_search.search() → 100ms
        ├─ keyword_search.search() → 80ms
        ├─ reciprocal_rank_fusion() → 10ms
        └─ Total: 245ms

Logs:
  [DEBUG] Generated cache key cache_key="search:a3f8..."
  [DEBUG] Cache miss - executing full search
  [INFO]  Completed full search execution search_time_ms=245
  [DEBUG] Cached search results ttl=1800

Next Request:
  [DEBUG] Generated cache key cache_key="search:a3f8..."
  [INFO]  Cache hit - returning cached search results cache_time_ms=3
```

## Redis Storage Format

```
Redis Key-Value Pair:

KEY: search:a3f8e9d2c1b4a6f5e8d7c9b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9e8d7c6b5a4f3

VALUE (JSON):
{
  "results": [
    {
      "content": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "title": "The Dark Knight",
        "overview": "Batman faces the Joker...",
        "release_year": 2008,
        "genres": ["action", "crime", "drama"],
        "platforms": ["netflix", "hbo"],
        "popularity_score": 0.95
      },
      "relevance_score": 0.87,
      "match_reasons": ["genre:action", "high_rating"],
      "vector_similarity": 0.89,
      "keyword_score": 0.85
    }
  ],
  "total_count": 150,
  "page": 1,
  "page_size": 20,
  "query_parsed": {
    "mood": ["intense"],
    "themes": ["crime", "heroism"],
    "references": [],
    "filters": {...},
    "fallback_query": "action movies",
    "confidence": 0.92
  },
  "search_time_ms": 245
}

TTL: 1800 seconds (auto-expires after 30 minutes)
```

---

## Summary

This architecture provides:
- ✅ **Sub-10ms cache hits** for repeated queries
- ✅ **30-minute TTL** balancing freshness and performance
- ✅ **Deterministic cache keys** preventing collisions
- ✅ **Graceful degradation** if Redis fails
- ✅ **Full observability** via tracing
- ✅ **Personalized caching** per user
- ✅ **Production-ready** error handling
