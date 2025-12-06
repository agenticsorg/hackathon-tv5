/**
 * RewardCalculator Tests
 * TDD approach for reward function validation
 */

import { RewardCalculator } from '../../../src/rl/reward-calculator';
import { EmotionalState, DesiredState } from '../../../src/rl/types';

describe('RewardCalculator', () => {
  let calculator: RewardCalculator;

  beforeEach(() => {
    calculator = new RewardCalculator();
  });

  describe('calculate', () => {
    it('should return positive reward for improvement toward desired state', () => {
      // Arrange
      const before: EmotionalState = {
        valence: -0.6,
        arousal: 0.5,
        stress: 0.7,
        confidence: 0.8
      };

      const after: EmotionalState = {
        valence: 0.2,
        arousal: 0.1,
        stress: 0.5,
        confidence: 0.8
      };

      const desired: DesiredState = {
        valence: 0.6,
        arousal: 0.3,
        confidence: 0.8
      };

      // Act
      const reward = calculator.calculate(before, after, desired);

      // Assert
      expect(reward).toBeGreaterThan(0);
      expect(reward).toBeLessThanOrEqual(1.0);
    });

    it('should return negative reward for movement away from desired state', () => {
      // Arrange
      const before: EmotionalState = {
        valence: 0.3,
        arousal: 0.2,
        stress: 0.3,
        confidence: 0.8
      };

      const after: EmotionalState = {
        valence: -0.5,
        arousal: 0.8,
        stress: 0.9,
        confidence: 0.8
      };

      const desired: DesiredState = {
        valence: 0.6,
        arousal: 0.3,
        confidence: 0.8
      };

      // Act
      const reward = calculator.calculate(before, after, desired);

      // Assert
      expect(reward).toBeLessThan(0);
    });

    it('should calculate direction alignment using cosine similarity', () => {
      // Arrange
      const before: EmotionalState = {
        valence: 0,
        arousal: 0,
        stress: 0.5,
        confidence: 0.8
      };

      const after: EmotionalState = {
        valence: 0.5,
        arousal: 0.3,
        stress: 0.3,
        confidence: 0.8
      };

      const desired: DesiredState = {
        valence: 0.6,
        arousal: 0.3,
        confidence: 0.8
      };

      // Act
      const reward = calculator.calculate(before, after, desired);

      // Assert - Should be positive due to alignment
      expect(reward).toBeGreaterThan(0.5);
    });

    it('should apply proximity bonus when close to desired state', () => {
      // Arrange
      const before: EmotionalState = {
        valence: 0.5,
        arousal: 0.25,
        stress: 0.3,
        confidence: 0.8
      };

      const after: EmotionalState = {
        valence: 0.58,
        arousal: 0.28,
        stress: 0.2,
        confidence: 0.8
      };

      const desired: DesiredState = {
        valence: 0.6,
        arousal: 0.3,
        confidence: 0.8
      };

      // Act
      const reward = calculator.calculate(before, after, desired);

      // Assert - Should include proximity bonus
      expect(reward).toBeGreaterThan(0.7);
    });
  });

  describe('directionAlignment', () => {
    it('should return high score for aligned movement', () => {
      // Arrange
      const before: EmotionalState = {
        valence: -0.5,
        arousal: -0.5,
        stress: 0.7,
        confidence: 0.8
      };

      const after: EmotionalState = {
        valence: 0.5,
        arousal: 0.3,
        stress: 0.3,
        confidence: 0.8
      };

      const desired: DesiredState = {
        valence: 0.6,
        arousal: 0.3,
        confidence: 0.8
      };

      // Act
      const alignment = calculator['directionAlignment'](before, after, desired);

      // Assert
      expect(alignment).toBeGreaterThan(0.8);
    });
  });

  describe('magnitude', () => {
    it('should score based on movement magnitude', () => {
      // Arrange
      const before: EmotionalState = {
        valence: 0,
        arousal: 0,
        stress: 0.5,
        confidence: 0.8
      };

      const after: EmotionalState = {
        valence: 0.3,
        arousal: 0.2,
        stress: 0.3,
        confidence: 0.8
      };

      const desired: DesiredState = {
        valence: 0.6,
        arousal: 0.4,
        confidence: 0.8
      };

      // Act
      const magnitude = calculator['magnitude'](before, after, desired);

      // Assert
      expect(magnitude).toBeGreaterThan(0);
      expect(magnitude).toBeLessThanOrEqual(1.0);
    });
  });

  describe('proximityBonus', () => {
    it('should return bonus when within threshold', () => {
      // Arrange
      const after: EmotionalState = {
        valence: 0.58,
        arousal: 0.28,
        stress: 0.2,
        confidence: 0.8
      };

      const desired: DesiredState = {
        valence: 0.6,
        arousal: 0.3,
        confidence: 0.8
      };

      // Act
      const bonus = calculator['proximityBonus'](after, desired);

      // Assert
      expect(bonus).toBeGreaterThan(0);
    });

    it('should return 0 when far from desired state', () => {
      // Arrange
      const after: EmotionalState = {
        valence: 0.0,
        arousal: 0.0,
        stress: 0.5,
        confidence: 0.8
      };

      const desired: DesiredState = {
        valence: 0.6,
        arousal: 0.3,
        confidence: 0.8
      };

      // Act
      const bonus = calculator['proximityBonus'](after, desired);

      // Assert
      expect(bonus).toBe(0);
    });
  });
});
