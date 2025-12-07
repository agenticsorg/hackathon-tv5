/**
 * Agentic Workflow Orchestration
 * Coordinates multi-agent recommendation and learning workflows
 */

import pino from 'pino';
import { v4 as uuidv4 } from 'uuid';
import {
  WorkflowTask,
  WorkflowTaskType,
  AgentConfig,
  AgentType,
  RecommendationRequest,
  RecommendationResponse,
  LearningFeedback,
  ContentEmbedding,
  UserPreference,
  SystemEvent
} from '../types/index.js';
import { TVDBClient } from '../services/tvdb-client.js';
import { EmbeddingService } from '../services/embedding-service.js';
import { RecommendationEngine } from '../services/recommendation-engine.js';
import { Repository } from '../db/repository.js';

const logger = pino({ name: 'workflow' });

interface WorkflowContext {
  sessionId: string;
  userId?: string;
  startTime: Date;
  tasks: WorkflowTask[];
  events: SystemEvent[];
}

interface AgentResult {
  agentId: string;
  taskId: string;
  success: boolean;
  data?: any;
  error?: string;
  duration: number;
}

export class RecommendationWorkflow {
  private tvdbClient: TVDBClient;
  private embedder: EmbeddingService;
  private engine: RecommendationEngine;
  private repository: Repository;
  private activeContexts: Map<string, WorkflowContext> = new Map();

  constructor(
    tvdbClient: TVDBClient,
    embedder: EmbeddingService,
    engine: RecommendationEngine,
    repository: Repository
  ) {
    this.tvdbClient = tvdbClient;
    this.embedder = embedder;
    this.engine = engine;
    this.repository = repository;
  }

  /**
   * Initialize the workflow system
   */
  async initialize(): Promise<void> {
    logger.info('Initializing recommendation workflow');

    // Initialize all services
    await Promise.all([
      this.embedder.initialize(),
      this.engine.initialize(),
      this.repository.initialize()
    ]);

    logger.info('Recommendation workflow initialized');
  }

  /**
   * Start a new workflow session
   */
  startSession(userId?: string): string {
    const sessionId = uuidv4();
    const context: WorkflowContext = {
      sessionId,
      userId,
      startTime: new Date(),
      tasks: [],
      events: []
    };

    this.activeContexts.set(sessionId, context);

    this.emitEvent(sessionId, 'session_started', { userId });
    logger.info({ sessionId, userId }, 'Workflow session started');

    return sessionId;
  }

  /**
   * End a workflow session
   */
  endSession(sessionId: string): WorkflowContext | null {
    const context = this.activeContexts.get(sessionId);
    if (!context) return null;

    this.emitEvent(sessionId, 'session_ended', {
      duration: Date.now() - context.startTime.getTime(),
      tasksCompleted: context.tasks.filter(t => t.status === 'completed').length
    });

    this.activeContexts.delete(sessionId);
    logger.info({ sessionId }, 'Workflow session ended');

    return context;
  }

  /**
   * Execute the full recommendation workflow
   */
  async executeRecommendationWorkflow(
    sessionId: string,
    request: RecommendationRequest
  ): Promise<RecommendationResponse> {
    const context = this.activeContexts.get(sessionId);
    if (!context) {
      throw new Error(`Session ${sessionId} not found`);
    }

    context.userId = request.userId;
    const workflowId = uuidv4();

    try {
      // Task 1: Fetch user preferences
      const userPrefTask = this.createTask('update_user_profile', { userId: request.userId });
      context.tasks.push(userPrefTask);

      const userPreference = await this.executeTask(userPrefTask, async () => {
        let pref = await this.repository.getUserPreference(request.userId);
        if (!pref) {
          // Create new user and preferences
          await this.repository.createUser();
          pref = {
            userId: request.userId,
            preferenceVector: new Float32Array(384),
            genreWeights: {},
            networkWeights: {},
            watchHistory: [],
            ratings: [],
            lastUpdated: new Date()
          };
        }
        return pref;
      });

      // Task 2: Get content embeddings
      const contentTask = this.createTask('fetch_content', {
        contentType: request.contentType,
        genres: request.genres
      });
      context.tasks.push(contentTask);

      const contentEmbeddings = await this.executeTask(contentTask, async () => {
        // Get content from database with embeddings
        if (userPreference.preferenceVector.some(v => v !== 0)) {
          const results = await this.repository.getContentByVector(
            userPreference.preferenceVector,
            100,
            request.contentType === 'all' ? undefined : request.contentType
          );
          return results.map(r => r.embedding);
        } else {
          // Cold start - get popular content
          const results = await this.repository.searchContent('', 100);
          return results;
        }
      });

      // Task 3: Generate recommendations
      const recsTask = this.createTask('generate_recommendations', {
        userId: request.userId,
        contentCount: contentEmbeddings.length
      });
      context.tasks.push(recsTask);

      const recommendations = await this.executeTask(recsTask, async () => {
        return this.engine.getRecommendations(request, userPreference, contentEmbeddings);
      });

      this.emitEvent(sessionId, 'recommendation_generated', {
        userId: request.userId,
        count: recommendations.recommendations.length,
        pattern: recommendations.patternUsed
      });

      return recommendations;

    } catch (error) {
      logger.error({ error, sessionId, workflowId }, 'Recommendation workflow failed');
      this.emitEvent(sessionId, 'error_occurred', { error: String(error) });
      throw error;
    }
  }

