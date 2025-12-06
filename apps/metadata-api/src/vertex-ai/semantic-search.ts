/**
 * Semantic Search Service
 *
 * High-level service combining Vertex AI Embeddings and Matching Engine
 * for intelligent content discovery and recommendation.
 */

import { VertexAIEmbeddings, getEmbeddingsInstance } from './embeddings';
import { MatchingEngineClient, getMatchingEngineInstance, VectorEntry, MatchingEngineSearchResult } from './matching-engine';
import { MediaMetadata, SearchResult } from '../types';
import winston from 'winston';

// Logger configuration
const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: winston.format.json(),
  transports: [
    new winston.transports.Console({
      format: winston.format.combine(
        winston.format.colorize(),
        winston.format.simple()
      )
    })
  ]
});

/**
 * Search filters for semantic search
 */
export interface SearchFilters {
  genres?: string[];
  type?: 'movie' | 'series' | 'episode' | 'documentary' | 'short';
  releaseYear?: { min?: number; max?: number };
  rating?: string[];
  platforms?: string[];
  language?: string;
  minSimilarity?: number;
}

/**
 * Indexing options
 */
export interface IndexingOptions {
  updateExisting?: boolean;
  generateEmbedding?: boolean;
  batchSize?: number;
}

/**
 * Search options
 */
export interface SearchOptions {
  limit?: number;
  offset?: number;
  filters?: SearchFilters;
  includeMetadata?: boolean;
  minScore?: number;
}

/**
 * Semantic search result with enriched metadata
 */
export interface SemanticSearchResult extends SearchResult {
  score: number;
  matchReason?: string;
  highlights?: string[];
}

/**
 * Search performance metrics
 */
export interface SearchMetrics {
  totalResults: number;
  embeddingLatencyMs: number;
  searchLatencyMs: number;
  totalLatencyMs: number;
  filteredResults: number;
}

/**
 * SemanticSearchService Class
 *
 * Orchestrates semantic search operations using Vertex AI embeddings
 * and Matching Engine for content discovery.
 */
export class SemanticSearchService {
  private embeddings: VertexAIEmbeddings;
  private matchingEngine: MatchingEngineClient;
  private indexId: string;
  private metadataCache: Map<string, MediaMetadata>;

  constructor(
    indexId: string = 'nexus-ummid-main',
    embeddings?: VertexAIEmbeddings,
    matchingEngine?: MatchingEngineClient
  ) {
    this.indexId = indexId;
    this.embeddings = embeddings || getEmbeddingsInstance();
    this.matchingEngine = matchingEngine || getMatchingEngineInstance();
    this.metadataCache = new Map();

    logger.info('SemanticSearchService initialized', { indexId });
  }

  /**
   * Search for content using natural language query
   *
   * @param query - Natural language search query
   * @param options - Search options and filters
   * @returns Array of search results with similarity scores
   */
  async search(
    query: string,
    options: SearchOptions = {}
  ): Promise<SemanticSearchResult[]> {
    if (!query || query.trim().length === 0) {
      throw new Error('Search query cannot be empty');
    }

    const startTime = Date.now();
    const limit = options.limit || 10;
    const minScore = options.minScore || 0.7;

    try {
      logger.info('Executing semantic search', {
        query: query.substring(0, 100),
        limit,
        filters: options.filters
      });

      // Step 1: Generate embedding for query
      const embeddingStartTime = Date.now();
      const queryEmbedding = await this.embeddings.generateEmbedding(query);
      const embeddingLatency = Date.now() - embeddingStartTime;

      logger.debug('Query embedding generated', {
        dimensions: queryEmbedding.length,
        latencyMs: embeddingLatency
      });

      // Step 2: Search for similar vectors
      const searchStartTime = Date.now();
      const rawResults = await this.matchingEngine.findNeighbors(
        this.indexId,
        queryEmbedding,
        limit * 2, // Fetch extra for filtering
        this.convertFiltersToMetadata(options.filters)
      );
      const searchLatency = Date.now() - searchStartTime;

      logger.debug('Vector search completed', {
        resultCount: rawResults.length,
        latencyMs: searchLatency
      });

      // Step 3: Convert to semantic search results
      const results = await this.enrichSearchResults(
        rawResults,
        query,
        minScore,
        options.includeMetadata !== false
      );

      // Step 4: Apply additional filters
      const filteredResults = this.applyFilters(results, options.filters);

      // Step 5: Limit results
      const finalResults = filteredResults.slice(0, limit);

      const totalLatency = Date.now() - startTime;

      logger.info('Semantic search completed', {
        query: query.substring(0, 50),
        totalResults: rawResults.length,
        filteredResults: filteredResults.length,
        returnedResults: finalResults.length,
        embeddingLatencyMs: embeddingLatency,
        searchLatencyMs: searchLatency,
        totalLatencyMs: totalLatency
      });

      return finalResults;

    } catch (error) {
      logger.error('Semantic search failed', {
        error: error instanceof Error ? error.message : 'Unknown error',
        query: query.substring(0, 50)
      });
      throw error;
    }
  }

