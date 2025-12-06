# TV5 Hackathon: AI Media Gateway

[![Agentics](https://img.shields.io/badge/Agentics-TV5_Hackathon-orange.svg)](https://agentics.org/hackathon)
[![RuVector](https://img.shields.io/badge/RuVector-PostgreSQL-green.svg)](https://www.npmjs.com/package/@ruvector/postgres-cli)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16+-blue.svg)](https://www.postgresql.org/)
[![Discord](https://img.shields.io/badge/Discord-Agentics-7289da.svg)](https://discord.agentics.org)

> **Agentics Foundation TV5 Hackathon Entry: Self-learning media discovery with RuVector PostgreSQL**

---

## 🏆 Hackathon Track: Entertainment Discovery

**Challenge:** Every night, millions spend up to **45 minutes deciding what to watch** — billions of hours lost globally.

**Our Solution:** An AI-powered **Media Gateway** using RuVector PostgreSQL — a high-performance vector database with self-learning capabilities, graph neural networks, and distributed consensus for cross-platform recommendations.

---

## 🎬 The Problem

Current streaming recommendation systems fail because they:

| Problem | Impact |
|---------|--------|
| **Siloed platforms** | Netflix doesn't know your Disney+ preferences |
| **Keyword-only search** | "Movies that make you think" returns nothing |
| **Flat categories** | Thriller ≠ Psychological Thriller ≠ Mind-Bending |
| **No learning** | Same bad recommendations after 100 thumbs-down |
| **Slow at scale** | Searching 80K+ titles takes seconds, not milliseconds |

---

## 💡 Our Solution: RuVector PostgreSQL Media Gateway

We built a **self-learning recommendation engine** directly in PostgreSQL using RuVector — delivering the same AI techniques powering modern recommendation systems with <5ms latency.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│               TV5 MEDIA GATEWAY - RUVECTOR POSTGRESQL                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│     "What should I watch tonight?"                                          │
│                         │                                                   │
│                         ▼                                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     RuVector PostgreSQL Engine                       │   │
│  │            High-performance vector operations + self-learning        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                         │                                                   │
│       ┌─────────────────┼─────────────────┐                                │
│       │                 │                 │                                │
│       ▼                 ▼                 ▼                                │
│  ┌─────────┐      ┌──────────┐      ┌──────────┐                          │
│  │  HNSW   │      │   Raft   │      │  Scale   │                          │
│  │ Search  │      │Consensus │      │ Shards   │                          │
│  │  <5ms   │      │Multi-plat│      │  80K+    │                          │
│  └─────────┘      └──────────┘      └──────────┘                          │
│       │                 │                 │                                │
│       └─────────────────┼─────────────────┘                                │
│                         ▼                                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Self-Learning AI Features                         │   │
│  │                                                                       │   │
│  │  🧠 Graph Neural Networks                                             │   │
│  │     User→Movie→Actor→Director relationship learning                  │   │
│  │                                                                       │   │
│  │  🎯 Attention Mechanisms                                              │   │
│  │     Focus on recent ratings, ignore abandoned watches                │   │
│  │                                                                       │   │
│  │  🌳 Hyperbolic Embeddings                                             │   │
│  │     Genre hierarchies: Action→Thriller→Noir→Neo-Noir                 │   │
│  │                                                                       │   │
│  │  🔒 Federated Learning                                                │   │
│  │     Cross-platform intelligence without sharing watch history        │   │
│  │                                                                       │   │
│  │  ⚡ Self-Optimization                                                 │   │
│  │     Quality-weighted feedback improves recommendations over time     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🧠 AI Techniques Explained

### 1. Graph Neural Networks — Understanding Connections

Instead of treating movies as isolated items, we model relationships:

```
You watched Inception → Stars Leonardo DiCaprio → Also in Shutter Island → You'll love it
```

The system learns through actor, director, writer, and genre connections — recommending content you'll love before you even know it exists.

### 2. Attention Mechanisms — Focusing on What Matters

Not all watch history is equal. The system learns to weight:
- **Recent 5-star ratings** → High influence
- **Completed movies** → Strong signal
- **Abandoned at 10 minutes** → Negative signal
- **Weekend binges vs. weeknight picks** → Context awareness

### 3. Hyperbolic Embeddings — Natural Genre Hierarchies

Genres are trees, not flat lists:

```
Action
  └→ Thriller
       └→ Psychological Thriller
            └→ Mind-Bending (Inception, Dark, Memento)
       └→ Crime Thriller
            └→ Heist (Ocean's 11, Heat)
```

Searching "mind-bending thriller" finds Inception, not Fast & Furious.

### 4. Federated Learning — Privacy-Preserving Intelligence

Your Netflix preferences can improve Disney+ recommendations **without sharing your watch history**:
- Each platform generates anonymous preference vectors
- Vectors aggregate through FedAvg algorithm
- Individual viewing data never leaves the platform

### 5. Self-Optimization — Continuous Improvement

Recommendations that get positive feedback gain influence. Bad recommendations lose weight. The system improves automatically — no manual tuning required.

---

## 📊 Performance Benchmarks

| Operation | Dataset | Latency |
|-----------|---------|---------|
| **Semantic search** ("movies like Inception") | 10K movies | <5ms |
| **Genre hierarchy** (Poincaré distance) | 5K embeddings | 15ms |
| **Relationship graph** (GraphSAGE) | 1K nodes | 25ms |
| **Hybrid search** (semantic + keywords) | 1K docs | 8ms |
| **Cross-platform consensus** (Raft) | 5 platforms | <1ms |
| **Federated aggregate** (FedAvg) | 100 users | 12ms |
| **Full catalog search** (sharded) | 80K movies | 45ms |
| **Compressed search** (quantized) | 1K binary | 3ms |

---

## 🚀 Quick Start

### Install RuVector PostgreSQL CLI

```bash
npm install -g @ruvector/postgres-cli

# Or use npx
npx @ruvector/postgres-cli --help
```

### Run Media Gateway Benchmarks

```bash
# Setup: 10K movies, genre graphs, user preferences
psql -d postgres -f benchmarks/ruvector_benchmark_optimized.sql
psql -d postgres -f benchmarks/run_benchmarks_optimized.sql

# Scale test: 80K movies across 8 streaming platforms
psql -d postgres -f benchmarks/tv5_raft_scale_benchmark.sql
psql -d postgres -f benchmarks/run_tv5_benchmarks.sql
```

### Example: Semantic Movie Search

```bash
# Find movies semantically similar to a query
ruvector-pg search --query "dark psychological thriller with time loops" --limit 10
```

---

## 🔧 Key SQL Functions

### Semantic Search
```sql
-- Find movies like "Inception"
SELECT title, embedding <=> query_embedding AS similarity
FROM movies
ORDER BY similarity
LIMIT 10;
```

### Genre Hierarchy (Hyperbolic)
```sql
-- Find movies in the "Mind-Bending" genre branch
SELECT title, poincare_distance(genre_embedding, target_genre, -1.0) AS distance
FROM movies
ORDER BY distance
LIMIT 10;
```

### Relationship Graph (GraphSAGE)
```sql
-- Aggregate user preferences from watch history
SELECT graphsage_mean(user.preference_vector, array_agg(movie.embedding))
FROM users JOIN watch_history JOIN movies
WHERE user_id = 'user-123';
```

### Cross-Platform Consensus (Raft)
```sql
-- Elect leader for recommendation aggregation
SELECT * FROM raft_elect_leader(5);  -- 5 streaming platforms
```

### Federated Learning (FedAvg)
```sql
-- Aggregate preferences without sharing watch history
SELECT federated_aggregate('user-123', 0.7);
```

---

## 📁 Project Structure

```
hackathon-tv5/
├── benchmarks/
│   ├── ruvector_benchmark_optimized.sql   # 10K movie benchmark
│   ├── run_benchmarks_optimized.sql       # Performance tests
│   ├── tv5_raft_scale_benchmark.sql       # 80K scale + Raft
│   └── run_tv5_benchmarks.sql             # Cross-platform tests
├── apps/
│   └── media-discovery/                   # Next.js demo app
└── README.md
```

---

## 🔗 Links

- **Hackathon:** [agentics.org/hackathon](https://agentics.org/hackathon)
- **Discord:** [discord.agentics.org](https://discord.agentics.org)
- **RuVector CLI:** [@ruvector/postgres-cli](https://www.npmjs.com/package/@ruvector/postgres-cli)
- **RuVector:** [npmjs.com/package/ruvector](https://www.npmjs.com/package/ruvector)

---

<div align="center">

## 🏆 Agentics Foundation TV5 Hackathon

**AI Media Gateway** — Solving the 45-minute problem with self-learning PostgreSQL

*Semantic search in <5ms • 80K movies across 8 platforms • Privacy-preserving cross-platform learning*

[Join the Hackathon](https://agentics.org/hackathon) • [Discord](https://discord.agentics.org)

</div>
