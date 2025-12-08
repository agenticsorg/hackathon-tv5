/**
 * Plutchik Emotion Vector Mapper
 * Generates 8D emotion vectors based on Plutchik's Wheel of Emotions
 */

import { PlutchikEmotion } from '../types';

/**
 * Plutchik's 8 basic emotions in wheel order
 */
const PLUTCHIK_EMOTIONS: PlutchikEmotion[] = [
  'joy',
  'trust',
  'fear',
  'surprise',
  'sadness',
  'disgust',
  'anger',
  'anticipation',
];

/**
 * Opposite emotion pairs in Plutchik's wheel
 */
const OPPOSITE_PAIRS: Record<PlutchikEmotion, PlutchikEmotion> = {
  joy: 'sadness',
  sadness: 'joy',
  trust: 'disgust',
  disgust: 'trust',
  fear: 'anger',
  anger: 'fear',
  surprise: 'anticipation',
  anticipation: 'surprise',
};

/**
 * Get index of emotion in the wheel
 */
function getEmotionIndex(emotion: PlutchikEmotion): number {
  const index = PLUTCHIK_EMOTIONS.indexOf(emotion);
  return index !== -1 ? index : 0; // Default to joy if not found
}

/**
 * Get adjacent emotions (neighbors in the wheel)
 */
function getAdjacentEmotions(emotion: PlutchikEmotion): PlutchikEmotion[] {
  const index = getEmotionIndex(emotion);
  const leftIndex = (index - 1 + 8) % 8;
  const rightIndex = (index + 1) % 8;

  return [PLUTCHIK_EMOTIONS[leftIndex], PLUTCHIK_EMOTIONS[rightIndex]];
}

/**
 * Generate normalized 8D emotion vector
 * @param primaryEmotion - Primary emotion
 * @param valence - Valence value (-1 to +1)
 * @param arousal - Arousal value (-1 to +1)
 * @returns Normalized 8D emotion vector (sum = 1.0)
 */
export function generatePlutchikVector(
  primaryEmotion: PlutchikEmotion,
  valence: number,
  arousal: number
): Float32Array {
  const vector = new Float32Array(8);

  // Calculate intensity from valence/arousal magnitude
  const intensity = Math.sqrt(valence ** 2 + arousal ** 2) / Math.sqrt(2);

  // Primary emotion gets highest weight (0.5 to 0.8 based on intensity)
  const primaryIndex = getEmotionIndex(primaryEmotion);
  const primaryWeight = 0.5 + intensity * 0.3;
  vector[primaryIndex] = primaryWeight;

  // Adjacent emotions get moderate weight (0.1 to 0.2 based on intensity)
  const adjacentEmotions = getAdjacentEmotions(primaryEmotion);
  const adjacentWeight = 0.1 + intensity * 0.1;

  adjacentEmotions.forEach((emotion) => {
    const index = getEmotionIndex(emotion);
    vector[index] = adjacentWeight;
  });

  // Opposite emotion gets zero or very low weight
  const oppositeEmotion = OPPOSITE_PAIRS[primaryEmotion];
  const oppositeIndex = getEmotionIndex(oppositeEmotion);
  vector[oppositeIndex] = 0.0;

  // Remaining emotions get small residual weight
  const residualWeight = (1.0 - primaryWeight - 2 * adjacentWeight) / 4;
  for (let i = 0; i < 8; i++) {
    if (vector[i] === 0) {
      vector[i] = Math.max(0, residualWeight);
    }
  }

  // Normalize to sum to 1.0
  const sum = Array.from(vector).reduce((a, b) => a + b, 0);
  if (sum > 0) {
    for (let i = 0; i < 8; i++) {
      vector[i] = vector[i] / sum;
    }
  }

  return vector;
}
