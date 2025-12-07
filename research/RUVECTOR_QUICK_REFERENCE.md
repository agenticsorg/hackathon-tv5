# RuVector TVDB System - Quick Reference Guide

## Core Tables (RuVector-Enabled)

| Table | Ruvector Field | Purpose | Dimension |
|-------|---|---|---|
| `content` | `embedding` | Semantic representation of show/movie | 384 |
| `user_preferences` | `preference_vector` | Aggregated user taste profile | 384 |
| `recommendation_patterns` | `embedding` | Pattern context for similarity search | 384 |
| `reflexion_episodes` | `embedding` | Learned experience for self-critique | 384 |

---

## Learning Feedback Records

```
user watches content → reward calculated
↓
INSERT learning_feedback {
  user_id, content_id, pattern_id,
  was_successful, reward (-1 to 1), user_action,
  recommendation_position, created_at
}
↓
recommendation_patterns.success_rate updated
recommendation_patterns.avg_reward updated
↓
reflexion_episodes entry created
↓
(Daily) optimization cycle runs
```

---

## Reward Values

| Action | Reward | Meaning |
|--------|--------|---------|
| Watched 90%+ | +0.9 to +1.0 | Strong engagement |
| Watched 50-80% | +0.4 to +0.7 | Moderate engagement |
| Watched <50% | -0.1 to +0.3 | Low engagement |
| Skipped | -0.2 to -0.3 | Not interested |
| Rated high (8+) | +0.8 to +1.0 | Quality confirmed |
| Rated low (<5) | -0.8 to -0.5 | Did not like |
| Added watchlist | +0.5 | Expressed intent |
| Dismissed | -0.2 | Explicitly rejected |

---

## RuVector Functions

### Enable Learning
```sql
SELECT ruvector_enable_learning('content', '{
  "algorithm": "q_learning",
  "reward_decay": 0.95,
  "learning_rate": 0.1,
  "exploration_rate": 0.3
}'::jsonb);
```

### Compute Distance
```sql
SELECT 1 - ruvector_cosine_distance(embedding1, embedding2) as similarity
-- Returns 0-1 (1 = identical, 0 = opposite)
```

### Learn from Feedback
```sql
SELECT ruvector_learn_from_feedback('content', '{
  "rewards": {
    "Drama": 0.85,
    "SciFi": 0.72
  },
  "strategy": "genre_Drama",
  "episode": 245
}'::jsonb);
```

---

## Pattern Types & Contexts

```
Pattern Type         User Segment    Time          Platform   Success Rate
─────────────────────────────────────────────────────────────────────────
cold_start          new             any           any        60%
genre_match         regular         any           any        75%
similar_content     power           any           any        80%
time_based          casual          evening       web        65%
network_based       regular         any           any        70%
drama_thriller      regular/power   evening       mobile     85%+
drama_scifi         power           night         tv         82%+
```

---

## Preference Vector Generation

```typescript
watchHistory = [
  { contentId: "123", vector: [...384 values...] },
  { contentId: "456", vector: [...384 values...] },
  // ... up to 100 items
]

ratings = [
  { contentId: "123", rating: 9 },  // High rating = weight 0.9
  { contentId: "456", rating: 7 },  // Lower rating = weight 0.7
  // ...
]

preferenceVector = weightedAverage(watchHistory, ratings)
                 = normalize(preferenceVector)  // L2 norm
```

---

## Key Metrics Tracked

### Per Pattern
- `success_rate` (0.0-1.0) - Percentage of successful recommendations
- `total_uses` (0-∞) - Times this pattern was applied
- `avg_reward` (-1.0 to 1.0) - Average reward value
- `last_used_at` (TIMESTAMP) - Recency indicator

### Per Feedback Record
- `reward` (-1.0 to 1.0) - Outcome signal
- `was_successful` (BOOLEAN) - Binary success
- `recommendation_position` (1-N) - List placement
- `user_action` (TEXT) - Type of engagement

### System Level
- `quality_score` (0.0-1.0) - Intra-cluster similarity
- `exploration_rate` (0.05-0.30) - Random exploration %
- `episode_count` (0-∞) - Learning iterations
- `total_reward` (-∞ to ∞) - Cumulative reward

---

## Optimization Cycle (Runs Daily @ 3 AM)

1. **Cluster Detection** (5 sec)
   - Find content pairs with similarity > 0.75
   - Group into semantic clusters
   - Extract primary genres

