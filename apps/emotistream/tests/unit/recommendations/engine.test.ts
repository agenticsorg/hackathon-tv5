import { RecommendationEngine } from '../../../src/recommendations/engine';
import { RLPolicyEngine } from '../../../src/rl/policy-engine';
import { ContentProfiler } from '../../../src/content/profiler';
import { EmotionDetector } from '../../../src/emotion/detector';

describe('RecommendationEngine', () => {
  let engine: RecommendationEngine;
  let mockRLPolicy: jest.Mocked<RLPolicyEngine>;
  let mockContentProfiler: jest.Mocked<ContentProfiler>;
  let mockEmotionDetector: jest.Mocked<EmotionDetector>;

  beforeEach(() => {
    // Create mock dependencies
    mockRLPolicy = {
      getQValue: jest.fn(),
      updateQValue: jest.fn(),
      getBestAction: jest.fn(),
    } as any;

    mockContentProfiler = {
      getProfile: jest.fn(),
      searchByTransition: jest.fn(),
    } as any;

    mockEmotionDetector = {
      getState: jest.fn(),
    } as any;

    engine = new RecommendationEngine(
      mockRLPolicy,
      mockContentProfiler,
      mockEmotionDetector
    );
  });

  describe('recommend', () => {
    it('should return top-k recommendations', async () => {
      // Arrange
      const userId = 'user123';
      const currentState = {
        id: 'state1',
        userId,
        timestamp: Date.now(),
        valence: -0.4,
        arousal: 0.6,
        stressLevel: 0.8,
        dominance: 0.0,
        rawMetrics: {},
      };

      const mockProfile = {
        contentId: 'content1',
        title: 'Calming Nature Documentary',
        platform: 'Netflix',
        valenceDelta: 0.7,
        arousalDelta: -0.6,
        stressReduction: 0.5,
        duration: 50,
        genres: ['Documentary', 'Nature'],
        embedding: new Float32Array(1536),
      };

      mockEmotionDetector.getState.mockResolvedValue(currentState);
      mockContentProfiler.searchByTransition.mockResolvedValue([
        { contentId: 'content1', profile: mockProfile, similarity: 0.89, distance: 0.22 },
      ]);
      mockContentProfiler.getProfile.mockResolvedValue(mockProfile);
      mockRLPolicy.getQValue.mockResolvedValue(0.82);

      // Act
      const recommendations = await engine.recommend(userId, currentState, 5);

      // Assert
      expect(recommendations).toHaveLength(1);
      expect(recommendations[0].contentId).toBe('content1');
      expect(mockEmotionDetector.getState).not.toHaveBeenCalled(); // We passed state directly
      expect(mockContentProfiler.searchByTransition).toHaveBeenCalled();
    });

    it('should use hybrid ranking (70% Q + 30% similarity)', async () => {
      // Arrange
      const userId = 'user123';
      const currentState = {
        id: 'state1',
        userId,
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
      };

      const profile2 = {
        contentId: 'content2',
        title: 'Content 2',
        platform: 'YouTube',
        valenceDelta: 0.6,
        arousalDelta: -0.4,
        stressReduction: 0.5,
        duration: 30,
        genres: ['Documentary'],
        embedding: new Float32Array(1536),
      };

      mockContentProfiler.searchByTransition.mockResolvedValue([
        { contentId: 'content1', profile: profile1, similarity: 0.9, distance: 0.2 },
        { contentId: 'content2', profile: profile2, similarity: 0.6, distance: 0.8 },
      ]);

      // content1: Q=0.3, sim=0.9 -> hybrid = (0.3 * 0.7) + (0.9 * 0.3) = 0.48
      // content2: Q=0.8, sim=0.6 -> hybrid = (0.8 * 0.7) + (0.6 * 0.3) = 0.74
      mockRLPolicy.getQValue.mockImplementation(async (uid, state, action) => {
        if (action.includes('content1')) return 0.3;
        if (action.includes('content2')) return 0.8;
        return 0.5;
      });

      // Act
      const recommendations = await engine.recommend(userId, currentState, 5);

      // Assert
      expect(recommendations).toHaveLength(2);
      // content2 should rank higher due to better Q-value
      expect(recommendations[0].contentId).toBe('content2');
      expect(recommendations[1].contentId).toBe('content1');
    });

    it('should include predicted outcomes', async () => {
      // Arrange
      const userId = 'user123';
      const currentState = {
        id: 'state1',
        userId,
        timestamp: Date.now(),
        valence: -0.3,
        arousal: 0.5,
        stressLevel: 0.7,
        dominance: 0.0,
        rawMetrics: {},
      };

      const mockProfile = {
        contentId: 'content1',
        title: 'Relaxing Content',
        platform: 'Netflix',
        valenceDelta: 0.6,
        arousalDelta: -0.5,
        stressReduction: 0.4,
        duration: 40,
        genres: ['Nature'],
        embedding: new Float32Array(1536),
        totalWatches: 100,
        outcomeVariance: 0.1,
      };

      mockContentProfiler.searchByTransition.mockResolvedValue([
        { contentId: 'content1', profile: mockProfile, similarity: 0.85, distance: 0.3 },
      ]);
      mockRLPolicy.getQValue.mockResolvedValue(0.75);

      // Act
      const recommendations = await engine.recommend(userId, currentState, 5);

      // Assert
      expect(recommendations[0].predictedOutcome).toBeDefined();
      expect(recommendations[0].predictedOutcome.postViewingValence).toBeCloseTo(0.3, 1);
      expect(recommendations[0].predictedOutcome.postViewingArousal).toBeCloseTo(0.0, 1);
      expect(recommendations[0].predictedOutcome.postViewingStress).toBeCloseTo(0.3, 1);
      expect(recommendations[0].predictedOutcome.confidence).toBeGreaterThan(0.5);
    });

    it('should generate reasoning for each recommendation', async () => {
      // Arrange
      const userId = 'user123';
      const currentState = {
        id: 'state1',
        userId,
        timestamp: Date.now(),
        valence: -0.4,
        arousal: 0.6,
        stressLevel: 0.8,
        dominance: 0.0,
        rawMetrics: {},
      };

      const mockProfile = {
        contentId: 'content1',
        title: 'Nature Documentary',
        platform: 'Netflix',
        valenceDelta: 0.7,
        arousalDelta: -0.6,
        stressReduction: 0.6,
        duration: 50,
        genres: ['Documentary'],
        embedding: new Float32Array(1536),
      };

      mockContentProfiler.searchByTransition.mockResolvedValue([
        { contentId: 'content1', profile: mockProfile, similarity: 0.88, distance: 0.24 },
      ]);
      mockRLPolicy.getQValue.mockResolvedValue(0.85);

      // Act
      const recommendations = await engine.recommend(userId, currentState, 5);

      // Assert
      expect(recommendations[0].reasoning).toBeDefined();
      expect(recommendations[0].reasoning).toContain('feel');
      expect(recommendations[0].reasoning.length).toBeGreaterThan(50);
    });

    it('should mark exploration items', async () => {
      // Arrange
      const userId = 'user123';
      const currentState = {
        id: 'state1',
        userId,
        timestamp: Date.now(),
        valence: 0.0,
        arousal: 0.0,
        stressLevel: 0.5,
        dominance: 0.0,
        rawMetrics: {},
      };

      const mockProfile = {
        contentId: 'content1',
        title: 'Unknown Content',
        platform: 'YouTube',
        valenceDelta: 0.4,
        arousalDelta: -0.2,
        stressReduction: 0.3,
        duration: 30,
        genres: ['Educational'],
        embedding: new Float32Array(1536),
      };

      mockContentProfiler.searchByTransition.mockResolvedValue([
        { contentId: 'content1', profile: mockProfile, similarity: 0.75, distance: 0.5 },
      ]);
      // Return null to simulate unexplored content
      mockRLPolicy.getQValue.mockResolvedValue(null);

      // Act
      const recommendations = await engine.recommend(userId, currentState, 5);

      // Assert
      expect(recommendations[0].isExploration).toBe(true);
      expect(recommendations[0].qValue).toBeCloseTo(0.5, 1); // Default Q-value
    });

    it('should throw error when state is not found', async () => {
      // Arrange
      mockEmotionDetector.getState.mockResolvedValue(null);

      // Act & Assert
      await expect(
        engine.recommendById('user123', 'invalid-state-id', 5)
      ).rejects.toThrow('Emotional state not found');
    });

    it('should handle empty search results gracefully', async () => {
      // Arrange
      const userId = 'user123';
      const currentState = {
        id: 'state1',
        userId,
        timestamp: Date.now(),
        valence: 0.0,
        arousal: 0.0,
        stressLevel: 0.5,
        dominance: 0.0,
        rawMetrics: {},
      };

      mockContentProfiler.searchByTransition.mockResolvedValue([]);

      // Act
      const recommendations = await engine.recommend(userId, currentState, 5);

      // Assert
      expect(recommendations).toHaveLength(0);
    });
  });
});
