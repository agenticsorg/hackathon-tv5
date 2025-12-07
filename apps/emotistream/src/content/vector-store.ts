/**
 * VectorStore - In-memory vector storage with cosine similarity search
 * MVP implementation - can be swapped to RuVector HNSW later
 */

/**
 * Metadata stored with each vector
 */
export interface VectorMetadata {
  contentId: string;
  title?: string;
  genres?: string[];
  category?: 'movie' | 'series' | 'documentary' | 'music' | 'meditation' | 'short';
  emotionalProfile?: {
    valenceDelta: number;
    arousalDelta: number;
    intensity: number;
  };
  [key: string]: unknown; // Allow additional properties
}

export interface SearchResult {
  id: string;
  score: number;
  metadata: VectorMetadata;
}

export class VectorStore {
  private vectors: Map<string, { vector: Float32Array; metadata: VectorMetadata }> = new Map();

  /**
   * Upsert (insert or update) a vector with metadata
   */
  async upsert(id: string, vector: Float32Array, metadata: VectorMetadata): Promise<void> {
    if (vector.length !== 1536) {
      throw new Error(`Invalid vector dimension: ${vector.length} (expected 1536)`);
    }

    this.vectors.set(id, { vector, metadata });
  }

  /**
   * Search for similar vectors using cosine similarity
   */
  async search(queryVector: Float32Array, limit: number): Promise<SearchResult[]> {
    if (this.vectors.size === 0) {
      return [];
    }

    const results: SearchResult[] = [];

    // Calculate cosine similarity for all vectors
    for (const [id, { vector, metadata }] of this.vectors.entries()) {
      const score = this.cosineSimilarity(queryVector, vector);
      results.push({ id, score, metadata });
    }

    // Sort by score (descending) and limit
    results.sort((a, b) => b.score - a.score);
    return results.slice(0, limit);
  }

  /**
   * Calculate cosine similarity between two vectors
   */
  private cosineSimilarity(a: Float32Array, b: Float32Array): number {
    if (a.length !== b.length) {
      throw new Error('Vectors must have same dimension');
    }

    let dotProduct = 0;
    let normA = 0;
    let normB = 0;

    for (let i = 0; i < a.length; i++) {
      dotProduct += a[i] * b[i];
      normA += a[i] * a[i];
      normB += b[i] * b[i];
    }

    normA = Math.sqrt(normA);
    normB = Math.sqrt(normB);

    if (normA === 0 || normB === 0) {
      return 0;
    }

    return dotProduct / (normA * normB);
  }

  /**
   * Get the number of vectors stored
   */
  size(): number {
    return this.vectors.size;
  }

  /**
   * Clear all vectors
   */
  clear(): void {
    this.vectors.clear();
  }
}
