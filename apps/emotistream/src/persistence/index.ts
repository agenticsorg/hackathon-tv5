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
  getPostgresFeedbackStore,
  getPostgresQValueStore,
  getPostgresUserStore,
  getPostgresContentStore,
} from './postgres-store.js';
import {
  initializeDatabase,
  checkConnection,
  closePool,
} from './postgres-client.js';

// Check if PostgreSQL is configured
const USE_POSTGRES = !!process.env.DB_HOST || process.env.USE_POSTGRES === 'true';

// Singleton instances for in-memory fallback
let feedbackStoreInstance: FeedbackStore | null = null;

/**
 * Initialize the persistence layer
 */
export async function initializePersistence(): Promise<boolean> {
  if (USE_POSTGRES) {
    console.log('Initializing PostgreSQL persistence...');
    try {
      const connected = await checkConnection();
      if (connected) {
        await initializeDatabase();
        console.log('PostgreSQL persistence initialized successfully');
        return true;
      }
    } catch (error) {
      console.error('PostgreSQL initialization failed:', error);
      console.log('Falling back to in-memory storage');
    }
  } else {
    console.log('Using in-memory storage (set DB_HOST or USE_POSTGRES=true for PostgreSQL)');
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
} from './postgres-store.js';
export {
  query,
  queryOne,
  queryAll,
  withTransaction,
} from './postgres-client.js';
