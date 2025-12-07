/**
 * Database Repository
 * Handles all PostgreSQL operations for the recommendation system
 */

import { Pool, PoolClient, QueryResult } from 'pg';
import pino from 'pino';
import {
  User,
  UserPreference,
  WatchHistoryItem,
  Rating,
  UserInteraction,
  ContentEmbedding,
  ContentMetadata,
  RecommendationPattern,
  LearningFeedback
} from '../types/index.js';

const logger = pino({ name: 'repository' });

export class Repository {
  private pool: Pool;

  constructor(connectionString: string, poolSize: number = 10) {
    this.pool = new Pool({
      connectionString,
      max: poolSize,
      idleTimeoutMillis: 30000,
      connectionTimeoutMillis: 10000
    });

    this.pool.on('error', (err) => {
      logger.error({ err }, 'Unexpected database pool error');
    });
  }

  /**
   * Initialize database (run migrations)
   */
  async initialize(): Promise<void> {
    const client = await this.pool.connect();
    try {
      // Check if tables exist
      const result = await client.query(`
        SELECT EXISTS (
          SELECT FROM information_schema.tables
          WHERE table_schema = 'public'
          AND table_name = 'users'
        );
      `);

      if (!result.rows[0].exists) {
        logger.warn('Database tables not found. Please run migrations.');
      } else {
        logger.info('Database connection verified');
      }
    } finally {
      client.release();
    }
  }

  /**
   * Close database connection
   */
  async close(): Promise<void> {
    await this.pool.end();
    logger.info('Database connection closed');
  }

  // ============================================================================
  // USER OPERATIONS
  // ============================================================================

  async createUser(email?: string, username?: string): Promise<User> {
    const result = await this.pool.query<User>(
      `INSERT INTO users (email, username)
       VALUES ($1, $2)
       RETURNING id, email, username, created_at, updated_at`,
      [email, username]
    );
    return this.mapUser(result.rows[0]);
  }

  async getUser(userId: string): Promise<User | null> {
    const result = await this.pool.query<User>(
      `SELECT id, email, username, created_at, updated_at, last_active_at
       FROM users WHERE id = $1`,
      [userId]
    );
    return result.rows[0] ? this.mapUser(result.rows[0]) : null;
  }

  async updateUserActivity(userId: string): Promise<void> {
    await this.pool.query(
      `UPDATE users SET last_active_at = NOW() WHERE id = $1`,
      [userId]
    );
  }

  // ============================================================================
  // CONTENT OPERATIONS
  // ============================================================================

  async upsertContent(embedding: ContentEmbedding): Promise<void> {
    const vectorStr = `[${Array.from(embedding.vector).join(',')}]`;

    await this.pool.query(
      `INSERT INTO content (
        id, content_type, title, year, overview, genres, rating,
        status, network_id, network_name, original_language, original_country,
        embedding, updated_at
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::vector, NOW())
      ON CONFLICT (id) DO UPDATE SET
        title = EXCLUDED.title,
        year = EXCLUDED.year,
        overview = EXCLUDED.overview,
        genres = EXCLUDED.genres,
        rating = EXCLUDED.rating,
        status = EXCLUDED.status,
        network_id = EXCLUDED.network_id,
        network_name = EXCLUDED.network_name,
        embedding = EXCLUDED.embedding,
        updated_at = NOW()`,
      [
        embedding.contentId,
        embedding.contentType,
        embedding.metadata.title,
        embedding.metadata.year,
        embedding.metadata.overview,
        embedding.metadata.genres,
        embedding.metadata.rating,
        embedding.metadata.status,
        embedding.metadata.networkId,
        embedding.metadata.networkName,
        embedding.metadata.language,
        embedding.metadata.country,
        vectorStr
      ]
    );
  }