  /**
   * Index content metadata for semantic search
   *
   * @param metadata - Media metadata to index
   * @param options - Indexing options
   * @returns Success status
   */
  async indexContent(
    metadata: MediaMetadata,
    options: IndexingOptions = {}
  ): Promise<boolean> {
    const startTime = Date.now();

    try {
      logger.info('Indexing content', {
        assetId: metadata.id,
        title: metadata.title,
        type: metadata.type
      });

      // Generate embedding if needed
      let embedding = metadata.embedding;

      if (!embedding || options.generateEmbedding) {
        const textToEmbed = this.prepareTextForEmbedding(metadata);
        embedding = await this.embeddings.generateEmbedding(textToEmbed);

        logger.debug('Embedding generated for content', {
          assetId: metadata.id,
          textLength: textToEmbed.length,
          dimensions: embedding.length
        });
      }

      // Prepare vector entry
      const vectorEntry: VectorEntry = {
        id: metadata.id,
        embedding,
        metadata: this.extractSearchableMetadata(metadata)
      };

      // Upsert to matching engine
      await this.matchingEngine.upsertVectors(this.indexId, [vectorEntry]);

      // Update cache
      this.metadataCache.set(metadata.id, metadata);

      const latency = Date.now() - startTime;

      logger.info('Content indexed', {
        assetId: metadata.id,
        latencyMs: latency
      });

      return true;

    } catch (error) {
      logger.error('Content indexing failed', {
        error: error instanceof Error ? error.message : 'Unknown error',
        assetId: metadata.id
      });
      throw error;
    }
  }

  /**
   * Batch index multiple content items
   *
   * @param metadataList - Array of media metadata
   * @param options - Indexing options
   * @returns Number of successfully indexed items
   */
  async indexBatch(
    metadataList: MediaMetadata[],
    options: IndexingOptions = {}
  ): Promise<number> {
    if (!metadataList || metadataList.length === 0) {
      throw new Error('Metadata list cannot be empty');
    }

    const startTime = Date.now();
    const batchSize = options.batchSize || 50;
    let successCount = 0;

    try {
      logger.info('Starting batch indexing', {
        totalItems: metadataList.length,
        batchSize
      });

      // Process in batches
      for (let i = 0; i < metadataList.length; i += batchSize) {
        const batch = metadataList.slice(i, i + batchSize);

        // Generate embeddings for batch
        const textsToEmbed = batch.map(m => this.prepareTextForEmbedding(m));
        const embeddings = await this.embeddings.generateBatchEmbeddings(textsToEmbed);

        // Prepare vector entries
        const vectorEntries: VectorEntry[] = batch.map((metadata, index) => ({
          id: metadata.id,
          embedding: embeddings[index],
          metadata: this.extractSearchableMetadata(metadata)
        }));

        // Upsert batch
        await this.matchingEngine.upsertVectors(this.indexId, vectorEntries);

        // Update cache
        batch.forEach(metadata => {
          this.metadataCache.set(metadata.id, metadata);
        });

        successCount += batch.length;

        logger.debug('Batch indexed', {
          batchNumber: Math.floor(i / batchSize) + 1,
          itemsInBatch: batch.length,
          totalIndexed: successCount
        });
      }

      const latency = Date.now() - startTime;

      logger.info('Batch indexing completed', {
        totalItems: metadataList.length,
        successCount,
        latencyMs: latency,
        avgLatencyPerItem: Math.round(latency / successCount)
      });

      return successCount;

    } catch (error) {
      logger.error('Batch indexing failed', {
        error: error instanceof Error ? error.message : 'Unknown error',
        totalItems: metadataList.length,
        successCount
      });
      throw error;
    }
  }

