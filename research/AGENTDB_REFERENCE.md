<!--
Source: npm package agentdb@alpha (v2.0.0-alpha.2.20)
Fetched: 2025-12-07T15:50:00Z
Topic: Vector database for AI agents with memory patterns
-->

# AgentDB v2.0 - Complete Reference Guide

**Version:** 2.0.0-alpha.2.20
**Repository:** https://github.com/ruvnet/agentic-flow/tree/main/packages/agentdb
**NPM:** https://www.npmjs.com/package/agentdb
**Homepage:** https://agentdb.ruv.io
**License:** MIT

---

## Overview

AgentDB is the first vector database built specifically for autonomous AI agents. Unlike traditional databases that just store vectors, AgentDB **learns from every interaction**, **heals itself automatically**, and **gets smarter over time** — all while being **150x faster** than cloud alternatives and running **anywhere** (Node.js, browsers, edge functions, even offline).

### Key Differentiators

- **Six Cognitive Memory Patterns** - Reflexion, Skills, Causal Memory, Explainable Recall, Utility Ranking, Nightly Learner
- **150x Faster Vector Search** - RuVector Rust backend with SIMD (61μs p50 latency, 8.2x faster than hnswlib)
- **97.9% Self-Healing** - Automatic degradation prevention using Model Predictive Control
- **Graph Neural Networks** - 8-head attention for adaptive query improvement (+12.4% recall)
- **Runs Anywhere** - Node.js, browsers, edge functions, works offline with graceful degradation
- **Zero Configuration** - Auto-selects optimal backend (RuVector → HNSWLib → better-sqlite3 → sql.js)
- **$0 Cost** - Fully local, no API keys, no cloud fees

---

## Installation

```bash
# Install alpha version (v2.0 with all new features)
npm install agentdb@alpha

# Or stable version
npm install agentdb@latest
```

### System Requirements

- **Node.js**: >=18.0.0
- **Platform**: Linux, macOS, Windows
- **Runtime**: Node.js, browsers (WASM), edge functions

---

## Quick Start (60 seconds)

```typescript
import { createDatabase, ReasoningBank, EmbeddingService } from 'agentdb';

// Initialize database
const db = await createDatabase('./agent-memory.db');

// Initialize embedding service (uses local Transformers.js)
const embedder = new EmbeddingService({
  model: 'Xenova/all-MiniLM-L6-v2',
  dimension: 384
});
await embedder.initialize();

// Create reasoning bank for pattern learning
const reasoningBank = new ReasoningBank(db, embedder);

// Store what your agent learned
await reasoningBank.storePattern({
  taskType: 'code_review',
  approach: 'Security-first analysis',
  successRate: 0.95
});

// Find similar successful patterns later (32.6M ops/sec!)
const patterns = await reasoningBank.searchPatterns({
  task: 'security code review',
  k: 10
});
```

---

## Core Features

### 1. Vector Storage & Similarity Search

AgentDB provides high-performance vector storage with multiple backend options:

**Backend Auto-Selection:**
```
RuVector (Rust+SIMD, 150x faster)
  ↓ fallback
HNSWLib (C++ HNSW, 100x faster)
  ↓ fallback
better-sqlite3 (Native Node)
  ↓ fallback
sql.js (WASM, zero deps - default)
```

**Performance:**
- Vector search: 61μs p50 latency (RuVector)
- Pattern search: 32.6M ops/sec (ultra-fast with caching)
- Super-linear scaling: Performance improves with data size

### 2. Embedding Models

AgentDB supports local embeddings via Transformers.js (no API keys needed):

| Model | Dimension | Quality | Speed | Best For |
|-------|-----------|---------|-------|----------|
| **all-MiniLM-L6-v2** (default) | 384 | ⭐⭐⭐⭐ | ⚡⚡⚡⚡⚡ | Prototyping, demos |
| **bge-small-en-v1.5** | 384 | ⭐⭐⭐⭐⭐ | ⚡⚡⚡⚡ | Best 384-dim quality |
| **bge-base-en-v1.5** | 768 | ⭐⭐⭐⭐⭐ | ⚡⚡⚡ | Production systems |
| all-mpnet-base-v2 | 768 | ⭐⭐⭐⭐⭐ | ⚡⚡⚡ | All-around excellence |
| e5-base-v2 | 768 | ⭐⭐⭐⭐⭐ | ⚡⚡⚡ | Multilingual (100+ languages) |

```typescript
const embedder = new EmbeddingService({
  model: 'Xenova/bge-base-en-v1.5',
  dimension: 768,
  provider: 'transformers'  // Local, no API needed
});
```

---

## API Reference

### Core Exports

```typescript
import {
  // Core
  AgentDB,
  createDatabase,

  // Frontier Memory Controllers
  ReasoningBank,
  ReflexionMemory,
  SkillLibrary,
  CausalMemoryGraph,
  CausalRecall,
  ExplainableRecall,
  NightlyLearner,

  // Services
  EmbeddingService,
  EnhancedEmbeddingService,
  WASMVectorSearch,
  HNSWIndex,
  AttentionService,

  // Optimizations
  BatchOperations,
  QueryOptimizer,
  QueryCache,

  // Security
  validateTableName,
  validateColumnName,
  ValidationError
} from 'agentdb';
```

### 1. ReasoningBank - Pattern Learning & Adaptive Memory

**Purpose:** Store successful reasoning patterns and retrieve them using semantic similarity. Learns which approaches work best for different types of tasks.

