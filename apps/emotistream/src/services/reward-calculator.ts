/**
 * Reward Calculator Service
 *
 * Calculates rewards for feedback submissions with detailed breakdowns.
 */

import { EmotionalState } from '../types/index.js';
import { RewardCalculation } from '../types/feedback.js';

export class RewardCalculator {
  /**
   * Calculate reward from feedback
   */
  calculate(
    emotionBefore: EmotionalState,
    emotionAfter: EmotionalState,
    desiredState: EmotionalState,
    completed: boolean,
    starRating: number,
    watchDuration: number,
    totalDuration: number
  ): RewardCalculation {
    // 1. Emotional alignment: How well did we move toward desired state?
    const emotionalAlignment = this.calculateEmotionalAlignment(
      emotionBefore,
      emotionAfter,
      desiredState
    );

    // 2. Completion bonus: Did they finish the content?
    const completionBonus = this.calculateCompletionBonus(
      completed,
      watchDuration,
      totalDuration
    );

    // 3. Rating bonus: What did they explicitly rate?
    const ratingBonus = this.calculateRatingBonus(starRating);

    // Combine components (weighted)
    const reward = (
      0.6 * emotionalAlignment +
      0.25 * completionBonus +
      0.15 * ratingBonus
    );

    // Clamp to [-1, 1]
    const clampedReward = Math.max(-1, Math.min(1, reward));

    const explanation = this.generateExplanation(
      emotionalAlignment,
      completionBonus,
      ratingBonus,
      clampedReward
    );

    return {
      reward: clampedReward,
      components: {
        emotionalAlignment,
        completionBonus,
        ratingBonus,
      },
      explanation,
    };
  }

  /**
   * Calculate emotional alignment score
   */
  private calculateEmotionalAlignment(
    before: EmotionalState,
    after: EmotionalState,
    desired: EmotionalState
  ): number {
    // Calculate distance from desired state before and after
    const distanceBefore = this.emotionalDistance(before, desired);
    const distanceAfter = this.emotionalDistance(after, desired);

    // Improvement is negative change in distance (closer is better)
    const improvement = distanceBefore - distanceAfter;

    // Normalize to [-1, 1]
    // Max possible improvement is ~3.46 (sqrt(12)) for 3D unit space
    const maxImprovement = Math.sqrt(12);
    return improvement / maxImprovement;
  }

  /**
   * Calculate Euclidean distance between emotional states
   */
  private emotionalDistance(state1: EmotionalState, state2: EmotionalState): number {
    const dValence = state1.valence - state2.valence;
    const dArousal = state1.arousal - state2.arousal;
    const dStress = state1.stressLevel - state2.stressLevel;

    return Math.sqrt(dValence ** 2 + dArousal ** 2 + dStress ** 2);
  }

  /**
   * Calculate completion bonus
   */
  private calculateCompletionBonus(
    completed: boolean,
    watchDuration: number,
    totalDuration: number
  ): number {
    if (completed) {
      return 1.0;
    }

    // Partial credit based on percentage watched
    const percentage = watchDuration / totalDuration;

    if (percentage < 0.1) return -0.5; // Barely watched
    if (percentage < 0.25) return -0.2;
    if (percentage < 0.5) return 0.0;
    if (percentage < 0.75) return 0.3;
    return 0.6; // Almost finished
  }

  /**
   * Calculate rating bonus
   */
  private calculateRatingBonus(starRating: number): number {
    // Map 1-5 stars to [-1, 1]
    // 3 stars = neutral (0)
    // 5 stars = +1
    // 1 star = -1
    return (starRating - 3) / 2;
  }

  /**
   * Generate human-readable explanation
   */
  private generateExplanation(
    emotional: number,
    completion: number,
    rating: number,
    total: number
  ): string {
    const parts: string[] = [];

    // Emotional component
    if (emotional > 0.3) {
      parts.push(`You moved significantly closer to your desired emotional state (+${(emotional * 100).toFixed(0)}%)`);
    } else if (emotional < -0.3) {
      parts.push(`Your emotions moved away from your goal (${(emotional * 100).toFixed(0)}%)`);
    } else {
      parts.push('Your emotional state had minimal change');
    }

    // Completion component
    if (completion === 1.0) {
      parts.push('You completed the content');
    } else if (completion > 0) {
      parts.push(`You watched ${(completion * 100 + 50).toFixed(0)}% of the content`);
    } else {
      parts.push('You stopped watching early');
    }

    // Rating component
    if (rating > 0.3) {
      parts.push('You gave a high rating');
    } else if (rating < -0.3) {
      parts.push('You gave a low rating');
    }

    // Overall
    let overall: string;
    if (total > 0.5) {
      overall = '✨ Great choice! ';
    } else if (total > 0) {
      overall = '👍 Good choice. ';
    } else if (total > -0.3) {
      overall = '🤔 This was okay. ';
    } else {
      overall = '😔 This wasn\'t ideal. ';
    }

    return overall + parts.join('. ') + '.';
  }
}
