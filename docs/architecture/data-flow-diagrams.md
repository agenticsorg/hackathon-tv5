# Data Flow Diagrams - Media Discovery Platform

**Version:** 1.0
**Date:** 2025-12-05

---

## 1. User Query Flow (End-to-End)

### Scenario: User searches "Find me a rom-com streaming tonight"

```
┌────────────────────────────────────────────────────────────────────────┐
│ STEP 1: Query Reception & Authentication                               │
└────────────────────────────────────────────────────────────────────────┘

[User Device]
    │
    │ POST /api/v1/search
    │ Headers: Authorization: Bearer <jwt>
    │ Body: { query: "Find me a rom-com streaming tonight" }
    │
    ▼
[API Gateway - Kong]
    │
    ├─▶ Validate JWT token
    ├─▶ Check rate limit (100 req/min)
    ├─▶ Log request
    │
    ▼
[Query Service]


┌────────────────────────────────────────────────────────────────────────┐
│ STEP 2: Cache Check                                                    │
└────────────────────────────────────────────────────────────────────────┘

[Query Service]
    │
    ├─▶ Generate cache key
    │   Key: query:hash("rom-com+tonight+user_context")
    │
    ▼
[Redis Cache]
    │
    ├─▶ Cache lookup
    │
    ├─▶ IF CACHE HIT:
    │   └─▶ Return cached results (latency: 2ms)
    │       └─▶ END
    │
    └─▶ IF CACHE MISS:
        └─▶ Continue to Step 3


┌────────────────────────────────────────────────────────────────────────┐
│ STEP 3: NLP Processing & Intent Detection                             │
└────────────────────────────────────────────────────────────────────────┘

[Query Service]
    │
    │ Send to AI/NLP Engine
    │
    ▼
[AWS Lambda - NLP Engine]
    │
    ├─▶ GPT-4 API Call
    │   Input: "Find me a rom-com streaming tonight"
    │   Prompt: "Extract intent and entities"
    │
    │   Response:
    │   {
    │     intent: "search",
    │     entities: {
    │       genre: ["romantic comedy"],
    │       availability: "streaming",
    │       timeframe: "today"
    │     },
    │     filters: {
    │       type: "movie",
    │       available_now: true
    │     }
    │   }
    │
    ├─▶ Generate embedding
    │   OpenAI API: text-embedding-3-large
    │   Output: [0.123, -0.456, ..., 0.789] (1536 dimensions)
    │
    ▼
[Return to Query Service]


┌────────────────────────────────────────────────────────────────────────┐
│ STEP 4: User Context Enrichment                                       │
└────────────────────────────────────────────────────────────────────────┘

[Query Service]
    │
    ├─▶ Fetch user context in parallel:
    │
    ├──▶ [User Service]
    │    └─▶ Get user preferences
    │        - Favorite genres: [comedy, drama]
    │        - Preferred platforms: [Netflix, Hulu]
    │        - Content rating: PG-13
    │
    ├──▶ [Redis Cache]
    │    └─▶ Get viewing history (last 30 days)
    │        - Recently watched rom-coms: [Movie A, Movie B]
    │        - Average rating given: 4.2/5
    │
    └──▶ [Geo Service]
         └─▶ Get user location
             - Country: US
             - Timezone: PST
             - Current time: 8:00 PM
             - "Tonight" = content available now
    │
    ▼
[Enriched Query Context]
    {
      intent: "search",
      genre: ["romantic comedy"],
      availability: "streaming",
      timeframe: "now",
      user_preferences: {...},
      user_history: [...],
      location: "US",
      platforms: ["Netflix", "Hulu"]
    }


┌────────────────────────────────────────────────────────────────────────┐
│ STEP 5: Hybrid Search Execution                                       │
└────────────────────────────────────────────────────────────────────────┘

[Query Service]
    │
    ├─────────────────┬────────────────────┐
    │ PARALLEL        │                    │
    ▼                 ▼                    ▼
[Vector Search]  [SQL Search]      [Elasticsearch]
    │                 │                    │
    │                 │                    │
[Pinecone VDB]   [PostgreSQL]        [ES Index]
    │                 │                    │
    │                 │                    │
Query:           Query:               Query:
{                SELECT *             {
  vector: [...], FROM content           "query": {
  filter: {      WHERE                   "bool": {
    genre:         genre @> '{comedy}'     "must": [
      "comedy",    AND type = 'movie'        {"match": {
    available:     AND available = true       "genres": "comedy"
      true         AND region = 'US'         }},
  },             ORDER BY                   {"term": {
  topK: 100        rating DESC,              "available": true
}                  popularity DESC          }}
                 LIMIT 100;              ]}}
                                        }
    │                 │                    │
    ▼                 ▼                    ▼
Results:          Results:             Results:
- 100 semantic    - 100 genre          - 100 text
  matches          matches               matches
- Cosine sim.     - Exact matches      - BM25 score
  scores           - SQL relevance      - TF-IDF


┌────────────────────────────────────────────────────────────────────────┐
│ STEP 6: Result Fusion & Ranking                                       │
└────────────────────────────────────────────────────────────────────────┘

[Query Service - Ranker]
    │
    ├─▶ Combine results from all sources
    │   - Remove duplicates
    │   - Total candidates: ~200 unique items
    │
    ├─▶ Apply ranking algorithm (RRF - Reciprocal Rank Fusion)
    │   score = Σ (1 / (k + rank_i))  where k=60
    │
    │   For each content:
    │   - Vector search rank
    │   - SQL search rank
    │   - ES search rank
    │   - User preference boost
    │   - Recency boost
    │
    ├─▶ Re-rank top 50 with ML model
    │   Features:
    │   - Relevance score
    │   - User viewing history similarity
    │   - Average rating
    │   - Popularity (views last 7 days)
    │   - Platform availability match
    │
    ▼
[Top 50 Candidates]


┌────────────────────────────────────────────────────────────────────────┐
│ STEP 7: Real-Time Availability Verification                           │
└────────────────────────────────────────────────────────────────────────┘

[Query Service]
    │
    ├─▶ For top 50 candidates, check availability:
    │
    │   ┌───────────────────────────────┐
    │   │ PARALLEL (Rate-limited)       │
    │   └───────────────────────────────┘
    │
    ├──▶ [Redis Cache]
    │    └─▶ Check cached availability (< 1 hour old)
    │        Hit Rate: ~85%
    │
    ├──▶ [Availability Verifier Lambda]
    │    │ For cache misses only (~15% = 7-8 items)
    │    │
    │    ├─▶ [Netflix API]
    │    ├─▶ [Hulu API]
    │    └─▶ [Disney+ API]
    │    │
    │    └─▶ Update availability status
    │        Update cache & database
    │
    ▼
[Verified Results]
    - Content with current availability
    - Platform links
    - Pricing info
    - Streaming quality


┌────────────────────────────────────────────────────────────────────────┐
│ STEP 8: Personalization & Final Filtering                             │
└────────────────────────────────────────────────────────────────────────┘

[Query Service]
    │
    ├─▶ Apply user-specific filters:
    │   - Remove already watched content (optional)
    │   - Filter by preferred platforms
    │   - Filter by content rating preference
    │   - Boost similar to liked content
    │
    ├─▶ Diversification:
    │   - Ensure variety in results
    │   - Mix of ratings (8.5, 7.2, 8.9, etc.)
    │   - Mix of release years
    │   - Multiple platforms represented
    │
    ├─▶ Limit to top 20 results
    │
    ▼
[Final Ranked Results]


┌────────────────────────────────────────────────────────────────────────┐
│ STEP 9: Response Enrichment & Formatting                              │
└────────────────────────────────────────────────────────────────────────┘

[Query Service]
    │
    ├─▶ Enrich results with additional data:
    │   - Cast & crew details
    │   - Poster images (CDN URLs)
    │   - Trailer links
    │   - Similar content suggestions
    │   - User-specific metadata (in watchlist?, rated?)
    │
    ├─▶ Format response:
    │   {
    │     query: "Find me a rom-com streaming tonight",
    │     total_results: 20,
    │     results: [
    │       {
    │         id: "uuid",
    │         title: "The Proposal",
    │         type: "movie",
    │         rating: 8.2,
    │         year: 2009,
    │         runtime: 108,
    │         genres: ["Comedy", "Romance"],
    │         cast: [...],
    │         availability: [
    │           {
    │             platform: "Netflix",
    │             available: true,
    │             url: "https://...",
    │             price: { type: "subscription" }
    │           }
    │         ],
    │         poster_url: "https://cdn...",
    │         relevance_score: 0.95
    │       },
    │       ...
    │     ],
    │     metadata: {
    │       search_time_ms: 185,
    │       sources: ["vector", "sql", "elasticsearch"],
    │       cache_hit: false
    │     }
    │   }
    │
    ▼
[Cache Response]
    │
    ├─▶ Store in Redis cache
    │   Key: query:hash
    │   TTL: 1 hour
    │   Size: ~50KB
    │
    ▼


┌────────────────────────────────────────────────────────────────────────┐
│ STEP 10: Response & Analytics                                         │
└────────────────────────────────────────────────────────────────────────┘

[Query Service]
    │
    ├─▶ Return response to client
    │   Status: 200 OK
    │   Latency: 185ms
    │
    ├─▶ Async: Log analytics event
    │   │
    │   └─▶ [Kafka Topic: search-events]
    │       {
    │         user_id: "...",
    │         query: "...",
    │         results_count: 20,
    │         latency_ms: 185,
    │         timestamp: "...",
    │         cache_hit: false
    │       }
    │
    ▼
[User Device - Display Results]
    │
    └─▶ User clicks on a result
        └─▶ Track interaction
            └─▶ [Kafka: interaction-events]
```

