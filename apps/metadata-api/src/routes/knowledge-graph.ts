/**
 * Knowledge Graph API Routes
 *
 * REST API endpoints for:
 * - Movie browsing and querying
 * - Semantic search with embeddings
 * - Platform feed generation (Netflix/Amazon/FAST)
 * - Data ingestion with embeddings
 */

import { Router, Request, Response } from 'express';
import { query, param, validationResult } from 'express-validator';
import { logger } from '../utils/logger';
import {
  getStore,
  getProcessor,
  GraphQuery,
  MovieNode,
} from '../knowledge-graph';
import { createIngestionPipeline } from '../knowledge-graph/ingestion-pipeline';
import { getEmbeddingsInstance } from '../vertex-ai/embeddings';
import { NetflixIMFConnector } from '../connectors/netflix-imf';
import { AmazonMECConnector } from '../connectors/amazon-mec';
import { FASTMRSSConnector } from '../connectors/fast-mrss';
import { Platform } from '../connectors/types';
import { MediaMetadata } from '../types';

const router = Router();

// Connector instances
const netflixConnector = new NetflixIMFConnector();
const amazonConnector = new AmazonMECConnector();
const fastConnector = new FASTMRSSConnector(Platform.FAST_PLUTO);

/**
 * Validation error handler
 */
function handleValidationErrors(req: Request, res: Response): boolean {
  const errors = validationResult(req);
  if (!errors.isEmpty()) {
    res.status(400).json({ errors: errors.array() });
    return true;
  }
  return false;
}

/**
 * Convert MovieNode to MediaMetadata for connectors
 */
function movieToMediaMetadata(movie: MovieNode): MediaMetadata {
  return {
    id: movie.id,
    title: movie.title,
    type: 'movie',
    synopsis: movie.overview,
    genres: [], // Would need to query edges
    keywords: [], // Would need to query edges
    language: 'en',
    rating: movie.adult ? 'R' : 'PG-13',
    duration: movie.runtime,
    releaseDate: movie.releaseDate ? new Date(movie.releaseDate) : undefined,
    resolution: '4K',
    createdAt: new Date(movie.createdAt),
    updatedAt: new Date(movie.updatedAt),
  };
}

/**
 * Calculate cosine similarity between two vectors
 */
function cosineSimilarity(a: number[], b: number[]): number {
  if (a.length !== b.length) return 0;

  let dotProduct = 0;
  let normA = 0;
  let normB = 0;

  for (let i = 0; i < a.length; i++) {
    dotProduct += a[i] * b[i];
    normA += a[i] * a[i];
    normB += b[i] * b[i];
  }

  const denominator = Math.sqrt(normA) * Math.sqrt(normB);
  return denominator === 0 ? 0 : dotProduct / denominator;
}

// ============================================
// Movie Endpoints
// ============================================

/**
 * GET /api/v1/knowledge-graph/movies
 * Query movies with filters
 */
