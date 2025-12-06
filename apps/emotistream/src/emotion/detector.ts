/**
 * EmotionDetector - Main emotion analysis orchestrator
 * Analyzes text and returns emotional state with desired state prediction
 */

import { EmotionalState, DesiredState, GeminiEmotionResponse, PlutchikEmotion } from './types';
import { mapValenceArousal } from './mappers/valence-arousal';
import { generatePlutchikVector } from './mappers/plutchik';
import { calculateStress } from './mappers/stress';
import { hashState } from './state-hasher';
import { predictDesiredState } from './desired-state';

/**
 * Mock Gemini API call based on keyword detection
 * Real implementation would call Google Gemini API
 */
function mockGeminiAPI(text: string): GeminiEmotionResponse {
  const lowerText = text.toLowerCase();

  // Keyword-based emotion detection
  if (
    lowerText.includes('happy') ||
    lowerText.includes('joy') ||
    lowerText.includes('excited') ||
    lowerText.includes('great') ||
    lowerText.includes('wonderful')
  ) {
    return {
      valence: 0.8,
      arousal: 0.7,
      primaryEmotion: 'joy',
      secondaryEmotions: ['anticipation', 'trust'],
      confidence: 0.85,
      reasoning: 'Text contains positive, high-energy expressions indicating joy',
    };
  }

  if (
    lowerText.includes('sad') ||
    lowerText.includes('depressed') ||
    lowerText.includes('down') ||
    lowerText.includes('unhappy')
  ) {
    return {
      valence: -0.7,
      arousal: -0.4,
      primaryEmotion: 'sadness',
      secondaryEmotions: ['fear'],
      confidence: 0.8,
      reasoning: 'Text contains low-energy negative expressions indicating sadness',
    };
  }

  if (
    lowerText.includes('angry') ||
    lowerText.includes('frustrated') ||
    lowerText.includes('mad') ||
    lowerText.includes('annoyed')
  ) {
    return {
      valence: -0.8,
      arousal: 0.8,
      primaryEmotion: 'anger',
      secondaryEmotions: ['disgust'],
      confidence: 0.9,
      reasoning: 'Text contains high-energy negative expressions indicating anger',
    };
  }

  if (
    lowerText.includes('stressed') ||
    lowerText.includes('anxious') ||
    lowerText.includes('worried') ||
    lowerText.includes('nervous')
  ) {
    return {
      valence: -0.6,
      arousal: 0.7,
      primaryEmotion: 'fear',
      secondaryEmotions: ['anticipation', 'sadness'],
      confidence: 0.85,
      reasoning: 'Text contains anxious, high-arousal expressions indicating fear/stress',
    };
  }

  if (
    lowerText.includes('calm') ||
    lowerText.includes('relaxed') ||
    lowerText.includes('peaceful') ||
    lowerText.includes('serene')
  ) {
    return {
      valence: 0.6,
      arousal: -0.5,
      primaryEmotion: 'trust',
      secondaryEmotions: ['joy'],
      confidence: 0.8,
      reasoning: 'Text contains calm, low-arousal positive expressions indicating trust/peace',
    };
  }

  if (
    lowerText.includes('tired') ||
    lowerText.includes('exhausted') ||
    lowerText.includes('drained')
  ) {
    return {
      valence: -0.4,
      arousal: -0.7,
      primaryEmotion: 'sadness',
      secondaryEmotions: [],
      confidence: 0.75,
      reasoning: 'Text contains low-energy fatigue expressions',
    };
  }

  if (
    lowerText.includes('surprise') ||
    lowerText.includes('shocked') ||
    lowerText.includes('wow')
  ) {
    return {
      valence: 0.3,
      arousal: 0.8,
      primaryEmotion: 'surprise',
      secondaryEmotions: ['anticipation'],
      confidence: 0.8,
      reasoning: 'Text contains surprising, high-arousal expressions',
    };
  }

  // Default neutral state
  return {
    valence: 0.0,
    arousal: 0.0,
    primaryEmotion: 'trust',
    secondaryEmotions: [],
    confidence: 0.6,
    reasoning: 'Neutral text without strong emotional indicators',
  };
}

/**
 * Main emotion detection class
 */
export class EmotionDetector {
  /**
   * Analyze text and return emotional state with desired state prediction
   * @param text - Input text to analyze (3-5000 characters recommended)
   * @returns Complete emotional state and desired state
   */
  async analyzeText(text: string): Promise<{
    currentState: EmotionalState;
    desiredState: DesiredState;
    stateHash: string;
  }> {
    // Validate input
    if (!text || text.trim().length === 0) {
      throw new Error('Input text cannot be empty');
    }

    if (text.length < 3) {
      throw new Error('Input text too short (minimum 3 characters)');
    }

    if (text.length > 5000) {
      throw new Error('Input text too long (maximum 5000 characters)');
    }

    // Call mock Gemini API
    const geminiResponse = mockGeminiAPI(text);

    // Map to valence-arousal space
    const { valence, arousal } = mapValenceArousal(geminiResponse);

    // Generate Plutchik 8D emotion vector
    const emotionVector = generatePlutchikVector(
      geminiResponse.primaryEmotion,
      valence,
      arousal
    );

    // Calculate stress level
    const stressLevel = calculateStress(valence, arousal);

    // Create emotional state
    const currentState: EmotionalState = {
      valence,
      arousal,
      stressLevel,
      primaryEmotion: geminiResponse.primaryEmotion,
      emotionVector,
      confidence: geminiResponse.confidence,
      timestamp: Date.now(),
    };

    // Predict desired state
    const desiredState = predictDesiredState(currentState);

    // Generate state hash for Q-learning
    const stateHash = hashState(currentState);

    return {
      currentState,
      desiredState,
      stateHash,
    };
  }
}
