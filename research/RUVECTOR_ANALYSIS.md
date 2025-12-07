# RuVector Schema Analysis: TVDB Self-Learning Recommendation System

## Executive Summary

The TVDB recommender system implements a sophisticated self-learning recommendation engine using PostgreSQL with RuVector (SIMD-optimized vector operations) extension. The system maintains a closed-loop learning mechanism that continuously improves recommendation quality through pattern learning, feedback collection, and adaptive optimization.

## 1. RuVector-Enabled Tables

### Tables Using ruvector Type (384 dimensions)

1. **content** - Semantic embeddings for TV shows and movies
2. **user_preferences** - Aggregated user preference vectors
3. **recommendation_patterns** - Pattern context embeddings for matching
4. **reflexion_episodes** - Self-critique episode embeddings

## 2. Self-Learning Functions

### ruvector_enable_learning()
```sql
SELECT ruvector_enable_learning('content', '{"algorithm": "q_learning", "reward_decay": 0.95}'::jsonb);
```
Enables Q-Learning on content embeddings with exponential reward decay.

### ruvector_learn_from_feedback()
```sql
SELECT ruvector_learn_from_feedback('content', $1::jsonb)
```
Updates embeddings based on aggregated user feedback rewards by genre/pattern.

## 3. Learning Feedback Table

Records user actions with reward signals (-1.0 to +1.0):
- watched: +0.3 to +1.0 (based on completion %)
- skipped: -0.1 to -0.3
- rated high: +0.7 to +1.0
- added_watchlist: +0.5
- dismissed: -0.2

## 4. Recommendation Patterns Table

Stores learned strategies with context:
- pattern_type: cold_start, genre_match, similar_content, time_based, network_based
- success_rate: 0.0-1.0 (trained via Q-Learning)
- total_uses: Usage count
- avg_reward: Average feedback value
- Context: user_segment, time_of_day, platform, content_type_preference

## 5. Self-Learning Loop

```
User Interaction → Feedback Recording → Pattern Evaluation
                      ↓
                Reflexion Storage → Optimization Cycle (Daily)
                      ↓
        Cluster Detection → Pattern Creation → Quality Measurement
                      ↓
        Embedding Optimization (SIMD) → State Persistence
```

## 6. Metrics Tracked

Per Pattern:
- success_rate: Pattern win rate (0-1)
- total_uses: Application count
- avg_reward: Average feedback (-1 to 1)
- embedding: Context vector for similarity search

Per Feedback:
- reward: Outcome signal (-1 to 1)
- was_successful: Binary success
- user_action: Type of engagement
- recommendation_position: List placement

## 7. Embedding Optimization Over Time

**Cluster Identification**: Find content pairs with similarity > 0.75
**Pattern Evolution**: Generate patterns from cluster genres
**Quality Measurement**: Track intra-cluster similarity improvement
**Strategy Selection**: Q-Learning updates best strategy per episode
**Exploration Decay**: Epsilon-greedy (starts 0.30, min 0.05)

## 8. User Preference Vector Generation

Weighted average of watched content embeddings:
- Weight = user rating / 10 (default 0.5 if unrated)
- Normalized to L2 unit vector
- Updated whenever watch history changes

## 9. Pattern Selection

```
Context → Text Embedding → Search top-5 similar patterns
          ↓
Score = similarity × 0.4 + success_rate × 0.6
          ↓
Select highest-scoring pattern → Apply strategy
```

## 10. Key Insights

1. **4 RuVector tables** for semantic search at scale
2. **Closed-loop learning** from user feedback to pattern optimization
3. **SIMD acceleration** provides 5-8x performance improvement
4. **Q-Learning framework** with reward decay factor (0.95)
5. **User segmentation** (new/casual/regular/power) with different patterns
6. **Temporal awareness** (time of day, day of week, platform)
7. **Self-critique** via reflexion episodes for meta-learning
8. **Continuous refinement** of embeddings toward high-reward clusters
9. **Persistence** of learning state for recovery and analysis
10. **Scalability** via SIMD operations on millions of embeddings

## 11. Integration Points

- **RecommendationEngine**: Selects patterns, generates recommendations, records feedback
- **RuVectorOptimizer**: Runs daily optimization, learns from feedback, updates embeddings
- **EmbeddingService**: Generates 384-dim vectors for content, users, patterns, episodes

## 12. Performance

- Pattern matching: <5ms (top-5 nearest)
- Daily optimization cycle: ~20 seconds (5-8x faster with RuVector)
- Preference update: ~10ms per user
- Similarity calculation: O(1) SIMD operation

## File Locations

- Schema: `/src/db/schema-ruvector.sql`
- Engine: `/src/services/recommendation-engine.ts`
- Optimizer: `/src/services/ruvector-optimizer.ts`
- Embeddings: `/src/services/embedding-service.ts`
- Repository: `/src/db/repository.ts`
