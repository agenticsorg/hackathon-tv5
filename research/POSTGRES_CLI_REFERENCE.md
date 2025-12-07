<!--
Source: npm registry + https://github.com/ruvnet/ruvector
Fetched: 2025-12-07T00:00:00.000Z
Topic: @ruvector/postgres-cli - PostgreSQL Vector Database CLI
Package Version: 0.2.5
-->

# @ruvector/postgres-cli Reference

## Overview

**The most advanced AI vector database CLI for PostgreSQL** - A drop-in pgvector replacement with 53+ SQL functions, 39 attention mechanisms, GNN layers, hyperbolic embeddings, and self-learning capabilities.

### Package Information

- **Package**: `@ruvector/postgres-cli`
- **Version**: 0.2.5 (Latest)
- **License**: MIT
- **Repository**: https://github.com/ruvnet/ruvector
- **Directory**: `npm/packages/postgres-cli`
- **Author**: ruv.io Team <info@ruv.io>
- **Node.js**: >=18.0.0
- **PostgreSQL**: 14, 15, 16, 17

### CLI Executables

Two command aliases are available:

```bash
ruvector-pg    # Full command name
rvpg           # Short alias
```

---

## 1. Core Features

### Vector Database Capabilities

| Feature | pgvector | RuVector |
|---------|----------|----------|
| **Vector Search** | HNSW, IVFFlat | HNSW, IVFFlat |
| **Distance Metrics** | 3 | 8+ (including hyperbolic) |
| **Attention Mechanisms** | - | 39 types |
| **Graph Neural Networks** | - | GCN, GraphSAGE, GAT |
| **Hyperbolic Embeddings** | - | Poincare, Lorentz |
| **Sparse Vectors / BM25** | - | Full support |
| **Self-Learning** | - | ReasoningBank |
| **Agent Routing** | - | Tiny Dancer |

### Key Features

1. **53+ SQL Functions** - Comprehensive vector operations via pgrx
2. **39 Attention Mechanisms** - Scaled dot-product, multi-head, flash attention, etc.
3. **Graph Neural Networks** - GCN, GraphSAGE, GAT layers
4. **Hyperbolic Geometry** - Poincare and Lorentz embeddings for hierarchical data
5. **Sparse Vectors** - BM25, TF-IDF, SPLADE for hybrid search
6. **Self-Learning** - ReasoningBank for adaptive optimization
7. **Agent Routing** - Tiny Dancer for intelligent agent selection
8. **Quantization** - Binary, scalar, and product quantization

---

## 2. Installation

### Quick Start (One Command)

```bash
# Auto-detect and install everything
npx @ruvector/postgres-cli install

# Native installation (no Docker)
npx @ruvector/postgres-cli install --method native
```

The installer automatically:
1. Detects your system (Linux/macOS)
2. Installs PostgreSQL (if not present)
3. Installs Rust (if not present, for native builds)
4. Builds and installs RuVector extension
5. Creates a ready-to-use database

### Platform Support

| Platform | Architecture | Docker | Native | Package Manager |
|----------|-------------|--------|--------|-----------------|
| **Ubuntu/Debian** | x64, arm64 | ✅ | ✅ | apt |
| **RHEL/CentOS/Fedora** | x64, arm64 | ✅ | ✅ | dnf/yum |
| **Arch Linux** | x64 | ✅ | ✅ | pacman |
| **macOS** | Intel, Apple Silicon | ✅ | ✅ | Homebrew |
| **Windows** | x64 | ✅ (WSL2) | ❌ | - |

### Installation Methods

#### Option 1: Docker (Recommended - Fastest)

```bash
# Auto-detect and install via Docker
npx @ruvector/postgres-cli install

# Explicitly use Docker
npx @ruvector/postgres-cli install --method docker

# Customize configuration
npx @ruvector/postgres-cli install \
  --port 5433 \
  --user myuser \
  --password mypass \
  --database mydb \
  --data-dir /path/to/data \
  --name my-ruvector
```

#### Option 2: Native (No Docker Required)

