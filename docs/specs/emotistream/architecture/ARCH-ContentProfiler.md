# EmotiStream Nexus - ContentProfiler Module Architecture

**Version**: 1.0
**Created**: 2025-12-05
**SPARC Phase**: Architecture (Phase 3)
**Component**: ContentProfiler Module
**Dependencies**: Gemini API, AgentDB, RuVector, MVP-003 Requirements

---

## 1. Executive Summary

The **ContentProfiler** module is responsible for analyzing content metadata using the Gemini API to generate emotional profiles and storing them as 1536-dimensional embeddings in RuVector for semantic search. This module enables the RL recommendation engine to find content that matches desired emotional transitions.

### 1.1 Core Responsibilities

- **Batch Content Profiling**: Process 200+ content items via Gemini API with rate limiting
- **Embedding Generation**: Create 1536D vectors encoding emotional characteristics
- **Vector Storage**: Store embeddings in RuVector with HNSW indexing
- **Semantic Search**: Find content matching emotional transitions
- **Mock Catalog**: Generate diverse test content across 6 categories

### 1.2 Key Architectural Decisions

| Decision | Rationale |
|----------|-----------|
| **Batch Processing (10 items/batch)** | Balances throughput with error isolation and rate limits |
| **1536D Embeddings** | Matches industry standard (OpenAI), allows rich encoding |
| **HNSW Index (M=16, efConstruction=200)** | Optimal balance of build time and search quality |
| **Gaussian Encoding** | Smooth transitions in emotion space for better similarity |
| **AgentDB + RuVector Dual Storage** | AgentDB for metadata, RuVector for semantic search |

---

## 2. Module Structure

### 2.1 Directory Layout

```
src/content/
├── index.ts                    # Public API exports
├── profiler.ts                 # ContentProfiler class (main orchestrator)
├── batch-processor.ts          # Batch processing with rate limiting
├── embedding-generator.ts      # 1536D embedding generation
├── ruvector-client.ts          # RuVector HNSW integration
├── gemini-client.ts            # Gemini API wrapper with retry logic
├── agentdb-store.ts            # AgentDB persistence layer
├── mock-catalog.ts             # Mock content generator (200 items)
├── search.ts                   # Semantic search by transition
├── types.ts                    # Module-specific TypeScript types
└── __tests__/
    ├── profiler.test.ts
    ├── embedding-generator.test.ts
    ├── batch-processor.test.ts
    └── search.test.ts
```

### 2.2 Public API Exports

```typescript
// src/content/index.ts
export { ContentProfiler } from './profiler';
export { RuVectorClient } from './ruvector-client';
export { generateMockCatalog } from './mock-catalog';
export {
  ContentMetadata,
  EmotionalContentProfile,
  SearchResult,
  ProfileResult,
  TargetState
} from './types';
```

---

## 3. Class Diagrams (ASCII)

### 3.1 ContentProfiler (Main Orchestrator)

```
┌─────────────────────────────────────────────────────────────────┐
│                      ContentProfiler                             │
├─────────────────────────────────────────────────────────────────┤
│ - geminiClient: GeminiClient                                     │
│ - embeddingGenerator: EmbeddingGenerator                         │
│ - ruVectorClient: RuVectorClient                                 │
│ - agentDBStore: AgentDBStore                                     │
│ - batchProcessor: BatchProcessor                                 │
├─────────────────────────────────────────────────────────────────┤
│ + constructor(config: ProfilerConfig)                            │
│ + profileContent(content: ContentMetadata): Promise<Profile>     │
│ + batchProfile(contents[], batchSize?): Promise<ProfileResult>   │
│ + getContentProfile(contentId: string): Promise<Profile | null>  │
│ + searchByTransition(current, desired, topK?): Promise<Result[]> │
│ - processSingleContent(content): Promise<ProcessResult>          │
│ - validateProfile(profile: Profile): boolean                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ uses
                              │
        ┌─────────────────────┼──────────────────────┐
        ▼                     ▼                      ▼
┌───────────────┐    ┌──────────────────┐   ┌─────────────────┐
│ GeminiClient  │    │ EmbeddingGenerator│   │ RuVectorClient  │
├───────────────┤    ├──────────────────┤   ├─────────────────┤
│ + analyze()   │    │ + generate()     │   │ + upsert()      │
│ + retry()     │    │ + encode()       │   │ + search()      │
│ + timeout()   │    │ + normalize()    │   │ + getCollection()│
└───────────────┘    └──────────────────┘   └─────────────────┘
```

### 3.2 RuVectorClient (HNSW Integration)

```
┌─────────────────────────────────────────────────────────────────┐
│                       RuVectorClient                             │
├─────────────────────────────────────────────────────────────────┤
│ - collectionName: string = "content_embeddings"                  │
│ - dimension: number = 1536                                       │
│ - indexConfig: HNSWConfig { m: 16, efConstruction: 200 }        │
│ - metric: 'cosine'                                               │
├─────────────────────────────────────────────────────────────────┤
│ + getOrCreateCollection(): Promise<Collection>                   │
│ + upsert(id, embedding, metadata): Promise<string>               │
│ + search(query, topK): Promise<SearchHit[]>                      │
│ + delete(id: string): Promise<boolean>                           │
│ + count(): Promise<number>                                       │
│ - ensureCollection(): Promise<void>                              │
│ - validateEmbedding(embedding: Float32Array): boolean            │
└─────────────────────────────────────────────────────────────────┘
```

### 3.3 EmbeddingGenerator (1536D Encoding)