router.get(
  '/movies',
  [
    query('limit').optional().isInt({ min: 1, max: 100 }).toInt(),
    query('offset').optional().isInt({ min: 0 }).toInt(),
    query('genre').optional().isString(),
    query('yearMin').optional().isInt({ min: 1900 }).toInt(),
    query('yearMax').optional().isInt({ max: 2030 }).toInt(),
    query('ratingMin').optional().isFloat({ min: 0, max: 10 }).toFloat(),
    query('ratingMax').optional().isFloat({ min: 0, max: 10 }).toFloat(),
    query('sortBy').optional().isIn(['popularity', 'voteAverage', 'year', 'title']),
    query('sortOrder').optional().isIn(['asc', 'desc']),
  ],
  async (req: Request, res: Response): Promise<void> => {
    if (handleValidationErrors(req, res)) return;

    try {
      const store = getStore();

      const graphQuery: GraphQuery = {
        limit: req.query.limit ? Number(req.query.limit) : 20,
        offset: req.query.offset ? Number(req.query.offset) : 0,
        sortBy: req.query.sortBy as string,
        sortOrder: req.query.sortOrder as 'asc' | 'desc',
      };

      if (req.query.yearMin || req.query.yearMax) {
        graphQuery.yearRange = {
          min: req.query.yearMin ? Number(req.query.yearMin) : 1900,
          max: req.query.yearMax ? Number(req.query.yearMax) : 2030,
        };
      }

      if (req.query.ratingMin || req.query.ratingMax) {
        graphQuery.ratingRange = {
          min: req.query.ratingMin ? Number(req.query.ratingMin) : 0,
          max: req.query.ratingMax ? Number(req.query.ratingMax) : 10,
        };
      }

      // If genre filter, use genre-based query
      if (req.query.genre) {
        const movies = await store.getMoviesByGenre(
          req.query.genre as string,
          graphQuery.limit,
          graphQuery.offset
        );

        res.json({
          movies,
          pagination: {
            limit: graphQuery.limit,
            offset: graphQuery.offset,
            hasMore: movies.length === graphQuery.limit,
          },
        });
        return;
      }

      const result = await store.queryMovies(graphQuery);

      res.json({
        movies: result.nodes,
        pagination: {
          limit: graphQuery.limit,
          offset: graphQuery.offset,
          total: result.totalCount,
          hasMore: result.hasMore,
        },
      });
    } catch (error) {
      logger.error('Failed to query movies', { error });
      res.status(500).json({ error: 'Failed to query movies' });
    }
  }
);

/**
 * GET /api/v1/knowledge-graph/movies/:id
 * Get movie by ID with relationships
 */
router.get(
  '/movies/:id',
  [param('id').isString().notEmpty()],
  async (req: Request, res: Response): Promise<void> => {
    if (handleValidationErrors(req, res)) return;

    try {
      const store = getStore();
      const movie = await store.getMovie(req.params.id);

      if (!movie) {
        res.status(404).json({ error: 'Movie not found' });
        return;
      }

      // Get edges for relationship data
      const edges = await store.getMovieEdges(req.params.id);

      // Get related movies
      const relatedMovies = await store.getRelatedMovies(req.params.id, 6);

      res.json({
        movie,
        edges,
        relatedMovies,
      });
    } catch (error) {
      logger.error('Failed to get movie', { error, movieId: req.params.id });
      res.status(500).json({ error: 'Failed to get movie' });
    }
  }
);

/**
 * GET /api/v1/knowledge-graph/movies/:id/related
 * Get related movies
 */
router.get(
  '/movies/:id/related',
  [
    param('id').isString().notEmpty(),
    query('limit').optional().isInt({ min: 1, max: 50 }).toInt(),
  ],
  async (req: Request, res: Response) => {
    if (handleValidationErrors(req, res)) return;

    try {
      const store = getStore();
      const limit = req.query.limit ? Number(req.query.limit) : 10;

      const relatedMovies = await store.getRelatedMovies(req.params.id, limit);

      res.json({ movies: relatedMovies });
    } catch (error) {
      logger.error('Failed to get related movies', { error, movieId: req.params.id });
      res.status(500).json({ error: 'Failed to get related movies' });
    }
  }
);

// ============================================
// Genre Endpoints
// ============================================

router.get('/genres', async (_req: Request, res: Response): Promise<void> => {
  try {
    const store = getStore();
    const genres = await store.getGenres();
    res.json({ genres });
  } catch (error) {
    logger.error('Failed to get genres', { error });
    res.status(500).json({ error: 'Failed to get genres' });
  }
});

router.get(
  '/genres/:id',
  [
    param('id').isString().notEmpty(),
    query('limit').optional().isInt({ min: 1, max: 100 }).toInt(),
  ],
  async (req: Request, res: Response): Promise<void> => {
    if (handleValidationErrors(req, res)) return;

    try {
      const store = getStore();
      const genre = await store.getGenre(req.params.id);

      if (!genre) {
        res.status(404).json({ error: 'Genre not found' });
        return;
      }

      const limit = req.query.limit ? Number(req.query.limit) : 20;
      const movies = await store.getMoviesByGenre(req.params.id, limit);

      res.json({ genre, movies });
    } catch (error) {
      logger.error('Failed to get genre', { error, genreId: req.params.id });
      res.status(500).json({ error: 'Failed to get genre' });
    }
  }
);

