/**
 * Metadata Service
 * Business logic for managing media metadata
 */

import {
  MediaMetadata,
  SearchResult,
  EnrichmentRequest,
  EnrichmentResult,
  ValidationResult,
  PaginatedResponse,
  PaginationParams
} from '../types';

/**
 * MetadataService - Core business logic for metadata operations
 */
export class MetadataService {
  private metadata: Map<string, MediaMetadata>;

  constructor() {
    this.metadata = new Map();
    this.initializeSampleData();
  }

  /**
   * Get metadata by ID
   */
  async getById(id: string): Promise<MediaMetadata | null> {
    const item = this.metadata.get(id);
    return item || null;
  }

  /**
   * Get all metadata with pagination
   */
  async getAll(params: PaginationParams): Promise<PaginatedResponse<MediaMetadata>> {
    const items = Array.from(this.metadata.values());
    const { page, limit } = params;

    const start = (page - 1) * limit;
    const end = start + limit;
    const paginatedItems = items.slice(start, end);

    return {
      items: paginatedItems,
      pagination: {
        page,
        limit,
        total: items.length,
        totalPages: Math.ceil(items.length / limit),
        hasNext: end < items.length,
        hasPrevious: page > 1
      }
    };
  }

  /**
   * Create new metadata
   */
  async create(data: Partial<MediaMetadata>): Promise<MediaMetadata> {
    const id = data.id || this.generateId();

    const metadata: MediaMetadata = {
      id,
      title: data.title || 'Untitled',
      type: data.type || 'movie',
      genres: data.genres || [],
      keywords: data.keywords || [],
      language: data.language || 'en',
      createdAt: new Date(),
      updatedAt: new Date(),
      ...data
    };

    this.metadata.set(id, metadata);
    return metadata;
  }

  /**
   * Update existing metadata
   */
  async update(id: string, updates: Partial<MediaMetadata>): Promise<MediaMetadata | null> {
    const existing = this.metadata.get(id);

    if (!existing) {
      return null;
    }

    const updated: MediaMetadata = {
      ...existing,
      ...updates,
      id, // Ensure ID doesn't change
      updatedAt: new Date()
    };

    this.metadata.set(id, updated);
    return updated;
  }

  /**
   * Delete metadata
   */
  async delete(id: string): Promise<boolean> {
    return this.metadata.delete(id);
  }

  /**
   * Search metadata by query
   */
  async search(query: string, limit: number = 10): Promise<SearchResult[]> {
    const lowerQuery = query.toLowerCase();
    const results: SearchResult[] = [];

    for (const metadata of this.metadata.values()) {
      const titleMatch = metadata.title.toLowerCase().includes(lowerQuery);
      const genreMatch = metadata.genres.some(g => g.toLowerCase().includes(lowerQuery));
      const keywordMatch = metadata.keywords.some(k => k.toLowerCase().includes(lowerQuery));
      const synopsisMatch = metadata.synopsis?.toLowerCase().includes(lowerQuery);

      if (titleMatch || genreMatch || keywordMatch || synopsisMatch) {
        results.push({
          assetId: metadata.id,
          metadata,
          similarity: this.calculateSimilarity(query, metadata),
          rank: results.length + 1
        });
      }

      if (results.length >= limit) {
        break;
      }
    }

    // Sort by similarity score
    return results.sort((a, b) => (b.similarity || 0) - (a.similarity || 0));
  }

  /**
   * Enrich metadata with AI
   * (Placeholder for integration with Vertex AI / Gemini)
   */
  async enrich(request: EnrichmentRequest): Promise<EnrichmentResult> {
    const startTime = Date.now();
    const metadata = await this.getById(request.assetId);

    if (!metadata) {
      throw new Error(`Asset not found: ${request.assetId}`);
    }

    // Simulate enrichment (replace with actual AI service)
    const enrichedFields: string[] = [];
    const enrichedMetadata: Partial<MediaMetadata> = {};

    if (!metadata.synopsis || request.fields?.includes('synopsis')) {
      enrichedMetadata.synopsis = this.generateSynopsis(metadata);
      enrichedFields.push('synopsis');
    }

    if (!metadata.keywords.length || request.fields?.includes('keywords')) {
      enrichedMetadata.keywords = this.generateKeywords(metadata);
      enrichedFields.push('keywords');
    }

    if (!metadata.moodTags || request.fields?.includes('moodTags')) {
      enrichedMetadata.moodTags = this.generateMoodTags(metadata);
      enrichedFields.push('moodTags');
    }

    // Update metadata
    await this.update(request.assetId, {
      ...enrichedMetadata,
      enrichedAt: new Date()
    });

    const latencyMs = Date.now() - startTime;

    return {
      assetId: request.assetId,
      title: metadata.title,
      approach: request.approach || 'default_enrichment',
      model: request.model || 'gemini-2.0',
      quality: 0.85, // Simulated quality score
      latencyMs,
      tokensUsed: 250, // Simulated token usage
      partialMetadata: { ...metadata },
      enrichedMetadata,
      fieldsEnriched: enrichedFields,
      timestamp: new Date()
    };
  }

