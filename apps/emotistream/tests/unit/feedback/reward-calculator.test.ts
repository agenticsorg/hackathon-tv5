/**
 * RewardCalculator Unit Tests
 * Testing multi-factor reward formula
 */

import { RewardCalculator } from '../../../src/feedback/reward-calculator';
import { EmotionalState, ViewingDetails } from '../../../src/feedback/types';

describe('RewardCalculator', () => {
  let calculator: RewardCalculator;

  beforeEach(() => {
    calculator = new RewardCalculator();
  });

  describe('calculate', () => {
    it('should weight direction alignment at 60%', () => {
      const stateBefore: EmotionalState = {
        valence: -0.6,
        arousal: 0.2,
        dominance: 0.0,
        confidence: 0.8,
        timestamp: new Date(),
      };

      const stateAfter: EmotionalState = {
        valence: 0.2,
        arousal: -0.3,
        dominance: 0.0,
        confidence: 0.8,
        timestamp: new Date(),
      };

      const desiredState: EmotionalState = {
        valence: 0.5,
        arousal: -0.2,
        dominance: 0.0,
        confidence: 1.0,
        timestamp: new Date(),
      };

      const reward = calculator.calculate(stateBefore, stateAfter, desiredState);

      // Perfect direction alignment should contribute significantly
      expect(reward).toBeGreaterThan(0.5);
    });

    it('should weight magnitude at 40%', () => {
      const stateBefore: EmotionalState = {
        valence: 0.0,
        arousal: 0.0,
        dominance: 0.0,
        confidence: 0.8,
        timestamp: new Date(),
      };

      const stateAfter: EmotionalState = {
        valence: 0.8,
        arousal: -0.6,
        dominance: 0.0,
        confidence: 0.8,
        timestamp: new Date(),
      };

      const desiredState: EmotionalState = {
        valence: 0.9,
        arousal: -0.7,
        dominance: 0.0,
        confidence: 1.0,
        timestamp: new Date(),
      };

      const reward = calculator.calculate(stateBefore, stateAfter, desiredState);

      // Large magnitude change in right direction
      expect(reward).toBeGreaterThan(0.6);
    });

    it('should add proximity bonus when close to desired state', () => {
      const stateBefore: EmotionalState = {
        valence: 0.4,
        arousal: -0.2,
        dominance: 0.0,
        confidence: 0.8,
        timestamp: new Date(),
      };

      const stateAfter: EmotionalState = {
        valence: 0.6,
        arousal: -0.3,
        dominance: 0.0,
        confidence: 0.8,
        timestamp: new Date(),
      };

      const desiredState: EmotionalState = {
        valence: 0.6,
        arousal: -0.3,
        dominance: 0.0,
        confidence: 1.0,
        timestamp: new Date(),
      };

      const reward = calculator.calculate(stateBefore, stateAfter, desiredState);

      // Should get proximity bonus for reaching target
      expect(reward).toBeGreaterThan(0.7);
    });

    it('should return reward in [-1, +1] range', () => {
      const stateBefore: EmotionalState = {
        valence: -0.8,
        arousal: 0.8,
        dominance: 0.0,
        confidence: 0.8,
        timestamp: new Date(),
      };

      const stateAfter: EmotionalState = {
        valence: 0.8,
        arousal: -0.8,
        dominance: 0.0,
        confidence: 0.8,
        timestamp: new Date(),
      };

      const desiredState: EmotionalState = {
        valence: 0.9,
        arousal: -0.9,
        dominance: 0.0,
        confidence: 1.0,
        timestamp: new Date(),
      };

      const reward = calculator.calculate(stateBefore, stateAfter, desiredState);

      expect(reward).toBeGreaterThanOrEqual(-1.0);
      expect(reward).toBeLessThanOrEqual(1.0);
    });

    it('should return negative reward for opposite direction', () => {
      const stateBefore: EmotionalState = {
        valence: -0.2,
        arousal: 0.4,
        dominance: 0.0,
        confidence: 0.8,
        timestamp: new Date(),
      };

      const stateAfter: EmotionalState = {
        valence: -0.8,
        arousal: 0.8,
        dominance: 0.0,
        confidence: 0.8,
        timestamp: new Date(),
      };

      const desiredState: EmotionalState = {
        valence: 0.5,
        arousal: -0.6,
        dominance: 0.0,
        confidence: 1.0,
        timestamp: new Date(),
      };

      const reward = calculator.calculate(stateBefore, stateAfter, desiredState);

      expect(reward).toBeLessThan(0);
    });

    it('should handle zero magnitude change', () => {
      const stateBefore: EmotionalState = {
        valence: 0.5,
        arousal: -0.2,
        dominance: 0.0,
        confidence: 0.8,
        timestamp: new Date(),
      };

      const stateAfter: EmotionalState = {
        valence: 0.5,
        arousal: -0.2,
        dominance: 0.0,
        confidence: 0.8,
        timestamp: new Date(),
      };

      const desiredState: EmotionalState = {
        valence: 0.6,
        arousal: -0.3,
        dominance: 0.0,
        confidence: 1.0,
        timestamp: new Date(),
      };

      const reward = calculator.calculate(stateBefore, stateAfter, desiredState);

      // No change should give low reward (might get proximity bonus)
      expect(reward).toBeLessThan(0.3);
    });
  });

  describe('calculateCompletionBonus', () => {
    it('should give positive bonus for >80% completion', () => {
      const viewingDetails: ViewingDetails = {
        completionRate: 0.95,
        durationSeconds: 1800,
      };

      const bonus = calculator.calculateCompletionBonus(viewingDetails);

      expect(bonus).toBeGreaterThan(0);
      expect(bonus).toBeLessThanOrEqual(0.2);
    });

    it('should give negative bonus for <30% completion', () => {
      const viewingDetails: ViewingDetails = {
        completionRate: 0.20,
        durationSeconds: 300,
      };

      const bonus = calculator.calculateCompletionBonus(viewingDetails);

      expect(bonus).toBeLessThan(0);
    });

    it('should apply pause penalty', () => {
      const withoutPauses: ViewingDetails = {
        completionRate: 0.90,
        durationSeconds: 1800,
        pauseCount: 0,
      };

      const withPauses: ViewingDetails = {
        completionRate: 0.90,
        durationSeconds: 1800,
        pauseCount: 5,
      };

      const bonusWithout = calculator.calculateCompletionBonus(withoutPauses);
      const bonusWith = calculator.calculateCompletionBonus(withPauses);

      expect(bonusWith).toBeLessThan(bonusWithout);
    });

    it('should apply skip penalty', () => {
      const withoutSkips: ViewingDetails = {
        completionRate: 0.90,
        durationSeconds: 1800,
        skipCount: 0,
      };

      const withSkips: ViewingDetails = {
        completionRate: 0.90,
        durationSeconds: 1800,
        skipCount: 3,
      };

      const bonusWithout = calculator.calculateCompletionBonus(withoutSkips);
      const bonusWith = calculator.calculateCompletionBonus(withSkips);

      expect(bonusWith).toBeLessThan(bonusWithout);
    });
  });

  describe('calculateInsights', () => {
    it('should return detailed breakdown of reward components', () => {
      const stateBefore: EmotionalState = {
        valence: -0.4,
        arousal: 0.6,
        dominance: 0.0,
        confidence: 0.8,
        timestamp: new Date(),
      };

      const stateAfter: EmotionalState = {
        valence: 0.5,
        arousal: -0.2,
        dominance: 0.0,
        confidence: 0.8,
        timestamp: new Date(),
      };

      const desiredState: EmotionalState = {
        valence: 0.6,
        arousal: -0.3,
        dominance: 0.0,
        confidence: 1.0,
        timestamp: new Date(),
      };

      const insights = calculator.calculateInsights(
        stateBefore,
        stateAfter,
        desiredState,
        0.15
      );

      expect(insights.directionAlignment).toBeDefined();
      expect(insights.magnitudeScore).toBeDefined();
      expect(insights.proximityBonus).toBeDefined();
      expect(insights.completionBonus).toBe(0.15);
      expect(insights.directionAlignment).toBeGreaterThanOrEqual(-1);
      expect(insights.directionAlignment).toBeLessThanOrEqual(1);
    });
  });
});