// ============================================
// Reference Data Endpoints
// ============================================

router.get('/countries', async (_req: Request, res: Response): Promise<void> => {
  try {
    const store = getStore();
    const countries = await store.getCountries();
    res.json({ countries });
  } catch (error) {
    logger.error('Failed to get countries', { error });
    res.status(500).json({ error: 'Failed to get countries' });
  }
});

router.get('/languages', async (_req: Request, res: Response): Promise<void> => {
  try {
    const store = getStore();
    const languages = await store.getLanguages();
    res.json({ languages });
  } catch (error) {
    logger.error('Failed to get languages', { error });
    res.status(500).json({ error: 'Failed to get languages' });
  }
});

// ============================================
// Statistics Endpoint
// ============================================

router.get('/stats', async (_req: Request, res: Response): Promise<void> => {
  try {
    const store = getStore();
    const stats = await store.getStats();
    res.json({ stats });
  } catch (error) {
    logger.error('Failed to get stats', {
      error: error instanceof Error ? error.message : 'Unknown error',
      stack: error instanceof Error ? error.stack : undefined,
    });
    // Return empty stats instead of 500 error - useful during initial setup
    res.json({
      stats: {
        totalMovies: 0,
        totalGenres: 0,
        totalCompanies: 0,
        totalCountries: 0,
        totalLanguages: 0,
        totalKeywords: 0,
        totalEdges: 0,
        readyForNetflix: 0,
        readyForAmazon: 0,
        readyForFAST: 0,
        pendingProcessing: 0,
        failedValidation: 0,
        lastUpdated: new Date().toISOString(),
      },
      warning: 'Stats unavailable - database may be initializing',
    });
  }
});

// ============================================
// Semantic Search Endpoint
// ============================================

/**
 * POST /api/v1/knowledge-graph/search/semantic
 * Semantic search using Vertex AI embeddings
 */
router.post('/search/semantic', async (req: Request, res: Response): Promise<void> => {
  try {
    const { query: searchQuery, limit = 20 } = req.body;

    if (!searchQuery || typeof searchQuery !== 'string') {
      res.status(400).json({ error: 'Query is required' });
      return;
    }

    const startTime = Date.now();

    // Generate embedding for search query
    const embeddings = getEmbeddingsInstance();
    const queryEmbedding = await embeddings.generateEmbedding(searchQuery);

    // Get movies with embeddings from store
    const store = getStore();
    const { nodes: allMovies } = await store.queryMovies({ limit: 1000 });

    // Filter movies with embeddings and calculate similarity
    const moviesWithEmbeddings = (allMovies as MovieNode[]).filter(
      (m) => m.embedding && m.embedding.length > 0
    );

    const scoredMovies = moviesWithEmbeddings
      .map((movie) => ({
        movie,
        similarity: cosineSimilarity(queryEmbedding, movie.embedding!),
      }))
      .filter(item => item.similarity > 0.3) // Minimum threshold
      .sort((a, b) => b.similarity - a.similarity)
      .slice(0, limit);

    const searchDuration = Date.now() - startTime;

    res.json({
      query: searchQuery,
      results: scoredMovies.map(item => ({
        ...item.movie,
        similarity: item.similarity,
      })),
      stats: {
        totalWithEmbeddings: moviesWithEmbeddings.length,
        resultsFound: scoredMovies.length,
        searchDurationMs: searchDuration,
      },
    });
  } catch (error) {
    logger.error('Semantic search failed', { error });
    res.status(500).json({ error: 'Semantic search failed' });
  }
});

