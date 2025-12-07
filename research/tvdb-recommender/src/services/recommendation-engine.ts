/**
 * Self-Learning Recommendation Engine
 * Uses AgentDB's ReasoningBank for pattern learning and adaptive recommendations
 */

import { createDatabase, ReasoningBank, ReflexionMemory, SkillLibrary, EmbeddingService as AgentDBEmbedding } from 'agentdb';
import pino from 'pino';
import { v4 as uuidv4 } from 'uuid';
import {
  Recommendation,
  RecommendationRequest,
  RecommendationResponse,
  RecommendationReason,
  RecommendationPattern,
  PatternTaskType,
  PatternContext,
  LearningFeedback,
  ReflexionEpisode,
  ContentEmbedding,
  UserPreference,
  ContentMetadata,
  RecommendationConfig,
  LearningConfig
} from '../types/index.js';
import { EmbeddingService } from './embedding-service.js';

const logger = pino({ name: 'recommendation-engine' });

interface PatternSearchResult {
  patternId: number;
  taskType: PatternTaskType;
  approach: string;
  successRate: number;
  similarity: number;
}

export class RecommendationEngine {
  private db: any;
  private reasoningBank: ReasoningBank | null = null;
  private reflexionMemory: ReflexionMemory | null = null;
  private skillLibrary: SkillLibrary | null = null;
  private embedder: AgentDBEmbedding | null = null;
  private localEmbedder: EmbeddingService;
  private config: RecommendationConfig;
  private learningConfig: LearningConfig;
  private initialized: boolean = false;

  constructor(
    localEmbedder: EmbeddingService,
    config: Partial<RecommendationConfig> = {},
    learningConfig: Partial<LearningConfig> = {}
  ) {
    this.localEmbedder = localEmbedder;
    this.config = {
      defaultLimit: config.defaultLimit || 20,
      maxLimit: config.maxLimit || 100,
      similarityThreshold: config.similarityThreshold || 0.3,
      diversityFactor: config.diversityFactor || 0.2,
      coldStartStrategy: config.coldStartStrategy || 'trending'
    };
    this.learningConfig = {
      enabled: learningConfig.enabled !== false,
      minSamplesForTraining: learningConfig.minSamplesForTraining || 100,
      consolidationSchedule: learningConfig.consolidationSchedule || '0 3 * * *',
      gnnEnabled: learningConfig.gnnEnabled || false,
      rewardDecayFactor: learningConfig.rewardDecayFactor || 0.95
    };
  }

  /**
   * Initialize the recommendation engine with AgentDB
   */
  async initialize(dbPath: string = './data/recommendations.db'): Promise<void> {
    if (this.initialized) return;

    try {
      logger.info({ dbPath }, 'Initializing recommendation engine');

      // Create AgentDB database
      this.db = await createDatabase(dbPath);

      // Initialize AgentDB embedding service
      this.embedder = new AgentDBEmbedding({
        model: 'Xenova/all-MiniLM-L6-v2',
        dimension: 384
      });
      await this.embedder.initialize();

      // Initialize memory controllers
      this.reasoningBank = new ReasoningBank(this.db, this.embedder);
      this.reflexionMemory = new ReflexionMemory(this.db);
      this.skillLibrary = new SkillLibrary(this.db, this.embedder);

      // Seed initial patterns if empty
      await this.seedInitialPatterns();

      this.initialized = true;
      logger.info('Recommendation engine initialized successfully');
    } catch (error) {
      logger.error({ error }, 'Failed to initialize recommendation engine');
      throw error;
    }
  }

  /**
   * Ensure engine is initialized
   */
  private ensureInitialized(): void {
    if (!this.initialized) {
      throw new Error('RecommendationEngine not initialized. Call initialize() first.');
    }
  }