**Performance:**
- Pattern storage: 388K ops/sec
- Pattern search: 32.6M ops/sec (ultra-fast with caching)
- Super-linear scaling: 4,536 patterns/sec @ 5k items

#### API

```typescript
class ReasoningBank {
  constructor(
    db: IDatabaseConnection,
    embedder: EmbeddingService,
    vectorBackend?: VectorBackend,
    learningBackend?: LearningBackend
  )

  // Store a reasoning pattern
  storePattern(pattern: ReasoningPattern): Promise<number>

  // Search patterns by semantic similarity
  searchPatterns(query: PatternSearchQuery): Promise<ReasoningPattern[]>

  // Get pattern statistics
  getPatternStats(): PatternStats

  // Update pattern after use
  updatePatternStats(patternId: number, success: boolean, reward: number): void

  // Record outcome for GNN learning (v2)
  recordOutcome(patternId: number, success: boolean, reward?: number): Promise<void>

  // Train GNN model (v2)
  trainGNN(options?: { epochs?: number; batchSize?: number }): Promise<{
    epochs: number;
    finalLoss: number;
  }>

  // Get/delete pattern
  getPattern(patternId: number): ReasoningPattern | null
  deletePattern(patternId: number): boolean

  // Cache management
  clearCache(): void
}
```

#### Types

```typescript
interface ReasoningPattern {
  id?: number;
  taskType: string;           // e.g., "code_review", "data_analysis"
  approach: string;           // Description of reasoning approach
  successRate: number;        // 0-1 success rate
  embedding?: Float32Array;   // Vector embedding for search
  uses?: number;
  avgReward?: number;
  tags?: string[];
  metadata?: Record<string, any>;
  createdAt?: number;
  similarity?: number;        // Set in search results
}

interface PatternSearchQuery {
  task?: string;              // v1 API: auto-embedded
  taskEmbedding?: Float32Array; // v2 API: pre-computed
  k?: number;                 // Number of results (default: 10)
  threshold?: number;         // Min similarity score (0-1)
  useGNN?: boolean;          // Enable GNN enhancement
  filters?: {
    taskType?: string;
    minSuccessRate?: number;
    tags?: string[];
  };
}

interface PatternStats {
  totalPatterns: number;
  avgSuccessRate: number;
  avgUses: number;
  topTaskTypes: Array<{ taskType: string; count: number }>;
  recentPatterns: number;
  highPerformingPatterns: number;
}
```

#### Example

```typescript
const reasoningBank = new ReasoningBank(db, embedder);

// Store pattern
const patternId = await reasoningBank.storePattern({
  taskType: 'bug_investigation',
  approach: 'Check logs → Reproduce issue → Binary search for root cause',
  successRate: 0.92,
  tags: ['debugging', 'systematic'],
  metadata: { avgTimeMs: 3000 }
});

// Search patterns
const patterns = await reasoningBank.searchPatterns({
  task: 'debug memory leak',
  k: 10,
  threshold: 0.7,
  filters: { taskType: 'bug_investigation' }
});

// Record outcome for learning
await reasoningBank.recordOutcome(patternId, true, 0.95);

// Train GNN model
const { epochs, finalLoss } = await reasoningBank.trainGNN({
  epochs: 10,
  batchSize: 32
});
```

---

### 2. ReflexionMemory - Episodic Replay & Self-Critique

**Purpose:** Store complete task episodes with self-generated critiques, then replay them to improve future performance. Based on the Reflexion paper (Shinn et al., 2023).

**Performance:**
- Episode retrieval: 957 ops/sec
- Batch operations: 3.4x faster (7,692 ops/sec)

#### API

```typescript
class ReflexionMemory {
  constructor(
    db: IDatabaseConnection,
    embedder: EmbeddingService,
    vectorBackend?: VectorBackend,
    learningBackend?: LearningBackend,
    graphBackend?: GraphBackend,
    cacheConfig?: QueryCacheConfig
  )

  // Store episode with critique
  storeEpisode(episode: Episode): Promise<number>

  // Retrieve similar episodes
  retrieveRelevant(query: ReflexionQuery): Promise<EpisodeWithEmbedding[]>

  // Get task statistics
  getTaskStats(task: string, timeWindowDays?: number): {
    totalAttempts: number;
    successRate: number;
    avgReward: number;
    avgLatency: number;
    improvementTrend: number;
  }

  // Build critique summary from failed episodes
  getCritiqueSummary(query: ReflexionQuery): Promise<string>

  // Get successful strategies
  getSuccessStrategies(query: ReflexionQuery): Promise<string>

  // Get recent episodes
  getRecentEpisodes(sessionId: string, limit?: number): Promise<Episode[]>

  // Prune low-quality episodes
  pruneEpisodes(config: {
    minReward?: number;
    maxAgeDays?: number;
    keepMinPerTask?: number;
  }): number

  // Graph relationships
  getEpisodeRelationships(episodeId: number): Promise<{
    similar: number[];
    session: string;
    learnedFrom: number[];
  }>

  // GNN training
  trainGNN(options?: { epochs?: number }): Promise<void>

  // Statistics
  getLearningStats(): LearningStats | null
  getGraphStats(): GraphStats | null
  getCacheStats(): CacheStatistics

  // Cache management
  clearCache(): void
  pruneCache(): number
  warmCache(sessionId?: string): Promise<void>
}
```

#### Types

