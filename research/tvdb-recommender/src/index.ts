/**
 * TVDB Self-Learning Recommendation System
 * Main entry point
 */

import 'dotenv/config';
import pino from 'pino';
import { createTVDBClient, TVDBClient } from './services/tvdb-client.js';
import { createEmbeddingService, EmbeddingService } from './services/embedding-service.js';
import { createRecommendationEngine, RecommendationEngine } from './services/recommendation-engine.js';
import { createRepository, Repository } from './db/repository.js';
import { createRecommendationWorkflow, RecommendationWorkflow } from './workflows/recommendation-workflow.js';
import { RecommendationRequest, RecommendationResponse, LearningFeedback } from './types/index.js';

const logger = pino({
  name: 'tvdb-recommender',
  level: process.env.LOG_LEVEL || 'info'
});

/**
 * TVDB Recommender System
 * Unified interface for the recommendation system
 */
export class TVDBRecommender {
  private tvdbClient: TVDBClient;
  private embedder: EmbeddingService;
  private engine: RecommendationEngine;
  private repository: Repository;
  private workflow: RecommendationWorkflow;
  private initialized: boolean = false;

  constructor() {
    // Initialize services
    this.tvdbClient = createTVDBClient();
    this.embedder = createEmbeddingService();
    this.engine = createRecommendationEngine(this.embedder);
    this.repository = createRepository();
    this.workflow = createRecommendationWorkflow(
      this.tvdbClient,
      this.embedder,
      this.engine,
      this.repository
    );
  }

  /**
   * Initialize all services
   */
  async initialize(): Promise<void> {
    if (this.initialized) return;

    logger.info('Initializing TVDB Recommender System');

    try {
      await this.workflow.initialize();
      this.initialized = true;
      logger.info('TVDB Recommender System initialized successfully');
    } catch (error) {
      logger.error({ error }, 'Failed to initialize TVDB Recommender System');
      throw error;
    }
  }

  /**
   * Get personalized recommendations for a user
   */
  async getRecommendations(request: RecommendationRequest): Promise<RecommendationResponse> {
    await this.ensureInitialized();

    const sessionId = this.workflow.startSession(request.userId);

    try {
      const recommendations = await this.workflow.executeRecommendationWorkflow(
        sessionId,
        request
      );
      return recommendations;
    } finally {
      this.workflow.endSession(sessionId);
    }
  }

  /**
   * Search and ingest content from TVDB
   */
  async ingestContent(query: string, limit: number = 50): Promise<{ indexed: number; errors: number }> {
    await this.ensureInitialized();

    const sessionId = this.workflow.startSession();

    try {
      return await this.workflow.executeIngestionWorkflow(sessionId, query, limit);
    } finally {
      this.workflow.endSession(sessionId);
    }
  }

  /**
   * Record user feedback and learn from it
   */
  async recordFeedback(feedback: LearningFeedback & { userId: string; contentId?: string }): Promise<void> {
    await this.ensureInitialized();

    const sessionId = this.workflow.startSession(feedback.userId);

    try {
      await this.workflow.executeLearningWorkflow(sessionId, feedback);
    } finally {
      this.workflow.endSession(sessionId);
    }
  }

  /**
   * Sync content updates from TVDB
   */
  async syncUpdates(
    since: number,
    type: 'series' | 'movies' | 'episodes' = 'series'
  ): Promise<{ synced: number; errors: number }> {
    await this.ensureInitialized();

    const sessionId = this.workflow.startSession();

    try {
      return await this.workflow.executeSyncWorkflow(sessionId, since, type);
    } finally {
      this.workflow.endSession(sessionId);
    }
  }

  /**
   * Search for content in TVDB
   */
  async searchTVDB(query: string, options?: {
    type?: 'series' | 'movie';
    year?: string;
    limit?: number;
  }): Promise<any[]> {
    await this.ensureInitialized();
    return this.tvdbClient.search(query, options);
  }

  /**
   * Get series details from TVDB
   */
  async getSeries(id: number): Promise<any> {
    await this.ensureInitialized();
    return this.tvdbClient.getSeriesExtended(id);
  }

  /**
   * Get movie details from TVDB
   */
  async getMovie(id: number): Promise<any> {
    await this.ensureInitialized();
    return this.tvdbClient.getMovie(id);
  }