```
┌─────────────────────────────────────────────────────────────────┐
│                     EmbeddingGenerator                           │
├─────────────────────────────────────────────────────────────────┤
│ - dimensions: number = 1536                                      │
│ - toneIndexMap: Map<string, number>                              │
│ - genreIndexMap: Map<string, number>                             │
│ - categoryIndexMap: Map<string, number>                          │
├─────────────────────────────────────────────────────────────────┤
│ + generate(profile, content): Promise<Float32Array>              │
│ - encodePrimaryTone(embedding, tone, offset): void               │
│ - encodeValenceArousal(embedding, v, a, offset): void            │
│ - encodeIntensityComplexity(embedding, i, c, offset): void       │
│ - encodeTargetStates(embedding, states, offset): void            │
│ - encodeGenresCategory(embedding, genres, cat, offset): void     │
│ - encodeRangeValue(embedding, start, end, value, min, max): void│
│ - normalizeVector(vector: Float32Array): Float32Array            │
│ - getToneIndex(tone: string): number                             │
│ - getGenreIndex(genre: string): number                           │
│ - getCategoryIndex(category: string): number                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## 4. TypeScript Interfaces

### 4.1 Core Data Types

```typescript
// src/content/types.ts

/**
 * Content metadata from mock catalog or external sources
 */
export interface ContentMetadata {
  contentId: string;
  title: string;
  description: string;
  platform: 'mock'; // MVP uses mock catalog only
  genres: string[];
  category: 'movie' | 'series' | 'documentary' | 'music' | 'meditation' | 'short';
  tags: string[];
  duration: number; // minutes
}

/**
 * Emotional profile generated by Gemini + embedding
 */
export interface EmotionalContentProfile {
  contentId: string;

  // Emotional characteristics
  primaryTone: string;      // 'uplifting', 'calming', 'thrilling', etc.
  valenceDelta: number;     // Expected change in valence (-1 to +1)
  arousalDelta: number;     // Expected change in arousal (-1 to +1)
  intensity: number;        // Emotional intensity (0 to 1)
  complexity: number;       // Emotional complexity (0 to 1)

  // Target states (when is this content effective?)
  targetStates: TargetState[];

  // RuVector embedding reference
  embeddingId: string;

  timestamp: number;
}

/**
 * Target viewer states for content
 */
export interface TargetState {
  currentValence: number;   // -1 to +1
  currentArousal: number;   // -1 to +1
  description: string;      // Human-readable description
}

/**
 * Search result from semantic search
 */
export interface SearchResult {
  contentId: string;
  title: string;
  similarityScore: number;  // 0 to 1 (cosine similarity)
  profile: EmotionalContentProfile;
  metadata: ContentMetadata;
  relevanceReason: string;  // Why this content matches the transition
}

/**
 * Batch processing result
 */
export interface ProfileResult {
  success: number;          // Successfully profiled items
  failed: number;           // Failed items
  errors: Array<{
    contentId: string;
    error: string;
    timestamp: number;
  }>;
  duration: number;         // Total processing time (ms)
}

/**
 * Single item processing result (internal)
 */
export interface ProcessResult {
  success: boolean;
  contentId: string;
  error: string | null;
  profile?: EmotionalContentProfile;
}

/**
 * RuVector entry format
 */
export interface RuVectorEntry {
  id: string;               // contentId
  embedding: Float32Array;  // 1536D vector
  metadata: {
    contentId: string;
    title: string;
    primaryTone: string;
    valenceDelta: number;
    arousalDelta: number;
    intensity: number;
    complexity: number;
    genres: string[];
    category: string;
    duration: number;
    tags: string[];
    platform: string;
    timestamp: number;
  };
}

/**
 * Configuration for ContentProfiler
 */
export interface ProfilerConfig {
  geminiApiKey: string;
  geminiModel?: string;     // Default: 'gemini-1.5-flash'
  geminiTimeout?: number;   // Default: 30000ms
  batchSize?: number;       // Default: 10
  maxRetries?: number;      // Default: 3
  retryDelay?: number;      // Default: 2000ms
  ruvectorUrl: string;
  agentdbUrl: string;
  memoryNamespace?: string; // Default: 'emotistream/content-profiler'
}
```

### 4.2 Profiler Interface

```typescript
/**
 * Main ContentProfiler interface
 */
export interface IContentProfiler {
  /**
   * Profile a single content item
   */
  profileContent(content: ContentMetadata): Promise<EmotionalContentProfile>;

  /**
   * Batch profile multiple content items with rate limiting
   * @param contents - Array of content metadata
   * @param batchSize - Items per batch (default: 10)
   * @returns ProfileResult with success/failure counts
   */
  batchProfile(
    contents: ContentMetadata[],
    batchSize?: number
  ): Promise<ProfileResult>;

  /**
   * Retrieve stored profile for a content item
   */
  getContentProfile(contentId: string): Promise<EmotionalContentProfile | null>;

  /**
   * Search for content matching an emotional transition
   * @param currentState - User's current emotional state
   * @param desiredState - User's desired emotional state
   * @param topK - Number of results to return (default: 10)
   * @returns Array of SearchResult ordered by relevance
   */
  searchByTransition(
    currentState: EmotionalState,
    desiredState: DesiredState,
    topK?: number
  ): Promise<SearchResult[]>;
}

/**
 * Emotional state (from EmotionalStateTracker module)
 */
export interface EmotionalState {
  valence: number;          // -1 to +1
  arousal: number;          // -1 to +1
  primaryEmotion: string;
  stressLevel: number;      // 0 to 1
  confidence: number;       // 0 to 1
  timestamp: number;
}

/**
 * Desired state (from EmotionalStateTracker module)
 */
