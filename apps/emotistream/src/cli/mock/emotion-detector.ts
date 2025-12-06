/**
 * Mock Emotion Detector for CLI Demo
 *
 * Simulates emotion detection without requiring Gemini API.
 */

import { EmotionalState, DesiredState } from '../../types/index.js';
import { PostViewingFeedback } from '../prompts.js';

/**
 * Mock emotion detector using keyword-based analysis
 */
export class MockEmotionDetector {
  /**
   * Analyze text to detect emotional state
   */
  async analyze(text: string): Promise<EmotionalState> {
    const lowercaseText = text.toLowerCase();

    // Detect primary emotion based on keywords
    const emotion = this.detectPrimaryEmotion(lowercaseText);

    // Calculate valence based on positive/negative keywords
    const valence = this.calculateValence(lowercaseText);

    // Calculate arousal based on energy level
    const arousal = this.calculateArousal(lowercaseText);

    // Calculate stress level
    const stressLevel = this.calculateStress(lowercaseText);

    // Create emotion vector (Plutchik 8D)
    const emotionVector = this.createEmotionVector(emotion);

    return {
      valence,
      arousal,
      stressLevel,
      primaryEmotion: emotion,
      emotionVector,
      confidence: 0.85,
      timestamp: Date.now()
    };
  }

  /**
   * Predict desired emotional state
   */
  predictDesiredState(current: EmotionalState): DesiredState {
    // Simple heuristic: move toward positive, calm, low-stress state
    let targetValence = 0.6;
    let targetArousal = -0.3;
    let targetStress = 0.2;
    let intensity: 'subtle' | 'moderate' | 'significant' = 'moderate';
    let reasoning = '';

    // If very negative, aim for positive but not too much change
    if (current.valence < -0.5) {
      targetValence = 0.3;
      intensity = 'moderate';
      reasoning = 'Gradual shift from negative to positive emotional state';
    }

    // If stressed, prioritize stress reduction
    if (current.stressLevel > 0.6) {
      targetStress = 0.2;
      targetArousal = -0.4;
      intensity = 'significant';
      reasoning = 'Focus on stress reduction and calming';
    }

    // If very aroused, aim for calm
    if (current.arousal > 0.5) {
      targetArousal = -0.3;
      reasoning = 'Reduce excitement, promote relaxation';
    }

    // If already positive and calm, maintain
    if (current.valence > 0.4 && current.arousal < 0 && current.stressLevel < 0.4) {
      targetValence = current.valence;
      targetArousal = current.arousal;
      targetStress = current.stressLevel;
      intensity = 'subtle';
      reasoning = 'Maintain current positive state';
    }

    return {
      targetValence,
      targetArousal,
      targetStress,
      intensity,
      reasoning: reasoning || 'Move toward balanced, positive emotional state'
    };
  }

  /**
   * Analyze post-viewing feedback
   */
  analyzePostViewing(feedback: PostViewingFeedback): EmotionalState {
    if (feedback.text) {
      return this.analyze(feedback.text) as any; // Sync version for demo
    } else if (feedback.rating !== undefined) {
      return this.convertRatingToState(feedback.rating);
    } else if (feedback.emoji) {
      return this.convertEmojiToState(feedback.emoji);
    }

    // Default neutral state
    return this.createNeutralState();
  }

  /**
   * Detect primary emotion from text
   */
  private detectPrimaryEmotion(text: string): string {
    const emotionKeywords: Record<string, string[]> = {
      sadness: ['sad', 'depressed', 'down', 'lonely', 'empty', 'hopeless', 'crying'],
      joy: ['happy', 'joyful', 'great', 'wonderful', 'excited', 'pleased', 'delighted'],
      anger: ['angry', 'mad', 'furious', 'annoyed', 'frustrated', 'irritated'],
      fear: ['scared', 'afraid', 'anxious', 'worried', 'nervous', 'terrified'],
      stress: ['stressed', 'overwhelmed', 'pressure', 'tense', 'burden'],
      relaxation: ['relaxed', 'calm', 'peaceful', 'serene', 'tranquil'],
      contentment: ['content', 'satisfied', 'comfortable', 'better', 'good']
    };

    let maxMatches = 0;
    let detectedEmotion = 'neutral';

    for (const [emotion, keywords] of Object.entries(emotionKeywords)) {
      const matches = keywords.filter(keyword => text.includes(keyword)).length;
      if (matches > maxMatches) {
        maxMatches = matches;
        detectedEmotion = emotion;
      }
    }

    return detectedEmotion;
  }

  /**
   * Calculate valence (-1 to 1)
   */
  private calculateValence(text: string): number {
    const positiveWords = ['happy', 'good', 'great', 'wonderful', 'better', 'relaxed', 'calm', 'content'];
    const negativeWords = ['sad', 'bad', 'stressed', 'angry', 'worried', 'anxious', 'overwhelmed', 'lonely'];

    const positiveCount = positiveWords.filter(word => text.includes(word)).length;
    const negativeCount = negativeWords.filter(word => text.includes(word)).length;

    const total = positiveCount + negativeCount;
    if (total === 0) return 0;

    const valence = (positiveCount - negativeCount) / total;
    return Math.max(-1, Math.min(1, valence));
  }