---

## 2. Content Ingestion Flow

### Scenario: Sync content from streaming platforms

```
┌────────────────────────────────────────────────────────────────────────┐
│ Content Ingestion Pipeline                                            │
└────────────────────────────────────────────────────────────────────────┘

[Scheduled Job - Cron: 0 2 * * *]  (Daily at 2 AM UTC)
    │
    ▼
[Content Scraper Service]
    │
    ├─▶ Determine platforms to sync
    │   - All active platforms
    │   - Check last sync time
    │   - Incremental vs full sync
    │
    ▼
[Platform API Clients]  (Parallel execution)
    │
    ├──▶ [Netflix API Client]
    │    │
    │    ├─▶ Authenticate (OAuth 2.0)
    │    ├─▶ Fetch catalog delta
    │    │   GET /api/v1/catalog/changes?since=2025-12-04
    │    │
    │    └─▶ Rate limit: 10 req/sec
    │        Response: { added: [...], removed: [...], updated: [...] }
    │
    ├──▶ [Hulu API Client]
    │    └─▶ Similar process
    │
    ├──▶ [Disney+ API Client]
    │    └─▶ Similar process
    │
    └──▶ [Prime Video API Client]
         └─▶ Similar process
    │
    ▼
[Raw Content Data]  (~50,000 items/day)
    │
    │ Store in S3 for backup
    │ s3://content-backups/2025-12-05/netflix-catalog.json
    │
    ▼
[Data Validation & Normalization]
    │
    ├─▶ Schema validation
    │   - Required fields present
    │   - Data type validation
    │   - Format standardization
    │
    ├─▶ Deduplication
    │   - Check by external_id
    │   - Merge duplicates
    │   - Resolve conflicts (newest wins)
    │
    ├─▶ Data enrichment
    │   ├─▶ [TMDB API] - Fetch metadata
    │   │   - Cast & crew
    │   │   - Ratings
    │   │   - Posters
    │   │
    │   └─▶ [IMDB API] - Fetch additional data
    │       - Reviews
    │       - Awards
    │
    ├─▶ Quality checks
    │   - Title not empty
    │   - Valid release year
    │   - At least one genre
    │
    ▼
[Validated Content Data]
    │
    │ Publish to Kafka
    │
    ▼
[Kafka Topic: content-updates]
    │
    │ Partitioned by content_id
    │ Retention: 7 days
    │
    ├────────────────┬────────────────┬────────────────┐
    │                │                │                │
    ▼                ▼                ▼                ▼
[Consumer 1]    [Consumer 2]    [Consumer 3]    [Consumer 4]
PostgreSQL      Vector DB       Elasticsearch   Cache
    │                │                │                │
    │                │                │                │
    ▼                ▼                ▼                ▼

┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ PostgreSQL   │ │  Pinecone    │ │Elasticsearch │ │    Redis     │
│              │ │              │ │              │ │              │
│ INSERT/      │ │ Generate     │ │ Index        │ │ Invalidate   │
│ UPDATE       │ │ embeddings   │ │ documents    │ │ cache keys   │
│ content      │ │ and upsert   │ │              │ │              │
│              │ │              │ │              │ │              │
│ - Metadata   │ │ - Title      │ │ - Full-text  │ │ Keys:        │
│ - Relations  │ │ - Desc       │ │   search     │ │ - content:*  │
│ - History    │ │ - Cast       │ │ - Facets     │ │ - query:*    │
└──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘


[Embedding Generation Detail]
    │
    ▼
[AWS Lambda - Embedding Generator]
    │
    ├─▶ Batch processing (100 items)
    │   To minimize OpenAI API calls
    │
    ├─▶ Create embedding text
    │   Combine: title + description + cast + genres
    │
    ├─▶ OpenAI API call
    │   Model: text-embedding-3-large
    │   Input: [text1, text2, ..., text100]
    │   Output: [[emb1], [emb2], ..., [emb100]]
    │
    ├─▶ Upsert to Pinecone
    │   vectors: [
    │     { id: "uuid1", values: emb1, metadata: {...} },
    │     ...
    │   ]
    │
    └─▶ Cost optimization
        - Cache embeddings for unchanged content
        - Use smaller model for updates detection
        - Batch API calls


[Success Metrics]
    │
    └─▶ [Monitoring Dashboard]
        - Total content synced: 50,000
        - New content: 1,200
        - Updated content: 3,800
        - Removed content: 500
        - Errors: 15 (0.03%)
        - Duration: 45 minutes
        - API calls: 15,000
        - Cost: $12.50
```