export interface DesiredState {
  valence: number;          // -1 to +1
  arousal: number;          // -1 to +1
  confidence: number;       // 0 to 1
  reasoning: string;
}
```

---

## 5. Sequence Diagrams (ASCII)

### 5.1 Batch Profiling Flow

```
User/CLI      BatchProcessor     GeminiClient     EmbeddingGen     RuVector     AgentDB
   │                │                  │                │             │            │
   │─batchProfile()─▶│                 │                │             │            │
   │                │                  │                │             │            │
   │                │─splitIntoBatches()                │             │            │
   │                │◀────────────────┘                 │             │            │
   │                │                  │                │             │            │
   │          ┌─────┤                  │                │             │            │
   │          │ For each batch (parallel within batch) │             │            │
   │          │     │                  │                │             │            │
   │          │     │─analyze(content)─▶│               │             │            │
   │          │     │                  │               │             │            │
   │          │     │                  │ [Gemini API call]           │            │
   │          │     │                  │ [Timeout: 30s]              │            │
   │          │     │◀─────profile─────┤               │             │            │
   │          │     │                  │                │             │            │
   │          │     │─generate(profile, content)────────▶│            │            │
   │          │     │                  │                │            │            │
   │          │     │◀────embedding────────────────────┤            │            │
   │          │     │                  │                │             │            │
   │          │     │─upsert(id, embedding, metadata)───────────────▶│           │
   │          │     │                  │                │             │            │
   │          │     │◀────embeddingId──────────────────────────────┤           │
   │          │     │                  │                │             │            │
   │          │     │─store(profile)────────────────────────────────────────────▶│
   │          │     │                  │                │             │            │
   │          │     │◀─────success──────────────────────────────────────────────┤
   │          │     │                  │                │             │            │
   │          └─────┤                  │                │             │            │
   │                │                  │                │             │            │
   │                │─rateLimitDelay()─│                │             │            │
   │                │  [Sleep 60s/batch]                │             │            │
   │                │                  │                │             │            │
   │◀───ProfileResult──────────────────┘                │             │            │
   │  {success, failed, errors}        │                │             │            │
```

### 5.2 Semantic Search Flow

```
User/RL      ContentProfiler    EmbeddingGen      RuVector      AgentDB
   │                │                 │                │            │
   │─searchByTransition(current, desired, topK=10)──▶ │            │
   │                │                 │                │            │
   │                │─createTransitionVector()─────────▶│           │
   │                │  (encode current→desired delta)  │           │
   │                │                 │                │            │
   │                │◀────queryVector────────────────┤            │
   │                │  (Float32Array[1536])           │            │
   │                │                 │                │            │
   │                │─search(queryVector, topK=10)─────▶│           │
   │                │  [HNSW search]                    │           │
   │                │  [O(log n) complexity]            │           │
   │                │                 │                │            │
   │                │◀────SearchHits────────────────────┤           │
   │                │  [{id, score, metadata}...]      │            │
   │                │                 │                │            │
   │          ┌─────┤                 │                │            │
   │          │ For each hit (parallel)                │            │
   │          │     │─getContentProfile(id)─────────────────────────▶│
   │          │     │                 │                │            │
   │          │     │◀────profile───────────────────────────────────┤
   │          │     │                 │                │            │
   │          │     │─explainRelevance(current, desired, profile)   │
   │          │     │                 │                │            │
   │          └─────┤                 │                │            │
   │                │                 │                │            │
   │◀───SearchResults[10]─────────────┘                │            │
   │  [{contentId, similarity, profile, relevanceReason}...]        │
```

### 5.3 Single Content Processing (with Retry)

```
BatchProcessor   GeminiClient    EmbeddingGen    RuVector    AgentDB
      │                │               │              │          │
      │─processSingleContent(content)──▶│             │          │
      │                │               │              │          │
      │          ┌─────┤               │              │          │
      │          │ Retry loop (max 3 attempts)       │          │
      │          │     │─analyze(content)────────────▶│         │
      │          │     │  [HTTP POST to Gemini]      │          │
      │          │     │  [30s timeout]              │          │
      │          │     │               │              │          │
      │          │     │ [If timeout/error]          │          │
      │          │     │  [Sleep 2s * retryCount]    │          │
      │          │     │  [Exponential backoff]      │          │
      │          │     │               │              │          │
      │          │     │◀─profile──────┘              │          │
      │          │     │  {primaryTone, deltas...}   │          │
      │          └─────┤               │              │          │
      │                │               │              │          │
      │                │─generate(profile, content)───▶│         │
      │                │               │              │          │
      │                │◀──embedding───────────────────┤         │
      │                │  Float32Array[1536]          │          │
      │                │               │              │          │
      │                │─upsert(id, embedding, metadata)─────────▶│
      │                │               │              │          │
      │                │◀──embeddingId────────────────────────────┤
      │                │               │              │          │
      │                │─store(profile)───────────────────────────▶│
      │                │               │              │          │
      │◀──ProcessResult{success: true, contentId}────────────────┤
```

---

## 6. RuVector Configuration

### 6.1 Collection Setup

```typescript
// src/content/ruvector-client.ts

export interface HNSWConfig {
  m: number;                // Number of bi-directional links per node
  efConstruction: number;   // Size of dynamic candidate list during construction
  efSearch?: number;        // Size of dynamic candidate list during search
}

export const RUVECTOR_CONFIG = {
  collectionName: 'content_embeddings',
  dimension: 1536,
  indexType: 'hnsw' as const,
  indexConfig: {
    m: 16,                  // Good balance: higher = better accuracy, slower build
    efConstruction: 200,    // Higher = better index quality, slower build
    efSearch: 100           // Higher = better search accuracy, slower queries
  },
  metric: 'cosine' as const // Cosine similarity for normalized vectors
};

/**
 * RuVector HNSW Configuration Rationale:
 *
 * **M = 16**:
 * - Each node connects to 16 neighbors in the graph
 * - Provides good balance between recall (~95%) and build time
 * - Lower M (8) would be faster but less accurate
 * - Higher M (32) would be more accurate but 2x slower to build
 *
 * **efConstruction = 200**:
 * - Candidate list size during index building
 * - 200 is recommended for high-quality indices (>90% recall)
 * - Our 200-item catalog can afford this quality investment
 * - Build time: ~30 seconds for 200 items (acceptable for MVP)
 *
 * **efSearch = 100**:
 * - Candidate list size during search queries
 * - 100 provides excellent recall (>95%) for topK=10 searches
 * - Query time: <50ms p95 (meets <3s total latency requirement)
 *
 * **Metric = cosine**:
 * - Cosine similarity for normalized embeddings
 * - Measures angle between vectors (direction, not magnitude)
 * - Ideal for semantic similarity in high-dimensional spaces
 */
