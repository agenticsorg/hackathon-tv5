/**
 * ContentProfiler - Main orchestrator for content profiling
 */

import { ContentMetadata, EmotionalContentProfile, SearchResult as TypeSearchResult } from './types.js';
import { EmbeddingGenerator } from './embedding-generator.js';
import { VectorStore, SearchResult as StoreSearchResult } from './vector-store.js';
import { BatchProcessor } from './batch-processor.js';
import { tmdbCatalog, TMDBCatalog } from './tmdb-catalog.js';
import { MockCatalogGenerator } from './mock-catalog.js';

export class ContentProfiler {
  private embeddingGenerator: EmbeddingGenerator;
  private vectorStore: VectorStore;
  private batchProcessor: BatchProcessor;
  private profiles: Map<string, EmotionalContentProfile> = new Map();
  private metadata: Map<string, ContentMetadata> = new Map();
  private tmdbCatalog: TMDBCatalog;
  private mockCatalog: MockCatalogGenerator;
  private initialized: boolean = false;

  constructor() {
    this.embeddingGenerator = new EmbeddingGenerator();
    this.vectorStore = new VectorStore();
    this.batchProcessor = new BatchProcessor();
    this.tmdbCatalog = tmdbCatalog;
    this.mockCatalog = new MockCatalogGenerator();
  }

  /**
   * Initialize the profiler with content catalog
   * Uses TMDB if configured, falls back to mock data
   */
  async initialize(contentCount: number = 100): Promise<void> {
    if (this.initialized) return;

    console.log('Initializing ContentProfiler...');

    let catalog: ContentMetadata[];

    // Try TMDB first, fall back to mock
    if (this.tmdbCatalog.isAvailable()) {
      console.log('TMDB configured - fetching real content...');
      catalog = await this.tmdbCatalog.fetchCatalog(contentCount);

      if (catalog.length === 0) {
        console.warn('TMDB fetch failed, falling back to mock data');
        catalog = this.mockCatalog.generate(contentCount);
      }
    } else {
      console.log('TMDB not configured - using mock data');
      catalog = this.mockCatalog.generate(contentCount);
    }

    console.log(`Processing ${catalog.length} content items...`);

    // Profile all content
    for (const content of catalog) {
      await this.profile(content);
      this.metadata.set(content.contentId, content);
    }

    this.initialized = true;
    console.log(`ContentProfiler initialized with ${catalog.length} items`);
  }

  /**
   * Check if using real TMDB data
   */
  isUsingTMDB(): boolean {
    return this.tmdbCatalog.isAvailable();
  }

  /**
   * Get content metadata by ID
   */
  getMetadata(contentId: string): ContentMetadata | undefined {
    return this.metadata.get(contentId);
  }

  /**
   * Get all content metadata
   */
  getAllMetadata(): ContentMetadata[] {
    return Array.from(this.metadata.values());
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
    // Try to get cached metadata first
    const cached = this.metadata.get(result.id);
    if (cached) return cached;

    // Fallback to basic metadata from store
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
