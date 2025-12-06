/**
 * RecommendationEngine Module Exports
 * EmotiStream Nexus - MVP Phase 5
 */

export { RecommendationEngine } from './engine';
export { HybridRanker } from './ranker';
export { OutcomePredictor } from './outcome-predictor';
export { ReasoningGenerator } from './reasoning';
export { ExplorationStrategy } from './exploration';
export { StateHasher } from './state-hasher';

export type {
  Recommendation,
  RecommendationRequest,
  PredictedOutcome,
  CandidateContent,
  RankedContent,
  StateHash,
  HybridRankingConfig,
  EmotionalState,
  DesiredState
} from './types';
