# RuVector Architecture: System Flow Diagrams

## 1. Complete Learning Cycle

```
┌─────────────────────────────────────────────────────────────┐
│ USER INTERACTION → FEEDBACK → LEARNING → OPTIMIZATION      │
└─────────────────────────────────────────────────────────────┘

1. USER ACTION (watch, skip, rate)
   └─→ Record in watch_history / ratings

2. FEEDBACK COLLECTION
   └─→ Calculate reward (-1.0 to +1.0)
   └─→ Link to pattern_id used
   └─→ INSERT learning_feedback

3. PATTERN EVALUATION
   └─→ Update recommendation_patterns.success_rate
   └─→ Update avg_reward
   └─→ Increment total_uses

4. REFLEXION STORAGE
   └─→ INSERT reflexion_episodes
   └─→ Generate embedding from context+action+outcome
   └─→ Store self-critique evaluation

5. DAILY OPTIMIZATION (3 AM)
   ├─ CLUSTER DETECTION
   │  └─ Find high-similarity content pairs (>0.75)
   │  └─ Group into semantic clusters
   │  └─ Extract primary genres
   │
   ├─ PATTERN CREATION
   │  └─ Generate new patterns from clusters
   │  └─ Estimate success_rate = 0.7 + (similarity × 0.2)
   │
   ├─ REWARD AGGREGATION
   │  └─ Query learning_feedback (last 7 days)
   │  └─ Group rewards by genre
   │  └─ Identify best-performing genre
   │
   ├─ EMBEDDING OPTIMIZATION
   │  └─ Call ruvector_learn_from_feedback()
   │  └─ SIMD updates content embeddings
   │  └─ Vectors shift toward high-reward clusters
   │
   ├─ QUALITY MEASUREMENT
   │  └─ Measure intra-cluster similarity per genre
   │  └─ Track overall quality improvement
   │
   └─ STATE PERSISTENCE
      └─ Save learning state to sync_status

6. PREFERENCE UPDATE
   └─→ Regenerate user preference vectors
   └─→ Weighted average of watched content
   └─→ L2 normalize to unit vector
```

## 2. PostgreSQL Schema (RuVector Tables)

```
CONTENT TABLE
├─ id (VARCHAR, PK)
├─ content_type (series|movie)
├─ title, year, overview
├─ genres (TEXT[])
├─ rating, status, network_info
├─ embedding [384] ← RUVECTOR
└─ search_vector (tsvector)

USER_PREFERENCES TABLE
├─ user_id (UUID, PK/FK)
├─ preference_vector [384] ← RUVECTOR
├─ genre_weights ({genre: weight})
├─ network_weights ({network: weight})
└─ timestamps

WATCH_HISTORY TABLE
├─ id (UUID, PK)
├─ user_id (UUID, FK)
├─ content_id (VARCHAR, FK)
├─ watched_at, duration_seconds
├─ completion_percentage
└─ episode metadata (for series)

RATINGS TABLE
├─ id (UUID, PK)
├─ user_id (UUID, FK)
├─ content_id (VARCHAR, FK)
├─ rating (0.0-10.0)
└─ rated_at

RECOMMENDATION_PATTERNS TABLE
├─ id (SERIAL, PK)
├─ pattern_type (cold_start|genre_match|etc)
├─ approach (TEXT description)
├─ Context fields (user_segment, time_of_day, platform)
├─ success_rate [0-1] ← TRAINED
├─ total_uses (usage count)
├─ avg_reward [-1,1] ← FROM FEEDBACK
├─ embedding [384] ← RUVECTOR (pattern context)
└─ timestamps

LEARNING_FEEDBACK TABLE
├─ id (UUID, PK)
├─ user_id (UUID, FK)
├─ content_id (VARCHAR, FK)
├─ pattern_id (INT, FK) ← Links to patterns
├─ was_successful (BOOLEAN)
├─ reward [-1,1] ← FROM USER ACTION
├─ user_action (watched|skipped|rated|etc)
├─ recommendation_position (1-N)
└─ created_at

REFLEXION_EPISODES TABLE
├─ id (SERIAL, PK)
├─ context (pattern used, user segment)
├─ action (user action taken)
├─ outcome (result of action)
├─ reward [-1,1]
├─ self_critique (evaluation)
├─ embedding [384] ← RUVECTOR (episode context)
└─ created_at
```

## 3. Recommendation Generation Flow

```
REQUEST: { userId, contentType, limit }
        │
        ├─ Load user preference vector
        ├─ Load user segment (from watch_history count)
        ├─ Determine context (time, platform)
        │
        ├─ PATTERN SELECTION
        │  ├─ Convert context to text
        │  ├─ Generate context embedding
        │  ├─ Search top-5 similar patterns via RuVector
        │  ├─ Score = similarity × 0.4 + success_rate × 0.6
        │  └─ Select highest-scored pattern
        │
        ├─ STRATEGY DISPATCH (based on pattern type)
        │  ├─ cold_start → Return popular content (by rating)
        │  ├─ genre_match → Filter by top user genres
        │  ├─ similar_content → Vector similarity search
        │  ├─ time_based → Adjust for time of day
        │  ├─ network_based → Filter by preferred networks
        │  └─ personalized → General similarity ranking
        │
        ├─ DIVERSITY BOOSTING
        │  ├─ Greedy selection to maximize genre diversity
        │  ├─ Minimize genre overlap between recommendations
        │  └─ Avoid 10 similar items
        │
        ├─ WATCHED FILTERING
        │  └─ Remove items in user's watch_history
        │
        ├─ POSITION ASSIGNMENT
        │  └─ Rank 1-20 based on relevance
        │
        └─ RETURN: RecommendationResponse {
             recommendations: [...],
             patternUsed: "...",
             learningFeedback: { patternId, reward, action }
           }
```