  /**
   * Calculate arousal (-1 to 1)
   */
  private calculateArousal(text: string): number {
    const highArousalWords = ['excited', 'anxious', 'stressed', 'angry', 'energetic', 'overwhelmed'];
    const lowArousalWords = ['calm', 'relaxed', 'tired', 'peaceful', 'sleepy', 'bored'];

    const highCount = highArousalWords.filter(word => text.includes(word)).length;
    const lowCount = lowArousalWords.filter(word => text.includes(word)).length;

    const total = highCount + lowCount;
    if (total === 0) return 0;

    const arousal = (highCount - lowCount) / total;
    return Math.max(-1, Math.min(1, arousal));
  }

  /**
   * Calculate stress level (0 to 1)
   */
  private calculateStress(text: string): number {
    const stressWords = ['stressed', 'overwhelmed', 'pressure', 'anxious', 'worried', 'tense', 'burden'];
    const count = stressWords.filter(word => text.includes(word)).length;

    return Math.min(1, count * 0.25);
  }

  /**
   * Create emotion vector (Plutchik 8D)
   */
  private createEmotionVector(emotion: string): Float32Array {
    const vector = new Float32Array(8);

    const emotionMap: Record<string, number[]> = {
      joy: [1, 0.5, 0, 0, 0, 0, 0, 0.3],
      trust: [0.3, 1, 0, 0, 0, 0, 0, 0.5],
      fear: [0, 0, 1, 0.3, 0, 0, 0, 0],
      surprise: [0.3, 0, 0.5, 1, 0, 0, 0, 0.5],
      sadness: [0, 0, 0, 0, 1, 0.3, 0, 0],
      disgust: [0, 0, 0, 0, 0.3, 1, 0, 0],
      anger: [0, 0, 0, 0, 0, 0, 1, 0],
      anticipation: [0.5, 0.3, 0, 0.3, 0, 0, 0, 1],
      neutral: [0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2],
      stress: [0, 0, 0.7, 0.3, 0.3, 0, 0.5, 0],
      relaxation: [0.6, 0.5, 0, 0, 0, 0, 0, 0.3],
      contentment: [0.7, 0.6, 0, 0, 0, 0, 0, 0.2]
    };

    const values = emotionMap[emotion] || emotionMap.neutral;
    for (let i = 0; i < 8; i++) {
      vector[i] = values[i];
    }

    return vector;
  }

  /**
   * Convert rating to emotional state
   */
  private convertRatingToState(rating: number): EmotionalState {
    const mappings: Record<number, { valence: number; arousal: number; stress: number; emotion: string }> = {
      1: { valence: -0.8, arousal: 0.3, stress: 0.7, emotion: 'sadness' },
      2: { valence: -0.4, arousal: 0.1, stress: 0.5, emotion: 'sadness' },
      3: { valence: 0.0, arousal: 0.0, stress: 0.3, emotion: 'neutral' },
      4: { valence: 0.5, arousal: -0.1, stress: 0.2, emotion: 'contentment' },
      5: { valence: 0.8, arousal: -0.2, stress: 0.1, emotion: 'joy' }
    };

    const mapping = mappings[rating] || mappings[3];

    return {
      valence: mapping.valence,
      arousal: mapping.arousal,
      stressLevel: mapping.stress,
      primaryEmotion: mapping.emotion,
      emotionVector: this.createEmotionVector(mapping.emotion),
      confidence: 0.6,
      timestamp: Date.now()
    };
  }

  /**
   * Convert emoji to emotional state
   */
  private convertEmojiToState(emoji: string): EmotionalState {
    const mappings: Record<string, { valence: number; arousal: number; stress: number; emotion: string }> = {
      '😊': { valence: 0.7, arousal: -0.2, stress: 0.2, emotion: 'joy' },
      '😌': { valence: 0.5, arousal: -0.6, stress: 0.1, emotion: 'relaxation' },
      '😐': { valence: 0.0, arousal: 0.0, stress: 0.3, emotion: 'neutral' },
      '😢': { valence: -0.6, arousal: -0.3, stress: 0.5, emotion: 'sadness' },
      '😡': { valence: -0.7, arousal: 0.8, stress: 0.8, emotion: 'anger' },
      '😴': { valence: 0.2, arousal: -0.8, stress: 0.2, emotion: 'relaxation' }
    };

    const mapping = mappings[emoji] || mappings['😐'];

    return {
      valence: mapping.valence,
      arousal: mapping.arousal,
      stressLevel: mapping.stress,
      primaryEmotion: mapping.emotion,
      emotionVector: this.createEmotionVector(mapping.emotion),
      confidence: 0.5,
      timestamp: Date.now()
    };
  }

  /**
   * Create neutral emotional state
   */
  private createNeutralState(): EmotionalState {
    return {
      valence: 0,
      arousal: 0,
      stressLevel: 0.3,
      primaryEmotion: 'neutral',
      emotionVector: this.createEmotionVector('neutral'),
      confidence: 0.5,
      timestamp: Date.now()
    };
  }
}
