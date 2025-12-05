/**
 * Core type definitions for the AI Media Discovery Platform
 */

// Base media content type
export interface MediaContent {
  id: number;
  title: string;
  overview: string;
  posterPath: string | null;
  backdropPath: string | null;
  releaseDate: string;
  voteAverage: number;
  voteCount: number;
  popularity: number;
  genreIds: number[];
  mediaType: 'movie' | 'tv';
}

// Extended movie type
export interface Movie extends MediaContent {
  mediaType: 'movie';
  runtime?: number;
  budget?: number;
  revenue?: number;
  tagline?: string;
  status?: string;
}

// Extended TV show type
export interface TVShow extends MediaContent {
  mediaType: 'tv';
  name: string; // TV shows use 'name' instead of 'title' in TMDB
  firstAirDate: string;
  lastAirDate?: string;
  numberOfSeasons?: number;
  numberOfEpisodes?: number;
  episodeRunTime?: number[];
  status?: string;
  inProduction?: boolean;
}

// Genre type
export interface Genre {
  id: number;
  name: string;
}

// Search query with semantic understanding
export interface SemanticSearchQuery {
  query: string;
  intent?: SearchIntent;
  filters?: SearchFilters;
  embedding?: number[];
}

// Search intent derived from natural language
export interface SearchIntent {
  mood?: string[];           // e.g., ["exciting", "suspenseful"]
  themes?: string[];         // e.g., ["redemption", "love"]
  pacing?: 'slow' | 'medium' | 'fast';
  era?: string;              // e.g., "1980s", "modern"
  setting?: string[];        // e.g., ["space", "urban"]
  similar_to?: string[];     // e.g., ["Inception", "The Matrix"]
  avoid?: string[];          // e.g., ["gore", "romance"]
}

// Search filters
export interface SearchFilters {
  mediaType?: 'movie' | 'tv' | 'all';
  genres?: number[];
  yearRange?: { min?: number; max?: number };
  ratingMin?: number;
  language?: string;
  region?: string;
}

// Search results with relevance scoring
export interface SearchResult {
  content: MediaContent;
  relevanceScore: number;
  matchReasons: string[];
  similarityScore?: number;
}

// User preference profile
export interface UserPreferences {
  userId: string;
  favoriteGenres: number[];
  likedContent: number[];
  dislikedContent: number[];
  watchHistory: WatchHistoryEntry[];
  preferenceVector?: number[];
  updatedAt: Date;
}

// Watch history entry
export interface WatchHistoryEntry {
  contentId: number;
  mediaType: 'movie' | 'tv';
  watchedAt: Date;
  progress: number; // 0-1 percentage watched
  rating?: number;  // 1-10 user rating
  completed: boolean;
}

// Recommendation with explanation
export interface Recommendation {
  content: MediaContent;
  score: number;
  reasons: string[];
  basedOn?: {
    type: 'similar' | 'genre' | 'history' | 'trending';
    references: string[];
  };
}

// Content embedding for vector search
export interface ContentEmbedding {
  contentId: number;
  mediaType: 'movie' | 'tv';
  embedding: number[];
  metadata: {
    title: string;
    genres: string[];
    keywords: string[];
    synopsis: string;
  };
  createdAt: Date;
  model: string;
}

// ARW Discovery manifest types
export interface ARWMediaManifest {
  version: string;
  profile: 'ARW-1';
  site: {
    name: string;
    description: string;
    homepage: string;
    contact?: string;
  };
  content: ARWContentEntry[];
  actions: ARWAction[];
  policies: ARWPolicies;
}

export interface ARWContentEntry {
  url: string;
  machine_view?: string;
  purpose: 'search' | 'browse' | 'details' | 'recommendations';
  priority: 'high' | 'medium' | 'low';
}

export interface ARWAction {
  id: string;
  name: string;
  endpoint: string;
  method: 'GET' | 'POST';
  description: string;
  schema?: object;
}

export interface ARWPolicies {
  training: { allowed: boolean; note?: string };
  inference: { allowed: boolean; restrictions?: string[] };
  attribution: { required: boolean; format?: string };
}