  /**
   * Generate recommendations for a user
   */
  async getRecommendations(
    request: RecommendationRequest,
    userPreference: UserPreference,
    contentEmbeddings: ContentEmbedding[]
  ): Promise<RecommendationResponse> {
    this.ensureInitialized();

    const startTime = Date.now();
    const limit = Math.min(request.limit || this.config.defaultLimit, this.config.maxLimit);

    try {
      // Determine user segment
      const userSegment = this.determineUserSegment(userPreference);

      // Build pattern context
      const context: PatternContext = {
        userSegment,
        timeOfDay: request.context?.timeOfDay || this.getCurrentTimeOfDay(),
        dayOfWeek: request.context?.dayOfWeek || this.getCurrentDayOfWeek(),
        platform: request.context?.platform || 'web',
        contentTypePreference: request.contentType === 'all' ? 'both' : request.contentType,
        topGenres: this.getTopGenres(userPreference, 3)
      };

      // Find best pattern for this context
      const pattern = await this.findBestPattern(context);
      logger.info({ pattern: pattern?.taskType, successRate: pattern?.successRate }, 'Selected recommendation pattern');

      // Generate recommendations based on pattern
      let recommendations: Recommendation[];

      if (userSegment === 'new') {
        // Cold start strategy
        recommendations = await this.coldStartRecommendations(
          contentEmbeddings,
          limit,
          request.genres
        );
      } else if (pattern) {
        // Use learned pattern
        recommendations = await this.patternBasedRecommendations(
          pattern,
          userPreference,
          contentEmbeddings,
          limit,
          request
        );
      } else {
        // Fallback to similarity-based
        recommendations = await this.similarityBasedRecommendations(
          userPreference,
          contentEmbeddings,
          limit,
          request
        );
      }

      // Apply diversity boosting
      recommendations = this.applyDiversity(recommendations, this.config.diversityFactor);

      // Filter watched content if requested
      if (request.excludeWatched) {
        const watchedIds = new Set(userPreference.watchHistory.map(w => w.contentId));
        recommendations = recommendations.filter(r => !watchedIds.has(r.contentId));
      }

      // Assign positions
      recommendations = recommendations.slice(0, limit).map((r, i) => ({
        ...r,
        position: i + 1
      }));

      const duration = Date.now() - startTime;

      return {
        recommendations,
        total: recommendations.length,
        generatedAt: new Date(),
        patternUsed: pattern?.approach,
        learningFeedback: pattern ? {
          patternId: pattern.patternId,
          wasSuccessful: false, // Will be updated when user interacts
          reward: 0,
          userAction: 'watched',
          timestamp: new Date()
        } : undefined
      };
    } catch (error) {
      logger.error({ error, userId: request.userId }, 'Failed to generate recommendations');
      throw error;
    }
  }

  /**
   * Find the best pattern for the given context
   */
  private async findBestPattern(context: PatternContext): Promise<PatternSearchResult | null> {
    if (!this.reasoningBank) return null;

    try {
      const contextText = this.contextToText(context);
      const patterns = await this.reasoningBank.searchPatterns({
        task: contextText,
        k: 5
      });

      if (patterns.length === 0) return null;

      // Select pattern with best combination of similarity and success rate
      const scored = patterns.map((p: any) => ({
        ...p,
        score: p.similarity * 0.4 + p.successRate * 0.6
      }));

      scored.sort((a: any, b: any) => b.score - a.score);
      return scored[0];
    } catch (error) {
      logger.error({ error }, 'Failed to find best pattern');
      return null;
    }
  }

  /**
   * Cold start recommendations for new users
   */
  private async coldStartRecommendations(
    contentEmbeddings: ContentEmbedding[],
    limit: number,
    genres?: string[]
  ): Promise<Recommendation[]> {
    let candidates = contentEmbeddings;

    // Filter by genres if specified
    if (genres && genres.length > 0) {
      const genreSet = new Set(genres.map(g => g.toLowerCase()));
      candidates = candidates.filter(c =>
        c.metadata.genres.some(g => genreSet.has(g.toLowerCase()))
      );
    }

    // Sort by rating (popularity proxy)
    candidates.sort((a, b) => (b.metadata.rating || 0) - (a.metadata.rating || 0));

    return candidates.slice(0, limit).map((content, i) => ({
      contentId: content.contentId,
      contentType: content.contentType,
      score: content.metadata.rating || 0,
      reason: {
        type: 'cold_start' as const,
        explanation: 'Popular content you might enjoy'
      },
      metadata: content.metadata,
      position: i + 1
    }));
  }