```

### 6.2 Index Management

```typescript
export class RuVectorClient {
  private collection: Collection | null = null;

  /**
   * Ensure collection exists with HNSW index
   */
  async getOrCreateCollection(): Promise<Collection> {
    if (this.collection) {
      return this.collection;
    }

    try {
      // Try to get existing collection
      this.collection = await this.ruVector.getCollection(
        RUVECTOR_CONFIG.collectionName
      );

      return this.collection;
    } catch (error) {
      // Collection doesn't exist, create it
      console.log('Creating RuVector collection with HNSW index...');

      this.collection = await this.ruVector.createCollection({
        name: RUVECTOR_CONFIG.collectionName,
        dimension: RUVECTOR_CONFIG.dimension,
        indexType: RUVECTOR_CONFIG.indexType,
        indexConfig: RUVECTOR_CONFIG.indexConfig,
        metric: RUVECTOR_CONFIG.metric
      });

      console.log('✅ RuVector collection created');
      return this.collection;
    }
  }

  /**
   * Upsert embedding with metadata
   */
  async upsert(
    contentId: string,
    embedding: Float32Array,
    metadata: Record<string, any>
  ): Promise<string> {
    const collection = await this.getOrCreateCollection();

    // Validate embedding dimensions
    if (embedding.length !== RUVECTOR_CONFIG.dimension) {
      throw new Error(
        `Invalid embedding dimension: ${embedding.length} (expected ${RUVECTOR_CONFIG.dimension})`
      );
    }

    // Upsert (insert or update)
    const result = await collection.upsert({
      id: contentId,
      embedding: Array.from(embedding), // Convert Float32Array to number[]
      metadata: {
        ...metadata,
        indexedAt: Date.now()
      }
    });

    return result.id;
  }

  /**
   * Search for similar embeddings
   */
  async search(
    queryVector: Float32Array,
    topK: number = 10
  ): Promise<SearchHit[]> {
    const collection = await this.getOrCreateCollection();

    const results = await collection.search({
      vector: Array.from(queryVector),
      topK,
      includeMetadata: true
    });

    return results.map(result => ({
      id: result.id,
      score: result.score,        // Cosine similarity (0 to 1)
      metadata: result.metadata
    }));
  }
}

export interface SearchHit {
  id: string;
  score: number;              // Cosine similarity score (0 to 1)
  metadata: Record<string, any>;
}
```

---

## 7. Embedding Generation (1536D)

### 7.1 Vector Encoding Strategy

```typescript
/**
 * 1536D Embedding Structure:
 *
 * Segment 1 (0-255):     Primary Tone Encoding (256 dimensions)
 * Segment 2 (256-511):   Valence/Arousal Deltas (256 dimensions)
 *   - 256-383: Valence delta (-1 to +1)
 *   - 384-511: Arousal delta (-1 to +1)
 * Segment 3 (512-767):   Intensity/Complexity (256 dimensions)
 *   - 512-639: Intensity (0 to 1)
 *   - 640-767: Complexity (0 to 1)
 * Segment 4 (768-1023):  Target States (256 dimensions)
 *   - 3 target states × 86 dimensions each (valence + arousal)
 * Segment 5 (1024-1279): Genres/Category (256 dimensions)
 *   - 128 genre slots (one-hot)
 *   - 128 category + tag slots
 * Segment 6 (1280-1535): Reserved for Future Use (256 dimensions)
 */

export class EmbeddingGenerator {
  private readonly DIMENSIONS = 1536;

  /**
   * Generate 1536D embedding from emotional profile
   */
  async generate(
    profile: EmotionalContentProfile,
    content: ContentMetadata
  ): Promise<Float32Array> {
    const embedding = new Float32Array(this.DIMENSIONS);
    embedding.fill(0);

    // Segment 1: Primary tone (0-255)
    this.encodePrimaryTone(embedding, profile.primaryTone, 0);

    // Segment 2: Valence/arousal deltas (256-511)
    this.encodeRangeValue(embedding, 256, 383, profile.valenceDelta, -1.0, 1.0);
    this.encodeRangeValue(embedding, 384, 511, profile.arousalDelta, -1.0, 1.0);

    // Segment 3: Intensity/complexity (512-767)
    this.encodeRangeValue(embedding, 512, 639, profile.intensity, 0.0, 1.0);
    this.encodeRangeValue(embedding, 640, 767, profile.complexity, 0.0, 1.0);

    // Segment 4: Target states (768-1023)
    this.encodeTargetStates(embedding, profile.targetStates, 768);

    // Segment 5: Genres/category (1024-1279)
    this.encodeGenresCategory(embedding, content.genres, content.category, 1024);

    // Normalize to unit length (required for cosine similarity)
    return this.normalizeVector(embedding);
  }

  /**
   * Encode continuous value with Gaussian distribution
   * Creates smooth transitions in embedding space
   */
  private encodeRangeValue(
    embedding: Float32Array,
    startIdx: number,
    endIdx: number,
    value: number,
    minValue: number,
    maxValue: number
  ): void {
    // Normalize value to [0, 1]
    const normalized = (value - minValue) / (maxValue - minValue);

    // Gaussian encoding for smooth similarity
    const rangeSize = endIdx - startIdx + 1;
    const center = normalized * rangeSize;
    const sigma = rangeSize / 6.0; // Standard deviation

    for (let i = 0; i < rangeSize; i++) {
      const distance = i - center;
      const gaussianValue = Math.exp(-(distance * distance) / (2 * sigma * sigma));
      embedding[startIdx + i] = gaussianValue;
    }
  }

