# TVDB Self-Learning Recommendation System Architecture

## System Overview

A self-learning recommendation system that uses TheTVDB.com API data with vector embeddings, pattern learning via AgentDB's ReasoningBank, and agentic workflow orchestration.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    TVDB Self-Learning Recommendation System                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    ┌───────────────┐    ┌──────────────────────────────┐ │
│  │   User       │───▶│  Agentic      │───▶│     Recommendation Engine    │ │
│  │   Request    │    │  Workflow     │    │                              │ │
│  └──────────────┘    │  Orchestrator │    │  ┌────────────────────────┐  │ │
│                      └───────────────┘    │  │   ReasoningBank        │  │ │
│                             │             │  │   (Pattern Learning)   │  │ │
│                             ▼             │  └────────────────────────┘  │ │
│  ┌──────────────────────────────────────┐ │  ┌────────────────────────┐  │ │
│  │         TVDB API Client              │ │  │   ReflexionMemory      │  │ │
│  │  • Authentication (JWT)              │ │  │   (Success/Failure)    │  │ │
│  │  • Search Series/Movies              │ │  └────────────────────────┘  │ │
│  │  • Get Details & Episodes            │ │  ┌────────────────────────┐  │ │
│  │  • Fetch Artwork & People            │ │  │   SkillLibrary         │  │ │
│  └──────────────────────────────────────┘ │  │   (Learned Patterns)   │  │ │
│                      │                    │  └────────────────────────┘  │ │
│                      ▼                    └──────────────────────────────┘ │
│  ┌──────────────────────────────────────┐                                  │
│  │      Vector Embedding Service        │                                  │
│  │  • Transformers.js (local)           │                                  │
│  │  • all-MiniLM-L6-v2 (384-dim)        │                                  │
│  │  • Content embeddings                │                                  │
│  │  • User preference vectors           │                                  │
│  └──────────────────────────────────────┘                                  │
│                      │                                                     │
│                      ▼                                                     │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │                         Data Layer                                    │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐   │ │
│  │  │    AgentDB      │  │   PostgreSQL    │  │   Content Cache     │   │ │
│  │  │  (Vector DB)    │  │   (pgvector)    │  │   (Redis/Memory)    │   │ │
│  │  │                 │  │                 │  │                     │   │ │
│  │  │ • Patterns      │  │ • User prefs    │  │ • TVDB responses    │   │ │
│  │  │ • Episodes      │  │ • Watch history │  │ • Embeddings        │   │ │
│  │  │ • Skills        │  │ • Ratings       │  │ • Search results    │   │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────────┘   │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. TVDB API Client (`src/services/tvdb-client.ts`)
- JWT authentication with token refresh
- Series/movie search and metadata retrieval
- Episode and season data
- Artwork and people information
- Rate limiting and caching

### 2. Vector Embedding Service (`src/services/embedding-service.ts`)
- Local embeddings using Transformers.js
- Content vectorization (titles, overviews, genres)
- User preference embeddings
- Similarity computation

### 3. Recommendation Engine (`src/services/recommendation-engine.ts`)
- Hybrid filtering (content-based + collaborative)
- ReasoningBank pattern matching
- Self-learning feedback loops
- Personalization layers

### 4. Agentic Workflow (`src/workflows/recommendation-workflow.ts`)
- Multi-agent coordination
- Task orchestration
- Memory persistence across sessions
- Hook-based automation

### 5. PostgreSQL Schema (`src/db/schema.sql`)
- User profiles and preferences
- Watch history tracking
- Ratings and interactions
- pgvector for embeddings

## Self-Learning Loop

```
┌──────────────────────────────────────────────────────────────────┐
│                     Self-Learning Cycle                          │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│   1. OBSERVE                                                     │
│      ├── User watches content                                    │
│      ├── User rates content                                      │
│      └── User skips/abandons content                             │
│                                                                  │
│   2. LEARN (ReflexionMemory)                                     │
│      ├── Store episode: {context, action, outcome, reward}       │
│      ├── Compute success/failure metrics                         │
│      └── Generate self-critique                                  │
│                                                                  │
│   3. ADAPT (ReasoningBank)                                       │
│      ├── Update pattern success rates                            │
│      ├── Store new successful patterns                           │
│      ├── Train GNN for better recall                             │
│      └── Consolidate patterns (NightlyLearner)                   │
│                                                                  │
│   4. PREDICT (SkillLibrary)                                      │
│      ├── Match user context to learned skills                    │
│      ├── Apply best recommendation strategies                    │
│      └── Personalize based on preference vectors                 │
│                                                                  │
│   5. VERIFY (CausalRecall)                                       │
│      ├── Track recommendation → watch correlation                │
│      ├── Measure intervention effects                            │
│      └── Compute causal uplift                                   │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

## Data Models

### Content Vector Schema
```typescript
interface ContentEmbedding {
  contentId: string;        // TVDB ID (series/movie)
  contentType: 'series' | 'movie';
  vector: Float32Array;     // 384-dim embedding
  metadata: {
    title: string;
    year: number;
    genres: string[];
    overview: string;
    rating: number;
    networkId?: number;
  };
  createdAt: Date;
  updatedAt: Date;
}
```

### User Preference Vector
```typescript
interface UserPreference {
  userId: string;
  preferenceVector: Float32Array;  // Aggregated from watched content
  genreWeights: Record<string, number>;
  watchHistory: WatchHistoryItem[];
  ratings: Rating[];
  lastUpdated: Date;
}
```

### Reasoning Pattern
```typescript
interface RecommendationPattern {
  patternId: number;
  taskType: 'cold_start' | 'genre_match' | 'similar_content' | 'trending';
  context: {
    userSegment: string;
    timeOfDay: string;
    dayOfWeek: string;
    platform: string;
  };
  approach: string;
  successRate: number;
  totalUses: number;
  avgReward: number;
}
```

## Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Vector DB | AgentDB (agentdb@alpha) | Pattern storage, similarity search |
| Relational DB | PostgreSQL + pgvector | User data, watch history |
| Embeddings | Transformers.js | Local content embeddings |
| Workflow | agentic-flow@alpha | Multi-agent orchestration |
| API Client | TypeScript/Node.js | TVDB API integration |
| Caching | Node-Cache | API response caching |

## Performance Targets

- **Pattern Search**: 32.6M ops/sec (AgentDB ReasoningBank)
- **Vector Search**: <100ms p99 latency
- **Recommendation Generation**: <500ms end-to-end
- **Learning Loop**: Real-time updates, nightly consolidation
- **Cache Hit Rate**: >80% for popular content

## Deployment

### Development
```bash
npm install
npm run dev
```

### Production
```bash
npm run build
npm start
```

### Environment Variables
```env
TVDB_API_KEY=your-api-key
TVDB_PIN=optional-subscriber-pin
DATABASE_URL=postgresql://user:pass@localhost:5432/tvdb_recommender
AGENTDB_PATH=./data/agent-memory.db
NODE_ENV=production
```
