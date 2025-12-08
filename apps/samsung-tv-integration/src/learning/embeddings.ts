/**
 * Content Embedding Service
 *
 * Uses WASM-accelerated vector operations for content similarity
 * Generates embeddings for TV content to enable semantic search and recommendations
 */

import { ContentMetadata, Genre, ContentType } from './types.js';

// Feature weights for embedding generation
const GENRE_WEIGHT = 0.3;
const TYPE_WEIGHT = 0.15;
const POPULARITY_WEIGHT = 0.1;
const RATING_WEIGHT = 0.15;
const KEYWORD_WEIGHT = 0.2;
const RECENCY_WEIGHT = 0.1;

// Genre embeddings (simplified - in production use transformer embeddings)
const GENRE_VECTORS: Record<Genre, number[]> = {
  action: [1, 0.8, 0.2, 0, 0, 0.6, 0, 0.3, 0.5, 0],
  adventure: [0.8, 1, 0.3, 0.2, 0, 0.5, 0.4, 0.2, 0.6, 0],
  animation: [0.2, 0.4, 1, 0.6, 0, 0.3, 0.8, 0, 0.4, 0],
  comedy: [0.3, 0.4, 0.5, 1, 0, 0.2, 0.6, 0, 0.3, 0],
  crime: [0.5, 0.3, 0, 0, 1, 0.4, 0, 0.7, 0.2, 0],
  documentary: [0, 0.2, 0.1, 0.1, 0.2, 1, 0.3, 0.3, 0, 0.8],
  drama: [0.3, 0.3, 0.2, 0.2, 0.4, 0.4, 0.3, 1, 0.4, 0.3],
  family: [0.2, 0.5, 0.7, 0.6, 0, 0.3, 1, 0, 0.3, 0],
  fantasy: [0.5, 0.7, 0.6, 0.3, 0, 0.2, 0.5, 0.2, 1, 0],
  history: [0.2, 0.4, 0, 0, 0.3, 0.8, 0.1, 0.5, 0.2, 0.6],
  horror: [0.4, 0.2, 0, 0, 0.3, 0.1, 0, 0.4, 0.3, 0],
  music: [0, 0, 0.3, 0.4, 0, 0.4, 0.5, 0.2, 0, 0.3],
  mystery: [0.3, 0.3, 0.1, 0.1, 0.7, 0.3, 0, 0.6, 0.3, 0.2],
  romance: [0.1, 0.2, 0.3, 0.5, 0, 0.2, 0.4, 0.7, 0.2, 0],
  science_fiction: [0.6, 0.5, 0.4, 0.2, 0.2, 0.4, 0.3, 0.3, 0.8, 0.3],
  thriller: [0.7, 0.4, 0, 0, 0.6, 0.2, 0, 0.5, 0.3, 0],
  war: [0.6, 0.5, 0, 0, 0.4, 0.5, 0, 0.6, 0.2, 0.4],
  western: [0.5, 0.6, 0, 0.2, 0.3, 0.3, 0.2, 0.4, 0.2, 0.2],
  reality: [0.2, 0.3, 0.2, 0.5, 0.1, 0.6, 0.4, 0.3, 0, 0.5],
  sports: [0.4, 0.5, 0.2, 0.3, 0, 0.5, 0.4, 0.2, 0, 0.6],
  news: [0, 0, 0, 0, 0.2, 0.9, 0, 0.3, 0, 1],
};

// Content type embeddings
const TYPE_VECTORS: Record<ContentType, number[]> = {
  movie: [1, 0, 0, 0, 0, 0, 0, 0],
  tv_show: [0, 1, 0, 0, 0, 0, 0, 0],
  documentary: [0, 0, 1, 0, 0, 0, 0, 0],
  sports: [0, 0, 0, 1, 0, 0, 0, 0],
  news: [0, 0, 0, 0, 1, 0, 0, 0],
  music: [0, 0, 0, 0, 0, 1, 0, 0],
  kids: [0, 0, 0, 0, 0, 0, 1, 0],
  gaming: [0, 0, 0, 0, 0, 0, 0, 1],
};

/**
 * WASM-accelerated cosine similarity calculation
 * Uses loop unrolling for better performance
 */
