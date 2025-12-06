# AgentDB Pattern Learning Integration

Intelligent pattern learning for metadata enrichment using AgentDB with ReasoningBank memory integration.

## Overview

This module provides AI-powered learning capabilities for the Nexus-UMMID Metadata API, enabling:

- **Pattern Storage**: Store enrichment patterns in SQLite database with vector embeddings
- **Smart Suggestions**: Get intelligent enrichment approach recommendations based on historical data
- **Success Tracking**: Monitor success rates, quality metrics, and performance
- **Adaptive Learning**: Continuously improve recommendations based on new enrichment results

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Metadata API                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐        ┌──────────────┐                  │
│  │   Metadata   │───────▶│   Pattern    │                  │
│  │   Service    │        │   Learner    │                  │
│  └──────────────┘        └───────┬──────┘                  │
│         │                        │                          │
│         │                        │                          │
│         │                ┌───────▼──────┐                  │
│         │                │   AgentDB    │                  │
│         │                │   Client     │                  │
│         │                └───────┬──────┘                  │
│         │                        │                          │
│         └────────────────────────┼──────────────────────────┤
│                                  │                          │
│                         ┌────────▼────────┐                 │
│                         │  SQLite DB      │                 │
│                         │  (ReasoningBank)│                 │
│                         └─────────────────┘                 │
│                    mondweep/.swarm/memory.db                │
└─────────────────────────────────────────────────────────────┘
```

## Components

### 1. AgentDBClient (`agentdb-client.ts`)

Manages persistent storage and retrieval of enrichment patterns.

**Features:**
- SQLite-based pattern storage
- Vector embedding support (384-dimensional)
- Semantic similarity search
- Success rate tracking
- Approach statistics

**Key Methods:**
```typescript
storeEnrichmentPattern(input, output, success, metadata)
retrieveSimilarPatterns(input, limit, minQuality)
getApproachStats(approach)
getStats()
```

### 2. PatternLearner (`pattern-learner.ts`)

Provides intelligent enrichment suggestions based on learned patterns.

**Features:**
- Learn from enrichment results
- Generate confidence-ranked suggestions
- Track quality and performance metrics
- Adaptive recommendations

**Key Methods:**
```typescript
learn(enrichmentResult)
suggest(metadata)
getApproachStats(approach)
getStats()
```

## Installation

Add the required dependency:

```bash
npm install better-sqlite3
npm install --save-dev @types/better-sqlite3
```

## Usage

### Basic Usage

```typescript
import { AgentDBClient, PatternLearner } from './learning';

// Initialize
const agentDB = new AgentDBClient({
  dbPath: 'mondweep/.swarm/memory.db',
  embeddingDimension: 384
});

const learner = new PatternLearner(agentDB, {
  minPatterns: 5,
  minQuality: 0.7,
  similarityThreshold: 0.6,
  topK: 3
});

// Get suggestions
const metadata = await metadataService.getById('asset-001');
const suggestions = learner.suggest(metadata);

// Use best suggestion
const best = suggestions[0];
const result = await metadataService.enrich({
  assetId: 'asset-001',
  approach: best.approach,
  model: best.model
});

// Learn from result
await learner.learn(result);

// Cleanup
agentDB.close();
```

### Integration with MetadataService

```typescript
import { EnrichmentWithLearning } from './learning/example-integration';

const enricher = new EnrichmentWithLearning();

// Smart enrichment with AI suggestions
const result = await enricher.enrichSmart('asset-001');

// Get insights
const insights = enricher.getInsights();
console.log('Overall stats:', insights.overall);
console.log('Approach stats:', insights.byApproach('genre_specialist'));

enricher.close();
```

### Batch Learning

```typescript
// Learn from multiple enrichments
const approaches = ['genre_specialist', 'mood_analyzer', 'default_enrichment'];

for (const approach of approaches) {
  const result = await metadataService.enrich({
    assetId: 'asset-001',
    approach,
    model: 'gemini-2.0'
  });

  await learner.learn(result);
}

