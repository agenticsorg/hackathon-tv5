/**
 * Vertex AI Integration Module
 *
 * Provides semantic search capabilities for the Nexus-UMMID Metadata API
 * using Google Cloud's Vertex AI Embeddings and Matching Engine.
 *
 * @module vertex-ai
 *
 * ## Features
 * - Text embeddings generation using text-embedding-004 model
 * - Vector similarity search with Matching Engine
 * - High-level semantic search service
 * - Metadata filtering and ranking
 * - Batch processing support
 * - Retry logic and error handling
 *
 * ## Usage Example
 *
 * ```typescript
 * import { SemanticSearchService } from './vertex-ai';
 *
 * const searchService = new SemanticSearchService('my-index');
 *
 * // Index content
 * await searchService.indexContent(metadata);
 *
 * // Search
 * const results = await searchService.search('action movies with cars', {
 *   limit: 10,
 *   filters: { genres: ['action'], type: 'movie' }
 * });
 * ```
 *
 * ## Environment Variables
 * - GOOGLE_CLOUD_PROJECT: GCP project ID (default: agentics-foundation25lon-1899)
 * - LOG_LEVEL: Logging level (default: info)
 *
 * ## Dependencies
 * Requires @google-cloud/aiplatform package:
 * ```bash
 * npm install @google-cloud/aiplatform
 * ```
 */

// ============================================================================
// Embeddings Module
// ============================================================================

export {
  VertexAIEmbeddings,
  getEmbeddingsInstance,
  type EmbeddingsConfig,
  type EmbeddingResult,
  type BatchEmbeddingResult
} from './embeddings';

// ============================================================================
// Matching Engine Module
// ============================================================================

export {
  MatchingEngineClient,
  getMatchingEngineInstance,
  type VectorEntry,
  type MatchingEngineSearchResult,
  type IndexConfig,
  type MatchingEngineConfig
} from './matching-engine';

// ============================================================================
// Semantic Search Module
// ============================================================================

export {
  SemanticSearchService,
  getSemanticSearchInstance,
  type SearchFilters,
  type IndexingOptions,
  type SearchOptions,
  type SemanticSearchResult,
  type SearchMetrics
} from './semantic-search';

// ============================================================================
// Module Information
// ============================================================================

/**
 * Module version
 */
export const VERSION = '1.0.0';

/**
 * Supported embedding model
 */
export const EMBEDDING_MODEL = 'text-embedding-004';

/**
 * Embedding dimensions
 */
export const EMBEDDING_DIMENSIONS = 768;

/**
 * Default GCP region
 */
export const DEFAULT_REGION = 'us-central1';

/**
 * Default GCP project
 */
export const DEFAULT_PROJECT = process.env.GOOGLE_CLOUD_PROJECT || 'agentics-foundation25lon-1899';

/**
 * Module metadata
 */
export const MODULE_INFO = {
  name: 'vertex-ai',
  version: VERSION,
  description: 'Vertex AI integration for semantic search',
  model: EMBEDDING_MODEL,
  dimensions: EMBEDDING_DIMENSIONS,
  region: DEFAULT_REGION,
  project: DEFAULT_PROJECT
} as const;

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Calculate cosine similarity between two vectors
 *
 * @param a - First vector
 * @param b - Second vector
 * @returns Cosine similarity score (0-1)
 */
export function cosineSimilarity(a: number[], b: number[]): number {
  if (a.length !== b.length) {
    throw new Error('Vectors must have the same dimensions');
  }

  let dotProduct = 0;
  let normA = 0;
  let normB = 0;

  for (let i = 0; i < a.length; i++) {
    dotProduct += a[i] * b[i];
    normA += a[i] * a[i];
    normB += b[i] * b[i];
  }

  const magnitude = Math.sqrt(normA) * Math.sqrt(normB);

  if (magnitude === 0) return 0;

  return dotProduct / magnitude;
}

