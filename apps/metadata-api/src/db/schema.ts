/**
 * Firestore Collection Definitions and Indexes
 * Nexus-UMMID Entertainment Discovery Platform
 *
 * Optimized for 400M+ users with hypergraph query patterns
 */

import { getFirestore, CollectionReference } from './firestore';
import type {
  MediaMetadata,
  UserProfile,
  UserPreference,
  ContentRecommendation,
  RecommendationSet,
  StreamingPlatform,
  Hyperedge,
  ContentMetrics,
} from './models';

// ============================================================================
// COLLECTION NAMES
// ============================================================================

export const COLLECTIONS = {
  // Core entities
  CONTENT: 'content',
  USERS: 'users',
  USER_PREFERENCES: 'user_preferences',
  RECOMMENDATIONS: 'recommendations',
  RECOMMENDATION_SETS: 'recommendation_sets',
  PLATFORMS: 'platforms',

  // Hypergraph
  HYPEREDGES: 'hyperedges',

  // Analytics
  CONTENT_METRICS: 'content_metrics',

  // System
  API_REQUESTS: 'api_requests',
  EIDR_CACHE: 'eidr_cache',
  HEALTH: '_health',
} as const;

// ============================================================================
// TYPED COLLECTION REFERENCES
// ============================================================================

/**
 * Get typed collection reference for content
 */
export function getContentCollection(): CollectionReference<MediaMetadata> {
  const db = getFirestore();
  return db.collection(COLLECTIONS.CONTENT) as CollectionReference<MediaMetadata>;
}

/**
 * Get typed collection reference for users
 */
export function getUsersCollection(): CollectionReference<UserProfile> {
  const db = getFirestore();
  return db.collection(COLLECTIONS.USERS) as CollectionReference<UserProfile>;
}

/**
 * Get typed collection reference for user preferences
 */
export function getUserPreferencesCollection(): CollectionReference<UserPreference> {
  const db = getFirestore();
  return db.collection(COLLECTIONS.USER_PREFERENCES) as CollectionReference<UserPreference>;
}

/**
 * Get typed collection reference for recommendations
 */
export function getRecommendationsCollection(): CollectionReference<ContentRecommendation> {
  const db = getFirestore();
  return db.collection(COLLECTIONS.RECOMMENDATIONS) as CollectionReference<ContentRecommendation>;
}

/**
 * Get typed collection reference for recommendation sets
 */
export function getRecommendationSetsCollection(): CollectionReference<RecommendationSet> {
  const db = getFirestore();
  return db.collection(COLLECTIONS.RECOMMENDATION_SETS) as CollectionReference<RecommendationSet>;
}

/**
 * Get typed collection reference for platforms
 */
export function getPlatformsCollection(): CollectionReference<StreamingPlatform> {
  const db = getFirestore();
  return db.collection(COLLECTIONS.PLATFORMS) as CollectionReference<StreamingPlatform>;
}

/**
 * Get typed collection reference for hyperedges
 */
export function getHyperedgesCollection(): CollectionReference<Hyperedge> {
  const db = getFirestore();
  return db.collection(COLLECTIONS.HYPEREDGES) as CollectionReference<Hyperedge>;
}

/**
 * Get typed collection reference for content metrics
 */
export function getContentMetricsCollection(): CollectionReference<ContentMetrics> {
  const db = getFirestore();
  return db.collection(COLLECTIONS.CONTENT_METRICS) as CollectionReference<ContentMetrics>;
}

// ============================================================================
// INDEX DEFINITIONS
// ============================================================================

/**
 * Required Firestore composite indexes
 *
 * These should be created via firestore.indexes.json or Firebase console
 * for optimal query performance at scale.
 */
export const REQUIRED_INDEXES = {
  // Content queries
  content_by_genre_and_year: {
    collection: COLLECTIONS.CONTENT,
    fields: [
      { field: 'genres', order: 'ASCENDING', arrayConfig: 'CONTAINS' },
      { field: 'release_year', order: 'DESCENDING' },
      { field: 'popularity_score', order: 'DESCENDING' },
    ],
  },

  content_by_platform_and_rating: {
    collection: COLLECTIONS.CONTENT,
    fields: [
      { field: 'platform', order: 'ASCENDING', arrayConfig: 'CONTAINS' },
      { field: 'rating', order: 'ASCENDING' },
      { field: 'popularity_score', order: 'DESCENDING' },
    ],
  },

  content_temporal_queries: {
    collection: COLLECTIONS.CONTENT,
    fields: [
      { field: 'valid_from', order: 'ASCENDING' },
      { field: 'valid_to', order: 'DESCENDING' },
      { field: 'version', order: 'DESCENDING' },
    ],
  },

  // User queries
  user_watch_history: {
    collection: COLLECTIONS.USERS,
    fields: [
      { field: 'user_id', order: 'ASCENDING' },
      { field: 'watch_history.watched_at', order: 'DESCENDING' },
    ],
  },

  // Recommendation queries
  recommendations_by_user_and_score: {
    collection: COLLECTIONS.RECOMMENDATIONS,
    fields: [
      { field: 'user_id', order: 'ASCENDING' },
      { field: 'score', order: 'DESCENDING' },
      { field: 'generated_at', order: 'DESCENDING' },
    ],
  },

  recommendations_by_audience: {
    collection: COLLECTIONS.RECOMMENDATIONS,
    fields: [
      { field: 'target_audience', order: 'ASCENDING' },
      { field: 'score', order: 'DESCENDING' },
    ],
  },

  // Hypergraph queries
  hyperedge_by_type_and_validity: {
    collection: COLLECTIONS.HYPEREDGES,
    fields: [
      { field: 'edge_type', order: 'ASCENDING' },
      { field: 'valid_from', order: 'ASCENDING' },
      { field: 'valid_to', order: 'DESCENDING' },
    ],
  },

  // Analytics queries
  content_metrics_trending: {
    collection: COLLECTIONS.CONTENT_METRICS,
    fields: [
      { field: 'trending_score', order: 'DESCENDING' },
      { field: 'updated_at', order: 'DESCENDING' },
    ],
  },
};

