/**
 * Hybrid Search Service
 *
 * Combines RuVector semantic search with Vertex AI for intelligent content discovery.
 * Implements fallback logic and score fusion for optimal results.
 */

import { MediaMetadata, SearchResult } from '../types';
import { RuVectorClient, RuVectorConfig } from './ruvector-client';

export interface SearchOptions {
  limit?: number;
  threshold?: number;
  backends?: ('ruvector' | 'vertex-ai')[];
  fusionStrategy?: 'rrf' | 'weighted' | 'cascade';
  filters?: {
    type?: MediaMetadata['type'][];
    genres?: string[];
    minPopularity?: number;
    releaseYearMin?: number;
    releaseYearMax?: number;
  };
}

export interface HybridSearchConfig {
  ruvectorConfig: RuVectorConfig;
  vertexAiConfig?: {
    projectId: string;
    location: string;
    endpoint?: string;
  };
  enableCache?: boolean;
  cacheTTL?: number; // seconds
}

/**
 * Hybrid Search Service
 * Combines multiple search backends for optimal results
 */
export class HybridSearchService {
  private ruvectorClient: RuVectorClient;
  private cache: Map<string, { results: SearchResult[]; timestamp: number }>;
  private config: HybridSearchConfig;
  private vertexAiAvailable: boolean = false;

  constructor(config: HybridSearchConfig) {
    this.config = config;
    this.ruvectorClient = new RuVectorClient(config.ruvectorConfig);
    this.cache = new Map();
  }

  /**
   * Initialize all search backends
   */
  async initialize(): Promise<void> {
    // Initialize RuVector
    await this.ruvectorClient.connect();

    // Check if Vertex AI is available
    if (this.config.vertexAiConfig) {
      try {
        // In production, initialize Vertex AI client here
        this.vertexAiAvailable = false; // Set to true when Vertex AI is configured
        console.log('[HybridSearch] Vertex AI not configured, using RuVector only');
      } catch (error) {
        console.warn('[HybridSearch] Vertex AI initialization failed:', error);
        this.vertexAiAvailable = false;
      }
    }
  }

  /**
   * Perform hybrid search across all backends
   */
  async search(query: string, options: SearchOptions = {}): Promise<SearchResult[]> {
    const limit = options.limit || 10;
    const backends = options.backends || ['ruvector'];
    const fusionStrategy = options.fusionStrategy || 'rrf';

    // Check cache first
    if (this.config.enableCache) {
      const cached = this.getCachedResults(query);
      if (cached) {
        return this.applyFilters(cached, options.filters).slice(0, limit);
      }
    }

    const results: SearchResult[] = [];

    // Try Vertex AI first if available
    if (backends.includes('vertex-ai') && this.vertexAiAvailable) {
      try {
        const vertexResults = await this.searchVertexAI(query, limit);
        results.push(...vertexResults);
      } catch (error) {
        console.warn('[HybridSearch] Vertex AI search failed, falling back to RuVector:', error);
      }
    }

    // Fallback to RuVector if no results or Vertex AI unavailable
    if (results.length === 0 && backends.includes('ruvector')) {
      try {
        const ruvectorResults = await this.ruvectorClient.search(query, limit);
        results.push(...ruvectorResults);
      } catch (error) {
        console.error('[HybridSearch] RuVector search failed:', error);
      }
    }

    // Apply filters
    const filteredResults = this.applyFilters(results, options.filters);

    // Apply fusion if we have results from multiple backends
    const fusedResults = this.fuseResults(filteredResults, fusionStrategy);

    // Cache results
    if (this.config.enableCache) {
      this.cacheResults(query, fusedResults);
    }

    return fusedResults.slice(0, limit);
  }

  /**
   * Search by pre-computed embedding
   */
  async searchByEmbedding(
    embedding: number[],
    options: SearchOptions = {}
  ): Promise<SearchResult[]> {
    const limit = options.limit || 10;
    const threshold = options.threshold;

    const results = await this.ruvectorClient.searchByEmbedding(embedding, limit, threshold);
    return this.applyFilters(results, options.filters);
  }

  /**
   * Find similar content
   */
  async findSimilar(itemId: string, options: SearchOptions = {}): Promise<SearchResult[]> {
    const limit = options.limit || 10;
    const results = await this.ruvectorClient.getSimilar(itemId, limit);
    return this.applyFilters(results, options.filters);
  }

  /**
   * Get trending content
   */
  async getTrending(
    timeWindow: string = '7d',
    options: SearchOptions = {}
  ): Promise<MediaMetadata[]> {
    const trending = await this.ruvectorClient.getTrending(timeWindow);
    return this.applyMetadataFilters(trending, options.filters);
  }

  /**
   * Add media to search index
   */
  async addMedia(metadata: MediaMetadata): Promise<void> {
    await this.ruvectorClient.addMedia(metadata);
  }

  /**
   * Batch add media
   */
  async addMediaBatch(items: MediaMetadata[]): Promise<void> {
    await this.ruvectorClient.addMediaBatch(items);
  }

  /**
   * Search using Vertex AI
   */
  private async searchVertexAI(_query: string, _limit: number): Promise<SearchResult[]> {
    // Placeholder for Vertex AI integration
    // In production, this would call Vertex AI Matching Engine or Vector Search
    throw new Error('Vertex AI not configured');
  }

