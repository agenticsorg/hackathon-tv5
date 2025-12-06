/**
 * Samsung TV Learning Module
 *
 * On-device WASM-accelerated self-learning for content recommendations
 * Uses Q-Learning with experience replay and pattern storage
 */

// Types
export * from './types.js';

// Embeddings
export {
  cosineSimilarity,
  batchSimilarity,
  generateContentEmbedding,
  generatePreferenceEmbedding,
  generateStateEmbedding,
  ContentEmbeddingCache,
} from './embeddings.js';

// Learning System
export { PreferenceLearningSystem } from './preference-learning.js';

// Persistence
export { LearningPersistence, IndexedDBPersistence } from './persistence.js';
