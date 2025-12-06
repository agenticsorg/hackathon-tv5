/**
 * EmotionDetector Unit Tests (TDD - Red Phase)
 * Following London School mockist approach
 */

import { EmotionDetector } from '../../../src/emotion/detector';
import { EmotionalState, EmotionErrorCode, PlutchikEmotion } from '../../../src/emotion/types';

describe('EmotionDetector', () => {
  let detector: EmotionDetector;
  let mockGeminiClient: any;
  let mockAgentDB: any;

  beforeEach(() => {
    // Mock collaborators (London School approach)
    mockGeminiClient = {
      analyzeEmotion: jest.fn(),
    };

    mockAgentDB = {
      insert: jest.fn().mockResolvedValue(undefined),
    };

    // Create detector with mocked dependencies
    detector = new EmotionDetector(mockGeminiClient, mockAgentDB);
  });

  describe('analyzeText', () => {
    it('should return EmotionalState for valid text input', async () => {
      // Arrange
      const text = 'I am feeling happy today!';
      const userId = 'user_123';

      mockGeminiClient.analyzeEmotion.mockResolvedValue({
        primaryEmotion: 'joy' as PlutchikEmotion,
        valence: 0.8,
        arousal: 0.6,
        stressLevel: 0.2,
        confidence: 0.9,
        reasoning: 'User expressed happiness',
        rawResponse: {},
      });

      // Act
      const result = await detector.analyzeText(text, userId);

      // Assert
      expect(result).toBeDefined();
      expect(result.emotionalStateId).toBeDefined();
      expect(result.userId).toBe(userId);
      expect(result.rawText).toBe(text);
      expect(mockGeminiClient.analyzeEmotion).toHaveBeenCalledWith(text);
    });

    it('should include valence, arousal, stress in valid ranges', async () => {
      // Arrange
      mockGeminiClient.analyzeEmotion.mockResolvedValue({
        primaryEmotion: 'joy' as PlutchikEmotion,
        valence: 0.75,
        arousal: 0.5,
        stressLevel: 0.3,
        confidence: 0.85,
        reasoning: 'Test',
        rawResponse: {},
      });

      // Act
      const result = await detector.analyzeText('test text', 'user_123');

      // Assert
      expect(result.valence).toBeGreaterThanOrEqual(-1.0);
      expect(result.valence).toBeLessThanOrEqual(1.0);
      expect(result.arousal).toBeGreaterThanOrEqual(-1.0);
      expect(result.arousal).toBeLessThanOrEqual(1.0);
      expect(result.stressLevel).toBeGreaterThanOrEqual(0.0);
      expect(result.stressLevel).toBeLessThanOrEqual(1.0);
    });

    it('should generate 8D emotion vector', async () => {
      // Arrange
      mockGeminiClient.analyzeEmotion.mockResolvedValue({
        primaryEmotion: 'joy' as PlutchikEmotion,
        valence: 0.8,
        arousal: 0.6,
        stressLevel: 0.2,
        confidence: 0.9,
        reasoning: 'Test',
        rawResponse: {},
      });

      // Act
      const result = await detector.analyzeText('test', 'user_123');

      // Assert
      expect(result.emotionVector).toBeInstanceOf(Float32Array);
      expect(result.emotionVector.length).toBe(8);

      // Vector should sum to approximately 1.0
      const sum = Array.from(result.emotionVector).reduce((a, b) => a + b, 0);
      expect(sum).toBeCloseTo(1.0, 2);
    });

    it('should return fallback state on API failure', async () => {
      // Arrange
      mockGeminiClient.analyzeEmotion.mockRejectedValue(
        new Error('API timeout')
      );

      // Act
      const result = await detector.analyzeText('test', 'user_123');

      // Assert
      expect(result.valence).toBe(0.0);
      expect(result.arousal).toBe(0.0);
      expect(result.confidence).toBe(0.0);
      expect(result.primaryEmotion).toBe('trust');
    });

    it('should calculate confidence score', async () => {
      // Arrange
      mockGeminiClient.analyzeEmotion.mockResolvedValue({
        primaryEmotion: 'joy' as PlutchikEmotion,
        valence: 0.8,
        arousal: 0.6,
        stressLevel: 0.2,
        confidence: 0.9,
        reasoning: 'Detailed reasoning provided',
        rawResponse: {},
      });

      // Act
      const result = await detector.analyzeText('test', 'user_123');

      // Assert
      expect(result.confidence).toBeGreaterThanOrEqual(0.0);
      expect(result.confidence).toBeLessThanOrEqual(1.0);
      expect(result.confidence).toBeGreaterThan(0.5); // Should be high for valid response
    });

    it('should reject text that is too short', async () => {
      // Act & Assert
      await expect(
        detector.analyzeText('ab', 'user_123')
      ).rejects.toThrow();
    });

    it('should reject empty text', async () => {
      // Act & Assert
      await expect(
        detector.analyzeText('', 'user_123')
      ).rejects.toThrow();
    });

    it('should save to AgentDB asynchronously', async () => {
      // Arrange
      mockGeminiClient.analyzeEmotion.mockResolvedValue({
        primaryEmotion: 'joy' as PlutchikEmotion,
        valence: 0.8,
        arousal: 0.6,
        stressLevel: 0.2,
        confidence: 0.9,
        reasoning: 'Test',
        rawResponse: {},
      });

      // Act
      await detector.analyzeText('test', 'user_123');

      // Wait for async save
      await new Promise(resolve => setTimeout(resolve, 100));

      // Assert
      expect(mockAgentDB.insert).toHaveBeenCalled();
    });

    it('should handle API timeout with retry logic', async () => {
      // Arrange - fail twice, succeed third time
      mockGeminiClient.analyzeEmotion
        .mockRejectedValueOnce(new Error('Timeout'))
        .mockRejectedValueOnce(new Error('Timeout'))
        .mockResolvedValueOnce({
          primaryEmotion: 'joy' as PlutchikEmotion,
          valence: 0.8,
          arousal: 0.6,
          stressLevel: 0.2,
          confidence: 0.9,
          reasoning: 'Test',
          rawResponse: {},
        });

      // Act
      const result = await detector.analyzeText('test', 'user_123');

      // Assert
      expect(mockGeminiClient.analyzeEmotion).toHaveBeenCalledTimes(3);
      expect(result.confidence).toBeGreaterThan(0);
    });

    it('should normalize valence-arousal to circumplex constraints', async () => {
      // Arrange - values outside unit circle
      mockGeminiClient.analyzeEmotion.mockResolvedValue({
        primaryEmotion: 'joy' as PlutchikEmotion,
        valence: 1.2, // Out of range
        arousal: 1.0,
        stressLevel: 0.2,
        confidence: 0.9,
        reasoning: 'Test',
        rawResponse: {},
      });

      // Act
      const result = await detector.analyzeText('test', 'user_123');

      // Assert
      const magnitude = Math.sqrt(result.valence ** 2 + result.arousal ** 2);
      expect(magnitude).toBeLessThanOrEqual(1.414); // √2
    });
  });

  describe('createFallbackState', () => {
    it('should create neutral state with zero confidence', () => {
      // Act
      const fallback = (detector as any).createFallbackState('user_123');

      // Assert
      expect(fallback.valence).toBe(0.0);
      expect(fallback.arousal).toBe(0.0);
      expect(fallback.confidence).toBe(0.0);
      expect(fallback.primaryEmotion).toBe('trust');
      expect(fallback.stressLevel).toBe(0.5);
    });

    it('should generate uniform emotion vector', () => {
      // Act
      const fallback = (detector as any).createFallbackState('user_123');

      // Assert
      const expectedValue = 1.0 / 8.0;
      for (let i = 0; i < 8; i++) {
        expect(fallback.emotionVector[i]).toBeCloseTo(expectedValue, 3);
      }
    });
  });

  describe('calculateConfidence', () => {
    it('should combine Gemini confidence with consistency score', () => {
      // Arrange
      const geminiResponse = {
        primaryEmotion: 'joy' as PlutchikEmotion,
        valence: 0.8,
        arousal: 0.6,
        stressLevel: 0.2,
        confidence: 0.9,
        reasoning: 'Detailed explanation',
        rawResponse: {},
      };

      // Act
      const confidence = (detector as any).calculateConfidence(geminiResponse);

      // Assert
      expect(confidence).toBeGreaterThan(0.7);
      expect(confidence).toBeLessThanOrEqual(1.0);
    });

    it('should penalize missing reasoning', () => {
      // Arrange
      const withReasoning = {
        primaryEmotion: 'joy' as PlutchikEmotion,
        valence: 0.8,
        arousal: 0.6,
        stressLevel: 0.2,
        confidence: 0.9,
        reasoning: 'Detailed explanation',
        rawResponse: {},
      };

      const withoutReasoning = {
        ...withReasoning,
        reasoning: '',
      };

      // Act
      const confidenceWith = (detector as any).calculateConfidence(withReasoning);
      const confidenceWithout = (detector as any).calculateConfidence(withoutReasoning);

      // Assert
      expect(confidenceWith).toBeGreaterThan(confidenceWithout);
    });
  });
});
