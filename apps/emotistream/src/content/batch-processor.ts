/**
 * BatchProcessor - Processes content in batches with rate limiting
 */

import { ContentMetadata, EmotionalContentProfile } from './types';
import { EmbeddingGenerator } from './embedding-generator';
import { VectorStore } from './vector-store';

export class BatchProcessor {
  private embeddingGenerator: EmbeddingGenerator;
  private vectorStore: VectorStore;

  constructor() {
    this.embeddingGenerator = new EmbeddingGenerator();
    this.vectorStore = new VectorStore();
  }

  /**
   * Profile multiple content items in batches
   * Returns async generator for streaming results
   */
  async *profile(
    contents: ContentMetadata[],
    batchSize: number = 10
  ): AsyncGenerator<EmotionalContentProfile> {
    const batches = this.splitIntoBatches(contents, batchSize);

    for (const batch of batches) {
      // Process batch items in parallel
      const promises = batch.map(content => this.profileSingle(content));
      const results = await Promise.all(promises);

      // Yield each result
      for (const profile of results) {
        yield profile;
      }

      // Rate limiting delay between batches (simulated)
      if (batches.length > 1) {
        await this.delay(100); // Small delay for testing
      }
    }
  }

  /**
   * Profile a single content item
   */
  private async profileSingle(content: ContentMetadata): Promise<EmotionalContentProfile> {
    // Generate mock emotional profile
    // In real implementation, this would call Gemini API
    const profile: EmotionalContentProfile = {
      contentId: content.contentId,
      primaryTone: this.inferTone(content),
      valenceDelta: this.randomInRange(-0.5, 0.7),
      arousalDelta: this.randomInRange(-0.6, 0.6),
      intensity: this.randomInRange(0.3, 0.9),
      complexity: this.randomInRange(0.3, 0.8),
      targetStates: [
        {
          currentValence: this.randomInRange(-0.5, 0.5),
          currentArousal: this.randomInRange(-0.5, 0.5),
          description: 'Target emotional state'
        }
      ],
      embeddingId: `emb_${content.contentId}`,
      timestamp: Date.now()
    };

    // Generate and store embedding
    const embedding = this.embeddingGenerator.generate(profile, content);
    await this.vectorStore.upsert(content.contentId, embedding, {
      title: content.title,
      category: content.category
    });

    return profile;
  }

  private inferTone(content: ContentMetadata): string {
    const tones = ['uplifting', 'calming', 'thrilling', 'dramatic', 'serene'];

    if (content.category === 'meditation') return 'calming';
    if (content.category === 'documentary') return 'serene';
    if (content.genres.includes('thriller')) return 'thrilling';
    if (content.genres.includes('comedy')) return 'uplifting';
    if (content.genres.includes('drama')) return 'dramatic';

    return tones[Math.floor(Math.random() * tones.length)];
  }

  private splitIntoBatches<T>(items: T[], batchSize: number): T[][] {
    const batches: T[][] = [];
    for (let i = 0; i < items.length; i += batchSize) {
      batches.push(items.slice(i, i + batchSize));
    }
    return batches;
  }

  private randomInRange(min: number, max: number): number {
    return Math.random() * (max - min) + min;
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}