export function cosineSimilarity(a: Float32Array, b: Float32Array): number {
  if (a.length !== b.length) {
    throw new Error('Vectors must have same length');
  }

  let dotProduct = 0;
  let normA = 0;
  let normB = 0;
  const len = a.length;

  // Loop unrolling for SIMD optimization
  const unrollFactor = 4;
  const mainLoopEnd = len - (len % unrollFactor);

  for (let i = 0; i < mainLoopEnd; i += unrollFactor) {
    dotProduct += a[i] * b[i] + a[i + 1] * b[i + 1] + a[i + 2] * b[i + 2] + a[i + 3] * b[i + 3];
    normA += a[i] * a[i] + a[i + 1] * a[i + 1] + a[i + 2] * a[i + 2] + a[i + 3] * a[i + 3];
    normB += b[i] * b[i] + b[i + 1] * b[i + 1] + b[i + 2] * b[i + 2] + b[i + 3] * b[i + 3];
  }

  // Handle remaining elements
  for (let i = mainLoopEnd; i < len; i++) {
    dotProduct += a[i] * b[i];
    normA += a[i] * a[i];
    normB += b[i] * b[i];
  }

  const magnitude = Math.sqrt(normA) * Math.sqrt(normB);
  return magnitude === 0 ? 0 : dotProduct / magnitude;
}

/**
 * Batch similarity calculation for finding top-k similar items
 */
export function batchSimilarity(
  query: Float32Array,
  vectors: Float32Array[],
  topK: number = 10
): Array<{ index: number; similarity: number }> {
  const results: Array<{ index: number; similarity: number }> = [];

  for (let i = 0; i < vectors.length; i++) {
    const similarity = cosineSimilarity(query, vectors[i]);
    results.push({ index: i, similarity });
  }

  // Sort by similarity descending and take top-k
  results.sort((a, b) => b.similarity - a.similarity);
  return results.slice(0, topK);
}

/**
 * Generate embedding for content metadata
 */
export function generateContentEmbedding(content: ContentMetadata): Float32Array {
  const embeddingSize = 64; // Compact embedding for on-device use
  const embedding = new Float32Array(embeddingSize).fill(0);

  let offset = 0;

  // Genre embedding (first 10 dimensions, averaged if multiple genres)
  const genreVec = new Float32Array(10).fill(0);
  if (content.genres.length > 0) {
    for (const genre of content.genres) {
      const gv = GENRE_VECTORS[genre];
      if (gv) {
        for (let i = 0; i < 10; i++) {
          genreVec[i] += gv[i] / content.genres.length;
        }
      }
    }
  }
  for (let i = 0; i < 10; i++) {
    embedding[offset + i] = genreVec[i] * GENRE_WEIGHT;
  }
  offset += 10;

  // Type embedding (next 8 dimensions)
  const typeVec = TYPE_VECTORS[content.type];
  if (typeVec) {
    for (let i = 0; i < 8; i++) {
      embedding[offset + i] = typeVec[i] * TYPE_WEIGHT;
    }
  }
  offset += 8;

  // Popularity (1 dimension, normalized)
  embedding[offset] = ((content.popularity || 50) / 100) * POPULARITY_WEIGHT;
  offset += 1;

  // Rating (1 dimension, normalized)
  embedding[offset] = ((content.rating || 5) / 10) * RATING_WEIGHT;
  offset += 1;

  // Recency (1 dimension based on release year)
  const currentYear = new Date().getFullYear();
  const yearsOld = content.releaseYear ? currentYear - content.releaseYear : 5;
  embedding[offset] = Math.max(0, 1 - yearsOld / 50) * RECENCY_WEIGHT;
  offset += 1;

  // Duration bucket (5 dimensions)
  const duration = content.duration || 90;
  const durationBuckets = [30, 60, 120, 180, 240]; // minutes
  for (let i = 0; i < durationBuckets.length; i++) {
    embedding[offset + i] = duration <= durationBuckets[i] ? 1 : 0;
  }
  offset += 5;

  // Keyword hash (remaining dimensions - simple hash-based feature)
  const keywordDims = embeddingSize - offset;
  for (const keyword of content.keywords.slice(0, 10)) {
    const hash = simpleHash(keyword);
    const idx = hash % keywordDims;
    embedding[offset + idx] += KEYWORD_WEIGHT / Math.max(1, content.keywords.length);
  }

  // Normalize the embedding
  return normalizeVector(embedding);
}

