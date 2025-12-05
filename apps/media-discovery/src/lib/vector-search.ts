/**
 * Vector Search Integration with RuVector
 * Uses the ruvector npm package for embedded vector database
 * https://www.npmjs.com/package/ruvector
 */

import { VectorDB } from 'ruvector';
import type { MediaContent } from '@/types/media';

// Embedding dimensions (text-embedding-3-small default, can be customized)
const EMBEDDING_DIMENSIONS = 768;

// Storage path for persistent vector database
const STORAGE_PATH = process.env.RUVECTOR_STORAGE_PATH || './data/media-vectors.db';

// Max elements for the HNSW index
const MAX_ELEMENTS = 100000;

// Singleton database instance
let db: InstanceType<typeof VectorDB> | null = null;

// Server-side embedding cache (survives across requests in the same process)
const embeddingCache = new Map<string, { embedding: Float32Array; timestamp: number }>();
const CACHE_TTL_MS = 5 * 60 * 1000; // 5 minute TTL

/**
 * Media content metadata stored alongside vectors
 */
interface MediaVectorMetadata {
  contentId: number;
  mediaType: 'movie' | 'tv';
  title: string;
  overview: string;
  genreIds: number[];
  voteAverage: number;
  releaseDate: string;
  posterPath: string | null;
}

/**
 * Get or create the vector database instance
 */
export function getVectorDb(): InstanceType<typeof VectorDB> {
  if (!db) {
    db = new VectorDB({
      dimensions: EMBEDDING_DIMENSIONS,
      maxElements: MAX_ELEMENTS,
      storagePath: STORAGE_PATH,
    });
    console.log(`✅ VectorDb initialized with ${EMBEDDING_DIMENSIONS} dimensions`);
  }
  return db;
}

/**
 * Check if the vector database is available
 */
export async function isVectorDbAvailable(): Promise<boolean> {
  try {
    const database = getVectorDb();
    await database.len();
    return true;
  } catch {
    return false;
  }
}

/**
 * Generate embedding for text content
 * Uses OpenAI embeddings API or falls back to mock embedding
 */
export async function getContentEmbedding(text: string): Promise<Float32Array | null> {
  try {
    // Normalize the text for cache key
    const cacheKey = text.toLowerCase().trim();

    // Check cache first
    const cached = embeddingCache.get(cacheKey);
    if (cached && Date.now() - cached.timestamp < CACHE_TTL_MS) {
      console.log(`📦 Embedding cache hit for: "${text.slice(0, 30)}..."`);
      return cached.embedding;
    }

    const OPENAI_API_KEY = process.env.OPENAI_API_KEY;
    if (!OPENAI_API_KEY) {
      console.warn('OpenAI API key not set, using mock embedding');
      return generateMockEmbedding(text);
    }

    console.log(`🔄 Generating embedding for: "${text.slice(0, 30)}..."`);
    const response = await fetch('https://api.openai.com/v1/embeddings', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${OPENAI_API_KEY}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        model: 'text-embedding-3-small',
        input: text,
        dimensions: EMBEDDING_DIMENSIONS,
      }),
    });

    if (!response.ok) {
      throw new Error(`OpenAI API error: ${response.status}`);
    }

    const data = await response.json();
    const embedding = new Float32Array(data.data[0].embedding);

    // Cache the result
    embeddingCache.set(cacheKey, { embedding, timestamp: Date.now() });

    // Clean old entries periodically (every 100 new entries)
    if (embeddingCache.size > 100) {
      const now = Date.now();
      for (const [key, value] of embeddingCache) {
        if (now - value.timestamp > CACHE_TTL_MS) {
          embeddingCache.delete(key);
        }
      }
    }

    return embedding;
  } catch (error) {
    console.error('Failed to generate embedding:', error);
    return null;
  }
}

/**
 * Generate a mock embedding for testing
 * Creates a deterministic embedding based on text hash
 */