  /**
   * Normalize vector to unit length
   * Required for cosine similarity to work correctly
   */
  private normalizeVector(vector: Float32Array): Float32Array {
    // Calculate magnitude
    let magnitude = 0;
    for (let i = 0; i < vector.length; i++) {
      magnitude += vector[i] * vector[i];
    }
    magnitude = Math.sqrt(magnitude);

    // Avoid division by zero
    if (magnitude === 0) {
      return vector;
    }

    // Normalize
    const normalized = new Float32Array(vector.length);
    for (let i = 0; i < vector.length; i++) {
      normalized[i] = vector[i] / magnitude;
    }

    return normalized;
  }
}
```

### 7.2 Encoding Examples

```typescript
/**
 * Example 1: Calming Nature Documentary
 *
 * Input:
 *   primaryTone: 'serene'
 *   valenceDelta: +0.3 (slight positive shift)
 *   arousalDelta: -0.5 (significant calming)
 *   intensity: 0.3 (gentle)
 *   complexity: 0.4 (simple)
 *
 * Encoding:
 *   [0-255]: Peak at index 42 (serene tone)
 *   [256-383]: Gaussian centered at ~82 (valence +0.3)
 *   [384-511]: Gaussian centered at ~32 (arousal -0.5)
 *   [512-639]: Gaussian centered at ~38 (intensity 0.3)
 *   [640-767]: Gaussian centered at ~51 (complexity 0.4)
 *   [768-1023]: Target states for stressed users
 *   [1024-1279]: Nature, documentary genres
 */

/**
 * Example 2: Uplifting Comedy
 *
 * Input:
 *   primaryTone: 'uplifting'
 *   valenceDelta: +0.6 (strong positive)
 *   arousalDelta: +0.2 (slight energy boost)
 *   intensity: 0.6 (moderate)
 *   complexity: 0.5 (balanced)
 *
 * Encoding:
 *   [0-255]: Peak at index 87 (uplifting tone)
 *   [256-383]: Gaussian centered at ~102 (valence +0.6)
 *   [384-511]: Gaussian centered at ~78 (arousal +0.2)
 *   [512-639]: Gaussian centered at ~77 (intensity 0.6)
 *   [640-767]: Gaussian centered at ~64 (complexity 0.5)
 *   [768-1023]: Target states for sad/low-energy users
 *   [1024-1279]: Comedy, humor genres
 */
```

---

## 8. Mock Catalog Design

### 8.1 Catalog Generator

```typescript
// src/content/mock-catalog.ts

export interface ContentTemplate {
  genres: string[];
  tags: string[];
  minDuration: number;
  maxDuration: number;
  emotionalRanges: {
    valenceDelta: [number, number];
    arousalDelta: [number, number];
    intensity: [number, number];
    complexity: [number, number];
  };
}

/**
 * Generate 200 mock content items across 6 categories
 */
export function generateMockCatalog(count: number = 200): ContentMetadata[] {
  const templates = getContentTemplates();
  const categories = Object.keys(templates) as ContentCategory[];

  const catalog: ContentMetadata[] = [];
  let idCounter = 1;

  // Distribute items across categories
  const itemsPerCategory = Math.floor(count / categories.length);

  for (const category of categories) {
    const template = templates[category];

    for (let i = 0; i < itemsPerCategory; i++) {
      const content: ContentMetadata = {
        contentId: `mock_${category}_${idCounter.toString().padStart(3, '0')}`,
        title: generateTitle(category, idCounter),
        description: generateDescription(category, template),
        platform: 'mock',
        genres: randomSample(template.genres, 2, 4),
        category,
        tags: randomSample(template.tags, 3, 6),
        duration: randomInt(template.minDuration, template.maxDuration)
      };

      catalog.push(content);
      idCounter++;
    }
  }

  return catalog;
}

/**
 * Content templates by category
 */
function getContentTemplates(): Record<ContentCategory, ContentTemplate> {
  return {
    movie: {
      genres: ['drama', 'comedy', 'thriller', 'romance', 'action', 'sci-fi', 'horror', 'fantasy'],
      tags: ['emotional', 'thought-provoking', 'feel-good', 'intense', 'inspiring', 'dark', 'uplifting'],
      minDuration: 90,
      maxDuration: 180,
      emotionalRanges: {
        valenceDelta: [-0.5, 0.7],
        arousalDelta: [-0.4, 0.7],
        intensity: [0.4, 0.9],
        complexity: [0.5, 0.9]
      }
    },

    series: {
      genres: ['drama', 'comedy', 'crime', 'fantasy', 'mystery', 'sci-fi', 'thriller'],
      tags: ['binge-worthy', 'character-driven', 'plot-twist', 'episodic', 'addictive', 'emotional'],
      minDuration: 30,
      maxDuration: 60,
      emotionalRanges: {
        valenceDelta: [-0.3, 0.6],
        arousalDelta: [-0.3, 0.6],
        intensity: [0.5, 0.8],
        complexity: [0.6, 0.9]
      }
    },

    documentary: {
      genres: ['nature', 'history', 'science', 'biographical', 'social', 'true-crime', 'wildlife'],
      tags: ['educational', 'eye-opening', 'inspiring', 'thought-provoking', 'informative', 'fascinating'],
      minDuration: 45,
      maxDuration: 120,
      emotionalRanges: {
        valenceDelta: [0.0, 0.5],
        arousalDelta: [-0.2, 0.4],
        intensity: [0.3, 0.7],
        complexity: [0.5, 0.8]
      }
    },

    music: {
      genres: ['classical', 'jazz', 'ambient', 'world', 'electronic', 'instrumental', 'acoustic'],
      tags: ['relaxing', 'energizing', 'meditative', 'uplifting', 'atmospheric', 'soothing', 'inspiring'],
      minDuration: 3,
      maxDuration: 60,
      emotionalRanges: {
        valenceDelta: [-0.2, 0.6],
        arousalDelta: [-0.6, 0.5],
        intensity: [0.2, 0.8],
        complexity: [0.3, 0.7]
      }
    },

    meditation: {
      genres: ['guided', 'ambient', 'nature-sounds', 'mindfulness', 'breathing', 'sleep', 'relaxation'],
      tags: ['calming', 'stress-relief', 'sleep', 'focus', 'breathing', 'peaceful', 'grounding'],
      minDuration: 5,
      maxDuration: 45,
      emotionalRanges: {
        valenceDelta: [0.1, 0.4],
        arousalDelta: [-0.8, -0.3],
        intensity: [0.1, 0.3],
        complexity: [0.1, 0.4]
      }
    },

    short: {
      genres: ['animation', 'comedy', 'experimental', 'musical', 'documentary', 'drama'],
      tags: ['quick-watch', 'creative', 'fun', 'bite-sized', 'quirky', 'entertaining', 'light'],
      minDuration: 1,
      maxDuration: 15,
      emotionalRanges: {
        valenceDelta: [-0.2, 0.7],
        arousalDelta: [-0.3, 0.5],
        intensity: [0.3, 0.7],
        complexity: [0.3, 0.8]
      }
    }
  };
}
```

### 8.2 Catalog Distribution (200 Items)

| Category | Count | Valence Range | Arousal Range | Primary Tones |
|----------|-------|---------------|---------------|---------------|
| **Movie** | 40 | -0.5 to +0.7 | -0.4 to +0.7 | dramatic, uplifting, thrilling |
| **Series** | 35 | -0.3 to +0.6 | -0.3 to +0.6 | engaging, suspenseful, emotional |
| **Documentary** | 30 | 0.0 to +0.5 | -0.2 to +0.4 | educational, awe-inspiring |
| **Music** | 30 | -0.2 to +0.6 | -0.6 to +0.5 | energizing, calming, uplifting |
| **Meditation** | 35 | +0.1 to +0.4 | -0.8 to -0.3 | serene, peaceful, grounding |
| **Short** | 30 | -0.2 to +0.7 | -0.3 to +0.5 | fun, quirky, lighthearted |

---

## 9. Deployment Architecture

### 9.1 Docker Compose Integration

```yaml
# docker-compose.yml (ContentProfiler services)

