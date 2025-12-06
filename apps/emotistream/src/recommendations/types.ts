/**
 * RecommendationEngine Type Definitions
 * EmotiStream Nexus - MVP Phase 5
 */

import { EmotionalContentProfile, ContentMetadata } from '../content/types';
import { EmotionalState as RLEmotionalState, DesiredState as RLDesiredState } from '../rl/types';

/**
 * Re-export RL types for convenience
 */
export type EmotionalState = RLEmotionalState;
export type DesiredState = RLDesiredState;

/**
 * Recommendation request from client
 */
export interface RecommendationRequest {
  userId: string;
  currentState: EmotionalState;
  desiredState?: DesiredState;
  limit?: number;
  includeExploration?: boolean;
  explorationRate?: number;
}

/**
 * Final recommendation output
 */
export interface Recommendation {
  contentId: string;
  title: string;
  qValue: number;
  similarityScore: number;
  combinedScore: number;
  predictedOutcome: PredictedOutcome;
  reasoning: string;
  isExploration: boolean;
  rank: number;
  profile: EmotionalContentProfile;
}

/**
 * Predicted emotional outcome after viewing
 */
export interface PredictedOutcome {
  expectedValence: number;
  expectedArousal: number;
  expectedStress: number;
  confidence: number;
}

/**
 * Candidate content from search
 */
export interface CandidateContent {
  contentId: string;
  title: string;
  emotionalVector: Float32Array;
  transitionVector: Float32Array;
  profile: EmotionalContentProfile;
  similarityScore: number;
}

/**
 * Ranked content after hybrid scoring
 */
export interface RankedContent {
  contentId: string;
  title: string;
  profile: EmotionalContentProfile;
  qValue: number;
  similarityScore: number;
  combinedScore: number;
  outcomeAlignment: number;
  isExploration: boolean;
}

/**
 * State hash for Q-table lookup
 */
export interface StateHash {
  valenceBucket: number;
  arousalBucket: number;
  stressBucket: number;
  hash: string;
}

/**
 * Hybrid ranking configuration
 */
export interface HybridRankingConfig {
  qWeight: number;
  similarityWeight: number;
  defaultQValue: number;
  explorationBonus: number;
}