  async batchUpsertContent(embeddings: ContentEmbedding[]): Promise<void> {
    const client = await this.pool.connect();
    try {
      await client.query('BEGIN');

      for (const embedding of embeddings) {
        const vectorStr = `[${Array.from(embedding.vector).join(',')}]`;
        await client.query(
          `INSERT INTO content (
            id, content_type, title, year, overview, genres, rating,
            status, network_id, network_name, original_language, original_country,
            embedding, updated_at
          ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::vector, NOW())
          ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            embedding = EXCLUDED.embedding,
            updated_at = NOW()`,
          [
            embedding.contentId,
            embedding.contentType,
            embedding.metadata.title,
            embedding.metadata.year,
            embedding.metadata.overview,
            embedding.metadata.genres,
            embedding.metadata.rating,
            embedding.metadata.status,
            embedding.metadata.networkId,
            embedding.metadata.networkName,
            embedding.metadata.language,
            embedding.metadata.country,
            vectorStr
          ]
        );
      }

      await client.query('COMMIT');
      logger.info({ count: embeddings.length }, 'Batch upserted content');
    } catch (error) {
      await client.query('ROLLBACK');
      throw error;
    } finally {
      client.release();
    }
  }

  async getContent(contentId: string): Promise<ContentEmbedding | null> {
    const result = await this.pool.query(
      `SELECT id, content_type, title, year, overview, genres, rating,
              status, network_id, network_name, original_language, original_country,
              embedding, created_at, updated_at
       FROM content WHERE id = $1`,
      [contentId]
    );

    if (!result.rows[0]) return null;
    return this.mapContentEmbedding(result.rows[0]);
  }

  async searchContent(query: string, limit: number = 20): Promise<ContentEmbedding[]> {
    const result = await this.pool.query(
      `SELECT id, content_type, title, year, overview, genres, rating,
              status, network_id, network_name, embedding,
              ts_rank(search_vector, plainto_tsquery('english', $1)) as rank
       FROM content
       WHERE search_vector @@ plainto_tsquery('english', $1)
       ORDER BY rank DESC
       LIMIT $2`,
      [query, limit]
    );

    return result.rows.map(row => this.mapContentEmbedding(row));
  }

  async findSimilarContent(
    contentId: string,
    limit: number = 10,
    threshold: number = 0.5
  ): Promise<{ embedding: ContentEmbedding; similarity: number }[]> {
    const result = await this.pool.query(
      `SELECT c.*, 1 - (c.embedding <=> (SELECT embedding FROM content WHERE id = $1)) as similarity
       FROM content c
       WHERE c.id != $1
       AND c.embedding IS NOT NULL
       AND 1 - (c.embedding <=> (SELECT embedding FROM content WHERE id = $1)) >= $3
       ORDER BY c.embedding <=> (SELECT embedding FROM content WHERE id = $1)
       LIMIT $2`,
      [contentId, limit, threshold]
    );

    return result.rows.map(row => ({
      embedding: this.mapContentEmbedding(row),
      similarity: parseFloat(row.similarity)
    }));
  }

  async getContentByVector(
    vector: Float32Array,
    limit: number = 20,
    contentType?: 'series' | 'movie'
  ): Promise<{ embedding: ContentEmbedding; similarity: number }[]> {
    const vectorStr = `[${Array.from(vector).join(',')}]`;

    let query = `
      SELECT c.*, 1 - (c.embedding <=> $1::vector) as similarity
      FROM content c
      WHERE c.embedding IS NOT NULL
    `;

    const params: any[] = [vectorStr, limit];

    if (contentType) {
      query += ` AND c.content_type = $3`;
      params.push(contentType);
    }

    query += ` ORDER BY c.embedding <=> $1::vector LIMIT $2`;

    const result = await this.pool.query(query, params);

    return result.rows.map(row => ({
      embedding: this.mapContentEmbedding(row),
      similarity: parseFloat(row.similarity)
    }));
  }

  // ============================================================================
  // USER PREFERENCES
  // ============================================================================

  async getUserPreference(userId: string): Promise<UserPreference | null> {
    const result = await this.pool.query(
      `SELECT up.*,
              COALESCE(
                (SELECT json_agg(json_build_object(
                  'contentId', wh.content_id,
                  'contentType', c.content_type,
                  'watchedAt', wh.watched_at,
                  'completionPercentage', wh.completion_percentage,
                  'duration', wh.duration_seconds
                ))
                FROM watch_history wh
                JOIN content c ON c.id = wh.content_id
                WHERE wh.user_id = up.user_id
                ORDER BY wh.watched_at DESC
                LIMIT 100),
                '[]'
              ) as watch_history,
              COALESCE(
                (SELECT json_agg(json_build_object(
                  'contentId', r.content_id,
                  'rating', r.rating,
                  'ratedAt', r.rated_at
                ))
                FROM ratings r
                WHERE r.user_id = up.user_id),
                '[]'
              ) as ratings
       FROM user_preferences up
       WHERE up.user_id = $1`,
      [userId]
    );

    if (!result.rows[0]) return null;
    return this.mapUserPreference(result.rows[0]);
  }