// ============================================================================
// QUERY HELPERS
// ============================================================================

/**
 * Query content by genre with pagination
 */
export async function queryContentByGenre(
  genre: string,
  limit: number = 20,
  startAfter?: any
) {
  const collection = getContentCollection();

  let query = collection
    .where('genres', 'array-contains', genre)
    .orderBy('popularity_score', 'desc')
    .limit(limit);

  if (startAfter) {
    query = query.startAfter(startAfter);
  }

  return query.get();
}

/**
 * Query content by platform and availability
 */
export async function queryContentByPlatform(
  platform: string,
  region: string = 'US',
  limit: number = 20
) {
  const collection = getContentCollection();

  return collection
    .where('platform', 'array-contains', platform)
    .where('availability', 'array-contains', {
      platform,
      region,
    })
    .orderBy('popularity_score', 'desc')
    .limit(limit)
    .get();
}

/**
 * Query recommendations for a user
 */
export async function queryUserRecommendations(
  userId: string,
  limit: number = 10
) {
  const collection = getRecommendationsCollection();

  return collection
    .where('user_id', '==', userId)
    .orderBy('score', 'desc')
    .limit(limit)
    .get();
}

/**
 * Query hyperedges by type with temporal filtering
 */
export async function queryHyperedgesByType(
  edgeType: string,
  asOfDate: Date = new Date()
) {
  const collection = getHyperedgesCollection();
  const timestamp = asOfDate;

  return collection
    .where('edge_type', '==', edgeType)
    .where('valid_from', '<=', timestamp)
    .where('valid_to', '>=', timestamp)
    .get();
}

/**
 * Query trending content
 */
export async function queryTrendingContent(limit: number = 20) {
  const collection = getContentMetricsCollection();

  const metricsSnapshot = await collection
    .orderBy('trending_score', 'desc')
    .limit(limit)
    .get();

  // Fetch actual content documents
  const contentIds = metricsSnapshot.docs.map(doc => doc.data().content_id);
  const contentCollection = getContentCollection();

  const contentPromises = contentIds.map(id =>
    contentCollection.doc(id).get()
  );

  return Promise.all(contentPromises);
}

// ============================================================================
// BATCH OPERATIONS
// ============================================================================

/**
 * Batch upsert content metadata
 */
export async function batchUpsertContent(
  items: MediaMetadata[]
): Promise<void> {
  const db = getFirestore();
  const collection = getContentCollection();
  const BATCH_SIZE = 500;

  for (let i = 0; i < items.length; i += BATCH_SIZE) {
    const batch = db.batch();
    const chunk = items.slice(i, i + BATCH_SIZE);

    chunk.forEach(item => {
      const ref = collection.doc(item.id);
      batch.set(ref, item, { merge: true });
    });

    await batch.commit();
  }
}

/**
 * Batch update user watch history
 */
export async function batchUpdateWatchHistory(
  updates: Array<{
    userId: string;
    contentId: string;
    progressPercent: number;
    timestamp: Date;
  }>
): Promise<void> {
  const db = getFirestore();
  const collection = getUserPreferencesCollection();
  const BATCH_SIZE = 500;

  for (let i = 0; i < updates.length; i += BATCH_SIZE) {
    const batch = db.batch();
    const chunk = updates.slice(i, i + BATCH_SIZE);

    chunk.forEach(update => {
      const ref = collection.doc(update.userId);
      batch.set(
        ref,
        {
          watchHistory: [
            {
              contentId: update.contentId,
              timestamp: update.timestamp,
            },
          ],
          updated_at: new Date(),
        },
        { merge: true }
      );
    });

    await batch.commit();
  }
}

// ============================================================================
// MIGRATION HELPERS
// ============================================================================

/**
 * Initialize database with required collections and indexes
 */
export async function initializeDatabase(): Promise<void> {
  const db = getFirestore();

  // Create health check document
  await db.collection(COLLECTIONS.HEALTH).doc('status').set({
    initialized: true,
    timestamp: new Date(),
  });

  console.log('Database initialized successfully');
  console.log('\nRequired indexes:');
  console.log(JSON.stringify(REQUIRED_INDEXES, null, 2));
  console.log('\nCreate these indexes in Firebase Console or via firestore.indexes.json');
}

/**
 * Validate database schema and indexes
 */
export async function validateDatabaseSchema(): Promise<{
  valid: boolean;
  errors: string[];
}> {
  const errors: string[] = [];

  try {
    const db = getFirestore();

    // Check if collections exist (by trying to read from them)
    const collectionsToCheck = Object.values(COLLECTIONS);

    for (const collectionName of collectionsToCheck) {
      try {
        await db.collection(collectionName).limit(1).get();
      } catch (error) {
        errors.push(`Collection ${collectionName} is not accessible`);
      }
    }

    return {
      valid: errors.length === 0,
      errors,
    };
  } catch (error) {
    errors.push(`Schema validation failed: ${error}`);
    return { valid: false, errors };
  }
}

// ============================================================================
// EXPORT ALL
// ============================================================================

export {
  getFirestore,
  batchWrite,
  runTransaction,
  closeFirestore,
  checkFirestoreHealth,
} from './firestore';

export * from './models';