## 4. RuVector Operations

```
ruvector_cosine_distance(embedding1, embedding2)
├─ Returns: 0 (identical) to 1 (opposite)
├─ Used in: Similarity search, cluster detection, pattern matching
└─ Hardware: SIMD-optimized (5-8x faster)

ruvector_enable_learning(table, config)
├─ Called on: content table during initialization
├─ Config: Q-Learning algorithm, reward_decay=0.95
└─ Effect: Enables embedding optimization via feedback

ruvector_learn_from_feedback(table, feedback_json)
├─ Input: { rewards: {genre: value}, strategy, episode }
├─ Called: During daily optimization cycle
├─ Effect: Updates embeddings toward high-reward clusters
└─ Result: Improved semantic grouping
```

## 5. Pattern Evolution Over Time

```
DAY 0: SEEDED PATTERNS (5 initial)
├─ cold_start (success: 60%)
├─ genre_match (success: 75%)
├─ similar_content (success: 80%)
├─ time_based (success: 65%)
└─ network_based (success: 70%)

DAYS 1-7: FEEDBACK ACCUMULATION
├─ 100+ user interactions
├─ Patterns scored based on outcomes
├─ best_strategy = highest avg_reward genre

DAYS 8-30: CLUSTER-BASED PATTERNS
├─ Identify high-similarity content groups
├─ Generate genre-combination patterns
│  ├─ drama_thriller (similarity: 0.82)
│  ├─ drama_scifi (similarity: 0.75)
│  └─ comedy_romance (similarity: 0.78)
├─ Initial success rates (0.75-0.85)

DAYS 31+: CONVERGENCE
├─ Successful patterns refined
├─ Low-performing patterns deprecated
├─ System reaches 80-85%+ success rate
├─ Exploration rate drops to 5-10%
└─ Patterns highly specialized to user base
```

## 6. Learning State Persistence

```
Learning State Object:
├─ episode: Current learning iteration (0-1000+)
├─ totalReward: Cumulative reward (-∞ to ∞)
├─ explorationRate: Current epsilon (0.05-0.30)
└─ bestStrategy: Top-performing strategy

Persistence:
├─ Saved after each optimization cycle
├─ Stored in sync_status table
├─ Enables recovery on system restart
├─ Tracks learning progress over weeks/months
```

## 7. Optimization Cycle Metrics

```
OptimizationMetrics {
  totalOptimized: 3456,           // Items processed
  clustersIdentified: 23,         // Semantic groups found
  patternsUpdated: 8,             // New patterns created
  qualityImprovement: 0.027,      // Similarity improvement
  searchSpeedImprovement: 4.2%    // Efficiency gain
}
```

## 8. User Segmentation Strategy

```
new (0 items)
├─ Pattern: cold_start
├─ Strategy: Popular content
└─ Success: 60%

casual (1-4 items)
├─ Pattern: genre_match + trending
├─ Strategy: Genre filtering
└─ Success: 65-70%

regular (5-19 items)
├─ Pattern: genre_match + similarity
├─ Strategy: Learned preferences
└─ Success: 75-80%

power (20+ items)
├─ Pattern: cluster-based + personalized
├─ Strategy: Fine-tuned preferences
└─ Success: 80-85%+
```

## 9. Time-of-Day Patterns

```
Morning (5-12)    → Short content, movies
Afternoon (12-17) → Varied content
Evening (17-21)   → Long series, dramas
Night (21-5)      → Dark/intense content, rewatches
```

## 10. Performance Characteristics

```
Single Recommendation: 50-100ms
  └─ Pattern selection: <5ms
  └─ Similarity search: <10ms
  └─ Ranking: <85ms

Batch (50 users): 2-4 seconds
  └─ Parallel processing

Daily Optimization: ~20 seconds
  └─ Cluster detection: 5s
  └─ Pattern creation: 2s
  └─ Reward aggregation: 3s
  └─ Embedding update: 5s
  └─ Quality measurement: 4s
  └─ State persistence: 1s

Without RuVector: ~160-240ms per operation
With RuVector SIMD: ~30-50ms per operation
Speedup: 5-8x
```

## Key Files

| File | Purpose |
|------|---------|
| `/src/db/schema-ruvector.sql` | PostgreSQL schema |
| `/src/services/recommendation-engine.ts` | Pattern selection & generation |
| `/src/services/ruvector-optimizer.ts` | Daily optimization |
| `/src/services/embedding-service.ts` | Vector generation |
| `/src/db/repository.ts` | Database operations |
| `/src/types/index.ts` | Type definitions |
