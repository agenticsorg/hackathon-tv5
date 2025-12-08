import { HybridRanker } from '../../../src/recommendations/ranker';
import { RLPolicyEngine } from '../../../src/rl/policy-engine';

describe('HybridRanker', () => {
  let ranker: HybridRanker;
  let mockRLPolicy: jest.Mocked<RLPolicyEngine>;

  beforeEach(() => {
    mockRLPolicy = {
      getQValue: jest.fn(),
      updateQValue: jest.fn(),
      getBestAction: jest.fn(),
    } as any;

    ranker = new HybridRanker(mockRLPolicy);
  });

  describe('rank', () => {
    it('should rank by combined score (70% Q + 30% similarity)', async () => {
      // Arrange
      const userId = 'user123';
      const stateHash = 'v:3:a:5:s:2';
      const candidates = [
        {
          contentId: 'A',
          profile: {
            contentId: 'A',
            title: 'Content A',
            platform: 'Netflix',
            valenceDelta: 0.5,
            arousalDelta: -0.3,
            stressReduction: 0.4,
            duration: 45,
            genres: ['Drama'],
            embedding: new Float32Array(1536),
          },
          similarity: 0.9,
          distance: 0.2,
        },
        {
          contentId: 'B',
          profile: {
            contentId: 'B',
            title: 'Content B',
            platform: 'YouTube',
            valenceDelta: 0.6,
            arousalDelta: -0.4,
            stressReduction: 0.5,
            duration: 30,
            genres: ['Documentary'],
            embedding: new Float32Array(1536),
          },
          similarity: 0.6,
          distance: 0.8,
        },
        {
          contentId: 'C',
          profile: {
            contentId: 'C',
            title: 'Content C',
            platform: 'Hulu',
            valenceDelta: 0.4,
            arousalDelta: -0.2,
            stressReduction: 0.3,
            duration: 50,
            genres: ['Nature'],
            embedding: new Float32Array(1536),
          },
          similarity: 0.7,
          distance: 0.6,
        },
      ];

      // Mock Q-values
      mockRLPolicy.getQValue.mockImplementation(async (uid, state, action) => {
        if (action.includes('A')) return 0.3;
        if (action.includes('B')) return 0.8;
        if (action.includes('C')) return 0.7;
        return 0.5;
      });

      // Act
      const ranked = await ranker.rank(userId, candidates, stateHash);

      // Assert
      // B: (0.8 * 0.7) + (0.6 * 0.3) = 0.74
      // C: (0.7 * 0.7) + (0.7 * 0.3) = 0.70
      // A: (0.3 * 0.7) + (0.9 * 0.3) = 0.48
      expect(ranked).toHaveLength(3);
      expect(ranked[0].contentId).toBe('B');
      expect(ranked[1].contentId).toBe('C');
      expect(ranked[2].contentId).toBe('A');
      expect(ranked[0].hybridScore).toBeCloseTo(0.74, 2);
    });

    it('should use default Q-value for unexplored content', async () => {
      // Arrange
      const userId = 'user123';
      const stateHash = 'v:5:a:5:s:2';
      const candidates = [
        {
          contentId: 'unexplored',
          profile: {
            contentId: 'unexplored',
            title: 'Unexplored Content',
            platform: 'Netflix',
            valenceDelta: 0.5,
            arousalDelta: -0.3,
            stressReduction: 0.4,
            duration: 40,
            genres: ['Unknown'],
            embedding: new Float32Array(1536),
          },
          similarity: 0.8,
          distance: 0.4,
        },
      ];

      mockRLPolicy.getQValue.mockResolvedValue(null);

      // Act
      const ranked = await ranker.rank(userId, candidates, stateHash);

      // Assert
      expect(ranked[0].qValue).toBe(0.5); // Default Q-value
      expect(ranked[0].isExploration).toBe(true);
    });

    it('should normalize Q-values to [0, 1]', async () => {
      // Arrange
      const userId = 'user123';
      const stateHash = 'v:5:a:5:s:2';
      const candidates = [
        {
          contentId: 'content1',
          profile: {
            contentId: 'content1',
            title: 'Content 1',
            platform: 'Netflix',
            valenceDelta: 0.5,
            arousalDelta: -0.3,
            stressReduction: 0.4,
            duration: 45,
            genres: ['Drama'],
            embedding: new Float32Array(1536),
          },
          similarity: 0.8,
          distance: 0.4,
        },
      ];

      // Q-value in [-1, 1] range
      mockRLPolicy.getQValue.mockResolvedValue(-0.5);

      // Act
      const ranked = await ranker.rank(userId, candidates, stateHash);

      // Assert
      // Normalized: (-0.5 + 1.0) / 2.0 = 0.25
      expect(ranked[0].qValueNormalized).toBeCloseTo(0.25, 2);
    });
  });

  describe('calculateOutcomeAlignment', () => {
    it('should return high alignment for matching deltas', () => {
      // Arrange
      const profile = {
        contentId: 'content1',
        title: 'Content 1',
        platform: 'Netflix',
        valenceDelta: 0.8,
        arousalDelta: -0.6,
        stressReduction: 0.5,
        duration: 45,
        genres: ['Drama'],
        embedding: new Float32Array(1536),
      };

      const desiredState = {
        valence: 0.8,
        arousal: -0.6,
      };

      // Act
      const alignment = ranker.calculateOutcomeAlignment(profile, desiredState);

      // Assert
      expect(alignment).toBeGreaterThan(0.9);
    });

    it('should return low alignment for opposite deltas', () => {
      // Arrange
      const profile = {
        contentId: 'content1',
        title: 'Content 1',
        platform: 'Netflix',
        valenceDelta: 0.8,
        arousalDelta: -0.6,
        stressReduction: 0.5,
        duration: 45,
        genres: ['Drama'],
        embedding: new Float32Array(1536),
      };

      const desiredState = {
        valence: -0.8,
        arousal: 0.6,
      };

      // Act
      const alignment = ranker.calculateOutcomeAlignment(profile, desiredState);

      // Assert
      expect(alignment).toBeLessThan(0.3);
    });

    it('should handle zero magnitude gracefully', () => {
      // Arrange
      const profile = {
        contentId: 'content1',
        title: 'Content 1',
        platform: 'Netflix',
        valenceDelta: 0.0,
        arousalDelta: 0.0,
        stressReduction: 0.5,
        duration: 45,
        genres: ['Drama'],
        embedding: new Float32Array(1536),
      };

      const desiredState = {
        valence: 0.5,
        arousal: 0.3,
      };

      // Act
      const alignment = ranker.calculateOutcomeAlignment(profile, desiredState);

      // Assert
      expect(alignment).toBe(0.5); // Neutral alignment
    });
  });
});
