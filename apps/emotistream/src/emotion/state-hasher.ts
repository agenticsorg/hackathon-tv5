/**
 * State Hasher
 * Discretizes continuous emotional state space for Q-learning
 */

import { EmotionalState } from './types';

/**
 * Discretization buckets
 * 5×5×3 grid = 75 possible states
 */
const VALENCE_BUCKETS = 5;
const AROUSAL_BUCKETS = 5;
const STRESS_BUCKETS = 3;

/**
 * Discretize a continuous value into buckets
 * @param value - Value to discretize (-1 to +1 for valence/arousal, 0 to 1 for stress)
 * @param buckets - Number of buckets
 * @param min - Minimum value (default: -1 for valence/arousal, 0 for stress)
 * @param max - Maximum value (default: +1)
 * @returns Bucket index (0 to buckets-1)
 */
function discretizeValue(
  value: number,
  buckets: number,
  min: number = -1,
  max: number = 1
): number {
  // Clamp value to [min, max]
  const clamped = Math.max(min, Math.min(max, value));

  // Map to [0, buckets-1]
  const normalized = (clamped - min) / (max - min);
  const bucket = Math.floor(normalized * buckets);

  // Ensure we don't exceed bucket range due to floating point precision
  return Math.min(bucket, buckets - 1);
}

/**
 * Hash emotional state into discrete state space
 * @param state - Emotional state to hash
 * @returns State hash string in format "v:a:s" (e.g., "2:3:1")
 */
export function hashState(state: EmotionalState): string {
  const valenceBucket = discretizeValue(state.valence, VALENCE_BUCKETS, -1, 1);
  const arousalBucket = discretizeValue(state.arousal, AROUSAL_BUCKETS, -1, 1);
  const stressBucket = discretizeValue(state.stressLevel, STRESS_BUCKETS, 0, 1);

  return `${valenceBucket}:${arousalBucket}:${stressBucket}`;
}
