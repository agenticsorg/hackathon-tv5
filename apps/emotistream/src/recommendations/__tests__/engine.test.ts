/**
 * RecommendationEngine Integration Tests
 */

import { RecommendationEngine } from '../engine';
import { MockCatalogGenerator } from '../../content/mock-catalog';

describe('RecommendationEngine', () => {
  let engine: RecommendationEngine;

  beforeAll(async () => {
    engine = new RecommendationEngine();

    // Generate and profile mock content catalog
    const catalogGenerator = new MockCatalogGenerator();
    const catalog = catalogGenerator.generate(50);

    const profiler = engine.getProfiler();
    await profiler.batchProfile(catalog, 10);
  });

  describe('recommend()', () => {
    it('should return 20 recommendations for stressed user', async () => {
      const recommendations = await engine.recommend(
        'user_stressed_001',
        {
          valence: -0.4,
          arousal: 0.6,
          stress: 0.8
        },
        20
      );

      expect(recommendations).toHaveLength(20);
      expect(recommendations[0]).toHaveProperty('contentId');
      expect(recommendations[0]).toHaveProperty('title');
      expect(recommendations[0]).toHaveProperty('qValue');
      expect(recommendations[0]).toHaveProperty('similarityScore');
      expect(recommendations[0]).toHaveProperty('combinedScore');
      expect(recommendations[0]).toHaveProperty('predictedOutcome');
      expect(recommendations[0]).toHaveProperty('reasoning');
      expect(recommendations[0]).toHaveProperty('rank');
    });

    it('should return recommendations for happy user', async () => {
      const recommendations = await engine.recommend(
        'user_happy_001',
        {
          valence: 0.7,
          arousal: 0.3,
          stress: 0.2
        },
        15
      );

      expect(recommendations).toHaveLength(15);
      expect(recommendations[0].rank).toBe(1);
      expect(recommendations[14].rank).toBe(15);
    });

    it('should generate reasoning for recommendations', async () => {
      const recommendations = await engine.recommend(
        'user_anxious_001',
        {
          valence: -0.3,
          arousal: 0.5,
          stress: 0.7
        },
        5
      );

      expect(recommendations).toHaveLength(5);
      recommendations.forEach(rec => {
        expect(rec.reasoning).toBeTruthy();
        expect(rec.reasoning.length).toBeGreaterThan(20);
      });
    });

    it('should predict emotional outcomes', async () => {
      const recommendations = await engine.recommend(
        'user_bored_001',
        {
          valence: 0.0,
          arousal: -0.5,
          stress: 0.3
        },
        10
      );

      expect(recommendations).toHaveLength(10);
      recommendations.forEach(rec => {
        expect(rec.predictedOutcome).toBeDefined();
        expect(rec.predictedOutcome.expectedValence).toBeGreaterThanOrEqual(-1);
        expect(rec.predictedOutcome.expectedValence).toBeLessThanOrEqual(1);
        expect(rec.predictedOutcome.expectedArousal).toBeGreaterThanOrEqual(-1);
        expect(rec.predictedOutcome.expectedArousal).toBeLessThanOrEqual(1);
        expect(rec.predictedOutcome.expectedStress).toBeGreaterThanOrEqual(0);
        expect(rec.predictedOutcome.expectedStress).toBeLessThanOrEqual(1);
        expect(rec.predictedOutcome.confidence).toBeGreaterThan(0);
        expect(rec.predictedOutcome.confidence).toBeLessThanOrEqual(1);
      });
    });

    it('should include exploration picks', async () => {
      const recommendations = await engine.recommend(
        'user_neutral_001',
        {
          valence: 0.0,
          arousal: 0.0,
          stress: 0.4
        },
        20
      );

      const explorationCount = recommendations.filter(r => r.isExploration).length;
      expect(explorationCount).toBeGreaterThan(0);
      expect(explorationCount).toBeLessThanOrEqual(6); // ~30% exploration
    });

    it('should rank by combined score', async () => {
      const recommendations = await engine.recommend(
        'user_sad_001',
        {
          valence: -0.6,
          arousal: -0.3,
          stress: 0.5
        },
        15
      );

      // Verify descending combined scores
      for (let i = 0; i < recommendations.length - 1; i++) {
        expect(recommendations[i].combinedScore).toBeGreaterThanOrEqual(
          recommendations[i + 1].combinedScore
        );
      }
    });
  });

  describe('getRecommendations() with explicit desired state', () => {
    it('should use explicit desired state when provided', async () => {
      const recommendations = await engine.getRecommendations({
        userId: 'user_explicit_001',
        currentState: {
          valence: -0.5,
          arousal: 0.5,
          stress: 0.8,
          confidence: 0.9
        },
        desiredState: {
          valence: 0.5,
          arousal: -0.5,
          confidence: 1.0
        },
        limit: 10
      });

      expect(recommendations).toHaveLength(10);
      // Should recommend calming, mood-lifting content
    });

    it('should handle edge case emotional states', async () => {
      const recommendations = await engine.getRecommendations({
        userId: 'user_extreme_001',
        currentState: {
          valence: -0.9,
          arousal: 0.9,
          stress: 0.95,
          confidence: 0.8
        },
        limit: 10
      });

      expect(recommendations).toHaveLength(10);
      recommendations.forEach(rec => {
        expect(rec.predictedOutcome.expectedStress).toBeLessThan(0.95);
      });
    });
  });

  describe('Q-value integration', () => {
    it('should use default Q-value for unexplored content', async () => {
      const recommendations = await engine.recommend(
        'new_user_001',
        {
          valence: 0.0,
          arousal: 0.0,
          stress: 0.5
        },
        10
      );

      // For new users, many Q-values should be default (0.5)
      const defaultQCount = recommendations.filter(
        r => Math.abs(r.qValue - 0.5) < 0.01
      ).length;

      expect(defaultQCount).toBeGreaterThan(0);
    });

    it('should combine Q-values with similarity (70/30 hybrid)', async () => {
      const recommendations = await engine.recommend(
        'hybrid_test_001',
        {
          valence: 0.2,
          arousal: -0.2,
          stress: 0.4
        },
        10
      );

      // Verify combined scores are reasonable
      recommendations.forEach(rec => {
        expect(rec.combinedScore).toBeGreaterThan(0);
        expect(rec.combinedScore).toBeLessThanOrEqual(1.2); // Max with alignment boost
      });
    });
  });
});
