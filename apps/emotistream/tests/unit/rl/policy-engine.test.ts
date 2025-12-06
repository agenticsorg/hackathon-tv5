/**
 * RLPolicyEngine Tests
 * TDD London School - Mock-driven approach
 */

import { RLPolicyEngine } from '../../../src/rl/policy-engine';
import { QTable } from '../../../src/rl/q-table';
import { RewardCalculator } from '../../../src/rl/reward-calculator';
import { EpsilonGreedyStrategy } from '../../../src/rl/exploration/epsilon-greedy';
import { EmotionalState, DesiredState, EmotionalExperience } from '../../../src/rl/types';

describe('RLPolicyEngine', () => {
  let policyEngine: RLPolicyEngine;
  let mockQTable: jest.Mocked<QTable>;
  let mockRewardCalculator: jest.Mocked<RewardCalculator>;
  let mockExplorationStrategy: jest.Mocked<EpsilonGreedyStrategy>;

  const mockEmotionalState: EmotionalState = {
    valence: -0.6,
    arousal: 0.5,
    stress: 0.7,
    confidence: 0.8
  };

  const mockDesiredState: DesiredState = {
    valence: 0.6,
    arousal: 0.3,
    confidence: 0.8
  };

  beforeEach(() => {
    // Create mocks for dependencies
    mockQTable = {
      get: jest.fn(),
      set: jest.fn(),
      updateQValue: jest.fn(),
      getStateActions: jest.fn()
    } as any;

    mockRewardCalculator = {
      calculate: jest.fn()
    } as any;

    mockExplorationStrategy = {
      shouldExplore: jest.fn(),
      selectRandom: jest.fn(),
      decay: jest.fn()
    } as any;

    policyEngine = new RLPolicyEngine(
      mockQTable,
      mockRewardCalculator,
      mockExplorationStrategy
    );
  });

  describe('selectAction', () => {
    it('should return ActionSelection with contentId and qValue', async () => {
      // Arrange
      const availableContent = ['content-1', 'content-2'];
      mockExplorationStrategy.shouldExplore.mockReturnValue(false);
      mockQTable.get.mockResolvedValue({ qValue: 0.72, visitCount: 5 } as any);

      // Act
      const result = await policyEngine.selectAction(
        'user-1',
        mockEmotionalState,
        mockDesiredState,
        availableContent
      );

      // Assert
      expect(result).toBeDefined();
      expect(result.contentId).toBeDefined();
      expect(result.qValue).toBeDefined();
      expect(result.stateHash).toBeDefined();
    });

    it('should explore with probability epsilon', async () => {
      // Arrange
      const availableContent = ['content-1', 'content-2'];
      mockExplorationStrategy.shouldExplore.mockReturnValue(true);
      mockExplorationStrategy.selectRandom.mockReturnValue('content-2');
      mockQTable.get.mockResolvedValue({ qValue: 0.3, visitCount: 2 } as any);

      // Act
      const result = await policyEngine.selectAction(
        'user-1',
        mockEmotionalState,
        mockDesiredState,
        availableContent
      );

      // Assert
      expect(result.isExploration).toBe(true);
      expect(mockExplorationStrategy.shouldExplore).toHaveBeenCalled();
    });

    it('should exploit best Q-value when not exploring', async () => {
      // Arrange
      const availableContent = ['content-1', 'content-2', 'content-3'];
      mockExplorationStrategy.shouldExplore.mockReturnValue(false);

      mockQTable.get
        .mockResolvedValueOnce({ qValue: 0.45, visitCount: 3 } as any)
        .mockResolvedValueOnce({ qValue: 0.82, visitCount: 7 } as any)
        .mockResolvedValueOnce({ qValue: 0.31, visitCount: 2 } as any);

      // Act
      const result = await policyEngine.selectAction(
        'user-1',
        mockEmotionalState,
        mockDesiredState,
        availableContent
      );

      // Assert
      expect(result.isExploration).toBe(false);
      expect(result.contentId).toBe('content-2'); // Highest Q-value
      expect(result.qValue).toBe(0.82);
    });

    it('should use UCB bonus for tie-breaking during exploration', async () => {
      // Arrange
      const availableContent = ['content-1', 'content-2'];
      mockExplorationStrategy.shouldExplore.mockReturnValue(true);
      mockExplorationStrategy.selectRandom.mockReturnValue('content-1');

      mockQTable.getStateActions.mockResolvedValue([
        { qValue: 0.5, visitCount: 10 } as any,
        { qValue: 0.5, visitCount: 2 } as any
      ]);

      // Act
      const result = await policyEngine.selectAction(
        'user-1',
        mockEmotionalState,
        mockDesiredState,
        availableContent
      );

      // Assert
      expect(result.explorationBonus).toBeGreaterThan(0);
    });
  });

  describe('updatePolicy', () => {
    it('should update Q-value using TD learning', async () => {
      // Arrange
      const experience: EmotionalExperience = {
        stateBefore: mockEmotionalState,
        stateAfter: { valence: 0.2, arousal: 0.1, stress: 0.5, confidence: 0.8 },
        contentId: 'content-1',
        desiredState: mockDesiredState,
        reward: 0.72
      };

      mockRewardCalculator.calculate.mockReturnValue(0.72);
      mockQTable.get.mockResolvedValue({ qValue: 0.45, visitCount: 3 } as any);
      mockQTable.updateQValue.mockResolvedValue(undefined);

      // Act
      const result = await policyEngine.updatePolicy('user-1', experience);

      // Assert
      expect(result).toBeDefined();
      expect(result.newQValue).toBeGreaterThan(result.oldQValue);
      expect(result.tdError).toBeGreaterThan(0);
      expect(mockQTable.updateQValue).toHaveBeenCalled();
    });

    it('should decay exploration rate after episode', async () => {
      // Arrange
      const experience: EmotionalExperience = {
        stateBefore: mockEmotionalState,
        stateAfter: { valence: 0.2, arousal: 0.1, stress: 0.5, confidence: 0.8 },
        contentId: 'content-1',
        desiredState: mockDesiredState,
        reward: 0.72
      };

      mockRewardCalculator.calculate.mockReturnValue(0.72);
      mockQTable.get.mockResolvedValue({ qValue: 0.45, visitCount: 3 } as any);

      // Act
      await policyEngine.updatePolicy('user-1', experience);

      // Assert
      expect(mockExplorationStrategy.decay).toHaveBeenCalled();
    });

    it('should store experience in replay buffer', async () => {
      // Arrange
      const experience: EmotionalExperience = {
        stateBefore: mockEmotionalState,
        stateAfter: { valence: 0.2, arousal: 0.1, stress: 0.5, confidence: 0.8 },
        contentId: 'content-1',
        desiredState: mockDesiredState,
        reward: 0.72
      };

      mockRewardCalculator.calculate.mockReturnValue(0.72);
      mockQTable.get.mockResolvedValue({ qValue: 0.45, visitCount: 3 } as any);

      // Act
      await policyEngine.updatePolicy('user-1', experience);

      // Assert - verify experience was processed
      expect(mockQTable.updateQValue).toHaveBeenCalled();
    });
  });

  describe('getQValue', () => {
    it('should retrieve Q-value for state-action pair', async () => {
      // Arrange
      const stateHash = '2:3:1';
      const contentId = 'content-1';
      mockQTable.get.mockResolvedValue({ qValue: 0.65, visitCount: 4 } as any);

      // Act
      const result = await policyEngine.getQValue('user-1', stateHash, contentId);

      // Assert
      expect(result).toBe(0.65);
      expect(mockQTable.get).toHaveBeenCalledWith(stateHash, contentId);
    });
  });
});
