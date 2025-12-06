import { OutcomePredictor } from '../../../src/recommendations/outcome-predictor';

describe('OutcomePredictor', () => {
  let predictor: OutcomePredictor;

  beforeEach(() => {
    predictor = new OutcomePredictor();
  });

  describe('predict', () => {
    it('should predict post-viewing state by adding deltas', () => {
      // Arrange
      const currentState = {
        id: 'state1',
        userId: 'user123',
        timestamp: Date.now(),
        valence: -0.4,
        arousal: 0.6,
        stressLevel: 0.8,
        dominance: 0.0,
        rawMetrics: {},
      };

      const profile = {
        contentId: 'content1',
        title: 'Content 1',
        platform: 'Netflix',
        valenceDelta: 0.7,
        arousalDelta: -0.6,
        stressReduction: 0.5,
        duration: 45,
        genres: ['Drama'],
        embedding: new Float32Array(1536),
      };

      // Act
      const outcome = predictor.predict(currentState, profile);

      // Assert
      expect(outcome.postViewingValence).toBeCloseTo(0.3, 1);
      expect(outcome.postViewingArousal).toBeCloseTo(0.0, 1);
      expect(outcome.postViewingStress).toBeCloseTo(0.3, 1);
    });

    it('should clamp values to valid ranges', () => {
      // Arrange
      const currentState = {
        id: 'state1',
        userId: 'user123',
        timestamp: Date.now(),
        valence: 0.8,
        arousal: 0.9,
        stressLevel: 0.1,
        dominance: 0.0,
        rawMetrics: {},
      };

      const profile = {
        contentId: 'content1',
        title: 'Content 1',
        platform: 'Netflix',
        valenceDelta: 0.5, // Would exceed 1.0
        arousalDelta: 0.5, // Would exceed 1.0
        stressReduction: 0.3, // Would go negative
        duration: 45,
        genres: ['Drama'],
        embedding: new Float32Array(1536),
      };

      // Act
      const outcome = predictor.predict(currentState, profile);

      // Assert
      expect(outcome.postViewingValence).toBe(1.0); // Clamped
      expect(outcome.postViewingArousal).toBe(1.0); // Clamped
      expect(outcome.postViewingStress).toBe(0.0); // Clamped to 0
    });

    it('should calculate confidence based on watch count and variance', () => {
      // Arrange
      const currentState = {
        id: 'state1',
        userId: 'user123',
        timestamp: Date.now(),
        valence: 0.0,
        arousal: 0.0,
        stressLevel: 0.5,
        dominance: 0.0,
        rawMetrics: {},
      };

      const profile1 = {
        contentId: 'content1',
        title: 'Content 1',
        platform: 'Netflix',
        valenceDelta: 0.5,
        arousalDelta: -0.3,
        stressReduction: 0.4,
        duration: 45,
        genres: ['Drama'],
        embedding: new Float32Array(1536),
        totalWatches: 0,
        outcomeVariance: 1.0,
      };

      const profile2 = {
        contentId: 'content2',
        title: 'Content 2',
        platform: 'YouTube',
        valenceDelta: 0.5,
        arousalDelta: -0.3,
        stressReduction: 0.4,
        duration: 30,
        genres: ['Documentary'],
        embedding: new Float32Array(1536),
        totalWatches: 100,
        outcomeVariance: 0.05,
      };

      // Act
      const outcome1 = predictor.predict(currentState, profile1);
      const outcome2 = predictor.predict(currentState, profile2);

      // Assert
      expect(outcome1.confidence).toBeLessThan(0.2); // Low confidence
      expect(outcome2.confidence).toBeGreaterThan(0.9); // High confidence
    });

    it('should handle missing totalWatches and outcomeVariance', () => {
      // Arrange
      const currentState = {
        id: 'state1',
        userId: 'user123',
        timestamp: Date.now(),
        valence: 0.0,
        arousal: 0.0,
        stressLevel: 0.5,
        dominance: 0.0,
        rawMetrics: {},
      };

      const profile = {
        contentId: 'content1',
        title: 'Content 1',
        platform: 'Netflix',
        valenceDelta: 0.5,
        arousalDelta: -0.3,
        stressReduction: 0.4,
        duration: 45,
        genres: ['Drama'],
        embedding: new Float32Array(1536),
        // No totalWatches or outcomeVariance
      };

      // Act
      const outcome = predictor.predict(currentState, profile);

      // Assert
      expect(outcome.confidence).toBeDefined();
      expect(outcome.confidence).toBeGreaterThanOrEqual(0.1);
      expect(outcome.confidence).toBeLessThanOrEqual(0.95);
    });
  });
});