```bash
# Full native installation
npx @ruvector/postgres-cli install --method native

# Specify PostgreSQL version
npx @ruvector/postgres-cli install --method native --pg-version 17

# Use existing PostgreSQL
npx @ruvector/postgres-cli install --method native --skip-postgres

# Use existing Rust
npx @ruvector/postgres-cli install --method native --skip-rust
```

#### Option 3: Global CLI

```bash
# Install globally
npm install -g @ruvector/postgres-cli

# Use anywhere
ruvector-pg install
ruvector-pg info
ruvector-pg vector create embeddings --dim 384
```

### Install Command Options

```bash
npx @ruvector/postgres-cli install [options]

Options:
  -m, --method <type>     Installation method: docker, native, auto (default: "auto")
  -p, --port <number>     PostgreSQL port (default: "5432")
  -u, --user <name>       Database user (default: "ruvector")
  --password <pass>       Database password (default: "ruvector")
  -d, --database <name>   Database name (default: "ruvector")
  --data-dir <path>       Persistent data directory (Docker only)
  --name <name>           Container name (default: "ruvector-postgres")
  --version <version>     RuVector version (default: "0.2.5")
  --pg-version <version>  PostgreSQL version: 14, 15, 16, 17 (default: "16")
  --skip-postgres         Skip PostgreSQL installation (use existing)
  --skip-rust             Skip Rust installation (use existing)
```

---

## 3. CLI Commands

### Server Management

```bash
# Check installation status
npx @ruvector/postgres-cli status

# Show version and configuration
npx @ruvector/postgres-cli info

# Start the server
npx @ruvector/postgres-cli start

# Stop the server
npx @ruvector/postgres-cli stop

# View logs
npx @ruvector/postgres-cli logs
npx @ruvector/postgres-cli logs --follow

# Connect with psql
npx @ruvector/postgres-cli psql
npx @ruvector/postgres-cli psql "SELECT ruvector_version();"

# Uninstall
npx @ruvector/postgres-cli uninstall
```

### Vector Operations

```bash
# Create vector table with index
ruvector-pg vector create <table> --dim <n> --index <hnsw|ivfflat>

# Examples:
ruvector-pg vector create documents --dim 384 --index hnsw
ruvector-pg vector create embeddings --dim 1536 --index ivfflat

# Insert vectors from JSON file
ruvector-pg vector insert <table> --file data.json

# JSON format:
# [
#   {"vector": [0.1, 0.2, 0.3], "metadata": {"title": "Doc 1"}},
#   {"vector": [0.4, 0.5, 0.6], "metadata": {"title": "Doc 2"}}
# ]

# Search for similar vectors
ruvector-pg vector search <table> \
  --query "[0.15, 0.25, 0.35]" \
  --top-k 10 \
  --metric cosine

# Distance metrics: cosine, l2, ip (inner product)

# Calculate distance between two vectors
ruvector-pg vector distance \
  --a "[0.1, 0.2, 0.3]" \
  --b "[0.4, 0.5, 0.6]" \
  --metric cosine

# Normalize a vector
ruvector-pg vector normalize --vector "[0.5, 0.3, 0.2]"
```

### Attention Mechanisms (39 Types)

```bash
# Compute attention
ruvector-pg attention compute \
  --query "[0.1, 0.2, ...]" \
  --keys "[[0.1, 0.2], [0.3, 0.4], ...]" \
  --values "[[0.5, 0.6], [0.7, 0.8], ...]" \
  --type scaled_dot

# List all available attention types
ruvector-pg attention list-types

# Available types include:
# - scaled_dot (Scaled Dot-Product Attention)
# - multi_head (Multi-Head Attention)
# - flash (Flash Attention)
# - sparse (Sparse Attention)
# - linear (Linear Attention)
# - And 34 more...
```

### Graph Neural Networks

```bash
# Create GNN layer
ruvector-pg gnn create <name> \
  --type gcn \
  --input-dim 384 \
  --output-dim 128

# Supported types: gcn, graphsage, gat

# Forward pass through GNN layer
ruvector-pg gnn forward <layer> \
  --features features.json \
  --edges edges.json

# Features JSON format:
# [[0.1, 0.2, ...], [0.3, 0.4, ...], ...]

# Edges JSON format:
# [[0, 1], [1, 2], [2, 3], ...]
```

