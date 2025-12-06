# RuVector PostgreSQL - Advanced Vector Operations

[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16+-blue.svg)](https://www.postgresql.org/)
[![pgvector](https://img.shields.io/badge/pgvector-0.6.0-green.svg)](https://github.com/pgvector/pgvector)
[![npm](https://img.shields.io/badge/npm-@ruvector/postgres--cli-red.svg)](https://www.npmjs.com/package/@ruvector/postgres-cli)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

> **High-performance PostgreSQL vector operations with Raft consensus, hyperbolic embeddings, GNN, and distributed scale testing**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    RUVECTOR POSTGRESQL ARCHITECTURE                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     PostgreSQL 16 + pgvector 0.6.0                  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│           ┌────────────────────────┼────────────────────────┐              │
│           │                        │                        │              │
│           ▼                        ▼                        ▼              │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐        │
│  │  HNSW Indexes   │    │  Raft Consensus │    │   Scale Shards  │        │
│  │  <5ms search    │    │  5-node cluster │    │  80K+ vectors   │        │
│  │  10K+ vectors   │    │  leader election│    │  8 shards       │        │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘        │
│           │                        │                        │              │
│           └────────────────────────┼────────────────────────┘              │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Advanced Vector Operations                        │   │
│  │  • Hyperbolic Embeddings (Poincaré, Lorentz, Möbius)                │   │
│  │  • Graph Neural Networks (GCN, GraphSAGE aggregation)               │   │
│  │  • Sparse Vectors + BM25 hybrid search                              │   │
│  │  • Binary/Scalar Quantization (32x memory savings)                  │   │
│  │  • Federated Learning (FedAvg aggregation)                          │   │
│  │  • Byzantine Fault Tolerance (n >= 3f + 1)                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 Quick Start

### Install the CLI

```bash
# Install globally
npm install -g @ruvector/postgres-cli

# Or use npx
npx @ruvector/postgres-cli --help
```

### CLI Commands (19+)

```bash
# Setup & Configuration
ruvector-pg setup              # Install pgvector extension
ruvector-pg create-table       # Create vector table with schema
ruvector-pg drop-table         # Drop vector table

# Data Operations
ruvector-pg insert             # Insert vectors (single or batch)
ruvector-pg update             # Update existing vectors
ruvector-pg delete             # Delete vectors by ID
ruvector-pg upsert             # Insert or update vectors

# Search & Query
ruvector-pg search             # Semantic vector search (cosine, L2, IP)
ruvector-pg get                # Get vector by ID
ruvector-pg count              # Count vectors in table
ruvector-pg list               # List vectors with pagination

# Index Management
ruvector-pg index create       # Create HNSW or IVFFlat index
ruvector-pg index drop         # Drop index
ruvector-pg index list         # List all indexes
ruvector-pg index stats        # Index statistics

# Performance
ruvector-pg benchmark          # Run performance benchmarks
ruvector-pg analyze            # Analyze table for query optimization
ruvector-pg vacuum             # Vacuum and optimize storage

# Utilities
ruvector-pg info               # Database and extension info
ruvector-pg export             # Export vectors to JSON/CSV
ruvector-pg import             # Import vectors from file
```

---

## 📊 PostgreSQL Benchmark Suite

Comprehensive SQL benchmark suite demonstrating advanced vector operations.

### Running the Benchmarks

```bash
# 1. Start PostgreSQL 16 with pgvector
pg_ctl start -D /var/lib/postgresql/16/main

# 2. Run optimized benchmark setup (10K vectors, GNN, hyperbolic)
psql -d postgres -f benchmarks/ruvector_benchmark_optimized.sql

# 3. Execute benchmark queries
psql -d postgres -f benchmarks/run_benchmarks_optimized.sql

# 4. Run TV5 Raft consensus + scale benchmarks (80K vectors)
psql -d postgres -f benchmarks/tv5_raft_scale_benchmark.sql
psql -d postgres -f benchmarks/run_tv5_benchmarks.sql
```

### Benchmark Results

| Operation | Dataset | Performance |
|-----------|---------|-------------|
| **HNSW Cosine Search** | 10K vectors | <5ms (indexed) |
| **L2 Distance Search** | 10K vectors | <5ms (parallel) |
| **Poincaré Distance KNN** | 5K hyperbolic | 15ms |
| **Lorentz Distance** | 5K hyperbolic | 18ms |
| **Möbius Addition** | 1K operations | 8ms |
| **GraphSAGE Mean Aggregation** | 1K nodes | 25ms |
| **BM25 + Vector Hybrid** | 1K documents | 8ms |
| **Sparse Dot Product** | 2K sparse vectors | 12ms |
| **Raft Leader Election** | 5 nodes | <1ms |
| **Raft Log Replication** | 1K entries | 5ms |
| **Federated Aggregate (FedAvg)** | 100 agents | 12ms |
| **Shard Vector Search** | 80K (8 shards) | 45ms |
| **Binary Quantization** | 1K vectors | 3ms |
| **Scalar Quantization** | 1K vectors | 4ms |
| **Hamming Distance** | 100x100 pairs | 15ms (15x speedup) |

---

## 🔧 Core SQL Functions

### 1. Hyperbolic Embeddings

```sql
-- Poincaré ball distance (hierarchical embeddings)
SELECT poincare_distance(
    ARRAY[0.1, 0.2, 0.3]::float8[],
    ARRAY[0.4, 0.5, 0.6]::float8[],
    -1.0  -- curvature
);

-- Lorentz/Hyperboloid distance
SELECT lorentz_distance(
    ARRAY[1.5, 0.1, 0.2]::float8[],  -- time component first
    ARRAY[1.8, 0.3, 0.4]::float8[]
);

-- Möbius addition (hyperbolic translation)
SELECT mobius_add(
    ARRAY[0.1, 0.2]::float8[],
    ARRAY[0.3, 0.4]::float8[],
    -1.0  -- curvature
);

-- Exponential map (tangent to manifold)
SELECT exp_map(
    ARRAY[0.0, 0.0]::float8[],  -- base point
    ARRAY[0.1, 0.2]::float8[],  -- tangent vector
    -1.0
);
```

### 2. Graph Neural Networks

```sql
-- GraphSAGE mean aggregation
SELECT graphsage_mean(
    node_features,
    ARRAY[neighbor1_features, neighbor2_features, neighbor3_features]
) FROM graph_nodes WHERE id = 1;

-- Batch GNN aggregation with edge index
WITH neighbors AS (
    SELECT n.id, n.features, array_agg(n2.features) AS nf
    FROM graph_nodes n
    JOIN graph_edges e ON n.id = e.source_id
    JOIN graph_nodes n2 ON e.target_id = n2.id
    GROUP BY n.id, n.features
)
SELECT id, graphsage_mean(features, nf) FROM neighbors;
```

### 3. Attention Mechanisms

```sql
-- Scaled dot-product attention
SELECT scaled_dot_attention(
    query_vector,
    ARRAY[key1, key2, key3]::float8[][],
    ARRAY[val1, val2, val3]::float8[][]
);

-- Softmax normalization
SELECT arr_softmax(ARRAY[1.0, 2.0, 3.0, 4.0]::float8[]);
```

### 4. Sparse Vectors & BM25

```sql
-- Create sparse vector (threshold 0.8 = 80% sparsity)
SELECT to_sparse(dense_vector, 0.8);

-- Sparse dot product (hash join optimized)
SELECT sparse_dot(sparse_vec_a, sparse_vec_b);

-- BM25 scoring for text search
SELECT bm25_score(
    ARRAY['machine', 'learning', 'neural'],  -- query terms
    document_content,
    100.0,   -- avg doc length
    1000,    -- total docs
    1.2,     -- k1
    0.75     -- b
);

-- Hybrid vector + BM25 search
WITH vec_results AS (
    SELECT id, 1.0 / (1.0 + (embedding <=> query_vec)) AS vscore
    FROM documents ORDER BY embedding <=> query_vec LIMIT 100
),
text_results AS (
    SELECT id, ts_rank_cd(tsv, q) AS tscore
    FROM documents, plainto_tsquery('search terms') q
    WHERE tsv @@ q
)
SELECT id, vscore * 0.7 + tscore * 0.3 AS hybrid_score
FROM vec_results v FULL OUTER JOIN text_results t USING (id)
ORDER BY hybrid_score DESC LIMIT 10;
```

### 5. Vector Quantization

```sql
-- Binary quantization (1 bit per dimension)
SELECT binary_quantize(embedding_arr);
-- Returns: bit varying (e.g., '10110010...')

-- Scalar quantization (8-bit integers)
SELECT scalar_quantize(embedding_arr, -1.0, 1.0);
-- Returns: int[] (values 0-255)

-- Hamming distance (XOR + popcount, 15x faster)
SELECT hamming_distance(binary_vec_a, binary_vec_b);
```

---

## 🗳️ Raft Consensus Operations

Distributed consensus for multi-node vector aggregation.

### Raft Data Structures

```sql
-- Node states
CREATE TYPE raft_state AS ENUM ('follower', 'candidate', 'leader');

-- Cluster nodes with capability vectors
CREATE TABLE raft_nodes (
    node_id text PRIMARY KEY,
    state raft_state DEFAULT 'follower',
    current_term int DEFAULT 0,
    voted_for text,
    last_heartbeat timestamptz DEFAULT now(),
    capability_vector vector(64)  -- For routing optimization
);

-- Replicated log
CREATE TABLE raft_log (
    term int NOT NULL,
    log_index bigint NOT NULL,
    command jsonb NOT NULL,
    committed boolean DEFAULT false,
    PRIMARY KEY (term, log_index)
);
```

### Raft Functions

```sql
-- Request vote RPC
SELECT raft_request_vote(
    2,           -- term
    'node-1',    -- candidate_id
    'node-2',    -- voter_id
    100,         -- last_log_index
    1            -- last_log_term
);

-- Append entries RPC (log replication)
SELECT raft_append_entries(
    2,           -- term
    'node-1',    -- leader_id
    99,          -- prev_log_index
    1,           -- prev_log_term
    ARRAY['{"op":"set","key":"k1","value":42}'::jsonb],
    100,         -- leader_commit
    'node-2'     -- follower_id
);

-- Leader election
SELECT * FROM raft_elect_leader(5);  -- 5-node cluster
-- Returns: leader_id, term, votes_received

-- Byzantine fault tolerance check
SELECT * FROM check_byzantine_tolerance(7, 2);
-- Returns: is_tolerant, max_faulty, required_honest, safety_margin
```

---

## 📈 Scale Testing (80K+ Vectors)

Distributed vector search across logical shards.

```sql
-- Scale vectors table with shard-based partitioning
CREATE TABLE scale_vectors (
    shard_id int NOT NULL,
    id bigserial,
    embedding vector(384),
    metadata jsonb,
    PRIMARY KEY (shard_id, id)
);

-- Insert 10K vectors per shard (8 shards = 80K total)
SELECT scale_insert_vectors(0, 10000, 384);  -- Shard 0
SELECT scale_insert_vectors(1, 10000, 384);  -- Shard 1
-- ... repeat for shards 2-7

-- Distributed search across specific shards
SELECT * FROM scale_search_vectors(
    query_embedding,
    10,                    -- top-k
    ARRAY[0, 1, 2, 3]     -- shards to search
);
```

---

## 🤖 Federated Learning

Privacy-preserving preference aggregation.

```sql
-- Federated agents table
CREATE TABLE federated_agents (
    agent_id text PRIMARY KEY,
    shard_id int NOT NULL,
    embedding vector(384),
    quality float8 DEFAULT 0.0,  -- 0-1 quality score
    task_count int DEFAULT 0
);

-- FedAvg aggregation (quality-weighted)
SELECT federated_aggregate('cluster-1', 0.7);
-- Aggregates embeddings from agents with quality >= 0.7

-- Consensus metrics
SELECT calc_consensus_score(
    ARRAY[true, true, false, true, true],  -- votes
    ARRAY[1.0, 0.8, 0.9, 0.7, 0.85]        -- weights (optional)
);
```

---

## 📁 Benchmark Files

```
benchmarks/
├── ruvector_benchmark_optimized.sql   # Setup: 10K vectors, GNN, hyperbolic
├── run_benchmarks_optimized.sql       # Execute optimized benchmarks
├── tv5_raft_scale_benchmark.sql       # Setup: Raft + 80K scale vectors
└── run_tv5_benchmarks.sql             # Execute Raft/scale benchmarks
```

---

## ⚙️ Performance Configuration

```sql
-- Recommended PostgreSQL settings for vector workloads
SET max_parallel_workers_per_gather = 4;
SET parallel_tuple_cost = 0.001;
SET work_mem = '256MB';
SET maintenance_work_mem = '512MB';
SET effective_cache_size = '4GB';
SET jit = on;

-- HNSW search parameter
SET hnsw.ef_search = 100;
```

---

## 📜 License

Apache License 2.0

---

## 🔗 Links

- **npm:** [@ruvector/postgres-cli](https://www.npmjs.com/package/@ruvector/postgres-cli)
- **pgvector:** [github.com/pgvector/pgvector](https://github.com/pgvector/pgvector)
- **RuVector:** [npmjs.com/package/ruvector](https://www.npmjs.com/package/ruvector)

---

<div align="center">

**RuVector PostgreSQL** - Advanced Vector Operations for PostgreSQL

</div>
