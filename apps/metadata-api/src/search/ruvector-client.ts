/**
 * RuVector Client for Semantic Search
 *
 * High-performance vector search client using RuVector engine
 * with <100µs search latency and native SIMD optimizations.
 */

import { MediaMetadata, SearchResult } from '../types';

export interface RuVectorConfig {
  baseUrl?: string;
  dimension: number;
  metric?: 'cosine' | 'euclidean' | 'l2' | 'ip';
  maxElements?: number;
  efConstruction?: number;
  M?: number;
}

export interface VectorSearchResult {
  id: string;
  distance: number;
  similarity: number;
  metadata?: Record<string, any>;
}

/**
 * RuVector Client
 * Provides semantic search capabilities for media metadata
 */
export class RuVectorClient {
  private db: any;
  private config: RuVectorConfig;
  private initialized: boolean = false;
  private metadata: Map<string, MediaMetadata> = new Map();

  constructor(config: RuVectorConfig) {
    this.config = {
      dimension: config.dimension || 384,
      metric: config.metric || 'cosine',
      maxElements: config.maxElements || 100000,
      efConstruction: config.efConstruction || 200,
      M: config.M || 16,
      ...config
    };
  }

  /**
   * Initialize RuVector database connection
   */
  async connect(baseUrl?: string): Promise<void> {
    if (this.initialized) return;

    try {
      // Try to import RuVector package
      let VectorDB;
      try {
        const ruvector = await import('ruvector');
        VectorDB = ruvector.VectorDB || ruvector.default?.VectorDB;
      } catch {
        // Fallback to @ruvector/core
        const core = await import('@ruvector/core');
        VectorDB = core.VectorDB || core.default?.VectorDB;
      }

      if (!VectorDB) {
        throw new Error('RuVector VectorDB not found. Install: npm install ruvector');
      }

      // Initialize VectorDB
      this.db = new VectorDB({
        dimensions: this.config.dimension,
        metric: this.config.metric,
        maxElements: this.config.maxElements,
        efConstruction: this.config.efConstruction,
        m: this.config.M
      });

      this.initialized = true;
      console.log('[RuVectorClient] Connected to RuVector database');
    } catch (error) {
      const errorMessage = (error as Error).message;
      throw new Error(`Failed to initialize RuVector: ${errorMessage}`);
    }
  }

  /**
   * Add media metadata to vector index
   */
  async addMedia(metadata: MediaMetadata): Promise<void> {
    this.ensureInitialized();

    if (!metadata.embedding || metadata.embedding.length === 0) {
      throw new Error(`Media ${metadata.id} has no embedding vector`);
    }

    // Validate embedding dimension
    if (metadata.embedding.length !== this.config.dimension) {
      throw new Error(
        `Embedding dimension mismatch: expected ${this.config.dimension}, got ${metadata.embedding.length}`
      );
    }

    // Store metadata separately
    this.metadata.set(metadata.id, metadata);

    // Insert vector into RuVector
    const embedding = new Float32Array(metadata.embedding);
    this.db.insert({
      id: metadata.id,
      vector: embedding,
      metadata: {
        title: metadata.title,
        type: metadata.type,
        genres: metadata.genres,
        releaseYear: metadata.releaseDate?.getFullYear(),
        popularity: metadata.popularity
      }
    });
  }

  /**
   * Batch add multiple media items
   */
  async addMediaBatch(items: MediaMetadata[]): Promise<void> {
    this.ensureInitialized();

    for (const item of items) {
      await this.addMedia(item);
    }
  }

  /**
   * Semantic search using query text embedding
   */
  async search(query: string, limit: number = 10): Promise<SearchResult[]> {
    this.ensureInitialized();

    // In a real implementation, you would:
    // 1. Generate embedding for the query text using an embedding model
    // 2. Use that embedding for vector search
    // For now, we'll return empty results as we need an embedding service

    console.warn('[RuVectorClient] Search requires embedding generation service');
    return [];
  }