### Hyperbolic Geometry

Perfect for hierarchical data like taxonomies, organizational charts, and knowledge graphs:

```bash
# Poincare distance (hyperbolic space)
ruvector-pg hyperbolic poincare-distance \
  --a "[0.1, 0.2]" \
  --b "[0.3, 0.4]" \
  --curvature -1.0

# Lorentz distance
ruvector-pg hyperbolic lorentz-distance \
  --a "[0.1, 0.2, 0.3]" \
  --b "[0.4, 0.5, 0.6]"

# Mobius addition (hyperbolic space)
ruvector-pg hyperbolic mobius-add \
  --a "[0.1, 0.2]" \
  --b "[0.3, 0.4]"

# Exponential map
ruvector-pg hyperbolic exp-map \
  --base "[0.0, 0.0]" \
  --tangent "[0.1, 0.2]"

# Convert between coordinate systems
ruvector-pg hyperbolic poincare-to-lorentz --vector "[0.1, 0.2]"
```

### Sparse Vectors & BM25

For hybrid search combining semantic and keyword-based retrieval:

```bash
# Create sparse vector
ruvector-pg sparse create \
  --indices "[0, 5, 10, 15]" \
  --values "[0.5, 0.3, 0.2, 0.1]" \
  --dim 10000

# Compute BM25 score
ruvector-pg sparse bm25 \
  --query '{"indices": [1, 5, 10], "values": [0.8, 0.5, 0.3]}' \
  --doc '{"indices": [1, 5], "values": [2, 1]}' \
  --doc-len 150 \
  --avg-doc-len 200

# Calculate sparse vector distance
ruvector-pg sparse distance \
  --a '{"indices": [0, 5], "values": [0.5, 0.3]}' \
  --b '{"indices": [5, 10], "values": [0.3, 0.2]}' \
  --metric cosine
```

### Graph Database

```bash
# Create a graph
ruvector-pg graph create <graph-name>

# Add nodes
ruvector-pg graph create-node <graph-name> \
  --labels "Category,Topic" \
  --properties '{"name": "Science", "level": 1}'

# Add edges
ruvector-pg graph create-edge <graph-name> \
  --from 1 \
  --to 2 \
  --type "SUBCATEGORY" \
  --properties '{"weight": 0.9}'

# Query with Cypher
ruvector-pg graph query <graph-name> \
  "MATCH (n:Category)-[:SUBCATEGORY]->(m) RETURN n, m"

# Graph traversal
ruvector-pg graph traverse <graph-name> \
  --start-node 1 \
  --max-depth 3
```

### Agent Routing (Tiny Dancer)

Intelligent agent selection based on capabilities, cost, latency, and quality:

```bash
# Register an agent
ruvector-pg routing register \
  --name "agent1" \
  --type llm \
  --capabilities "summarization,translation" \
  --cost 0.01 \
  --latency 100 \
  --quality 0.9

# Route a request to best agent
ruvector-pg routing route \
  --embedding "[0.1, 0.2, 0.3, ...]" \
  --optimize-for balanced

# Optimization strategies: cost, latency, quality, balanced

# List registered agents
ruvector-pg routing list

# Get agent statistics
ruvector-pg routing stats --name "agent1"
```

### Self-Learning (ReasoningBank)

Enable the system to learn from user feedback and optimize over time:

```bash
# Enable learning for a table
ruvector-pg learning enable <table> \
  --max-trajectories 1000 \
  --num-clusters 10

# Record search trajectory
ruvector-pg learning record \
  --input "[0.1, 0.2, ...]" \
  --output "[0.3, 0.4, ...]" \
  --success true \
  --reward 0.95

# Get optimized search parameters
ruvector-pg learning get-params <table> \
  --query "[0.15, 0.25, ...]"

# View learning statistics
ruvector-pg learning stats <table>

# Cluster analysis
ruvector-pg learning clusters <table>
```

### Quantization

Reduce memory and improve performance:

