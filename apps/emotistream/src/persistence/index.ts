/**
 * Persistence Layer - Unified Exports
 *
 * Provides shared instances of stores with PostgreSQL or in-memory fallback.
 * Uses PostgreSQL when DB_HOST is configured, otherwise falls back to in-memory.
 */

import { FeedbackStore } from './feedback-store.js';
import {
  PostgresFeedbackStore,
  PostgresQValueStore,
  PostgresUserStore,
  PostgresContentStore,
  PostgresRecommendationHistoryStore,
  PostgresEmotionHistoryStore,
  getPostgresFeedbackStore,
  getPostgresQValueStore,
  getPostgresUserStore,
  getPostgresContentStore,
  getPostgresRecommendationHistoryStore,
  getPostgresEmotionHistoryStore,
} from './postgres-store.js';
import {
  initializeDatabase,
  checkConnection,
  closePool,
} from './postgres-client.js';
import { createLogger } from '../utils/logger.js';

const logger = createLogger('Persistence');

// Check if PostgreSQL is configured
const USE_POSTGRES = !!process.env.DB_HOST || process.env.USE_POSTGRES === 'true';

// Singleton instances for in-memory fallback
let feedbackStoreInstance: FeedbackStore | null = null;

/**
 * Initialize the persistence layer
 */
export async function initializePersistence(): Promise<boolean> {
  if (USE_POSTGRES) {
    logger.info('Initializing PostgreSQL persistence...');
    try {
      const connected = await checkConnection();
      if (connected) {
        await initializeDatabase();
        logger.info('PostgreSQL persistence initialized successfully');
        return true;
      }
    } catch (error) {
      logger.error('PostgreSQL initialization failed', error);
      logger.warn('Falling back to in-memory storage');
    }
  } else {
    logger.info('Using in-memory storage (set DB_HOST or USE_POSTGRES=true for PostgreSQL)');
  }
  return false;
}

/**
 * Check if using PostgreSQL
 */
export function isUsingPostgres(): boolean {
  return USE_POSTGRES;
}

/**
 * Get the shared FeedbackStore instance
 * Returns PostgreSQL store if configured, otherwise in-memory
 */
export function getFeedbackStore(): FeedbackStore | PostgresFeedbackStore {
  if (USE_POSTGRES) {
    return getPostgresFeedbackStore();
  }

  if (!feedbackStoreInstance) {
    feedbackStoreInstance = new FeedbackStore();
    feedbackStoreInstance.initialize().catch(console.error);
  }
  return feedbackStoreInstance;
}

/**
 * Get Q-Value store (PostgreSQL only)
 */
export function getQValueStore(): PostgresQValueStore {
  return getPostgresQValueStore();
}

/**
 * Get User store (PostgreSQL only)
 */
export function getUserStore(): PostgresUserStore {
  return getPostgresUserStore();
}

/**
 * Get Content store (PostgreSQL only)
 */
export function getContentStore(): PostgresContentStore {
  return getPostgresContentStore();
}

/**
 * Get Recommendation History store (PostgreSQL only)
 */
export function getRecommendationHistoryStore(): PostgresRecommendationHistoryStore {
  return getPostgresRecommendationHistoryStore();
}

/**
 * Get Emotion History store (PostgreSQL only)
 */
export function getEmotionHistoryStore(): PostgresEmotionHistoryStore {
  return getPostgresEmotionHistoryStore();
}

/**
 * Close all database connections
 */
export async function closePersistence(): Promise<void> {
  if (USE_POSTGRES) {
    await closePool();
  }
}

// Re-export stores
export { FeedbackStore } from './feedback-store.js';
export {
  PostgresFeedbackStore,
  PostgresQValueStore,
  PostgresUserStore,
  PostgresContentStore,
  PostgresRecommendationHistoryStore,
  PostgresEmotionHistoryStore,
} from './postgres-store.js';
export {
  query,
  queryOne,
  queryAll,
  withTransaction,
} from './postgres-client.js';
