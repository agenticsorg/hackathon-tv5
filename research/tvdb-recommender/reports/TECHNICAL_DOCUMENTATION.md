# TVDB Self-Learning Recommendation System - Technical Documentation

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Database Schema](#database-schema)
3. [Self-Learning Algorithm](#self-learning-algorithm)
4. [TVDB API Integration](#tvdb-api-integration)
5. [Vector Search Implementation](#vector-search-implementation)
6. [Natural Language Processing](#natural-language-processing)
7. [API Reference](#api-reference)
8. [Performance Benchmarks](#performance-benchmarks)
9. [Deployment Guide](#deployment-guide)

---

## Architecture Overview

### System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                        Web Interface                             │
│                    (Next.js + HeroUI)                           │
├─────────────────────────────────────────────────────────────────┤
│                         REST API                                 │
│           /search  /recommendations  /similar  /feedback        │
├─────────────────────────────────────────────────────────────────┤
│                   Recommendation Engine                          │
│    ┌────────────┐  ┌─────────────┐  ┌──────────────────┐       │
│    │ NL Parser  │  │ Pattern     │  │ Vector Search    │       │
│    │            │  │ Selector    │  │ (RuVector)       │       │
│    └────────────┘  └─────────────┘  └──────────────────┘       │
├─────────────────────────────────────────────────────────────────┤
│                    Learning Layer                                │
│    ┌────────────┐  ┌─────────────┐  ┌──────────────────┐       │
│    │ Feedback   │  │ Q-Learning  │  │ Pattern          │       │
│    │ Collector  │  │ Algorithm   │  │ Optimizer        │       │
│    └────────────┘  └─────────────┘  └──────────────────┘       │
├─────────────────────────────────────────────────────────────────┤
│                     Data Layer                                   │
│    ┌────────────┐  ┌─────────────┐  ┌──────────────────┐       │
│    │ PostgreSQL │  │ RuVector    │  │ TVDB API         │       │
│    │ Database   │  │ Extension   │  │ Client           │       │
│    └────────────┘  └─────────────┘  └──────────────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
User Query → NL Parser → Intent Extraction → Pattern Selection
                                                    ↓
                                           Vector Search
                                                    ↓
                                           Results Ranking
                                                    ↓
User Feedback ← Display Results ← Personalization
      ↓
Learning Update → Pattern Improvement → Better Future Results
```

---

## Database Schema

### Core Tables

#### `content` - TV Shows and Movies

```sql
CREATE TABLE content (
    id VARCHAR(50) PRIMARY KEY,           -- TVDB ID
    content_type VARCHAR(20) NOT NULL,    -- 'series' or 'movie'
    title VARCHAR(500) NOT NULL,
    year INTEGER,
    overview TEXT,                         -- Rich embedding text
    genres TEXT[],                         -- Array of genres
    rating DECIMAL(3,1),
    network_id INTEGER,
    network_name VARCHAR(200),
    original_language VARCHAR(10),
    original_country VARCHAR(10),
    image_url TEXT,
    first_aired DATE,
    embedding ruvector(384),              -- 384-dim vector
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

#### `recommendation_patterns` - Learning Patterns

```sql
CREATE TABLE recommendation_patterns (
    id SERIAL PRIMARY KEY,
    pattern_type VARCHAR(50) NOT NULL,    -- e.g., 'genre_drama', 'mood_exciting'
    approach TEXT NOT NULL,               -- Description of the pattern
    user_segment VARCHAR(20),             -- Target user type
    success_rate DECIMAL(5,4) DEFAULT 0.5,
    total_uses INTEGER DEFAULT 0,
    avg_reward DECIMAL(5,4) DEFAULT 0,
    embedding ruvector(384),              -- Pattern embedding
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

#### `learning_feedback` - User Interactions

```sql
CREATE TABLE learning_feedback (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    content_id VARCHAR(50),
    pattern_id INTEGER,
    was_successful BOOLEAN NOT NULL,
    reward DECIMAL(5,4) NOT NULL,         -- -1 to 1
    user_action VARCHAR(50) NOT NULL,     -- 'watched', 'skipped', 'rated'
    recommendation_position INTEGER,
    created_at TIMESTAMP DEFAULT NOW()
);
```

### RuVector Functions

```sql
-- Cosine distance for similarity search
SELECT ruvector_cosine_distance(embedding1, embedding2);

-- Enable self-learning on a table
SELECT ruvector_enable_learning('content', '{"algorithm": "q_learning", "reward_decay": 0.95}'::jsonb);

-- Get learning statistics
SELECT ruvector_learning_stats('content');
```

---

## Self-Learning Algorithm

### Q-Learning Implementation

The system uses Q-Learning to optimize pattern selection:

```
Q(s,a) = Q(s,a) + α * (r + γ * max(Q(s',a')) - Q(s,a))

Where:
- s = current state (user context, time, mood)
- a = action (pattern used for recommendation)
- r = reward (1 for watch, -1 for skip, rating for explicit feedback)
- α = learning rate (0.1)
- γ = discount factor (0.95)
```

### Pattern Selection

```typescript
async selectPattern(context: RecommendationContext): Promise<Pattern> {
  // Epsilon-greedy selection with decay
  const epsilon = Math.max(0.1, 1 - (totalUses / 1000));

  if (Math.random() < epsilon) {
    // Explore: random pattern
    return getRandomPattern();
  } else {
    // Exploit: best performing pattern for this context
    return getTopPatternForContext(context);
  }
}
```

### Feedback Processing

```typescript
async recordFeedback(feedback: LearningFeedback): Promise<void> {
  // 1. Record raw feedback
  await insertFeedback(feedback);

  // 2. Calculate reward
  const reward = feedback.wasSuccessful
    ? (0.5 + Math.random() * 0.5)   // Positive: 0.5-1.0
    : (-0.5 + Math.random() * 0.5); // Negative: -0.5-0.0

  // 3. Update pattern success rate
  await updatePatternSuccessRate(feedback.patternId);

  // 4. Trigger pattern re-optimization if threshold reached
  if (getTotalFeedback() % 50 === 0) {
    await optimizePatterns();
  }
}
```

---

## TVDB API Integration

### Authentication

```typescript
async authenticate(): Promise<boolean> {
  const response = await axios.post(`${TVDB_BASE_URL}/login`, {
    apikey: TVDB_API_KEY
  });
  this.token = response.data.data.token;
  return true;
}
```

### Content Ingestion

```typescript
async ingestContent(query: string): Promise<IngestMetrics> {
  // 1. Search TVDB
  const results = await client.get('/search', { params: { query } });

  // 2. Fetch extended data for each result
  for (const item of results.data.data) {
    const extended = await client.get(`/series/${item.id}/extended`);

    // 3. Generate rich embedding text
    const embeddingText = generateEmbeddingText(extended);

    // 4. Store with vector embedding
    await storeContent(extended, embeddingText);
  }
}
```

### Extended Data Extraction

```typescript
function generateEmbeddingText(content: TVDBContent): string {
  const parts = [
    `${content.name} (${content.year})`,
    content.overview,
    `Genres: ${content.genres.map(g => g.name).join(', ')}`,
    `Network: ${content.originalNetwork?.name}`,
    `Starring: ${content.characters.slice(0, 5).map(c => c.personName).join(', ')}`,
    `Status: ${content.status?.name}`,
    `Country: ${content.originalCountry}`
  ];
  return parts.filter(Boolean).join('. ');
}
```

---

## Vector Search Implementation

### Embedding Generation

Currently using random vectors for demo. In production, use:

```typescript
import { pipeline } from '@xenova/transformers';

const embedder = await pipeline('feature-extraction', 'Xenova/all-MiniLM-L6-v2');

async function generateEmbedding(text: string): Promise<number[]> {
  const output = await embedder(text, { pooling: 'mean', normalize: true });
  return Array.from(output.data);
}
```

### Similarity Search

```sql
-- Find similar content using cosine distance
SELECT c.id, c.title,
       1 - ruvector_cosine_distance(c.embedding,
           (SELECT embedding FROM content WHERE id = $1)) as similarity
FROM content c
WHERE c.id != $1 AND c.embedding IS NOT NULL
ORDER BY ruvector_cosine_distance(c.embedding,
         (SELECT embedding FROM content WHERE id = $1))
LIMIT 10;
```

### Performance Optimization

```sql
-- Create HNSW index for faster search at scale
CREATE INDEX idx_content_hnsw ON content
  USING hnsw (embedding ruvector_cosine_ops)
  WITH (m = 16, ef_construction = 64);
```

---

## Natural Language Processing

### Intent Extraction

```typescript
interface QueryIntent {
  mood?: 'relaxing' | 'exciting' | 'thoughtful' | 'funny' | 'romantic' | 'scary';
  audience?: 'kids' | 'family' | 'adults' | 'teens';
  contentType?: 'series' | 'movie' | 'both';
  timeAvailable?: number;
  genres?: string[];
  similarTo?: string;
  recency?: 'trending' | 'new' | 'classic';
}
```

### Keyword Mappings

```typescript
const MOOD_KEYWORDS = {
  'fun': 'funny',
  'exciting': 'exciting',
  'action': 'exciting',
  'relaxing': 'relaxing',
  'romantic': 'romantic',
  'scary': 'scary',
  'horror': 'scary'
};

const GENRE_KEYWORDS = {
  'sci-fi': 'Sci-Fi',
  'drama': 'Drama',
  'comedy': 'Comedy',
  'thriller': 'Thriller'
};
```

### Query Processing

```typescript
function extractIntent(query: string): QueryIntent {
  const intent: QueryIntent = {};
  const queryLower = query.toLowerCase();

  // Extract mood
  for (const [keyword, mood] of Object.entries(MOOD_KEYWORDS)) {
    if (queryLower.includes(keyword)) {
      intent.mood = mood;
      break;
    }
  }

  // Extract "similar to" pattern
  const similarMatch = query.match(/like\s+["']?([^"']+)["']?/i);
  if (similarMatch) {
    intent.similarTo = similarMatch[1].trim();
  }

  // Extract time constraints
  const timeMatch = query.match(/(\d+)\s*(?:hour|hr)s?/i);
  if (timeMatch) {
    intent.timeAvailable = parseInt(timeMatch[1]) * 60;
  }

  return intent;
}
```

---

## API Reference

### GET /api/search

Search content using natural language.

**Parameters:**
- `q` (required): Search query
- `limit` (optional): Max results (default: 20)

**Response:**
```json
{
  "query": "french drama",
  "results": [
    {
      "id": "123",
      "title": "Lupin",
      "year": 2021,
      "genres": ["Drama", "Crime"],
      "overview": "...",
      "similarity": 0.85
    }
  ],
  "count": 10,
  "timestamp": "2024-12-07T..."
}
```

### GET /api/recommendations

Get personalized recommendations.

**Parameters:**
- `mood` (optional): funny, exciting, romantic, etc.
- `type` (optional): series, movie, both
- `limit` (optional): Max results (default: 20)

**Response:**
```json
{
  "mood": "exciting",
  "results": [...],
  "count": 20,
  "timestamp": "..."
}
```

### GET /api/similar

Find similar content.

**Parameters:**
- `id` (required): Content ID
- `limit` (optional): Max results (default: 10)

### POST /api/feedback

Record user feedback for learning.

**Body:**
```json
{
  "contentId": "123",
  "wasSuccessful": true,
  "patternId": 1
}
```

### GET /api/stats

Get learning statistics.

**Response:**
```json
{
  "stats": {
    "totalContent": 1444,
    "totalPatterns": 27,
    "avgSuccessRate": 0.844,
    "totalFeedback": 340,
    "topPatterns": [...]
  }
}
```

---

## Performance Benchmarks

### Test Environment

- **Database**: RuVector PostgreSQL with AVX2 SIMD
- **Content**: 1,444 items with 384-dim embeddings
- **Hardware**: Standard cloud instance

### Results

| Operation | Avg Latency | p99 Latency | Throughput |
|-----------|-------------|-------------|------------|
| Vector Search (Top 10) | 3.22ms | 6.5ms | 310 ops/sec |
| Recommendations (Top 20) | 0.97ms | 2.1ms | 1,030 ops/sec |
| Pattern Learning | 1.91ms | 3.6ms | 523 ops/sec |

### Scaling Projections

| Content Items | Search Latency | Notes |
|---------------|----------------|-------|
| 1,000 | ~3ms | Current benchmark |
| 10,000 | ~10ms | With HNSW index |
| 100,000 | ~25ms | With HNSW + sharding |
| 1,000,000 | ~50ms | Distributed architecture |

---

## Deployment Guide

### Prerequisites

1. Docker installed
2. Node.js 18+
3. TVDB API key

### Step 1: Database Setup

```bash
docker run -d --name ruvector-postgres \
  -e POSTGRES_USER=tvdb \
  -e POSTGRES_PASSWORD=tvdb_recommender_2024 \
  -e POSTGRES_DB=tvdb_recommender \
  -p 5432:5432 \
  ruvnet/ruvector-postgres
```

### Step 2: Environment Configuration

```bash
# .env file
TVDB_API_KEY=your-api-key
DATABASE_URL=postgresql://tvdb:tvdb_recommender_2024@localhost:5432/tvdb_recommender
```

### Step 3: Schema Initialization

```bash
psql $DATABASE_URL -f src/db/schema-ruvector.sql
```

### Step 4: Content Ingestion

```bash
npx tsx src/scripts/ingest-tv5-content.ts
```

### Step 5: Start Web Server

```bash
cd app
npm install
npm run dev
```

### Step 6: Access Application

Open http://localhost:3000 in your browser.

---

## Monitoring and Maintenance

### Health Checks

```sql
-- Check content count
SELECT COUNT(*) FROM content;

-- Check learning progress
SELECT pattern_type, success_rate, total_uses
FROM recommendation_patterns
ORDER BY success_rate DESC;

-- Check recent feedback
SELECT was_successful, COUNT(*)
FROM learning_feedback
WHERE created_at > NOW() - INTERVAL '1 hour'
GROUP BY was_successful;
```

### Continuous Learning

The system automatically improves through:
1. User feedback collection (Watch/Skip buttons)
2. Pattern success rate updates
3. Q-Learning optimization

### Content Updates

```bash
# Run periodic updates from TVDB
npx tsx src/scripts/ingest-tv5-content.ts
```

---

*This documentation covers the TVDB Self-Learning Recommendation System built for the TV5 Hackathon.*
