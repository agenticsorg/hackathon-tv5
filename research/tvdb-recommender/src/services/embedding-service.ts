/**
 * Embedding Service
 * Generates vector embeddings for content using local Transformers.js
 */

import { pipeline, Pipeline, FeatureExtractionPipeline } from '@xenova/transformers';
import pino from 'pino';
import NodeCache from 'node-cache';
import {
  ContentEmbedding,
  ContentMetadata,
  EmbeddingConfig,
  SeriesExtendedRecord,
  MovieBaseRecord,
  UserPreference
} from '../types/index.js';

const logger = pino({ name: 'embedding-service' });

export class EmbeddingService {
  private extractor: FeatureExtractionPipeline | null = null;
  private config: EmbeddingConfig;
  private cache: NodeCache;
  private initialized: boolean = false;

  constructor(config: Partial<EmbeddingConfig> = {}) {
    this.config = {
      model: config.model || 'Xenova/all-MiniLM-L6-v2',
      dimension: config.dimension || 384,
      batchSize: config.batchSize || 32,
      cacheEnabled: config.cacheEnabled !== false
    };

    this.cache = new NodeCache({
      stdTTL: 86400, // 24 hours
      checkperiod: 600,
      useClones: false
    });
  }

  /**
   * Initialize the embedding model
   */
  async initialize(): Promise<void> {
    if (this.initialized) return;

    try {
      logger.info({ model: this.config.model }, 'Initializing embedding model');

      this.extractor = await pipeline('feature-extraction', this.config.model, {
        quantized: true // Use quantized model for faster inference
      }) as FeatureExtractionPipeline;

      this.initialized = true;
      logger.info('Embedding model initialized successfully');
    } catch (error) {
      logger.error({ error }, 'Failed to initialize embedding model');
      throw error;
    }
  }

  /**
   * Ensure service is initialized
   */
  private ensureInitialized(): void {
    if (!this.initialized || !this.extractor) {
      throw new Error('EmbeddingService not initialized. Call initialize() first.');
    }
  }

  /**
   * Generate embedding for text
   */
  async embed(text: string): Promise<Float32Array> {
    this.ensureInitialized();

    // Check cache
    if (this.config.cacheEnabled) {
      const cacheKey = `embed:${this.hashText(text)}`;
      const cached = this.cache.get<Float32Array>(cacheKey);
      if (cached) return cached;
    }

    try {
      const output = await this.extractor!(text, {
        pooling: 'mean',
        normalize: true
      });

      // Convert to Float32Array
      const embedding = new Float32Array(output.data);

      // Cache result
      if (this.config.cacheEnabled) {
        const cacheKey = `embed:${this.hashText(text)}`;
        this.cache.set(cacheKey, embedding);
      }

      return embedding;
    } catch (error) {
      logger.error({ error, textLength: text.length }, 'Failed to generate embedding');
      throw error;
    }
  }

  /**
   * Generate embeddings for multiple texts (batch processing)
   */
  async embedBatch(texts: string[]): Promise<Float32Array[]> {
    this.ensureInitialized();

    const results: Float32Array[] = [];
    const uncachedTexts: { index: number; text: string }[] = [];

    // Check cache for each text
    if (this.config.cacheEnabled) {
      for (let i = 0; i < texts.length; i++) {
        const cacheKey = `embed:${this.hashText(texts[i])}`;
        const cached = this.cache.get<Float32Array>(cacheKey);
        if (cached) {
          results[i] = cached;
        } else {
          uncachedTexts.push({ index: i, text: texts[i] });
        }
      }
    } else {
      texts.forEach((text, i) => uncachedTexts.push({ index: i, text }));
    }

    // Process uncached texts in batches
    for (let i = 0; i < uncachedTexts.length; i += this.config.batchSize) {
      const batch = uncachedTexts.slice(i, i + this.config.batchSize);
      const batchTexts = batch.map(b => b.text);

      try {
        const outputs = await this.extractor!(batchTexts, {
          pooling: 'mean',
          normalize: true
        });

        // Process each output
        for (let j = 0; j < batch.length; j++) {
          const embedding = new Float32Array(
            outputs.data.slice(
              j * this.config.dimension,
              (j + 1) * this.config.dimension
            )
          );

          results[batch[j].index] = embedding;

          // Cache result
          if (this.config.cacheEnabled) {
            const cacheKey = `embed:${this.hashText(batch[j].text)}`;
            this.cache.set(cacheKey, embedding);
          }
        }
      } catch (error) {
        logger.error({ error, batchSize: batchTexts.length }, 'Failed to generate batch embeddings');
        throw error;
      }
    }

    return results;
  }

  /**
   * Generate embedding for series content
   */
  async embedSeries(series: SeriesExtendedRecord): Promise<ContentEmbedding> {
    const text = this.buildContentText({
      title: series.name,
      overview: series.overview || '',
      genres: series.genres?.map(g => g.name) || [],
      year: parseInt(series.year) || new Date(series.firstAired).getFullYear(),
      network: series.originalNetwork?.name
    });

    const vector = await this.embed(text);

    return {
      contentId: series.id.toString(),
      contentType: 'series',
      vector,
      metadata: {
        title: series.name,
        year: parseInt(series.year) || new Date(series.firstAired).getFullYear(),
        genres: series.genres?.map(g => g.name) || [],
        overview: series.overview || '',
        rating: series.score || 0,
        networkId: series.originalNetwork?.id,
        networkName: series.originalNetwork?.name,
        status: series.status?.name,
        language: series.originalLanguage,
        country: series.originalCountry
      },
      createdAt: new Date(),
      updatedAt: new Date()
    };
  }

