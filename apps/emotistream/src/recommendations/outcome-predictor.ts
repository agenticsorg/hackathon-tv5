/**
 * OutcomePredictor - Predict post-viewing emotional states
 */

import { EmotionalState, PredictedOutcome } from './types';
import { EmotionalContentProfile } from '../content/types';

export class OutcomePredictor {
  /**
   * Predict emotional state after viewing content
   */
  predict(
    currentState: EmotionalState,
    contentProfile: EmotionalContentProfile
  ): PredictedOutcome {
    // Calculate post-viewing state by applying deltas
    let postValence = currentState.valence + contentProfile.valenceDelta;
    let postArousal = currentState.arousal + contentProfile.arousalDelta;
    let postStress = Math.max(0.0, currentState.stress - (contentProfile.intensity * 0.3));

    // Clamp to valid ranges
    postValence = this.clamp(postValence, -1.0, 1.0);
    postArousal = this.clamp(postArousal, -1.0, 1.0);
    postStress = this.clamp(postStress, 0.0, 1.0);

    // Calculate confidence based on content complexity and intensity
    // Higher complexity = lower confidence in prediction
    // More data/watches would increase confidence (simulated here)
    const baseConfidence = 0.7;
    const complexityPenalty = contentProfile.complexity * 0.2;
    const confidence = this.clamp(baseConfidence - complexityPenalty, 0.3, 0.95);

    return {
      expectedValence: postValence,
      expectedArousal: postArousal,
      expectedStress: postStress,
      confidence
    };
  }

  private clamp(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, value));
  }
}