```bash
# Binary quantization (1-bit)
ruvector-pg quantization binary --vector "[0.1, 0.2, -0.3, 0.4]"

# Scalar quantization (8-bit)
ruvector-pg quantization scalar --vector "[0.1, 0.2, 0.3, 0.4]"

# Product quantization
ruvector-pg quantization product \
  --vector "[0.1, 0.2, 0.3, 0.4]" \
  --num-subvectors 2

# Compare quantization methods
ruvector-pg quantization compare "[0.1, 0.2, 0.3, 0.4]"
```

### Benchmarking

```bash
# Run all benchmarks
ruvector-pg bench run \
  --type all \
  --size 10000 \
  --dim 384

# Specific benchmark types
ruvector-pg bench run --type vector --size 100000
ruvector-pg bench run --type graph --size 50000
ruvector-pg bench run --type attention --size 10000

# View benchmark report
ruvector-pg bench report --format table
ruvector-pg bench report --format json
```

---

## 4. Vector Operations Deep Dive

### Creating Vector Tables

```bash
# Basic table with HNSW index
ruvector-pg vector create embeddings --dim 384 --index hnsw

# Generated SQL (approximate):
# CREATE TABLE embeddings (
#   id SERIAL PRIMARY KEY,
#   vector vector(384),
#   metadata JSONB
# );
# CREATE INDEX ON embeddings USING hnsw (vector vector_cosine_ops);
```

### Index Types

1. **HNSW (Hierarchical Navigable Small World)**
   - Best for: High-dimensional data, fast approximate search
   - Trade-off: More memory, slower build time
   - Use when: Query speed is critical

2. **IVFFlat (Inverted File with Flat Compression)**
   - Best for: Large datasets, less memory usage
   - Trade-off: Slower queries than HNSW
   - Use when: Memory is limited

### Distance Metrics

```bash
# Cosine similarity (normalized dot product)
ruvector-pg vector search docs --query "[...]" --metric cosine
# Range: [-1, 1], higher is more similar

# L2 distance (Euclidean)
ruvector-pg vector search docs --query "[...]" --metric l2
# Range: [0, ∞], lower is more similar

# Inner product
ruvector-pg vector search docs --query "[...]" --metric ip
# Range: (-∞, ∞), higher is more similar
```

### Inserting Vectors

JSON file format for batch insert:

```json
[
  {
    "vector": [0.1, 0.2, 0.3, ...],
    "metadata": {
      "title": "Document Title",
      "author": "Author Name",
      "tags": ["tag1", "tag2"],
      "timestamp": "2025-12-07T00:00:00Z"
    }
  },
  {
    "vector": [0.4, 0.5, 0.6, ...],
    "metadata": {
      "title": "Another Document"
    }
  }
]
```

### Searching Vectors

```bash
# Basic search
ruvector-pg vector search documents \
  --query "[0.15, 0.25, 0.35, ...]" \
  --top-k 5 \
  --metric cosine

# Returns JSON:
# [
#   {
#     "id": 1,
#     "distance": 0.95,
#     "metadata": {"title": "Most Similar Doc"}
#   },
#   {
#     "id": 2,
#     "distance": 0.89,
#     "metadata": {"title": "Second Match"}
#   }
# ]
```

---

## 5. Schema Management

### Automatic Schema Creation

The CLI automatically handles schema creation when you create vector tables:

```bash
# This command creates:
# 1. Table with vector column
# 2. Appropriate index (HNSW or IVFFlat)
# 3. Metadata JSONB column
# 4. Serial primary key
ruvector-pg vector create <table> --dim <n> --index <type>
```

### Extension Installation

The PostgreSQL extension provides 53+ SQL functions:

```sql
-- Check extension version
SELECT ruvector_version();

-- Vector operations
SELECT vector_cosine_distance('[0.1, 0.2]', '[0.3, 0.4]');
SELECT vector_l2_distance('[0.1, 0.2]', '[0.3, 0.4]');
SELECT vector_normalize('[0.5, 0.3, 0.2]');

-- Attention mechanisms
SELECT scaled_dot_attention(query, keys, values);

-- Hyperbolic operations
SELECT poincare_distance('[0.1, 0.2]', '[0.3, 0.4]', -1.0);
SELECT lorentz_distance('[0.1, 0.2, 0.3]', '[0.4, 0.5, 0.6]');

-- Sparse vectors
SELECT sparse_bm25_score(query_sparse, doc_sparse, doc_len, avg_doc_len);

-- And 40+ more functions...
```