  /**
   * Validate metadata against platform requirements
   */
  async validate(assetId: string, platform: string): Promise<ValidationResult> {
    const metadata = await this.getById(assetId);

    if (!metadata) {
      throw new Error(`Asset not found: ${assetId}`);
    }

    const errors = [];
    const warnings = [];

    // Basic validation rules (extend for each platform)
    if (!metadata.title) {
      errors.push({
        field: 'title',
        message: 'Title is required',
        severity: 'critical' as const,
        platformRequirement: 'All platforms require a title'
      });
    }

    if (!metadata.synopsis || metadata.synopsis.length < 50) {
      warnings.push({
        field: 'synopsis',
        message: 'Synopsis should be at least 50 characters',
        recommendation: 'Add detailed synopsis for better discovery'
      });
    }

    if (metadata.genres.length === 0) {
      errors.push({
        field: 'genres',
        message: 'At least one genre is required',
        severity: 'error' as const,
        platformRequirement: `${platform} requires genre classification`
      });
    }

    return {
      platform,
      valid: errors.length === 0,
      errors,
      warnings,
      validatedAt: new Date()
    };
  }

  /**
   * Generate unique ID
   */
  private generateId(): string {
    return `asset-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Calculate similarity score (simple implementation)
   */
  private calculateSimilarity(query: string, metadata: MediaMetadata): number {
    const lowerQuery = query.toLowerCase();
    let score = 0;

    if (metadata.title.toLowerCase().includes(lowerQuery)) {
      score += 0.5;
    }

    if (metadata.genres.some(g => g.toLowerCase().includes(lowerQuery))) {
      score += 0.3;
    }

    if (metadata.keywords.some(k => k.toLowerCase().includes(lowerQuery))) {
      score += 0.2;
    }

    return Math.min(score, 1.0);
  }

  /**
   * Generate synopsis (placeholder for AI)
   */
  private generateSynopsis(metadata: MediaMetadata): string {
    return `An engaging ${metadata.type} titled "${metadata.title}" in the ${metadata.genres.join(', ')} genre(s).`;
  }

  /**
   * Generate keywords (placeholder for AI)
   */
  private generateKeywords(metadata: MediaMetadata): string[] {
    return [
      ...metadata.genres,
      metadata.type,
      'entertainment',
      'streaming'
    ];
  }

  /**
   * Generate mood tags (placeholder for AI)
   */
  private generateMoodTags(metadata: MediaMetadata): string[] {
    const moodMap: Record<string, string[]> = {
      action: ['exciting', 'thrilling', 'intense'],
      comedy: ['funny', 'lighthearted', 'entertaining'],
      drama: ['emotional', 'thought-provoking', 'serious'],
      horror: ['scary', 'suspenseful', 'dark'],
      romance: ['romantic', 'heartwarming', 'touching']
    };

    const moods: string[] = [];
    for (const genre of metadata.genres) {
      const genreMoods = moodMap[genre.toLowerCase()];
      if (genreMoods) {
        moods.push(...genreMoods);
      }
    }

    return [...new Set(moods)]; // Remove duplicates
  }

  /**
   * Initialize sample data for demonstration
   */
  private initializeSampleData(): void {
    const sampleData: Partial<MediaMetadata>[] = [
      {
        id: 'asset-001',
        eidr: '10.5240/ABCD-1234-5678-90AB-CDEF',
        title: 'The Nexus Chronicles',
        type: 'series',
        genres: ['sci-fi', 'thriller', 'drama'],
        keywords: ['artificial intelligence', 'future', 'technology', 'conspiracy'],
        language: 'en',
        synopsis: 'A gripping series about AI consciousness and the battle for humanity\'s future.',
        rating: 'TV-MA',
        duration: 45,
        releaseDate: new Date('2024-01-15')
      },
      {
        id: 'asset-002',
        title: 'Metadata: The Movie',
        type: 'movie',
        genres: ['documentary', 'technology'],
        keywords: ['metadata', 'information', 'discovery', 'streaming'],
        language: 'en',
        synopsis: 'An exploration of how metadata shapes our digital entertainment landscape.',
        rating: 'PG',
        duration: 90,
        releaseDate: new Date('2024-03-20')
      }
    ];

    for (const data of sampleData) {
      this.create(data);
    }
  }
}

export default MetadataService;
