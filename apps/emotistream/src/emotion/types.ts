/**
 * EmotionDetector Type Definitions
 * Based on ARCH-EmotionDetector.md
 */

/**
 * Plutchik's 8 basic emotions
 */
export type PlutchikEmotion =
  | 'joy'
  | 'sadness'
  | 'anger'
  | 'fear'
  | 'trust'
  | 'disgust'
  | 'surprise'
  | 'anticipation';

/**
 * Gemini API response structure (mocked for MVP)
 */
export interface GeminiEmotionResponse {
  /** Valence value from Gemini */
  valence: number;

  /** Arousal value from Gemini */
  arousal: number;

  /** Primary emotion detected */
  primaryEmotion: PlutchikEmotion;

  /** Secondary emotions with lower confidence */
  secondaryEmotions: PlutchikEmotion[];

  /** Gemini's confidence in this analysis */
  confidence: number;

  /** Gemini's explanation */
  reasoning: string;
}

/**
 * Emotional state derived from text analysis
 */
export interface EmotionalState {
  /** Valence: emotional pleasantness (-1.0 to +1.0) */
  valence: number;

  /** Arousal: emotional activation level (-1.0 to +1.0) */
  arousal: number;

  /** Stress level (0.0 to 1.0) */
  stressLevel: number;

  /** Primary emotion from Plutchik's 8 basic emotions */
  primaryEmotion: PlutchikEmotion;

  /** 8D emotion vector (normalized to sum to 1.0) */
  emotionVector: Float32Array;

  /** Confidence in this analysis (0.0 to 1.0) */
  confidence: number;

  /** Unix timestamp in milliseconds */
  timestamp: number;
}

/**
 * Desired emotional state predicted from current state
 */
export interface DesiredState {
  /** Target valence (-1.0 to +1.0) */
  targetValence: number;

  /** Target arousal (-1.0 to +1.0) */
  targetArousal: number;

  /** Target stress level (0.0 to 1.0) */
  targetStress: number;

  /** Intensity of adjustment needed */
  intensity: 'subtle' | 'moderate' | 'significant';

  /** Human-readable reasoning for this prediction */
  reasoning: string;
}