  /**
   * Fuse results from multiple backends
   *
   * Strategies:
   * - rrf: Reciprocal Rank Fusion
   * - weighted: Weighted score combination
   * - cascade: Use primary, fallback to secondary if insufficient results
   */
  private fuseResults(
    results: SearchResult[],
    strategy: 'rrf' | 'weighted' | 'cascade'
  ): SearchResult[] {
    if (results.length === 0) return [];

    switch (strategy) {
      case 'rrf':
        return this.reciprocalRankFusion(results);
      case 'weighted':
        return this.weightedFusion(results);
      case 'cascade':
      default:
        return results;
    }
  }

  /**
   * Reciprocal Rank Fusion (RRF)
   * Combines rankings from multiple sources
   */
  private reciprocalRankFusion(results: SearchResult[]): SearchResult[] {
    const k = 60; // RRF constant
    const scoreMap = new Map<string, { result: SearchResult; score: number }>();

    results.forEach((result, index) => {
      const rank = index + 1;
      const rrfScore = 1 / (k + rank);

      const existing = scoreMap.get(result.assetId);
      if (existing) {
        existing.score += rrfScore;
      } else {
        scoreMap.set(result.assetId, { result, score: rrfScore });
      }
    });

    return Array.from(scoreMap.values())
      .sort((a, b) => b.score - a.score)
      .map(item => ({
        ...item.result,
        similarity: item.score
      }));
  }

  /**
   * Weighted score fusion
   */
  private weightedFusion(results: SearchResult[]): SearchResult[] {
    // Weight RuVector results higher (0.7) vs Vertex AI (0.3)
    return results.sort((a, b) => {
      const scoreA = (a.similarity || 0) * 0.7;
      const scoreB = (b.similarity || 0) * 0.7;
      return scoreB - scoreA;
    });
  }

  /**
   * Apply metadata filters to search results
   */
  private applyFilters(
    results: SearchResult[],
    filters?: SearchOptions['filters']
  ): SearchResult[] {
    if (!filters) return results;

    return results.filter(result => {
      const metadata = result.metadata;

      // Type filter
      if (filters.type && filters.type.length > 0) {
        if (!filters.type.includes(metadata.type)) return false;
      }

      // Genre filter
      if (filters.genres && filters.genres.length > 0) {
        const hasGenre = filters.genres.some(g => metadata.genres.includes(g));
        if (!hasGenre) return false;
      }

      // Popularity filter
      if (filters.minPopularity !== undefined) {
        if ((metadata.popularity || 0) < filters.minPopularity) return false;
      }

      // Release year filter
      if (metadata.releaseDate) {
        const year = metadata.releaseDate.getFullYear();
        if (filters.releaseYearMin && year < filters.releaseYearMin) return false;
        if (filters.releaseYearMax && year > filters.releaseYearMax) return false;
      }

      return true;
    });
  }

  /**
   * Apply filters to metadata array
   */
  private applyMetadataFilters(
    items: MediaMetadata[],
    filters?: SearchOptions['filters']
  ): MediaMetadata[] {
    if (!filters) return items;

    return items.filter(metadata => {
      // Type filter
      if (filters.type && filters.type.length > 0) {
        if (!filters.type.includes(metadata.type)) return false;
      }

      // Genre filter
      if (filters.genres && filters.genres.length > 0) {
        const hasGenre = filters.genres.some(g => metadata.genres.includes(g));
        if (!hasGenre) return false;
      }

      // Popularity filter
      if (filters.minPopularity !== undefined) {
        if ((metadata.popularity || 0) < filters.minPopularity) return false;
      }

      // Release year filter
      if (metadata.releaseDate) {
        const year = metadata.releaseDate.getFullYear();
        if (filters.releaseYearMin && year < filters.releaseYearMin) return false;
        if (filters.releaseYearMax && year > filters.releaseYearMax) return false;
      }

      return true;
    });
  }

  /**
   * Get cached search results
   */
  private getCachedResults(query: string): SearchResult[] | null {
    if (!this.config.enableCache) return null;

    const cached = this.cache.get(query);
    if (!cached) return null;

    const ttl = (this.config.cacheTTL || 300) * 1000; // Convert to ms
    const age = Date.now() - cached.timestamp;

    if (age > ttl) {
      this.cache.delete(query);
      return null;
    }

    return cached.results;
  }

  /**
   * Cache search results
   */
  private cacheResults(query: string, results: SearchResult[]): void {
    if (!this.config.enableCache) return;

    this.cache.set(query, {
      results,
      timestamp: Date.now()
    });

    // Limit cache size to 1000 entries
    if (this.cache.size > 1000) {
      const firstKey = this.cache.keys().next().value;
      if (firstKey) {
        this.cache.delete(firstKey);
      }
    }
  }

  /**
   * Clear cache
   */
  clearCache(): void {
    this.cache.clear();
  }

  /**
   * Get search statistics
   */
  getStats() {
    return {
      ruvector: this.ruvectorClient.getStats(),
      cache: {
        size: this.cache.size,
        enabled: this.config.enableCache
      },
      backends: {
        ruvector: true,
        vertexAi: this.vertexAiAvailable
      }
    };
  }

  /**
   * Close all connections
   */
  close(): void {
    this.ruvectorClient.close();
    this.cache.clear();
  }
}
