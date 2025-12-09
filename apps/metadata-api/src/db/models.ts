/**
 * TypeScript Models for Nexus-UMMID
 * Firestore Document Models
 *
 * Optimized for 400M+ users with hypergraph relationships
 */

import { Timestamp, FieldValue } from '@google-cloud/firestore';

// ============================================================================
// MEDIA METADATA MODELS
// ============================================================================

/**
 * Core media content metadata
 * Collection: content
 */
export interface MediaMetadata {
  // Identity
  id: string;
  eidr_id?: string; // Entertainment Identifier Registry (canonical anchor)

  // Basic Information
  title: string;
  original_title?: string;
  type: 'movie' | 'series' | 'episode' | 'short' | 'documentary';

  // Classification
  genres: string[]; // ['thriller', 'sci-fi', 'action']
  subgenres?: string[];
  mood_tags?: string[]; // ['dark', 'psychological', 'intense']
  keywords?: string[]; // For semantic search

  // Release Information
  release_year: number;
  release_date?: Timestamp;
  original_language: string;

  // Content Details
  synopsis: string;
  synopsis_short?: string; // <200 chars for UI
  duration_minutes?: number;
  rating: string; // 'G', 'PG', 'PG-13', 'R', 'NC-17', 'TV-MA', etc.
  content_warnings?: string[];

  // People
  directors?: string[];
  cast?: Array<{
    name: string;
    character?: string;
    order: number;
  }>;
  producers?: string[];
  writers?: string[];

  // Platform & Distribution
  platform: string[]; // ['netflix', 'amazon', 'hbo', 'hulu']
  streaming_urls: Record<string, string>; // { netflix: 'https://...', amazon: '...' }
  availability: Array<{
    platform: string;
    region: string;
    available_from: Timestamp;
    available_to?: Timestamp;
    quality: 'SD' | 'HD' | 'UHD' | '4K';
  }>;

  // Quality & Technical
  video_quality?: string[]; // ['UHD', '4K', 'HDR10', 'Dolby Vision']
  audio_formats?: string[]; // ['5.1', 'Atmos', 'DTS:X']
  subtitles?: string[]; // Language codes

  // Metrics
  imdb_rating?: number;
  tmdb_rating?: number;
  rotten_tomatoes_score?: number;
  popularity_score?: number; // Calculated based on views, ratings
  view_count?: number;

  // Embeddings for semantic search (stored in Cloud SQL pgvector)
  embedding_id?: string; // Reference to vector in Cloud SQL

  // Metadata
  created_at: Timestamp;
  updated_at: Timestamp;
  source: string; // 'user_upload', 'tmdb_api', 'gemini_enrichment'
  enrichment_status?: 'pending' | 'complete' | 'failed';

  // Versioning for bitemporal queries
  valid_from: Timestamp;
  valid_to?: Timestamp;
  version: number;
}

/**
 * Series-specific metadata
 * Collection: content (type: series)
 */
export interface SeriesMetadata extends MediaMetadata {
  type: 'series';

  // Series-specific
  total_seasons: number;
  total_episodes: number;
  episode_runtime_avg?: number;
  status: 'returning' | 'ended' | 'cancelled';

  // References
  season_ids?: string[]; // References to season documents
}

/**
 * Episode-specific metadata
 * Collection: content (type: episode)
 */
export interface EpisodeMetadata extends MediaMetadata {
  type: 'episode';

  // Episode-specific
  series_id: string; // Parent series
  season_number: number;
  episode_number: number;
  air_date?: Timestamp;
}

// ============================================================================
// USER MODELS
// ============================================================================

/**
 * User profile and preferences
 * Collection: users
 */
export interface UserProfile {
  // Identity
  user_id: string;

  // Profile
  display_name?: string;
  email?: string;
  avatar_url?: string;

  // Preferences
  preferred_genres: string[];
  preferred_languages: string[];
  preferred_platforms: string[];

  // Viewing History
  watch_history: Array<{
    content_id: string;
    watched_at: Timestamp;
    progress_percent: number; // 0-100
    completed: boolean;
    rating?: number; // 1-5 stars
  }>;

  // Ratings & Reviews
  ratings: Record<string, number>; // { content_id: rating }
  reviews?: Array<{
    content_id: string;
    review_text: string;
    rating: number;
    created_at: Timestamp;
  }>;

  // Watchlist
  watchlist: string[]; // content_ids
  favorites: string[]; // content_ids

  // Recommendations
  recommendation_profile?: {
    genres_affinity: Record<string, number>; // { thriller: 0.85, comedy: 0.3 }
    directors_affinity: Record<string, number>;
    actors_affinity: Record<string, number>;
    mood_affinity: Record<string, number>;
  };

  // Privacy & Settings
  share_watch_history: boolean;
  content_rating_limit?: string; // Max rating they want to see

  // Metadata
  created_at: Timestamp;
  last_active_at: Timestamp;
  account_status: 'active' | 'suspended' | 'deleted';
}

/**
 * Simplified user preference model for quick lookups
 * Collection: user_preferences
 */
export interface UserPreference {
  userId: string;

  // Preferences
  genres: string[];
  platforms: string[];

  // History (last 100 items for quick access)
  watchHistory: Array<{
    contentId: string;
    timestamp: Timestamp;
    rating?: number;
  }>;

  // Ratings (for collaborative filtering)
  ratings: Record<string, number>;

  // Updated timestamp
  updated_at: Timestamp;
}

// ============================================================================
// RECOMMENDATION MODELS
// ============================================================================

/**
 * Content recommendations
 * Collection: recommendations
 */
export interface ContentRecommendation {
  // Identity
  id: string;

