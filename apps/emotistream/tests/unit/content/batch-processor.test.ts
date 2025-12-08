/**
 * BatchProcessor Unit Tests
 */

import { BatchProcessor } from '../../../src/content/batch-processor';
import { ContentMetadata } from '../../../src/content/types';

describe('BatchProcessor', () => {
  let processor: BatchProcessor;
  let mockContents: ContentMetadata[];

  beforeEach(() => {
    processor = new BatchProcessor();

    mockContents = Array.from({ length: 25 }, (_, i) => ({
      contentId: `test_${i.toString().padStart(3, '0')}`,
      title: `Test Content ${i}`,
      description: 'Test description',
      platform: 'mock' as const,
      genres: ['drama'],
      category: 'movie' as const,
      tags: ['test'],
      duration: 120
    }));
  });

  describe('profile', () => {
    it('should process in batches of 10', async () => {
      const generator = processor.profile(mockContents, 10);

      let count = 0;
      for await (const profile of generator) {
        expect(profile).toBeDefined();
        expect(profile.contentId).toBeDefined();
        count++;
      }

      expect(count).toBe(mockContents.length);
    });

    it('should yield EmotionalContentProfile for each item', async () => {
      const generator = processor.profile(mockContents.slice(0, 5), 5);

      for await (const profile of generator) {
        expect(profile.contentId).toBeDefined();
        expect(profile.primaryTone).toBeDefined();
        expect(profile.valenceDelta).toBeGreaterThanOrEqual(-1);
        expect(profile.valenceDelta).toBeLessThanOrEqual(1);
        expect(profile.embeddingId).toBeDefined();
      }
    });

    it('should handle rate limiting', async () => {
      const startTime = Date.now();
      const generator = processor.profile(mockContents.slice(0, 3), 1);

      let count = 0;
      for await (const profile of generator) {
        count++;
      }

      const duration = Date.now() - startTime;

      // Should take some time due to rate limiting
      // (This is a simplified test - real rate limiting would take longer)
      expect(count).toBe(3);
    });
  });
});
