/**
 * OutcomePredictor Unit Tests
 */

import { OutcomePredictor } from '../outcome-predictor';
import { EmotionalContentProfile } from '../../content/types';

describe('OutcomePredictor', () => {
  let predictor: OutcomePredictor;

  beforeEach(() => {
    predictor = new OutcomePredictor();
  });

  const createProfile = (
    valenceDelta: number,
    arousalDelta: number,
    intensity: number = 0.5,
    complexity: number = 0.5
  ): EmotionalContentProfile => ({
    contentId: 'test',
    primaryTone: 'neutral',
    valenceDelta,
    arousalDelta,
    intensity,
    complexity,
    targetStates: [],
    embeddingId: 'emb_test',
    timestamp: Date.now()
  });

  describe('predict()', () => {
    it('should predict post-viewing state by applying deltas', () => {
      const currentState = {
        valence: -0.4,
        arousal: 0.6,
        stress: 0.8,
        confidence: 0.8
      };

      const profile = createProfile(0.7, -0.6);

      const outcome = predictor.predict(currentState, profile);

      expect(outcome.expectedValence).toBeCloseTo(0.3, 1);
      expect(outcome.expectedArousal).toBeCloseTo(0.0, 1);
      expect(outcome.expectedStress).toBeLessThan(0.8);
    });

    it('should clamp values to valid ranges', () => {
      const currentState = {
        valence: 0.8,
        arousal: 0.9,
        stress: 0.1,
        confidence: 0.8
      };

      const profile = createProfile(0.5, 0.5); // Would exceed 1.0

      const outcome = predictor.predict(currentState, profile);

      expect(outcome.expectedValence).toBeLessThanOrEqual(1.0);
      expect(outcome.expectedArousal).toBeLessThanOrEqual(1.0);
      expect(outcome.expectedStress).toBeGreaterThanOrEqual(0.0);
    });

    it('should calculate confidence based on complexity', () => {
      const state = {
        valence: 0.0,
        arousal: 0.0,
        stress: 0.5,
        confidence: 0.8
      };

      const simpleProfile = createProfile(0.3, -0.2, 0.5, 0.2);
      const complexProfile = createProfile(0.3, -0.2, 0.5, 0.9);

      const simpleOutcome = predictor.predict(state, simpleProfile);
      const complexOutcome = predictor.predict(state, complexProfile);

      // Higher complexity = lower confidence
      expect(simpleOutcome.confidence).toBeGreaterThan(complexOutcome.confidence);
    });

    it('should reduce stress based on intensity', () => {
      const state = {
        valence: 0.0,
        arousal: 0.0,
        stress: 0.8,
        confidence: 0.8
      };

      const lowIntensity = createProfile(0.2, -0.3, 0.2);
      const highIntensity = createProfile(0.2, -0.3, 0.9);

      const lowOutcome = predictor.predict(state, lowIntensity);
      const highOutcome = predictor.predict(state, highIntensity);

      // Higher intensity = more stress reduction
      expect(highOutcome.expectedStress).toBeLessThan(lowOutcome.expectedStress);
    });
  });
});
