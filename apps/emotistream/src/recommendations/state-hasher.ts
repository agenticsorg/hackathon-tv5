/**
 * StateHasher - Discretize continuous emotional states for Q-table lookup
 */

import { EmotionalState, StateHash } from './types';

export class StateHasher {
  private valenceBuckets: number;
  private arousalBuckets: number;
  private stressBuckets: number;

  constructor(
    valenceBuckets: number = 10,
    arousalBuckets: number = 10,
    stressBuckets: number = 5
  ) {
    this.valenceBuckets = valenceBuckets;
    this.arousalBuckets = arousalBuckets;
    this.stressBuckets = stressBuckets;
  }

  /**
   * Hash emotional state to discrete buckets
   */
  hash(state: EmotionalState): StateHash {
    // Discretize valence [-1, 1] → buckets
    const valenceBucket = Math.floor(
      ((state.valence + 1.0) / 2.0) * this.valenceBuckets
    );

    // Discretize arousal [-1, 1] → buckets
    const arousalBucket = Math.floor(
      ((state.arousal + 1.0) / 2.0) * this.arousalBuckets
    );

    // Discretize stress [0, 1] → buckets
    const stressBucket = Math.floor(state.stress * this.stressBuckets);

    // Clamp to valid ranges
    const clampedValence = Math.max(0, Math.min(this.valenceBuckets - 1, valenceBucket));
    const clampedArousal = Math.max(0, Math.min(this.arousalBuckets - 1, arousalBucket));
    const clampedStress = Math.max(0, Math.min(this.stressBuckets - 1, stressBucket));

    // Create deterministic hash string
    const hashString = `v:${clampedValence}:a:${clampedArousal}:s:${clampedStress}`;

    return {
      valenceBucket: clampedValence,
      arousalBucket: clampedArousal,
      stressBucket: clampedStress,
      hash: hashString
    };
  }

  /**
   * Get total state space size
   */
  getStateSpaceSize(): number {
    return this.valenceBuckets * this.arousalBuckets * this.stressBuckets;
  }
}
