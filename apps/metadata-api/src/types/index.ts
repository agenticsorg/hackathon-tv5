/**
 * Nexus-UMMID Metadata API Type Definitions
 *
 * Core TypeScript interfaces for the Entertainment Discovery platform
 */

/**
 * Media Metadata Interface
 * Represents comprehensive metadata for media assets
 */
export interface MediaMetadata {
  id: string;
  eidr?: string; // Entertainment Identifier Registry ID
  title: string;
  type: 'movie' | 'series' | 'episode' | 'documentary' | 'short';
  releaseDate?: Date;
  duration?: number; // in minutes

  // Descriptive Metadata
  synopsis?: string;
  description?: string;
  genres: string[];
  keywords: string[];
  moodTags?: string[];
  themes?: string[];

  // Credits
  director?: string[];
  cast?: CastMember[];
  producers?: string[];
  writers?: string[];

  // Technical Metadata
  language: string;
  originalLanguage?: string;
  subtitles?: string[];
  audioTracks?: string[];
  resolution?: '4K' | '1080p' | '720p' | 'SD';
  aspectRatio?: string;

  // Content Ratings
  rating?: string; // e.g., "PG-13", "TV-MA"
  contentWarnings?: string[];

  // Platform & Rights
  platforms?: PlatformAvailability[];
  territories?: string[]; // ISO country codes
  rightsExpiry?: Date;

  // Discovery & Enrichment
  embedding?: number[]; // Semantic embedding vector
  similarContent?: string[]; // IDs of similar content
  popularity?: number;
  userRating?: number;

  // Timestamps
  createdAt: Date;
  updatedAt: Date;
  enrichedAt?: Date;
}

/**
 * Cast Member Interface
 */
export interface CastMember {
  name: string;
  role: string;
  characterName?: string;
  order?: number;
}

/**
 * Platform Availability Interface
 */
export interface PlatformAvailability {
  platform: 'netflix' | 'amazon' | 'hulu' | 'disney' | 'apple' | 'hbo' | 'paramount' | 'peacock' | 'custom';
  region: string; // ISO country code
  availableFrom?: Date;
  availableUntil?: Date;
  url?: string;
  subscriptionRequired: boolean;
  validated: boolean;
  validatedAt?: Date;
}

/**
 * Content Recommendation Interface
 */
export interface ContentRecommendation {
  assetId: string;
  title: string;
  type: string;
  similarity: number; // 0-1 score
  reason: string; // Why this was recommended
  thumbnailUrl?: string;
  platforms: string[];
  rating?: string;
  genres: string[];
}

/**
 * Metadata Enrichment Request
 */
export interface EnrichmentRequest {
  assetId: string;
  fields?: string[]; // Specific fields to enrich, or all if empty
  model?: 'gemini-2.0' | 'claude-3.5' | 'gpt-4';
  approach?: string; // Enrichment strategy (learned from AgentDB)
}

/**
 * Enrichment Result
 */
export interface EnrichmentResult {
  assetId: string;
  title: string;
  approach: string;
  model: string;
  quality: number; // 0-1 quality score
  latencyMs: number;
  tokensUsed: number;
  partialMetadata: Partial<MediaMetadata>;
  enrichedMetadata: Partial<MediaMetadata>;
  fieldsEnriched: string[];
  timestamp: Date;
}

/**
 * Search Result Interface
 */
export interface SearchResult {
  assetId: string;
  metadata: MediaMetadata;
  similarity?: number; // For semantic search
  rank?: number;
}

/**
 * API Response Wrapper
 */
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: any;
  };
  metadata?: {
    timestamp: Date;
    requestId: string;
    latency?: number;
  };
}

/**
 * Pagination Parameters
 */
export interface PaginationParams {
  page: number;
  limit: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

/**
 * Paginated Response
 */
export interface PaginatedResponse<T> {
  items: T[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrevious: boolean;
  };
}

/**
 * Platform Validation Result
 */
export interface ValidationResult {
  platform: string;
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
  validatedAt: Date;
}

/**
 * Validation Error
 */
export interface ValidationError {
  field: string;
  message: string;
  severity: 'error' | 'critical';
  platformRequirement?: string;
}

/**
 * Validation Warning
 */
export interface ValidationWarning {
  field: string;
  message: string;
  recommendation?: string;
}

/**
 * Rights Collision
 */
export interface RightsCollision {
  assetId: string;
  platform: string;
  territory: string;
  timeRange: {
    start: Date;
    end: Date;
  };
  conflictingAssets: string[];
  severity: 'high' | 'medium' | 'low';
  detected: Date;
}
