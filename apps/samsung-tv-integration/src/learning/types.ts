import { z } from 'zod';

// Content types for TV viewing
export const ContentTypeSchema = z.enum([
  'movie',
  'tv_show',
  'documentary',
  'sports',
  'news',
  'music',
  'kids',
  'gaming',
]);

export type ContentType = z.infer<typeof ContentTypeSchema>;

// Genre schema
export const GenreSchema = z.enum([
  'action',
  'adventure',
  'animation',
  'comedy',
  'crime',
  'documentary',
  'drama',
  'family',
  'fantasy',
  'history',
  'horror',
  'music',
  'mystery',
  'romance',
  'science_fiction',
  'thriller',
  'war',
  'western',
  'reality',
  'sports',
  'news',
]);

export type Genre = z.infer<typeof GenreSchema>;

// Content metadata for embedding
export const ContentMetadataSchema = z.object({
  id: z.string(),
  title: z.string(),
  type: ContentTypeSchema,
  genres: z.array(GenreSchema).default([]),
  duration: z.number().optional(), // minutes
  releaseYear: z.number().optional(),
  rating: z.number().min(0).max(10).optional(),
  popularity: z.number().min(0).max(100).optional(),
  description: z.string().optional(),
  actors: z.array(z.string()).default([]),
  directors: z.array(z.string()).default([]),
  keywords: z.array(z.string()).default([]),
  appId: z.string().optional(), // streaming app ID
  appName: z.string().optional(),
});

export type ContentMetadata = z.infer<typeof ContentMetadataSchema>;

// Viewing session tracking
export const ViewingSessionSchema = z.object({
  id: z.string(),
  contentId: z.string(),
  contentMetadata: ContentMetadataSchema,
  startTime: z.string().datetime(),
  endTime: z.string().datetime().optional(),
  watchDuration: z.number().default(0), // minutes
  completionRate: z.number().min(0).max(1).default(0),
  userRating: z.number().min(1).max(5).optional(),
  implicit: z.object({
    paused: z.number().default(0), // pause count
    rewound: z.number().default(0), // rewind count
    fastForwarded: z.number().default(0),
    volumeChanges: z.number().default(0),
  }).default({}),
  contextual: z.object({
    timeOfDay: z.enum(['morning', 'afternoon', 'evening', 'night']),
    dayOfWeek: z.enum(['weekday', 'weekend']),
    isAlone: z.boolean().optional(),
  }).optional(),
});

export type ViewingSession = z.infer<typeof ViewingSessionSchema>;

// User preference profile
export const UserPreferenceSchema = z.object({
  userId: z.string().default('default'),
  favoriteGenres: z.array(GenreSchema).default([]),
  favoriteTypes: z.array(ContentTypeSchema).default([]),
  preferredDuration: z.object({
    min: z.number().default(0),
    max: z.number().default(180),
  }).default({}),
  preferredTimeSlots: z.record(z.string(), z.array(ContentTypeSchema)).default({}),
  dislikedGenres: z.array(GenreSchema).default([]),
  watchedContentIds: z.array(z.string()).default([]),
  lastUpdated: z.string().datetime().optional(),
});

export type UserPreference = z.infer<typeof UserPreferenceSchema>;

// Learning action for Q-learning
export const LearningActionSchema = z.enum([
  'recommend_similar',
  'recommend_popular',
  'recommend_trending',
  'recommend_genre',
  'recommend_new_release',
  'recommend_continue_watching',
  'recommend_based_on_time',
  'explore_new_genre',
  'explore_new_type',
]);

export type LearningAction = z.infer<typeof LearningActionSchema>;

// Learning state representation
export const LearningStateSchema = z.object({
  timeOfDay: z.enum(['morning', 'afternoon', 'evening', 'night']),
  dayOfWeek: z.enum(['weekday', 'weekend']),
  recentGenres: z.array(GenreSchema).max(5),
  recentTypes: z.array(ContentTypeSchema).max(3),
  sessionCount: z.number(),
  avgCompletionRate: z.number(),
  lastContentId: z.string().optional(),
});

export type LearningState = z.infer<typeof LearningStateSchema>;

// Recommendation result
export const RecommendationSchema = z.object({
  contentId: z.string(),
  title: z.string(),
  type: ContentTypeSchema,
  genres: z.array(GenreSchema),
  score: z.number().min(0).max(1),
  reason: z.string(),
  action: LearningActionSchema,
  confidence: z.number().min(0).max(1),
  appId: z.string().optional(),
});

export type Recommendation = z.infer<typeof RecommendationSchema>;

// Pattern for ReasoningBank storage
export const ViewingPatternSchema = z.object({
  patternId: z.string(),
  state: LearningStateSchema,
  action: LearningActionSchema,
  reward: z.number(),
  successRate: z.number(),
  occurrences: z.number(),
  embedding: z.array(z.number()).optional(),
  createdAt: z.string().datetime(),
  updatedAt: z.string().datetime(),
});

export type ViewingPattern = z.infer<typeof ViewingPatternSchema>;

// Learning configuration
export const LearningConfigSchema = z.object({
  learningRate: z.number().min(0).max(1).default(0.1),
  discountFactor: z.number().min(0).max(1).default(0.95),
  explorationRate: z.number().min(0).max(1).default(0.15),
  minExploration: z.number().min(0).max(1).default(0.05),
  explorationDecay: z.number().min(0).max(1).default(0.995),
  batchSize: z.number().default(32),
  memorySize: z.number().default(10000),
  embeddingDimension: z.number().default(384),
  similarityThreshold: z.number().min(0).max(1).default(0.7),
});

export type LearningConfig = z.infer<typeof LearningConfigSchema>;

// Feedback for learning
export interface LearningFeedback {
  recommendationId: string;
  contentId: string;
  action: LearningAction;
  selected: boolean;
  watchDuration?: number;
  completionRate?: number;
  userRating?: number;
  timestamp: string;
}

// Stats for the learning system
export interface LearningStats {
  totalSessions: number;
  totalPatterns: number;
  avgReward: number;
  explorationRate: number;
  topActions: Array<{ action: LearningAction; count: number; avgReward: number }>;
  learningProgress: number; // 0-1
  lastTrainingTime?: string;
}