  /**
   * Pattern-based recommendations using learned strategies
   */
  private async patternBasedRecommendations(
    pattern: PatternSearchResult,
    userPreference: UserPreference,
    contentEmbeddings: ContentEmbedding[],
    limit: number,
    request: RecommendationRequest
  ): Promise<Recommendation[]> {
    switch (pattern.taskType) {
      case 'genre_match':
        return this.genreMatchRecommendations(userPreference, contentEmbeddings, limit);
      case 'similar_content':
        return this.similarityBasedRecommendations(userPreference, contentEmbeddings, limit, request);
      case 'time_based':
        return this.timeBasedRecommendations(userPreference, contentEmbeddings, limit, request);
      case 'network_based':
        return this.networkBasedRecommendations(userPreference, contentEmbeddings, limit);
      default:
        return this.similarityBasedRecommendations(userPreference, contentEmbeddings, limit, request);
    }
  }

  /**
   * Genre-match recommendations
   */
  private async genreMatchRecommendations(
    userPreference: UserPreference,
    contentEmbeddings: ContentEmbedding[],
    limit: number
  ): Promise<Recommendation[]> {
    const topGenres = this.getTopGenres(userPreference, 5);
    const genreSet = new Set(topGenres.map(g => g.toLowerCase()));

    const scored = contentEmbeddings.map(content => {
      const matchedGenres = content.metadata.genres.filter(g =>
        genreSet.has(g.toLowerCase())
      );
      return {
        content,
        score: matchedGenres.length / Math.max(content.metadata.genres.length, 1),
        matchedGenres
      };
    });

    scored.sort((a, b) => b.score - a.score);

    return scored.slice(0, limit).map((item, i) => ({
      contentId: item.content.contentId,
      contentType: item.content.contentType,
      score: item.score,
      reason: {
        type: 'genre_match' as const,
        explanation: `Matches your preferred genres: ${item.matchedGenres.join(', ')}`,
        matchedGenres: item.matchedGenres
      },
      metadata: item.content.metadata,
      position: i + 1
    }));
  }

  /**
   * Similarity-based recommendations using user preference vector
   */
  private async similarityBasedRecommendations(
    userPreference: UserPreference,
    contentEmbeddings: ContentEmbedding[],
    limit: number,
    request: RecommendationRequest
  ): Promise<Recommendation[]> {
    if (!userPreference.preferenceVector || userPreference.preferenceVector.length === 0) {
      return this.coldStartRecommendations(contentEmbeddings, limit, request.genres);
    }

    const similar = this.localEmbedder.findSimilar(
      userPreference.preferenceVector,
      contentEmbeddings,
      limit * 2, // Get more for diversity
      this.config.similarityThreshold
    );

    return similar.slice(0, limit).map((item, i) => ({
      contentId: item.content.contentId,
      contentType: item.content.contentType,
      score: item.similarity,
      reason: {
        type: 'personalized' as const,
        explanation: 'Based on your viewing preferences'
      },
      metadata: item.content.metadata,
      position: i + 1
    }));
  }

  /**
   * Time-based recommendations
   */
  private async timeBasedRecommendations(
    userPreference: UserPreference,
    contentEmbeddings: ContentEmbedding[],
    limit: number,
    request: RecommendationRequest
  ): Promise<Recommendation[]> {
    const timeOfDay = request.context?.timeOfDay || this.getCurrentTimeOfDay();

    // Adjust for time of day (e.g., shorter content in morning, longer in evening)
    let candidates = contentEmbeddings;

    // For evening, prefer longer series
    if (timeOfDay === 'evening' || timeOfDay === 'night') {
      candidates.sort((a, b) => {
        const aIsSeries = a.contentType === 'series' ? 1 : 0;
        const bIsSeries = b.contentType === 'series' ? 1 : 0;
        return bIsSeries - aIsSeries;
      });
    }

    // Filter by content type if specified
    if (request.contentType && request.contentType !== 'all') {
      candidates = candidates.filter(c => c.contentType === request.contentType);
    }

    // Use similarity for final ranking
    return this.similarityBasedRecommendations(userPreference, candidates, limit, request);
  }

