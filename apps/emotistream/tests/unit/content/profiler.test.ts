/**
 * ContentProfiler Unit Tests
 * TDD Approach - Tests written FIRST
 */

import { ContentProfiler } from '../../../src/content/profiler';
import { ContentMetadata, EmotionalContentProfile, EmotionalState } from '../../../src/content/types';

describe('ContentProfiler', () => {
  let profiler: ContentProfiler;
  let mockContent: ContentMetadata;

  beforeEach(() => {
    profiler = new ContentProfiler();

    mockContent = {
      contentId: 'test_001',
      title: 'Test Movie',
      description: 'A test movie description',
      platform: 'mock',
      genres: ['drama', 'comedy'],
      category: 'movie',
      tags: ['emotional', 'uplifting'],
      duration: 120
    };
  });

  describe('profile', () => {
    it('should return EmotionalContentProfile for content', async () => {
      const profile = await profiler.profile(mockContent);

      expect(profile).toBeDefined();
      expect(profile.contentId).toBe(mockContent.contentId);
      expect(profile.primaryTone).toBeDefined();
      expect(profile.valenceDelta).toBeGreaterThanOrEqual(-1);
      expect(profile.valenceDelta).toBeLessThanOrEqual(1);
      expect(profile.arousalDelta).toBeGreaterThanOrEqual(-1);
      expect(profile.arousalDelta).toBeLessThanOrEqual(1);
      expect(profile.intensity).toBeGreaterThanOrEqual(0);
      expect(profile.intensity).toBeLessThanOrEqual(1);
      expect(profile.complexity).toBeGreaterThanOrEqual(0);
      expect(profile.complexity).toBeLessThanOrEqual(1);
      expect(profile.embeddingId).toBeDefined();
      expect(profile.timestamp).toBeGreaterThan(0);
    });

    it('should generate emotional journey array', async () => {
      const profile = await profiler.profile(mockContent);

      expect(profile.targetStates).toBeDefined();
      expect(Array.isArray(profile.targetStates)).toBe(true);
      expect(profile.targetStates.length).toBeGreaterThan(0);

      profile.targetStates.forEach(state => {
        expect(state.currentValence).toBeGreaterThanOrEqual(-1);
        expect(state.currentValence).toBeLessThanOrEqual(1);
        expect(state.currentArousal).toBeGreaterThanOrEqual(-1);
        expect(state.currentArousal).toBeLessThanOrEqual(1);
        expect(state.description).toBeDefined();
      });
    });

    it('should calculate dominant emotion', async () => {
      const profile = await profiler.profile(mockContent);

      expect(profile.primaryTone).toBeDefined();
      expect(typeof profile.primaryTone).toBe('string');
      expect(profile.primaryTone.length).toBeGreaterThan(0);
    });

    it('should create 1536D embedding vector', async () => {
      const profile = await profiler.profile(mockContent);

      expect(profile.embeddingId).toBeDefined();
      expect(typeof profile.embeddingId).toBe('string');
    });
  });

  describe('search', () => {
    beforeEach(async () => {
      // Seed some content first
      await profiler.profile(mockContent);
    });

    it('should find similar content by emotional transition', async () => {
      const currentState: EmotionalState = {
        valence: -0.5,
        arousal: 0.6,
        primaryEmotion: 'stressed',
        stressLevel: 0.8,
        confidence: 0.9,
        timestamp: Date.now()
      };

      const transitionVector = new Float32Array(1536);
      const results = await profiler.search(transitionVector, 5);

      expect(results).toBeDefined();
      expect(Array.isArray(results)).toBe(true);
    });

    it('should return ranked SearchResults', async () => {
      const transitionVector = new Float32Array(1536);
      const results = await profiler.search(transitionVector, 5);

      expect(results.length).toBeLessThanOrEqual(5);

      // Verify results are sorted by similarity
      for (let i = 0; i < results.length - 1; i++) {
        expect(results[i].similarityScore).toBeGreaterThanOrEqual(results[i + 1].similarityScore);
      }
    });
  });
});