  async upsertUserPreference(
    userId: string,
    preferenceVector?: Float32Array,
    genreWeights?: Record<string, number>,
    networkWeights?: Record<string, number>
  ): Promise<void> {
    const vectorStr = preferenceVector
      ? `[${Array.from(preferenceVector).join(',')}]`
      : null;

    await this.pool.query(
      `INSERT INTO user_preferences (user_id, preference_vector, genre_weights, network_weights)
       VALUES ($1, $2::vector, $3, $4)
       ON CONFLICT (user_id) DO UPDATE SET
         preference_vector = COALESCE($2::vector, user_preferences.preference_vector),
         genre_weights = COALESCE($3, user_preferences.genre_weights),
         network_weights = COALESCE($4, user_preferences.network_weights),
         updated_at = NOW()`,
      [userId, vectorStr, JSON.stringify(genreWeights || {}), JSON.stringify(networkWeights || {})]
    );
  }

  async updateUserPreferenceVector(userId: string): Promise<void> {
    await this.pool.query(
      `SELECT update_user_preference_vector($1)`,
      [userId]
    );
  }

  // ============================================================================
  // WATCH HISTORY
  // ============================================================================

  async addWatchHistory(item: Omit<WatchHistoryItem, 'watchedAt'> & { userId: string }): Promise<void> {
    await this.pool.query(
      `INSERT INTO watch_history (
        user_id, content_id, episode_id, season_number, episode_number,
        duration_seconds, completion_percentage
      ) VALUES ($1, $2, $3, $4, $5, $6, $7)`,
      [
        item.userId,
        item.contentId,
        item.episodeId,
        item.seasonNumber,
        item.episodeNumber,
        item.duration,
        item.completionPercentage
      ]
    );
  }

  async getWatchHistory(
    userId: string,
    limit: number = 100,
    offset: number = 0
  ): Promise<WatchHistoryItem[]> {
    const result = await this.pool.query(
      `SELECT wh.content_id, c.content_type, wh.watched_at, wh.completion_percentage,
              wh.episode_id, wh.season_number, wh.episode_number, wh.duration_seconds
       FROM watch_history wh
       JOIN content c ON c.id = wh.content_id
       WHERE wh.user_id = $1
       ORDER BY wh.watched_at DESC
       LIMIT $2 OFFSET $3`,
      [userId, limit, offset]
    );

    return result.rows.map(row => ({
      contentId: row.content_id,
      contentType: row.content_type,
      watchedAt: new Date(row.watched_at),
      completionPercentage: parseFloat(row.completion_percentage),
      episodeId: row.episode_id,
      seasonNumber: row.season_number,
      episodeNumber: row.episode_number,
      duration: row.duration_seconds
    }));
  }

  // ============================================================================
  // RATINGS
  // ============================================================================

  async upsertRating(userId: string, contentId: string, rating: number): Promise<void> {
    await this.pool.query(
      `INSERT INTO ratings (user_id, content_id, rating)
       VALUES ($1, $2, $3)
       ON CONFLICT (user_id, content_id) DO UPDATE SET
         rating = $3,
         rated_at = NOW()`,
      [userId, contentId, rating]
    );
  }

  async getUserRatings(userId: string): Promise<Rating[]> {
    const result = await this.pool.query(
      `SELECT r.content_id, c.content_type, r.rating, r.rated_at
       FROM ratings r
       JOIN content c ON c.id = r.content_id
       WHERE r.user_id = $1
       ORDER BY r.rated_at DESC`,
      [userId]
    );

    return result.rows.map(row => ({
      contentId: row.content_id,
      contentType: row.content_type,
      rating: parseFloat(row.rating),
      ratedAt: new Date(row.rated_at)
    }));
  }

  // ============================================================================
  // LEARNING FEEDBACK
  // ============================================================================