  /**
   * Execute content ingestion workflow
   */
  async executeIngestionWorkflow(
    sessionId: string,
    query: string,
    limit: number = 50
  ): Promise<{ indexed: number; errors: number }> {
    const context = this.activeContexts.get(sessionId);
    if (!context) {
      throw new Error(`Session ${sessionId} not found`);
    }

    let indexed = 0;
    let errors = 0;

    try {
      // Task 1: Search TVDB
      const searchTask = this.createTask('fetch_content', { query, limit });
      context.tasks.push(searchTask);

      const searchResults = await this.executeTask(searchTask, async () => {
        return this.tvdbClient.search(query, { limit });
      });

      // Task 2: Generate embeddings and store
      const embeddingTask = this.createTask('generate_embeddings', {
        count: searchResults.length
      });
      context.tasks.push(embeddingTask);

      await this.executeTask(embeddingTask, async () => {
        const embeddings: ContentEmbedding[] = [];

        for (const result of searchResults) {
          try {
            if (result.type === 'series') {
              const series = await this.tvdbClient.getSeriesExtended(parseInt(result.id));
              const embedding = await this.embedder.embedSeries(series);
              embeddings.push(embedding);
              indexed++;
            } else if (result.type === 'movie') {
              const movie = await this.tvdbClient.getMovie(parseInt(result.id));
              // Add genres and overview if available from search
              const movieWithMeta = {
                ...movie,
                overview: result.overview,
                genres: [] // Would need extended movie endpoint
              };
              const embedding = await this.embedder.embedMovie(movieWithMeta);
              embeddings.push(embedding);
              indexed++;
            }
          } catch (err) {
            logger.warn({ err, id: result.id }, 'Failed to process content');
            errors++;
          }
        }

        // Batch insert to database
        if (embeddings.length > 0) {
          await this.repository.batchUpsertContent(embeddings);
        }

        return embeddings;
      });

      this.emitEvent(sessionId, 'content_indexed', { indexed, errors, query });

      return { indexed, errors };

    } catch (error) {
      logger.error({ error, sessionId }, 'Ingestion workflow failed');
      this.emitEvent(sessionId, 'error_occurred', { error: String(error) });
      throw error;
    }
  }

  /**
   * Execute learning workflow from user feedback
   */
  async executeLearningWorkflow(
    sessionId: string,
    feedback: LearningFeedback & { userId: string; contentId?: string }
  ): Promise<void> {
    const context = this.activeContexts.get(sessionId);
    if (!context) {
      throw new Error(`Session ${sessionId} not found`);
    }

    try {
      // Task 1: Record feedback
      const feedbackTask = this.createTask('learn_from_feedback', {
        userId: feedback.userId,
        patternId: feedback.patternId
      });
      context.tasks.push(feedbackTask);

      await this.executeTask(feedbackTask, async () => {
        // Store in PostgreSQL
        await this.repository.recordLearningFeedback(feedback);

        // Update AgentDB patterns
        await this.engine.recordFeedback(feedback);
      });

      // Task 2: Update user preferences if positive feedback
      if (feedback.wasSuccessful && feedback.contentId) {
        const prefTask = this.createTask('update_user_profile', {
          userId: feedback.userId,
          action: 'preference_update'
        });
        context.tasks.push(prefTask);

        await this.executeTask(prefTask, async () => {
          await this.repository.updateUserPreferenceVector(feedback.userId);
        });
      }

      this.emitEvent(sessionId, 'pattern_learned', {
        patternId: feedback.patternId,
        success: feedback.wasSuccessful,
        reward: feedback.reward
      });

    } catch (error) {
      logger.error({ error, sessionId }, 'Learning workflow failed');
      this.emitEvent(sessionId, 'error_occurred', { error: String(error) });
      throw error;
    }
  }