---

## 3. Recommendation Generation Flow

### Scenario: Generate personalized recommendations for a user

```
┌────────────────────────────────────────────────────────────────────────┐
│ Recommendation Pipeline                                               │
└────────────────────────────────────────────────────────────────────────┘

[Trigger: User opens app / Periodic refresh]
    │
    ▼
[API Request]
    GET /api/v1/recommendations?user_id=xyz&limit=20
    │
    ▼
[Recommendation Service]
    │
    ├─▶ Check cache
    │   Key: recommendations:user:xyz
    │   TTL: 1 hour
    │
    │   IF CACHE HIT && not_stale:
    │   └─▶ Return cached recommendations
    │
    │   IF CACHE MISS || stale:
    │   └─▶ Continue to generation
    │
    ▼
[User Profile Aggregation]  (Parallel)
    │
    ├──▶ [User Service]
    │    └─▶ Get user preferences
    │        - Favorite genres
    │        - Preferred platforms
    │        - Watch history (last 3 months)
    │        - Ratings given
    │
    ├──▶ [PostgreSQL]
    │    └─▶ Query viewing history
    │        SELECT content_id, watched_at, completed
    │        FROM user_history
    │        WHERE user_id = 'xyz'
    │          AND watched_at > NOW() - INTERVAL '90 days'
    │        ORDER BY watched_at DESC
    │        LIMIT 100
    │
    └──▶ [Redis]
         └─▶ Get real-time user state
             - Current session activity
             - Recent searches
             - Recently clicked items
    │
    ▼
[User Feature Vector]
    {
      user_id: "xyz",
      favorite_genres: ["thriller", "sci-fi"],
      watch_history: [
        { content_id: "abc", rating: 4.5, completed: true },
        ...
      ],
      genre_distribution: { thriller: 0.4, scifi: 0.3, drama: 0.3 },
      avg_rating: 4.2,
      watch_frequency: "high",
      preferred_platforms: ["Netflix", "HBO"],
      viewing_time: "evening"
    }


┌────────────────────────────────────────────────────────────────────────┐
│ Multi-Strategy Recommendation Generation                              │
└────────────────────────────────────────────────────────────────────────┘

[Recommendation Engine]
    │
    ├─────────────────┬─────────────────┬─────────────────┐
    │ PARALLEL        │                 │                 │
    ▼                 ▼                 ▼                 ▼
[Collaborative]  [Content-Based]  [Trending]      [Serendipity]
  Filtering         Filtering       Boost           Discovery
    │                 │                 │                 │


[1. Collaborative Filtering]
    │
    ├─▶ Find similar users (user-user CF)
    │   │
    │   ├─▶ Query user similarity matrix
    │   │   (Pre-computed daily via Spark job)
    │   │
    │   │   SELECT similar_user_id, similarity_score
    │   │   FROM user_similarity
    │   │   WHERE user_id = 'xyz'
    │   │   ORDER BY similarity_score DESC
    │   │   LIMIT 100
    │   │
    │   └─▶ Get content watched by similar users
    │       Content not yet watched by current user
    │
    ├─▶ Item-item CF
    │   │
    │   ├─▶ For each watched item, find similar items
    │   │   (Pre-computed similarity matrix)
    │   │
    │   └─▶ Score based on:
    │       - Item similarity
    │       - User's rating of source item
    │       - Recency of source item view
    │
    └─▶ Generate scores for candidate items
        score_cf = Σ (similarity * rating)
    │
    ▼
[CF Candidates: 200 items with scores]


[2. Content-Based Filtering]
    │
    ├─▶ Create user content profile
    │   Average embeddings of liked content (rating > 4)
    │   user_embedding = avg([emb1, emb2, ..., embN])
    │
    ├─▶ Vector similarity search
    │   │
    │   └─▶ [Pinecone]
    │       query(
    │         vector: user_embedding,
    │         topK: 200,
    │         filter: {
    │           not_in: [already_watched],
    │           genres: [preferred_genres]
    │         }
    │       )
    │
    └─▶ Score by cosine similarity
        score_cb = cosine_similarity(user_emb, content_emb)
    │
    ▼
[CB Candidates: 200 items with scores]


[3. Trending Boost]
    │
    ├─▶ [Redis Leaderboard]
    │   ZREVRANGE trending:content:week 0 100
    │
    │   Returns:
    │   - Content IDs ranked by popularity
    │   - Scores based on views last 7 days
    │
    └─▶ Apply trending boost
        score_trend = log(1 + view_count_7d)
    │
    ▼
[Trending: 100 items with boost scores]


[4. Serendipity / Discovery]
    │
    ├─▶ Explore new genres/types
    │   - Genres user hasn't tried much
    │   - Highly rated content outside comfort zone
    │   - "Hidden gems" with high rating but lower popularity
    │
    ├─▶ Diversity promotion
    │   - Different release years
    │   - International content
    │   - Various runtimes
    │
    └─▶ Score by novelty * quality
        score_serendipity = novelty_score * avg_rating
    │
    ▼
[Discovery: 50 items with scores]


┌────────────────────────────────────────────────────────────────────────┐
│ Fusion & Re-Ranking                                                   │
└────────────────────────────────────────────────────────────────────────┘

[Hybrid Ensemble]
    │
    ├─▶ Combine all candidate pools
    │   Total candidates: ~400-500 unique items
    │
    ├─▶ Weighted ensemble scoring
    │   final_score =
    │     0.40 * score_cf +
    │     0.30 * score_cb +
    │     0.20 * score_trend +
    │     0.10 * score_serendipity
    │
    ├─▶ Apply business rules
    │   - Boost content available on user's platforms
    │   - Boost recently released content
    │   - Penalize very long content (if user prefers short)
    │   - Boost content in user's preferred languages
    │
    ├─▶ Remove already watched/rated content
    │
    └─▶ Sort by final_score DESC
    │
    ▼
[Top 100 Candidates]
    │
    │
[ML Re-Ranker]  (Optional, for premium users)
    │
    ├─▶ Load trained model (XGBoost)
    │   Model predicts: P(user will watch and like)
    │
    ├─▶ Generate features for each candidate
    │   - All previous scores
    │   - User-item interactions
    │   - Temporal features (time of day, day of week)
    │   - Context (device type, location)
    │   - Social signals (friends watching)
    │
    ├─▶ Predict scores
    │   ml_score = model.predict(features)
    │
    └─▶ Final ranking by ml_score
    │
    ▼
[Top 50 Re-ranked]


┌────────────────────────────────────────────────────────────────────────┐
│ Diversification & Presentation                                        │
└────────────────────────────────────────────────────────────────────────┘

[Diversification]
    │
    ├─▶ Ensure variety in top 20
    │   - Max 3 items from same genre
    │   - Mix of ratings (don't show all 9.0+)
    │   - Mix of release years
    │   - Multiple platforms represented
    │   - Mix of runtimes
    │
    ├─▶ Interleave strategies
    │   Position 1: Collaborative (safe bet)
    │   Position 2: Content-based (relevant)
    │   Position 3: Trending (social proof)
    │   Position 4: Discovery (novelty)
    │   ...repeat pattern
    │
    └─▶ Select top 20
    │
    ▼
[Final 20 Recommendations]
    │
    │
[Enrich & Format]
    │
    ├─▶ Fetch full content details
    │   - Metadata
    │   - Cast & crew
    │   - Images
    │   - Availability
    │   - Trailers
    │
    ├─▶ Add explanation (optional)
    │   "Because you watched The Matrix"
    │   "Trending in your area"
    │   "Hidden gem in Sci-Fi"
    │
    └─▶ Format response
    │
    ▼
[Response]
    {
      user_id: "xyz",
      recommendations: [
        {
          content: {...},
          score: 0.92,
          reason: "Because you liked The Matrix",
          strategy: "collaborative"
        },
        ...
      ],
      generated_at: "2025-12-05T12:00:00Z",
      expires_at: "2025-12-05T13:00:00Z"
    }
    │
    │
[Cache & Return]
    │
    ├─▶ Store in Redis
    │   Key: recommendations:user:xyz
    │   TTL: 1 hour
    │   Size: ~80KB
    │
    └─▶ Return to client
        Latency: 320ms
```