function generateMockEmbedding(text: string): Float32Array {
  const embedding = new Float32Array(EMBEDDING_DIMENSIONS);
  const textLower = text.toLowerCase();

  for (let i = 0; i < textLower.length; i++) {
    const charCode = textLower.charCodeAt(i);
    const idx = (charCode + i) % EMBEDDING_DIMENSIONS;
    embedding[idx] += Math.sin(charCode / 10) * 0.1;
  }

  // Normalize the vector
  let magnitude = 0;
  for (let i = 0; i < embedding.length; i++) {
    magnitude += embedding[i] * embedding[i];
  }
  magnitude = Math.sqrt(magnitude);

  if (magnitude > 0) {
    for (let i = 0; i < embedding.length; i++) {
      embedding[i] /= magnitude;
    }
  }

  return embedding;
}

/**
 * Store media content embedding in the vector database
 */
export async function storeContentEmbedding(
  content: MediaContent,
  embedding: Float32Array
): Promise<string> {
  const database = getVectorDb();
  const id = `${content.mediaType}-${content.id}`;

  await database.insert({
    id,
    vector: embedding,
    metadata: {
      contentId: content.id,
      mediaType: content.mediaType,
      title: content.title,
      overview: content.overview,
      genreIds: content.genreIds,
      voteAverage: content.voteAverage,
      releaseDate: content.releaseDate,
      posterPath: content.posterPath,
    } as MediaVectorMetadata,
  });

  return id;
}

/**
 * Batch store content embeddings for better performance
 */
export async function batchStoreEmbeddings(
  contents: Array<{ content: MediaContent; embedding: Float32Array }>
): Promise<string[]> {
  const database = getVectorDb();
  const ids: string[] = [];

  for (const { content, embedding } of contents) {
    const id = `${content.mediaType}-${content.id}`;
    ids.push(id);

    await database.insert({
      id,
      vector: embedding,
      metadata: {
        contentId: content.id,
        mediaType: content.mediaType,
        title: content.title,
        overview: content.overview,
        genreIds: content.genreIds,
        voteAverage: content.voteAverage,
        releaseDate: content.releaseDate,
        posterPath: content.posterPath,
      } as MediaVectorMetadata,
    });
  }

  console.log(`✅ Stored ${ids.length} embeddings`);
  return ids;
}

/**
 * Search for similar content by embedding vector
 */
export async function searchByEmbedding(
  queryEmbedding: Float32Array,
  k: number = 10,
  threshold: number = 0.5,
  filter?: { mediaType?: 'movie' | 'tv'; genres?: number[] }
): Promise<Array<{ content: MediaContent; score: number }>> {
  const database = getVectorDb();

  const results = await database.search({
    vector: queryEmbedding,
    k: k * 2, // Get more results to filter
    threshold,
  });

  // Apply post-search filtering (ruvector doesn't have built-in metadata filtering)
  type SearchResultItem = { id: string; score: number; metadata: Record<string, unknown> };
  let filtered: SearchResultItem[] = (results as SearchResultItem[]).filter(
    (r) => r.metadata && typeof r.metadata === 'object' && 'contentId' in r.metadata
  );

  if (filter?.mediaType) {
    filtered = filtered.filter(
      (r: SearchResultItem) => (r.metadata as unknown as MediaVectorMetadata).mediaType === filter.mediaType
    );
  }

  if (filter?.genres && filter.genres.length > 0) {
    filtered = filtered.filter((r: SearchResultItem) => {
      const meta = r.metadata as unknown as MediaVectorMetadata;
      return meta.genreIds && filter.genres!.some((g: number) => meta.genreIds.includes(g));
    });
  }

  return filtered.slice(0, k).map((result: SearchResultItem) => {
    const meta = result.metadata as unknown as MediaVectorMetadata;
    return {
      content: {
        id: meta.contentId,
        title: meta.title || 'Unknown',
        overview: meta.overview || '',
        mediaType: meta.mediaType || 'movie',
        genreIds: meta.genreIds || [],
        voteAverage: meta.voteAverage || 0,
        releaseDate: meta.releaseDate || '',
        posterPath: meta.posterPath || null,
        backdropPath: null,
        voteCount: 0,
        popularity: 0,
      },
      score: result.score,
    };
  });
}

