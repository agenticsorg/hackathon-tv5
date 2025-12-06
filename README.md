# Agentics Foundation TV5 Hackathon

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![npm version](https://img.shields.io/badge/npm-agentics--hackathon-red.svg)](https://www.npmjs.com/package/agentics-hackathon)
[![RuVector](https://img.shields.io/badge/RuVector-v0.1.31-green.svg)](https://www.npmjs.com/package/ruvector)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-pgvector-blue.svg)](https://github.com/pgvector/pgvector)
[![Discord](https://img.shields.io/badge/Discord-Agentics-7289da.svg)](https://discord.agentics.org)

> **Solving the 45-Minute Media Discovery Problem with AI-Powered Vector Search**

## 🎬 The Media Gateway Challenge

Every night, millions spend up to **45 minutes deciding what to watch** — billions of hours lost globally. The problem isn't lack of content, but **fragmentation across streaming platforms** and ineffective search algorithms.

Our solution: An **AI Media Gateway** that uses semantic vector search, personalized embeddings, and distributed consensus to deliver instant, personalized recommendations.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        AI MEDIA GATEWAY ARCHITECTURE                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐    ┌──────────────┐    ┌──────────────────────────────┐   │
│  │   User UI   │───▶│  Next.js 15  │───▶│     RuVector Engine          │   │
│  │  (React 19) │    │   API Layer  │    │  ┌────────────────────────┐  │   │
│  └─────────────┘    └──────────────┘    │  │ HNSW Index (100K+ docs)│  │   │
│         │                  │            │  │ <100µs search latency  │  │   │
│         │           ┌──────┴──────┐     │  └────────────────────────┘  │   │
│         │           │             │     │  ┌────────────────────────┐  │   │
│         │    ┌──────▼─────┐ ┌─────▼───┐ │  │ Embedding Cache (5min) │  │   │
│         │    │   TMDB     │ │ OpenAI  │ │  │ Float32Array Storage   │  │   │
│         │    │   API      │ │Embeddings│ │  └────────────────────────┘  │   │
│         │    └────────────┘ └─────────┘ └──────────────────────────────┘   │
│         │                                         │                         │
│         │    ┌────────────────────────────────────▼──────────────────────┐  │
│         │    │              PostgreSQL + pgvector Backend                │  │
│         │    │  ┌──────────────────────────────────────────────────────┐ │  │
│         │    │  │  TV5 Benchmark Suite (Raft Consensus + Scale)        │ │  │
│         │    │  │  • 80K vectors across 8 shards                       │ │  │
│         │    │  │  • Raft leader election & log replication            │ │  │
│         │    │  │  • Federated learning (FedAvg aggregation)           │ │  │
│         │    │  │  • Byzantine fault tolerance (n >= 3f + 1)           │ │  │
│         │    │  └──────────────────────────────────────────────────────┘ │  │
│         │    │  ┌──────────────────────────────────────────────────────┐ │  │
│         │    │  │  Advanced Vector Operations                          │ │  │
│         │    │  │  • Hyperbolic embeddings (Poincaré, Lorentz)         │ │  │
│         │    │  │  • Graph Neural Networks (GCN, GraphSAGE)            │ │  │
│         │    │  │  • Sparse vectors with BM25 scoring                  │ │  │
│         │    │  │  • Binary/Scalar quantization (32x memory savings)   │ │  │
│         │    │  └──────────────────────────────────────────────────────┘ │  │
│         │    └───────────────────────────────────────────────────────────┘  │
│         │                                                                   │
│  ┌──────▼─────────────────────────────────────────────────────────────────┐ │
│  │                    Agent-Ready Web (ARW) Protocol                      │ │
│  │  • 85% token reduction vs HTML scraping                                │ │
│  │  • 10x faster AI agent discovery                                       │ │
│  │  • OAuth-enforced actions for safe transactions                        │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 Key Features for Media Gateway

### 1. **Semantic Vector Search** (RuVector + OpenAI Embeddings)
```typescript
// Natural language to personalized recommendations in <100µs
const results = await semanticSearch("mind-bending sci-fi like Inception", 10);
// Returns: Interstellar, The Matrix, Arrival, Dark...
```

### 2. **Distributed Consensus** (TV5 Raft Implementation)
```sql
-- 5-node Raft cluster for distributed recommendation aggregation
SELECT * FROM raft_elect_leader(5);  -- Leader election
SELECT raft_append_entries(...);      -- Log replication
```

### 3. **Federated Learning** (Cross-Platform Preference Aggregation)
```sql
-- FedAvg algorithm across user preference vectors
SELECT federated_aggregate('cluster-1', 0.7);
-- Aggregates quality-filtered embeddings from multiple sources
```

### 4. **Scale Testing** (80K+ Vectors, 8 Shards)
```sql
-- Parallel vector search across distributed shards
SELECT * FROM scale_search_vectors(query_vec, 10, ARRAY[0,1,2,3]);
```

---

🌐 **Website:** [agentics.org/hackathon](https://agentics.org/hackathon)
💬 **Discord:** [discord.agentics.org](https://discord.agentics.org)
📦 **npm:** `npx agentics-hackathon`

---

## 🚀 Quick Start

```bash
# Initialize your hackathon project
npx agentics-hackathon init

# Browse and install 17+ AI tools
npx agentics-hackathon tools

# Check project status
npx agentics-hackathon status

# Start MCP server for AI assistant integration
npx agentics-hackathon mcp
```

---

## 📊 RuVector PostgreSQL Benchmark Suite

Our hackathon entry includes a comprehensive PostgreSQL benchmark suite demonstrating advanced vector operations for media recommendation systems.

### Running the Benchmarks

```bash
# 1. Start PostgreSQL with pgvector
pg_ctl start -D /var/lib/postgresql/16/main

# 2. Run the optimized benchmark setup (10K vectors, GNN, hyperbolic)
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
| HNSW Cosine Search | 10K vectors | <5ms (indexed) |
| Poincaré Distance KNN | 5K hyperbolic | 15ms |
| GraphSAGE Aggregation | 1K nodes | 25ms |
| BM25 + Vector Hybrid | 1K docs | 8ms |
| Raft Leader Election | 5 nodes | <1ms |
| Federated Aggregate | 100 agents | 12ms |
| Shard Vector Search | 80K (8 shards) | 45ms |
| Binary Quantize | 1K vectors | 3ms |
| Hamming Distance | 100x100 pairs | 15ms (15x speedup) |

### Advanced Features Demonstrated

- **Hyperbolic Embeddings**: Poincaré ball model for hierarchical content (genres → subgenres → titles)
- **Graph Neural Networks**: GCN and GraphSAGE for user-content interaction graphs
- **Sparse Vectors + BM25**: Hybrid semantic + keyword search for robust recommendations
- **Quantization**: 32x memory reduction with binary/scalar quantization
- **Raft Consensus**: Distributed recommendation aggregation across multiple servers
- **Federated Learning**: Privacy-preserving preference learning across platforms

---

## 🏆 Hackathon Tracks

| Track | Description |
|-------|-------------|
| **Entertainment Discovery** | Solve the 45-minute decision problem - help users find what to watch |
| **Multi-Agent Systems** | Build collaborative AI agents with Google ADK and Vertex AI |
| **Agentic Workflows** | Create autonomous workflows with Claude, Gemini, and orchestration |
| **Open Innovation** | Bring your own idea - any agentic AI solution that makes an impact |

---

## ✨ Features

### 🛠 CLI Tool (`npx agentics-hackathon`)

- **`init`** - Interactive project setup with track selection and tool installation
- **`tools`** - Browse and install 17+ AI development tools across 6 categories
- **`status`** - View project configuration and installed tools
- **`info`** - Hackathon information and resources
- **`mcp`** - Start MCP server (stdio or SSE transport)
- **`discord`** - Join the community
- **`help`** - Detailed guides and examples

### 🤖 MCP Server

Full Model Context Protocol implementation with:
- **Tools**: `get_hackathon_info`, `get_tracks`, `get_available_tools`, `get_project_status`, `check_tool_installed`, `get_resources`
- **Resources**: Project configuration, track information
- **Prompts**: `hackathon_starter`, `choose_track`

### 📱 Demo Applications

| App | Description |
|-----|-------------|
| **[Media Discovery](apps/media-discovery/)** | AI-powered movie/TV discovery with ARW implementation |
| **[ARW Chrome Extension](apps/arw-chrome-extension/)** | Browser extension for inspecting ARW compliance |

### 🎬 Media Discovery App - How It Works

The Media Discovery app showcases the AI Media Gateway in action:

```
User Query: "movies like Inception but darker"
                    │
                    ▼
┌──────────────────────────────────────────────────────────┐
│  1. EMBEDDING GENERATION (OpenAI text-embedding-3-small) │
│     Query → 768-dim Float32Array                         │
│     Caching: 5-min TTL, server-side                      │
└──────────────────────────────────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────────────┐
│  2. VECTOR SEARCH (RuVector HNSW Index)                  │
│     • 100K media items indexed                           │
│     • Cosine similarity with threshold 0.3              │
│     • <100µs search latency                              │
└──────────────────────────────────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────────────┐
│  3. HYBRID RANKING (Vector + TMDB Metadata)              │
│     • Vector similarity score (0.85 weight)              │
│     • Genre matching boost                               │
│     • Popularity/rating signals                          │
└──────────────────────────────────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────────────┐
│  4. PERSONALIZED RESULTS                                 │
│     [1] Dark (score: 0.92) - "Semantically similar"      │
│     [2] The Prestige (score: 0.88) - "Genre match"       │
│     [3] Memento (score: 0.85) - "Similar director"       │
└──────────────────────────────────────────────────────────┘
```

**Key API Endpoints:**
- `POST /api/search` - Semantic search with natural language
- `POST /api/recommendations` - Personalized recommendations
- `GET /api/discover` - Genre/trending discovery
- `POST /api/preferences` - User preference learning

### 📐 ARW (Agent-Ready Web) Components

This repository includes reference implementations of the ARW specification:

- **Specification**: [ARW v0.1 Draft](spec/ARW-0.1-draft.md)
- **Schemas**: JSON schemas for validation (`packages/schemas/`)
- **Validators**: Python and Node.js validation tools (`packages/validators/`)
- **Badges**: Compliance level badges (`packages/badges/`)

---

## 📦 Repository Structure

```plaintext
hackathon-tv5/
├── src/                             # Hackathon CLI source
│   ├── cli.ts                      # Main CLI entry point
│   ├── commands/                   # CLI commands (init, tools, status, etc.)
│   ├── mcp/                        # MCP server implementation
│   │   ├── server.ts              # MCP tools, resources, prompts
│   │   ├── stdio.ts               # STDIO transport
│   │   └── sse.ts                 # SSE transport
│   ├── constants.ts               # Tracks, tools, configuration
│   └── utils/                     # Helpers and utilities
│
├── apps/                           # Demo Applications
│   ├── media-discovery/           # AI Media Discovery (Next.js + ARW)
│   │   ├── public/
│   │   │   ├── .well-known/arw-manifest.json  # ARW manifest
│   │   │   └── llms.txt                       # ARW discovery file
│   │   └── src/                   # React components & API routes
│   └── arw-chrome-extension/      # ARW Inspector Chrome Extension
│       ├── manifest.json          # Chrome Manifest V3
│       └── src/                   # Popup, content script, service worker
│
├── packages/                       # Shared Packages
│   ├── @arw/schemas/              # TypeScript ARW schemas with Zod
│   ├── schemas/                   # JSON schemas for ARW validation
│   ├── validators/                # Python & Node.js validators
│   ├── validator/                 # ARW validator CLI tool
│   ├── badges/                    # ARW compliance badges (SVG)
│   ├── cli/                       # Rust ARW CLI (advanced)
│   ├── crawler-sdk/               # TypeScript SDK for ARW crawler service
│   ├── crawler-service/           # High-performance crawler API service
│   ├── nextjs-plugin/             # Next.js plugin for ARW integration
│   └── benchmark/                 # ARW benchmark evaluation
│
├── spec/                           # ARW Specification
│   └── ARW-0.1-draft.md           # Editor's draft specification
│
├── benchmarks/                     # PostgreSQL Benchmark Suites
│   ├── ruvector_benchmark_optimized.sql  # Vector + GNN + Hyperbolic setup
│   ├── run_benchmarks_optimized.sql      # Optimized benchmark execution
│   ├── tv5_raft_scale_benchmark.sql      # Raft consensus + 80K scale
│   └── run_tv5_benchmarks.sql            # TV5 benchmark execution
│
├── docs/                           # Documentation
├── ai_docs/                        # AI-focused documentation
├── scripts/                        # Build and utility scripts
│
├── .claude/                        # Claude Code configuration
│   ├── commands/                  # Slash commands
│   └── agents/                    # Sub-agent definitions
│
├── CLAUDE.md                       # Claude Code guidance
└── README.md                       # This file
```

---

## 🔧 Available Tools (17+)

The CLI provides access to tools across 6 categories:

### AI Assistants
- **Claude Code CLI** - Anthropic's AI-powered coding assistant
- **Gemini CLI** - Google's Gemini model interface

### Orchestration & Agent Frameworks
- **Claude Flow** - #1 agent orchestration platform with 101 MCP tools
- **Agentic Flow** - Production AI orchestration with 66 agents
- **Flow Nexus** - Competitive agentic platform on MCP
- **Google ADK** - Build multi-agent systems with Google's Agent Development Kit

### Cloud Platform
- **Google Cloud CLI** - gcloud SDK for Vertex AI, Cloud Functions
- **Vertex AI SDK** - Google Cloud's unified ML platform

### Databases & Memory
- **RuVector** - Vector database with native SIMD, HNSW indexing, <100µs search
- **@ruvector/postgres-cli** - PostgreSQL vector operations CLI (19+ commands)
- **AgentDB** - 150x faster vector search with RuVector backend

### Synthesis & Advanced Tools
- **Agentic Synth** - Synthesis tools for agentic development
- **Strange Loops** - Consciousness exploration SDK
- **SPARC 2.0** - Autonomous vector coding agent

### Python Frameworks
- **LionPride** - Python agentic AI framework
- **Agentic Framework** - AI agents with natural language
- **OpenAI Agents SDK** - Multi-agent workflows from OpenAI

---

## 🌐 ARW (Agent-Ready Web)

This repository demonstrates the ARW specification through the **Media Discovery** app.

### What is ARW?

ARW provides infrastructure for efficient agent-web interaction:

- **85% token reduction** - Machine views vs HTML scraping
- **10x faster discovery** - Structured manifests vs crawling
- **OAuth-enforced actions** - Safe agent transactions
- **AI-* headers** - Full observability of agent traffic

### ARW in Media Discovery

The media-discovery app implements ARW with:

```json
// /.well-known/arw-manifest.json
{
  "version": "0.1",
  "profile": "ARW-1",
  "site": {
    "name": "AI Media Discovery",
    "description": "Discover movies and TV shows through natural language"
  },
  "actions": [
    {
      "id": "semantic_search",
      "endpoint": "/api/search",
      "method": "POST"
    }
  ]
}
```

See the [ARW Specification](spec/ARW-0.1-draft.md) for full details.

---

## 💻 Development

### Prerequisites

- Node.js 18+
- npm or pnpm

### Build & Run

```bash
# Install dependencies
npm install

# Build the CLI
npm run build

# Run locally
npm start

# Development mode (watch)
npm run dev

# Run linter
npm run lint
```

### MCP Server

```bash
# STDIO transport (for Claude Desktop, etc.)
npm run mcp:stdio

# SSE transport (for web integrations)
npm run mcp:sse
```

### Media Discovery App (AI Media Gateway)

```bash
cd apps/media-discovery
npm install

# Set up environment
cp .env.example .env.local
# Add your TMDB_API_KEY and OPENAI_API_KEY

# Run the app
npm run dev

# Sync media embeddings to RuVector
npm run embed:sync
```

### RuVector PostgreSQL CLI

```bash
# Install globally
npm install -g @ruvector/postgres-cli

# Or use npx
npx @ruvector/postgres-cli --help

# Key commands:
ruvector-pg setup              # Install pgvector extension
ruvector-pg create-table       # Create vector table
ruvector-pg insert             # Insert vectors
ruvector-pg search             # Semantic search
ruvector-pg index create       # Create HNSW index
ruvector-pg benchmark          # Run performance benchmarks
```

---

## 🔌 MCP Integration

Add to your Claude Desktop config (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "agentics-hackathon": {
      "command": "npx",
      "args": ["agentics-hackathon", "mcp"]
    }
  }
}
```

Or use SSE transport:

```bash
npx agentics-hackathon mcp sse --port 3000
```

---

## 🤝 Contributing

We welcome contributions! Areas of focus:

1. **CLI Improvements** - New commands, better UX
2. **Tool Integrations** - Add more AI tools
3. **Demo Apps** - Build showcases for hackathon tracks
4. **ARW Implementation** - Expand specification coverage
5. **Documentation** - Guides and tutorials

### Development Workflow

See [CLAUDE.md](CLAUDE.md) for development guidelines including:
- SPARC methodology for systematic development
- Concurrent execution patterns
- File organization rules

---

## 📜 License

This project is licensed under the [Apache License 2.0](LICENSE).

---

## 🔗 Links

- **🌐 Hackathon Website:** [agentics.org/hackathon](https://agentics.org/hackathon)
- **💬 Discord:** [discord.agentics.org](https://discord.agentics.org)
- **📦 GitHub:** [github.com/agenticsorg/hackathon-tv5](https://github.com/agenticsorg/hackathon-tv5)
- **📖 ARW Spec:** [ARW v0.1 Draft](spec/ARW-0.1-draft.md)

---

<div align="center">

**🚀 Agentics Foundation TV5 Hackathon**

*Building the Future of Agentic AI - Supported by Google Cloud*

[Website](https://agentics.org/hackathon) | [Discord](https://discord.agentics.org) | [GitHub](https://github.com/agenticsorg/hackathon-tv5)

</div>