// ============================================
// Feed Generation Endpoints
// ============================================

/**
 * POST /api/v1/knowledge-graph/feeds/netflix/:movieId
 * Generate Netflix IMF package for a movie
 */
router.post('/feeds/netflix/:movieId', async (req: Request, res: Response): Promise<void> => {
  try {
    const store = getStore();
    const movie = await store.getMovie(req.params.movieId);

    if (!movie) {
      res.status(404).json({ error: 'Movie not found' });
      return;
    }

    const mediaMetadata = movieToMediaMetadata(movie);

    // Validate and generate
    const validation = netflixConnector.validate(mediaMetadata);
    const imfPackage = netflixConnector.generatePackage(mediaMetadata);

    res.json({
      movie: {
        id: movie.id,
        title: movie.title,
      },
      validation,
      package: imfPackage,
    });
  } catch (error) {
    logger.error('Netflix feed generation failed', { error, movieId: req.params.movieId });
    res.status(500).json({ error: 'Feed generation failed' });
  }
});

/**
 * POST /api/v1/knowledge-graph/feeds/amazon/:movieId
 * Generate Amazon MEC feed for a movie
 */
router.post('/feeds/amazon/:movieId', async (req: Request, res: Response): Promise<void> => {
  try {
    const store = getStore();
    const movie = await store.getMovie(req.params.movieId);

    if (!movie) {
      res.status(404).json({ error: 'Movie not found' });
      return;
    }

    const mediaMetadata = movieToMediaMetadata(movie);

    // Validate
    const validation = await amazonConnector.validate(mediaMetadata);

    // Generate feed (may throw if validation fails)
    let mecFeed = null;
    let mecXml = null;
    try {
      mecFeed = await amazonConnector.generateFeed(mediaMetadata);
      mecXml = amazonConnector.exportFeed(mecFeed);
    } catch (e) {
      // Feed generation failed due to validation
    }

    res.json({
      movie: {
        id: movie.id,
        title: movie.title,
      },
      validation,
      feed: mecFeed,
      xml: mecXml,
    });
  } catch (error) {
    logger.error('Amazon feed generation failed', { error, movieId: req.params.movieId });
    res.status(500).json({ error: 'Feed generation failed' });
  }
});

/**
 * POST /api/v1/knowledge-graph/feeds/fast/:movieId
 * Generate FAST MRSS feed for a movie
 */
router.post('/feeds/fast/:movieId', async (req: Request, res: Response): Promise<void> => {
  try {
    const { platform = 'pluto' } = req.body;
    const store = getStore();
    const movie = await store.getMovie(req.params.movieId);

    if (!movie) {
      res.status(404).json({ error: 'Movie not found' });
      return;
    }

    const mediaMetadata = movieToMediaMetadata(movie);

    // Get the right connector for the platform
    const platformMap: Record<string, Platform> = {
      pluto: Platform.FAST_PLUTO,
      tubi: Platform.FAST_TUBI,
      roku: Platform.FAST_ROKU,
      samsung: Platform.FAST_SAMSUNG,
    };

    const targetPlatform = platformMap[platform] || Platform.FAST_PLUTO;
    const connector = new FASTMRSSConnector(targetPlatform);

    // Validate and generate
    const validation = await connector.validate(mediaMetadata);
    const mrssPackage = await connector.generate(mediaMetadata);
    const mrssXml = await connector.serialize(mrssPackage, 'xml');

    res.json({
      movie: {
        id: movie.id,
        title: movie.title,
      },
      platform: targetPlatform,
      validation,
      package: mrssPackage,
      xml: mrssXml,
    });
  } catch (error) {
    logger.error('FAST feed generation failed', { error, movieId: req.params.movieId });
    res.status(500).json({ error: 'Feed generation failed' });
  }
});

/**
 * POST /api/v1/knowledge-graph/feeds/batch
 * Generate feeds for multiple movies (for preview)
 */
