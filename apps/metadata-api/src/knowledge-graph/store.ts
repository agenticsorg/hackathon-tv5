/**
 * Knowledge Graph Firestore Store
 *
 * Stores and retrieves knowledge graph nodes and edges from Firestore.
 * Optimized for batch operations and efficient querying.
 */

import { Firestore } from '@google-cloud/firestore';
import { getFirestore, batchWrite } from '../db/firestore';
import { logger } from '../utils/logger';
import {
  MovieNode,
  GenreNode,
  CountryNode,
  LanguageNode,
  Hyperedge,
  GraphQuery,
  GraphQueryResult,
  KnowledgeGraphStats,
} from './schema';
import { ProcessedMovie } from './processor';

/**
 * Collection names in Firestore
 */
const COLLECTIONS = {
  MOVIES: 'kg_movies',
  GENRES: 'kg_genres',
  COMPANIES: 'kg_companies',
  COUNTRIES: 'kg_countries',
  LANGUAGES: 'kg_languages',
  KEYWORDS: 'kg_keywords',
  EDGES: 'kg_edges',
  STATS: 'kg_stats',
};

/**
 * Store configuration
 */
export interface StoreConfig {
  batchSize: number;
  enableCompression: boolean;
}

const DEFAULT_CONFIG: StoreConfig = {
  batchSize: 500, // Firestore batch limit
  enableCompression: false,
};

/**
 * Knowledge Graph Store Class
 */
export class KnowledgeGraphStore {
  private db: Firestore;
  private config: StoreConfig;

  constructor(config?: Partial<StoreConfig>) {
    this.db = getFirestore();
    this.config = { ...DEFAULT_CONFIG, ...config };

    logger.info('Knowledge Graph Store initialized', {
      batchSize: this.config.batchSize,
    });
  }

  // ============================================
  // Write Operations
  // ============================================

  /**
   * Store a processed movie with all its relationships
   */
  async storeProcessedMovie(processed: ProcessedMovie): Promise<void> {
    const batch = this.db.batch();

    // Store movie node
    const movieRef = this.db.collection(COLLECTIONS.MOVIES).doc(processed.movie.id);
    batch.set(movieRef, processed.movie, { merge: true });

    // Store genre nodes
    for (const genre of processed.genres) {
      const ref = this.db.collection(COLLECTIONS.GENRES).doc(genre.id);
      batch.set(ref, genre, { merge: true });
    }

    // Store company nodes
    for (const company of processed.companies) {
      const ref = this.db.collection(COLLECTIONS.COMPANIES).doc(company.id);
      batch.set(ref, company, { merge: true });
    }

    // Store country nodes
    for (const country of processed.countries) {
      const ref = this.db.collection(COLLECTIONS.COUNTRIES).doc(country.id);
      batch.set(ref, country, { merge: true });
    }

    // Store language nodes
    for (const language of processed.languages) {
      const ref = this.db.collection(COLLECTIONS.LANGUAGES).doc(language.id);
      batch.set(ref, language, { merge: true });
    }

    // Store keyword nodes
    for (const keyword of processed.keywords) {
      const ref = this.db.collection(COLLECTIONS.KEYWORDS).doc(keyword.id);
      batch.set(ref, keyword, { merge: true });
    }

    // Store edges
    for (const edge of processed.edges) {
      const ref = this.db.collection(COLLECTIONS.EDGES).doc(edge.id);
      batch.set(ref, edge, { merge: true });
    }

    await batch.commit();
  }

