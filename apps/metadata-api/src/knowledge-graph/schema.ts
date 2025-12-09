/**
 * Knowledge Graph Schema for TMDB → Distribution Pipeline
 *
 * This schema defines the hypergraph structure for the Nexus-UMMID platform.
 * It transforms TMDB movie data into a distribution-ready knowledge graph
 * that can generate Netflix IMF, Amazon MEC, and FAST MRSS feeds.
 */

// ============================================
// Node Types (Vertices in the Hypergraph)
// ============================================

export interface MovieNode {
  id: string;                      // TMDB ID
  type: 'movie';

  // Core Metadata
  title: string;
  originalTitle?: string;
  overview: string;
  tagline?: string;

  // Classification
  adult: boolean;
  status: string;                  // Released, Post Production, etc.

  // Dates & Numbers
  releaseDate?: string;
  year?: number;
  runtime?: number;                // minutes
  budget?: number;
  revenue?: number;

  // Ratings & Popularity
  voteAverage: number;
  voteCount: number;
  popularity: number;

  // Media Assets
  posterPath?: string;
  backdropPath?: string;
  homepage?: string;

  // External IDs (for platform mapping)
  imdbId?: string;

  // Vector Embedding (768-dim from Vertex AI)
  embedding?: number[];
  embeddingModel?: string;

  // Distribution Readiness
  distributionStatus: DistributionStatus;
  validationResults?: PlatformValidation[];

  // Timestamps
  createdAt: string;
  updatedAt: string;
  processedAt?: string;
}

export interface GenreNode {
  id: string;                      // Genre ID from TMDB
  type: 'genre';
  name: string;

  // Platform mappings
  netflixGenreCode?: string;
  amazonGenreId?: string;
  fastGenreCategory?: string;

  // Statistics
  movieCount: number;

  createdAt: string;
  updatedAt: string;
}

export interface ProductionCompanyNode {
  id: string;                      // Company ID from TMDB
  type: 'production_company';
  name: string;
  originCountry?: string;
  logoPath?: string;

  // Statistics
  movieCount: number;

  createdAt: string;
  updatedAt: string;
}

export interface CountryNode {
  id: string;                      // ISO 3166-1 alpha-2 code
  type: 'country';
  name: string;
  iso31661: string;

  // Territory mapping for distribution
  netflixTerritory?: string;
  amazonTerritory?: string;

  // Statistics
  movieCount: number;

  createdAt: string;
  updatedAt: string;
}

export interface LanguageNode {
  id: string;                      // ISO 639-1 code
  type: 'language';
  name: string;
  iso6391: string;
  englishName?: string;

  // BCP-47 tag for platform compliance
  bcp47Tag: string;

  // Statistics
  movieCount: number;

  createdAt: string;
  updatedAt: string;
}

export interface KeywordNode {
  id: string;                      // Keyword ID from TMDB
  type: 'keyword';
  name: string;

  // Semantic grouping
  category?: string;               // mood, theme, setting, etc.

  // Statistics
  movieCount: number;

  createdAt: string;
  updatedAt: string;
}

// Union type for all nodes
export type GraphNode =
  | MovieNode
  | GenreNode
  | ProductionCompanyNode
  | CountryNode
  | LanguageNode
  | KeywordNode;

// ============================================
// Hyperedge Types (N-ary Relationships)
// ============================================

export interface HyperedgeBase {
  id: string;
  type: HyperedgeType;
  createdAt: string;
  updatedAt: string;
}

export type HyperedgeType =
  | 'HAS_GENRE'
  | 'PRODUCED_BY'
  | 'PRODUCED_IN'
  | 'SPOKEN_IN'
  | 'HAS_KEYWORD'
  | 'DISTRIBUTION_RIGHT'
  | 'SIMILAR_TO';

export interface MovieGenreEdge extends HyperedgeBase {
  type: 'HAS_GENRE';
  movieId: string;
  genreId: string;
  primary: boolean;                // Is this the primary genre?
}

export interface MovieCompanyEdge extends HyperedgeBase {
  type: 'PRODUCED_BY';
  movieId: string;
  companyId: string;
  role?: 'producer' | 'distributor';
}

export interface MovieCountryEdge extends HyperedgeBase {
  type: 'PRODUCED_IN';
  movieId: string;
  countryId: string;
}

export interface MovieLanguageEdge extends HyperedgeBase {
  type: 'SPOKEN_IN';
  movieId: string;
  languageId: string;
  primary: boolean;
}

export interface MovieKeywordEdge extends HyperedgeBase {
  type: 'HAS_KEYWORD';
  movieId: string;
  keywordId: string;
  relevanceScore?: number;
}

