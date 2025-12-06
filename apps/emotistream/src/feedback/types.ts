/**
 * FeedbackProcessor Module Type Definitions
 * EmotiStream MVP - Feedback and Reward Processing
 */

import type { EmotionalState, DesiredState } from '../emotion/types';

/**
 * Feedback request from user after content consumption
 */
export interface FeedbackRequest {
  userId: string;
  contentId: string;
  actualPostState: EmotionalState;
  watchDuration: number;
  completed: boolean;
  explicitRating?: number; // 1-5 star rating
}

/**
 * Feedback response with learning progress
 */
export interface FeedbackResponse {
  reward: number;
  policyUpdated: boolean;
  newQValue: number;
  learningProgress: LearningProgress;
}

/**
 * Learning progress metrics
 */
export interface LearningProgress {
  totalExperiences: number;
  avgReward: number;
  explorationRate: number;
  convergenceScore: number;
}

/**
 * Emotional experience for replay buffer
 */
export interface EmotionalExperience {
  userId: string;
  timestamp: number;
  stateBefore: EmotionalState;
  action: string; // contentId
  stateAfter: EmotionalState;
  reward: number;
  desiredState: DesiredState;
}

/**
 * Reward calculation components breakdown
 */
export interface RewardComponents {
  directionAlignment: number; // Cosine similarity [-1, 1]
  magnitude: number; // Normalized distance [0, 1]
  proximityBonus: number; // Bonus for reaching target [0, 0.1]
  completionPenalty: number; // Penalty for not completing [-0.2, 0]
  totalReward: number; // Final reward [-1, 1]
}

/**
 * User statistics for learning
 */
export interface UserStats {
  userId: string;
  totalExperiences: number;
  avgReward: number;
  explorationRate: number;
  lastUpdated: number;
}
