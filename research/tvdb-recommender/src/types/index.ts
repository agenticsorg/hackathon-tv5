/**
 * TVDB Self-Learning Recommendation System - Type Definitions
 */

// ============================================================================
// TVDB API Types
// ============================================================================

export interface TVDBAuthResponse {
  status: 'success' | 'failure';
  data?: {
    token: string;
  };
  message?: string;
}

export interface TVDBSearchResult {
  objectID: string;
  id: string;
  type: 'series' | 'movie' | 'person' | 'company';
  name: string;
  year?: string;
  image_url?: string;
  thumbnail?: string;
  status?: string;
  overview?: string;
  primary_language?: string;
  country?: string;
  network?: string;
  remote_ids?: RemoteId[];
}

export interface RemoteId {
  id: string;
  type: number;
  sourceName: string;
}

export interface SeriesBaseRecord {
  id: number;
  name: string;
  slug: string;
  image: string;
  nameTranslations: string[];
  overviewTranslations: string[];
  aliases: Alias[];
  firstAired: string;
  lastAired: string;
  nextAired: string;
  score: number;
  status: Status;
  originalCountry: string;
  originalLanguage: string;
  defaultSeasonType: number;
  isOrderRandomized: boolean;
  lastUpdated: string;
  averageRuntime: number;
  episodes: EpisodeBaseRecord[] | null;
  overview: string;
  year: string;
}

export interface SeriesExtendedRecord extends SeriesBaseRecord {
  artworks: Artwork[];
  networks: Network[];
  genres: Genre[];
  trailers: Trailer[];
  lists: List[];
  remoteIds: RemoteId[];
  characters: Character[];
  airsDays: AirsDays;
  airsTime: string;
  seasons: SeasonBaseRecord[];
  tags: Tag[];
  contentRatings: ContentRating[];
  originalNetwork: Network;
}

export interface EpisodeBaseRecord {
  id: number;
  seriesId: number;
  name: string;
  aired: string;
  runtime: number;
  nameTranslations: string[];
  overview: string;
  overviewTranslations: string[];
  image: string;
  imageType: number;
  isMovie: number;
  seasons: SeasonBaseRecord[];
  number: number;
  absoluteNumber: number;
  seasonNumber: number;
  lastUpdated: string;
  finaleType: string | null;
  airsBeforeSeason: number | null;
  airsBeforeEpisode: number | null;
  airsAfterSeason: number | null;
  year: string;
}

export interface MovieBaseRecord {
  id: number;
  name: string;
  slug: string;
  image: string;
  nameTranslations: string[];
  overviewTranslations: string[];
  aliases: Alias[];
  score: number;
  status: Status;
  runtime: number;
  lastUpdated: string;
  year: string;
}

export interface Alias {
  language: string;
  name: string;
}

export interface Status {
  id: number;
  name: string;
  recordType: string;
  keepUpdated: boolean;
}

export interface Artwork {
  id: number;
  image: string;
  thumbnail: string;
  language: string;
  type: number;
  score: number;
  width: number;
  height: number;
}

export interface Network {
  id: number;
  name: string;
  slug: string;
  nameTranslations: string[];
  overviewTranslations: string[];
  abbreviation: string;
  country: string;
  primaryCompanyType: number;
}

export interface Genre {
  id: number;
  name: string;
  slug: string;
}

export interface Trailer {
  id: number;
  name: string;
  url: string;
  language: string;
  runtime: number;
}

export interface List {
  id: number;
  name: string;
  overview: string;
  url: string;
  isOfficial: boolean;
}

export interface Character {
  id: number;
  name: string;
  peopleId: number;
  seriesId: number;
  series: SeriesBaseRecord | null;
  movie: MovieBaseRecord | null;
  movieId: number | null;
  episodeId: number | null;
  type: number;
  image: string;
  sort: number;
  isFeatured: boolean;
  url: string;
  nameTranslations: string[];
  overviewTranslations: string[];
  aliases: Alias[];
  peopleType: string;
  personName: string;
  tagOptions: TagOption[];
  personImgURL: string;
}

export interface TagOption {
  id: number;
  tag: number;
  tagName: string;
  name: string;
  helpText: string;
}

export interface AirsDays {
  sunday: boolean;
  monday: boolean;
  tuesday: boolean;
  wednesday: boolean;
  thursday: boolean;
  friday: boolean;
  saturday: boolean;
}