2. **Pattern Creation** (2 sec)
   - Generate new patterns from clusters
   - Estimate success rate from similarity
   - Insert into `recommendation_patterns`

3. **Reward Aggregation** (3 sec)
   - Query `learning_feedback` (last 7 days)
   - Group rewards by genre
   - Identify best-performing genre

4. **Embedding Update** (5 sec)
   - Call `ruvector_learn_from_feedback()`
   - SIMD-optimized vector updates
   - Adjust embeddings toward high-reward clusters

5. **Quality Measurement** (4 sec)
   - Sample 100 random items
   - Measure intra-cluster similarity per genre
   - Calculate overall quality score

6. **State Persistence** (1 sec)
   - Save learning state to `sync_status`
   - Enable recovery on restart

**Total:** ~20 seconds, 5-8x faster with RuVector SIMD

---

## Pattern Selection Algorithm

```
Given: User context (segment, time, platform, genres)
       ↓
       Convert to embedding text
       ↓
       Search recommendation_patterns for top-5 similar
       ↓
       Score = similarity × 0.4 + success_rate × 0.6
       ↓
       Return highest-scored pattern
       ↓
       Apply pattern strategy (genre_match, similar_content, etc.)
       ↓
       Generate recommendations
       ↓
       Diversity boosting (minimize genre overlap)
       ↓
       Filter watched content
       ↓
       Assign positions (1-20)
```

---

## Cold Start Handling

For new users without preference vectors:
1. Return top-rated content (global popularity)
2. Filter by requested genres if available
3. Sort by rating (proxy for quality)
4. After 3-5 items watched → switch to learned patterns

---

## User Segmentation

| Segment | Watch Count | Characteristics | Strategy |
|---------|------------|-----------------|----------|
| **new** | 0 | No history | Popular trending |
| **casual** | 1-4 | Light usage | Genre + trending |
| **regular** | 5-19 | Active | Genre matching + similarity |
| **power** | 20+ | Heavy usage | Cluster-based + personalized |

---

## Time-Based Patterns

| Time | Content Preference | Rationale |
|------|-------------------|-----------|
| Morning (5-12) | Movies, short shows | Limited time |
| Afternoon (12-17) | Varied | Work/school breaks |
| Evening (17-21) | Long series | Wind down |
| Night (21-5) | Dark/intense content | Late-night viewing |

---

## Embedding Model Details

**Model:** Xenova/all-MiniLM-L6-v2
- **Dimension:** 384
- **Type:** Sentence-level embeddings
- **Training:** MSMarco dataset
- **Quantized:** Yes (faster inference)
- **Cache TTL:** 24 hours

**Text Built From:**
- Title (highest weight)
- Overview (medium weight)
- Genres (medium weight)
- Year (light weight)
- Network (if available)

---

## Database Schema Indexes

```sql
-- Content table
idx_content_embedding      -- For vector similarity search
idx_content_search         -- Full-text search (GIN)
idx_content_type           -- Content type filtering
idx_content_rating         -- Popular content ranking

-- User preferences
idx_user_prefs_vector      -- For personalization search

-- Patterns
idx_patterns_embedding     -- For pattern matching
idx_patterns_success       -- Sort by success rate
idx_patterns_type          -- Filter by pattern type

-- Feedback & episodes
idx_feedback_pattern       -- Link to patterns
idx_feedback_success       -- Filter successful feedback
idx_reflexion_reward       -- High-reward episodes
```

---

## Learning Loop Example

### Day 1-2: New User (Cold Start)
```
Pattern Used: cold_start
Recommendations: [Top 20 by rating]
User Action: Watches 3, skips 2
Rewards: +0.9, +0.8, +0.7, -0.1, -0.2
Pattern success_rate: (3 successes / 5) = 60%
```

### Day 3-7: Building Profile
```
Pattern Used: genre_match
User watched: Drama, Drama, Drama, SciFi, Comedy
Genre weights learned: Drama: 0.75, SciFi: 0.5, Comedy: 0.3
Preference vector: Heavy drama, some scifi
Pattern success_rate: 73%
```

### Day 8-30: Personalization
```
Pattern Used: similar_content + cluster patterns
Watched 15 items, 12 drama, 3 scifi thrillers
Preference vector refined
New pattern discovered: drama_thriller (cluster similarity 0.82)
Pattern success_rate: 82%+
Exploration rate decays: 0.30 → 0.10
```

