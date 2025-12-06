/**
 * FeedbackProcessor Unit Tests
 * TDD: Red-Green-Refactor Cycle
 */

import { FeedbackProcessor } from '../../../src/feedback/processor';
import { RewardCalculator } from '../../../src/feedback/reward-calculator';
import { ExperienceStore } from '../../../src/feedback/experience-store';
import { UserProfileManager } from '../../../src/feedback/user-profile';
import {
  EmotionDetector,
  RLPolicyEngine,
  EmotionalStateStore,
  RecommendationStore,
  FeedbackRequest,
  EmotionalState,
  ValidationError,
  NotFoundError,
} from '../../../src/feedback/types';

describe('FeedbackProcessor', () => {
  let processor: FeedbackProcessor;
  let mockEmotionDetector: jest.Mocked<EmotionDetector>;
  let mockRLEngine: jest.Mocked<RLPolicyEngine>;
  let mockExperienceStore: jest.Mocked<ExperienceStore>;
  let mockRewardCalculator: jest.Mocked<RewardCalculator>;
  let mockProfileManager: jest.Mocked<UserProfileManager>;
  let mockStateStore: jest.Mocked<EmotionalStateStore>;
  let mockRecommendationStore: jest.Mocked<RecommendationStore>;

  const mockStateBefore: EmotionalState = {
    valence: -0.6,
    arousal: 0.3,
    dominance: -0.2,
    confidence: 0.8,
    timestamp: new Date('2025-12-05T10:00:00Z'),
  };

  const mockStateAfter: EmotionalState = {
    valence: 0.4,
    arousal: -0.2,
    dominance: 0.1,
    confidence: 0.7,
    timestamp: new Date('2025-12-05T11:00:00Z'),
  };

  const mockDesiredState: EmotionalState = {
    valence: 0.6,
    arousal: -0.3,
    dominance: 0.2,
    confidence: 1.0,
    timestamp: new Date('2025-12-05T10:00:00Z'),
  };

  beforeEach(() => {
    // Create mocks
    mockEmotionDetector = {
      analyzeText: jest.fn(),
    } as any;

    mockRLEngine = {
      getQValue: jest.fn(),
      updateQValue: jest.fn(),
    } as any;

    mockExperienceStore = {
      store: jest.fn(),
    } as any;

    mockRewardCalculator = {
      calculate: jest.fn(),
      calculateCompletionBonus: jest.fn(),
      calculateInsights: jest.fn(),
    } as any;

    mockProfileManager = {
      update: jest.fn(),
    } as any;

    mockStateStore = {
      get: jest.fn(),
    } as any;

    mockRecommendationStore = {
      get: jest.fn(),
    } as any;

    processor = new FeedbackProcessor(
      mockEmotionDetector,
      mockRLEngine,
      mockExperienceStore,
      mockRewardCalculator,
      mockProfileManager,
      mockStateStore,
      mockRecommendationStore
    );
  });

  describe('processFeedback', () => {
    it('should calculate reward from feedback', async () => {
      const request: FeedbackRequest = {
        userId: 'user-123',
        contentId: 'content-456',
        emotionalStateId: 'state-789',
        postViewingState: {
          text: 'I feel much better now, very relaxed and happy',
        },
      };

      mockStateStore.get.mockResolvedValue(mockStateBefore);
      mockRecommendationStore.get.mockResolvedValue({
        targetEmotionalState: mockDesiredState,
        recommendedAt: new Date(),
        qValue: 0.5,
      });
      mockEmotionDetector.analyzeText.mockResolvedValue(mockStateAfter);
      mockRewardCalculator.calculate.mockReturnValue(0.85);
      mockRewardCalculator.calculateCompletionBonus.mockReturnValue(0.0);
      mockRewardCalculator.calculateInsights.mockReturnValue({
        directionAlignment: 0.95,
        magnitudeScore: 0.60,
        proximityBonus: 0.15,
        completionBonus: 0.0,
      });
      mockRLEngine.getQValue.mockResolvedValue(0.5);
      mockRLEngine.updateQValue.mockResolvedValue(true);
      mockExperienceStore.store.mockResolvedValue(true);
      mockProfileManager.update.mockResolvedValue(true);

      const response = await processor.processFeedback(request);

      expect(response).toBeDefined();
      expect(response.reward).toBeCloseTo(0.85, 2);
      expect(response.policyUpdated).toBe(true);
      expect(response.qValueAfter).toBeGreaterThan(response.qValueBefore);
      expect(mockRLEngine.updateQValue).toHaveBeenCalled();
    });

    it('should update RL policy with experience', async () => {
      const request: FeedbackRequest = {
        userId: 'user-123',
        contentId: 'content-456',
        emotionalStateId: 'state-789',
        postViewingState: { text: 'Great content!' },
      };

      mockStateStore.get.mockResolvedValue(mockStateBefore);
      mockRecommendationStore.get.mockResolvedValue({
        targetEmotionalState: mockDesiredState,
        recommendedAt: new Date(),
        qValue: 0.5,
      });
      mockEmotionDetector.analyzeText.mockResolvedValue(mockStateAfter);
      mockRewardCalculator.calculate.mockReturnValue(0.7);
      mockRewardCalculator.calculateCompletionBonus.mockReturnValue(0.0);
      mockRewardCalculator.calculateInsights.mockReturnValue({
        directionAlignment: 0.8,
        magnitudeScore: 0.5,
        proximityBonus: 0.1,
        completionBonus: 0.0,
      });
      mockRLEngine.getQValue.mockResolvedValue(0.4);
      mockRLEngine.updateQValue.mockResolvedValue(true);
      mockExperienceStore.store.mockResolvedValue(true);
      mockProfileManager.update.mockResolvedValue(true);

      await processor.processFeedback(request);

      expect(mockRLEngine.updateQValue).toHaveBeenCalledWith(
        mockStateBefore,
        'content-456',
        expect.any(Number)
      );
      const qValueAfter = (mockRLEngine.updateQValue as jest.Mock).mock.calls[0][2];
      expect(qValueAfter).toBeGreaterThan(0.4); // Should increase due to positive reward
    });

    it('should store experience in replay buffer', async () => {
      const request: FeedbackRequest = {
        userId: 'user-123',
        contentId: 'content-456',
        emotionalStateId: 'state-789',
        postViewingState: { explicitRating: 5 },
      };

      mockStateStore.get.mockResolvedValue(mockStateBefore);
      mockRecommendationStore.get.mockResolvedValue({
        targetEmotionalState: mockDesiredState,
        recommendedAt: new Date(),
        qValue: 0.5,
      });
      mockRewardCalculator.calculate.mockReturnValue(0.8);
      mockRewardCalculator.calculateCompletionBonus.mockReturnValue(0.0);
      mockRewardCalculator.calculateInsights.mockReturnValue({
        directionAlignment: 0.9,
        magnitudeScore: 0.6,
        proximityBonus: 0.15,
        completionBonus: 0.0,
      });
      mockRLEngine.getQValue.mockResolvedValue(0.5);
      mockRLEngine.updateQValue.mockResolvedValue(true);
      mockExperienceStore.store.mockResolvedValue(true);
      mockProfileManager.update.mockResolvedValue(true);

      await processor.processFeedback(request);

      expect(mockExperienceStore.store).toHaveBeenCalledWith(
        expect.objectContaining({
          userId: 'user-123',
          contentId: 'content-456',
          stateBeforeId: 'state-789',
          reward: expect.any(Number),
        })
      );
    });

    it('should return learning progress stats', async () => {
      const request: FeedbackRequest = {
        userId: 'user-123',
        contentId: 'content-456',
        emotionalStateId: 'state-789',
        postViewingState: { text: 'Good' },
      };

      mockStateStore.get.mockResolvedValue(mockStateBefore);
      mockRecommendationStore.get.mockResolvedValue({
        targetEmotionalState: mockDesiredState,
        recommendedAt: new Date(),
        qValue: 0.5,
      });
      mockEmotionDetector.analyzeText.mockResolvedValue(mockStateAfter);
      mockRewardCalculator.calculate.mockReturnValue(0.6);
      mockRewardCalculator.calculateCompletionBonus.mockReturnValue(0.0);
      mockRewardCalculator.calculateInsights.mockReturnValue({
        directionAlignment: 0.7,
        magnitudeScore: 0.5,
        proximityBonus: 0.1,
        completionBonus: 0.0,
      });
      mockRLEngine.getQValue.mockResolvedValue(0.4);
      mockRLEngine.updateQValue.mockResolvedValue(true);
      mockExperienceStore.store.mockResolvedValue(true);
      mockProfileManager.update.mockResolvedValue(true);

      const response = await processor.processFeedback(request);

      expect(response.insights).toBeDefined();
      expect(response.insights.directionAlignment).toBeDefined();
      expect(response.insights.magnitudeScore).toBeDefined();
      expect(response.insights.proximityBonus).toBeDefined();
      expect(response.message).toBeDefined();
    });

    it('should throw ValidationError for missing userId', async () => {
      const request: FeedbackRequest = {
        userId: '',
        contentId: 'content-456',
        emotionalStateId: 'state-789',
        postViewingState: { text: 'Good' },
      };

      await expect(processor.processFeedback(request)).rejects.toThrow(ValidationError);
    });

    it('should throw NotFoundError when state not found', async () => {
      const request: FeedbackRequest = {
        userId: 'user-123',
        contentId: 'content-456',
        emotionalStateId: 'state-999',
        postViewingState: { text: 'Good' },
      };

      mockStateStore.get.mockResolvedValue(null);

      await expect(processor.processFeedback(request)).rejects.toThrow(NotFoundError);
    });
  });

  describe('Feedback Type Handling', () => {
    it('should handle text feedback', async () => {
      const request: FeedbackRequest = {
        userId: 'user-123',
        contentId: 'content-456',
        emotionalStateId: 'state-789',
        postViewingState: {
          text: 'I feel amazing!',
        },
      };

      mockStateStore.get.mockResolvedValue(mockStateBefore);
      mockRecommendationStore.get.mockResolvedValue({
        targetEmotionalState: mockDesiredState,
        recommendedAt: new Date(),
        qValue: 0.5,
      });
      mockEmotionDetector.analyzeText.mockResolvedValue(mockStateAfter);
      mockRewardCalculator.calculate.mockReturnValue(0.8);
      mockRewardCalculator.calculateCompletionBonus.mockReturnValue(0.0);
      mockRewardCalculator.calculateInsights.mockReturnValue({
        directionAlignment: 0.9,
        magnitudeScore: 0.6,
        proximityBonus: 0.15,
        completionBonus: 0.0,
      });
      mockRLEngine.getQValue.mockResolvedValue(0.5);
      mockRLEngine.updateQValue.mockResolvedValue(true);
      mockExperienceStore.store.mockResolvedValue(true);
      mockProfileManager.update.mockResolvedValue(true);

      await processor.processFeedback(request);

      expect(mockEmotionDetector.analyzeText).toHaveBeenCalledWith('I feel amazing!');
    });

    it('should handle explicit rating feedback', async () => {
      const request: FeedbackRequest = {
        userId: 'user-123',
        contentId: 'content-456',
        emotionalStateId: 'state-789',
        postViewingState: {
          explicitRating: 5,
        },
      };

      mockStateStore.get.mockResolvedValue(mockStateBefore);
      mockRecommendationStore.get.mockResolvedValue({
        targetEmotionalState: mockDesiredState,
        recommendedAt: new Date(),
        qValue: 0.5,
      });
      mockRewardCalculator.calculate.mockReturnValue(0.8);
      mockRewardCalculator.calculateCompletionBonus.mockReturnValue(0.0);
      mockRewardCalculator.calculateInsights.mockReturnValue({
        directionAlignment: 0.9,
        magnitudeScore: 0.6,
        proximityBonus: 0.15,
        completionBonus: 0.0,
      });
      mockRLEngine.getQValue.mockResolvedValue(0.5);
      mockRLEngine.updateQValue.mockResolvedValue(true);
      mockExperienceStore.store.mockResolvedValue(true);
      mockProfileManager.update.mockResolvedValue(true);

      await processor.processFeedback(request);

      expect(mockEmotionDetector.analyzeText).not.toHaveBeenCalled();
    });

    it('should handle emoji feedback', async () => {
      const request: FeedbackRequest = {
        userId: 'user-123',
        contentId: 'content-456',
        emotionalStateId: 'state-789',
        postViewingState: {
          explicitEmoji: '😊',
        },
      };

      mockStateStore.get.mockResolvedValue(mockStateBefore);
      mockRecommendationStore.get.mockResolvedValue({
        targetEmotionalState: mockDesiredState,
        recommendedAt: new Date(),
        qValue: 0.5,
      });
      mockRewardCalculator.calculate.mockReturnValue(0.7);
      mockRewardCalculator.calculateCompletionBonus.mockReturnValue(0.0);
      mockRewardCalculator.calculateInsights.mockReturnValue({
        directionAlignment: 0.8,
        magnitudeScore: 0.5,
        proximityBonus: 0.12,
        completionBonus: 0.0,
      });
      mockRLEngine.getQValue.mockResolvedValue(0.5);
      mockRLEngine.updateQValue.mockResolvedValue(true);
      mockExperienceStore.store.mockResolvedValue(true);
      mockProfileManager.update.mockResolvedValue(true);

      await processor.processFeedback(request);

      expect(mockEmotionDetector.analyzeText).not.toHaveBeenCalled();
    });
  });

  describe('Viewing Details Integration', () => {
    it('should apply completion bonus for high completion rate', async () => {
      const request: FeedbackRequest = {
        userId: 'user-123',
        contentId: 'content-456',
        emotionalStateId: 'state-789',
        postViewingState: { text: 'Good' },
        viewingDetails: {
          completionRate: 0.95,
          durationSeconds: 1800,
        },
      };

      mockStateStore.get.mockResolvedValue(mockStateBefore);
      mockRecommendationStore.get.mockResolvedValue({
        targetEmotionalState: mockDesiredState,
        recommendedAt: new Date(),
        qValue: 0.5,
      });
      mockEmotionDetector.analyzeText.mockResolvedValue(mockStateAfter);
      mockRewardCalculator.calculate.mockReturnValue(0.6);
      mockRewardCalculator.calculateCompletionBonus.mockReturnValue(0.19);
      mockRewardCalculator.calculateInsights.mockReturnValue({
        directionAlignment: 0.7,
        magnitudeScore: 0.5,
        proximityBonus: 0.1,
        completionBonus: 0.19,
      });
      mockRLEngine.getQValue.mockResolvedValue(0.5);
      mockRLEngine.updateQValue.mockResolvedValue(true);
      mockExperienceStore.store.mockResolvedValue(true);
      mockProfileManager.update.mockResolvedValue(true);

      const response = await processor.processFeedback(request);

      expect(mockRewardCalculator.calculateCompletionBonus).toHaveBeenCalledWith({
        completionRate: 0.95,
        durationSeconds: 1800,
      });
      expect(response.insights.completionBonus).toBeCloseTo(0.19, 2);
    });
  });
});
