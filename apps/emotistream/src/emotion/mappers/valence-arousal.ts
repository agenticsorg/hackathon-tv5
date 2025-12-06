/**
 * Valence-Arousal Mapper
 * Maps Gemini response to Russell's Circumplex Model coordinates
 */

import { GeminiEmotionResponse } from '../types';

/**
 * Map Gemini response to normalized valence-arousal coordinates
 * @param response - Gemini API response
 * @returns Normalized valence and arousal values in [-1, +1]
 */
export function mapValenceArousal(response: GeminiEmotionResponse): {
  valence: number;
  arousal: number;
} {
  let { valence, arousal } = response;

  // Clamp values to [-1, +1] range
  valence = Math.max(-1, Math.min(1, valence));
  arousal = Math.max(-1, Math.min(1, arousal));

  // Normalize to unit circle if magnitude exceeds √2
  const magnitude = Math.sqrt(valence ** 2 + arousal ** 2);
  const maxMagnitude = Math.sqrt(2); // √2 ≈ 1.414

  if (magnitude > maxMagnitude) {
    const scale = maxMagnitude / magnitude;
    valence *= scale;
    arousal *= scale;
  }

  return {
    valence: Number(valence.toFixed(3)),
    arousal: Number(arousal.toFixed(3)),
  };
}
