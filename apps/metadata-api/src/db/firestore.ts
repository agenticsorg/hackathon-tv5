/**
 * Firestore Client Initialization
 * Nexus-UMMID Entertainment Discovery Platform
 *
 * Optimized for 400M+ users with hypergraph queries
 */

import { Firestore, Settings } from '@google-cloud/firestore';
import { logger } from '../utils/logger';

// Firestore client instance (singleton)
let firestoreInstance: Firestore | null = null;

/**
 * Firestore configuration for production scale
 */
const FIRESTORE_CONFIG: Settings = {
  projectId: process.env.GCP_PROJECT_ID || 'agentics-foundation25lon-1899',

  // Performance optimizations for high-scale operations
  ignoreUndefinedProperties: true,

  // Connection pooling for 400M+ users
  maxIdleChannels: 10,

  // Prefer HTTP/2 for better throughput
  preferRest: false,

  // Credentials - only use keyFilename if explicitly set (not in Cloud Run)
  ...(process.env.GOOGLE_APPLICATION_CREDENTIALS && {
    keyFilename: process.env.GOOGLE_APPLICATION_CREDENTIALS,
  }),
};

/**
 * Initialize and return Firestore client instance
 * Uses singleton pattern to prevent connection pool exhaustion
 */
export function getFirestore(): Firestore {
  if (!firestoreInstance) {
    try {
      firestoreInstance = new Firestore(FIRESTORE_CONFIG);

      // Configure batch settings for bulk operations
      firestoreInstance.settings({
        timestampsInSnapshots: true,
      });

      logger.info('Firestore client initialized successfully', {
        project: FIRESTORE_CONFIG.projectId,
      });
    } catch (error) {
      logger.error('Failed to initialize Firestore client', { error });
      throw new Error(`Firestore initialization failed: ${error}`);
    }
  }

  return firestoreInstance;
}

/**
 * Batch write helper for bulk operations
 * Handles automatic chunking for >500 operations
 */
export async function batchWrite(
  operations: Array<{
    type: 'set' | 'update' | 'delete';
    ref: FirebaseFirestore.DocumentReference;
    data?: any;
  }>
): Promise<void> {
  const db = getFirestore();
  const BATCH_SIZE = 500; // Firestore limit

  for (let i = 0; i < operations.length; i += BATCH_SIZE) {
    const batch = db.batch();
    const chunk = operations.slice(i, i + BATCH_SIZE);

    chunk.forEach(({ type, ref, data }) => {
      switch (type) {
        case 'set':
          batch.set(ref, data);
          break;
        case 'update':
          batch.update(ref, data);
          break;
        case 'delete':
          batch.delete(ref);
          break;
      }
    });

    await batch.commit();

    logger.debug(`Batch committed: ${i + chunk.length}/${operations.length}`);
  }
}

/**
 * Transaction helper with retry logic
 */
export async function runTransaction<T>(
  updateFunction: (transaction: FirebaseFirestore.Transaction) => Promise<T>,
  maxAttempts: number = 5
): Promise<T> {
  const db = getFirestore();

  let lastError: Error | null = null;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await db.runTransaction(updateFunction);
    } catch (error) {
      lastError = error as Error;

      if (attempt < maxAttempts) {
        const backoffMs = Math.min(1000 * Math.pow(2, attempt), 10000);
        logger.warn(`Transaction failed, retrying in ${backoffMs}ms`, {
          attempt,
          error: error instanceof Error ? error.message : error,
        });
        await new Promise(resolve => setTimeout(resolve, backoffMs));
      }
    }
  }

  throw new Error(
    `Transaction failed after ${maxAttempts} attempts: ${lastError?.message}`
  );
}

/**
 * Cleanup function for graceful shutdown
 */
export async function closeFirestore(): Promise<void> {
  if (firestoreInstance) {
    await firestoreInstance.terminate();
    firestoreInstance = null;
    logger.info('Firestore client terminated');
  }
}

/**
 * Health check for Firestore connection
 */
export async function checkFirestoreHealth(): Promise<boolean> {
  try {
    const db = getFirestore();
    // Simple read to verify connection
    await db.collection('_health').limit(1).get();
    return true;
  } catch (error) {
    logger.error('Firestore health check failed', { error });
    return false;
  }
}

// Export Firestore types for convenience
export type {
  Firestore,
  DocumentReference,
  CollectionReference,
  Query,
  QuerySnapshot,
  DocumentSnapshot,
  Timestamp,
  FieldValue,
  WriteBatch,
  Transaction,
} from '@google-cloud/firestore';
