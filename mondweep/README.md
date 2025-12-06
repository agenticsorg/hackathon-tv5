# Nexus-UMMID: Cognitive Hypergraph Media Metadata Platform

**Unified Media Metadata Integration &amp; Distribution Platform**

> **Solving the 30-minute decision problem** with agentic AI and hypergraph architecture  
> *Agentics Foundation TV5 Hackathon - Entertainment Discovery Track*

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![GCP](https://img.shields.io/badge/GCP-Exclusive-4285F4?logo=google-cloud)](https://cloud.google.com)
[![Track](https://img.shields.io/badge/Track-Entertainment%20Discovery-orange)](https://agentics.org/hackathon)

---

## 🎯 The Challenge

> *"Every night, millions spend up to **30 minutes deciding what to watch** — billions of hours lost every day. Not from lack of content, but from fragmentation across streaming platforms."*

**The Core Problem:** Metadata integration and discovery across fragmented platforms.

**Our Solution:** A **cognitive hypergraph platform** that transforms metadata from static records into intelligent, semantic discovery infrastructure.

---

## 🚀 What Makes Nexus-UMMID Special

### **1. Hypergraph Architecture**
Unlike traditional databases with binary relationships, we use **n-ary hypergraphs** to model complex media rights:

```
Hyperedge: Distribution_Right {
  Asset: "Inception"
  Territory: "France"
  Platform: "Netflix"
  Window: "2025-01-01 to 2025-06-30"
  Quality: "UHD/4K, Dolby Vision"
  License: "Exclusive SVOD"
}
```

### **2. Agentic Learning with AgentDB**
The platform **learns and improves over time**:
- Discovers optimal metadata enrichment strategies
- Stores successful patterns (32.6M retrievals/sec!)
- Self-improves through reflexion and critique
- Builds institutional knowledge automatically

### **3. ARW (Agent-Ready Web) Compliance**
Built for AI agents from the ground up:
- **85% token reduction** vs HTML scraping
- **10x faster discovery** with structured manifests
- OAuth-enforced actions for safe transactions
- Full observability with AI-* headers

### **4. Production-Ready Platform Connectors**
Real integrations, not just demos:
- Netflix IMF package generation
- Amazon Prime MEC feeds
- FAST platform MRSS feeds
- Automated validation and delivery

---

## 🏆 Competitive Advantages

| Feature | Competitors | Nexus-UMMID | Winner |
|---------|-------------|-------------|--------|
| **Data Model** | Graph (binary edges) | Hypergraph (n-ary edges) | ✅ **Us** |
| **Learning** | Static algorithms | AgentDB adaptive | ✅ **Us** |
| **Platform Integration** | Generic search | Netflix, Amazon, FAST | ✅ **Us** |
| **Production Ready** | Research prototypes | Real connectors | ✅ **Us** |
| **ARW Compliance** | No | Yes (85% token reduction) | ✅ **Us** |
| **Cost** | GPU required | CPU-only CloudRun | ✅ **Us** |

---

## 🎯 Hackathon Track

**Entertainment Discovery** - Solving the 30-minute decision problem through:
- Semantic metadata search (natural language queries)
- AI-powered enrichment (Gemini 2.0 + AgentDB learning)
- Hypergraph rights management (collision detection)
- Platform validation (Netflix, Amazon, FAST)
- Automated delivery workflows (Cloud Workflows)

---

## 🏗️ Production Build Status

### **🔄 PHASE 2 IN PROGRESS**

**Phase 1 Complete:** Foundation & Strategy ✅
- ✅ 13 comprehensive documents (8000+ lines)
- ✅ Master PRD + Enhanced Implementation Plan
- ✅ **13-Agent Production Swarm Strategy** (Odd Prime)
- ✅ TDD (London School) + SPARC methodology
- ✅ CI/CD pipeline design
- ✅ Scalability architecture for **400M+ users**

**Phase 2 Progress:** Production Build with Claude-Flow 🔄

| Component | Lines | Status |
|-----------|-------|--------|
| **Metadata API** | 1,500+ | ✅ Complete |
| **Firestore Schema** | 1,291 | ✅ Complete |
| **Platform Connectors** | 3,066 | ✅ Complete |
| **AgentDB Learning** | 909 | ✅ Complete |
| **Test Suite** | 1,550+ | ✅ 75+ tests |
| **CI/CD Workflows** | 400+ | ✅ Complete |
| **OpenAPI Spec** | 1,178 | ✅ Complete |

**Total: 48+ files, 15,000+ lines of production code**

### 🧠 Claude-Flow Integration (Recommended)

We use **Claude-Flow** with Claude models for reliable agent orchestration:

```bash
cd mondweep

# Option 1: Hive Mind (interactive wizard)
npx claude-flow@alpha hive-mind wizard

# Option 2: Direct spawn with objective
npx claude-flow@alpha hive-mind spawn "Complete remaining Phase 2 tasks" --claude

# Option 3: SPARC methodology
npx claude-flow@alpha sparc run architect "Design Vertex AI integration"
```

**Why Claude-Flow over Agentic-Flow?**
| Feature | Agentic-Flow | Claude-Flow |
|---------|--------------|-------------|
| **MCP Tools** | ~20 | **87 tools** |
| **Models** | Gemini (proxy issues) | **Claude Opus/Sonnet** |
| **Memory** | None | **ReasoningBank + AgentDB** |
| **SPARC** | Basic | **Built-in modes** |

**See:** [CLAUDE.md](CLAUDE.md) | [start-swarm-claude.sh](start-swarm-claude.sh)

---

## 🚀 Key Features

### 1. **Agentic Learning (AgentDB)**
- **Pattern Discovery**: Learns optimal metadata enrichment strategies over time
- **32.6M ops/sec**: Ultra-fast pattern retrieval from learned experiences
- **Reflexion Memory**: Self-critique and continuous improvement
- **Skill Library**: Consolidates successful approaches into reusable skills

### 2. **Intelligent Orchestration (Claude Flow)**
- **101 MCP Tools**: #1 agent orchestration platform integration
- **Workflow Automation**: Complex metadata pipelines automated
- **Multi-step Processing**: Fetch → Embed → Search → Enrich → Validate → Learn
- **Parallel Delivery**: Simultaneous distribution to multiple platforms

### 3. **Production Semantic Search (Vertex AI)**
- **Matching Engine**: Google Cloud's production vector search
- **Auto-scaling**: Handles millions of queries without manual tuning
- **Gemini 2.0 Embeddings**: State-of-the-art semantic understanding
- **<100ms Latency**: Sub-second natural language queries

### 4. **RuVector Integration**
- **GPU-less Architecture**: CPU-optimized hypergraph engine
- **FastRP & Node2Vec**: Efficient embedding generation
- **SPSA Optimization**: Gradient-free parameter tuning
- **CloudRun-native**: Stateless, auto-scaling design

### 5. **Platform-Specific Validation**
- **Netflix IMF**: Automated package generation with Dolby Vision support
- **Amazon MEC**: Prime Video metadata feed compliance
- **FAST Platforms**: MRSS feed generation for linear TV
- **Pre-flight Checks**: Validation before delivery prevents rejections

### 6. **ARW (Agent-Ready Web) Compliance**
- **85% Token Reduction**: Machine-readable views vs HTML scraping
- **MCP Server**: 5 tools for AI assistant integration
- **Structured Manifests**: /.well-known/arw-manifest.json
- **AI Headers**: Full observability of agent traffic

## 📁 Project Structure

```
hackathon-tv5/
├── mondweep/                          # Your hackathon workspace
│   ├── vibe-cast/                     # Git submodule (RuVector source)
│   │   └── ruvector-engine/           # Vector-based recommendation engine
│   ├── ruvector-engine -> vibe-cast/ruvector-engine  # Convenience symlink
│   ├── docs/
│   │   ├── Metadata Optimization Platform PRD.md
│   │   ├── PRD_ Hypergraph Metadata Platform.md
│   │   ├── ruvector-integration.md    # Integration guide
│   │   └── setup-ruvector-reference.md # Setup options
│   └── apps/                          # UMMID platform applications (TBD)
│       ├── metadata-api/              # Core metadata API
│       ├── enrichment-service/        # AI enrichment service
│       └── distribution-engine/       # Platform connectors
```

## 🔧 Technology Stack

### Hackathon Tools Integration
- **AgentDB**: Agentic AI state management and pattern learning
- **Claude Flow**: #1 agent orchestration platform (101 MCP tools)
- **Agentic-Synth**: Synthetic data generation for testing
- **Gemini CLI**: Google Gemini model interface
- **Vertex AI SDK**: Production ML platform

### Core Platform
- **Backend**: Node.js / TypeScript
- **API**: REST + GraphQL + MCP (Model Context Protocol)
- **Database**: Firestore (hypergraph), Cloud SQL with pgvector (embeddings)
- **Cloud**: Google Cloud Platform (Cloud Run, Vertex AI, Cloud Workflows)

### RuVector Engine
- **Embeddings**: FastRP, Node2Vec (CPU-optimized)
- **Optimization**: SPSA (Simultaneous Perturbation Stochastic Approximation)
- **Vector Store**: IVF-based approximate nearest neighbor
- **Architecture**: Hypergraph data model for n-ary relationships

### AI/ML
- **LLMs**: Gemini 2.0 (embeddings, enrichment, reasoning)
- **Vector Search**: Vertex AI Matching Engine + RuVector
- **Learning**: AgentDB (pattern discovery, reflexion, skill consolidation)
- **Orchestration**: Claude Flow (workflow automation)

### ARW (Agent-Ready Web)
- **Manifest**: /.well-known/arw-manifest.json
- **Discovery**: /llms.txt for AI agent consumption
- **MCP Server**: 5 tools for AI assistant integration
- **Headers**: AI-* headers for observability

## 🚀 Quick Start

### Prerequisites

```bash
# Install hackathon tools
npx agentics-hackathon init
# Select: Entertainment Discovery track
# Install: claudeFlow, geminiCli, vertexAi, agentDb

# Verify Node.js and npm
node --version  # v18+
npm --version   # v9+

# Authenticate with GCP
gcloud auth login
gcloud config set project agentics-foundation25lon-1899

# Configure API keys (already set in mondweep/.env)
# - GOOGLE_AI_STUDIO_API_KEY - For Gemini 2.0 embeddings and enrichment
# - ANTHROPIC_API_KEY - For Claude agents (optional)
# - OPENROUTER_API_KEY - For cost optimization (optional)
```

**Note:** Google AI Studio API key is already configured in `mondweep/.env` (gitignored for security).

### Setup

```bash
# Clone with submodules
git clone --recursive https://github.com/mondweep/hackathon-tv5.git
cd hackathon-tv5

# Or initialize submodules if already cloned
git submodule update --init --recursive

# Install RuVector Engine
cd mondweep/ruvector-engine
npm install
cp .env.example .env
# Edit .env with your GCP project settings

# Run RuVector demo
npm run demo

# Start RuVector server
npm start
```

### Verify Installation

```bash
# Check RuVector health
curl http://localhost:8080/api/v1/health

# Initialize with sample data
curl -X POST http://localhost:8080/api/v1/initialize \
  -H "Content-Type: application/json" \
  -d '{"config": {"embeddingDimensions": 128}}'

# Get recommendations
curl http://localhost:8080/api/v1/recommendations/user-alice?limit=10
```

## 📚 Documentation

### **🎯 Start Here**
- **[🏆 Hackathon Strategy](docs/HACKATHON_STRATEGY.md)** - ⭐⭐⭐ Complete competition strategy
- **[🚀 Enhanced Implementation Plan](docs/ENHANCED_IMPLEMENTATION_PLAN.md)** - ⭐⭐⭐ Week-by-week with tools
- **[📋 Master PRD](docs/MASTER_PRD.md)** - ⭐⭐ Consolidated product requirements

### **Implementation Guides**
- **[📖 Documentation Index](docs/README.md)** - Complete documentation overview
- **[📊 Comprehensive Summary](docs/COMPREHENSIVE_SUMMARY.md)** - All deliverables summary
- **[📝 Implementation Plan](docs/IMPLEMENTATION_PLAN.md)** - Original 4-week plan

### **Integration Guides**
- **[RuVector Integration](docs/ruvector-integration.md)** - Detailed integration documentation
- **[Quick Reference](docs/QUICK_REFERENCE.md)** - Common commands and APIs
- **[Setup Summary](docs/SETUP_SUMMARY.md)** - Environment setup guide
- **[Setup Options](docs/setup-ruvector-reference.md)** - Multiple setup approaches

### **Original PRDs (Reference)**
- **[UMMID PRD](docs/Metadata%20Optimization%20Platform%20PRD.md)** - Original metadata platform PRD
- **[Hypergraph PRD](docs/PRD_%20Hypergraph%20Metadata%20Platform.md)** - Original hypergraph architecture

### **Quick Links**
- **Hackathon**: [agentics.org/hackathon](https://agentics.org/hackathon)
- **Discord**: [discord.agentics.org](https://discord.agentics.org)
- **Tools**: `npx agentics-hackathon`
- **GCP Project**: `agentics-foundation25lon-1899`

## 🎨 Use Cases

### 1. Semantic Content Discovery
```javascript
// Find movies based on natural language
const results = await ruvector.semanticSearch(
  "dark psychological thriller with twist ending",
  { filters: { type: 'movie', rating: 'R' }, limit: 10 }
);
```

### 2. Metadata Gap Analysis
```javascript
// Identify and fill missing metadata
const similar = await ruvector.findSimilar(incompleteItemId, { limit: 5 });
const suggestions = extractCommonMetadata(similar);
await ummid.enrichMetadata(itemId, suggestions);
```

### 3. Platform-Specific Validation
```javascript
// Validate before distribution
const validation = await ummid.validateForPlatform(contentId, 'netflix');
if (!validation.isValid) {
  console.log('Missing fields:', validation.missingFields);
}
```

### 4. Trending Analysis
```javascript
// Prioritize metadata for trending content
const trending = await ruvector.getTrending({ timeWindow: '7d' });
await ummid.prioritizeEnrichment(trending.map(t => t.id));
```

## 🔌 API Endpoints

### RuVector Engine

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/health` | GET | Health check |
| `/api/v1/initialize` | POST | Initialize engine |
| `/api/v1/recommendations/:userId` | GET | Personalized recommendations |
| `/api/v1/similar/:itemId` | GET | Find similar items |
| `/api/v1/trending` | GET | Get trending items |
| `/api/v1/media` | POST | Add media item |
| `/api/v1/interactions` | POST | Record interaction |
| `/api/v1/stats` | GET | Engine statistics |

## 🚢 Deployment

### Local Development
```bash
# Terminal 1: RuVector Engine
cd mondweep/ruvector-engine
npm start

# Terminal 2: UMMID Platform (when ready)
cd mondweep/apps/metadata-api
npm run dev
```

### Google Cloud Run
```bash
# Deploy RuVector
cd mondweep/ruvector-engine
gcloud run deploy ruvector-engine \
  --source . \
  --region us-central1 \
  --platform managed \
  --memory 2Gi \
  --cpu 2 \
  --max-instances 100
```

## 📊 Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    UMMID Platform                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Metadata    │  │  Validation  │  │ Distribution │          │
│  │  Ingestion   │  │   Engine     │  │   Engine     │          │
│  └──────┬───────┘  └──────────────┘  └──────────────┘          │
│         │                                                        │
│         ▼                                                        │
│  ┌─────────────────────────────────────────────────┐           │
│  │         RuVector Integration Layer              │           │
│  │  ┌──────────────┐  ┌──────────────┐            │           │
│  │  │  Embedding   │  │  Similarity  │            │           │
│  │  │  Generator   │  │    Search    │            │           │
│  │  └──────────────┘  └──────────────┘            │           │
│  └─────────────────────────────────────────────────┘           │
└─────────────────────────────────────────────────────────────────┘
```

## 🎯 Hackathon Milestones

> **Timeline Reference:** See [BUILD_READINESS.md](docs/BUILD_READINESS.md) and [PRODUCTION_SWARM_STRATEGY.md](docs/PRODUCTION_SWARM_STRATEGY.md) for detailed phase breakdowns.

---

### ✅ Phase 1: Foundation & Strategy (Day 1) — COMPLETE
*Swarm Initialization & Documentation*

- [x] Project setup and repository structure
- [x] RuVector Engine integration (git submodule)
- [x] **Comprehensive documentation (13 documents, 8000+ lines)**
- [x] **Master PRD consolidating both original PRDs**
- [x] **Enhanced implementation plan with hackathon tools**
- [x] **Hackathon strategy and competitive analysis**
- [x] **Production swarm strategy (13 agents - odd prime)**
- [x] **TDD (London School) + SPARC methodology**
- [x] **CI/CD pipeline design**
- [x] **Scalability architecture for 400M+ users**

**Deliverables:** Strategy docs in `mondweep/docs/`, swarm config in `swarm-config-production-v2.ts`

---

### 🔄 Phase 2: Production Build (Days 2-21) — IN PROGRESS

*Parallel Development with 13-Agent Swarm*

#### ✅ Completed Deliverables

| Component | Location | Lines | Status |
|-----------|----------|-------|--------|
| **Metadata API** | `apps/metadata-api/src/` | 1,500+ | ✅ Complete |
| **Firestore Schema** | `apps/metadata-api/src/db/` | 1,291 | ✅ Complete |
| **Netflix IMF Connector** | `apps/metadata-api/src/connectors/netflix-imf.ts` | 786 | ✅ Complete |
| **Amazon MEC Connector** | `apps/metadata-api/src/connectors/amazon-mec.ts` | 883 | ✅ Complete |
| **FAST MRSS Connector** | `apps/metadata-api/src/connectors/fast-mrss.ts` | 860 | ✅ Complete |
| **Connector Types** | `apps/metadata-api/src/connectors/types.ts` | 421 | ✅ Complete |
| **AgentDB Learning** | `apps/metadata-api/src/learning/` | 909 | ✅ Complete |
| **TDD Test Suite** | `apps/metadata-api/tests/` | 1,550+ | ✅ 75+ tests |
| **OpenAPI Spec** | `apps/metadata-api/docs/openapi.yaml` | 1,178 | ✅ Complete |
| **CI/CD Workflows** | `.github/workflows/metadata-api-*.yml` | 400+ | ✅ Complete |

**Total: 48+ files, 15,000+ lines of production code**

#### 13-Agent Swarm Architecture (Odd Prime Consensus)

| Team | Agents | Responsibility | Status |
|------|--------|----------------|--------|
| **Coordinator** | `adaptive-coordinator` | Orchestration, conflict resolution | ✅ Active |
| **Backend** | `backend-dev`, `database-architect`, `api-docs`, `platform-integrator` | API, schemas, docs, connectors | ✅ Delivered |
| **QA/Testing** | `tdd-london-swarm`, `sparc-agent`, `tester`, `production-validator` | TDD, validation, load testing | ✅ Delivered |
| **DevOps** | `cicd-engineer`, `release-manager`, `system-architect` | CI/CD, deployment, scalability | ✅ Delivered |
| **Data/ML** | `data-scientist`, `ml-developer` | AgentDB learning, pattern storage | ✅ Delivered |

#### 🧠 Orchestration: Claude-Flow (Recommended)

```bash
cd mondweep
# Hive Mind with persistent memory
npx claude-flow@alpha hive-mind spawn "Build remaining features" --claude

# Or use SPARC modes
npx claude-flow@alpha sparc run architect "Design Vertex AI integration"
```

#### Quick Start (Run the Build)

```bash
cd apps/metadata-api
npm install
npm run dev      # Start server at http://localhost:8080
npm test         # Run 75+ TDD tests
```

#### Remaining Phase 2 Tasks

- [x] Platform connectors (Netflix IMF, Amazon MEC, FAST) ✅
- [x] AgentDB pattern learning ✅
- [ ] Vertex AI Matching Engine integration
- [ ] Agentic-Synth data generation (1M+ records)
- [ ] RuVector semantic search integration

---

### 📅 Phase 3: Production Deployment (Days 22-28)

*Cloud Run Deployment & Global Scale*

- [ ] Cloud Run deployment (auto-scaling 1-100 instances)
- [ ] Vertex AI Matching Engine (1000 replicas)
- [ ] Global Load Balancer + Cloud CDN
- [ ] Multi-region deployment (us-central1, europe-west1, asia-east1)
- [ ] Monitoring + alerting (Cloud Monitoring)
- [ ] Production URL: `https://metadata-api-<hash>-uc.a.run.app`

**Target:** 400M+ users, <20ms P99 latency, 99.99% uptime

---

### 🎬 Phase 4: Demo & Presentation (Week 4)

*Final Polish & Submission*

- [ ] Demo application with learning dashboard
- [ ] Performance validation (400M user simulation)
- [ ] Presentation materials (slides, talking points)
- [ ] Video walkthrough (5-10 min)
- [ ] Hackathon submission package

---

### 📊 Success Metrics

| Metric | Target | Current Status |
|--------|--------|----------------|
| **Documentation** | Complete | ✅ 15+ docs, 10,000+ lines |
| **Swarm Strategy** | Production-ready | ✅ 13 agents (odd prime) |
| **Metadata API** | Functional | ✅ Express + TypeScript |
| **Database Schema** | 400M+ users | ✅ Firestore hypergraph |
| **Platform Connectors** | 3+ platforms | ✅ Netflix IMF, Amazon MEC, FAST MRSS |
| **AgentDB Learning** | Pattern storage | ✅ SQLite + ReasoningBank |
| **Test Suite** | TDD coverage | ✅ 75+ tests (London School) |
| **OpenAPI Spec** | Full documentation | ✅ 1,178 lines |
| **CI/CD Pipeline** | Automated | ✅ GitHub Actions + Cloud Run |
| **Claude-Flow** | Orchestration | ✅ Hive Mind + SPARC modes |
| **Semantic Search** | <100ms latency | 🎯 Phase 3 |
| **Code Coverage** | >95% | 🎯 TDD enforced |
| **Production Deploy** | Cloud Run live | 🎯 Phase 3 |

---

### 🎓 Key Innovations

| Innovation | Description | Benefit |
|------------|-------------|---------|
| **13-Agent Swarm (Odd Prime)** | Optimal consensus voting, no deadlocks | Reliable parallel development |
| **TDD London School** | Tests written FIRST, mocks for dependencies | High code quality |
| **SPARC Methodology** | Specification → Pseudocode → Architecture → Refinement → Completion | Systematic development |
| **Hypergraph Architecture** | N-ary relationships for rights management | Complex media rights modeling |
| **Bitemporal Modeling** | Valid time + transaction time | Time-travel queries for auditing |
| **Auto-Fix Loop** | Agents fix issues until all tests pass | Self-healing builds |
| **Production Scale** | 400M+ users, ~$215K/month | Enterprise-ready from day 1 |

## 🤝 Contributing

This is a hackathon project, but contributions and suggestions are welcome!

### Development Workflow

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make changes and test**
   ```bash
   npm test
   npm run lint
   ```

3. **Update submodules if needed**
   ```bash
   cd mondweep/vibe-cast
   git pull origin claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn
   cd ../..
   git add mondweep/vibe-cast
   ```

4. **Commit and push**
   ```bash
   git commit -m "feat: add your feature"
   git push origin feature/your-feature-name
   ```

## 📜 License

Apache-2.0

## 🔗 Links

- **Hackathon**: [agentics.org/hackathon](https://agentics.org/hackathon)
- **Discord**: [discord.agentics.org](https://discord.agentics.org)
- **RuVector Source**: [vibe-cast/ruvector-engine](https://github.com/mondweep/vibe-cast/tree/claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn/ruvector-engine)
- **GCP Project**: `agentics-foundation25lon-1899`

## 🙏 Acknowledgments

- **Agentics Foundation** - Hackathon organization and support
- **Google Cloud** - Infrastructure and AI/ML services
- **RuVector Engine** - GPU-less recommendation architecture
- **Claude/Gemini** - AI assistance in development

---

<div align="center">

**🚀 Building the Future of Media Metadata with Agentic AI**

*Agentics Foundation TV5 Hackathon 2025 - Entertainment Discovery Track*

**📚 Documentation**  
[Hackathon Strategy](docs/HACKATHON_STRATEGY.md) • [Enhanced Plan](docs/ENHANCED_IMPLEMENTATION_PLAN.md) • [Master PRD](docs/MASTER_PRD.md)

**🔗 Quick Links**  
[Hackathon](https://agentics.org/hackathon) • [Discord](https://discord.agentics.org) • [Tools](https://www.npmjs.com/package/agentics-hackathon)

**⭐ Star this repo if you find it helpful!**

</div>
