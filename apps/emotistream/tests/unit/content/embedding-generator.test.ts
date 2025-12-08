/**
 * EmbeddingGenerator Unit Tests
 */

import { EmbeddingGenerator } from '../../../src/content/embedding-generator';
import { EmotionalContentProfile, ContentMetadata } from '../../../src/content/types';

describe('EmbeddingGenerator', () => {
  let generator: EmbeddingGenerator;
  let mockProfile: EmotionalContentProfile;
  let mockContent: ContentMetadata;

  beforeEach(() => {
    generator = new EmbeddingGenerator();

    mockProfile = {
      contentId: 'test_001',
      primaryTone: 'uplifting',
      valenceDelta: 0.6,
      arousalDelta: 0.2,
      intensity: 0.7,
      complexity: 0.5,
      targetStates: [
        {
          currentValence: -0.3,
          currentArousal: 0.1,
          description: 'neutral to happy'
        }
      ],
      embeddingId: '',
      timestamp: Date.now()
    };

    mockContent = {
      contentId: 'test_001',
      title: 'Test Content',
      description: 'Test description',
      platform: 'mock',
      genres: ['comedy', 'drama'],
      category: 'movie',
      tags: ['uplifting', 'emotional'],
      duration: 120
    };
  });

  describe('generate', () => {
    it('should generate 1536D embedding', () => {
      const embedding = generator.generate(mockProfile, mockContent);

      expect(embedding).toBeDefined();
      expect(embedding.length).toBe(1536);
      expect(embedding).toBeInstanceOf(Float32Array);
    });

    it('should normalize embedding to unit length', () => {
      const embedding = generator.generate(mockProfile, mockContent);

      // Calculate magnitude
      let magnitude = 0;
      for (let i = 0; i < embedding.length; i++) {
        magnitude += embedding[i] * embedding[i];
      }
      magnitude = Math.sqrt(magnitude);

      expect(magnitude).toBeCloseTo(1.0, 5);
    });

    it('should encode valence delta correctly', () => {
      const embedding = generator.generate(mockProfile, mockContent);

      // Check that segment 2 (256-383) has encoded valence
      const segment2 = Array.from(embedding.slice(256, 384));
      const maxValue = Math.max(...segment2);

      expect(maxValue).toBeGreaterThan(0);
    });

    it('should encode arousal delta correctly', () => {
      const embedding = generator.generate(mockProfile, mockContent);

      // Check that segment 2 (384-511) has encoded arousal
      const segment3 = Array.from(embedding.slice(384, 512));
      const maxValue = Math.max(...segment3);

      expect(maxValue).toBeGreaterThan(0);
    });

    it('should encode all segments', () => {
      const embedding = generator.generate(mockProfile, mockContent);

      // Verify not all zeros
      const nonZeroCount = Array.from(embedding).filter(v => v !== 0).length;
      expect(nonZeroCount).toBeGreaterThan(0);
    });
  });
});
