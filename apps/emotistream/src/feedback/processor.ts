/**
 * Feedback Processor
 * EmotiStream MVP
 *
 * Processes user feedback and updates RL policy
 */

import type { EmotionalState, DesiredState } from '../emotion/types';
import type {
  FeedbackRequest,
  FeedbackResponse,
  EmotionalExperience,
  LearningProgress,
} from './types';
import { RewardCalculator } from './reward-calculator';
import { ExperienceStore } from './experience-store';
import { UserProfileManager } from './user-profile';

export class FeedbackProcessor {
  private rewardCalculator: RewardCalculator;
  private experienceStore: ExperienceStore;
  private profileManager: UserProfileManager;

  constructor() {
    this.rewardCalculator = new RewardCalculator();
    this.experienceStore = new ExperienceStore();
    this.profileManager = new UserProfileManager();
  }

  /**
   * Process user feedback and calculate reward
   */
  process(
    request: FeedbackRequest,
    stateBefore: EmotionalState,
    desiredState: DesiredState
  ): FeedbackResponse {
    // Step 1: Calculate reward components
    const components = this.rewardCalculator.calculateComponents(
      stateBefore,
      request.actualPostState,
      desiredState
    );

    // Step 2: Apply completion penalty if not completed
    let finalReward = components.totalReward;
    if (!request.completed) {
      const penalty = this.rewardCalculator.calculateCompletionPenalty(
        request.completed,
        request.watchDuration,
        30 // Assume 30min average content duration for MVP
      );
      finalReward = Math.max(-1, Math.min(1, finalReward + penalty));
    }

    // Step 3: Store experience for replay learning
    const experience: EmotionalExperience = {
      userId: request.userId,
      timestamp: Date.now(),
      stateBefore,
      action: request.contentId,
      stateAfter: request.actualPostState,
      reward: finalReward,
      desiredState,
    };
    this.experienceStore.store(experience);

    // Step 4: Update user profile
    this.profileManager.update(request.userId, finalReward);

    // Step 5: Get learning progress
    const learningProgress = this.profileManager.getStats(request.userId);

    // Note: This processor only calculates the reward. The actual Q-value update
    // happens in RLPolicyEngine.updatePolicy() which is called by the feedback route.
    return {
      reward: finalReward,
      policyUpdated: false, // Actual update happens in route handler via PolicyEngine
      newQValue: 0, // Placeholder - real value comes from PolicyEngine.updatePolicy()
      learningProgress,
    };
  }

  /**
   * Get recent experiences for a user
   */
  getRecentExperiences(userId: string, limit: number = 10): EmotionalExperience[] {
    return this.experienceStore.getRecent(userId, limit);
  }

  /**
   * Get learning progress for a user
   */
  getLearningProgress(userId: string): LearningProgress {
    return this.profileManager.getStats(userId);
  }

  /**
   * Get average reward for a user
   */
  getAverageReward(userId: string): number {
    return this.experienceStore.getAverageReward(userId);
  }

  /**
   * Clear user data (for testing)
   */
  clearUser(userId: string): void {
    this.experienceStore.clear(userId);
    this.profileManager.clear(userId);
  }

  /**
   * Clear all data (for testing)
   */
  clearAll(): void {
    this.experienceStore.clearAll();
    this.profileManager.clearAll();
  }
}