```typescript
interface Episode {
  id?: number;
  ts?: number;
  sessionId: string;
  task: string;
  input?: string;
  output?: string;
  critique?: string;          // Self-generated critique
  reward: number;             // 0-1 reward score
  success: boolean;
  latencyMs?: number;
  tokensUsed?: number;
  tags?: string[];
  metadata?: Record<string, any>;
}

interface EpisodeWithEmbedding extends Episode {
  embedding?: Float32Array;
  similarity?: number;
}

interface ReflexionQuery {
  task: string;
  currentState?: string;
  k?: number;
  minReward?: number;
  onlyFailures?: boolean;
  onlySuccesses?: boolean;
  timeWindowDays?: number;
}
```

#### Example

```typescript
const reflexion = new ReflexionMemory(db, embedder);

// Store episode with self-critique
const episodeId = await reflexion.storeEpisode({
  sessionId: 'debug-session-1',
  task: 'Fix authentication bug',
  reward: 0.95,
  success: true,
  critique: 'OAuth2 PKCE flow was more secure than basic flow. Should always check token expiration.',
  input: 'Users can\'t log in',
  output: 'Working OAuth2 implementation with refresh tokens',
  latencyMs: 1200,
  tokensUsed: 500
});

// Retrieve similar episodes (learn from past)
const similar = await reflexion.retrieveRelevant({
  task: 'authentication issues',
  k: 10,
  onlySuccesses: true,  // Learn from what worked
  minReward: 0.7
});

// Get task statistics
const stats = await reflexion.getTaskStats('debug-session-1');
console.log(`Success rate: ${stats.successRate}`);
console.log(`Avg reward: ${stats.avgReward}`);
```

---

### 3. SkillLibrary - Lifelong Learning

**Purpose:** Transform successful patterns into reusable, composable skills. Automatically consolidate repeated successful task executions into parameterized skills.

**Performance:**
- Skill search: 694 ops/sec
- Skill creation: 304 → 900 ops/sec (with batch)
- Batch operations: 3.6x faster (5,556 ops/sec)

#### API

```typescript
class SkillLibrary {
  constructor(
    db: IDatabaseConnection,
    embedder: EmbeddingService,
    vectorBackend?: VectorBackend,
    graphBackend?: any,
    cacheConfig?: QueryCacheConfig
  )

  // Create skill manually or from episode
  createSkill(skill: Skill): Promise<number>

  // Update skill after use
  updateSkillStats(
    skillId: number,
    success: boolean,
    reward: number,
    latencyMs: number
  ): void

  // Search for applicable skills
  searchSkills(query: SkillQuery): Promise<Skill[]>
  retrieveSkills(query: SkillQuery): Promise<Skill[]>

  // Link skills with relationships
  linkSkills(link: SkillLink): void

  // Get skill composition plan
  getSkillPlan(skillId: number): {
    skill: Skill;
    prerequisites: Skill[];
    alternatives: Skill[];
    refinements: Skill[];
  }

  // Auto-consolidate from successful episodes
  consolidateEpisodesIntoSkills(config: {
    minAttempts?: number;
    minReward?: number;
    timeWindowDays?: number;
    extractPatterns?: boolean;
  }): Promise<{
    created: number;
    updated: number;
    patterns: Array<{
      task: string;
      commonPatterns: string[];
      successIndicators: string[];
      avgReward: number;
    }>;
  }>

  // Prune underperforming skills
  pruneSkills(config: {
    minUses?: number;
    minSuccessRate?: number;
    maxAgeDays?: number;
  }): number

  // Cache management
  getCacheStats(): CacheStatistics
  clearCache(): void
  pruneCache(): number
  warmCache(commonTasks: string[]): Promise<void>
}
```

#### Types

```typescript
interface Skill {
  id?: number;
  name: string;
  description?: string;
  signature?: {
    inputs: Record<string, any>;
    outputs: Record<string, any>;
  };
  code?: string;
  successRate: number;
  uses?: number;
  avgReward?: number;
  avgLatencyMs?: number;
  createdFromEpisode?: number;
  metadata?: Record<string, any>;
}

interface SkillLink {
  parentSkillId: number;
  childSkillId: number;
  relationship: 'prerequisite' | 'alternative' | 'refinement' | 'composition';
  weight: number;
  metadata?: Record<string, any>;
}

interface SkillQuery {
  task?: string;              // v2 API
  query?: string;             // v1 API (alias)
  k?: number;
  minSuccessRate?: number;
  preferRecent?: boolean;
}
```

#### Example

```typescript
const skills = new SkillLibrary(db, embedder);

// Create skill manually
const skillId = await skills.createSkill({
  name: 'jwt_authentication',
  description: 'Generate and validate JWT tokens with refresh flow',
  signature: {
    inputs: { userId: 'string', permissions: 'array' },
    outputs: { accessToken: 'string', refreshToken: 'string' }
  },
  code: 'implementation code...',
  successRate: 0.92
});

// Search for applicable skills
const applicable = await skills.searchSkills({
  task: 'user authentication with tokens',
  k: 5,
  minSuccessRate: 0.7
});

// Auto-consolidate from successful episodes
const consolidated = await skills.consolidateEpisodesIntoSkills({
  minAttempts: 3,      // Need 3+ successful executions
  minSuccessRate: 0.7, // With 70%+ success rate
  lookbackDays: 7      // In the last 7 days
});

console.log(`Created ${consolidated.created} new skills`);
console.log(`Updated ${consolidated.updated} existing skills`);
```

