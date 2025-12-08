/**
 * ContentProfiler - Main orchestrator for content profiling
 */

import { ContentMetadata, EmotionalContentProfile, SearchResult as TypeSearchResult } from './types.js';
import { EmbeddingGenerator } from './embedding-generator.js';
import { VectorStore, SearchResult as StoreSearchResult } from './vector-store.js';
import { BatchProcessor } from './batch-processor.js';
import { tmdbCatalog, TMDBCatalog } from './tmdb-catalog.js';
import { MockCatalogGenerator } from './mock-catalog.js';
import { createLogger } from '../utils/logger.js';

const logger = createLogger('ContentProfiler');

export class ContentProfiler {
  private embeddingGenerator: EmbeddingGenerator;
  private vectorStore: VectorStore;
  private batchProcessor: BatchProcessor;
  private profiles: Map<string, EmotionalContentProfile> = new Map();
  private metadata: Map<string, ContentMetadata> = new Map();
  private tmdbCatalog: TMDBCatalog;
  private mockCatalog: MockCatalogGenerator;
  private initialized: boolean = false;

  // Genre-based emotional profiles for deterministic content profiling
  private GENRE_EMOTIONAL_PROFILES: Record<string, {
    valenceDelta: number;
    arousalDelta: number;
    intensity: number;
  }> = {
    'comedy': { valenceDelta: 0.5, arousalDelta: 0.2, intensity: 0.6 },
    'horror': { valenceDelta: -0.3, arousalDelta: 0.7, intensity: 0.9 },
    'romance': { valenceDelta: 0.4, arousalDelta: -0.1, intensity: 0.5 },
    'action': { valenceDelta: 0.3, arousalDelta: 0.6, intensity: 0.8 },
    'drama': { valenceDelta: 0.1, arousalDelta: 0.1, intensity: 0.7 },
    'documentary': { valenceDelta: 0.2, arousalDelta: -0.2, intensity: 0.4 },
    'thriller': { valenceDelta: -0.1, arousalDelta: 0.5, intensity: 0.8 },
    'animation': { valenceDelta: 0.4, arousalDelta: 0.3, intensity: 0.5 },
    'family': { valenceDelta: 0.5, arousalDelta: 0.1, intensity: 0.4 },
    'sci-fi': { valenceDelta: 0.2, arousalDelta: 0.4, intensity: 0.7 },
    'science fiction': { valenceDelta: 0.2, arousalDelta: 0.4, intensity: 0.7 },
    'mystery': { valenceDelta: 0.0, arousalDelta: 0.3, intensity: 0.6 },
    'fantasy': { valenceDelta: 0.3, arousalDelta: 0.4, intensity: 0.7 },
    'adventure': { valenceDelta: 0.4, arousalDelta: 0.5, intensity: 0.7 },
    'crime': { valenceDelta: -0.1, arousalDelta: 0.4, intensity: 0.7 },
    'war': { valenceDelta: -0.2, arousalDelta: 0.6, intensity: 0.9 },
    'history': { valenceDelta: 0.1, arousalDelta: 0.2, intensity: 0.5 },
    'music': { valenceDelta: 0.4, arousalDelta: 0.3, intensity: 0.6 },
    'western': { valenceDelta: 0.1, arousalDelta: 0.4, intensity: 0.6 },
    'tv movie': { valenceDelta: 0.2, arousalDelta: 0.1, intensity: 0.5 },
  };

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

    logger.info('Initializing ContentProfiler...');

    let catalog: ContentMetadata[];

    // Try TMDB first, fall back to mock
    if (this.tmdbCatalog.isAvailable()) {
      logger.info('TMDB configured - fetching real content...');
      catalog = await this.tmdbCatalog.fetchCatalog(contentCount);

      if (catalog.length === 0) {
        logger.warn('TMDB fetch failed, falling back to mock data');
        catalog = this.mockCatalog.generate(contentCount);
      }
    } else {
      logger.info('TMDB not configured - using mock data');
      catalog = this.mockCatalog.generate(contentCount);
    }

    logger.debug(`Processing ${catalog.length} content items...`);

    // Profile all content
    for (const content of catalog) {
      await this.profile(content);
      this.metadata.set(content.contentId, content);
    }