  /**
   * Generate embedding for movie content
   */
  async embedMovie(movie: MovieBaseRecord & { genres?: { name: string }[]; overview?: string }): Promise<ContentEmbedding> {
    const text = this.buildContentText({
      title: movie.name,
      overview: movie.overview || '',
      genres: movie.genres?.map(g => g.name) || [],
      year: parseInt(movie.year)
    });

    const vector = await this.embed(text);

    return {
      contentId: movie.id.toString(),
      contentType: 'movie',
      vector,
      metadata: {
        title: movie.name,
        year: parseInt(movie.year),
        genres: movie.genres?.map(g => g.name) || [],
        overview: movie.overview || '',
        rating: movie.score || 0,
        status: movie.status?.name
      },
      createdAt: new Date(),
      updatedAt: new Date()
    };
  }

  /**
   * Generate user preference vector from watch history
   */
  async generateUserPreferenceVector(
    watchedContentEmbeddings: ContentEmbedding[],
    ratings: { contentId: string; rating: number }[]
  ): Promise<Float32Array> {
    if (watchedContentEmbeddings.length === 0) {
      // Return zero vector for cold start users
      return new Float32Array(this.config.dimension);
    }

    // Create rating weights map
    const ratingWeights = new Map(ratings.map(r => [r.contentId, r.rating / 10]));

    // Weighted average of content embeddings
    const aggregatedVector = new Float32Array(this.config.dimension);
    let totalWeight = 0;

    for (const content of watchedContentEmbeddings) {
      const weight = ratingWeights.get(content.contentId) || 0.5; // Default weight for unrated
      totalWeight += weight;

      for (let i = 0; i < this.config.dimension; i++) {
        aggregatedVector[i] += content.vector[i] * weight;
      }
    }

    // Normalize
    if (totalWeight > 0) {
      for (let i = 0; i < this.config.dimension; i++) {
        aggregatedVector[i] /= totalWeight;
      }
    }

    // L2 normalize the final vector
    return this.normalizeVector(aggregatedVector);
  }

  /**
   * Compute cosine similarity between two vectors
   */
  cosineSimilarity(a: Float32Array, b: Float32Array): number {
    if (a.length !== b.length) {
      throw new Error('Vectors must have the same dimension');
    }

    let dotProduct = 0;
    let normA = 0;
    let normB = 0;

    for (let i = 0; i < a.length; i++) {
      dotProduct += a[i] * b[i];
      normA += a[i] * a[i];
      normB += b[i] * b[i];
    }

    if (normA === 0 || normB === 0) return 0;

    return dotProduct / (Math.sqrt(normA) * Math.sqrt(normB));
  }

  /**
   * Find most similar content to a query embedding
   */
  findSimilar(
    queryEmbedding: Float32Array,
    contentEmbeddings: ContentEmbedding[],
    k: number = 10,
    threshold: number = 0.0
  ): { content: ContentEmbedding; similarity: number }[] {
    const results = contentEmbeddings
      .map(content => ({
        content,
        similarity: this.cosineSimilarity(queryEmbedding, content.vector)
      }))
      .filter(r => r.similarity >= threshold)
      .sort((a, b) => b.similarity - a.similarity)
      .slice(0, k);

    return results;
  }

  /**
   * Build content text for embedding
   */
  private buildContentText(content: {
    title: string;
    overview: string;
    genres: string[];
    year: number;
    network?: string;
  }): string {
    const parts = [
      content.title,
      content.overview,
      `Genres: ${content.genres.join(', ')}`,
      `Year: ${content.year}`
    ];

    if (content.network) {
      parts.push(`Network: ${content.network}`);
    }

    return parts.filter(Boolean).join('. ');
  }

  /**
   * L2 normalize a vector
   */
  private normalizeVector(vector: Float32Array): Float32Array {
    let norm = 0;
    for (let i = 0; i < vector.length; i++) {
      norm += vector[i] * vector[i];
    }
    norm = Math.sqrt(norm);

    if (norm === 0) return vector;

    const normalized = new Float32Array(vector.length);
    for (let i = 0; i < vector.length; i++) {
      normalized[i] = vector[i] / norm;
    }

    return normalized;
  }

  /**
   * Simple hash function for cache keys
   */
  private hashText(text: string): string {
    let hash = 0;
    for (let i = 0; i < text.length; i++) {
      const char = text.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32bit integer
    }
    return hash.toString(36);
  }

  /**
   * Get embedding dimension
   */
  getDimension(): number {
    return this.config.dimension;
  }

  /**
   * Get model name
   */
  getModelName(): string {
    return this.config.model;
  }

  /**
   * Clear embedding cache
   */
  clearCache(): void {
    this.cache.flushAll();
    logger.info('Embedding cache cleared');
  }

  /**
   * Get cache stats
   */
  getCacheStats(): { hits: number; misses: number; keys: number } {
    const stats = this.cache.getStats();
    return {
      hits: stats.hits,
      misses: stats.misses,
      keys: this.cache.keys().length
    };
  }
}

// Factory function
export function createEmbeddingService(config?: Partial<EmbeddingConfig>): EmbeddingService {
  return new EmbeddingService(config);
}

export default EmbeddingService;