---

## 4. Real-Time Availability Update Flow

### Scenario: Netflix adds a new movie, updates propagate

```
┌────────────────────────────────────────────────────────────────────────┐
│ Real-Time Availability Update                                         │
└────────────────────────────────────────────────────────────────────────┘

[Netflix API]
    │
    │ Webhook POST /api/v1/webhooks/netflix
    │ Body: {
    │   event: "content.added",
    │   content_id: "netflix:12345",
    │   title: "New Action Movie",
    │   available: true,
    │   timestamp: "2025-12-05T10:00:00Z"
    │ }
    │ Headers: X-Netflix-Signature: "sha256=..."
    │
    ▼
[Webhook Handler Service]
    │
    ├─▶ Verify webhook signature
    │   HMAC-SHA256 validation
    │
    ├─▶ Validate payload schema
    │
    ▼
[Process Availability Change]
    │
    ├─▶ Lookup internal content ID
    │   Netflix external_id → internal UUID
    │
    │   IF NOT FOUND:
    │   └─▶ Trigger content fetch job
    │       └─▶ Fetch full metadata from Netflix API
    │           └─▶ Create new content record
    │
    │   IF FOUND:
    │   └─▶ Continue with update
    │
    ▼
[Update Database]
    │
    ├─▶ [PostgreSQL]
    │   UPDATE platform_availability
    │   SET
    │     available = true,
    │     last_verified_at = NOW()
    │   WHERE content_id = 'uuid'
    │     AND platform_id = 'netflix-id'
    │   RETURNING *
    │
    ▼
[Invalidate Caches]  (Parallel)
    │
    ├──▶ [Redis]
    │    │
    │    ├─▶ Delete availability cache
    │    │   DEL availability:uuid
    │    │
    │    ├─▶ Delete related query caches
    │    │   SCAN query:*:uuid:*
    │    │   DEL [matched keys]
    │    │
    │    └─▶ Delete recommendation caches
    │        For users with content in watchlist
    │        DEL recommendations:user:*
    │
    └──▶ [CDN]
         └─▶ Invalidate cached API responses
             POST /api/invalidate
             { paths: ["/api/v1/content/uuid"] }
    │
    ▼
[Publish Events]
    │
    ├─▶ [Kafka Topic: availability-changed]
    │   {
    │     content_id: "uuid",
    │     platform: "netflix",
    │     available: true,
    │     timestamp: "2025-12-05T10:00:05Z",
    │     change_type: "added"
    │   }
    │
    ▼
[Event Consumers]  (Multiple)
    │
    ├──▶ [Notification Service]
    │    │
    │    ├─▶ Query watchlist users
    │    │   SELECT user_id
    │    │   FROM watchlist
    │    │   WHERE content_id = 'uuid'
    │    │
    │    └─▶ Send notifications
    │        - Push notifications
    │        - Email (if enabled)
    │        - In-app notifications
    │
    │        Message: "New Action Movie is now available on Netflix!"
    │
    ├──▶ [Analytics Service]
    │    └─▶ Track availability change
    │        - Log to data warehouse
    │        - Update availability metrics
    │        - Track platform catalog growth
    │
    ├──▶ [Search Index Updater]
    │    └─▶ [Elasticsearch]
    │        Update document with new availability
    │
    └──▶ [ML Pipeline]
         └─▶ Update content freshness score
             New content gets temporary boost in rankings
    │
    ▼
[Real-Time WebSocket Updates]  (Optional)
    │
    └─▶ For active users viewing this content:
        │
        └─▶ [WebSocket Server]
            Push update to connected clients
            {
              type: "availability_update",
              content_id: "uuid",
              platform: "netflix",
              available: true
            }

            Client automatically updates UI
```