### Database Management

```bash
# The CLI manages:
# - Extension creation (CREATE EXTENSION ruvector)
# - Table schemas
# - Index creation and optimization
# - Connection pooling
# - Query optimization
```

---

## 6. Integration

### Environment Variables

```bash
# Connection configuration
export DATABASE_URL="postgresql://localhost:5432/ruvector"
export RUVECTOR_POOL_SIZE=10
export RUVECTOR_TIMEOUT=30000
export RUVECTOR_RETRIES=3

# Then use CLI
ruvector-pg vector search documents --query "[...]"
```

### Connection String Format

```
postgresql://[user[:password]@][host][:port][/database][?param=value]

# Examples:
postgresql://localhost:5432/ruvector
postgresql://user:pass@localhost:5432/mydb
postgresql://user:pass@example.com:5433/vectors?sslmode=require
```

### Programmatic Usage (Node.js)

While the CLI is the primary interface, you can also use the underlying library:

```javascript
// Install dependencies
// npm install @ruvector/postgres-cli pg

import { Client } from 'pg';

const client = new Client({
  connectionString: 'postgresql://localhost:5432/ruvector'
});

await client.connect();

// The extension provides SQL functions
const result = await client.query(`
  SELECT id, metadata,
    vector_cosine_distance(vector, $1::vector) as distance
  FROM embeddings
  ORDER BY distance
  LIMIT 10
`, [[0.1, 0.2, 0.3, /* ... */]]);

console.log(result.rows);
```

### Integration with Other Tools

#### 1. LangChain Integration

```javascript
import { RuvectorStore } from '@ruvector/langchain';

const vectorStore = new RuvectorStore({
  connectionString: 'postgresql://localhost:5432/ruvector',
  tableName: 'documents',
  dimension: 384
});

await vectorStore.addDocuments(docs);
const results = await vectorStore.similaritySearch(query, 10);
```

#### 2. Python Integration (via psycopg2)

```python
import psycopg2
import numpy as np

conn = psycopg2.connect("postgresql://localhost:5432/ruvector")
cur = conn.cursor()

# Use RuVector functions
cur.execute("""
    SELECT id, metadata,
        vector_cosine_distance(vector, %s) as distance
    FROM embeddings
    ORDER BY distance
    LIMIT 10
""", (np.array([0.1, 0.2, 0.3]).tolist(),))

results = cur.fetchall()
```

#### 3. REST API Wrapper

You can build a REST API around the CLI:

```javascript
import express from 'express';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);
const app = express();

app.post('/search', async (req, res) => {
  const { query, topK } = req.body;

  const cmd = `ruvector-pg vector search documents \
    --query "${JSON.stringify(query)}" \
    --top-k ${topK}`;

  const { stdout } = await execAsync(cmd);
  res.json(JSON.parse(stdout));
});

app.listen(3000);
```

---