  /**
   * Execute TVDB sync workflow
   */
  async executeSyncWorkflow(
    sessionId: string,
    since: number,
    type: 'series' | 'movies' | 'episodes' = 'series'
  ): Promise<{ synced: number; errors: number }> {
    const context = this.activeContexts.get(sessionId);
    if (!context) {
      throw new Error(`Session ${sessionId} not found`);
    }

    let synced = 0;
    let errors = 0;

    try {
      // Task 1: Get updates from TVDB
      const updatesTask = this.createTask('sync_tvdb_updates', { since, type });
      context.tasks.push(updatesTask);

      const updates = await this.executeTask(updatesTask, async () => {
        return this.tvdbClient.getUpdates(since, { type });
      });

      logger.info({ count: updates.length, type }, 'Fetched TVDB updates');

      // Task 2: Process updates
      const processTask = this.createTask('generate_embeddings', {
        count: updates.length
      });
      context.tasks.push(processTask);

      await this.executeTask(processTask, async () => {
        for (const update of updates) {
          try {
            if (update.method === 'delete') {
              // Handle deletion if needed
              continue;
            }

            if (type === 'series') {
              const series = await this.tvdbClient.getSeriesExtended(update.id);
              const embedding = await this.embedder.embedSeries(series);
              await this.repository.upsertContent(embedding);
              synced++;
            } else if (type === 'movies') {
              const movie = await this.tvdbClient.getMovie(update.id);
              const embedding = await this.embedder.embedMovie({
                ...movie,
                genres: []
              });
              await this.repository.upsertContent(embedding);
              synced++;
            }
          } catch (err) {
            logger.warn({ err, id: update.id }, 'Failed to sync content');
            errors++;
          }
        }
      });

      this.emitEvent(sessionId, 'tvdb_sync_completed', { synced, errors, type });

      return { synced, errors };

    } catch (error) {
      logger.error({ error, sessionId }, 'Sync workflow failed');
      this.emitEvent(sessionId, 'error_occurred', { error: String(error) });
      throw error;
    }
  }

  /**
   * Execute pattern consolidation (nightly learning)
   */
  async executeConsolidationWorkflow(sessionId: string): Promise<void> {
    const context = this.activeContexts.get(sessionId);
    if (!context) {
      throw new Error(`Session ${sessionId} not found`);
    }

    try {
      const consolidateTask = this.createTask('consolidate_patterns', {});
      context.tasks.push(consolidateTask);

      await this.executeTask(consolidateTask, async () => {
        // Get pattern stats
        const stats = this.engine.getPatternStats();
        logger.info({ stats }, 'Pattern consolidation stats');

        // Could add more sophisticated consolidation logic here:
        // - Merge similar patterns
        // - Prune low-performing patterns
        // - Generate new patterns from successful episodes
      });

      this.emitEvent(sessionId, 'pattern_consolidated', {});

    } catch (error) {
      logger.error({ error, sessionId }, 'Consolidation workflow failed');
      throw error;
    }
  }

  /**
   * Create a new task
   */
  private createTask(type: WorkflowTaskType, input: Record<string, unknown>): WorkflowTask {
    return {
      id: uuidv4(),
      type,
      status: 'pending',
      input
    };
  }

  /**
   * Execute a task with timing and status tracking
   */
  private async executeTask<T>(
    task: WorkflowTask,
    executor: () => Promise<T>
  ): Promise<T> {
    task.status = 'in_progress';
    task.startedAt = new Date();

    try {
      const result = await executor();
      task.status = 'completed';
      task.completedAt = new Date();
      task.output = result as any;
      return result;
    } catch (error) {
      task.status = 'failed';
      task.completedAt = new Date();
      task.error = String(error);
      throw error;
    }
  }

  /**
   * Emit a system event
   */
  private emitEvent(
    sessionId: string,
    type: string,
    payload: Record<string, unknown>
  ): void {
    const context = this.activeContexts.get(sessionId);
    if (!context) return;

    const event: SystemEvent = {
      type: type as any,
      payload,
      timestamp: new Date(),
      source: 'workflow'
    };

    context.events.push(event);
    logger.debug({ sessionId, event: type }, 'Event emitted');
  }

  /**
   * Get workflow context
   */
  getContext(sessionId: string): WorkflowContext | null {
    return this.activeContexts.get(sessionId) || null;
  }

  /**
   * Get all active sessions
   */
  getActiveSessions(): string[] {
    return Array.from(this.activeContexts.keys());
  }
}

// Factory function
export function createRecommendationWorkflow(
  tvdbClient: TVDBClient,
  embedder: EmbeddingService,
  engine: RecommendationEngine,
  repository: Repository
): RecommendationWorkflow {
  return new RecommendationWorkflow(tvdbClient, embedder, engine, repository);
}

export default RecommendationWorkflow;
