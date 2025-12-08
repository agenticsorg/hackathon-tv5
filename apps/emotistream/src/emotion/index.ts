/**
 * EmotionDetector Module
 * Main entry point for emotion detection functionality
 */

// Export main class
export { EmotionDetector } from './detector';

// Export types
export type {
  EmotionalState,
  DesiredState,
  GeminiEmotionResponse,
  PlutchikEmotion,
} from './types';

// Export mappers
export { mapValenceArousal } from './mappers/valence-arousal';
export { generatePlutchikVector } from './mappers/plutchik';
export { calculateStress } from './mappers/stress';

// Export utilities
export { hashState } from './state-hasher';
export { predictDesiredState } from './desired-state';