### Day 31+: Optimized
```
Pattern Used: custom_drama_thriller (high success)
System learned:
  - User prefers evening recommendations
  - Mobile platform most used
  - Series > Movies
  - Drama + Thriller best combination
Success rate: 85%+
Exploration rate: ~5% (minimal random)
```

---

## Performance Characteristics

| Operation | Time | Complexity |
|-----------|------|-----------|
| Single recommendation | 50-100ms | O(n log n) |
| Batch (50 users) | 2-4 seconds | O(n log n) |
| Daily optimization | 20 seconds | O(n²) sampled |
| Pattern search (top-5) | <5ms | O(log m) |
| Similarity calculation | <1ms | O(1) SIMD |
| Preference update | 10ms | O(h) - h = watch history |

---

## Testing Pattern Success

```
SELECT pattern_type, approach, success_rate, total_uses, avg_reward
FROM recommendation_patterns
ORDER BY success_rate DESC;

-- Find best pattern for specific context
SELECT p.*,
       1 - ruvector_cosine_distance(p.embedding, $1::ruvector) as similarity
FROM recommendation_patterns p
WHERE user_segment = 'regular'
ORDER BY similarity DESC
LIMIT 5;

-- Analyze feedback quality
SELECT pattern_id, COUNT(*) as uses,
       SUM(CASE WHEN was_successful THEN 1 ELSE 0 END)::float / COUNT(*) as success,
       AVG(reward) as avg_reward
FROM learning_feedback
WHERE created_at > NOW() - INTERVAL '7 days'
GROUP BY pattern_id
ORDER BY success DESC;
```

---

## Common Issues & Solutions

| Issue | Cause | Solution |
|-------|-------|----------|
| Cold recommendations | No preference vector | Use popular content, build history |
| Low success rate | Stale patterns | Run optimization cycle |
| Slow searches | Unindexed vectors | Check ruvector indexes |
| Memory spike | Large batch processing | Process in smaller batches |
| Inconsistent results | Non-deterministic clustering | Sort results by ID for tie-breaking |

---

## Files & Locations

| File | Purpose |
|------|---------|
| `/src/db/schema-ruvector.sql` | PostgreSQL schema (4 ruvector tables) |
| `/src/services/recommendation-engine.ts` | Pattern selection & application |
| `/src/services/ruvector-optimizer.ts` | Daily optimization cycle |
| `/src/services/embedding-service.ts` | Embedding generation |
| `/src/db/repository.ts` | Database operations |
| `/src/types/index.ts` | Type definitions |

---

## Quick Start Code

```typescript
// Initialize embedding service
const embedder = new EmbeddingService({
  model: 'Xenova/all-MiniLM-L6-v2',
  dimension: 384
});
await embedder.initialize();

// Initialize recommendation engine
const engine = new RecommendationEngine(embedder, {
  defaultLimit: 20,
  similarityThreshold: 0.3,
  diversityFactor: 0.2
});
await engine.initialize('./data/recommendations.db');

// Get recommendations
const recommendations = await engine.getRecommendations({
  userId: 'user123',
  limit: 10,
  contentType: 'series'
}, userPreference, contentEmbeddings);

// Record feedback
await engine.recordFeedback({
  patternId: 5,
  wasSuccessful: true,
  reward: 0.85,
  userAction: 'watched',
  timestamp: new Date()
});

// Run optimization (daily)
const optimizer = new RuVectorOptimizer(dbUrl);
const metrics = await optimizer.runOptimizationCycle();
```

---

## Key Takeaways

1. **4 RuVector Tables:** All use 384-dimensional embeddings for semantic search
2. **Closed-Loop Learning:** User actions → Feedback → Pattern optimization → Better recommendations
3. **Self-Learning Functions:** `ruvector_enable_learning()` and `ruvector_learn_from_feedback()`
4. **Pattern Evolution:** Seeded patterns improve via Q-learning, new patterns emerge from clusters
5. **SIMD Acceleration:** RuVector provides 5-8x speedup via hardware-optimized vector ops
6. **Metrics:** Track success_rate, avg_reward, total_uses per pattern
7. **Exploration:** Epsilon-greedy strategy balances exploitation (80%) with exploration (20%)
8. **Temporal Awareness:** Patterns include time-of-day, day-of-week, platform context
9. **User Segmentation:** Different strategies for new/casual/regular/power users
10. **Persistence:** Learning state saved to enable recovery and analysis

