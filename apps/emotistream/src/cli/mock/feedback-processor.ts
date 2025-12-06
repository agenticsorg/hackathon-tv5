/**
 * Mock Feedback Processor for CLI Demo
 *
 * Simulates feedback processing and Q-value updates.
 */

import {
  FeedbackRequest,
  FeedbackResponse,
  EmotionalState,
  DesiredState,
  LearningProgress
} from '../../types/index.js';

/**
 * Mock feedback processor with RL update simulation
 */
export class MockFeedbackProcessor {
  private totalExperiences: number;
  private rewardHistory: number[];
  private explorationRate: number;

  constructor() {
    this.totalExperiences = 0;
    this.rewardHistory = [];
    this.explorationRate = 0.2;
  }

  /**
   * Process feedback and update Q-values
   */
  async processFeedback(
    request: FeedbackRequest,
    stateBefore: EmotionalState,
    desiredState: DesiredState
  ): Promise<FeedbackResponse> {
    // Calculate multi-factor reward
    const reward = this.calculateReward(
      stateBefore,
      request.actualPostState,
      desiredState,
      request.completed
    );

    // Simulate Q-value update
    const oldQValue = 0.5; // Simplified for demo
    const learningRate = 0.1;
    const newQValue = oldQValue + learningRate * (reward - oldQValue);

    // Update statistics
    this.totalExperiences++;
    this.rewardHistory.push(reward);

    // Decay exploration rate
    this.explorationRate = Math.max(0.05, this.explorationRate * 0.99);

    // Calculate learning progress
    const learningProgress: LearningProgress = {
      totalExperiences: this.totalExperiences,
      avgReward: this.calculateAvgReward(),
      explorationRate: this.explorationRate,
      convergenceScore: this.calculateConvergence()
    };

    return {
      reward,
      policyUpdated: true,
      newQValue,
      learningProgress
    };
  }

  /**
   * Calculate multi-factor reward
   */
  private calculateReward(
    before: EmotionalState,
    after: EmotionalState,
    desired: DesiredState,
    completed: boolean
  ): number {
    // Direction score: cosine similarity of emotional change
    const directionScore = this.calculateDirectionAlignment(before, after, desired);

    // Magnitude score: distance traveled
    const magnitudeScore = this.calculateMagnitude(before, after);

    // Proximity bonus: closeness to target
    const proximityBonus = this.calculateProximityBonus(after, desired);

    // Completion penalty
    const completionPenalty = completed ? 0 : -0.2;

    // Combined reward
    const reward = (
      directionScore * 0.6 +
      magnitudeScore * 0.4 +
      proximityBonus +
      completionPenalty
    );

    return Math.max(-1, Math.min(1, reward));
  }

  /**
   * Calculate direction alignment (cosine similarity)
   */
  private calculateDirectionAlignment(
    before: EmotionalState,
    after: EmotionalState,
    desired: DesiredState
  ): number {
    // Actual change vector
    const actualDelta = {
      valence: after.valence - before.valence,
      arousal: after.arousal - before.arousal,
      stress: after.stressLevel - before.stressLevel
    };

    // Desired change vector
    const desiredDelta = {
      valence: desired.targetValence - before.valence,
      arousal: desired.targetArousal - before.arousal,
      stress: desired.targetStress - before.stressLevel
    };

    // Dot product
    const dotProduct =
      actualDelta.valence * desiredDelta.valence +
      actualDelta.arousal * desiredDelta.arousal +
      actualDelta.stress * desiredDelta.stress;

    // Magnitudes
    const actualMag = Math.sqrt(
      actualDelta.valence ** 2 +
      actualDelta.arousal ** 2 +
      actualDelta.stress ** 2
    );

    const desiredMag = Math.sqrt(
      desiredDelta.valence ** 2 +
      desiredDelta.arousal ** 2 +
      desiredDelta.stress ** 2
    );

    if (actualMag === 0 || desiredMag === 0) return 0;

    const alignment = dotProduct / (actualMag * desiredMag);
    return Math.max(-1, Math.min(1, alignment));
  }

  /**
   * Calculate magnitude of emotional change
   */
  private calculateMagnitude(before: EmotionalState, after: EmotionalState): number {
    const delta = Math.sqrt(
      (after.valence - before.valence) ** 2 +
      (after.arousal - before.arousal) ** 2 +
      (after.stressLevel - before.stressLevel) ** 2
    );

    return Math.min(1, delta / 2); // Normalize to 0-1
  }

  /**
   * Calculate proximity bonus
   */
  private calculateProximityBonus(after: EmotionalState, desired: DesiredState): number {
    const distance = Math.sqrt(
      (after.valence - desired.targetValence) ** 2 +
      (after.arousal - desired.targetArousal) ** 2 +
      (after.stressLevel - desired.targetStress) ** 2
    );

    const maxDistance = Math.sqrt(2 ** 2 + 2 ** 2 + 1 ** 2);
    const normalized = 1 - (distance / maxDistance);

    return Math.max(0, normalized * 0.2); // Max bonus: 0.2
  }

  /**
   * Calculate average reward
   */
  private calculateAvgReward(): number {
    if (this.rewardHistory.length === 0) return 0;

    // Use exponential moving average for recent bias
    const alpha = 0.1;
    let ema = this.rewardHistory[0];

    for (let i = 1; i < this.rewardHistory.length; i++) {
      ema = alpha * this.rewardHistory[i] + (1 - alpha) * ema;
    }

    return ema;
  }

  /**
   * Calculate convergence score
   */
  private calculateConvergence(): number {
    if (this.rewardHistory.length < 3) return 0.1;

    // Look at recent reward variance
    const recentRewards = this.rewardHistory.slice(-5);
    const mean = recentRewards.reduce((sum, r) => sum + r, 0) / recentRewards.length;
    const variance = recentRewards.reduce((sum, r) => sum + (r - mean) ** 2, 0) / recentRewards.length;

    // Lower variance = higher convergence
    const convergence = 1 - Math.min(1, variance);

    // Factor in experience count
    const experienceFactor = Math.min(1, this.totalExperiences / 20);

    return convergence * experienceFactor;
  }
}
