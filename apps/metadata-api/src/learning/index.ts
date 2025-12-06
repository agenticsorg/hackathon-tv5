/**
 * AgentDB Learning Integration for Nexus-UMMID Metadata API
 *
 * Provides pattern learning and intelligent enrichment suggestions
 * using AgentDB with ReasoningBank memory integration.
 *
 * @module learning
 */

export {
  AgentDBClient,
  AgentDBClientConfig,
  EnrichmentPattern,
  PatternMatch
} from './agentdb-client';

export {
  PatternLearner,
  EnrichmentSuggestion,
  LearningConfig
} from './pattern-learner';

// Re-export for convenience
export { default as AgentDB } from './agentdb-client';
export { default as Learner } from './pattern-learner';