  /**
   * Batch store multiple processed movies
   */
  async storeProcessedMoviesBatch(movies: ProcessedMovie[]): Promise<{
    stored: number;
    failed: number;
    durationMs: number;
  }> {
    const startTime = Date.now();
    let stored = 0;
    let failed = 0;

    // Collect all operations
    const operations: Array<{
      type: 'set';
      ref: FirebaseFirestore.DocumentReference;
      data: any;
    }> = [];

    for (const processed of movies) {
      try {
        // Movie - use JSON parse/stringify to clean undefined values and ensure plain object
        const cleanMovieData = JSON.parse(JSON.stringify(processed.movie));
        operations.push({
          type: 'set',
          ref: this.db.collection(COLLECTIONS.MOVIES).doc(String(processed.movie.id)),
          data: cleanMovieData,
        });

        // Genres (deduplicated by cache in processor)
        for (const genre of processed.genres) {
          operations.push({
            type: 'set',
            ref: this.db.collection(COLLECTIONS.GENRES).doc(genre.id),
            data: genre,
          });
        }

        // Companies
        for (const company of processed.companies) {
          operations.push({
            type: 'set',
            ref: this.db.collection(COLLECTIONS.COMPANIES).doc(company.id),
            data: company,
          });
        }

        // Countries
        for (const country of processed.countries) {
          operations.push({
            type: 'set',
            ref: this.db.collection(COLLECTIONS.COUNTRIES).doc(country.id),
            data: country,
          });
        }

        // Languages
        for (const language of processed.languages) {
          operations.push({
            type: 'set',
            ref: this.db.collection(COLLECTIONS.LANGUAGES).doc(language.id),
            data: language,
          });
        }

        // Keywords
        for (const keyword of processed.keywords) {
          operations.push({
            type: 'set',
            ref: this.db.collection(COLLECTIONS.KEYWORDS).doc(keyword.id),
            data: keyword,
          });
        }

        // Edges
        for (const edge of processed.edges) {
          operations.push({
            type: 'set',
            ref: this.db.collection(COLLECTIONS.EDGES).doc(edge.id),
            data: edge,
          });
        }

        stored++;
      } catch (error) {
        logger.error('Failed to prepare movie for storage', {
          movieId: processed.movie.id,
          error: error instanceof Error ? error.message : 'Unknown error',
        });
        failed++;
      }
    }

    // Execute batch writes
    try {
      await batchWrite(operations);
    } catch (error) {
      logger.error('Batch write failed', {
        operationCount: operations.length,
        error: error instanceof Error ? error.message : 'Unknown error',
      });
      throw error;
    }

    const durationMs = Date.now() - startTime;

    logger.info('Batch store completed', {
      stored,
      failed,
      operations: operations.length,
      durationMs,
    });

    return { stored, failed, durationMs };
  }

  /**
   * Store embedding for a movie
   */
  async storeMovieEmbedding(
    movieId: string,
    embedding: number[],
    embeddingModel: string
  ): Promise<void> {
    const movieRef = this.db.collection(COLLECTIONS.MOVIES).doc(movieId);
    await movieRef.update({
      embedding,
      embeddingModel,
      updatedAt: new Date().toISOString(),
    });
  }

  // ============================================
  // Read Operations
  // ============================================

  /**
   * Get movie by ID
   */
  async getMovie(id: string): Promise<MovieNode | null> {
    const doc = await this.db.collection(COLLECTIONS.MOVIES).doc(id).get();
    return doc.exists ? (doc.data() as MovieNode) : null;
  }

  /**
   * Get movies by IDs
   */
  async getMovies(ids: string[]): Promise<MovieNode[]> {
    if (ids.length === 0) return [];

    const refs = ids.map(id => this.db.collection(COLLECTIONS.MOVIES).doc(id));
    const docs = await this.db.getAll(...refs);

    return docs
      .filter(doc => doc.exists)
      .map(doc => doc.data() as MovieNode);
  }

  /**
   * Query movies with filters
   */
  async queryMovies(query: GraphQuery): Promise<GraphQueryResult> {
    let firestoreQuery: FirebaseFirestore.Query = this.db.collection(COLLECTIONS.MOVIES);

    // Apply filters
    if (query.yearRange) {
      if (query.yearRange.min) {
        firestoreQuery = firestoreQuery.where('year', '>=', query.yearRange.min);
      }
      if (query.yearRange.max) {
        firestoreQuery = firestoreQuery.where('year', '<=', query.yearRange.max);
      }
    }

    if (query.ratingRange) {
      if (query.ratingRange.min) {
        firestoreQuery = firestoreQuery.where('voteAverage', '>=', query.ratingRange.min);
      }
      if (query.ratingRange.max) {
        firestoreQuery = firestoreQuery.where('voteAverage', '<=', query.ratingRange.max);
      }
    }

    // Apply sorting
    if (query.sortBy) {
      const order = query.sortOrder === 'asc' ? 'asc' : 'desc';
      firestoreQuery = firestoreQuery.orderBy(query.sortBy, order);
    }

    // Apply pagination
    const limit = query.limit || 20;
    const offset = query.offset || 0;

    firestoreQuery = firestoreQuery.limit(limit + 1); // +1 to check hasMore

    if (offset > 0) {
      // Note: Firestore doesn't support offset directly, would need cursor pagination
      // For now, we fetch extra and skip
      firestoreQuery = firestoreQuery.limit(limit + offset + 1);
    }

    const snapshot = await firestoreQuery.get();
    let docs = snapshot.docs;

    // Apply offset
    if (offset > 0) {
      docs = docs.slice(offset);
    }

    // Check hasMore
    const hasMore = docs.length > limit;
    if (hasMore) {
      docs = docs.slice(0, limit);
    }

    const nodes = docs.map(doc => doc.data() as MovieNode);

    // Get related edges if requested
    const edges: Hyperedge[] = [];

    return {
      nodes,
      edges,
      totalCount: snapshot.size,
      hasMore,
    };
  }