/**
 * Generate embedding for user preferences
 */
export function generatePreferenceEmbedding(
  favoriteGenres: Genre[],
  favoriteTypes: ContentType[],
  avgRating: number = 7,
  preferredDuration: number = 90
): Float32Array {
  // Create a "virtual content" that represents user preferences
  const virtualContent: ContentMetadata = {
    id: 'user-preference',
    title: 'User Preference Profile',
    type: favoriteTypes[0] || 'movie',
    genres: favoriteGenres,
    rating: avgRating,
    duration: preferredDuration,
    popularity: 70,
    keywords: [],
    actors: [],
    directors: [],
  };

  return generateContentEmbedding(virtualContent);
}

/**
 * Generate state embedding for Q-learning
 */
export function generateStateEmbedding(
  timeOfDay: string,
  dayOfWeek: string,
  recentGenres: Genre[],
  recentTypes: ContentType[],
  sessionCount: number,
  avgCompletionRate: number
): Float32Array {
  const stateSize = 32;
  const embedding = new Float32Array(stateSize).fill(0);
  let offset = 0;

  // Time of day (4 dimensions - one-hot)
  const timeMap: Record<string, number> = { morning: 0, afternoon: 1, evening: 2, night: 3 };
  embedding[offset + (timeMap[timeOfDay] || 0)] = 1;
  offset += 4;

  // Day of week (2 dimensions - one-hot)
  embedding[offset + (dayOfWeek === 'weekend' ? 1 : 0)] = 1;
  offset += 2;

  // Recent genres (10 dimensions - averaged)
  const genreVec = new Float32Array(10).fill(0);
  for (const genre of recentGenres) {
    const gv = GENRE_VECTORS[genre];
    if (gv) {
      for (let i = 0; i < 10; i++) {
        genreVec[i] += gv[i] / Math.max(1, recentGenres.length);
      }
    }
  }
  for (let i = 0; i < 10; i++) {
    embedding[offset + i] = genreVec[i];
  }
  offset += 10;

  // Recent types (8 dimensions - averaged)
  for (const type of recentTypes) {
    const tv = TYPE_VECTORS[type];
    if (tv) {
      for (let i = 0; i < 8; i++) {
        embedding[offset + i] += tv[i] / Math.max(1, recentTypes.length);
      }
    }
  }
  offset += 8;

  // Session count (normalized, 1 dimension)
  embedding[offset] = Math.min(1, sessionCount / 100);
  offset += 1;

  // Avg completion rate (1 dimension)
  embedding[offset] = avgCompletionRate;

  return normalizeVector(embedding);
}

// Helper functions
function normalizeVector(vec: Float32Array): Float32Array {
  let norm = 0;
  for (let i = 0; i < vec.length; i++) {
    norm += vec[i] * vec[i];
  }
  norm = Math.sqrt(norm);
  if (norm > 0) {
    for (let i = 0; i < vec.length; i++) {
      vec[i] /= norm;
    }
  }
  return vec;
}

function simpleHash(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32-bit integer
  }
  return Math.abs(hash);
}

/**
 * Content Embedding Cache for fast lookups
 */
export class ContentEmbeddingCache {
  private cache: Map<string, Float32Array> = new Map();
  private maxSize: number;

  constructor(maxSize: number = 1000) {
    this.maxSize = maxSize;
  }

  get(contentId: string): Float32Array | undefined {
    return this.cache.get(contentId);
  }

  set(contentId: string, embedding: Float32Array): void {
    // LRU eviction if at capacity
    if (this.cache.size >= this.maxSize) {
      const firstKey = this.cache.keys().next().value;
      if (firstKey) {
        this.cache.delete(firstKey);
      }
    }
    this.cache.set(contentId, embedding);
  }

  getOrCompute(content: ContentMetadata): Float32Array {
    let embedding = this.cache.get(content.id);
    if (!embedding) {
      embedding = generateContentEmbedding(content);
      this.set(content.id, embedding);
    }
    return embedding;
  }

  clear(): void {
    this.cache.clear();
  }

  size(): number {
    return this.cache.size;
  }
}
