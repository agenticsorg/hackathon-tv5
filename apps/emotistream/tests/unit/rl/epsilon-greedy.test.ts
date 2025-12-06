/**
 * EpsilonGreedy Strategy Tests
 */

import { EpsilonGreedyStrategy } from '../../../src/rl/exploration/epsilon-greedy';

describe('EpsilonGreedyStrategy', () => {
  let strategy: EpsilonGreedyStrategy;

  beforeEach(() => {
    strategy = new EpsilonGreedyStrategy(0.15, 0.10, 0.95);
  });

  describe('shouldExplore', () => {
    it('should return boolean based on epsilon', () => {
      // Act & Assert - run multiple times due to randomness
      const results = Array.from({ length: 100 }, () => strategy.shouldExplore());
      const explorationCount = results.filter(r => r).length;

      // Expect roughly 15% exploration (with some variance)
      expect(explorationCount).toBeGreaterThan(5);
      expect(explorationCount).toBeLessThan(30);
    });
  });

  describe('selectRandom', () => {
    it('should select random action from available actions', () => {
      // Arrange
      const actions = ['action-1', 'action-2', 'action-3', 'action-4'];

      // Act
      const selected = strategy.selectRandom(actions);

      // Assert
      expect(actions).toContain(selected);
    });

    it('should return different actions over multiple calls', () => {
      // Arrange
      const actions = ['action-1', 'action-2', 'action-3', 'action-4', 'action-5'];
      const selections = new Set<string>();

      // Act - select 50 times
      for (let i = 0; i < 50; i++) {
        selections.add(strategy.selectRandom(actions));
      }

      // Assert - should see multiple different selections
      expect(selections.size).toBeGreaterThan(2);
    });
  });

  describe('decay', () => {
    it('should decay epsilon by decay rate', () => {
      // Arrange
      const initialEpsilon = 0.15;
      const decayRate = 0.95;

      // Act
      strategy.decay();

      // Assert - epsilon should be 0.15 * 0.95 = 0.1425
      // We test this indirectly through exploration rate
      expect(strategy['epsilon']).toBeCloseTo(initialEpsilon * decayRate, 3);
    });

    it('should not decay below minimum epsilon', () => {
      // Arrange - decay many times
      for (let i = 0; i < 20; i++) {
        strategy.decay();
      }

      // Assert
      expect(strategy['epsilon']).toBeGreaterThanOrEqual(0.10);
    });

    it('should reach minimum epsilon after sufficient decays', () => {
      // Act - decay until minimum
      for (let i = 0; i < 10; i++) {
        strategy.decay();
      }

      // Assert
      expect(strategy['epsilon']).toBe(0.10);
    });
  });
});