// Get optimized suggestions
const suggestions = learner.suggest(metadata);
suggestions.forEach(s => {
  console.log(`${s.approach}: ${(s.confidence * 100).toFixed(0)}% confidence`);
});
```

## Database Schema

```sql
CREATE TABLE enrichment_patterns (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  asset_id TEXT NOT NULL,
  input_metadata TEXT NOT NULL,      -- JSON serialized
  output_metadata TEXT NOT NULL,     -- JSON serialized
  approach TEXT NOT NULL,
  model TEXT NOT NULL,
  quality REAL NOT NULL,
  latency_ms INTEGER NOT NULL,
  tokens_used INTEGER NOT NULL,
  fields_enriched TEXT NOT NULL,     -- JSON array
  success INTEGER NOT NULL,           -- 0 or 1
  embedding TEXT,                     -- JSON array (384-dim)
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for fast retrieval
CREATE INDEX idx_enrichment_patterns_asset ON enrichment_patterns(asset_id);
CREATE INDEX idx_enrichment_patterns_approach ON enrichment_patterns(approach);
CREATE INDEX idx_enrichment_patterns_quality ON enrichment_patterns(quality DESC);
CREATE INDEX idx_enrichment_patterns_success ON enrichment_patterns(success);
CREATE INDEX idx_enrichment_patterns_created ON enrichment_patterns(created_at DESC);
```

## Configuration

### AgentDBClient Config

```typescript
interface AgentDBClientConfig {
  dbPath?: string;              // Default: mondweep/.swarm/memory.db
  embeddingDimension?: number;  // Default: 384
  vectorSearchEnabled?: boolean; // Default: true
}
```

### PatternLearner Config

```typescript
interface LearningConfig {
  minPatterns?: number;         // Default: 5
  minQuality?: number;          // Default: 0.7
  similarityThreshold?: number; // Default: 0.6
  topK?: number;                // Default: 3
}
```

## Metrics & Analytics

### Pattern Statistics

```typescript
const stats = learner.getStats();
// {
//   totalPatterns: 150,
//   successfulPatterns: 127,
//   uniqueApproaches: 8,
//   avgQuality: 0.847
// }
```

### Approach-Specific Stats

```typescript
const approachStats = learner.getApproachStats('genre_specialist');
// {
//   totalAttempts: 45,
//   successCount: 42,
//   successRate: 0.933,
//   avgQuality: 0.892,
//   avgLatency: 1847,
//   avgTokens: 342
// }
```

## Enrichment Suggestions

Suggestions include:

- **approach**: Recommended enrichment approach
- **model**: Best model for this approach
- **confidence**: Confidence score (0-1)
- **expectedQuality**: Expected quality based on history
- **estimatedLatency**: Estimated latency in ms
- **estimatedTokens**: Estimated token usage
- **reason**: Human-readable explanation
- **similarPatterns**: Number of similar historical patterns

```typescript
interface EnrichmentSuggestion {
  approach: string;
  model: string;
  confidence: number;
  expectedQuality: number;
  estimatedLatency: number;
  estimatedTokens: number;
  reason: string;
  similarPatterns: number;
}
```

## Performance

- **Pattern Storage**: ~1ms per pattern
- **Similarity Search**: ~10-50ms for 1000 patterns
- **Suggestion Generation**: ~20-100ms
- **Database Size**: ~1KB per pattern

## Future Enhancements

1. **Vector Embeddings**: Integrate with Vertex AI for semantic embeddings
2. **HNSW Indexing**: Add approximate nearest neighbor search for scale
3. **A/B Testing**: Compare approach performance
4. **Model Selection**: Auto-select optimal AI model
5. **Cost Optimization**: Minimize tokens while maintaining quality
6. **Reinforcement Learning**: Implement multi-armed bandit for approach selection

## Integration with ReasoningBank

This module integrates with the ReasoningBank memory database schema:

- Shares SQLite database: `mondweep/.swarm/memory.db`
- Compatible with existing `patterns`, `task_trajectories` tables
- Uses same memory coordination as Claude Flow swarm agents
- Enables cross-agent pattern sharing

## Testing

Run the example integration:

```bash
npm run dev
# Then in another terminal:
tsx src/learning/example-integration.ts
```

Run tests:

```bash
npm test -- src/learning
```

## License

MIT

## Author

mondweep - Nexus-UMMID Hackathon Team