  /**
   * Search content in local database
   */
  async searchLocal(query: string, limit: number = 20): Promise<any[]> {
    await this.ensureInitialized();
    return this.repository.searchContent(query, limit);
  }

  /**
   * Find similar content
   */
  async findSimilar(contentId: string, limit: number = 10): Promise<any[]> {
    await this.ensureInitialized();
    const results = await this.repository.findSimilarContent(contentId, limit);
    return results.map(r => ({
      ...r.embedding.metadata,
      contentId: r.embedding.contentId,
      contentType: r.embedding.contentType,
      similarity: r.similarity
    }));
  }

  /**
   * Get pattern learning statistics
   */
  getPatternStats(): { totalPatterns: number; avgSuccessRate: number } {
    return this.engine.getPatternStats();
  }

  /**
   * Get cache statistics
   */
  getCacheStats(): {
    tvdb: { hits: number; misses: number; keys: number };
    embeddings: { hits: number; misses: number; keys: number };
  } {
    return {
      tvdb: this.tvdbClient.getCacheStats(),
      embeddings: this.embedder.getCacheStats()
    };
  }

  /**
   * Close all connections
   */
  async close(): Promise<void> {
    await this.repository.close();
    logger.info('TVDB Recommender System closed');
  }

  /**
   * Ensure system is initialized
   */
  private async ensureInitialized(): Promise<void> {
    if (!this.initialized) {
      await this.initialize();
    }
  }
}

// ============================================================================
// CLI Interface
// ============================================================================

async function main() {
  const args = process.argv.slice(2);
  const command = args[0];

  const recommender = new TVDBRecommender();

  try {
    await recommender.initialize();

    switch (command) {
      case 'search': {
        const query = args[1];
        if (!query) {
          console.error('Usage: npm start search <query>');
          process.exit(1);
        }
        const results = await recommender.searchTVDB(query);
        console.log(JSON.stringify(results, null, 2));
        break;
      }

      case 'ingest': {
        const query = args[1];
        const limit = parseInt(args[2]) || 50;
        if (!query) {
          console.error('Usage: npm start ingest <query> [limit]');
          process.exit(1);
        }
        const result = await recommender.ingestContent(query, limit);
        console.log(`Indexed: ${result.indexed}, Errors: ${result.errors}`);
        break;
      }

      case 'recommend': {
        const userId = args[1];
        if (!userId) {
          console.error('Usage: npm start recommend <userId> [limit]');
          process.exit(1);
        }
        const limit = parseInt(args[2]) || 10;
        const recommendations = await recommender.getRecommendations({
          userId,
          limit
        });
        console.log(JSON.stringify(recommendations, null, 2));
        break;
      }

      case 'similar': {
        const contentId = args[1];
        if (!contentId) {
          console.error('Usage: npm start similar <contentId>');
          process.exit(1);
        }
        const similar = await recommender.findSimilar(contentId);
        console.log(JSON.stringify(similar, null, 2));
        break;
      }

      case 'stats': {
        console.log('Pattern Stats:', recommender.getPatternStats());
        console.log('Cache Stats:', recommender.getCacheStats());
        break;
      }

      default:
        console.log(`
TVDB Self-Learning Recommendation System

Commands:
  search <query>              Search TVDB for content
  ingest <query> [limit]      Ingest content from TVDB to local database
  recommend <userId> [limit]  Get personalized recommendations
  similar <contentId>         Find similar content
  stats                       Show system statistics

Environment Variables:
  TVDB_API_KEY     TheTVDB API key (required)
  TVDB_PIN         TheTVDB subscriber PIN (optional)
  DATABASE_URL     PostgreSQL connection string
  LOG_LEVEL        Logging level (default: info)
        `);
    }
  } catch (error) {
    logger.error({ error }, 'Command failed');
    process.exit(1);
  } finally {
    await recommender.close();
  }
}

// Export for programmatic use
export { TVDBRecommender };
export { createTVDBClient, TVDBClient } from './services/tvdb-client.js';
export { createEmbeddingService, EmbeddingService } from './services/embedding-service.js';
export { createRecommendationEngine, RecommendationEngine } from './services/recommendation-engine.js';
export { createRepository, Repository } from './db/repository.js';
export { createRecommendationWorkflow, RecommendationWorkflow } from './workflows/recommendation-workflow.js';
export * from './types/index.js';

// Run CLI if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}
