/**
 * UCB Calculator Tests
 */

import { UCBCalculator } from '../../../src/rl/exploration/ucb';

describe('UCBCalculator', () => {
  let calculator: UCBCalculator;

  beforeEach(() => {
    calculator = new UCBCalculator(2.0);
  });

  describe('calculate', () => {
    it('should return UCB value with exploration bonus', () => {
      // Arrange
      const qValue = 0.5;
      const visitCount = 10;
      const totalVisits = 45;

      // Act
      const ucb = calculator.calculate(qValue, visitCount, totalVisits);

      // Assert
      // UCB = Q + c * sqrt(ln(N) / n)
      // UCB = 0.5 + 2.0 * sqrt(ln(45) / 10)
      // UCB = 0.5 + 2.0 * sqrt(3.807 / 10)
      // UCB = 0.5 + 2.0 * 0.617 = 0.5 + 1.234 = 1.234
      expect(ucb).toBeGreaterThan(qValue);
      expect(ucb).toBeCloseTo(1.234, 1);
    });

    it('should return Infinity for unvisited actions', () => {
      // Arrange
      const qValue = 0.5;
      const visitCount = 0;
      const totalVisits = 45;

      // Act
      const ucb = calculator.calculate(qValue, visitCount, totalVisits);

      // Assert
      expect(ucb).toBe(Infinity);
    });

    it('should give higher bonus to less-visited actions', () => {
      // Arrange
      const qValue = 0.5;
      const totalVisits = 100;

      // Act
      const ucb1 = calculator.calculate(qValue, 50, totalVisits);
      const ucb2 = calculator.calculate(qValue, 5, totalVisits);

      // Assert - less visited (5) should have higher UCB
      expect(ucb2).toBeGreaterThan(ucb1);
    });

    it('should use constant c for exploration weight', () => {
      // Arrange
      const calculator1 = new UCBCalculator(1.0);
      const calculator2 = new UCBCalculator(3.0);

      const qValue = 0.5;
      const visitCount = 10;
      const totalVisits = 45;

      // Act
      const ucb1 = calculator1.calculate(qValue, visitCount, totalVisits);
      const ucb2 = calculator2.calculate(qValue, visitCount, totalVisits);

      // Assert - higher c should give higher exploration bonus
      expect(ucb2).toBeGreaterThan(ucb1);
    });
  });
});