  /**
   * Search using pre-computed embedding vector
   */
  async searchByEmbedding(
    embedding: number[],
    limit: number = 10,
    threshold?: number
  ): Promise<SearchResult[]> {
    this.ensureInitialized();

    if (embedding.length !== this.config.dimension) {
      throw new Error(
        `Embedding dimension mismatch: expected ${this.config.dimension}, got ${embedding.length}`
      );
    }

    const queryVector = new Float32Array(embedding);
    const results = this.db.search({
      vector: queryVector,
      k: limit,
      threshold: threshold
    });

    return results
      .map((r: VectorSearchResult) => {
        const metadata = this.metadata.get(r.id);
        if (!metadata) return null;

        return {
          assetId: r.id,
          metadata: metadata,
          similarity: r.similarity,
          rank: undefined
        };
      })
      .filter((r: SearchResult | null): r is SearchResult => r !== null);
  }

  /**
   * Find similar content based on item ID
   */
  async getSimilar(itemId: string, limit: number = 10): Promise<SearchResult[]> {
    this.ensureInitialized();

    const sourceMetadata = this.metadata.get(itemId);
    if (!sourceMetadata || !sourceMetadata.embedding) {
      throw new Error(`Media ${itemId} not found or has no embedding`);
    }

    return this.searchByEmbedding(sourceMetadata.embedding, limit + 1).then(results =>
      // Filter out the source item itself
      results.filter(r => r.assetId !== itemId).slice(0, limit)
    );
  }

  /**
   * Get trending content based on popularity and recency
   */
  async getTrending(timeWindow: string = '7d'): Promise<MediaMetadata[]> {
    this.ensureInitialized();

    // Parse time window (e.g., '7d', '30d', '24h')
    const now = new Date();
    let cutoffDate = new Date(now);

    if (timeWindow.endsWith('d')) {
      const days = parseInt(timeWindow);
      cutoffDate.setDate(now.getDate() - days);
    } else if (timeWindow.endsWith('h')) {
      const hours = parseInt(timeWindow);
      cutoffDate.setHours(now.getHours() - hours);
    }

    // Get all metadata and filter/sort
    const allItems = Array.from(this.metadata.values());

    return allItems
      .filter(item => {
        // Filter by recency if releaseDate exists
        if (item.releaseDate) {
          return item.releaseDate >= cutoffDate;
        }
        return true;
      })
      .sort((a, b) => {
        // Sort by popularity score (descending)
        const popA = a.popularity || 0;
        const popB = b.popularity || 0;
        return popB - popA;
      })
      .slice(0, 20); // Return top 20 trending items
  }

  /**
   * Remove media from index
   */
  async removeMedia(itemId: string): Promise<boolean> {
    this.ensureInitialized();

    this.metadata.delete(itemId);
    return this.db.remove(itemId);
  }

  /**
   * Get index statistics
   */
  getStats(): {
    count: number;
    dimension: number;
    metric: string;
    memoryUsage?: number;
  } {
    this.ensureInitialized();

    return {
      count: this.db.count(),
      dimension: this.config.dimension,
      metric: this.config.metric,
      memoryUsage: this.db.memoryUsage?.()
    };
  }

  /**
   * Save index to disk
   */
  async save(path: string): Promise<void> {
    this.ensureInitialized();

    // Save vector index
    this.db.save(path);

    // Save metadata separately
    const metadataPath = path + '.meta.json';
    const fs = await import('fs/promises');
    const metadataObj = Object.fromEntries(this.metadata);
    await fs.writeFile(metadataPath, JSON.stringify(metadataObj, null, 2));
  }

  /**
   * Load index from disk
   */
  async load(path: string): Promise<void> {
    this.ensureInitialized();

    // Load vector index
    this.db.load(path);

    // Load metadata
    const metadataPath = path + '.meta.json';
    try {
      const fs = await import('fs/promises');
      const data = await fs.readFile(metadataPath, 'utf-8');
      const metadataObj = JSON.parse(data);
      this.metadata = new Map(Object.entries(metadataObj));
    } catch (error) {
      console.warn(`[RuVectorClient] No metadata file found at ${metadataPath}`);
    }
  }

  /**
   * Close connection and cleanup
   */
  close(): void {
    this.metadata.clear();
    this.initialized = false;
  }

  /**
   * Ensure client is initialized
   */
  private ensureInitialized(): void {
    if (!this.initialized) {
      throw new Error('RuVectorClient not initialized. Call connect() first.');
    }
  }
}