  /**
   * Find similar content to a given item
   *
   * @param assetId - Asset ID to find similar content for
   * @param limit - Maximum number of results
   * @returns Array of similar content
   */
  async findSimilar(
    assetId: string,
    limit: number = 10
  ): Promise<SemanticSearchResult[]> {
    try {
      // Get metadata from cache or fetch
      const metadata = this.metadataCache.get(assetId);

      if (!metadata) {
        throw new Error(`Asset not found in cache: ${assetId}`);
      }

      // Use embedding if available, otherwise generate
      let queryEmbedding = metadata.embedding;

      if (!queryEmbedding) {
        const textToEmbed = this.prepareTextForEmbedding(metadata);
        queryEmbedding = await this.embeddings.generateEmbedding(textToEmbed);
      }

      // Search for similar vectors
      const rawResults = await this.matchingEngine.findNeighbors(
        this.indexId,
        queryEmbedding,
        limit + 1 // Add 1 to account for self-match
      );

      // Filter out self-match and enrich results
      const results = await this.enrichSearchResults(
        rawResults.filter(r => r.id !== assetId),
        metadata.title,
        0.5,
        true
      );

      logger.info('Similar content found', {
        assetId,
        resultCount: results.length
      });

      return results.slice(0, limit);

    } catch (error) {
      logger.error('Find similar failed', {
        error: error instanceof Error ? error.message : 'Unknown error',
        assetId
      });
      throw error;
    }
  }

  /**
   * Prepare text for embedding generation
   *
   * @param metadata - Media metadata
   * @returns Concatenated text representation
   */
  private prepareTextForEmbedding(metadata: MediaMetadata): string {
    const parts: string[] = [
      metadata.title,
      metadata.synopsis || '',
      metadata.description || '',
      metadata.genres.join(', '),
      metadata.keywords.join(', '),
      metadata.moodTags?.join(', ') || '',
      metadata.themes?.join(', ') || ''
    ];

    return parts.filter(p => p.trim().length > 0).join('. ');
  }

  /**
   * Extract searchable metadata for filtering
   *
   * @param metadata - Media metadata
   * @returns Searchable metadata object
   */
  private extractSearchableMetadata(metadata: MediaMetadata): Record<string, any> {
    return {
      type: metadata.type,
      genres: metadata.genres,
      rating: metadata.rating,
      language: metadata.language,
      releaseYear: metadata.releaseDate ? new Date(metadata.releaseDate).getFullYear() : null,
      platforms: metadata.platforms?.map(p => p.platform) || []
    };
  }

  /**
   * Convert search filters to metadata format
   *
   * @param filters - Search filters
   * @returns Metadata object for matching engine
   */
  private convertFiltersToMetadata(filters?: SearchFilters): Record<string, any> | undefined {
    if (!filters) return undefined;

    const metadata: Record<string, any> = {};

    if (filters.type) metadata.type = filters.type;
    if (filters.genres) metadata.genres = filters.genres;
    if (filters.rating) metadata.rating = filters.rating;
    if (filters.language) metadata.language = filters.language;
    if (filters.platforms) metadata.platforms = filters.platforms;

    return Object.keys(metadata).length > 0 ? metadata : undefined;
  }