## 7. Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    @ruvector/postgres-cli                          │
├─────────────────────────────────────────────────────────────────────┤
│  Installation Layer                                                 │
│    ├── Docker      - Pull/build image, run container              │
│    └── Native      - Install PG + Rust + pgrx + extension         │
├─────────────────────────────────────────────────────────────────────┤
│  CLI Layer (Commander.js)                                          │
│    ├── install/status/start/stop/logs - Server management         │
│    ├── vector    - CRUD & search operations                        │
│    ├── attention - 39 attention mechanism types                    │
│    ├── gnn       - Graph Neural Network layers                     │
│    ├── graph     - Cypher queries & traversal                      │
│    ├── hyperbolic- Poincare/Lorentz embeddings                     │
│    ├── sparse    - BM25/SPLADE scoring                             │
│    ├── routing   - Tiny Dancer agent routing                       │
│    ├── learning  - ReasoningBank self-learning                     │
│    └── bench     - Performance benchmarking                        │
├─────────────────────────────────────────────────────────────────────┤
│  Client Layer (pg with connection pooling)                         │
│    ├── Connection pooling (max 10, idle timeout 30s)               │
│    ├── Automatic retry (3 attempts, exponential backoff)           │
│    └── SQL injection protection                                    │
├─────────────────────────────────────────────────────────────────────┤
│  PostgreSQL Extension (ruvector-postgres 0.2.5)                    │
│    └── 53+ SQL functions exposed via pgrx                          │
└─────────────────────────────────────────────────────────────────────┘
```

### Dependencies

**Runtime:**
- `commander` (^11.1.0) - CLI framework and argument parsing
- `chalk` (^5.3.0) - Terminal string styling
- `pg` (^8.11.3) - PostgreSQL client for Node.js
- `inquirer` (^9.2.12) - Interactive command-line prompts
- `ora` (^8.0.1) - Elegant terminal spinners
- `cli-table3` (^0.6.3) - Pretty unicode tables

**Development:**
- `typescript` (^5.3.3) - TypeScript compiler
- `@types/node` (^20.10.5) - Node.js type definitions
- `@types/pg` (^8.10.9) - PostgreSQL client types
- `@types/inquirer` (^9.0.7) - Inquirer types

### Build System

```bash
# TypeScript compilation
npm run build        # Compile to dist/

# Development
npm run dev          # Watch mode compilation

# Testing
npm run test         # Run test suite

# Code quality
npm run lint         # ESLint validation
npm run typecheck    # TypeScript type checking

# Cleanup
npm run clean        # Remove build artifacts
```

---

## 8. Performance Benchmarks

Measured on AMD EPYC 7763 (64 cores), 256GB RAM:

| Operation | 10K vectors | 100K vectors | 1M vectors |
|-----------|-------------|--------------|------------|
| **HNSW Build** | 0.8s | 8.2s | 95s |
| **HNSW Search (top-10)** | 0.3ms | 0.5ms | 1.2ms |
| **Cosine Distance** | 0.01ms | 0.01ms | 0.01ms |
| **Poincare Distance** | 0.02ms | 0.02ms | 0.02ms |
| **GCN Forward** | 2.1ms | 18ms | 180ms |
| **BM25 Score** | 0.05ms | 0.08ms | 0.15ms |

*Vector dimensions: 384, GNN dimensions: 128*

### Optimization Tips

1. **Use HNSW for queries** - 4-10x faster than IVFFlat
2. **Pre-normalize vectors** - Saves computation during search
3. **Use binary quantization** - Reduces memory by 32x
4. **Connection pooling** - Set `RUVECTOR_POOL_SIZE=20` for high load
5. **Batch inserts** - Use JSON file insert for 10-100x speedup
6. **Self-learning** - Enable learning mode for 20-40% query improvement

---

## 9. Tutorials

### Tutorial 1: Semantic Search in 5 Minutes

```bash
# Step 1: Install
npx @ruvector/postgres-cli install

# Step 2: Verify
npx @ruvector/postgres-cli info

# Step 3: Create table
npx @ruvector/postgres-cli vector create documents --dim 384 --index hnsw

# Step 4: Prepare data (save as docs.json)
cat > docs.json << 'EOF'
[
  {"vector": [0.1, 0.2, 0.3, ...], "metadata": {"title": "AI Overview"}},
  {"vector": [0.4, 0.5, 0.6, ...], "metadata": {"title": "ML Basics"}},
  {"vector": [0.7, 0.8, 0.9, ...], "metadata": {"title": "Deep Learning"}}
]
EOF

# Step 5: Insert
npx @ruvector/postgres-cli vector insert documents --file docs.json

# Step 6: Search
npx @ruvector/postgres-cli vector search documents \
  --query "[0.15, 0.25, 0.35, ...]" \
  --top-k 5 \
  --metric cosine
```

### Tutorial 2: Hybrid Search (Semantic + Keyword)

```bash
# Create sparse vector for keyword matching
npx @ruvector/postgres-cli sparse create \
  --indices "[0, 5, 10, 42, 100]" \
  --values "[0.5, 0.3, 0.2, 0.1, 0.05]" \
  --dim 10000