  async recordLearningFeedback(feedback: LearningFeedback & { userId: string; contentId?: string }): Promise<void> {
    await this.pool.query(
      `INSERT INTO learning_feedback (
        user_id, content_id, pattern_id, was_successful, reward, user_action
      ) VALUES ($1, $2, $3, $4, $5, $6)`,
      [
        feedback.userId,
        feedback.contentId,
        feedback.patternId,
        feedback.wasSuccessful,
        feedback.reward,
        feedback.userAction
      ]
    );

    // Update pattern success rate
    if (feedback.patternId) {
      await this.pool.query(
        `UPDATE recommendation_patterns
         SET success_rate = (
           SELECT AVG(CASE WHEN was_successful THEN 1.0 ELSE 0.0 END)
           FROM learning_feedback
           WHERE pattern_id = $1
         ),
         total_uses = total_uses + 1,
         avg_reward = (
           SELECT AVG(reward)
           FROM learning_feedback
           WHERE pattern_id = $1
         ),
         last_used_at = NOW(),
         updated_at = NOW()
         WHERE id = $1`,
        [feedback.patternId]
      );
    }
  }

  // ============================================================================
  // RECOMMENDATION PATTERNS
  // ============================================================================

  async getPatterns(limit: number = 100): Promise<RecommendationPattern[]> {
    const result = await this.pool.query(
      `SELECT * FROM recommendation_patterns
       ORDER BY success_rate DESC
       LIMIT $1`,
      [limit]
    );

    return result.rows.map(row => this.mapPattern(row));
  }

  async getPatternById(patternId: number): Promise<RecommendationPattern | null> {
    const result = await this.pool.query(
      `SELECT * FROM recommendation_patterns WHERE id = $1`,
      [patternId]
    );

    return result.rows[0] ? this.mapPattern(result.rows[0]) : null;
  }

  // ============================================================================
  // HELPER METHODS
  // ============================================================================

  private mapUser(row: any): User {
    return {
      id: row.id,
      email: row.email,
      createdAt: new Date(row.created_at),
      updatedAt: new Date(row.updated_at)
    };
  }

  private mapContentEmbedding(row: any): ContentEmbedding {
    return {
      contentId: row.id,
      contentType: row.content_type,
      vector: row.embedding ? new Float32Array(row.embedding.slice(1, -1).split(',').map(Number)) : new Float32Array(384),
      metadata: {
        title: row.title,
        year: row.year,
        genres: row.genres || [],
        overview: row.overview || '',
        rating: parseFloat(row.rating) || 0,
        networkId: row.network_id,
        networkName: row.network_name,
        status: row.status,
        language: row.original_language,
        country: row.original_country
      },
      createdAt: new Date(row.created_at),
      updatedAt: new Date(row.updated_at)
    };
  }

  private mapUserPreference(row: any): UserPreference {
    return {
      userId: row.user_id,
      preferenceVector: row.preference_vector
        ? new Float32Array(row.preference_vector.slice(1, -1).split(',').map(Number))
        : new Float32Array(384),
      genreWeights: row.genre_weights || {},
      networkWeights: row.network_weights || {},
      watchHistory: (row.watch_history || []).map((wh: any) => ({
        contentId: wh.contentId,
        contentType: wh.contentType,
        watchedAt: new Date(wh.watchedAt),
        completionPercentage: wh.completionPercentage,
        duration: wh.duration
      })),
      ratings: (row.ratings || []).map((r: any) => ({
        contentId: r.contentId,
        rating: r.rating,
        ratedAt: new Date(r.ratedAt)
      })),
      lastUpdated: new Date(row.updated_at)
    };
  }

  private mapPattern(row: any): RecommendationPattern {
    return {
      patternId: row.id,
      taskType: row.pattern_type,
      context: {
        userSegment: row.user_segment,
        timeOfDay: row.time_of_day,
        dayOfWeek: row.day_of_week,
        platform: row.platform,
        contentTypePreference: row.content_type_preference
      },
      approach: row.approach,
      successRate: parseFloat(row.success_rate),
      totalUses: row.total_uses,
      avgReward: parseFloat(row.avg_reward),
      createdAt: new Date(row.created_at),
      updatedAt: new Date(row.updated_at)
    };
  }
}

// Factory function
export function createRepository(connectionString?: string, poolSize?: number): Repository {
  const connStr = connectionString || process.env.DATABASE_URL || 'postgresql://localhost:5432/tvdb_recommender';
  return new Repository(connStr, poolSize);
}

export default Repository;
