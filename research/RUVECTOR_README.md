# RuVector TVDB Recommender System - Research & Analysis

## Overview

This directory contains comprehensive analysis of the RuVector-enabled self-learning recommendation system for TVDB content. The system uses PostgreSQL with RuVector (SIMD-optimized vector operations) to implement a closed-loop learning mechanism that continuously improves recommendation quality.

## Documents

### 1. RUVECTOR_ANALYSIS.md (Executive Summary)
**Purpose:** High-level overview of the RuVector schema and learning system
**Contents:**
- 4 RuVector-enabled tables (384-dimensional embeddings)
- Self-learning functions (ruvector_enable_learning, ruvector_learn_from_feedback)
- Learning feedback table structure and reward values
- Recommendation patterns table with context matching
- Complete self-learning loop workflow
- Metrics tracked at pattern, feedback, and system levels
- Embedding optimization over time
- User preference vector generation
- Pattern selection algorithm
- Integration points with core services

**Read this if:** You want a quick overview of how the system works

### 2. RUVECTOR_ARCHITECTURE.md (Technical Deep Dive)
**Purpose:** System architecture and detailed flow diagrams
**Contents:**
- Complete learning cycle (8 phases)
- PostgreSQL schema details (all 7 core tables)
- Recommendation generation flow with decision tree
- RuVector operations and hardware acceleration
- Pattern evolution over time (seeded → learned → converged)
- Learning state persistence mechanism
- Optimization cycle metrics
- User segmentation strategy (4 segments)
- Time-of-day patterns
- Performance characteristics (50-100ms per recommendation)
- File locations and modules

**Read this if:** You need to understand system design and data flows

### 3. RUVECTOR_QUICK_REFERENCE.md (Developer Guide)
**Purpose:** Quick reference for developers and system operators
**Contents:**
- Core tables summary table
- Learning feedback records and reward values
- RuVector SQL functions with examples
- Pattern types and contexts
- Preference vector generation algorithm
- Key metrics tracked
- Optimization cycle phases (1-6)
- Pattern selection algorithm pseudo-code
- Cold start handling
- User segmentation rules
- Time-based patterns
- Embedding model details (Xenova/all-MiniLM-L6-v2)
- Database indexes
- Learning loop example (day-by-day)
- Performance characteristics
- Testing queries
- Common issues & solutions
- Quick start code examples

**Read this if:** You're implementing features or debugging the system

## Key System Characteristics

### RuVector Tables (384-dimensional embeddings)

1. **content** - Semantic representation of shows/movies
2. **user_preferences** - Aggregated user taste profiles
3. **recommendation_patterns** - Pattern context for matching
4. **reflexion_episodes** - Learned experiences for self-critique

### Learning Functions

- `ruvector_enable_learning()` - Enables Q-Learning on content
- `ruvector_cosine_distance()` - SIMD-optimized similarity
- `ruvector_learn_from_feedback()` - Updates embeddings from feedback

### Self-Learning Loop

```
User Action → Feedback Collection → Pattern Evaluation
   ↓
Reflexion Storage → Daily Optimization Cycle
   ↓
Cluster Detection → Pattern Creation → Embedding Update
   ↓
Quality Measurement → State Persistence
```

### Reward System

- Watched 90%+: +0.9 to +1.0
- Watched 50-80%: +0.4 to +0.7
- Skipped: -0.2 to -0.3
- Rated high: +0.7 to +1.0
- Added watchlist: +0.5
- Dismissed: -0.2

### Performance

- Single recommendation: 50-100ms
- Pattern selection: <5ms
- Similarity calculation: O(1) with SIMD
- Daily optimization: ~20 seconds
- Speedup vs non-SIMD: 5-8x

## Core Services

### RecommendationEngine
**Location:** `/src/services/recommendation-engine.ts`

Responsibilities:
- Pattern selection based on user context
- Recommendation generation via pattern strategies
- Feedback recording for learning
- User segmentation (new/casual/regular/power)

### RuVectorOptimizer
**Location:** `/src/services/ruvector-optimizer.ts`

Responsibilities:
- Cluster identification via similarity search
- Pattern creation from clusters
- Reward aggregation and strategy learning
- Embedding optimization via RuVector
- Quality measurement
- Learning state persistence

### EmbeddingService
**Location:** `/src/services/embedding-service.ts`

Responsibilities:
- Generate 384-dimensional embeddings
- Support batch processing
- Implement caching (24-hour TTL)
- Compute cosine similarity
- Normalize vectors (L2)

### Repository
**Location:** `/src/db/repository.ts`

Responsibilities:
- Database operations for all tables
- User preference management
- Watch history and ratings
- Pattern and feedback operations
- Content search and similarity queries

## Data Model

### User Lifecycle

```
New (0 items)
  ↓ (cold start)
Casual (1-4 items)
  ↓ (genre learning)
Regular (5-19 items)
  ↓ (personalization)
Power (20+ items)
  ↓ (optimization)
```

### Pattern Lifecycle

```
Seeded (5 initial patterns)
  ↓
Tested (feedback collected)
  ↓
Scored (success rate calculated)
  ↓
Evolved (new patterns from clusters)
  ↓
Refined (low performers deprecated)
  ↓
Specialized (user-base optimized)
```

## Learning Mechanics

### Q-Learning Configuration