# Compute BM25 score (for ranking)
npx @ruvector/postgres-cli sparse bm25 \
  --query '{"indices": [1,5,10], "values": [0.8,0.5,0.3]}' \
  --doc '{"indices": [1,5], "values": [2,1]}' \
  --doc-len 150 \
  --avg-doc-len 200

# Combine with semantic search:
# 1. Get semantic results (vector search)
# 2. Get keyword results (BM25)
# 3. Merge with weighted scoring
```

### Tutorial 3: Knowledge Graph with Hierarchies

```bash
# Create graph
npx @ruvector/postgres-cli graph create taxonomy

# Add root category
npx @ruvector/postgres-cli graph create-node taxonomy \
  --labels "Category" \
  --properties '{"name": "Science", "level": 0}'

# Add subcategories
npx @ruvector/postgres-cli graph create-node taxonomy \
  --labels "Category" \
  --properties '{"name": "Physics", "level": 1}'

npx @ruvector/postgres-cli graph create-node taxonomy \
  --labels "Category" \
  --properties '{"name": "Biology", "level": 1}'

# Add relationships
npx @ruvector/postgres-cli graph create-edge taxonomy \
  --from 1 --to 2 --type "SUBCATEGORY"

npx @ruvector/postgres-cli graph create-edge taxonomy \
  --from 1 --to 3 --type "SUBCATEGORY"

# Use hyperbolic embeddings (better for hierarchies)
npx @ruvector/postgres-cli hyperbolic poincare-distance \
  --a "[0.1, 0.2]" \
  --b "[0.3, 0.4]" \
  --curvature -1.0

# Query graph
npx @ruvector/postgres-cli graph query taxonomy \
  "MATCH (root:Category {level: 0})-[:SUBCATEGORY*]->(child) RETURN root, child"
```

### Tutorial 4: Self-Learning Search System

```bash
# Enable learning on table
npx @ruvector/postgres-cli learning enable documents \
  --max-trajectories 1000 \
  --num-clusters 10

# Simulate user interactions
# When user searches and clicks result:
npx @ruvector/postgres-cli learning record \
  --input "[0.1, 0.2, ...]" \      # Original query
  --output "[0.3, 0.4, ...]" \     # Selected result
  --success true \
  --reward 0.95

# When user searches but doesn't click:
npx @ruvector/postgres-cli learning record \
  --input "[0.1, 0.2, ...]" \
  --output "[0.3, 0.4, ...]" \
  --success false \
  --reward 0.1

# Get optimized parameters for future searches
npx @ruvector/postgres-cli learning get-params documents \
  --query "[0.15, 0.25, ...]"

# View learning statistics
npx @ruvector/postgres-cli learning stats documents
```

### Tutorial 5: Multi-Model Agent Routing

```bash
# Register different LLM agents
npx @ruvector/postgres-cli routing register \
  --name "gpt4-turbo" \
  --type llm \
  --capabilities "reasoning,code,math" \
  --cost 0.03 \
  --latency 500 \
  --quality 0.95

npx @ruvector/postgres-cli routing register \
  --name "claude-3-haiku" \
  --type llm \
  --capabilities "summarization,writing" \
  --cost 0.0008 \
  --latency 200 \
  --quality 0.85

npx @ruvector/postgres-cli routing register \
  --name "llama-70b" \
  --type llm \
  --capabilities "general,chat" \
  --cost 0.0 \
  --latency 1000 \
  --quality 0.75

# Route request based on embedding
npx @ruvector/postgres-cli routing route \
  --embedding "[0.1, 0.2, ...]" \
  --optimize-for cost         # Use cheapest suitable agent

npx @ruvector/postgres-cli routing route \
  --embedding "[0.1, 0.2, ...]" \
  --optimize-for latency      # Use fastest agent

npx @ruvector/postgres-cli routing route \
  --embedding "[0.1, 0.2, ...]" \
  --optimize-for quality      # Use best agent

npx @ruvector/postgres-cli routing route \
  --embedding "[0.1, 0.2, ...]" \
  --optimize-for balanced     # Balance cost/latency/quality
```

---

## 10. Troubleshooting

### Docker Issues

```bash
# Check if Docker is running
docker info

# View container logs
npx @ruvector/postgres-cli logs

