/**
 * Persistence Layer - Singleton Exports
 *
 * Provides shared instances of stores to ensure data consistency
 * across different API routes.
 */

import { FeedbackStore } from './feedback-store.js';

// Singleton instance
let feedbackStoreInstance: FeedbackStore | null = null;

/**
 * Get the shared FeedbackStore instance
 */
export function getFeedbackStore(): FeedbackStore {
  if (!feedbackStoreInstance) {
    feedbackStoreInstance = new FeedbackStore();
    feedbackStoreInstance.initialize().catch(console.error);
  }
  return feedbackStoreInstance;
}

// Re-export for convenience
export { FeedbackStore } from './feedback-store.js';