  /**
   * Get movies by genre
   */
  async getMoviesByGenre(
    genreId: string,
    limit: number = 20,
    offset: number = 0
  ): Promise<MovieNode[]> {
    // Query edges to find movie IDs
    const edgesSnapshot = await this.db
      .collection(COLLECTIONS.EDGES)
      .where('type', '==', 'HAS_GENRE')
      .where('genreId', '==', genreId)
      .limit(limit + offset)
      .get();

    const movieIds = edgesSnapshot.docs
      .slice(offset)
      .map(doc => doc.data().movieId);

    return this.getMovies(movieIds);
  }

  /**
   * Get all genres
   */
  async getGenres(): Promise<GenreNode[]> {
    const snapshot = await this.db
      .collection(COLLECTIONS.GENRES)
      .orderBy('movieCount', 'desc')
      .get();

    return snapshot.docs.map(doc => doc.data() as GenreNode);
  }

  /**
   * Get genre by ID
   */
  async getGenre(id: string): Promise<GenreNode | null> {
    const doc = await this.db.collection(COLLECTIONS.GENRES).doc(id).get();
    return doc.exists ? (doc.data() as GenreNode) : null;
  }

  /**
   * Get all countries
   */
  async getCountries(): Promise<CountryNode[]> {
    const snapshot = await this.db
      .collection(COLLECTIONS.COUNTRIES)
      .orderBy('movieCount', 'desc')
      .get();

    return snapshot.docs.map(doc => doc.data() as CountryNode);
  }

  /**
   * Get all languages
   */
  async getLanguages(): Promise<LanguageNode[]> {
    const snapshot = await this.db
      .collection(COLLECTIONS.LANGUAGES)
      .orderBy('movieCount', 'desc')
      .get();

    return snapshot.docs.map(doc => doc.data() as LanguageNode);
  }

  /**
   * Get edges for a movie
   */
  async getMovieEdges(movieId: string): Promise<Hyperedge[]> {
    const snapshot = await this.db
      .collection(COLLECTIONS.EDGES)
      .where('movieId', '==', movieId)
      .get();

    return snapshot.docs.map(doc => doc.data() as Hyperedge);
  }

  /**
   * Get related movies (same genre or keywords)
   */
  async getRelatedMovies(movieId: string, limit: number = 10): Promise<MovieNode[]> {
    // Get movie's genres
    const genreEdges = await this.db
      .collection(COLLECTIONS.EDGES)
      .where('type', '==', 'HAS_GENRE')
      .where('movieId', '==', movieId)
      .get();

    if (genreEdges.empty) {
      return [];
    }

    const genreIds = genreEdges.docs.map(doc => doc.data().genreId);

    // Find other movies with same genres
    const relatedEdges = await this.db
      .collection(COLLECTIONS.EDGES)
      .where('type', '==', 'HAS_GENRE')
      .where('genreId', 'in', genreIds.slice(0, 10)) // Firestore 'in' limit
      .limit(limit * 2) // Get extra to filter out source movie
      .get();

    const relatedMovieIds = [...new Set(
      relatedEdges.docs
        .map(doc => doc.data().movieId)
        .filter(id => id !== movieId)
    )].slice(0, limit);

    return this.getMovies(relatedMovieIds);
  }

  // ============================================
  // Statistics
  // ============================================

