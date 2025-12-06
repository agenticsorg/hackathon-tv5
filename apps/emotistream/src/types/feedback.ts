/**
 * Feedback and Progress Tracking Types
 *
 * Types for feedback collection, watch tracking, and learning progress analytics.
 */

import { EmotionalState } from './index.js';

/**
 * Watch tracking session
 */
export interface WatchSession {
  sessionId: string;
  userId: string;
  contentId: string;
  contentTitle: string;
  startTime: Date;
  endTime?: Date;
  duration: number; // milliseconds
  completed: boolean;
  paused: boolean;
  pauseCount: number;
}

/**
 * Emotion comparison (before/after)
 */
export interface EmotionComparison {
  before: EmotionalState;
  after: EmotionalState;
  delta: {
    valence: number;
    arousal: number;
    stress: number;
  };
  improvement: number; // 0-1 score, higher is better
}

/**
 * User feedback submission
 */
export interface FeedbackSubmission {
  userId: string;
  contentId: string;
  contentTitle: string;
  sessionId: string;

  // Emotions
  emotionBefore: EmotionalState;
  emotionAfter: EmotionalState;

  // Ratings
  starRating: number; // 1-5
  completed: boolean;

  // Watch data
  watchDuration: number; // milliseconds
  totalDuration: number; // content length in milliseconds

  // Calculated
  reward?: number;
  timestamp: Date;
}

/**
 * Stored feedback record
 */
export interface FeedbackRecord extends FeedbackSubmission {
  feedbackId: string;
  reward: number;
  qValueBefore: number;
  qValueAfter: number;
  processed: boolean;
}

/**
 * Reward calculation result
 */
export interface RewardCalculation {
  reward: number; // -1 to 1
  components: {
    emotionalAlignment: number; // How well emotion moved toward goal
    completionBonus: number; // Bonus for completing content
    ratingBonus: number; // Bonus from star rating
  };
  explanation: string;
}

/**
 * Learning progress metrics
 */
export interface LearningProgress {
  userId: string;

  // Experience counts
  totalExperiences: number;
  completedContent: number;

  // Reward statistics
  averageReward: number;
  rewardTrend: 'improving' | 'stable' | 'declining';
  recentRewards: number[]; // Last 10 rewards

  // Exploration metrics
  explorationRate: number; // Current epsilon value
  explorationCount: number; // Times explored vs exploited
  exploitationCount: number;

  // Convergence
  convergenceScore: number; // 0-100, how well policy has converged
  convergenceStage: 'exploring' | 'learning' | 'confident';

  // Emotional journey
  emotionalJourney: EmotionalJourneyPoint[];

  // Performance
  bestContent: ContentPerformance[];
  worstContent: ContentPerformance[];

  timestamp: Date;
}

/**
 * Point in emotional journey
 */
export interface EmotionalJourneyPoint {
  experienceNumber: number;
  timestamp: Date;
  contentId: string;
  contentTitle: string;

  emotionBefore: EmotionalState;
  emotionAfter: EmotionalState;

  reward: number;
  completed: boolean;
}

/**
 * Content performance record
 */
export interface ContentPerformance {
  contentId: string;
  contentTitle: string;
  timesWatched: number;
  averageReward: number;
  completionRate: number;
  averageRating: number;
  lastWatched: Date;
}

/**
 * Reward timeline data point
 */
export interface RewardTimelinePoint {
  experienceNumber: number;
  timestamp: Date;
  reward: number;
  contentTitle: string;
  contentId: string;
  completed: boolean;
}

/**
 * Convergence analysis
 */
export interface ConvergenceAnalysis {
  score: number; // 0-100
  stage: 'exploring' | 'learning' | 'confident';
  explanation: string;

  metrics: {
    qValueStability: number; // 0-1, how stable Q-values are
    rewardVariance: number; // Variance in recent rewards
    explorationRate: number; // Current epsilon
    policyChanges: number; // How often best action changes
  };

  recommendations: string[];
}

/**
 * Experience list item for recent experiences
 */
export interface ExperienceListItem {
  experienceId: string;
  experienceNumber: number;
  timestamp: Date;

  contentId: string;
  contentTitle: string;

  emotionChange: EmotionComparison;
  reward: number;
  starRating: number;
  completed: boolean;

  watchDuration: number;
  completionPercentage: number;
}
