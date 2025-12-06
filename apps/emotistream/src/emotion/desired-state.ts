/**
 * Desired State Predictor
 * Predicts desired emotional state using rule-based heuristics
 */

import { EmotionalState, DesiredState } from './types';

/**
 * Thresholds for heuristic rules
 */
const STRESS_THRESHOLD = 0.6;
const LOW_MOOD_THRESHOLD = -0.3;
const HIGH_AROUSAL_THRESHOLD = 0.5;
const LOW_AROUSAL_THRESHOLD = -0.3;

/**
 * Rule 1: High stress -> Reduce stress (calming)
 */
function applyStressRule(state: EmotionalState): DesiredState | null {
  if (state.stressLevel > STRESS_THRESHOLD) {
    return {
      targetValence: 0.5, // Mildly positive
      targetArousal: -0.4, // Calming (low arousal)
      targetStress: 0.3, // Reduce stress significantly
      intensity: state.stressLevel > 0.8 ? 'significant' : 'moderate',
      reasoning:
        'User is experiencing high stress. Recommend calming, low-arousal content to reduce stress levels.',
    };
  }
  return null;
}

/**
 * Rule 2: Negative valence -> Improve mood (uplifting)
 */
function applyLowMoodRule(state: EmotionalState): DesiredState | null {
  if (state.valence < LOW_MOOD_THRESHOLD) {
    return {
      targetValence: 0.6, // Positive mood
      targetArousal: 0.3, // Moderately energizing
      targetStress: Math.max(0.2, state.stressLevel - 0.2), // Slight stress reduction
      intensity: state.valence < -0.6 ? 'significant' : 'moderate',
      reasoning:
        'User is experiencing low mood. Recommend uplifting, moderately energizing content to improve emotional state.',
    };
  }
  return null;
}

/**
 * Rule 3: High arousal + negative valence -> Calm down (anxiety/anger reduction)
 */
function applyAnxiousRule(state: EmotionalState): DesiredState | null {
  if (state.arousal > HIGH_AROUSAL_THRESHOLD && state.valence < 0) {
    return {
      targetValence: 0.4, // Mildly positive
      targetArousal: -0.5, // Significantly calming
      targetStress: 0.2, // Low stress
      intensity: 'significant',
      reasoning:
        'User is experiencing anxiety or anger (high arousal + negative valence). Recommend calming content to reduce arousal and improve mood.',
    };
  }
  return null;
}

/**
 * Rule 4: Low arousal -> Increase engagement (energizing)
 */
function applyLowEnergyRule(state: EmotionalState): DesiredState | null {
  if (state.arousal < LOW_AROUSAL_THRESHOLD && state.valence > -0.2) {
    return {
      targetValence: 0.7, // Positive
      targetArousal: 0.5, // Energizing
      targetStress: 0.3, // Low stress
      intensity: 'moderate',
      reasoning:
        'User has low energy. Recommend energizing, engaging content to increase arousal while maintaining positive mood.',
    };
  }
  return null;
}

/**
 * Rule 5: Default -> Maintain with slight improvement
 */
function getDefaultDesiredState(state: EmotionalState): DesiredState {
  // Slightly improve current state
  const targetValence = Math.min(1.0, state.valence + 0.2);
  const targetArousal = state.arousal; // Keep arousal similar
  const targetStress = Math.max(0.0, state.stressLevel - 0.1);

  return {
    targetValence,
    targetArousal,
    targetStress,
    intensity: 'subtle',
    reasoning:
      'User is in a relatively balanced state. Recommend content that maintains current mood with slight positive enhancement.',
  };
}

/**
 * Predict desired emotional state from current state
 * Applies heuristic rules in priority order
 * @param currentState - Current emotional state
 * @returns Predicted desired state
 */
export function predictDesiredState(currentState: EmotionalState): DesiredState {
  // Apply rules in priority order
  let desiredState: DesiredState | null = null;

  // Priority 1: High stress (most urgent)
  desiredState = applyStressRule(currentState);
  if (desiredState) return desiredState;

  // Priority 2: High arousal + negative (anxiety/anger)
  desiredState = applyAnxiousRule(currentState);
  if (desiredState) return desiredState;

  // Priority 3: Low mood
  desiredState = applyLowMoodRule(currentState);
  if (desiredState) return desiredState;

  // Priority 4: Low energy
  desiredState = applyLowEnergyRule(currentState);
  if (desiredState) return desiredState;

  // Default: Maintain with slight improvement
  return getDefaultDesiredState(currentState);
}
