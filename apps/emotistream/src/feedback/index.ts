/**
 * Feedback Module - Public Exports
 * EmotiStream MVP
 */

// Main processor
export { FeedbackProcessor } from './processor';

// Components
export { RewardCalculator } from './reward-calculator';
export { ExperienceStore } from './experience-store';
export { UserProfileManager } from './user-profile';

// Types
export type {
  FeedbackRequest,
  FeedbackResponse,
  LearningProgress,
  EmotionalExperience,
  RewardComponents,
  UserStats,
} from './types';
