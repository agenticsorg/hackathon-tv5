/**
 * Experience Store - In-Memory Storage
 * EmotiStream MVP
 *
 * Stores emotional experiences for learning and analytics
 */

import type { EmotionalExperience } from './types';

export class ExperienceStore {
  private experiences: Map<string, EmotionalExperience[]> = new Map();
  private readonly MAX_EXPERIENCES_PER_USER = 1000;

  /**
   * Store an emotional experience
   */
  store(experience: EmotionalExperience): void {
    const userId = experience.userId;

    // Get existing experiences for user
    let userExperiences = this.experiences.get(userId);

    if (!userExperiences) {
      userExperiences = [];
      this.experiences.set(userId, userExperiences);
    }

    // Add new experience
    userExperiences.push(experience);

    // Maintain size limit (FIFO)
    if (userExperiences.length > this.MAX_EXPERIENCES_PER_USER) {
      userExperiences.shift(); // Remove oldest
    }
  }

  /**
   * Get recent experiences for a user
   */
  getRecent(userId: string, limit: number = 10): EmotionalExperience[] {
    const userExperiences = this.experiences.get(userId);

    if (!userExperiences || userExperiences.length === 0) {
      return [];
    }

    // Return most recent experiences (from end of array)
    const start = Math.max(0, userExperiences.length - limit);
    return userExperiences.slice(start);
  }

  /**
   * Get all experiences for a user
   */
  getAll(userId: string): EmotionalExperience[] {
    return this.experiences.get(userId) || [];
  }

  /**
   * Get total number of experiences for a user
   */
  getCount(userId: string): number {
    const userExperiences = this.experiences.get(userId);
    return userExperiences ? userExperiences.length : 0;
  }

  /**
   * Get average reward for a user
   */
  getAverageReward(userId: string): number {
    const userExperiences = this.experiences.get(userId);

    if (!userExperiences || userExperiences.length === 0) {
      return 0;
    }

    const totalReward = userExperiences.reduce((sum, exp) => sum + exp.reward, 0);
    return totalReward / userExperiences.length;
  }

  /**
   * Clear all experiences for a user
   */
  clear(userId: string): void {
    this.experiences.delete(userId);
  }

  /**
   * Clear all experiences (for testing)
   */
  clearAll(): void {
    this.experiences.clear();
  }
}