  /**
   * Get knowledge graph statistics
   */
  async getStats(): Promise<KnowledgeGraphStats> {
    // Get counts from each collection
    const [moviesCount, genresCount, companiesCount, countriesCount, languagesCount, keywordsCount, edgesCount] = await Promise.all([
      this.getCollectionCount(COLLECTIONS.MOVIES),
      this.getCollectionCount(COLLECTIONS.GENRES),
      this.getCollectionCount(COLLECTIONS.COMPANIES),
      this.getCollectionCount(COLLECTIONS.COUNTRIES),
      this.getCollectionCount(COLLECTIONS.LANGUAGES),
      this.getCollectionCount(COLLECTIONS.KEYWORDS),
      this.getCollectionCount(COLLECTIONS.EDGES),
    ]);

    // Get platform readiness counts
    const [netflixReady, amazonReady, fastReady, pendingCount, failedCount] = await Promise.all([
      this.getPlatformReadyCount('netflix'),
      this.getPlatformReadyCount('amazon'),
      this.getPlatformReadyCount('fast'),
      this.getDistributionStatusCount('pending'),
      this.getDistributionStatusCount('failed'),
    ]);

    return {
      totalMovies: moviesCount,
      totalGenres: genresCount,
      totalCompanies: companiesCount,
      totalCountries: countriesCount,
      totalLanguages: languagesCount,
      totalKeywords: keywordsCount,
      totalEdges: edgesCount,
      readyForNetflix: netflixReady,
      readyForAmazon: amazonReady,
      readyForFAST: fastReady,
      pendingProcessing: pendingCount,
      failedValidation: failedCount,
      lastUpdated: new Date().toISOString(),
    };
  }

  /**
   * Get count of movies ready for a specific platform
   */
  private async getPlatformReadyCount(platform: 'netflix' | 'amazon' | 'fast'): Promise<number> {
    try {
      const snapshot = await this.db
        .collection(COLLECTIONS.MOVIES)
        .where(`platformReadiness.${platform}`, '==', true)
        .count()
        .get();
      return snapshot.data().count;
    } catch (error) {
      // Field might not exist yet
      logger.debug(`Platform readiness count for ${platform} failed`, {
        error: error instanceof Error ? error.message : 'Unknown error',
      });
      return 0;
    }
  }

  /**
   * Get count of movies with a specific distribution status
   */
  private async getDistributionStatusCount(status: string): Promise<number> {
    try {
      const snapshot = await this.db
        .collection(COLLECTIONS.MOVIES)
        .where('distributionStatus', '==', status)
        .count()
        .get();
      return snapshot.data().count;
    } catch (error) {
      logger.debug(`Distribution status count for ${status} failed`, {
        error: error instanceof Error ? error.message : 'Unknown error',
      });
      return 0;
    }
  }

  /**
   * Get approximate collection count
   */
  private async getCollectionCount(collectionName: string): Promise<number> {
    try {
      // Use aggregation query for efficiency
      const snapshot = await this.db.collection(collectionName).count().get();
      return snapshot.data().count;
    } catch (error) {
      // Collection might not exist yet - return 0
      logger.debug(`Collection ${collectionName} count failed, returning 0`, {
        error: error instanceof Error ? error.message : 'Unknown error',
      });
      return 0;
    }
  }

  /**
   * Update stats document
   */
  async updateStats(stats: Partial<KnowledgeGraphStats>): Promise<void> {
    const statsRef = this.db.collection(COLLECTIONS.STATS).doc('global');
    await statsRef.set({
      ...stats,
      lastUpdated: new Date().toISOString(),
    }, { merge: true });
  }

  // ============================================
  // Cleanup
  // ============================================

  /**
   * Delete all knowledge graph data
   */
  async clearAll(): Promise<void> {
    const collections = Object.values(COLLECTIONS);

    for (const collectionName of collections) {
      let deleted = 0;
      let snapshot = await this.db.collection(collectionName).limit(500).get();

      while (!snapshot.empty) {
        const batch = this.db.batch();
        snapshot.docs.forEach(doc => batch.delete(doc.ref));
        await batch.commit();
        deleted += snapshot.docs.length;

        // Re-query for next batch
        snapshot = await this.db.collection(collectionName).limit(500).get();
      }

      if (deleted > 0) {
        logger.info(`Cleared ${deleted} documents from ${collectionName}`);
      }
    }

    logger.info('Knowledge graph data cleared');
  }
}

/**
 * Singleton instance
 */
let storeInstance: KnowledgeGraphStore | null = null;

export function getStore(config?: Partial<StoreConfig>): KnowledgeGraphStore {
  if (!storeInstance) {
    storeInstance = new KnowledgeGraphStore(config);
  }
  return storeInstance;
}