export interface SeasonBaseRecord {
  id: number;
  seriesId: number;
  type: SeasonType;
  name: string;
  number: number;
  nameTranslations: string[];
  overviewTranslations: string[];
  image: string;
  imageType: number;
  companies: Companies;
  lastUpdated: string;
}

export interface SeasonType {
  id: number;
  name: string;
  type: string;
  alternateName: string | null;
}

export interface Companies {
  studio: Company[];
  network: Company[];
  production: Company[];
  distributor: Company[];
  special_effects: Company[];
}

export interface Company {
  id: number;
  name: string;
  slug: string;
  nameTranslations: string[];
  overviewTranslations: string[];
  aliases: Alias[];
  country: string;
  primaryCompanyType: number;
  activeDate: string;
  inactiveDate: string;
  companyType: CompanyType;
  parentCompany: ParentCompany;
  tagOptions: TagOption[];
}

export interface CompanyType {
  companyTypeId: number;
  companyTypeName: string;
}

export interface ParentCompany {
  id: number;
  name: string;
  relation: Relation;
}

export interface Relation {
  id: number;
  typeName: string;
}

export interface Tag {
  id: number;
  tag: number;
  tagName: string;
  name: string;
  helpText: string | null;
}

export interface ContentRating {
  id: number;
  name: string;
  country: string;
  description: string;
  contentType: string;
  order: number;
  fullName: string;
}

// ============================================================================
// Embedding & Vector Types
// ============================================================================

export interface ContentEmbedding {
  contentId: string;
  contentType: 'series' | 'movie';
  vector: Float32Array;
  metadata: ContentMetadata;
  createdAt: Date;
  updatedAt: Date;
}

export interface ContentMetadata {
  title: string;
  year: number;
  genres: string[];
  overview: string;
  rating: number;
  networkId?: number;
  networkName?: string;
  status?: string;
  language?: string;
  country?: string;
}

export interface EmbeddingSearchResult {
  contentId: string;
  contentType: 'series' | 'movie';
  similarity: number;
  metadata: ContentMetadata;
}

// ============================================================================
// User & Preference Types
// ============================================================================