# Check container status
docker ps -a | grep ruvector

# Restart container
npx @ruvector/postgres-cli stop
npx @ruvector/postgres-cli start

# Remove and reinstall
npx @ruvector/postgres-cli uninstall
npx @ruvector/postgres-cli install
```

### Native Installation Issues

```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Check PostgreSQL version
psql --version

# Check pgrx installation
cargo pgrx --version

# Reinstall extension only
npx @ruvector/postgres-cli install --method native --skip-postgres --skip-rust

# Check extension is loaded
npx @ruvector/postgres-cli psql "SELECT * FROM pg_extension WHERE extname = 'ruvector';"
```

### Connection Issues

```bash
# Test connection
npx @ruvector/postgres-cli psql "SELECT version();"

# Check connection string
echo $DATABASE_URL

# Test with explicit connection
DATABASE_URL="postgresql://localhost:5432/ruvector" \
  npx @ruvector/postgres-cli status
```

### Permission Issues

```bash
# Docker may need sudo
sudo docker ps

# Native install requires sudo for PostgreSQL
# The installer will prompt for password when needed

# Check file permissions
ls -la ~/.ruvector/
```

### Performance Issues

```bash
# Check index status
npx @ruvector/postgres-cli psql "SELECT * FROM pg_indexes WHERE tablename = 'documents';"

# Rebuild index
npx @ruvector/postgres-cli psql "REINDEX TABLE documents;"

# Vacuum and analyze
npx @ruvector/postgres-cli psql "VACUUM ANALYZE documents;"

# Check table size
npx @ruvector/postgres-cli psql "SELECT pg_size_pretty(pg_total_relation_size('documents'));"
```

### Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| `Connection refused` | PostgreSQL not running | `npx @ruvector/postgres-cli start` |
| `Extension not found` | Extension not installed | Reinstall with `install` command |
| `Dimension mismatch` | Vector dimensions don't match table | Check table schema and vector size |
| `Index build failed` | Insufficient memory | Reduce dataset size or increase memory |
| `Permission denied` | Database permissions | Check user has CREATE/INSERT permissions |

---

## 11. Related Packages

### RuVector Ecosystem

1. **ruvector-postgres** (Rust crate)
   - Core PostgreSQL extension
   - 53+ SQL functions
   - Version: 0.2.5
   - Crates.io: https://crates.io/crates/ruvector-postgres

2. **ruvector-core** (Rust crate)
   - Vector operations library
   - HNSW/IVFFlat implementations
   - Native SIMD acceleration

3. **ruvector** (npm)
   - Main vector database package
   - Automatic native/WASM fallback
   - Node.js API

4. **@ruvector/gnn** (npm)
   - Graph Neural Network bindings
   - Native performance

5. **@ruvector/graph-node** (npm)
   - Graph database with hypergraph support
   - Cypher query language

6. **@ruvector/router** (npm)
   - Tiny Dancer agent routing

7. **ruvector-extensions** (npm)
   - Advanced features: embeddings, UI, exports

---

## 12. Additional Resources

### Documentation

- **GitHub Repository**: https://github.com/ruvnet/ruvector
- **npm Package**: https://www.npmjs.com/package/@ruvector/postgres-cli
- **Issue Tracker**: https://github.com/ruvnet/ruvector/issues

### Community

- **Author**: ruv.io Team <info@ruv.io>
- **Website**: https://ruv.io
- **Funding**: https://github.com/sponsors/ruvnet

### Contributing

Contributions welcome! See CONTRIBUTING.md in the repository.

### License

MIT License - see LICENSE file for details.

---

## Version History

- **0.2.5** (2025-12-06) - Latest release
- **0.2.4** (2025-12-06)
- **0.2.3** (2025-12-06)
- **0.2.2** (2025-12-06)
- **0.2.1** (2025-12-06)
- **0.2.0** (2025-12-03) - Major update
- **0.1.2** (2025-12-03)
- **0.1.1** (2025-12-03)
- **0.1.0** (2025-12-03) - Initial release

Package created: 2025-12-03
Last modified: 2025-12-06

---

**Note**: This is a rapidly evolving package with frequent updates. Check npm for the latest version and features.
