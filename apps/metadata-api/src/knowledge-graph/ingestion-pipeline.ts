/**
 * Knowledge Graph Ingestion Pipeline with Embeddings
 *
 * Enhanced pipeline that:
 * 1. Streams TMDB data from GCS
 * 2. Sorts by popularity to prioritize top 100k movies
 * 3. Generates Vertex AI embeddings for semantic search
 * 4. Stores everything in Firestore
 */

import { logger } from '../utils/logger';
import { GCSReader, getGCSReader } from './gcs-reader';
import { KnowledgeGraphProcessor, getProcessor, ProcessedMovie } from './processor';
import { KnowledgeGraphStore, getStore } from './store';
import { VertexAIEmbeddings, getEmbeddingsInstance } from '../vertex-ai/embeddings';
import { TMDBMovieRow } from './schema';
import { quickValidateMovie, getDistributionStatus } from './platform-validator';

/**
 * Ingestion configuration
 */
export interface IngestionConfig {
  limit: number;                    // Max movies to process
  batchSize: number;                // Batch size for processing
  generateEmbeddings: boolean;      // Whether to generate embeddings
  embeddingBatchSize: number;       // Batch size for embedding generation
  sortByPopularity: boolean;        // Sort by popularity before processing
  minVoteCount: number;             // Minimum vote count filter
  progressCallback?: ProgressCallback;
}

export type ProgressCallback = (progress: IngestionProgress) => void;

export interface IngestionProgress {
  phase: 'loading' | 'processing' | 'embeddings' | 'storing' | 'complete';
  processed: number;
  total: number;
  currentMovie?: string;
  embeddingsGenerated?: number;
  errors: number;
  elapsedMs: number;
}

export interface IngestionResult {
  success: boolean;
  stats: {
    totalProcessed: number;
    moviesStored: number;
    embeddingsGenerated: number;
    errors: number;
    durationMs: number;
    genresFound: number;
    companiesFound: number;
    countriesFound: number;
    languagesFound: number;
    // Platform readiness counts
    netflixReady: number;
    amazonReady: number;
    fastReady: number;
  };
  error?: string;
}

const DEFAULT_CONFIG: IngestionConfig = {
  limit: 100000,           // 100k movies
  batchSize: 100,
  generateEmbeddings: true,
  embeddingBatchSize: 5,   // Vertex AI recommends 5
  sortByPopularity: true,
  minVoteCount: 10,        // Filter out low-quality entries
};

/**
 * Knowledge Graph Ingestion Pipeline
 */
export class IngestionPipeline {
  private gcsReader: GCSReader;
  private processor: KnowledgeGraphProcessor;
  private store: KnowledgeGraphStore;
  private embeddings: VertexAIEmbeddings | null = null;
  private config: IngestionConfig;

  constructor(config?: Partial<IngestionConfig>) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.gcsReader = getGCSReader();
    this.processor = getProcessor();
    this.store = getStore();

    if (this.config.generateEmbeddings) {
      this.embeddings = getEmbeddingsInstance();
    }