---

## 5. Performance Metrics by Stage

```
┌────────────────────────────────────────────────────────────────────────┐
│ Query Processing Latency Breakdown (Target: 200ms p95)                │
└────────────────────────────────────────────────────────────────────────┘

Stage                          | Target  | p50   | p95   | p99   |
-------------------------------|---------|-------|-------|-------|
API Gateway                    |  5ms    | 2ms   | 5ms   | 8ms   |
Auth & Rate Limit              |  3ms    | 1ms   | 3ms   | 5ms   |
Cache Lookup (Redis)           |  2ms    | 1ms   | 2ms   | 4ms   |
NLP Processing (Lambda)        | 50ms    | 35ms  | 50ms  | 75ms  |
User Context Fetch             | 10ms    | 5ms   | 10ms  | 15ms  |
Vector Search (Pinecone)       | 40ms    | 25ms  | 40ms  | 60ms  |
SQL Search (PostgreSQL)        | 30ms    | 18ms  | 30ms  | 50ms  |
Elasticsearch Query            | 25ms    | 15ms  | 25ms  | 40ms  |
Result Ranking                 | 15ms    | 10ms  | 15ms  | 25ms  |
Availability Check             | 40ms    | 28ms  | 40ms  | 65ms  |
Response Formatting            |  8ms    | 5ms   | 8ms   | 12ms  |
Cache Write (Redis)            |  2ms    | 1ms   | 2ms   | 4ms   |
-------------------------------|---------|-------|-------|-------|
TOTAL (Cache Miss)             | 200ms   | 145ms | 200ms | 330ms |
TOTAL (Cache Hit)              |  15ms   | 8ms   | 15ms  | 22ms  |

Cache Hit Rate Target: 85%
Effective Average Latency: (0.85 * 15ms) + (0.15 * 200ms) = 42.75ms
```

This comprehensive data flow documentation shows exactly how data moves through the system for all major operations.