router.post('/feeds/batch', async (req: Request, res: Response): Promise<void> => {
  try {
    const { platform, genreId, limit = 10 } = req.body;

    if (!platform || !['netflix', 'amazon', 'fast'].includes(platform)) {
      res.status(400).json({ error: 'Valid platform required (netflix, amazon, fast)' });
      return;
    }

    const store = getStore();
    let movies: MovieNode[];

    if (genreId) {
      movies = await store.getMoviesByGenre(genreId, limit);
    } else {
      const result = await store.queryMovies({ limit, sortBy: 'popularity', sortOrder: 'desc' });
      movies = result.nodes as MovieNode[];
    }

    const feeds = await Promise.all(
      movies.map(async (movie) => {
        const metadata = movieToMediaMetadata(movie);
        let validation;

        try {
          if (platform === 'netflix') {
            validation = netflixConnector.validate(metadata);
          } else if (platform === 'amazon') {
            validation = await amazonConnector.validate(metadata);
          } else {
            validation = await fastConnector.validate(metadata);
          }
        } catch {
          validation = { valid: false, errors: [{ message: 'Validation failed' }] };
        }

        return {
          movieId: movie.id,
          title: movie.title,
          posterPath: movie.posterPath,
          validation: {
            valid: validation.valid,
            errorCount: validation.errors?.length || 0,
            warningCount: validation.warnings?.length || 0,
          },
        };
      })
    );

    res.json({
      platform,
      totalMovies: movies.length,
      feeds,
    });
  } catch (error) {
    logger.error('Batch feed generation failed', { error });
    res.status(500).json({ error: 'Batch feed generation failed' });
  }
});

// ============================================
// Diagnostic Endpoints
// ============================================

/**
 * GET /api/v1/knowledge-graph/diagnostics/csv-sample
 * Get a raw CSV row to debug column names and data format
 */
router.get('/diagnostics/csv-sample', async (_req: Request, res: Response): Promise<void> => {
  try {
    const gcsReader = getGCSReader();
    let sampleRow: Record<string, unknown> | null = null;

    for await (const row of gcsReader.streamRows(undefined, 1)) {
      sampleRow = row as Record<string, unknown>;
      break;
    }

    if (!sampleRow) {
      res.status(404).json({ error: 'No rows found in CSV' });
      return;
    }

    // Return column names and sample values
    res.json({
      columns: Object.keys(sampleRow),
      columnCount: Object.keys(sampleRow).length,
      sampleRow,
      genresField: sampleRow.genres,
      genresType: typeof sampleRow.genres,
    });
  } catch (error) {
    logger.error('CSV sample failed', { error: error instanceof Error ? error.message : 'Unknown' });
    res.status(500).json({ error: 'Failed to read CSV sample' });
  }
});

/**
 * GET /api/v1/knowledge-graph/diagnostics
 * Check connectivity to Firestore, GCS, and Vertex AI
 */
router.get('/diagnostics', async (_req: Request, res: Response): Promise<void> => {
  const results: {
    firestore: { connected: boolean; error?: string };
    gcs: { connected: boolean; error?: string; fileSize?: number };
    vertexAI: { connected: boolean; error?: string };
  } = {
    firestore: { connected: false },
    gcs: { connected: false },
    vertexAI: { connected: false },
  };

  // Test Firestore
  try {
    const store = getStore();
    await store.getStats();
    results.firestore.connected = true;
  } catch (error) {
    results.firestore.error = error instanceof Error ? error.message : 'Unknown error';
  }

  // Test GCS
  try {
    const { Storage } = await import('@google-cloud/storage');
    const storage = new Storage({
      projectId: process.env.GCP_PROJECT_ID || 'agentics-foundation25lon-1899',
    });
    const bucket = storage.bucket('nexus-ummid-datasets');
    const file = bucket.file('TMDB_movie_dataset_v11.csv');
    const [metadata] = await file.getMetadata();
    results.gcs.connected = true;
    results.gcs.fileSize = parseInt(metadata.size as string, 10);
  } catch (error) {
    results.gcs.error = error instanceof Error ? error.message : 'Unknown error';
  }

  // Test Vertex AI Embeddings
  try {
    const embeddings = getEmbeddingsInstance();
    const testEmbedding = await embeddings.generateEmbedding('test connectivity');
    results.vertexAI.connected = testEmbedding.length > 0;
  } catch (error) {
    results.vertexAI.error = error instanceof Error ? error.message : 'Unknown error';
  }

  res.json({
    timestamp: new Date().toISOString(),
    diagnostics: results,
    environment: {
      gcpProject: process.env.GCP_PROJECT_ID || 'agentics-foundation25lon-1899',
      nodeEnv: process.env.NODE_ENV,
    },
  });
});