---

### 4. CausalMemoryGraph - Intervention-Based Causality

**Purpose:** Learn what interventions cause what outcomes, not just correlations. Track `p(y|do(x))` using doubly robust estimation and instrumental variables.

#### API

```typescript
class CausalMemoryGraph {
  constructor(db: IDatabaseConnection)

  // Create causal experiment (A/B test)
  createExperiment(config: {
    name: string;
    hypothesis: string;
    treatmentId: number;
    treatmentType: string;
    controlId: number;
    startTime: number;
    sampleSize: number;
    status: string;
  }): number

  // Record observations
  recordObservation(obs: {
    experimentId: number;
    episodeId: number;
    isTreatment: boolean;
    outcomeValue: number;
    outcomeType: string;
  }): void

  // Calculate causal uplift
  calculateUplift(experimentId: number): {
    uplift: number;
    pValue: number;
    confidenceInterval: [number, number];
  }

  // Add causal edge
  addCausalEdge(edge: {
    fromMemoryId: number;
    fromMemoryType: string;
    toMemoryId: number;
    toMemoryType: string;
    similarity: number;
    uplift: number;
    confidence: number;
    sampleSize: number;
  }): number

  // Query causal effects
  queryCausalEffects(query: {
    interventionMemoryId: number;
    interventionMemoryType: string;
    minConfidence: number;
    minUplift: number;
  }): Array<any>
}
```

#### Example

```typescript
import { CausalMemoryGraph } from 'agentdb/controllers/CausalMemoryGraph';

const causalGraph = new CausalMemoryGraph(db);

// Create experiment
const experimentId = causalGraph.createExperiment({
  name: 'test_error_handling_approach',
  hypothesis: 'Try-catch reduces crash rate',
  treatmentId: 123,  // Episode with error handling
  treatmentType: 'episode',
  controlId: 124,    // Episode without
  startTime: Date.now(),
  sampleSize: 0,
  status: 'running'
});

// Record observations
causalGraph.recordObservation({
  experimentId,
  episodeId: 123,
  isTreatment: true,
  outcomeValue: 0.95,  // Success rate
  outcomeType: 'success'
});

// Calculate causal uplift
const { uplift, pValue, confidenceInterval } =
  causalGraph.calculateUplift(experimentId);

console.log(`Causal uplift: ${uplift}`);
console.log(`p-value: ${pValue}`);
console.log(`95% CI: [${confidenceInterval[0]}, ${confidenceInterval[1]}]`);
```

---

### 5. CausalRecall - Utility-Based Reranking

**Purpose:** Retrieve what actually works, not just what's similar. Reranks results by utility: `U = α·similarity + β·uplift − γ·latency`

#### API

```typescript
class CausalRecall {
  constructor(
    db: IDatabaseConnection,
    embedder: EmbeddingService,
    vectorBackend: VectorBackend,
    config?: {
      alpha?: number;  // Similarity weight (default: 0.7)
      beta?: number;   // Uplift weight (default: 0.2)
      gamma?: number;  // Latency penalty (default: 0.1)
    }
  )

  // Retrieve with utility ranking
  recall(
    queryId: string,
    queryText: string,
    k: number,
    requirements?: string[],
    accessLevel?: string
  ): Promise<{
    candidates: Array<{
      id: number;
      similarity: number;
      uplift?: number;
      latencyMs: number;
      utilityScore: number;
    }>;
    certificate: ProvencanceCertificate;
  }>
}
```

#### Example

```typescript
import { CausalRecall } from 'agentdb/controllers/CausalRecall';

const causalRecall = new CausalRecall(db, embedder, vectorBackend, {
  alpha: 0.7,  // Similarity weight
  beta: 0.2,   // Causal uplift weight
  gamma: 0.1   // Latency penalty
});

// Retrieve with utility ranking
const result = await causalRecall.recall(
  'query-456',
  'Optimize database query performance',
  10,
  undefined,
  'internal'
);

// Results ranked by utility, not just similarity
result.candidates.forEach((candidate, i) => {
  console.log(`${i + 1}. Utility: ${candidate.utilityScore.toFixed(3)}`);
  console.log(`   Similarity: ${candidate.similarity.toFixed(3)}`);
  console.log(`   Uplift: ${candidate.uplift?.toFixed(3) || 'N/A'}`);
});
```

---

### 6. NightlyLearner - Automated Pattern Discovery

**Purpose:** Background process that discovers patterns automatically. Runs automated causal discovery on episode history.

#### API

```typescript
class NightlyLearner {
  constructor(db: IDatabaseConnection, embedder: EmbeddingService)

  // Discover patterns (dry-run first)
  discover(config: {
    minAttempts: number;      // Min attempts to detect pattern
    minSuccessRate: number;   // Min success rate
    minConfidence: number;    // Statistical confidence
    dryRun: boolean;          // Preview without saving
  }): Promise<Array<CausalEdge>>

  // Prune low-quality edges
  pruneEdges(config: {
    minConfidence: number;
    minUplift: number;
    maxAgeDays: number;
  }): Promise<number>
}
```

#### Example

```typescript
import { NightlyLearner } from 'agentdb/controllers/NightlyLearner';

const learner = new NightlyLearner(db, embedder);

// Discover patterns (dry-run first)
const discovered = await learner.discover({
  minAttempts: 3,
  minSuccessRate: 0.6,
  minConfidence: 0.7,
  dryRun: true
});

console.log(`Would create ${discovered.length} causal edges`);

// Run for real
const created = await learner.discover({
  minAttempts: 3,
  minSuccessRate: 0.6,
  minConfidence: 0.7,
  dryRun: false
});

console.log(`Created ${created.length} causal edges`);
```