  // Target
  user_id?: string; // null = global recommendation
  target_audience?: string; // 'thriller_fans', 'sci-fi_enthusiasts'

  // Content
  content_id: string;

  // Scoring
  score: number; // 0-1 relevance score
  confidence: number; // 0-1 confidence in recommendation

  // Reasoning
  reason: string; // Human-readable explanation
  reasoning_factors: Array<{
    factor: string; // 'genre_match', 'director_affinity', 'similar_users'
    weight: number; // 0-1
  }>;

  // Similar content
  similar_to?: string[]; // content_ids that influenced this recommendation

  // Metadata
  generated_at: Timestamp;
  expires_at?: Timestamp;
  algorithm: string; // 'collaborative_filtering', 'content_based', 'hybrid', 'gemini_ai'

  // Performance tracking
  impressions?: number;
  clicks?: number;
  ctr?: number; // Click-through rate
}

/**
 * Pre-computed recommendation sets for performance
 * Collection: recommendation_sets
 */
export interface RecommendationSet {
  // Identity
  user_id: string;

  // Recommendations by category
  personalized: string[]; // content_ids
  trending: string[];
  because_you_watched: Record<string, string[]>; // { content_id: [recommended_ids] }
  new_releases: string[];
  genre_picks: Record<string, string[]>; // { genre: [content_ids] }

  // Metadata
  generated_at: Timestamp;
  expires_at: Timestamp;
  version: number;
}

// ============================================================================
// PLATFORM MODELS
// ============================================================================

/**
 * Streaming platform information
 * Collection: platforms
 */
export interface StreamingPlatform {
  // Identity
  id: string;
  name: string;
  slug: string; // 'netflix', 'amazon-prime', 'hbo-max'

  // Details
  description?: string;
  website_url: string;
  logo_url?: string;

  // API Integration
  api_endpoint?: string;
  api_key_required: boolean;

  // Supported Features
  supports_4k: boolean;
  supports_hdr: boolean;
  supports_atmos: boolean;
  supports_download: boolean;

  // Regions
  available_regions: string[]; // ISO country codes

  // Pricing
  subscription_tiers?: Array<{
    name: string;
    price_usd: number;
    features: string[];
  }>;

  // Metadata
  added_at: Timestamp;
  updated_at: Timestamp;
  active: boolean;
}

// ============================================================================
// HYPERGRAPH EDGE MODELS
// ============================================================================

/**
 * Hypergraph edge for complex relationships
 * Collection: hyperedges
 *
 * Represents n-dimensional relationships (rights, territories, time windows)
 */
export interface Hyperedge {
  // Identity
  id: string;
  edge_type: string; // 'distribution_right', 'co_watch_pattern', 'genre_cluster'

  // Nodes (entities connected by this edge)
  nodes: Array<{
    collection: string; // 'content', 'users', 'platforms'
    document_id: string;
    role?: string; // 'asset', 'territory', 'platform', 'licensor'
  }>;

  // Edge properties
  properties: Record<string, any>; // Flexible metadata

  // Temporal validity (bitemporal modeling)
  valid_from: Timestamp;
  valid_to?: Timestamp;
  transaction_time: Timestamp;

  // Weight for graph algorithms
  weight?: number;

  // Metadata
  created_at: Timestamp;
  created_by?: string;
}

/**
 * Distribution rights hyperedge (use case example)
 */
export interface DistributionRightEdge extends Hyperedge {
  edge_type: 'distribution_right';

  // Override properties for type safety
  properties: {
    asset_id: string;
    territory: string; // ISO country code or 'worldwide'
    platform: string;
    license_type: 'exclusive' | 'non-exclusive';
    quality: string[]; // ['UHD', '4K', 'HDR10']
    audio_formats: string[];
    price?: number;
    currency?: string;
  };
}

// ============================================================================
// ANALYTICS MODELS
// ============================================================================

/**
 * Content performance metrics
 * Collection: content_metrics
 */
export interface ContentMetrics {
  content_id: string;

  // Engagement
  total_views: number;
  unique_viewers: number;
  avg_watch_time_minutes: number;
  completion_rate: number; // 0-1

  // Ratings
  avg_rating: number;
  total_ratings: number;
  rating_distribution: Record<string, number>; // { '1': 10, '2': 5, '3': 20, ... }

  // Recommendations
  total_recommendations: number;
  recommendation_ctr: number;

  // Time-based metrics
  views_last_24h: number;
  views_last_7d: number;
  views_last_30d: number;

  // Trending score (calculated)
  trending_score: number;

  // Updated timestamp
  updated_at: Timestamp;
}

// ============================================================================
// SYSTEM MODELS
// ============================================================================

/**
 * API request tracking for rate limiting
 * Collection: api_requests
 */
export interface ApiRequest {
  // Identity
  request_id: string;

  // Client
  user_id?: string;
  api_key?: string;
  ip_address: string;

  // Request details
  endpoint: string;
  method: string;
  status_code: number;
  response_time_ms: number;

  // Timestamp
  timestamp: Timestamp;

  // Errors
  error?: string;
}

/**
 * Cache entry for EIDR lookups
 * Collection: eidr_cache
 */
export interface EidrCacheEntry {
  // Key: `${title}_${year}`
  key: string;

  // EIDR result
  eidr_id: string;
  title: string;
  year: number;

  // Confidence
  match_confidence: number;

  // Timestamps
  cached_at: Timestamp;
  expires_at: Timestamp;
}

// ============================================================================
// HELPER TYPES
// ============================================================================

/**
 * Firestore server timestamp helper
 */
export type ServerTimestamp = FieldValue;

/**
 * Partial update type
 */
export type PartialUpdate<T> = Partial<T> & {
  updated_at: Timestamp | ServerTimestamp;
};