services:
  content-profiler:
    build: ./src/content
    environment:
      - GEMINI_API_KEY=${GEMINI_API_KEY}
      - RUVECTOR_URL=http://ruvector:8080
      - AGENTDB_URL=redis://agentdb:6379
      - BATCH_SIZE=10
      - MAX_RETRIES=3
    depends_on:
      - ruvector
      - agentdb
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3001/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  ruvector:
    image: ruvector:latest
    ports:
      - "8080:8080"
    volumes:
      - ruvector-data:/data
    environment:
      - HNSW_M=16
      - HNSW_EF_CONSTRUCTION=200
      - HNSW_EF_SEARCH=100
    mem_limit: 2g
    cpu_count: 2

  agentdb:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - agentdb-data:/data
    command: redis-server --appendonly yes
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 3s
      retries: 3

volumes:
  ruvector-data:
  agentdb-data:
```

### 9.2 Resource Requirements

| Service | CPU | Memory | Disk | Notes |
|---------|-----|--------|------|-------|
| **content-profiler** | 1 core | 512 MB | 100 MB | Batch processing |
| **ruvector** | 2 cores | 2 GB | 500 MB | HNSW index in memory |
| **agentdb** | 1 core | 256 MB | 200 MB | Redis persistence |

---

## 10. Performance Characteristics

### 10.1 Time Complexity

| Operation | Complexity | Details |
|-----------|-----------|---------|
| **Batch Profile (n items)** | O(n × G) | G = Gemini API call (~3s each) |
| **Embedding Generation** | O(d) = O(1) | d = 1536 (constant) |
| **RuVector Upsert** | O(log n) | HNSW insertion |
| **Semantic Search** | O(log n + k) | HNSW search + retrieve k profiles |
| **AgentDB Store/Get** | O(1) | Redis key-value operations |

### 10.2 Throughput Estimates

**Batch Profiling (200 items)**:
- Batch size: 10 items
- Total batches: 20
- Gemini API time: ~3s per item (parallel within batch)
- Rate limit delay: 6s between batches (60 req/min)
- **Total time: ~20 batches × (3s + 6s) = 180 seconds (3 minutes)**
- Actual with parallelization: **~4-5 minutes for 200 items**

**Search Performance**:
- Query vector generation: <10ms
- HNSW search (200 items): <50ms (p95)
- Profile retrieval (10 items): <20ms
- **Total search latency: <100ms (p95)**

### 10.3 Space Complexity

**Per Content Item**:
- Profile object: ~500 bytes (JSON)
- Embedding: 1536 × 4 bytes = 6 KB (Float32)
- Metadata: ~1 KB
- **Total: ~7.5 KB per item**

**200-Item Catalog**:
- Profiles (AgentDB): 200 × 0.5 KB = 100 KB
- Embeddings (RuVector): 200 × 6 KB = 1.2 MB
- HNSW index overhead: 200 × 16 × 8 bytes = 25 KB (M=16)
- **Total: ~1.4 MB (easily fits in memory)**

---

## 11. Error Handling & Resilience

### 11.1 Failure Modes

| Failure | Detection | Recovery Strategy |
|---------|-----------|------------------|
| **Gemini API timeout** | 30s timeout | Retry with exponential backoff (3 attempts) |
| **Gemini rate limit (429)** | HTTP status code | Delay 60s between batches |
| **Invalid JSON response** | JSON parse error | Log error, mark item as failed, continue |
| **RuVector unavailable** | Connection error | Queue embeddings for later storage |
| **AgentDB connection lost** | Redis error | Retry with backoff, fail gracefully |
| **Embedding dimension mismatch** | Validation check | Regenerate embedding, log error |

### 11.2 Retry Logic

```typescript
async function processSingleContent(
  content: ContentMetadata,
  maxRetries: number = 3
): Promise<ProcessResult> {
  let retryCount = 0;
  let lastError: Error | null = null;

  while (retryCount < maxRetries) {
    try {
      // Step 1: Gemini profiling
      const profile = await this.geminiClient.analyze(content);

      // Step 2: Embedding generation
      const embedding = await this.embeddingGenerator.generate(profile, content);

      // Step 3: RuVector storage
      const embeddingId = await this.ruVectorClient.upsert(
        content.contentId,
        embedding,
        createMetadata(profile, content)
      );

      // Step 4: AgentDB storage
      profile.embeddingId = embeddingId;
      await this.agentDBStore.store(profile);

      return { success: true, contentId: content.contentId, error: null };

    } catch (error) {
      lastError = error as Error;
      retryCount++;

      if (retryCount < maxRetries) {
        const delayMs = 2000 * retryCount; // Exponential backoff
        await sleep(delayMs);
        console.log(`Retry ${retryCount}/${maxRetries} for ${content.contentId}`);
      }
    }
  }

  // All retries failed
  return {
    success: false,
    contentId: content.contentId,
    error: lastError?.message || 'Unknown error'
  };
}
```

---

## 12. Testing Strategy

### 12.1 Unit Tests

```typescript
// __tests__/embedding-generator.test.ts

