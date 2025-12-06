/**
 * User Profile Manager
 * EmotiStream MVP
 *
 * Tracks user learning progress and statistics
 */

import type { UserStats, LearningProgress } from './types';

export class UserProfileManager {
  private profiles: Map<string, UserStats> = new Map();
  private readonly INITIAL_EXPLORATION_RATE = 0.3; // 30% exploration
  private readonly MIN_EXPLORATION_RATE = 0.05; // 5% minimum
  private readonly EXPLORATION_DECAY = 0.995; // Decay per experience

  /**
   * Update user profile after feedback
   */
  update(userId: string, reward: number): void {
    let profile = this.profiles.get(userId);

    if (!profile) {
      // Initialize new profile
      profile = {
        userId,
        totalExperiences: 0,
        avgReward: 0,
        explorationRate: this.INITIAL_EXPLORATION_RATE,
        lastUpdated: Date.now(),
      };
    }

    // Update experience count
    profile.totalExperiences += 1;

    // Update average reward using exponential moving average
    const alpha = 0.1; // Smoothing factor
    profile.avgReward = alpha * reward + (1 - alpha) * profile.avgReward;

    // Decay exploration rate (explore less as we learn more)
    profile.explorationRate = Math.max(
      this.MIN_EXPLORATION_RATE,
      profile.explorationRate * this.EXPLORATION_DECAY
    );

    // Update timestamp
    profile.lastUpdated = Date.now();

    // Save profile
    this.profiles.set(userId, profile);
  }

  /**
   * Get user statistics
   */
  getStats(userId: string): LearningProgress {
    const profile = this.profiles.get(userId);

    if (!profile) {
      // Return default stats for new users
      return {
        totalExperiences: 0,
        avgReward: 0,
        explorationRate: this.INITIAL_EXPLORATION_RATE,
        convergenceScore: 0,
      };
    }

    // Calculate convergence score (0-1)
    // Higher score = better learned policy
    const convergenceScore = this.calculateConvergenceScore(profile);

    return {
      totalExperiences: profile.totalExperiences,
      avgReward: profile.avgReward,
      explorationRate: profile.explorationRate,
      convergenceScore,
    };
  }

  /**
   * Calculate convergence score based on learning progress
   * Returns value in range [0, 1]
   */
  private calculateConvergenceScore(profile: UserStats): number {
    // Component 1: Experience count (saturates at 100 experiences)
    const experienceScore = Math.min(1, profile.totalExperiences / 100);

    // Component 2: Average reward (normalized from [-1, 1] to [0, 1])
    const rewardScore = (profile.avgReward + 1) / 2;

    // Component 3: Exploration rate (inverse - lower exploration = higher convergence)
    const explorationScore = 1 - (profile.explorationRate - this.MIN_EXPLORATION_RATE) /
      (this.INITIAL_EXPLORATION_RATE - this.MIN_EXPLORATION_RATE);

    // Weighted average
    const convergence =
      0.4 * experienceScore +
      0.4 * rewardScore +
      0.2 * explorationScore;

    return Math.max(0, Math.min(1, convergence));
  }

  /**
   * Get exploration rate for a user
   */
  getExplorationRate(userId: string): number {
    const profile = this.profiles.get(userId);
    return profile ? profile.explorationRate : this.INITIAL_EXPLORATION_RATE;
  }

  /**
   * Clear user profile (for testing)
   */
  clear(userId: string): void {
    this.profiles.delete(userId);
  }

  /**
   * Clear all profiles (for testing)
   */
  clearAll(): void {
    this.profiles.clear();
  }
}
