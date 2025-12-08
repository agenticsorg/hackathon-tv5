/**
 * EmotionDetector Integration Tests
 * Tests the complete emotion detection pipeline
 */

import { EmotionDetector } from '../src/emotion';

describe('EmotionDetector', () => {
  let detector: EmotionDetector;

  beforeEach(() => {
    detector = new EmotionDetector();
  });

  describe('analyzeText()', () => {
    it('should detect happy/joyful emotion', async () => {
      const result = await detector.analyzeText('I am so happy and excited about today!');

      expect(result.currentState.primaryEmotion).toBe('joy');
      expect(result.currentState.valence).toBeGreaterThan(0.5);
      expect(result.currentState.arousal).toBeGreaterThan(0.5);
      expect(result.currentState.stressLevel).toBeLessThan(0.5);
      expect(result.currentState.confidence).toBeGreaterThan(0.7);
    });

    it('should detect sad emotion', async () => {
      const result = await detector.analyzeText('I feel so sad and down today');

      expect(result.currentState.primaryEmotion).toBe('sadness');
      expect(result.currentState.valence).toBeLessThan(0);
      expect(result.currentState.arousal).toBeLessThan(0);
      expect(result.desiredState.targetValence).toBeGreaterThan(0.5);
    });

    it('should detect stressed/anxious emotion', async () => {
      const result = await detector.analyzeText('I am so stressed and anxious about work');

      expect(result.currentState.primaryEmotion).toBe('fear');
      expect(result.currentState.valence).toBeLessThan(0);
      expect(result.currentState.arousal).toBeGreaterThan(0.5);
      expect(result.currentState.stressLevel).toBeGreaterThan(0.6);
      expect(result.desiredState.targetArousal).toBeLessThan(0); // Should want calming
    });

    it('should detect angry emotion', async () => {
      const result = await detector.analyzeText('I am so frustrated and angry');

      expect(result.currentState.primaryEmotion).toBe('anger');
      expect(result.currentState.valence).toBeLessThan(0);
      expect(result.currentState.arousal).toBeGreaterThan(0.5);
      expect(result.currentState.stressLevel).toBeGreaterThan(0.7);
    });

    it('should detect calm emotion', async () => {
      const result = await detector.analyzeText('I feel calm and peaceful');

      expect(result.currentState.primaryEmotion).toBe('trust');
      expect(result.currentState.valence).toBeGreaterThan(0);
      expect(result.currentState.arousal).toBeLessThan(0);
      expect(result.currentState.stressLevel).toBeLessThan(0.3);
    });

    it('should handle neutral text', async () => {
      const result = await detector.analyzeText('The weather is normal today');

      expect(result.currentState.valence).toBeCloseTo(0, 1);
      expect(result.currentState.arousal).toBeCloseTo(0, 1);
      expect(result.currentState.confidence).toBeGreaterThan(0.5);
    });

    it('should generate valid emotion vectors', async () => {
      const result = await detector.analyzeText('I am happy');

      expect(result.currentState.emotionVector).toBeInstanceOf(Float32Array);
      expect(result.currentState.emotionVector.length).toBe(8);

      // Vector should sum to approximately 1.0
      const sum = Array.from(result.currentState.emotionVector).reduce((a, b) => a + b, 0);
      expect(sum).toBeCloseTo(1.0, 2);
    });

    it('should generate valid state hash', async () => {
      const result = await detector.analyzeText('I am happy');

      expect(result.stateHash).toMatch(/^\d:\d:\d$/);
    });

    it('should predict desired state for high stress', async () => {
      const result = await detector.analyzeText('I am extremely stressed and anxious');

      expect(result.desiredState.targetStress).toBeLessThan(result.currentState.stressLevel);
      expect(result.desiredState.targetArousal).toBeLessThan(0); // Want calming
      expect(result.desiredState.intensity).toMatch(/moderate|significant/);
      expect(result.desiredState.reasoning).toContain('stress');
    });

    it('should reject empty input', async () => {
      await expect(detector.analyzeText('')).rejects.toThrow('empty');
    });

    it('should reject too-short input', async () => {
      await expect(detector.analyzeText('ab')).rejects.toThrow('short');
    });

    it('should reject too-long input', async () => {
      const longText = 'a'.repeat(5001);
      await expect(detector.analyzeText(longText)).rejects.toThrow('long');
    });
  });
});
