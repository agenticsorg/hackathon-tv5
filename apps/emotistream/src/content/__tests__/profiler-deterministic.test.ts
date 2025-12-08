/**
 * Tests for deterministic genre-based emotional profiling (Fix 2)
 */

import { ContentProfiler } from '../profiler';
import { ContentMetadata } from '../types';

describe('ContentProfiler - Deterministic Genre-Based Profiling', () => {
  let profiler: ContentProfiler;

  beforeEach(() => {
    profiler = new ContentProfiler();
  });

  describe('Genre-based emotional profiles', () => {
    it('should generate consistent profiles for same genres', async () => {
      const content: ContentMetadata = {
        contentId: 'test-1',
        title: 'Test Comedy',
        description: 'A funny movie',
        platform: 'mock',
        genres: ['comedy'],
        category: 'movie',
        tags: [],
        duration: 120,
      };

      const profile1 = await profiler.profile(content);
      const profile2 = await profiler.profile(content);

      // Same content should produce same emotional values
      expect(profile1.valenceDelta).toBe(profile2.valenceDelta);
      expect(profile1.arousalDelta).toBe(profile2.arousalDelta);
      expect(profile1.intensity).toBe(profile2.intensity);
      expect(profile1.complexity).toBe(profile2.complexity);
    });

    it('should use genre mapping for comedy', async () => {
      const content: ContentMetadata = {
        contentId: 'comedy-test',
        title: 'Comedy Movie',
        description: 'Funny content',
        platform: 'mock',
        genres: ['comedy'],
        category: 'movie',
        tags: [],
        duration: 120,
      };

      const profile = await profiler.profile(content);

      // Comedy should have positive valenceDelta (0.5)
      expect(profile.valenceDelta).toBe(0.5);
      expect(profile.arousalDelta).toBe(0.2);
      expect(profile.intensity).toBe(0.6);
    });

    it('should use genre mapping for horror', async () => {
      const content: ContentMetadata = {
        contentId: 'horror-test',
        title: 'Horror Movie',
        description: 'Scary content',
        platform: 'mock',
        genres: ['horror'],
        category: 'movie',
        tags: [],
        duration: 120,
      };

      const profile = await profiler.profile(content);

      // Horror should have negative valenceDelta (-0.3) and high arousal (0.7)
      expect(profile.valenceDelta).toBe(-0.3);
      expect(profile.arousalDelta).toBe(0.7);
      expect(profile.intensity).toBe(0.9);
    });

    it('should average multiple genres', async () => {
      const content: ContentMetadata = {
        contentId: 'mixed-test',
        title: 'Action Comedy',
        description: 'Funny action movie',
        platform: 'mock',
        genres: ['action', 'comedy'],
        category: 'movie',
        tags: [],
        duration: 120,
      };

      const profile = await profiler.profile(content);

      // Average of action (0.3) and comedy (0.5) = 0.4
      expect(profile.valenceDelta).toBeCloseTo(0.4, 2);
      // Average of action (0.6) and comedy (0.2) = 0.4
      expect(profile.arousalDelta).toBeCloseTo(0.4, 2);
      // Average of action (0.8) and comedy (0.6) = 0.7
      expect(profile.intensity).toBeCloseTo(0.7, 2);
    });

    it('should handle case-insensitive genres', async () => {
      const content: ContentMetadata = {
        contentId: 'case-test',
        title: 'Drama Movie',
        description: 'Dramatic content',
        platform: 'mock',
        genres: ['Drama', 'THRILLER'],
        category: 'movie',
        tags: [],
        duration: 120,
      };

      const profile = await profiler.profile(content);

      // Should match lowercased genres: drama (0.1) and thriller (-0.1) = 0.0
      expect(profile.valenceDelta).toBeCloseTo(0.0, 2);
      // drama (0.1) and thriller (0.5) = 0.3
      expect(profile.arousalDelta).toBeCloseTo(0.3, 2);
    });

    it('should use neutral defaults for unknown genres', async () => {
      const content: ContentMetadata = {
        contentId: 'unknown-test',
        title: 'Unknown Genre',
        description: 'No matching genres',
        platform: 'mock',
        genres: ['unknown-genre', 'fake-genre'],
        category: 'movie',
        tags: [],
        duration: 120,
      };

      const profile = await profiler.profile(content);

      // Should use neutral defaults
      expect(profile.valenceDelta).toBe(0.2);
      expect(profile.arousalDelta).toBe(0.1);
      expect(profile.intensity).toBe(0.5);
    });

    it('should use neutral defaults for empty genres', async () => {
      const content: ContentMetadata = {
        contentId: 'empty-test',
        title: 'No Genres',
        description: 'Content without genres',
        platform: 'mock',
        genres: [],
        category: 'movie',
        tags: [],
        duration: 120,
      };

      const profile = await profiler.profile(content);

      // Should use neutral defaults
      expect(profile.valenceDelta).toBe(0.2);
      expect(profile.arousalDelta).toBe(0.1);
      expect(profile.intensity).toBe(0.5);
    });
  });

  describe('Complexity calculation', () => {
    it('should calculate complexity based on genre count', async () => {
      const singleGenre: ContentMetadata = {
        contentId: 'single',
        title: 'Single Genre',
        description: 'One genre',
        platform: 'mock',
        genres: ['action'],
        category: 'movie',
        tags: [],
        duration: 120,
      };

      const multiGenre: ContentMetadata = {
        contentId: 'multi',
        title: 'Multiple Genres',
        description: 'Many genres',
        platform: 'mock',
        genres: ['action', 'comedy', 'drama'],
        category: 'movie',
        tags: [],
        duration: 120,
      };

      const profile1 = await profiler.profile(singleGenre);
      const profile2 = await profiler.profile(multiGenre);

      // More genres should have higher complexity
      expect(profile2.complexity).toBeGreaterThan(profile1.complexity);

      // Single genre: 0.3 + (1 * 0.15) = 0.45
      expect(profile1.complexity).toBeCloseTo(0.45, 2);

      // Three genres: 0.3 + (3 * 0.15) = 0.75
      expect(profile2.complexity).toBeCloseTo(0.75, 2);
    });

    it('should cap complexity at 0.9', async () => {
      const manyGenres: ContentMetadata = {
        contentId: 'many',
        title: 'Many Genres',
        description: 'Lots of genres',
        platform: 'mock',
        genres: ['action', 'comedy', 'drama', 'thriller', 'horror', 'romance'],
        category: 'movie',
        tags: [],
        duration: 120,
      };

      const profile = await profiler.profile(manyGenres);

      // Should be capped at 0.9
      expect(profile.complexity).toBeLessThanOrEqual(0.9);
      expect(profile.complexity).toBe(0.9);
    });
  });

  describe('Target states', () => {
    it('should derive target states from emotional profile', async () => {
      const content: ContentMetadata = {
        contentId: 'target-test',
        title: 'Action Movie',
        description: 'Action content',
        platform: 'mock',
        genres: ['action'],
        category: 'movie',
        tags: [],
        duration: 120,
      };

      const profile = await profiler.profile(content);

      // Target states should be deterministic based on valenceDelta/arousalDelta
      expect(profile.targetStates).toHaveLength(2);

      // First target state is 50% of delta values
      expect(profile.targetStates[0].currentValence).toBeCloseTo(0.3 * 0.5, 2);
      expect(profile.targetStates[0].currentArousal).toBeCloseTo(0.6 * 0.5, 2);

      // Second target state is 30% of delta values
      expect(profile.targetStates[1].currentValence).toBeCloseTo(0.3 * 0.3, 2);
      expect(profile.targetStates[1].currentArousal).toBeCloseTo(0.6 * 0.3, 2);
    });
  });
});
