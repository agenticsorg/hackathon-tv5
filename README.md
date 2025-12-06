# TV5 Hackathon: AI Media Gateway

[![Agentics](https://img.shields.io/badge/Agentics-TV5_Hackathon-orange.svg)](https://agentics.org/hackathon)
[![RuVector](https://img.shields.io/badge/RuVector-PostgreSQL-green.svg)](https://www.npmjs.com/package/@ruvector/postgres-cli)
[![Docker](https://img.shields.io/badge/Docker-ruvector--postgres-blue.svg)](https://hub.docker.com/r/ruvnet/ruvector-postgres)
[![npm](https://img.shields.io/badge/npm-ruvector-red.svg)](https://www.npmjs.com/package/ruvector)
[![Discord](https://img.shields.io/badge/Discord-Agentics-7289da.svg)](https://discord.agentics.org)

> **Agentics Foundation TV5 Hackathon Entry: Self-learning media discovery powered by RuVector**

---

## 🏆 Hackathon Track: Entertainment Discovery

**Challenge:** Every night, millions spend up to **45 minutes deciding what to watch** — billions of hours lost globally.

**Our Solution:** An AI-powered **Media Gateway** using [RuVector](https://github.com/ruvnet/ruvector) — a distributed vector database that learns. Unlike traditional vector DBs that just store and search, RuVector's index **improves itself** through Graph Neural Networks, routes AI requests intelligently, and scales horizontally with Raft consensus.

---

## ⚡ Quick Start

```bash
# Run RuVector PostgreSQL with Docker
docker run -d --name ruvector-pg \
  -e POSTGRES_PASSWORD=secret \
  -p 5432:5432 \
  ruvnet/ruvector-postgres:latest

# Connect and start using 53+ AI-powered SQL functions
psql -h localhost -U postgres -d postgres
```

```sql
-- Create vector table with HNSW index
CREATE TABLE movies (id serial, title text, embedding vector(768));
CREATE INDEX ON movies USING hnsw (embedding vector_cosine_ops);

-- Self-learning search that improves over time
SELECT ruvector_adaptive_search(query_embedding, 'movies', 100);
```

---

## 🎬 Why RuVector for Media Discovery?

### pgvector vs ruvector-postgres

| Feature | pgvector | ruvector-postgres |
|---------|----------|-------------------|
| Vector Search | HNSW, IVFFlat | HNSW, IVFFlat (SIMD optimized) |
| Distance Metrics | 3 | **8+ (including hyperbolic)** |
| Attention Mechanisms | ❌ | **39 types** |
| Graph Neural Networks | ❌ | **GCN, GraphSAGE, GAT** |
| Hyperbolic Embeddings | ❌ | **Poincaré, Lorentz** |
| Sparse Vectors / BM25 | Partial | **Full support (14 functions)** |
| Self-Learning | ❌ | **ReasoningBank (7 functions)** |
| Agent Routing | ❌ | **Tiny Dancer (11 functions)** |
| Graph/Cypher Queries | ❌ | **Full support (8 functions)** |
| AVX-512/NEON SIMD | Partial | **Full (~2x faster)** |
| SQL Functions | ~10 | **53+** |

### Traditional vs RuVector

| Traditional Approach | RuVector Media Gateway |
|---------------------|------------------------|
| Static search results | **Self-learning GNN** — results improve over time |
| One-size-fits-all ranking | **39 attention mechanisms** — focus on what matters to YOU |
| Isolated platform data | **Federated Raft consensus** — aggregate across Netflix, Disney+, HBO |
| Slow at scale | **61µs latency** — instant recommendations from 80K+ titles |
| Manual tuning | **SONA runtime adaptation** — learns from feedback without retraining |
| Flat genre categories | **Hyperbolic embeddings** — natural genre hierarchies |
| Keyword matching | **Cypher graph queries** — traverse actor→movie→director relationships |

---

## 🧠 How RuVector Makes Recommendations Smarter

### The Self-Learning Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    TV5 MEDIA GATEWAY - RUVECTOR                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  "What should I watch tonight?"                                             │
│              │                                                              │
│              ▼                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         RuVector Engine                              │   │
│  │   Query → HNSW Index → GNN Layer → Enhanced Results                 │   │
│  │                  ↑                      │                            │   │
│  │                  └──── learns from ─────┘                            │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│              │                                                              │
│    ┌─────────┼─────────┬─────────────┬──────────────┐                      │
│    │         │         │             │              │                      │
│    ▼         ▼         ▼             ▼              ▼                      │
│ ┌──────┐ ┌──────┐ ┌─────────┐ ┌──────────┐ ┌────────────┐                 │
│ │ HNSW │ │ Raft │ │  SONA   │ │  Cypher  │ │ Compression│                 │
│ │<61µs │ │Consns│ │LoRA+EWC │ │  Graphs  │ │  2-32x     │                 │
│ └──────┘ └──────┘ └─────────┘ └──────────┘ └────────────┘                 │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    AI Features for Media                             │   │
│  │                                                                       │   │
│  │  🧠 GNN Self-Learning        Results improve with every search       │   │
│  │  🎯 39 Attention Mechanisms  Flash, linear, graph, hyperbolic        │   │
│  │  🌳 Hyperbolic Embeddings    Genre trees: Action→Thriller→Noir       │   │
│  │  🔗 Cypher Graph Queries     MATCH (you)-[:WATCHED]->(similar)       │   │
│  │  🔒 Federated Learning       Cross-platform without sharing history  │   │
│  │  ⚡ SONA Runtime Learning    Adapts from feedback in <0.8ms          │   │
│  │  📦 Auto-Compression         Hot movies: f32, Archive: 32x smaller   │   │
│  │  🚀 Tiny Dancer Routing      Route to best recommendation engine     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🔑 Key RuVector Features for Media Gateway

### 1. **Self-Learning GNN** — Search Gets Smarter
Traditional vector search returns static results. RuVector's GNN layer learns which paths lead to good recommendations:

```
You search "mind-bending thriller" → Results include Inception
You click Inception → GNN reinforces this path
Next user searches similar query → Better results immediately
```

### 2. **SONA Runtime Adaptation** — Learns Without Retraining
The Self-Optimizing Neural Architecture enables continuous improvement:
- **MicroLoRA (rank 1-2)** — Instant adaptation to your taste
- **BaseLoRA (rank 4-16)** — Long-term preference learning
- **EWC++** — Doesn't forget what it learned yesterday
- **ReasoningBank** — Stores successful recommendation patterns

```javascript
// System learns from your feedback in <0.8ms
engine.learn_from_feedback(LearningSignal.positive(watchTime, rating));
```

### 3. **39 Attention Mechanisms** — Focus on What Matters
Not all watch history is equal. RuVector includes specialized attention for:

| Mechanism | Media Gateway Use |
|-----------|-------------------|
| **FlashAttention** | Process 8K+ movie descriptions efficiently |
| **HyperbolicAttention** | Navigate genre hierarchies |
| **GraphAttention** | Weight actor/director relationships |
| **LinearAttention** | Real-time streaming recommendations |
| **SparseAttention** | Focus on key moments, ignore noise |

### 4. **Cypher Graph Queries** — Relationship Intelligence
Query the user-movie-actor-director graph like Neo4j:

```cypher
-- Find movies through relationship paths
MATCH (you:User)-[:WATCHED]->(m:Movie)-[:STARS]->(a:Actor)-[:ALSO_IN]->(rec:Movie)
WHERE rec.rating > 7.5
RETURN rec ORDER BY rec.similarity DESC LIMIT 10
```

### 5. **Hyperbolic Embeddings** — Natural Hierarchies
Genres aren't flat. Poincaré ball embeddings capture the tree structure:

```
Action
  └→ Thriller
       └→ Psychological Thriller
            └→ Mind-Bending (Inception, Memento, Dark)
       └→ Crime Thriller
            └→ Heist (Ocean's Eleven, Heat)
```

Search "mind-bending thriller" → Finds Inception, NOT Fast & Furious.

### 6. **Automatic Compression Tiers** — Scale Efficiently
RuVector automatically manages hot vs. cold data:

| Your Data | Format | Compression | Example |
|-----------|--------|-------------|---------|
| **Trending now** | f32 | 1x | Top 100 movies this week |
| **Recent releases** | f16 | 2x | 2024 releases |
| **Popular classics** | PQ8 | 8x | Frequently searched |
| **Full catalog** | PQ4 | 16x | 80K movies |
| **Deep archive** | Binary | 32x | Rarely accessed |

### 7. **Raft Consensus** — Cross-Platform Aggregation
Aggregate recommendations from multiple streaming platforms:

```javascript
// 5 platforms vote on recommendations
const cluster = ['netflix', 'disney', 'hbo', 'paramount', 'apple'];
const consensus = await raft.elect_leader(cluster);
const aggregated = await consensus.aggregate_recommendations(userId);
```

### 8. **Tiny Dancer Routing** — Intelligent AI Orchestration
Route queries to the optimal recommendation engine:

```javascript
const router = new ruvector.Router();
// Routes to: content-based, collaborative, trending, or hybrid
const decision = router.route(query, { optimize: 'relevance' });
```

---

## 📊 Performance Benchmarks

| Operation | Latency | Throughput |
|-----------|---------|------------|
| **HNSW Search (k=10)** | 61µs | 16,400 QPS |
| **HNSW Search (k=100)** | 164µs | 6,100 QPS |
| **GNN Enhanced Search** | <1ms | 1,000 QPS |
| **SONA Learning Step** | <0.8ms | 1,250/sec |
| **Cypher Graph Query** | 2-5ms | 200 QPS |
| **Poincaré Distance** | 15ms | 66 QPS |
| **Cross-Platform Raft** | <1ms | 1,000 QPS |
| **80K Catalog (sharded)** | 45ms | 22 QPS |

---

## 🚀 Quick Start

### Install RuVector

```bash
# All-in-one package (vectors, graphs, GNN, routing)
npm install ruvector

# Or use instantly
npx ruvector
```

### Media Gateway Example

```javascript
const ruvector = require('ruvector');

// Create vector database with GNN enhancement
const db = new ruvector.VectorDB(768);  // embedding dimension

// Index movies with Cypher relationships
db.execute(`
  CREATE (m:Movie {title: 'Inception', embedding: $embedding})
  CREATE (a:Actor {name: 'Leonardo DiCaprio'})
  CREATE (m)-[:STARS]->(a)
`, { embedding: inceptionEmbedding });

// GNN-enhanced semantic search
const gnn = new ruvector.GNNLayer(768, 1024, 8);  // 8 attention heads
const results = gnn.forward(queryEmbedding, neighbors, weights);

// Hyperbolic genre search
const genreResults = ruvector.hyperbolic.search(
  queryEmbedding,
  'poincare',
  { curvature: -1.0 }
);

// Route to best recommendation strategy
const router = new ruvector.Router();
const strategy = router.route(query, {
  candidates: ['content', 'collaborative', 'trending'],
  optimize: 'relevance'
});
```

### Run PostgreSQL Benchmarks

The benchmarks use **ruvector-postgres** — a drop-in pgvector replacement with **53+ SQL functions**:

```bash
# Option 1: Docker (recommended)
docker pull ruvnet/ruvector-postgres
docker run -d --name ruvector-pg \
  -e POSTGRES_PASSWORD=secret \
  -p 5432:5432 \
  ruvnet/ruvector-postgres:latest

# Option 2: CLI install into existing PostgreSQL
npm install -g @ruvector/postgres-cli
ruvector-pg install --method native

# Option 3: npx (no install required)
npx @ruvector/postgres-cli@latest install --method docker --port 5432

# Run media gateway benchmarks
psql -d postgres -f benchmarks/ruvector_benchmark_optimized.sql
psql -d postgres -f benchmarks/tv5_raft_scale_benchmark.sql
```

### RuVector-Postgres SQL Functions for Media Gateway

```sql
-- Hyperbolic embeddings for genre hierarchies
SELECT ruvector_poincare_distance(movie_a.embedding, movie_b.embedding, -1.0);
SELECT ruvector_mobius_add(genre_a, genre_b, -1.0);  -- Combine genres

-- Graph Neural Networks for recommendation graphs
SELECT ruvector_gnn_graphsage_layer(user_features, movie_features, weights);
SELECT ruvector_gnn_gat_layer(features, adjacency, attention_weights);

-- 39 Attention mechanisms
SELECT ruvector_attention_multi_head(query, keys, values, 8);  -- 8 heads
SELECT ruvector_attention_flash(query, keys, values, 64);      -- Memory efficient

-- Sparse vectors + BM25 for hybrid search
SELECT ruvector_bm25_score(query_terms, doc_freqs, doc_len, avg_len, total);
SELECT ruvector_sparse_cosine(user_prefs, movie_tags);

-- Agent routing (Tiny Dancer)
SELECT ruvector_route_query(query_embedding, agent_registry);
SELECT ruvector_adaptive_route(query, context, 0.01);

-- Self-learning (ReasoningBank)
SELECT ruvector_record_trajectory(input, output, success, context);
SELECT ruvector_adaptive_search(query, context, ef_search);
SELECT ruvector_learning_feedback(search_id, relevance_scores);

-- Graph queries with Cypher
SELECT ruvector_cypher_query('
  MATCH (u:User)-[:WATCHED]->(m:Movie)-[:STARS]->(a:Actor)
  RETURN a.name, count(m) as movies
  ORDER BY movies DESC LIMIT 10
');
```

---

## 📁 Project Structure

```
hackathon-tv5/
├── benchmarks/
│   ├── ruvector_benchmark_optimized.sql   # 10K movies, GNN, hyperbolic
│   ├── run_benchmarks_optimized.sql       # Performance tests
│   ├── tv5_raft_scale_benchmark.sql       # 80K scale + Raft consensus
│   └── run_tv5_benchmarks.sql             # Cross-platform tests
├── apps/
│   └── media-discovery/                   # Next.js demo app
└── README.md
```

---

## 🔗 Links

- **RuVector:** [github.com/ruvnet/ruvector](https://github.com/ruvnet/ruvector)
- **Docker Hub:** [ruvnet/ruvector-postgres](https://hub.docker.com/r/ruvnet/ruvector-postgres)
- **npm:** [npmjs.com/package/ruvector](https://www.npmjs.com/package/ruvector)
- **PostgreSQL CLI:** [@ruvector/postgres-cli](https://www.npmjs.com/package/@ruvector/postgres-cli)
- **Hackathon:** [agentics.org/hackathon](https://agentics.org/hackathon)
- **Discord:** [discord.agentics.org](https://discord.agentics.org)

---

<div align="center">

## 🏆 Agentics Foundation TV5 Hackathon

**AI Media Gateway** — Self-learning recommendations powered by RuVector

*61µs search latency • 39 attention mechanisms • Self-improving GNN • Cross-platform Raft consensus*

**Traditional vector DBs just store and search. RuVector learns.**

[Join the Hackathon](https://agentics.org/hackathon) • [Discord](https://discord.agentics.org) • [RuVector Docs](https://github.com/ruvnet/ruvector)

</div>
