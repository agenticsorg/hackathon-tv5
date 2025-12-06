/**
 * Multi-Factor Reward Calculator
 * EmotiStream MVP
 *
 * Reward Formula:
 * reward = 0.6 × directionAlignment + 0.4 × magnitude + proximityBonus
 *
 * Where:
 * - directionAlignment: cosine similarity between actual and desired movement [-1, 1]
 * - magnitude: normalized change magnitude [0, 1]
 * - proximityBonus: +0.1 if distance to desired < 0.3
 */

import type { EmotionalState, DesiredState } from '../emotion/types';
import type { RewardComponents } from './types';

export class RewardCalculator {
  private readonly DIRECTION_WEIGHT = 0.6;
  private readonly MAGNITUDE_WEIGHT = 0.4;
  private readonly PROXIMITY_THRESHOLD = 0.3;
  private readonly PROXIMITY_BONUS = 0.1;

  /**
   * Calculate reward based on emotional state transition
   */
  calculate(
    stateBefore: EmotionalState,
    stateAfter: EmotionalState,
    desiredState: DesiredState
  ): number {
    const components = this.calculateComponents(stateBefore, stateAfter, desiredState);
    return components.totalReward;
  }

  /**
   * Calculate all reward components with detailed breakdown
   */
  calculateComponents(
    stateBefore: EmotionalState,
    stateAfter: EmotionalState,
    desiredState: DesiredState
  ): RewardComponents {
    // Component 1: Direction Alignment (60% weight)
    // Measures if movement is in the right direction
    const directionAlignment = this.calculateDirectionAlignment(
      stateBefore,
      stateAfter,
      desiredState
    );

    // Component 2: Magnitude (40% weight)
    // Measures how much emotional change occurred
    const magnitude = this.calculateMagnitude(stateBefore, stateAfter);

    // Component 3: Proximity Bonus
    // Bonus if we got close to the desired state
    const proximityBonus = this.calculateProximityBonus(stateAfter, desiredState);

    // Component 4: Completion Penalty (applied separately)
    const completionPenalty = 0;

    // Calculate total reward
    const baseReward =
      directionAlignment * this.DIRECTION_WEIGHT +
      magnitude * this.MAGNITUDE_WEIGHT;

    const totalReward = this.clamp(baseReward + proximityBonus, -1, 1);

    return {
      directionAlignment,
      magnitude,
      proximityBonus,
      completionPenalty,
      totalReward,
    };
  }

  /**
   * Calculate direction alignment using cosine similarity
   * Returns value in range [-1, 1]:
   *  1.0 = perfect alignment (same direction)
   *  0.0 = perpendicular
   * -1.0 = opposite direction
   */
  private calculateDirectionAlignment(
    stateBefore: EmotionalState,
    stateAfter: EmotionalState,
    desiredState: DesiredState
  ): number {
    // Calculate actual emotional change vector
    const actualDelta = {
      valence: stateAfter.valence - stateBefore.valence,
      arousal: stateAfter.arousal - stateBefore.arousal,
    };

    // Calculate desired emotional change vector
    const desiredDelta = {
      valence: desiredState.targetValence - stateBefore.valence,
      arousal: desiredState.targetArousal - stateBefore.arousal,
    };

    // Calculate cosine similarity: cos(θ) = (A·B) / (|A||B|)
    const dotProduct =
      actualDelta.valence * desiredDelta.valence +
      actualDelta.arousal * desiredDelta.arousal;

    const actualMagnitude = Math.sqrt(
      actualDelta.valence ** 2 + actualDelta.arousal ** 2
    );

    const desiredMagnitude = Math.sqrt(
      desiredDelta.valence ** 2 + desiredDelta.arousal ** 2
    );

    // Handle edge cases
    if (actualMagnitude === 0 || desiredMagnitude === 0) {
      return 0.0; // No change or no desired change
    }

    const alignment = dotProduct / (actualMagnitude * desiredMagnitude);
    return this.clamp(alignment, -1, 1);
  }

  /**
   * Calculate magnitude of emotional change
   * Returns normalized value in range [0, 1]
   */
  private calculateMagnitude(
    stateBefore: EmotionalState,
    stateAfter: EmotionalState
  ): number {
    const deltaValence = stateAfter.valence - stateBefore.valence;
    const deltaArousal = stateAfter.arousal - stateBefore.arousal;

    // Euclidean distance in 2D emotional space
    const distance = Math.sqrt(deltaValence ** 2 + deltaArousal ** 2);

    // Normalize by maximum possible distance (diagonal of 2x2 square = 2√2)
    const maxDistance = Math.sqrt(2 * 2 + 2 * 2); // 2.828
    const normalized = distance / maxDistance;

    return this.clamp(normalized, 0, 1);
  }

  /**
   * Calculate proximity bonus if close to desired state
   * Returns 0.1 if within threshold, 0 otherwise
   */
  private calculateProximityBonus(
    stateAfter: EmotionalState,
    desiredState: DesiredState
  ): number {
    const deltaValence = stateAfter.valence - desiredState.targetValence;
    const deltaArousal = stateAfter.arousal - desiredState.targetArousal;

    const distance = Math.sqrt(deltaValence ** 2 + deltaArousal ** 2);

    return distance < this.PROXIMITY_THRESHOLD ? this.PROXIMITY_BONUS : 0;
  }

  /**
   * Calculate completion penalty based on watch behavior
   * Returns value in range [-0.2, 0]
   */
  calculateCompletionPenalty(completed: boolean, watchDuration: number, totalDuration: number): number {
    if (completed) {
      return 0; // No penalty
    }

    const completionRate = totalDuration > 0 ? watchDuration / totalDuration : 0;

    if (completionRate < 0.2) {
      return -0.2; // Strong penalty for early abandonment
    } else if (completionRate < 0.5) {
      return -0.1; // Moderate penalty
    } else {
      return -0.05; // Small penalty
    }
  }

  /**
   * Clamp value to range [min, max]
   */
  private clamp(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, value));
  }
}