---

### 7. BatchOperations - 3-4x Faster Bulk Operations

**Purpose:** Process multiple items efficiently with parallel embeddings and SQL transactions.

#### API

```typescript
class BatchOperations {
  constructor(
    db: IDatabaseConnection,
    embedder: EmbeddingService,
    config?: {
      batchSize?: number;      // Items per batch (default: 100)
      parallelism?: number;    // Concurrent embeddings (default: 4)
      progressCallback?: (completed: number, total: number) => void;
    }
  )

  // Batch insert skills (3x faster)
  insertSkills(skills: Skill[]): Promise<number[]>

  // Batch insert patterns (4x faster)
  insertPatterns(patterns: ReasoningPattern[]): Promise<number[]>

  // Batch insert episodes (3.3x faster)
  insertEpisodes(episodes: Episode[]): Promise<number>

  // Prune old data
  pruneData(config: {
    maxAge?: number;           // Days to keep
    minReward?: number;        // Min reward threshold
    minSuccessRate?: number;   // Min success rate
    maxRecords?: number;       // Max records per table
    dryRun?: boolean;          // Preview mode
  }): Promise<{
    episodesPruned: number;
    skillsPruned: number;
    patternsPruned: number;
    spaceSaved: number;
  }>
}
```

#### Example

```typescript
import { BatchOperations } from 'agentdb';

const batchOps = new BatchOperations(db, embedder, {
  batchSize: 100,
  parallelism: 4,
  progressCallback: (completed, total) => {
    console.log(`Progress: ${completed}/${total}`);
  }
});

// Batch create skills (3x faster)
const skillIds = await batchOps.insertSkills([
  { name: 'skill-1', description: 'First skill', successRate: 0.8 },
  { name: 'skill-2', description: 'Second skill', successRate: 0.9 },
  // ... 50 more
]);

// Prune old data
const results = await batchOps.pruneData({
  maxAge: 90,           // Keep last 90 days
  minReward: 0.3,       // Keep episodes with reward >= 0.3
  minSuccessRate: 0.5,  // Keep skills/patterns >= 50% success
  maxRecords: 100000,   // Max 100k per table
  dryRun: false
});

console.log(`Pruned ${results.episodesPruned} episodes`);
console.log(`Saved ${results.spaceSaved} bytes`);
```

---

## Memory Patterns

AgentDB implements six cognitive memory patterns inspired by how humans learn:

### 1. Reflexion Memory (Episodic)
- **What:** Store complete task episodes with self-critiques
- **When:** Learn from successes and failures
- **How:** Semantic search for similar past experiences

### 2. Skill Library (Procedural)
- **What:** Reusable, parameterized skills from successful patterns
- **When:** Build institutional knowledge over time
- **How:** Auto-consolidation from high-reward episodes

### 3. Causal Memory (Causal)
- **What:** Track interventions and their effects
- **When:** Understand what actions lead to outcomes
- **How:** Doubly robust estimation, A/B testing

### 4. Reasoning Bank (Semantic)
- **What:** Store successful reasoning patterns
- **When:** Find approaches that worked before
- **How:** Pattern matching with success rates

### 5. Explainable Recall (Provenance)
- **What:** Cryptographic proofs for why memories were selected
- **When:** Debug agent decision-making, build trust
- **How:** Merkle proofs, completeness scores

### 6. Nightly Learner (Meta-Learning)
- **What:** Automated pattern discovery
- **When:** Background learning while you sleep
- **How:** Causal discovery on episode history

---

## Performance Benchmarks

### Core Operations (v2.0)

```
Pattern search:     32.6M ops/sec  (ultra-fast with caching)
Pattern storage:    388K ops/sec   (excellent)
Episode retrieval:  957 ops/sec    (very good)
Skill search:       694 ops/sec    (very good)
Skill creation:     304 → 900 ops/sec (with batch)
Episode storage:    152 → 500 ops/sec (with batch)
```

### Vector Search Performance

```
RuVector (Rust+SIMD):
  - 61μs p50 latency
  - 96.8% recall@10
  - 8.2x faster than hnswlib
  - 150x faster than cloud alternatives

HNSW Configuration:
  - M=32 optimal (empirically validated)
  - efConstruction=200
  - efSearch=100
```

### Batch Operations

```
Skills:    304 → 900 ops/sec    (3x faster)
Patterns:  4x faster than sequential
Episodes:  152 → 500 ops/sec   (3.3x faster)
```

### Caching Impact

```
agentdb_stats:     176ms → ~20ms  (8.8x faster)
pattern_stats:     Similar improvement
learning_metrics:  120s TTL for expensive ops
Hit rates:         80%+ for frequent data
```

### Memory Efficiency

```
5,000 patterns: 4MB memory (0.8KB per pattern)
Latency:        0.22-0.68ms per pattern
Scaling:        Super-linear (improves with size)
```

---

## Integration Examples

### Example 1: Learning Code Review Agent