describe('EmbeddingGenerator', () => {
  let generator: EmbeddingGenerator;

  beforeEach(() => {
    generator = new EmbeddingGenerator();
  });

  test('should generate 1536D embedding', async () => {
    const profile = mockEmotionalProfile();
    const content = mockContentMetadata();

    const embedding = await generator.generate(profile, content);

    expect(embedding.length).toBe(1536);
    expect(embedding).toBeInstanceOf(Float32Array);
  });

  test('should normalize embedding to unit length', async () => {
    const profile = mockEmotionalProfile();
    const content = mockContentMetadata();

    const embedding = await generator.generate(profile, content);

    // Calculate magnitude
    let magnitude = 0;
    for (let i = 0; i < embedding.length; i++) {
      magnitude += embedding[i] * embedding[i];
    }
    magnitude = Math.sqrt(magnitude);

    expect(magnitude).toBeCloseTo(1.0, 5);
  });

  test('should encode valence delta in segment 2', async () => {
    const profile = mockEmotionalProfile({ valenceDelta: 0.5 });
    const content = mockContentMetadata();

    const embedding = await generator.generate(profile, content);

    // Check segment 2 (256-383) has high values for positive valence
    const segment2 = Array.from(embedding.slice(256, 384));
    const maxIndex = segment2.indexOf(Math.max(...segment2));

    // Positive valence (0.5) should peak in upper half of segment
    expect(maxIndex).toBeGreaterThan(64); // Upper half
  });
});
```

### 12.2 Integration Tests

```typescript
// __tests__/profiler.integration.test.ts

describe('ContentProfiler Integration', () => {
  let profiler: ContentProfiler;
  let ruVector: RuVectorClient;
  let agentDB: AgentDBStore;

  beforeAll(async () => {
    // Set up test environment
    ruVector = new RuVectorClient(process.env.RUVECTOR_TEST_URL!);
    agentDB = new AgentDBStore(process.env.AGENTDB_TEST_URL!);

    profiler = new ContentProfiler({
      geminiApiKey: process.env.GEMINI_API_KEY!,
      ruvectorUrl: process.env.RUVECTOR_TEST_URL!,
      agentdbUrl: process.env.AGENTDB_TEST_URL!
    });
  });

  afterAll(async () => {
    // Clean up
    await ruVector.deleteCollection('content_embeddings_test');
    await agentDB.flushAll();
  });

  test('should profile single content item end-to-end', async () => {
    const content = mockContentMetadata();

    const profile = await profiler.profileContent(content);

    expect(profile.contentId).toBe(content.contentId);
    expect(profile.valenceDelta).toBeGreaterThanOrEqual(-1);
    expect(profile.valenceDelta).toBeLessThanOrEqual(1);
    expect(profile.embeddingId).toBeTruthy();

    // Verify storage
    const storedProfile = await profiler.getContentProfile(content.contentId);
    expect(storedProfile).toEqual(profile);
  });

  test('should batch profile 10 items with rate limiting', async () => {
    const contents = generateMockCatalog(10);

    const startTime = Date.now();
    const result = await profiler.batchProfile(contents, 10);
    const duration = Date.now() - startTime;

    expect(result.success).toBe(10);
    expect(result.failed).toBe(0);

    // Should take ~30s for 10 items (3s each, parallel)
    expect(duration).toBeGreaterThan(25000);
    expect(duration).toBeLessThan(40000);
  });

  test('should search by emotional transition', async () => {
    // Seed some profiles
    const contents = generateMockCatalog(20);
    await profiler.batchProfile(contents);

    const currentState: EmotionalState = {
      valence: -0.6,
      arousal: 0.5,
      primaryEmotion: 'stressed',
      stressLevel: 0.8,
      confidence: 0.85,
      timestamp: Date.now()
    };

    const desiredState: DesiredState = {
      valence: 0.5,
      arousal: -0.4,
      confidence: 0.75,
      reasoning: 'Want calming content'
    };

    const results = await profiler.searchByTransition(
      currentState,
      desiredState,
      5
    );

    expect(results.length).toBe(5);
    expect(results[0].similarityScore).toBeGreaterThan(0);
    expect(results[0].profile).toBeDefined();

    // Results should be ordered by similarity
    for (let i = 0; i < results.length - 1; i++) {
      expect(results[i].similarityScore).toBeGreaterThanOrEqual(
        results[i + 1].similarityScore
      );
    }
  });
});
```

---

## 13. Monitoring & Observability

### 13.1 Key Metrics

```typescript
export interface ProfilerMetrics {
  // Profiling metrics
  totalProfiled: number;
  successRate: number;          // 0 to 1
  failureRate: number;           // 0 to 1
  averageProfileTime: number;    // milliseconds

  // Gemini API metrics
  geminiCallCount: number;
  geminiTimeoutCount: number;
  geminiRateLimitCount: number;
  averageGeminiLatency: number;  // milliseconds

  // Storage metrics
  embeddingsStored: number;
  averageStorageTime: number;    // milliseconds

  // Search metrics
  searchCount: number;
  averageSearchLatency: number;  // milliseconds
  averageSimilarityScore: number; // 0 to 1

