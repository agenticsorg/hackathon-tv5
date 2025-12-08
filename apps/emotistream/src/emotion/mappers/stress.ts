/**
 * Stress Level Calculator
 * Calculates stress from valence and arousal coordinates
 */

/**
 * Quadrant weights for stress calculation
 * Q1 (positive valence, high arousal): Low stress (excitement)
 * Q2 (negative valence, high arousal): High stress (anxiety, anger)
 * Q3 (negative valence, low arousal): Moderate stress (depression)
 * Q4 (positive valence, low arousal): Very low stress (calm)
 */
const QUADRANT_WEIGHTS = {
  Q1: 0.3, // High arousal + Positive (excited, happy)
  Q2: 0.9, // High arousal + Negative (stressed, anxious, angry)
  Q3: 0.6, // Low arousal + Negative (sad, tired)
  Q4: 0.1, // Low arousal + Positive (calm, relaxed)
};

/**
 * Get quadrant weight based on valence and arousal
 */
function getQuadrantWeight(valence: number, arousal: number): number {
  if (arousal >= 0) {
    return valence >= 0 ? QUADRANT_WEIGHTS.Q1 : QUADRANT_WEIGHTS.Q2;
  } else {
    return valence >= 0 ? QUADRANT_WEIGHTS.Q4 : QUADRANT_WEIGHTS.Q3;
  }
}

/**
 * Calculate emotional intensity (distance from origin)
 */
function calculateEmotionalIntensity(valence: number, arousal: number): number {
  return Math.sqrt(valence ** 2 + arousal ** 2) / Math.sqrt(2);
}

/**
 * Apply negative valence boost to stress
 * Extreme negative valence increases stress more
 */
function applyNegativeBoost(stress: number, valence: number): number {
  if (valence < 0) {
    const negativeIntensity = Math.abs(valence);
    const boost = negativeIntensity * 0.2; // Up to 20% boost
    return Math.min(1.0, stress + boost);
  }
  return stress;
}

/**
 * Calculate stress level from valence and arousal
 * @param valence - Valence value (-1.0 to +1.0)
 * @param arousal - Arousal value (-1.0 to +1.0)
 * @returns Stress level (0.0 to 1.0)
 */
export function calculateStress(valence: number, arousal: number): number {
  // Get base stress from quadrant
  const quadrantWeight = getQuadrantWeight(valence, arousal);

  // Calculate emotional intensity
  const intensity = calculateEmotionalIntensity(valence, arousal);

  // Base stress = quadrant weight * intensity
  let stress = quadrantWeight * intensity;

  // Apply negative valence boost
  stress = applyNegativeBoost(stress, valence);

  // Ensure stress is in [0, 1] range
  stress = Math.max(0.0, Math.min(1.0, stress));

  return Number(stress.toFixed(3));
}