export interface User {
  id: string;
  email?: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface UserPreference {
  userId: string;
  preferenceVector: Float32Array;
  genreWeights: Record<string, number>;
  networkWeights: Record<string, number>;
  watchHistory: WatchHistoryItem[];
  ratings: Rating[];
  lastUpdated: Date;
}

export interface WatchHistoryItem {
  contentId: string;
  contentType: 'series' | 'movie';
  watchedAt: Date;
  completionPercentage: number;
  episodeId?: number;
  seasonNumber?: number;
  episodeNumber?: number;
  duration: number; // in seconds
}

export interface Rating {
  contentId: string;
  contentType: 'series' | 'movie';
  rating: number; // 1-10
  ratedAt: Date;
}

export interface UserInteraction {
  userId: string;
  contentId: string;
  contentType: 'series' | 'movie';
  interactionType: 'view' | 'click' | 'search' | 'add_watchlist' | 'remove_watchlist' | 'share';
  timestamp: Date;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// Recommendation Types
// ============================================================================

export interface Recommendation {
  contentId: string;
  contentType: 'series' | 'movie';
  score: number;
  reason: RecommendationReason;
  metadata: ContentMetadata;
  position: number;
}

export interface RecommendationReason {
  type: 'similar_to' | 'because_you_watched' | 'genre_match' | 'trending' | 'personalized' | 'cold_start';
  explanation: string;
  sourceContentIds?: string[];
  matchedGenres?: string[];
  patternId?: number;
}

export interface RecommendationRequest {
  userId: string;
  limit?: number;
  offset?: number;
  contentType?: 'series' | 'movie' | 'all';
  genres?: string[];
  excludeWatched?: boolean;
  context?: RecommendationContext;
}

export interface RecommendationContext {
  timeOfDay: 'morning' | 'afternoon' | 'evening' | 'night';
  dayOfWeek: string;
  platform: 'web' | 'mobile' | 'tv';
  sessionDuration?: number;
}

export interface RecommendationResponse {
  recommendations: Recommendation[];
  total: number;
  generatedAt: Date;
  patternUsed?: string;
  learningFeedback?: LearningFeedback;
}

// ============================================================================
// Learning & Pattern Types
// ============================================================================

export interface RecommendationPattern {
  patternId: number;
  taskType: PatternTaskType;
  context: PatternContext;
  approach: string;
  successRate: number;
  totalUses: number;
  avgReward: number;
  embedding?: Float32Array;
  createdAt: Date;
  updatedAt: Date;
}

export type PatternTaskType =
  | 'cold_start'
  | 'genre_match'
  | 'similar_content'
  | 'trending'
  | 'personalized'
  | 'time_based'
  | 'network_based'
  | 'actor_based';

export interface PatternContext {
  userSegment: 'new' | 'casual' | 'regular' | 'power';
  timeOfDay: string;
  dayOfWeek: string;
  platform: string;
  contentTypePreference?: 'series' | 'movie' | 'both';
  avgWatchDuration?: number;
  topGenres?: string[];
}

export interface LearningFeedback {
  patternId: number;
  wasSuccessful: boolean;
  reward: number;
  userAction: 'watched' | 'skipped' | 'added_watchlist' | 'rated';
  timestamp: Date;
}

export interface ReflexionEpisode {
  episodeId: number;
  context: string;
  action: string;
  outcome: string;
  reward: number;
  selfCritique: string;
  timestamp: Date;
}

export interface LearnedSkill {
  skillId: number;
  name: string;
  description: string;
  code: string;
  applicableContexts: string[];
  successRate: number;
  usageCount: number;
}

// ============================================================================
// Workflow & Agent Types
// ============================================================================

export interface WorkflowTask {
  id: string;
  type: WorkflowTaskType;
  status: 'pending' | 'in_progress' | 'completed' | 'failed';
  input: Record<string, unknown>;
  output?: Record<string, unknown>;
  error?: string;
  startedAt?: Date;
  completedAt?: Date;
  agentId?: string;
}

export type WorkflowTaskType =
  | 'fetch_content'
  | 'generate_embeddings'
  | 'update_user_profile'
  | 'generate_recommendations'
  | 'learn_from_feedback'
  | 'consolidate_patterns'
  | 'sync_tvdb_updates';

export interface AgentConfig {
  id: string;
  type: AgentType;
  capabilities: string[];
  memoryNamespace: string;
  hooks: AgentHook[];
}

export type AgentType =
  | 'content_fetcher'
  | 'embeddings_generator'
  | 'recommendation_engine'
  | 'learning_agent'
  | 'user_profile_manager'
  | 'coordinator';

export interface AgentHook {
  event: 'pre_task' | 'post_task' | 'on_error' | 'on_success';
  action: string;
  config?: Record<string, unknown>;
}

export interface MemoryEntry {
  key: string;
  value: unknown;
  namespace: string;
  ttl?: number;
  createdAt: Date;
  updatedAt: Date;
}

// ============================================================================
// Configuration Types
// ============================================================================

export interface AppConfig {
  tvdb: TVDBConfig;
  database: DatabaseConfig;
  agentdb: AgentDBConfig;
  embedding: EmbeddingConfig;
  recommendation: RecommendationConfig;
  learning: LearningConfig;
}

export interface TVDBConfig {
  apiKey: string;
  pin?: string;
  baseUrl: string;
  cacheTtlSeconds: number;
  rateLimitPerMinute: number;
}

export interface DatabaseConfig {
  url: string;
  poolSize: number;
  ssl: boolean;
}

export interface AgentDBConfig {
  path: string;
  backend: 'ruvector' | 'hnsw' | 'sqlite' | 'auto';
  embeddingModel: string;
  embeddingDimension: number;
}

export interface EmbeddingConfig {
  model: string;
  dimension: number;
  batchSize: number;
  cacheEnabled: boolean;
}

export interface RecommendationConfig {
  defaultLimit: number;
  maxLimit: number;
  similarityThreshold: number;
  diversityFactor: number;
  coldStartStrategy: 'trending' | 'popular' | 'genre_based';
}

export interface LearningConfig {
  enabled: boolean;
  minSamplesForTraining: number;
  consolidationSchedule: string; // cron expression
  gnnEnabled: boolean;
  rewardDecayFactor: number;
}

// ============================================================================
// API Response Types
// ============================================================================

export interface APIResponse<T> {
  success: boolean;
  data?: T;
  error?: APIError;
  meta?: APIMeta;
}

export interface APIError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}

export interface APIMeta {
  requestId: string;
  timestamp: Date;
  duration: number;
  version: string;
}

// ============================================================================
// Event Types
// ============================================================================

export interface SystemEvent {
  type: SystemEventType;
  payload: Record<string, unknown>;
  timestamp: Date;
  source: string;
}

export type SystemEventType =
  | 'content_indexed'
  | 'user_action'
  | 'recommendation_generated'
  | 'pattern_learned'
  | 'pattern_consolidated'
  | 'tvdb_sync_completed'
  | 'error_occurred';