```typescript
import {
  createDatabase,
  ReasoningBank,
  ReflexionMemory,
  EmbeddingService
} from 'agentdb';

// Setup
const db = await createDatabase('./code-reviewer.db');
const embedder = new EmbeddingService({ model: 'Xenova/all-MiniLM-L6-v2' });
await embedder.initialize();

const reasoningBank = new ReasoningBank(db, embedder);
const reflexion = new ReflexionMemory(db, embedder);

// Store successful review pattern
await reasoningBank.storePattern({
  taskType: 'code_review',
  approach: 'Security scan → Type safety → Code quality → Performance',
  successRate: 0.94,
  tags: ['security', 'typescript']
});

// Review code and learn from it
const reviewResult = await performCodeReview(codeToReview);

await reflexion.storeEpisode({
  sessionId: 'review-session-1',
  task: 'Review authentication PR',
  reward: reviewResult.issuesFound > 0 ? 0.9 : 0.6,
  success: true,
  critique: 'Found SQL injection vulnerability - security checks work!',
  input: codeToReview,
  output: reviewResult.findings,
  latencyMs: reviewResult.timeMs,
  tokensUsed: reviewResult.tokensUsed
});

// Next time, find similar successful reviews
const similarReviews = await reflexion.retrieveRelevant({
  task: 'authentication code review',
  k: 5,
  onlySuccesses: true
});

console.log(`Found ${similarReviews.length} successful reviews to learn from`);
```

### Example 2: RAG System with Self-Learning

```typescript
import {
  createDatabase,
  ReasoningBank,
  SkillLibrary,
  EmbeddingService
} from 'agentdb';

const db = await createDatabase('./rag-system.db');
const embedder = new EmbeddingService({ model: 'Xenova/all-MiniLM-L6-v2' });
await embedder.initialize();

const reasoningBank = new ReasoningBank(db, embedder);
const skills = new SkillLibrary(db, embedder);

// Store document retrieval pattern
await reasoningBank.storePattern({
  taskType: 'document_retrieval',
  approach: 'Expand query with synonyms → Semantic search → Re-rank by relevance',
  successRate: 0.88,
  tags: ['rag', 'retrieval']
});

// Create reusable query expansion skill
await skills.createSkill({
  name: 'expand_query',
  description: 'Expand user query with domain-specific synonyms',
  signature: {
    inputs: { query: 'string' },
    outputs: { expanded: 'string[]' }
  },
  code: `
    const synonymMap = { 'bug': ['issue', 'defect', 'error'], ... };
    return query.split(' ').flatMap(word => synonymMap[word] || [word]);
  `,
  successRate: 0.92
});

// Search for retrieval patterns
const patterns = await reasoningBank.searchPatterns({
  task: 'find technical documentation',
  k: 10
});

// Apply best pattern
const bestPattern = patterns[0];
console.log(`Using approach: ${bestPattern.approach}`);
```

### Example 3: MCP Integration (Claude Code)

```bash
# One-command setup
claude mcp add agentdb npx agentdb@latest mcp start

# Now Claude Code can:
# - Store reasoning patterns automatically
# - Search 32.6M patterns/sec for relevant approaches
# - Learn from successful task completions
# - Build reusable skills over time
```