  /**
   * Network-based recommendations
   */
  private async networkBasedRecommendations(
    userPreference: UserPreference,
    contentEmbeddings: ContentEmbedding[],
    limit: number
  ): Promise<Recommendation[]> {
    const topNetworks = Object.entries(userPreference.networkWeights || {})
      .sort((a, b) => b[1] - a[1])
      .slice(0, 5)
      .map(([network]) => network.toLowerCase());

    if (topNetworks.length === 0) {
      return contentEmbeddings.slice(0, limit).map((content, i) => ({
        contentId: content.contentId,
        contentType: content.contentType,
        score: content.metadata.rating || 0,
        reason: {
          type: 'trending' as const,
          explanation: 'Popular content'
        },
        metadata: content.metadata,
        position: i + 1
      }));
    }

    const networkSet = new Set(topNetworks);

    const scored = contentEmbeddings
      .filter(c => c.metadata.networkName && networkSet.has(c.metadata.networkName.toLowerCase()))
      .map(content => ({
        content,
        score: userPreference.networkWeights?.[content.metadata.networkName!] || 0.5
      }));

    scored.sort((a, b) => b.score - a.score);

    return scored.slice(0, limit).map((item, i) => ({
      contentId: item.content.contentId,
      contentType: item.content.contentType,
      score: item.score,
      reason: {
        type: 'because_you_watched' as const,
        explanation: `From ${item.content.metadata.networkName}, a network you enjoy`
      },
      metadata: item.content.metadata,
      position: i + 1
    }));
  }

  /**
   * Apply diversity to recommendations
   */
  private applyDiversity(recommendations: Recommendation[], factor: number): Recommendation[] {
    if (factor <= 0 || recommendations.length <= 1) return recommendations;

    const result: Recommendation[] = [recommendations[0]];
    const remaining = recommendations.slice(1);

    while (result.length < recommendations.length && remaining.length > 0) {
      // Find the most different item from what's already selected
      let maxDiversity = -1;
      let maxIndex = 0;

      for (let i = 0; i < remaining.length; i++) {
        const item = remaining[i];
        let minSimilarity = 1;

        for (const selected of result) {
          // Genre overlap as similarity proxy
          const overlapGenres = item.metadata.genres.filter(g =>
            selected.metadata.genres.includes(g)
          );
          const similarity = overlapGenres.length /
            Math.max(item.metadata.genres.length, selected.metadata.genres.length, 1);
          minSimilarity = Math.min(minSimilarity, similarity);
        }

        const diversity = (1 - minSimilarity) * factor + item.score * (1 - factor);
        if (diversity > maxDiversity) {
          maxDiversity = diversity;
          maxIndex = i;
        }
      }

      result.push(remaining[maxIndex]);
      remaining.splice(maxIndex, 1);
    }

    return result;
  }

  /**
   * Record user feedback and learn from it
   */
  async recordFeedback(feedback: LearningFeedback): Promise<void> {
    if (!this.learningConfig.enabled) return;
    this.ensureInitialized();

    try {
      // Update pattern success rate
      if (this.reasoningBank && feedback.patternId) {
        await this.reasoningBank.recordOutcome(
          feedback.patternId,
          feedback.wasSuccessful,
          feedback.reward
        );
      }

      // Store reflexion episode for learning
      if (this.reflexionMemory) {
        await this.reflexionMemory.storeEpisode({
          context: `Pattern ${feedback.patternId} used for recommendation`,
          action: feedback.userAction,
          outcome: feedback.wasSuccessful ? 'User engaged with content' : 'User did not engage',
          reward: feedback.reward,
          selfCritique: feedback.wasSuccessful
            ? 'Pattern worked well for this user segment'
            : 'Need to refine pattern for this context'
        });
      }

      logger.info({ feedback }, 'Recorded learning feedback');
    } catch (error) {
      logger.error({ error, feedback }, 'Failed to record feedback');
    }
  }

