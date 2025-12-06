# RuVector PostgreSQL - AI Media Gateway

[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16+-blue.svg)](https://www.postgresql.org/)
[![pgvector](https://img.shields.io/badge/pgvector-0.6.0-green.svg)](https://github.com/pgvector/pgvector)
[![npm](https://img.shields.io/badge/npm-@ruvector/postgres--cli-red.svg)](https://www.npmjs.com/package/@ruvector/postgres-cli)
[![Hackathon](https://img.shields.io/badge/TV5-Hackathon-orange.svg)](https://agentics.org/hackathon)

> **Next-generation media discovery powered by self-learning PostgreSQL vector intelligence**

## 🎬 The Problem We're Solving

Every night, millions spend up to **45 minutes deciding what to watch** — that's billions of hours lost globally. Why? Because current recommendation systems:

- **Don't understand relationships** — They miss that loving "Inception" means you might love "Dark" (both are mind-bending, non-linear narratives)
- **Can't learn across platforms** — Your Netflix preferences are invisible to Disney+
- **Don't scale intelligently** — Searching 80K+ titles efficiently requires specialized techniques
- **Lack true personalization** — They recommend what's popular, not what's *you*

## 💡 Our Solution: Intelligent PostgreSQL

We've built a **self-learning media gateway** directly in PostgreSQL that uses the same AI techniques powering ChatGPT and modern recommendation systems — but running in your database with <5ms latency.

### What Makes This Different?

| Traditional Search | RuVector PostgreSQL |
|-------------------|---------------------|
| Keyword matching ("action movies") | **Semantic understanding** ("movies that make you think") |
| Flat category filters | **Hierarchical learning** (knows Noir is a type of Thriller) |
| Isolated user data | **Federated intelligence** (learns across platforms privately) |
| Slow at scale | **80K movies in 45ms** across 8 platforms |
| Static algorithms | **Self-optimizing** through attention and feedback |

---

## 🧠 The AI Techniques (In Plain English)

### 1. **Graph Neural Networks** — Understanding Relationships
Instead of treating movies as isolated items, we model the *connections*:
- You watched Movie A → Movie A stars Actor X → Actor X directed Movie B → **You might love Movie B**
- This is how Netflix knows you'll like a show before it even exists in your history

### 2. **Attention Mechanisms** — Focusing on What Matters
Not all your watch history matters equally. The system learns to *pay attention* to:
- Your recent 5-star ratings (high weight)
- Movies you finished vs. abandoned (completion signals)
- Time-of-day preferences (weekend binge vs. weeknight quick watch)

### 3. **Hyperbolic Embeddings** — Natural Hierarchies
Genres aren't flat lists. They're trees:
```
Action → Thriller → Psychological Thriller → Mind-Bending
                 → Crime Thriller → Heist
```
Our Poincaré ball embeddings capture this naturally, so "mind-bending thriller" finds Inception, not Fast & Furious.

### 4. **Federated Learning** — Privacy-Preserving Intelligence
Your Netflix preferences can improve Disney+ recommendations *without sharing your actual watch history*. Each platform contributes anonymized preference vectors that aggregate into better global recommendations.

### 5. **Self-Optimization** — Getting Smarter Over Time
The system uses **quality-weighted aggregation**: recommendations that get positive feedback gain influence, poor recommendations lose weight. No manual tuning required.

---

## 📊 Why PostgreSQL?

Large-scale media systems need:

| Requirement | How We Solve It |
|-------------|-----------------|
| **Millisecond latency** | HNSW indexes: <5ms for 10K vectors |
| **Massive scale** | Sharding: 80K movies across 8 platforms |
| **Consistency** | Raft consensus: distributed agreement |
| **Memory efficiency** | Quantization: 32x compression (384 dims → 48 bytes) |
| **Hybrid search** | Vector + BM25: semantic + keyword combined |
| **Production ready** | PostgreSQL: battle-tested, ACID-compliant |

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                 MEDIA GATEWAY - POSTGRESQL ARCHITECTURE                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     PostgreSQL 16 + pgvector 0.6.0                  │   │
│  │                    "What should I watch tonight?"                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│           ┌────────────────────────┼────────────────────────┐              │
│           │                        │                        │              │
│           ▼                        ▼                        ▼              │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐        │
│  │  HNSW Search    │    │  Raft Consensus │    │   Shard Scale   │        │
│  │  <5ms latency   │    │  Multi-platform │    │  80K+ movies    │        │
│  │  Semantic match │    │  aggregation    │    │  8 platforms    │        │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘        │
│           │                        │                        │              │
│           └────────────────────────┼────────────────────────┘              │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Self-Learning Features                            │   │
│  │  • Graph Neural Networks: User→Movie→Actor→Director connections     │   │
│  │  • Attention: Weight recent preferences, ignore noise               │   │
│  │  • Hyperbolic: Genre trees (Action→Thriller→Noir→Neo-Noir)          │   │
│  │  • Federated: Cross-platform learning without data sharing          │   │
│  │  • Quantization: 80K movies compressed for edge/mobile              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 Quick Start

### Install RuVector PostgreSQL CLI

```bash
npm install -g @ruvector/postgres-cli
# or
npx @ruvector/postgres-cli --help
```

### Key Commands for Media Gateway

```bash
# Setup media catalog
ruvector-pg setup                    # Install pgvector
ruvector-pg create-table media_embeddings 768  # Movie embeddings

# Index 80K movies
ruvector-pg import movies.json       # Bulk import
ruvector-pg index create --type hnsw # Create HNSW index

# Semantic search: "dark sci-fi thriller like Blade Runner"
ruvector-pg search --query "dark sci-fi thriller" --limit 10

# Performance benchmarks
ruvector-pg benchmark --vectors 10000
```

---

## 📊 Media Gateway Benchmarks

Run the PostgreSQL benchmark suite for media discovery operations:

```bash
# Setup (10K movies, genre graphs, preference vectors)
psql -d postgres -f benchmarks/ruvector_benchmark_optimized.sql

# Run benchmarks
psql -d postgres -f benchmarks/run_benchmarks_optimized.sql

# TV5 scale test (80K movies, 8 streaming platforms)
psql -d postgres -f benchmarks/tv5_raft_scale_benchmark.sql
```

### Performance Results

| Media Gateway Operation | Dataset | Latency |
|------------------------|---------|---------|
| **"Movies like Inception"** | 10K movies | <5ms |
| **Genre hierarchy search** | 5K (Poincaré) | 15ms |
| **Actor/director graph** | 1K nodes | 25ms |
| **Hybrid keyword + semantic** | 1K docs | 8ms |
| **Cross-platform consensus** | 5 platforms | <1ms |
| **Multi-platform aggregate** | 100 users | 12ms |
| **Full catalog search** | 80K (8 shards) | 45ms |
| **Compressed catalog** | 1K (binary) | 3ms |

---

## 🔧 PostgreSQL Functions for Media Discovery

### 1. Genre Hierarchies (Hyperbolic Embeddings)

Model hierarchical genre relationships: Action → Thriller → Noir → Neo-Noir

```sql
-- Poincaré distance for genre hierarchy
-- Closer in hierarchy = smaller distance
SELECT m.title, poincare_distance(
    m.genre_embedding,
    (SELECT genre_embedding FROM genres WHERE name = 'Film Noir'),
    -1.0
) AS genre_distance
FROM movies m
ORDER BY genre_distance
LIMIT 10;

-- Find movies "between" two genres (Möbius midpoint)
SELECT mobius_add(
    (SELECT embedding FROM genres WHERE name = 'Sci-Fi'),
    (SELECT embedding FROM genres WHERE name = 'Horror'),
    -1.0
) AS scifi_horror_midpoint;
```

### 2. Recommendation Graphs (GraphSAGE)

User-Movie-Actor-Director relationship graphs for recommendations:

```sql
-- Aggregate neighbor features for collaborative filtering
WITH user_watched AS (
    SELECT u.id, u.preference_vector, array_agg(m.embedding) AS movie_embeddings
    FROM users u
    JOIN watch_history w ON u.id = w.user_id
    JOIN movies m ON w.movie_id = m.id
    WHERE u.id = 'user-123'
    GROUP BY u.id, u.preference_vector
)
SELECT graphsage_mean(preference_vector, movie_embeddings)
FROM user_watched;
```

### 3. Hybrid Search: Semantic + Keywords

"Dark sci-fi like Blade Runner but newer":

```sql
WITH semantic_matches AS (
    SELECT id, title, embedding <=> query_embedding AS distance
    FROM movies
    ORDER BY embedding <=> query_embedding
    LIMIT 100
),
keyword_matches AS (
    SELECT id, ts_rank_cd(search_vector,
        plainto_tsquery('dark sci-fi blade runner')) AS rank
    FROM movies
    WHERE search_vector @@ plainto_tsquery('dark sci-fi blade runner')
),
combined AS (
    SELECT
        COALESCE(s.id, k.id) AS id,
        (1.0 / (1.0 + COALESCE(s.distance, 2))) * 0.7 +
        COALESCE(k.rank, 0) * 0.3 AS score
    FROM semantic_matches s
    FULL OUTER JOIN keyword_matches k ON s.id = k.id
)
SELECT m.title, m.year, c.score
FROM combined c
JOIN movies m ON c.id = m.id
WHERE m.year >= 2010  -- "but newer"
ORDER BY score DESC
LIMIT 10;
```

### 4. Cross-Platform Aggregation (Raft Consensus)

Aggregate recommendations across Netflix, Disney+, HBO, etc.:

```sql
-- Each streaming platform votes on recommendations
SELECT raft_request_vote(
    1,                    -- term
    'netflix-recommender', -- candidate
    'disney-recommender',  -- voter
    last_recommendation_id,
    current_term
);

-- Elect leader for final recommendation
SELECT * FROM raft_elect_leader(5);  -- 5 platforms

-- Check Byzantine tolerance (handles 1 malicious platform in 5)
SELECT * FROM check_byzantine_tolerance(5, 1);
```

### 5. Federated Preference Learning

Privacy-preserving aggregation of user preferences:

```sql
-- Each platform has local user preference embeddings
INSERT INTO federated_agents (agent_id, shard_id, embedding, quality)
VALUES
    ('netflix-prefs', 0, user_embedding_from_netflix, 0.9),
    ('disney-prefs', 1, user_embedding_from_disney, 0.85),
    ('hbo-prefs', 2, user_embedding_from_hbo, 0.8);

-- FedAvg: Weighted average of quality-filtered preferences
SELECT federated_aggregate('user-123', 0.7);
-- Returns: Combined preference vector without sharing watch history
```

### 6. Compressed Catalog (Quantization)

Store 80K movies efficiently for mobile/edge:

```sql
-- Binary quantize (32x compression, 384 dims → 48 bytes)
UPDATE movies SET binary_embedding = binary_quantize(embedding);

-- Fast approximate search with Hamming distance
SELECT title, hamming_distance(
    binary_embedding,
    (SELECT binary_embedding FROM movies WHERE title = 'Inception')
) AS similarity
FROM movies
ORDER BY similarity
LIMIT 10;  -- 15x faster than cosine distance
```

---

## 📈 Scale: 80K Movies Across 8 Platforms

Shard movies by streaming platform for distributed search:

```sql
-- Sharded by platform: 0=Netflix, 1=Disney+, 2=HBO, etc.
CREATE TABLE platform_movies (
    platform_id int NOT NULL,
    movie_id bigserial,
    embedding vector(384),
    PRIMARY KEY (platform_id, movie_id)
);

-- Insert 10K movies per platform
SELECT scale_insert_vectors(0, 10000, 384);  -- Netflix
SELECT scale_insert_vectors(1, 10000, 384);  -- Disney+
-- ... 8 platforms total = 80K movies

-- Search across user's subscribed platforms only
SELECT * FROM scale_search_vectors(
    query_embedding,
    10,                  -- top 10
    ARRAY[0, 2, 5]       -- Netflix, HBO, Paramount+
);
```

---

## 📁 Benchmark Files

```
benchmarks/
├── ruvector_benchmark_optimized.sql   # 10K movies, genre graphs
├── run_benchmarks_optimized.sql       # Performance tests
├── tv5_raft_scale_benchmark.sql       # 80K movies, 8 platforms
└── run_tv5_benchmarks.sql             # Cross-platform tests
```

---

## ⚙️ PostgreSQL Configuration for Media Gateway

```sql
-- Optimized for media catalog workloads
SET max_parallel_workers_per_gather = 4;
SET work_mem = '256MB';
SET effective_cache_size = '4GB';
SET hnsw.ef_search = 100;  -- Balance speed/accuracy
SET jit = on;
```

---

## 🔗 Links

- **npm:** [@ruvector/postgres-cli](https://www.npmjs.com/package/@ruvector/postgres-cli)
- **Hackathon:** [agentics.org/hackathon](https://agentics.org/hackathon)
- **pgvector:** [github.com/pgvector/pgvector](https://github.com/pgvector/pgvector)

---

<div align="center">

**RuVector PostgreSQL** - AI Media Gateway for the TV5 Hackathon

*Solving the 45-minute problem with <5ms semantic search*

</div>
