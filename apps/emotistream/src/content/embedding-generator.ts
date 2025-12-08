/**
 * EmbeddingGenerator - Creates 1536D embeddings from emotional profiles
 */

import { EmotionalContentProfile, ContentMetadata } from './types';

export class EmbeddingGenerator {
  private readonly DIMENSIONS = 1536;
  private readonly toneMap: Map<string, number>;
  private readonly genreMap: Map<string, number>;
  private readonly categoryMap: Map<string, number>;

  constructor() {
    // Initialize tone mapping (256 possible tones)
    this.toneMap = new Map([
      ['uplifting', 0], ['calming', 32], ['thrilling', 64], ['melancholic', 96],
      ['serene', 128], ['dramatic', 160], ['cathartic', 192], ['neutral', 224]
    ]);

    // Initialize genre mapping (128 slots)
    this.genreMap = new Map([
      ['drama', 0], ['comedy', 1], ['thriller', 2], ['romance', 3],
      ['action', 4], ['sci-fi', 5], ['horror', 6], ['fantasy', 7],
      ['documentary', 8], ['nature', 9], ['history', 10], ['science', 11],
      ['biographical', 12], ['classical', 13], ['jazz', 14], ['ambient', 15],
      ['guided', 16], ['mindfulness', 17], ['animation', 18], ['experimental', 19]
    ]);

    // Initialize category mapping
    this.categoryMap = new Map([
      ['movie', 0], ['series', 1], ['documentary', 2],
      ['music', 3], ['meditation', 4], ['short', 5]
    ]);
  }

  /**
   * Generate 1536D embedding from emotional profile
   */
  generate(profile: EmotionalContentProfile, content: ContentMetadata): Float32Array {
    const embedding = new Float32Array(this.DIMENSIONS);
    embedding.fill(0);

    // Segment 1 (0-255): Primary tone encoding
    this.encodePrimaryTone(embedding, profile.primaryTone, 0);

    // Segment 2 (256-511): Valence/arousal deltas
    this.encodeRangeValue(embedding, 256, 383, profile.valenceDelta, -1.0, 1.0);
    this.encodeRangeValue(embedding, 384, 511, profile.arousalDelta, -1.0, 1.0);

    // Segment 3 (512-767): Intensity/complexity
    this.encodeRangeValue(embedding, 512, 639, profile.intensity, 0.0, 1.0);
    this.encodeRangeValue(embedding, 640, 767, profile.complexity, 0.0, 1.0);

    // Segment 4 (768-1023): Target states
    this.encodeTargetStates(embedding, profile.targetStates, 768);

    // Segment 5 (1024-1279): Genres/category
    this.encodeGenresCategory(embedding, content.genres, content.category, 1024);

    // Normalize to unit length
    return this.normalizeVector(embedding);
  }

  private encodePrimaryTone(embedding: Float32Array, tone: string, offset: number): void {
    const index = this.toneMap.get(tone.toLowerCase()) ?? 224; // Default to neutral
    embedding[offset + index] = 1.0;
  }

  private encodeRangeValue(
    embedding: Float32Array,
    startIdx: number,
    endIdx: number,
    value: number,
    minValue: number,
    maxValue: number
  ): void {
    const normalized = (value - minValue) / (maxValue - minValue);
    const rangeSize = endIdx - startIdx + 1;
    const center = normalized * rangeSize;
    const sigma = rangeSize / 6.0;

    for (let i = 0; i < rangeSize; i++) {
      const distance = i - center;
      const gaussianValue = Math.exp(-(distance * distance) / (2 * sigma * sigma));
      embedding[startIdx + i] = gaussianValue;
    }
  }

  private encodeTargetStates(
    embedding: Float32Array,
    targetStates: Array<{ currentValence: number; currentArousal: number }>,
    offset: number
  ): void {
    const statesToEncode = targetStates.slice(0, 3); // Encode up to 3 states

    statesToEncode.forEach((state, i) => {
      const stateOffset = offset + (i * 86);

      // Encode valence
      this.encodeRangeValue(embedding, stateOffset, stateOffset + 42, state.currentValence, -1.0, 1.0);

      // Encode arousal
      this.encodeRangeValue(embedding, stateOffset + 43, stateOffset + 85, state.currentArousal, -1.0, 1.0);
    });
  }

  private encodeGenresCategory(
    embedding: Float32Array,
    genres: string[],
    category: string,
    offset: number
  ): void {
    // Encode genres (one-hot in first 128 dimensions)
    genres.forEach(genre => {
      const index = this.genreMap.get(genre.toLowerCase());
      if (index !== undefined && index < 128) {
        embedding[offset + index] = 1.0;
      }
    });

    // Encode category (one-hot in next 128 dimensions)
    const categoryIndex = this.categoryMap.get(category);
    if (categoryIndex !== undefined) {
      embedding[offset + 128 + categoryIndex] = 1.0;
    }
  }

  private normalizeVector(vector: Float32Array): Float32Array {
    let magnitude = 0;
    for (let i = 0; i < vector.length; i++) {
      magnitude += vector[i] * vector[i];
    }
    magnitude = Math.sqrt(magnitude);

    if (magnitude === 0) {
      return vector;
    }

    const normalized = new Float32Array(vector.length);
    for (let i = 0; i < vector.length; i++) {
      normalized[i] = vector[i] / magnitude;
    }

    return normalized;
  }
}