  // Resource metrics
  ruvectorSize: number;          // bytes
  agentdbSize: number;           // bytes

  timestamp: number;
}
```

### 13.2 Logging Strategy

```typescript
// Structured logging with context
logger.info('Starting batch profiling', {
  component: 'ContentProfiler',
  operation: 'batchProfile',
  totalItems: contents.length,
  batchSize,
  timestamp: Date.now()
});

logger.error('Gemini API call failed', {
  component: 'GeminiClient',
  operation: 'analyze',
  contentId: content.contentId,
  error: error.message,
  retryCount,
  timestamp: Date.now()
});

logger.debug('Embedding generated', {
  component: 'EmbeddingGenerator',
  contentId: content.contentId,
  embeddingMagnitude: magnitude,
  timestamp: Date.now()
});
```

---

## 14. Future Enhancements (Post-MVP)

### 14.1 Advanced Features

1. **Multi-Model Profiling**: Use multiple LLMs (Gemini + Claude) for consensus
2. **Dynamic Embeddings**: Update embeddings based on user feedback
3. **Category-Specific Models**: Train specialized embeddings per category
4. **Temporal Embeddings**: Encode time-of-day, season preferences
5. **Social Embeddings**: Incorporate collaborative filtering signals

### 14.2 Optimization Opportunities

1. **Embedding Compression**: Reduce from 1536D to 512D with PCA
2. **Quantization**: Use 8-bit quantized embeddings (75% size reduction)
3. **Caching**: Cache frequently accessed profiles in Redis
4. **Batch Search**: Search for multiple transitions in parallel
5. **Incremental Indexing**: Update HNSW index without rebuild

---

## 15. Appendix

### 15.1 Gemini Prompt Template

```typescript
const GEMINI_PROMPT_TEMPLATE = `
Analyze the emotional impact of this content:

Title: {TITLE}
Description: {DESCRIPTION}
Genres: {GENRES}
Category: {CATEGORY}
Tags: {TAGS}
Duration: {DURATION} minutes

Provide a detailed emotional analysis:

1. **Primary Emotional Tone**: The dominant emotional quality (e.g., uplifting, calming, thrilling, melancholic, cathartic, thought-provoking)

2. **Valence Delta**: Expected change in viewer's emotional valence from before to after viewing
   - Range: -1.0 (very negative shift) to +1.0 (very positive shift)
   - Example: A sad drama might be -0.3, an uplifting comedy +0.7

3. **Arousal Delta**: Expected change in viewer's arousal/energy level
   - Range: -1.0 (very calming) to +1.0 (very energizing)
   - Example: A nature documentary might be -0.5, a thriller +0.8

4. **Emotional Intensity**: How strong the emotional impact is
   - Range: 0.0 (subtle, gentle) to 1.0 (very intense, overwhelming)
   - Example: A light comedy might be 0.3, a heavy drama 0.9

5. **Emotional Complexity**: How simple or nuanced the emotional journey is
   - Range: 0.0 (simple, single emotion) to 1.0 (complex, mixed emotions)
   - Example: A feel-good movie might be 0.3, an art film 0.9

6. **Target Viewer States**: What emotional states would this content be good for? (provide 2-3)
   - For each state, specify:
     - currentValence: -1.0 to +1.0
     - currentArousal: -1.0 to +1.0
     - description: Brief text description

**IMPORTANT**: Respond ONLY with valid JSON in this exact format:

{
  "primaryTone": "...",
  "valenceDelta": 0.0,
  "arousalDelta": 0.0,
  "intensity": 0.0,
  "complexity": 0.0,
  "targetStates": [
    {
      "currentValence": 0.0,
      "currentArousal": 0.0,
      "description": "..."
    }
  ]
}
`;
```

### 15.2 Example Mock Content Items

```json
[
  {
    "contentId": "mock_movie_001",
    "title": "The Pursuit of Happiness",
    "description": "Inspirational drama about overcoming adversity",
    "platform": "mock",
    "genres": ["drama", "biographical"],
    "category": "movie",
    "tags": ["inspiring", "emotional", "feel-good", "uplifting"],
    "duration": 117
  },
  {
    "contentId": "mock_meditation_001",
    "title": "Ocean Waves for Deep Sleep",
    "description": "Natural ocean sounds for relaxation and sleep",
    "platform": "mock",
    "genres": ["ambient", "nature-sounds"],
    "category": "meditation",
    "tags": ["calming", "sleep-aid", "stress-relief", "peaceful"],
    "duration": 30
  },
  {
    "contentId": "mock_series_001",
    "title": "The Office (US)",
    "description": "Mockumentary-style sitcom about office life",
    "platform": "mock",
    "genres": ["comedy", "mockumentary"],
    "category": "series",
    "tags": ["funny", "light", "feel-good", "binge-worthy"],
    "duration": 22
  }
]
```

---

## 16. Conclusion

The **ContentProfiler** module architecture provides a robust, scalable foundation for emotion-driven content analysis. Key architectural strengths:

✅ **Efficient Batch Processing**: 200 items profiled in ~5 minutes
✅ **High-Quality Embeddings**: 1536D vectors with semantic encoding
✅ **Fast Semantic Search**: <100ms p95 with HNSW indexing
✅ **Resilient Design**: Retry logic, rate limiting, graceful failures
✅ **Clear Separation of Concerns**: Modular design for maintainability

### Next Steps (SPARC Phase 4: Refinement)

1. Implement ContentProfiler class with TDD approach
2. Build GeminiClient with retry logic
3. Create EmbeddingGenerator with unit tests
4. Integrate RuVectorClient with HNSW configuration
5. Generate 200-item mock catalog
6. Run integration tests with real APIs
7. Profile and optimize bottlenecks

---

**Document Version**: 1.0
**Created**: 2025-12-05
**SPARC Phase**: Architecture (Phase 3)
**Component**: ContentProfiler Module
**Next Phase**: Refinement (TDD Implementation)