    this.initialized = true;
    logger.info(`ContentProfiler initialized with ${catalog.length} items`);
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
   * Uses deterministic genre-based emotional profiles
   */
  async profile(content: ContentMetadata): Promise<EmotionalContentProfile> {
    // Calculate emotional profile deterministically from genres
    const emotionalProfile = this.calculateEmotionalProfile(content.genres);

    const profile: EmotionalContentProfile = {
      contentId: content.contentId,
      primaryTone: this.inferTone(content),
      valenceDelta: emotionalProfile.valenceDelta,
      arousalDelta: emotionalProfile.arousalDelta,
      intensity: emotionalProfile.intensity,
      complexity: this.calculateComplexity(content.genres),
      targetStates: [
        {
          currentValence: emotionalProfile.valenceDelta * 0.5,
          currentArousal: emotionalProfile.arousalDelta * 0.5,
          description: 'Recommended for users seeking emotional balance'
        },
        {
          currentValence: emotionalProfile.valenceDelta * 0.3,
          currentArousal: emotionalProfile.arousalDelta * 0.3,
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
      contentId: content.contentId,
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

  /**
   * Calculate emotional profile deterministically from genres
   * Averages the emotional characteristics of all genres
   */
  private calculateEmotionalProfile(genres: string[]): {
    valenceDelta: number;
    arousalDelta: number;
    intensity: number;
  } {
    if (!genres || genres.length === 0) {
      // Default neutral profile for content without genres
      return { valenceDelta: 0.2, arousalDelta: 0.1, intensity: 0.5 };
    }

    const valenceDeltaValues: number[] = [];
    const arousalDeltaValues: number[] = [];
    const intensityValues: number[] = [];

    for (const genre of genres) {
      const normalizedGenre = genre.toLowerCase();
      const genreProfile = this.GENRE_EMOTIONAL_PROFILES[normalizedGenre];

      if (genreProfile) {
        valenceDeltaValues.push(genreProfile.valenceDelta);
        arousalDeltaValues.push(genreProfile.arousalDelta);
        intensityValues.push(genreProfile.intensity);
      }
    }

    // If no matching genres found, use neutral defaults
    if (valenceDeltaValues.length === 0) {
      return { valenceDelta: 0.2, arousalDelta: 0.1, intensity: 0.5 };
    }

    return {
      valenceDelta: this.average(valenceDeltaValues),
      arousalDelta: this.average(arousalDeltaValues),
      intensity: this.average(intensityValues),
    };
  }

  /**
   * Calculate complexity based on genre count and diversity
   */
  private calculateComplexity(genres: string[]): number {
    if (!genres || genres.length === 0) return 0.3;

    // More genres = more complexity, capped at 0.9
    const baseComplexity = Math.min(0.3 + (genres.length * 0.15), 0.9);

    return baseComplexity;
  }

  /**
   * Calculate average of an array of numbers
   */
  private average(numbers: number[]): number {
    if (numbers.length === 0) return 0;
    return numbers.reduce((sum, num) => sum + num, 0) / numbers.length;
  }

  /**
   * Infer primary tone deterministically from content metadata
   * Maps genres and categories to emotional tones
   */
  private inferTone(content: ContentMetadata): string {
    // Category-based tones (highest priority)
    if (content.category === 'meditation') return 'calming';
    if (content.category === 'documentary') return 'serene';
    if (content.category === 'music') return 'uplifting';

    // Genre-based tones (deterministic mapping)
    const genreToneMap: Record<string, string> = {
      'thriller': 'thrilling',
      'horror': 'thrilling',
      'action': 'thrilling',
      'comedy': 'uplifting',
      'animation': 'uplifting',
      'family': 'uplifting',
      'music': 'uplifting',
      'drama': 'dramatic',
      'romance': 'dramatic',
      'war': 'dramatic',
      'history': 'dramatic',
      'crime': 'dramatic',
      'documentary': 'serene',
      'nature': 'serene',
      'sci-fi': 'serene',
      'science fiction': 'serene',
      'mystery': 'melancholic',
      'western': 'melancholic',
      'fantasy': 'uplifting',
      'adventure': 'uplifting',
    };

    // Check genres in order and return first match
    for (const genre of content.genres) {
      const normalizedGenre = genre.toLowerCase();
      if (genreToneMap[normalizedGenre]) {
        return genreToneMap[normalizedGenre];
      }
    }

    // Default tone based on first letter of contentId for determinism
    // This ensures same content always gets same tone
    const charCode = content.contentId.charCodeAt(0) || 0;
    const defaultTones = ['uplifting', 'calming', 'serene', 'dramatic'];
    return defaultTones[charCode % defaultTones.length];
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
    const category = result.metadata.category as 'movie' | 'series' | 'documentary' | 'music' | 'meditation' | 'short' | undefined;
    return {
      contentId: result.id,
      title: result.metadata.title || result.id,
      description: 'Generated content',
      platform: 'mock',
      genres: result.metadata.genres || [],
      category: category || 'movie',
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