// ============================================
// Ingestion Endpoints
// ============================================

/**
 * POST /api/v1/knowledge-graph/ingest/start
 * Start dataset ingestion with embeddings
 */
router.post('/ingest/start', async (req: Request, res: Response): Promise<void> => {
  try {
    const { limit = 100, generateEmbeddings = false } = req.body;

    logger.info('Starting ingestion', { limit, generateEmbeddings });

    const pipeline = createIngestionPipeline({
      limit: Math.min(limit, 100000), // Cap at 100k
      generateEmbeddings,
      batchSize: 100,
      embeddingBatchSize: 5,
      sortByPopularity: true,
      minVoteCount: limit > 1000 ? 10 : 0, // Only filter for larger ingestions
    });

    const result = await pipeline.run();

    res.json({
      success: result.success,
      stats: result.stats,
      error: result.error,
    });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Unknown error';
    const errorStack = error instanceof Error ? error.stack : undefined;
    logger.error('Ingestion failed', { error: errorMessage, stack: errorStack });
    res.status(500).json({
      error: 'Ingestion failed',
      details: errorMessage,
      stack: process.env.NODE_ENV !== 'production' ? errorStack : undefined,
    });
  }
});

/**
 * POST /api/v1/knowledge-graph/ingest/quick-test
 * Quick test ingestion (100 movies, no embeddings)
 */
router.post('/ingest/quick-test', async (_req: Request, res: Response): Promise<void> => {
  try {
    const pipeline = createIngestionPipeline({
      limit: 100,
      generateEmbeddings: false,
      minVoteCount: 0,
    });

    const result = await pipeline.runQuickTest();

    res.json({
      success: result.success,
      stats: result.stats,
      error: result.error,
    });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Unknown error';
    const errorStack = error instanceof Error ? error.stack : undefined;
    logger.error('Quick test failed', { error: errorMessage, stack: errorStack });
    res.status(500).json({
      error: 'Quick test failed',
      details: errorMessage,
      stack: process.env.NODE_ENV !== 'production' ? errorStack : undefined,
    });
  }
});

/**
 * GET /api/v1/knowledge-graph/ingest/status
 * Get ingestion status
 */
router.get('/ingest/status', async (_req: Request, res: Response): Promise<void> => {
  try {
    const processor = getProcessor();
    const store = getStore();
    const stats = await store.getStats();

    // Check if embeddings exist
    const { nodes: sampleMovies } = await store.queryMovies({ limit: 10 });
    const moviesWithEmbeddings = (sampleMovies as MovieNode[]).filter(
      (m) => m.embedding && m.embedding.length > 0
    ).length;

    res.json({
      status: stats.totalMovies > 0 ? 'completed' : 'not_started',
      stats,
      cacheStats: processor.getCacheStats(),
      embeddingStatus: {
        sampleSize: sampleMovies.length,
        withEmbeddings: moviesWithEmbeddings,
        percentage: sampleMovies.length > 0
          ? Math.round((moviesWithEmbeddings / sampleMovies.length) * 100)
          : 0,
      },
    });
  } catch (error) {
    logger.error('Failed to get ingestion status', { error });
    res.status(500).json({ error: 'Failed to get status' });
  }
});

export default router;