  /**
   * Enrich search results with metadata
   *
   * @param rawResults - Raw matching engine results
   * @param query - Original query
   * @param minScore - Minimum similarity score
   * @param includeMetadata - Whether to include full metadata
   * @returns Enriched search results
   */
  private async enrichSearchResults(
    rawResults: MatchingEngineSearchResult[],
    query: string,
    minScore: number,
    includeMetadata: boolean
  ): Promise<SemanticSearchResult[]> {
    const results: SemanticSearchResult[] = [];

    for (const rawResult of rawResults) {
      // Convert distance to similarity score (0-1)
      const score = this.distanceToSimilarity(rawResult.distance);

      if (score < minScore) continue;

      const metadata = this.metadataCache.get(rawResult.id);

      if (!metadata && includeMetadata) {
        logger.warn('Metadata not found in cache', { assetId: rawResult.id });
        continue;
      }

      results.push({
        assetId: rawResult.id,
        metadata: metadata!,
        similarity: score,
        score,
        rank: results.length + 1,
        matchReason: this.generateMatchReason(metadata!, query)
      });
    }

    return results;
  }

  /**
   * Convert distance metric to similarity score
   *
   * @param distance - Distance from matching engine
   * @returns Similarity score (0-1)
   */
  private distanceToSimilarity(distance: number): number {
    // For cosine distance: similarity = 1 - distance
    // Distance ranges from 0 (identical) to 2 (opposite)
    return Math.max(0, Math.min(1, 1 - distance / 2));
  }

  /**
   * Generate human-readable match reason
   *
   * @param metadata - Content metadata
   * @param query - Search query
   * @returns Match reason string
   */
  private generateMatchReason(metadata: MediaMetadata, query: string): string {
    const reasons: string[] = [];

    if (metadata.genres.some(g => query.toLowerCase().includes(g.toLowerCase()))) {
      reasons.push('matching genre');
    }

    if (metadata.keywords.some(k => query.toLowerCase().includes(k.toLowerCase()))) {
      reasons.push('related keywords');
    }

    if (metadata.moodTags?.some(t => query.toLowerCase().includes(t.toLowerCase()))) {
      reasons.push('similar mood');
    }

    return reasons.length > 0 ? reasons.join(', ') : 'semantic similarity';
  }

  /**
   * Apply post-search filters
   *
   * @param results - Search results
   * @param filters - Search filters
   * @returns Filtered results
   */
  private applyFilters(
    results: SemanticSearchResult[],
    filters?: SearchFilters
  ): SemanticSearchResult[] {
    if (!filters) return results;

    return results.filter(result => {
      const metadata = result.metadata;

      // Release year filter
      if (filters.releaseYear && metadata.releaseDate) {
        const year = new Date(metadata.releaseDate).getFullYear();
        if (filters.releaseYear.min && year < filters.releaseYear.min) return false;
        if (filters.releaseYear.max && year > filters.releaseYear.max) return false;
      }

      // Minimum similarity filter
      if (filters.minSimilarity && result.score < filters.minSimilarity) {
        return false;
      }

      return true;
    });
  }

  /**
   * Clear metadata cache
   */
  clearCache(): void {
    this.metadataCache.clear();
    logger.info('Metadata cache cleared');
  }

  /**
   * Get cache statistics
   */
  getCacheStats() {
    return {
      size: this.metadataCache.size,
      keys: Array.from(this.metadataCache.keys())
    };
  }

  /**
   * Close all connections
   */
  async close(): Promise<void> {
    await this.embeddings.close();
    await this.matchingEngine.close();
    this.clearCache();
    logger.info('SemanticSearchService closed');
  }
}

/**
 * Singleton instance
 */
let semanticSearchInstance: SemanticSearchService | null = null;

/**
 * Get or create singleton instance
 *
 * @param indexId - Index identifier
 * @returns SemanticSearchService instance
 */
export function getSemanticSearchInstance(indexId?: string): SemanticSearchService {
  if (!semanticSearchInstance) {
    semanticSearchInstance = new SemanticSearchService(indexId);
  }
  return semanticSearchInstance;
}