```json
{
  "algorithm": "q_learning",
  "reward_decay": 0.95,
  "learning_rate": 0.1,
  "exploration_rate": 0.3 (decays to 0.05)
}
```

### Exploration Strategy

- Epsilon-greedy approach
- Initial exploration: 30%
- Decay rate: 1% per cycle
- Minimum exploration: 5%

### Best Strategy Selection

Tracks top-performing strategy by genre:
- Drama: 0.85 avg reward
- SciFi: 0.72 avg reward
- Comedy: 0.65 avg reward

System learns which genres drive engagement.

## Integration Points

1. **Frontend** → RecommendationEngine.getRecommendations()
2. **User Interactions** → RecommendationEngine.recordFeedback()
3. **Daily Scheduler** → RuVectorOptimizer.runOptimizationCycle()
4. **Content Ingestion** → Repository.batchUpsertContent()
5. **User Profile Updates** → Repository.updateUserPreferenceVector()

## Testing & Validation

### Pattern Effectiveness

```sql
-- See pattern success rates
SELECT pattern_type, approach, success_rate, total_uses, avg_reward
FROM recommendation_patterns
ORDER BY success_rate DESC;

-- Analyze feedback quality
SELECT pattern_id,
       COUNT(*) as uses,
       SUM(CASE WHEN was_successful THEN 1 END)::float / COUNT(*) as success,
       AVG(reward) as avg_reward
FROM learning_feedback
WHERE created_at > NOW() - INTERVAL '7 days'
GROUP BY pattern_id
ORDER BY success DESC;
```

### Embedding Quality

```sql
-- Measure intra-cluster similarity
SELECT
  primary_genre,
  AVG(1 - ruvector_cosine_distance(e1, e2)) as similarity
FROM content c1
JOIN content c2 ON c1.primary_genre = c2.primary_genre
GROUP BY primary_genre;
```

## Common Operations

### Get Top Patterns
```sql
SELECT * FROM recommendation_patterns
ORDER BY success_rate DESC LIMIT 5;
```

### Find Similar Content
```sql
SELECT c.*, 1 - ruvector_cosine_distance(c.embedding, $1::ruvector)
FROM content c
WHERE 1 - ruvector_cosine_distance(c.embedding, $1::ruvector) > 0.7
ORDER BY ruvector_cosine_distance(c.embedding, $1::ruvector)
LIMIT 20;
```

### Track Optimization Progress
```sql
SELECT * FROM sync_status
WHERE sync_type = 'learning_state'
ORDER BY created_at DESC LIMIT 10;
```

## Files & Structure

```
research/tvdb-recommender/
├── src/
│   ├── db/
│   │   ├── schema-ruvector.sql          (4 RuVector tables + 7 total)
│   │   └── repository.ts                (Database operations)
│   ├── services/
│   │   ├── recommendation-engine.ts     (Pattern selection & generation)
│   │   ├── ruvector-optimizer.ts        (Daily optimization)
│   │   ├── embedding-service.ts         (Vector generation)
│   │   └── tvdb-data-ingestion.ts       (Content fetching)
│   ├── types/
│   │   └── index.ts                    (Type definitions)
│   └── workflows/
│       └── recommendation-workflow.ts   (Orchestration)
└── [Analysis Documents]
    ├── RUVECTOR_ANALYSIS.md             (This document)
    ├── RUVECTOR_ARCHITECTURE.md         (Technical deep dive)
    └── RUVECTOR_QUICK_REFERENCE.md      (Developer guide)
```

## Key Insights

1. **Scalable Vector Operations**: RuVector provides 5-8x speedup via SIMD
2. **Automated Pattern Learning**: System discovers best strategies from data
3. **User-Centric Optimization**: Learns what works for different user segments
4. **Temporal Awareness**: Patterns include time-of-day and platform context
5. **Self-Critique Learning**: Reflexion episodes enable meta-learning
6. **Continuous Improvement**: Daily optimization cycles refine embeddings
7. **Graceful Degradation**: Cold-start strategy for new users
8. **Persistence**: Learning state survives restarts
9. **Measurable Quality**: Multiple metrics track improvement
10. **Flexible Architecture**: Easy to add new pattern types

## Recommendations for Further Development

1. Implement A/B testing framework for pattern comparison
2. Add cross-genre preference discovery
3. Implement user clustering for cohort analysis
4. Add real-time feedback loop (minutes vs. hours)
5. Implement collaborative filtering via user similarity
6. Add content-based filtering for new items
7. Implement bandit algorithms for exploration
8. Add temporal dynamics (trending content)
9. Implement sequence modeling for series watching
10. Add user feedback confidence scoring

## Related Documentation

- **SQLSchema**: `/src/db/schema-ruvector.sql`
- **TypeDefinitions**: `/src/types/index.ts`
- **APIDocumentation**: `/docs/API.md` (if available)
- **ConfigurationGuide**: `/docs/CONFIG.md` (if available)

## Contact & Questions

For questions about the RuVector system:
1. Check RUVECTOR_QUICK_REFERENCE.md for specific queries
2. Review RUVECTOR_ARCHITECTURE.md for system design
3. Examine RUVECTOR_ANALYSIS.md for high-level overview
4. Check source files in `/src/services/` for implementation details

---

**Analysis Created**: 2025-12-07
**System**: TVDB Self-Learning Recommender with RuVector
**Status**: Production-ready for learning and optimization
