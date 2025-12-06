/**
 * ContentProfiler - Main orchestrator for content profiling
 */

import { ContentMetadata, EmotionalContentProfile, SearchResult as TypeSearchResult } from './types';
import { EmbeddingGenerator } from './embedding-generator';
import { VectorStore, SearchResult as StoreSearchResult } from './vector-store';
import { BatchProcessor } from './batch-processor';

export class ContentProfiler {
  private embeddingGenerator: EmbeddingGenerator;
  private vectorStore: VectorStore;
  private batchProcessor: BatchProcessor;
  private profiles: Map<string, EmotionalContentProfile> = new Map();

  constructor() {
    this.embeddingGenerator = new EmbeddingGenerator();
    this.vectorStore = new VectorStore();
    this.batchProcessor = new BatchProcessor();
  }

  /**
   * Profile a single content item
   */
  async profile(content: ContentMetadata): Promise<EmotionalContentProfile> {
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
          description: 'Recommended for users seeking emotional balance'
        },
        {
          currentValence: this.randomInRange(-0.3, 0.3),
          currentArousal: this.randomInRange(-0.3, 0.3),
          description: 'Good for relaxation and stress relief'
        }
      ],
      embeddingId: `emb_${content.contentId}_${Date.now()}`,
      timestamp: Date.now()
    };

    // Generate embedding
    const embedding = this.embeddingGenerator.generate(profile, content);

    // Store embedding
    await this.vectorStore.upsert(content.contentId, embedding, {
      title: content.title,
      category: content.category,
      genres: content.genres
    });

    // Store profile
    this.profiles.set(content.contentId, profile);

    return profile;
  }

  /**
   * Search for similar content by transition vector
   */
  async search(transitionVector: Float32Array, limit: number = 10): Promise<TypeSearchResult[]> {
    const storeResults = await this.vectorStore.search(transitionVector, limit);

    return storeResults.map(result => ({
      contentId: result.id,
      title: result.metadata.title || result.id,
      similarityScore: result.score,
      profile: this.profiles.get(result.id) || this.createDummyProfile(result.id),
      metadata: this.createMetadataFromStore(result),
      relevanceReason: this.explainRelevance(result.score)
    }));
  }

  /**
   * Batch profile multiple items
   */
  async batchProfile(contents: ContentMetadata[], batchSize: number = 10): Promise<void> {
    const generator = this.batchProcessor.profile(contents, batchSize);

    for await (const profile of generator) {
      this.profiles.set(profile.contentId, profile);
    }
  }

  private inferTone(content: ContentMetadata): string {
    const tones = ['uplifting', 'calming', 'thrilling', 'dramatic', 'serene', 'melancholic'];

    if (content.category === 'meditation') return 'calming';
    if (content.category === 'documentary') return 'serene';
    if (content.genres.includes('thriller')) return 'thrilling';
    if (content.genres.includes('comedy')) return 'uplifting';
    if (content.genres.includes('drama')) return 'dramatic';

    return tones[Math.floor(Math.random() * tones.length)];
  }

  private randomInRange(min: number, max: number): number {
    return Math.random() * (max - min) + min;
  }

  private createDummyProfile(contentId: string): EmotionalContentProfile {
    return {
      contentId,
      primaryTone: 'neutral',
      valenceDelta: 0,
      arousalDelta: 0,
      intensity: 0.5,
      complexity: 0.5,
      targetStates: [],
      embeddingId: '',
      timestamp: Date.now()
    };
  }

  private createMetadataFromStore(result: StoreSearchResult): ContentMetadata {
    return {
      contentId: result.id,
      title: result.metadata.title || result.id,
      description: 'Generated content',
      platform: 'mock',
      genres: result.metadata.genres || [],
      category: result.metadata.category || 'movie',
      tags: [],
      duration: 120
    };
  }

  private explainRelevance(score: number): string {
    if (score > 0.9) return 'Excellent match for your emotional transition';
    if (score > 0.7) return 'Good match for your desired emotional state';
    if (score > 0.5) return 'Moderate match with similar emotional characteristics';
    return 'May provide some emotional benefit';
  }
}
