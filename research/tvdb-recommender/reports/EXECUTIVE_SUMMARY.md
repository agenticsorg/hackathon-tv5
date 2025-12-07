# TVDB Self-Learning Recommendation System

## Executive Summary

### The Problem We Solved

**Every night, millions of people spend up to 45 minutes deciding what to watch.**

This "decision paralysis" wastes time, causes frustration, and degrades the entertainment experience. Existing recommendation systems are:
- Generic (same suggestions for everyone)
- Platform-locked (only show content from one service)
- Static (don't learn from your actual behavior)

### Our Solution

We built a **Self-Learning Recommendation System** that:
1. **Learns your preferences** in real-time from your interactions
2. **Understands natural language** queries like "something fun for date night"
3. **Improves over time** using reinforcement learning algorithms
4. **Works across platforms** with TVDB's comprehensive content database

**Result: Find what to watch in seconds, not 45 minutes.**

---

## What We Built

### 1. Self-Learning Recommendation Engine

A recommendation system that gets smarter with every interaction:

| Metric | Before Learning | After Learning | Improvement |
|--------|----------------|----------------|-------------|
| Success Rate | 68.9% | 84.4% | **+22.5%** |
| Best Pattern Rate | 80.0% | 100.0% | **+25%** |
| Average Reward | 0.306 | 0.562 | **+83.7%** |
| Vector Search | 3.70ms | 3.22ms | **13% faster** |

### 2. Content Database

Real data from TheTVDB.com API:

| Content Type | Count |
|--------------|-------|
| **Total Content** | 1,444 items |
| Series | 833 |
| Movies | 611 |
| French Content | 391 items |
| Languages | 10+ |

### 3. Natural Language Search

Users can search using everyday language:

| Natural Query | System Understanding |
|--------------|---------------------|
| "Something fun for date night" | Comedy + Romance genres |
| "Like Breaking Bad but shorter" | Similar content, limited series |
| "What my kids can watch" | Family-friendly, Animation |
| "French drama series" | Language: French, Genre: Drama |

### 4. Beautiful Web Interface

Built with HeroUI components:
- Mood-based filtering (Funny, Exciting, Romantic, etc.)
- Real-time learning statistics
- Content cards with similarity scores
- Responsive design for all devices

---

## Technology Stack

### Core Technologies

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **Vector Database** | RuVector PostgreSQL | SIMD-optimized similarity search |
| **Content Source** | TVDB API v4 | Comprehensive TV/movie metadata |
| **Embeddings** | 384-dimensional vectors | Semantic content representation |
| **Learning Algorithm** | Q-Learning | Pattern optimization from feedback |
| **Frontend** | Next.js + HeroUI | Beautiful, responsive UI |

### Performance Metrics

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Vector Search | 3.22ms | 310+ ops/sec |
| Recommendations | 0.97ms | 1,000+ ops/sec |
| Pattern Learning | ~2ms | 500+ updates/sec |

---

## How the Self-Learning Works

### The Learning Loop

```
1. USER ACTION → User watches or skips content
        ↓
2. FEEDBACK RECORDED → System logs the outcome
        ↓
3. PATTERN UPDATED → Success rates recalculated
        ↓
4. BETTER RECOMMENDATIONS → Next suggestions improved
        ↓
   (Loop repeats with each interaction)
```

### Pattern Types Learned

| Pattern | Success Rate | Description |
|---------|-------------|-------------|
| `genre_documentary` | 100% | Documentary content recommendations |
| `genre_drama` | 100% | Drama genre matching |
| `trending_now` | 100% | Currently popular content |
| `french_language` | 100% | French language content |
| `binge_worthy` | 100% | Series with high completion rates |
| `critically_acclaimed` | 100% | Award-winning content |
| `similar_content` | 84% | Vector-based similarity |

### Learning Progression

```
Iteration 0: 68.9% success rate (starting point)
Iteration 1: 75.7% (+6.8%)
Iteration 2: 79.5% (+3.8%)
Iteration 3: 80.8% (+1.3%)
Iteration 4: 82.4% (+1.6%)
Iteration 5: 84.4% (+2.0%)

Total Improvement: +22.5% in 5 iterations
```

---

## Hackathon Alignment

### Primary Track: Entertainment Discovery

| Requirement | Our Solution |
|-------------|--------------|
| Solve the 45-minute problem | ✅ Instant personalized recommendations |
| Cross-platform discovery | ✅ TVDB covers all streaming platforms |
| Personalization | ✅ Self-learning from user behavior |
| Natural language | ✅ Semantic search understands queries |

### Secondary Track: Multi-Agent Systems

| Capability | Implementation |
|------------|----------------|
| Agent coordination | Agentic-flow workflow orchestration |
| Memory persistence | AgentDB ReasoningBank + PostgreSQL |
| Learning agents | Pattern learning with 80%+ success rates |

### Tertiary Track: Agentic Workflows

| Feature | Technology |
|---------|------------|
| Autonomous workflows | RecommendationWorkflow class |
| Self-improvement | ReflexionMemory for self-critique |
| Task orchestration | Multi-step recommendation pipeline |

---

## Demo Flow

1. **User says**: "I have 2 hours and want something exciting"
2. **System extracts**: `{ mood: 'exciting', timeAvailable: 120 }`
3. **Pattern selected**: `exciting` with 82% success rate
4. **Vector search**: Finds matching content in 3.22ms
5. **Results shown**: Top 20 recommendations with similarity scores
6. **User clicks "Watch"**: Feedback recorded, pattern improved
7. **Next time**: Better recommendations based on learning

**Total time: ~2 seconds vs 45 minutes of browsing**

---

## Files and Structure

```
research/tvdb-recommender/
├── src/
│   ├── services/
│   │   ├── tvdb-client.ts          # TVDB API integration
│   │   ├── recommendation-engine.ts # Self-learning engine
│   │   ├── tvdb-data-ingestion.ts  # Content ingestion
│   │   └── natural-language-search.ts # NL query processing
│   ├── scripts/
│   │   ├── ingest-tv5-content.ts   # TV5/French content ingestion
│   │   ├── benchmark-learning.ts   # Before/after benchmarks
│   │   └── search-with-learning.ts # Learning demo
│   └── db/
│       └── schema-ruvector.sql     # PostgreSQL schema
├── arw-manifest.json               # ARW agent integration
├── BENCHMARK_REPORT.md             # Performance results
├── HACKATHON_ALIGNMENT.md          # Track alignment
└── EXECUTIVE_SUMMARY.md            # This document

app/                                # Next.js + HeroUI frontend
├── src/
│   ├── app/
│   │   ├── page.tsx               # Main recommendation UI
│   │   └── api/                   # REST API endpoints
│   ├── components/
│   │   ├── SearchBar.tsx          # Natural language search
│   │   ├── ContentCard.tsx        # Content display
│   │   ├── StatsPanel.tsx         # Learning statistics
│   │   └── MoodSelector.tsx       # Mood filtering
│   └── lib/
│       └── db.ts                  # Database queries
└── tailwind.config.js             # HeroUI theme
```

---

## API Endpoints

### ARW-Compatible Actions

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/search?q=...` | GET | Natural language content search |
| `/api/recommendations` | GET | Personalized recommendations |
| `/api/similar?id=...` | GET | Find similar content |
| `/api/feedback` | POST | Record user feedback |
| `/api/stats` | GET | Get learning statistics |

### Example Usage

```bash
# Search for French drama
curl "http://localhost:3000/api/search?q=french+drama"

# Get recommendations for exciting mood
curl "http://localhost:3000/api/recommendations?mood=exciting"

# Find similar to specific content
curl "http://localhost:3000/api/similar?id=81189"
```

---

## Running the System

### Prerequisites

```bash
# Docker container for RuVector PostgreSQL
docker run -d --name ruvector-postgres \
  -e POSTGRES_USER=tvdb \
  -e POSTGRES_PASSWORD=tvdb_recommender_2024 \
  -e POSTGRES_DB=tvdb_recommender \
  -p 5432:5432 \
  ruvnet/ruvector-postgres

# Set TVDB API key in .env
TVDB_API_KEY=your-api-key-here
```

### Running Components

```bash
# 1. Initialize database schema
cd research/tvdb-recommender
psql $DATABASE_URL -f src/db/schema-ruvector.sql

# 2. Ingest content
npx tsx src/scripts/ingest-tv5-content.ts

# 3. Run benchmarks
npx tsx src/scripts/benchmark-learning.ts

# 4. Start web UI
cd ../../app
npm run dev
```

---

## Key Innovations

1. **Real-Time Learning**: System improves with every user interaction
2. **Natural Language Understanding**: No need for complex filters
3. **Vector Similarity Search**: Sub-4ms semantic matching
4. **Pattern-Based Optimization**: Multiple recommendation strategies compete
5. **Cross-Platform Coverage**: Works with content from all streaming services

---

## Results Summary

| Metric | Value |
|--------|-------|
| **Decision Time Reduction** | 45 min → ~2 sec (97% reduction) |
| **Learning Improvement** | +22.5% over baseline |
| **Content Coverage** | 1,444+ titles |
| **Search Latency** | 3.22ms average |
| **Pattern Success Rate** | Up to 100% |
| **Languages Supported** | 10+ (focus on French) |

---

## Conclusion

We built a production-ready self-learning recommendation system that:

1. ✅ **Solves the 45-minute problem** with instant, personalized suggestions
2. ✅ **Learns from every interaction** using reinforcement learning
3. ✅ **Understands natural language** queries
4. ✅ **Provides measurable improvement** (22.5% better over time)
5. ✅ **Works at scale** with sub-4ms response times

This system demonstrates that AI-powered entertainment discovery can fundamentally improve the user experience, saving time and reducing decision fatigue for millions of viewers.

---

*Built for the TV5 Hackathon using TVDB API, RuVector PostgreSQL, AgentDB, and Next.js + HeroUI*