// Distribution rights hyperedge (N-ary relationship)
export interface DistributionRightEdge extends HyperedgeBase {
  type: 'DISTRIBUTION_RIGHT';

  // Connected nodes
  movieId: string;
  territoryId: string;             // Country ID
  platformId: string;              // netflix, amazon, fast-pluto, etc.

  // Temporal validity (bitemporal)
  validFrom: string;
  validTo: string;
  transactionTime: string;         // When this data was entered

  // Rights details
  licenseType: 'exclusive' | 'non-exclusive';
  distributionType: 'SVOD' | 'TVOD' | 'AVOD' | 'FAST';

  // Quality constraints
  maxResolution?: 'SD' | 'HD' | 'UHD' | '4K';
  audioRequirements?: string[];

  // Status
  status: 'active' | 'expired' | 'pending' | 'conflict';
}

// Similarity edge (for recommendations)
export interface SimilarityEdge extends HyperedgeBase {
  type: 'SIMILAR_TO';
  movieId1: string;
  movieId2: string;
  similarityScore: number;         // 0-1 cosine similarity
  similarityType: 'content' | 'genre' | 'keyword' | 'hybrid';
}

// Union type for all edges
export type Hyperedge =
  | MovieGenreEdge
  | MovieCompanyEdge
  | MovieCountryEdge
  | MovieLanguageEdge
  | MovieKeywordEdge
  | DistributionRightEdge
  | SimilarityEdge;

// ============================================
// Distribution Types
// ============================================

export type DistributionStatus =
  | 'pending'           // Not yet processed
  | 'validated'         // Passed validation
  | 'enriched'          // AI enrichment complete
  | 'ready'             // Ready for distribution
  | 'delivered'         // Sent to platforms
  | 'failed';           // Validation failed

export interface PlatformValidation {
  platform: 'netflix' | 'amazon' | 'fast';
  valid: boolean;
  score: number;                   // 0-100 compliance score
  errors: ValidationError[];
  warnings: ValidationWarning[];
  validatedAt: string;
}

export interface ValidationError {
  field: string;
  message: string;
  severity: 'critical' | 'error';
  code: string;
}

export interface ValidationWarning {
  field: string;
  message: string;
  recommendation: string;
}

// ============================================
// TMDB CSV Row Type (Input)
// ============================================

export interface TMDBMovieRow {
  id: string;
  title: string;
  vote_average: string;
  vote_count: string;
  status: string;
  release_date: string;
  revenue: string;
  runtime: string;
  adult: string;
  backdrop_path: string;
  budget: string;
  homepage: string;
  imdb_id: string;
  original_language: string;
  original_title: string;
  overview: string;
  popularity: string;
  poster_path: string;
  tagline: string;
  genres: string;                  // JSON string: [{"id": 28, "name": "Action"}]
  production_companies: string;     // JSON string
  production_countries: string;     // JSON string
  spoken_languages: string;         // JSON string
  keywords: string;                 // JSON string
}

// ============================================
// Statistics & Analytics
// ============================================

export interface KnowledgeGraphStats {
  totalMovies: number;
  totalGenres: number;
  totalCompanies: number;
  totalCountries: number;
  totalLanguages: number;
  totalKeywords: number;
  totalEdges: number;

  // Distribution readiness
  readyForNetflix: number;
  readyForAmazon: number;
  readyForFAST: number;

  // Processing status
  pendingProcessing: number;
  failedValidation: number;

  lastUpdated: string;
}

// ============================================
// Query Types
// ============================================

export interface GraphQuery {
  nodeTypes?: string[];
  filters?: Record<string, any>;
  genres?: string[];
  countries?: string[];
  languages?: string[];
  yearRange?: { min: number; max: number };
  ratingRange?: { min: number; max: number };
  limit?: number;
  offset?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface SemanticSearchQuery {
  query: string;
  limit?: number;
  filters?: GraphQuery;
}

export interface GraphQueryResult {
  nodes: GraphNode[];
  edges: Hyperedge[];
  totalCount: number;
  hasMore: boolean;
}

// ============================================
// Platform Feed Types
// ============================================

export interface NetflixIMFRequest {
  movieId: string;
  territory: string;
  includeArtwork: boolean;
  includeDolbyVision?: boolean;
}

export interface AmazonMECRequest {
  movieId: string;
  territories: string[];
  transactionType: 'EST' | 'VOD' | 'SVOD' | 'AVOD' | 'TVOD';
}

export interface FASTMRSSRequest {
  genre: string;
  platform: 'pluto' | 'tubi' | 'roku' | 'samsung' | 'vizio';
  limit?: number;
  includeSchedule?: boolean;
}