    logger.info('Ingestion Pipeline initialized', {
      limit: this.config.limit,
      generateEmbeddings: this.config.generateEmbeddings,
    });
  }

  /**
   * Run the full ingestion pipeline
   */
  async run(): Promise<IngestionResult> {
    const startTime = Date.now();
    let totalProcessed = 0;
    let moviesStored = 0;
    let embeddingsGenerated = 0;
    let errors = 0;
    let netflixReady = 0;
    let amazonReady = 0;
    let fastReady = 0;

    try {
      logger.info('Starting ingestion pipeline', { config: this.config });

      // Phase 1: Load and sort movies by popularity
      this.reportProgress({
        phase: 'loading',
        processed: 0,
        total: this.config.limit,
        errors: 0,
        elapsedMs: Date.now() - startTime,
      });

      const allRows: TMDBMovieRow[] = [];

      // Stream all rows first to sort by popularity
      for await (const row of this.gcsReader.streamRows()) {
        const voteCount = parseInt(row.vote_count || '0', 10);
        const popularity = parseFloat(row.popularity || '0');

        // Filter by minimum vote count for quality
        if (voteCount >= this.config.minVoteCount && popularity > 0) {
          allRows.push(row);
        }

        // Progress update every 10k rows
        if (allRows.length % 10000 === 0) {
          this.reportProgress({
            phase: 'loading',
            processed: allRows.length,
            total: this.config.limit,
            errors: 0,
            elapsedMs: Date.now() - startTime,
          });
        }

        // Stop if we have enough candidates (2x limit for filtering margin)
        if (allRows.length >= this.config.limit * 2) {
          break;
        }
      }

      logger.info(`Loaded ${allRows.length} candidate movies`);

      // Sort by popularity (descending)
      if (this.config.sortByPopularity) {
        allRows.sort((a, b) => {
          const popA = parseFloat(a.popularity || '0');
          const popB = parseFloat(b.popularity || '0');
          return popB - popA;
        });
      }

      // Take top N movies
      const topMovies = allRows.slice(0, this.config.limit);
      logger.info(`Selected top ${topMovies.length} movies by popularity`);

      // Phase 2: Process movies into graph nodes
      this.reportProgress({
        phase: 'processing',
        processed: 0,
        total: topMovies.length,
        errors: 0,
        elapsedMs: Date.now() - startTime,
      });

      const processedMovies: ProcessedMovie[] = [];

      for (let i = 0; i < topMovies.length; i += this.config.batchSize) {
        const batch = topMovies.slice(i, i + this.config.batchSize);
        const { movies, stats } = this.processor.processBatch(batch);

        processedMovies.push(...movies);
        totalProcessed += stats.totalRows || 0;
        errors += stats.failedRows || 0;

        this.reportProgress({
          phase: 'processing',
          processed: totalProcessed,
          total: topMovies.length,
          currentMovie: batch[batch.length - 1]?.title,
          errors,
          elapsedMs: Date.now() - startTime,
        });
      }

      logger.info(`Processed ${processedMovies.length} movies into graph nodes`);

      // Phase 3: Generate embeddings
      if (this.config.generateEmbeddings && this.embeddings) {
        this.reportProgress({
          phase: 'embeddings',
          processed: 0,
          total: processedMovies.length,
          embeddingsGenerated: 0,
          errors,
          elapsedMs: Date.now() - startTime,
        });

        for (let i = 0; i < processedMovies.length; i += this.config.embeddingBatchSize) {
          const batch = processedMovies.slice(i, i + this.config.embeddingBatchSize);

          try {
            // Generate embedding text from title + overview
            const texts = batch.map(m => {
              const text = `${m.movie.title}. ${m.movie.overview || ''}`.trim();
              return text.substring(0, 2000); // Limit text length
            });

            const embeddings = await this.embeddings.generateBatchEmbeddings(texts);

            // Attach embeddings to movies
            for (let j = 0; j < batch.length; j++) {
              if (embeddings[j] && embeddings[j].length > 0) {
                batch[j].movie.embedding = embeddings[j];
                batch[j].movie.embeddingModel = 'text-embedding-004';
                embeddingsGenerated++;
              }
            }
          } catch (error) {
            logger.error('Embedding generation failed for batch', {
              batchStart: i,
              error: error instanceof Error ? error.message : 'Unknown',
            });
            // Continue without embeddings for this batch
          }

          this.reportProgress({
            phase: 'embeddings',
            processed: Math.min(i + this.config.embeddingBatchSize, processedMovies.length),
            total: processedMovies.length,
            embeddingsGenerated,
            currentMovie: batch[batch.length - 1]?.movie.title,
            errors,
            elapsedMs: Date.now() - startTime,
          });

          // Rate limiting delay
          await this.sleep(100);
        }

        logger.info(`Generated ${embeddingsGenerated} embeddings`);
      }

      // Phase 3.5: Validate for platform distribution
      logger.info('Validating movies for platform distribution...');
      for (const processed of processedMovies) {
        try {
          const readiness = quickValidateMovie(processed.movie);

          // Set platform readiness on the movie
          (processed.movie as any).platformReadiness = {
            netflix: readiness.netflix,
            amazon: readiness.amazon,
            fast: readiness.fast,
            validatedAt: new Date().toISOString(),
          };

          // Set distribution status
          processed.movie.distributionStatus = getDistributionStatus(readiness);

          // Track counts
          if (readiness.netflix) netflixReady++;
          if (readiness.amazon) amazonReady++;
          if (readiness.fast) fastReady++;
        } catch (error) {
          logger.error('Platform validation failed for movie', {
            movieId: processed.movie.id,
            error: error instanceof Error ? error.message : 'Unknown',
          });
          processed.movie.distributionStatus = 'failed';
        }
      }
      logger.info(`Platform validation complete: Netflix=${netflixReady}, Amazon=${amazonReady}, FAST=${fastReady}`);

      // Phase 4: Store in Firestore
      this.reportProgress({
        phase: 'storing',
        processed: 0,
        total: processedMovies.length,
        embeddingsGenerated,
        errors,
        elapsedMs: Date.now() - startTime,
      });

      // Store in batches
      const storeBatchSize = 50;
      for (let i = 0; i < processedMovies.length; i += storeBatchSize) {
        const batch = processedMovies.slice(i, i + storeBatchSize);

        try {
          const result = await this.store.storeProcessedMoviesBatch(batch);
          moviesStored += result.stored;
        } catch (error) {
          logger.error('Store batch failed', {
            batchStart: i,
            error: error instanceof Error ? error.message : 'Unknown',
          });
          errors += batch.length;
        }

        this.reportProgress({
          phase: 'storing',
          processed: Math.min(i + storeBatchSize, processedMovies.length),
          total: processedMovies.length,
          embeddingsGenerated,
          errors,
          elapsedMs: Date.now() - startTime,
        });
      }

      // Get cache stats
      const cacheStats = this.processor.getCacheStats();

      // Complete
      this.reportProgress({
        phase: 'complete',
        processed: totalProcessed,
        total: this.config.limit,
        embeddingsGenerated,
        errors,
        elapsedMs: Date.now() - startTime,
      });

      const durationMs = Date.now() - startTime;

      logger.info('Ingestion pipeline completed', {
        totalProcessed,
        moviesStored,
        embeddingsGenerated,
        errors,
        durationMs,
        cacheStats,
      });

      return {
        success: true,
        stats: {
          totalProcessed,
          moviesStored,
          embeddingsGenerated,
          errors,
          durationMs,
          genresFound: cacheStats.genres,
          companiesFound: cacheStats.companies,
          countriesFound: cacheStats.countries,
          languagesFound: cacheStats.languages,
          netflixReady,
          amazonReady,
          fastReady,
        },
      };
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      logger.error('Ingestion pipeline failed', { error: errorMessage });

      return {
        success: false,
        stats: {
          totalProcessed,
          moviesStored,
          embeddingsGenerated,
          errors,
          durationMs: Date.now() - startTime,
          genresFound: 0,
          companiesFound: 0,
          countriesFound: 0,
          languagesFound: 0,
          netflixReady: 0,
          amazonReady: 0,
          fastReady: 0,
        },
        error: errorMessage,
      };
    }
  }

  /**
   * Run a quick test ingestion (100 movies, no embeddings)
   */
  async runQuickTest(): Promise<IngestionResult> {
    const originalConfig = { ...this.config };
    this.config = {
      ...this.config,
      limit: 100,
      generateEmbeddings: false,
      minVoteCount: 0,
    };

    try {
      return await this.run();
    } finally {
      this.config = originalConfig;
    }
  }

  /**
   * Report progress
   */
  private reportProgress(progress: IngestionProgress): void {
    if (this.config.progressCallback) {
      this.config.progressCallback(progress);
    }

    // Log significant milestones
    if (progress.processed > 0 && progress.processed % 10000 === 0) {
      logger.info(`Ingestion progress: ${progress.phase}`, {
        processed: progress.processed,
        total: progress.total,
        embeddingsGenerated: progress.embeddingsGenerated,
        errors: progress.errors,
        elapsedMs: progress.elapsedMs,
      });
    }
  }

  /**
   * Sleep utility
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

/**
 * Factory function
 */
export function createIngestionPipeline(config?: Partial<IngestionConfig>): IngestionPipeline {
  return new IngestionPipeline(config);
}