**Manual Setup** (`~/.config/claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "agentdb": {
      "command": "npx",
      "args": ["agentdb@latest", "mcp", "start"],
      "env": { "AGENTDB_PATH": "./agentdb.db" }
    }
  }
}
```

---

## CLI Reference

AgentDB includes 59 CLI commands for database management:

### Initialization

```bash
# Initialize database
agentdb init --db ./agent-memory.db --dimension 384

# With custom model
agentdb init --db ./db.db --dimension 768 --model "Xenova/bge-base-en-v1.5"
```

### Reflexion Memory

```bash
# Store episode
agentdb reflexion store "session-1" "fix_auth_bug" 0.95 true \
  "OAuth2 PKCE worked perfectly" "login failing" "fixed tokens" 1200 500

# Retrieve similar
agentdb reflexion retrieve "authentication issues" 10 0.8

# Get critique summary
agentdb reflexion critique "fix_auth_bug" 10 0.5
```

### Skill Library

```bash
# Create skill
agentdb skill create "jwt_auth" "Generate JWT tokens" \
  '{"inputs": {"user": "object"}}' "code..." 1

# Search skills
agentdb skill search "authentication" 5 0.5

# Auto-consolidate from episodes
agentdb skill consolidate 3 0.7 7

# Update skill stats
agentdb skill update 1 1 0.95 true 1200
```

### Nightly Learner

```bash
# Discover patterns (dry-run)
agentdb learner run 3 0.6 0.7 true

# Create patterns for real
agentdb learner run 3 0.6 0.7 false

# Prune low-quality edges
agentdb learner prune 0.5 0.05 90
```

### Data Management

```bash
# Prune old data (preview)
agentdb prune --max-age 90 --min-reward 0.3 --dry-run

# Actually prune
agentdb prune --max-age 90 --min-reward 0.3 --min-success-rate 0.5
```

### Simulations

```bash
# Test HNSW optimization
agentdb simulate hnsw --iterations 3

# Test GNN attention
agentdb simulate attention --iterations 3

# Test self-healing
agentdb simulate self-organizing --days 30

# Interactive wizard
agentdb simulate --wizard
```

---

## MCP Tools (32 Total)

AgentDB provides 32 MCP tools for zero-code integration:

### Core Vector DB (5 tools)
- `agentdb_init` - Initialize database
- `agentdb_insert` - Insert single vector
- `agentdb_insert_batch` - Batch insert (141x faster)
- `agentdb_search` - Semantic k-NN search
- `agentdb_delete` - Delete vectors

### Core AgentDB (5 tools)
- `agentdb_stats` - Database statistics (8.8x faster cached)
- `agentdb_pattern_store` - Store reasoning patterns
- `agentdb_pattern_search` - Search patterns (32.6M ops/sec)
- `agentdb_pattern_stats` - Pattern analytics
- `agentdb_clear_cache` - Cache management

### Frontier Memory (9 tools)
- `reflexion_store` - Store episode with critique
- `reflexion_retrieve` - Retrieve similar episodes
- `skill_create` - Create reusable skill
- `skill_search` - Search for applicable skills
- `causal_add_edge` - Add causal relationship
- `causal_query` - Query causal effects
- `recall_with_certificate` - Utility-based retrieval
- `learner_discover` - Automated pattern discovery
- `db_stats` - Database statistics

### Learning System (10 tools)
- `learning_start_session` - Start RL session
- `learning_end_session` - End & save policy
- `learning_predict` - AI recommendations
- `learning_feedback` - Submit action feedback
- `learning_train` - Batch policy training
- `learning_metrics` - Performance analytics
- `learning_transfer` - Transfer learning
- `learning_explain` - Explainable AI
- `experience_record` - Record tool execution
- `reward_signal` - Calculate rewards

**Supported RL Algorithms:** Q-Learning, SARSA, DQN, Policy Gradient, Actor-Critic, PPO, Decision Transformer, MCTS, Model-Based

---

## Architecture

### Multi-Backend System

```
┌─────────────────────────────────────────────────────────┐
│                   AgentDB v2.0 Core                      │
├─────────────────────────────────────────────────────────┤
│  Frontier Memory:                                        │
│  • ReasoningBank    • Reflexion Memory                   │
│  • Skill Library    • Causal Memory Graph                │
│  • Causal Recall    • Nightly Learner                    │
├─────────────────────────────────────────────────────────┤
│  Optimizations:                                          │
│  • BatchOperations  • ToolCache (LRU + TTL)              │
│  • Enhanced Validation                                   │
├─────────────────────────────────────────────────────────┤
│  Backend Auto-Selection (fastest → most compatible):     │
│  RuVector → HNSWLib → better-sqlite3 → sql.js (WASM)     │
└─────────────────────────────────────────────────────────┘
         ↓                    ↓                    ↓
┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐
│   RuVector      │  │    HNSWLib      │  │   SQLite     │
│  Rust + SIMD    │  │   C++ HNSW      │  │  better-sql3 │
│  150x faster    │  │   100x faster   │  │  Native Node │
│  (optional)     │  │   (optional)    │  │  (optional)  │
└─────────────────┘  └─────────────────┘  └──────────────┘
                                                   ↓
                                          ┌──────────────┐
                                          │  sql.js WASM │
                                          │   Default    │
                                          │  Zero deps   │
                                          └──────────────┘
```

### Data Flow

```
User Input
   ↓
Input Validation (XSS/injection detection)
   ↓
ToolCache Check (LRU + TTL)
   ├── Cache Hit → Return cached result (8.8x faster)
   └── Cache Miss → Continue
       ↓
   Embedding Service
   (Transformers.js or mock)
       ↓
   Vector Backend
   (Auto-selected: RuVector → HNSWLib → SQLite)
       ↓
   Frontier Memory Layer
   (ReasoningBank, Reflexion, Skills, Causal)
       ↓
   Result + Provenance Certificate
       ↓
   Cache Result (with TTL)
       ↓
   Return to User
```

---

## Advanced Topics

### GNN Learning Backend

AgentDB v2 includes optional Graph Neural Network support for query enhancement:

```typescript
interface LearningBackend {
  // Enhance query using GNN and neighbor context
  enhance(
    query: Float32Array,
    neighbors: Float32Array[],
    weights: number[]
  ): Float32Array;

  // Add training sample
  addSample(embedding: Float32Array, success: boolean): void;

  // Train GNN model
  train(options?: {
    epochs?: number;
    batchSize?: number
  }): Promise<{ epochs: number; finalLoss: number }>;
}
```

**Benefits:**
- +12.4% recall improvement
- 3.8ms forward pass
- 91% transferability
- Adaptive query refinement

### Graph Backend Integration

```typescript
interface GraphBackend {
  // Execute Cypher query
  executeCypher(query: string, params?: Record<string, any>): Promise<any[]>;

  // Create node
  createNode(labels: string[], properties: Record<string, any>): Promise<number>;

  // Create relationship
  createRelationship(
    fromId: number,
    toId: number,
    type: string,
    properties?: Record<string, any>
  ): Promise<void>;

  // Get statistics
  getStats(): GraphStats;
}
```

**Use Cases:**
- Episode relationships
- Skill composition graphs
- Causal chains
- Learning trajectories

### Security Features

AgentDB includes comprehensive input validation:

```typescript
import {
  validateTaskString,
  validateNumericRange,
  validateArrayLength,
  validateObject,
  validateBoolean,
  validateEnum,
  ValidationError
} from 'agentdb';

try {
  // String validation (length + XSS detection)
  const task = validateTaskString(userInput, 'task');

  // Numeric range
  const k = validateNumericRange(kValue, 'k', 1, 100);

  // Array length
  const items = validateArrayLength(array, 'items', 1, 100);

  // Enum validation
  const format = validateEnum(formatValue, 'format', ['concise', 'detailed', 'json']);
} catch (error) {
  if (error instanceof ValidationError) {
    console.error(`Validation error: ${error.message}`);
  }
}
```

**Security Features:**
- XSS detection (`<script>`, `javascript:`, `onclick=`)
- Injection detection (null bytes, malicious patterns)
- Length limits (10k characters max)
- Type validation with TypeScript types
- Safe error messages (no sensitive data leakage)

---

## Migration from v1.x

AgentDB v2.0 is 100% backward compatible with v1.x APIs:

### v1 (Legacy Mode)
```typescript
// Still works in v2.0
const reasoningBank = new ReasoningBank(db, embedder);
await reasoningBank.storePattern(pattern);
const results = await reasoningBank.searchPatterns({ task: 'query' });
```

### v2 (New Mode with VectorBackend)
```typescript
// Enhanced with VectorBackend
const reasoningBank = new ReasoningBank(db, embedder, vectorBackend, learningBackend);
await reasoningBank.storePattern(pattern);
const results = await reasoningBank.searchPatterns({
  task: 'query',
  useGNN: true  // New feature
});

// Record outcome for learning
await reasoningBank.recordOutcome(patternId, true, 0.95);
```

### Migration Performance
- **173x faster migration** (48ms vs 8.3s for 10K vectors)
- **Zero regressions** (100% backward compatibility)
- **Automatic backend selection** (no config needed)

---

## Best Practices

### 1. Choose Right Memory Pattern

| Use Case | Pattern |
|----------|---------|
| Learn from past attempts | ReflexionMemory |
| Build reusable skills | SkillLibrary |
| Find successful approaches | ReasoningBank |
| Understand causality | CausalMemoryGraph |
| Debug decisions | ExplainableRecall |
| Background learning | NightlyLearner |

### 2. Optimize Performance

```typescript
// Use batch operations for bulk inserts
const batchOps = new BatchOperations(db, embedder);
await batchOps.insertSkills(manySkills);  // 3x faster

// Enable caching for frequent queries
const reflexion = new ReflexionMemory(db, embedder, vectorBackend, learningBackend, graphBackend, {
  maxSize: 1000,
  ttl: 60000  // 60 seconds
});

// Warm cache for common queries
await reflexion.warmCache('session-id');
```

### 3. Maintain Database Hygiene

```typescript
// Prune old episodes
reflexion.pruneEpisodes({
  minReward: 0.3,
  maxAgeDays: 90,
  keepMinPerTask: 5
});

// Prune underperforming skills
skills.pruneSkills({
  minUses: 3,
  minSuccessRate: 0.5,
  maxAgeDays: 90
});

// Prune low-quality causal edges
await learner.pruneEdges({
  minConfidence: 0.5,
  minUplift: 0.05,
  maxAgeDays: 90
});
```

### 4. Monitor Performance

```typescript
// Get statistics
const stats = reasoningBank.getPatternStats();
console.log(`Total patterns: ${stats.totalPatterns}`);
console.log(`Avg success rate: ${stats.avgSuccessRate}`);

// Cache statistics
const cacheStats = reflexion.getCacheStats();
console.log(`Hit rate: ${(cacheStats.hitRate * 100).toFixed(1)}%`);

// Learning statistics
const learningStats = reflexion.getLearningStats();
if (learningStats) {
  console.log(`Samples: ${learningStats.totalSamples}`);
}
```

---

## Troubleshooting

### Common Issues

#### 1. Database Not Initialized

**Error:** `Table 'reasoning_patterns' does not exist`

**Solution:**
```bash
# Initialize via CLI first
npx agentdb@alpha init --db ./my-database.db --dimension 384
```

#### 2. Embedding Service Fails

**Error:** `Failed to load model`

**Solution:**
```typescript
// Use mock embeddings for testing
const embedder = new EmbeddingService({
  provider: 'mock',
  dimension: 384
});
```

#### 3. Slow Performance

**Check backend:**
```typescript
// Verify backend selection
console.log('Backend:', vectorBackend?.constructor.name);

// Enable caching
reasoningBank.clearCache();  // Clear stale cache

// Use batch operations
const batchOps = new BatchOperations(db, embedder);
```

#### 4. Memory Usage High

**Solution:**
```typescript
// Prune old data
await batchOps.pruneData({
  maxAge: 90,
  minReward: 0.3,
  maxRecords: 100000
});

// Clear cache
reflexion.clearCache();
skills.clearCache();
```

---

## Resources

### Documentation
- **GitHub:** https://github.com/ruvnet/agentic-flow/tree/main/packages/agentdb
- **NPM:** https://www.npmjs.com/package/agentdb
- **Homepage:** https://agentdb.ruv.io

### Research Papers
- **Reflexion:** [Language Agents with Verbal Reinforcement Learning](https://arxiv.org/abs/2303.11366)
- **Voyager:** [Open-Ended Embodied Agent with LLMs](https://arxiv.org/abs/2305.16291)
- **HNSW:** [Efficient and Robust ANN Search](https://arxiv.org/abs/1603.09320)
- **Decision Transformer:** [Reinforcement Learning via Sequence Modeling](https://arxiv.org/abs/2106.01345)

### Community
- **Issues:** https://github.com/ruvnet/agentic-flow/issues
- **Author:** ruv (ruv@ruv.net)

---

## License

MIT License

Copyright (c) 2025 ruv

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

---

**Last Updated:** 2025-12-07
**AgentDB Version:** 2.0.0-alpha.2.20
**Status:** Alpha (Early Adopters)