/**
 * Calculate Euclidean distance between two vectors
 *
 * @param a - First vector
 * @param b - Second vector
 * @returns Euclidean distance
 */
export function euclideanDistance(a: number[], b: number[]): number {
  if (a.length !== b.length) {
    throw new Error('Vectors must have the same dimensions');
  }

  let sum = 0;

  for (let i = 0; i < a.length; i++) {
    const diff = a[i] - b[i];
    sum += diff * diff;
  }

  return Math.sqrt(sum);
}

/**
 * Normalize a vector to unit length
 *
 * @param vector - Input vector
 * @returns Normalized vector
 */
export function normalizeVector(vector: number[]): number[] {
  const magnitude = Math.sqrt(vector.reduce((sum, val) => sum + val * val, 0));

  if (magnitude === 0) {
    return vector.map(() => 0);
  }

  return vector.map(val => val / magnitude);
}

/**
 * Validate embedding vector
 *
 * @param embedding - Embedding vector to validate
 * @param expectedDimensions - Expected number of dimensions
 * @returns True if valid
 */
export function validateEmbedding(
  embedding: number[],
  expectedDimensions: number = EMBEDDING_DIMENSIONS
): boolean {
  if (!Array.isArray(embedding)) return false;
  if (embedding.length !== expectedDimensions) return false;
  if (embedding.some(val => typeof val !== 'number' || !isFinite(val))) return false;

  return true;
}

/**
 * Check if Vertex AI is properly configured
 *
 * @returns Configuration status
 */
export function checkConfiguration(): {
  configured: boolean;
  project: string | undefined;
  region: string;
  model: string;
  issues: string[];
} {
  const issues: string[] = [];

  const project = process.env.GOOGLE_CLOUD_PROJECT;
  if (!project) {
    issues.push('GOOGLE_CLOUD_PROJECT environment variable not set');
  }

  // Check for credentials (simplified check)
  if (!process.env.GOOGLE_APPLICATION_CREDENTIALS && !process.env.GCLOUD_PROJECT) {
    issues.push('Google Cloud credentials not configured (GOOGLE_APPLICATION_CREDENTIALS or Application Default Credentials)');
  }

  return {
    configured: issues.length === 0,
    project,
    region: DEFAULT_REGION,
    model: EMBEDDING_MODEL,
    issues
  };
}

// ============================================================================
// Re-export Common Types
// ============================================================================

// Re-export commonly used types from other modules for convenience
export type { MediaMetadata, SearchResult } from '../types';

/**
 * Health check for Vertex AI services
 *
 * @returns Health status
 */
export async function healthCheck(): Promise<{
  status: 'healthy' | 'degraded' | 'unhealthy';
  services: {
    embeddings: boolean;
    matchingEngine: boolean;
  };
  timestamp: Date;
}> {
  const timestamp = new Date();
  const services = {
    embeddings: false,
    matchingEngine: false
  };

  try {
    // Test embeddings service
    const { getEmbeddingsInstance } = await import('./embeddings.js');
    const embeddings = getEmbeddingsInstance();
    await embeddings.generateEmbedding('health check');
    services.embeddings = true;
  } catch (error) {
    console.error('Embeddings health check failed:', error);
  }

  try {
    // Test matching engine service
    const { getMatchingEngineInstance } = await import('./matching-engine.js');
    const matchingEngine = getMatchingEngineInstance();
    await matchingEngine.listIndexes();
    services.matchingEngine = true;
  } catch (error) {
    console.error('Matching Engine health check failed:', error);
  }

  const healthyCount = Object.values(services).filter(Boolean).length;
  let status: 'healthy' | 'degraded' | 'unhealthy';

  if (healthyCount === 2) {
    status = 'healthy';
  } else if (healthyCount === 1) {
    status = 'degraded';
  } else {
    status = 'unhealthy';
  }

  return { status, services, timestamp };
}