  /**
   * Store a new learned pattern
   */
  async storePattern(pattern: Omit<RecommendationPattern, 'patternId' | 'createdAt' | 'updatedAt'>): Promise<number> {
    this.ensureInitialized();

    if (!this.reasoningBank) {
      throw new Error('ReasoningBank not initialized');
    }

    const contextText = this.contextToText(pattern.context);

    const patternId = await this.reasoningBank.storePattern({
      taskType: pattern.taskType,
      approach: pattern.approach,
      successRate: pattern.successRate,
      context: contextText
    });

    logger.info({ patternId, taskType: pattern.taskType }, 'Stored new pattern');
    return patternId;
  }

  /**
   * Get pattern statistics
   */
  getPatternStats(): { totalPatterns: number; avgSuccessRate: number } {
    if (!this.reasoningBank) {
      return { totalPatterns: 0, avgSuccessRate: 0 };
    }
    return this.reasoningBank.getPatternStats();
  }

  /**
   * Seed initial patterns for cold start
   */
  private async seedInitialPatterns(): Promise<void> {
    if (!this.reasoningBank) return;

    const stats = this.reasoningBank.getPatternStats();
    if (stats.totalPatterns > 0) return; // Already seeded

    const initialPatterns = [
      {
        taskType: 'cold_start' as PatternTaskType,
        approach: 'Show popular and trending content for new users',
        successRate: 0.6,
        context: { userSegment: 'new', timeOfDay: 'any', dayOfWeek: 'any', platform: 'any' }
      },
      {
        taskType: 'genre_match' as PatternTaskType,
        approach: 'Match user top genres for regular users',
        successRate: 0.75,
        context: { userSegment: 'regular', timeOfDay: 'any', dayOfWeek: 'any', platform: 'any' }
      },
      {
        taskType: 'similar_content' as PatternTaskType,
        approach: 'Use vector similarity for personalized recommendations',
        successRate: 0.8,
        context: { userSegment: 'power', timeOfDay: 'any', dayOfWeek: 'any', platform: 'any' }
      },
      {
        taskType: 'time_based' as PatternTaskType,
        approach: 'Recommend series in evening, movies in morning',
        successRate: 0.65,
        context: { userSegment: 'casual', timeOfDay: 'evening', dayOfWeek: 'any', platform: 'any' }
      }
    ];

    for (const pattern of initialPatterns) {
      await this.storePattern(pattern);
    }

    logger.info({ count: initialPatterns.length }, 'Seeded initial patterns');
  }

  /**
   * Determine user segment based on watch history
   */
  private determineUserSegment(userPreference: UserPreference): 'new' | 'casual' | 'regular' | 'power' {
    const watchCount = userPreference.watchHistory.length;

    if (watchCount === 0) return 'new';
    if (watchCount < 5) return 'casual';
    if (watchCount < 20) return 'regular';
    return 'power';
  }

  /**
   * Get top genres from user preferences
   */
  private getTopGenres(userPreference: UserPreference, count: number): string[] {
    return Object.entries(userPreference.genreWeights || {})
      .sort((a, b) => b[1] - a[1])
      .slice(0, count)
      .map(([genre]) => genre);
  }

  /**
   * Convert context to text for embedding
   */
  private contextToText(context: PatternContext): string {
    return `User segment: ${context.userSegment}. Time: ${context.timeOfDay}. Day: ${context.dayOfWeek}. Platform: ${context.platform}. Preference: ${context.contentTypePreference || 'any'}. Top genres: ${context.topGenres?.join(', ') || 'none'}.`;
  }

  /**
   * Get current time of day
   */
  private getCurrentTimeOfDay(): 'morning' | 'afternoon' | 'evening' | 'night' {
    const hour = new Date().getHours();
    if (hour >= 5 && hour < 12) return 'morning';
    if (hour >= 12 && hour < 17) return 'afternoon';
    if (hour >= 17 && hour < 21) return 'evening';
    return 'night';
  }

  /**
   * Get current day of week
   */
  private getCurrentDayOfWeek(): string {
    return ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'][new Date().getDay()];
  }
}

// Factory function
export function createRecommendationEngine(
  embedder: EmbeddingService,
  config?: Partial<RecommendationConfig>,
  learningConfig?: Partial<LearningConfig>
): RecommendationEngine {
  return new RecommendationEngine(embedder, config, learningConfig);
}

export default RecommendationEngine;