/**
 * Semantic search using natural language query
 */
export async function semanticSearch(
  query: string,
  k: number = 10,
  filter?: { mediaType?: 'movie' | 'tv'; genres?: number[] }
): Promise<Array<{ content: MediaContent; score: number }>> {
  const embedding = await getContentEmbedding(query);
  if (!embedding) {
    return [];
  }

  return searchByEmbedding(embedding, k, 0.3, filter);
}

/**
 * Find similar content to a given item
 */
export async function findSimilarContent(
  contentId: number,
  mediaType: 'movie' | 'tv',
  k: number = 10
): Promise<Array<{ content: MediaContent; score: number }>> {
  const database = getVectorDb();
  const id = `${mediaType}-${contentId}`;

  // Get the existing embedding
  const existing = await database.get(id);
  if (!existing) {
    return [];
  }

  // Search for similar (excluding self)
  const results = await searchByEmbedding(existing.vector as Float32Array, k + 1, 0.3);
  return results.filter((r) => r.content.id !== contentId).slice(0, k);
}

/**
 * Get a specific content vector by ID
 */
export async function getContentVector(
  contentId: number,
  mediaType: 'movie' | 'tv'
): Promise<{ vector: Float32Array; metadata: MediaVectorMetadata } | null> {
  const database = getVectorDb();
  const id = `${mediaType}-${contentId}`;

  const result = await database.get(id);
  if (!result) {
    return null;
  }

  return {
    vector: result.vector as Float32Array,
    metadata: result.metadata as MediaVectorMetadata,
  };
}

/**
 * Delete a content vector from the database
 */
export async function deleteContentVector(
  contentId: number,
  mediaType: 'movie' | 'tv'
): Promise<boolean> {
  const database = getVectorDb();
  const id = `${mediaType}-${contentId}`;

  return await database.delete(id);
}

/**
 * Get the total number of vectors in the database
 */
export async function getVectorCount(): Promise<number> {
  const database = getVectorDb();
  return await database.len();
}

/**
 * Calculate cosine similarity between two embeddings
 */
export function calculateSimilarity(
  embedding1: Float32Array,
  embedding2: Float32Array
): number {
  if (embedding1.length !== embedding2.length) {
    throw new Error('Embeddings must have the same length');
  }

  let dotProduct = 0;
  let magnitude1 = 0;
  let magnitude2 = 0;

  for (let i = 0; i < embedding1.length; i++) {
    dotProduct += embedding1[i] * embedding2[i];
    magnitude1 += embedding1[i] * embedding1[i];
    magnitude2 += embedding2[i] * embedding2[i];
  }

  magnitude1 = Math.sqrt(magnitude1);
  magnitude2 = Math.sqrt(magnitude2);

  if (magnitude1 === 0 || magnitude2 === 0) {
    return 0;
  }

  return dotProduct / (magnitude1 * magnitude2);
}

/**
 * Create combined embedding from multiple text fields
 * Useful for combining title, overview, and genres
 */
export async function createCombinedEmbedding(
  content: MediaContent,
  genreNames: string[] = []
): Promise<Float32Array | null> {
  // Combine relevant text fields for richer embedding
  const combinedText = [
    content.title,
    content.overview,
    genreNames.join(', '),
  ]
    .filter(Boolean)
    .join(' | ');

  return getContentEmbedding(combinedText);
}

/**
 * Get database statistics
 */
export async function getDbStats(): Promise<{
  vectorCount: number;
  dimensions: number;
  storagePath: string;
}> {
  const database = getVectorDb();
  const count = await database.len();

  return {
    vectorCount: count,
    dimensions: EMBEDDING_DIMENSIONS,
    storagePath: STORAGE_PATH,
  };
}
