/**
 * MCP Tools for Self-Learning System
 *
 * Exposes on-device learning capabilities to AI agents
 */

import { PreferenceLearningSystem } from '../learning/preference-learning.js';
import { LearningPersistence } from '../learning/persistence.js';
import { SmartTVClient, createSmartTVClientFromIP } from '../learning/smart-tv-client.js';
import {
  ContentMetadata,
  ContentMetadataSchema,
  Recommendation,
  LearningStats,
  LearningFeedback,
  Genre,
  ContentType,
} from '../learning/types.js';
import { MCPToolResult, SamsungTVDevice } from '../lib/types.js';
import { getDefaultDevice, getDevices } from '../utils/config.js';

// Singleton learning system (shared across tools)
let learner: PreferenceLearningSystem | null = null;
let persistence: LearningPersistence | null = null;
let smartClient: SmartTVClient | null = null;

function getLearner(): PreferenceLearningSystem {
  if (!learner) {
    learner = new PreferenceLearningSystem();
    persistence = new LearningPersistence();
    // Load existing model
    persistence.loadModel(learner);
  }
  return learner;
}

function getPersistence(): LearningPersistence {
  if (!persistence) {
    persistence = new LearningPersistence();
  }
  return persistence;
}

function getSmartClient(): SmartTVClient | null {
  if (smartClient) return smartClient;

  const device = getDefaultDevice();
  if (!device) return null;

  smartClient = new SmartTVClient(device);
  return smartClient;
}

// Learning MCP Tool definitions
export const LEARNING_TOOLS = [
  {
    name: 'samsung_tv_learn_get_recommendations',
    description: 'Get personalized content recommendations based on learned preferences. Uses on-device Q-Learning to select the best recommendation strategy.',
    inputSchema: {
      type: 'object',
      properties: {
        count: {
          type: 'number',
          description: 'Number of recommendations to return (default: 5)',
        },
      },
    },
  },
  {
    name: 'samsung_tv_learn_add_content',
    description: 'Add content to the learning system content library. Content embeddings are generated automatically.',
    inputSchema: {
      type: 'object',
      properties: {
        content: {
          type: 'object',
          description: 'Content metadata object',
          properties: {
            id: { type: 'string', description: 'Unique content ID' },
            title: { type: 'string', description: 'Content title' },
            type: { type: 'string', enum: ['movie', 'tv_show', 'documentary', 'sports', 'news', 'music', 'kids', 'gaming'] },
            genres: { type: 'array', items: { type: 'string' }, description: 'List of genres' },
            duration: { type: 'number', description: 'Duration in minutes' },
            releaseYear: { type: 'number' },
            rating: { type: 'number', description: 'Rating 0-10' },
            popularity: { type: 'number', description: 'Popularity 0-100' },
            description: { type: 'string' },
            appId: { type: 'string', description: 'Streaming app ID' },
            appName: { type: 'string', description: 'Streaming app name' },
          },
          required: ['id', 'title', 'type'],
        },
      },
      required: ['content'],
    },
  },
  {
    name: 'samsung_tv_learn_record_session',
    description: 'Record a viewing session for learning. The system will calculate rewards and update the Q-Learning policy.',
    inputSchema: {
      type: 'object',
      properties: {
        contentId: { type: 'string', description: 'ID of the content that was watched' },
        watchDuration: { type: 'number', description: 'How long the user watched (minutes)' },
        completionRate: { type: 'number', description: 'Completion rate 0-1' },
        userRating: { type: 'number', description: 'User rating 1-5 (optional)' },
        action: {
          type: 'string',
          enum: ['recommend_similar', 'recommend_popular', 'recommend_trending', 'recommend_genre', 'recommend_new_release', 'recommend_continue_watching', 'recommend_based_on_time', 'explore_new_genre', 'explore_new_type'],
          description: 'The recommendation action that led to this content',
        },
      },
      required: ['contentId', 'watchDuration', 'action'],
    },
  },
  {
    name: 'samsung_tv_learn_feedback',
    description: 'Process explicit feedback on a recommendation (selected, skipped, rated)',
    inputSchema: {
      type: 'object',
      properties: {
        recommendationId: { type: 'string', description: 'ID of the recommendation' },
        contentId: { type: 'string', description: 'Content ID' },
        action: { type: 'string', description: 'The action that generated this recommendation' },
        selected: { type: 'boolean', description: 'Whether the user selected this recommendation' },
        watchDuration: { type: 'number', description: 'Watch duration if selected (minutes)' },
        completionRate: { type: 'number', description: 'Completion rate if watched' },
        userRating: { type: 'number', description: 'User rating 1-5' },
      },
      required: ['recommendationId', 'contentId', 'action', 'selected'],
    },
  },
  {
    name: 'samsung_tv_learn_get_stats',
    description: 'Get learning system statistics including patterns learned, average rewards, and top actions',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
  {
    name: 'samsung_tv_learn_get_preferences',
    description: 'Get learned user preferences including favorite genres, types, and time-based patterns',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
  {
    name: 'samsung_tv_learn_train',
    description: 'Trigger experience replay to train the model on past experiences',
    inputSchema: {
      type: 'object',
      properties: {
        batchSize: { type: 'number', description: 'Number of experiences to replay (default: 32)' },
      },
    },
  },
  {
    name: 'samsung_tv_learn_save',
    description: 'Save the learned model to disk for persistence',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
  {
    name: 'samsung_tv_learn_load',
    description: 'Load a previously saved model from disk',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
  {
    name: 'samsung_tv_learn_clear',
    description: 'Clear all learned data and start fresh',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
  {
    name: 'samsung_tv_learn_storage_stats',
    description: 'Get storage statistics for the learning system',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
  {
    name: 'samsung_tv_smart_launch',
    description: 'Launch content with automatic session tracking and learning',
    inputSchema: {
      type: 'object',
      properties: {
        contentId: { type: 'string', description: 'Content ID to launch' },
        title: { type: 'string', description: 'Content title' },
        type: { type: 'string', description: 'Content type' },
        genres: { type: 'array', items: { type: 'string' } },
        appId: { type: 'string', description: 'App to use for launching' },
      },
      required: ['contentId', 'title', 'type'],
    },
  },
  {
    name: 'samsung_tv_smart_end_session',
    description: 'End current viewing session and record learning data',
    inputSchema: {
      type: 'object',
      properties: {
        userRating: { type: 'number', description: 'User rating 1-5 (optional)' },
      },
    },
  },
];

/**
 * Handle learning-related MCP tool calls
 */
export async function handleLearningToolCall(
  toolName: string,
  args: Record<string, unknown>
): Promise<MCPToolResult> {
  try {
    switch (toolName) {
      case 'samsung_tv_learn_get_recommendations': {
        const learner = getLearner();
        const count = typeof args.count === 'number' ? args.count : 5;
        const recommendations = learner.getRecommendations(count);

        return {
          success: true,
          data: {
            count: recommendations.length,
            recommendations: recommendations.map(r => ({
              contentId: r.contentId,
              title: r.title,
              type: r.type,
              genres: r.genres,
              score: Math.round(r.score * 100) / 100,
              reason: r.reason,
              action: r.action,
              confidence: Math.round(r.confidence * 100) / 100,
              appId: r.appId,
            })),
          },
        };
      }

      case 'samsung_tv_learn_add_content': {
        const learner = getLearner();
        const contentArg = args.content as Record<string, unknown>;

        const content: ContentMetadata = {
          id: String(contentArg.id),
          title: String(contentArg.title),
          type: contentArg.type as ContentType,
          genres: (contentArg.genres as string[] || []) as Genre[],
          duration: typeof contentArg.duration === 'number' ? contentArg.duration : undefined,
          releaseYear: typeof contentArg.releaseYear === 'number' ? contentArg.releaseYear : undefined,
          rating: typeof contentArg.rating === 'number' ? contentArg.rating : undefined,
          popularity: typeof contentArg.popularity === 'number' ? contentArg.popularity : undefined,
          description: typeof contentArg.description === 'string' ? contentArg.description : undefined,
          actors: [],
          directors: [],
          keywords: [],
          appId: typeof contentArg.appId === 'string' ? contentArg.appId : undefined,
          appName: typeof contentArg.appName === 'string' ? contentArg.appName : undefined,
        };

        learner.addContent(content);
        return { success: true, data: { contentId: content.id, added: true } };
      }

      case 'samsung_tv_learn_record_session': {
        const learner = getLearner();
        const contentId = String(args.contentId);
        const watchDuration = Number(args.watchDuration);
        const action = args.action as string;
        const completionRate = typeof args.completionRate === 'number' ? args.completionRate : watchDuration / 90;

        // Create a session object
        const now = new Date();
        const session = {
          id: `session-${Date.now()}`,
          contentId,
          contentMetadata: {
            id: contentId,
            title: contentId, // Placeholder if content not in library
            type: 'movie' as ContentType,
            genres: [] as Genre[],
          },
          startTime: new Date(now.getTime() - watchDuration * 60 * 1000).toISOString(),
          endTime: now.toISOString(),
          watchDuration,
          completionRate,
          userRating: typeof args.userRating === 'number' ? args.userRating : undefined,
          implicit: { paused: 0, rewound: 0, fastForwarded: 0, volumeChanges: 0 },
        };

        learner.recordSession(session as any, action as any);

        return {
          success: true,
          data: {
            sessionId: session.id,
            reward: learner.getStats().avgReward,
          },
        };
      }

      case 'samsung_tv_learn_feedback': {
        const learner = getLearner();
        const feedback: LearningFeedback = {
          recommendationId: String(args.recommendationId),
          contentId: String(args.contentId),
          action: args.action as any,
          selected: Boolean(args.selected),
          watchDuration: typeof args.watchDuration === 'number' ? args.watchDuration : undefined,
          completionRate: typeof args.completionRate === 'number' ? args.completionRate : undefined,
          userRating: typeof args.userRating === 'number' ? args.userRating : undefined,
          timestamp: new Date().toISOString(),
        };

        learner.processFeedback(feedback);
        return { success: true };
      }

      case 'samsung_tv_learn_get_stats': {
        const learner = getLearner();
        const stats = learner.getStats();

        return {
          success: true,
          data: {
            totalSessions: stats.totalSessions,
            totalPatterns: stats.totalPatterns,
            avgReward: Math.round(stats.avgReward * 100) / 100,
            explorationRate: Math.round(stats.explorationRate * 100) / 100,
            learningProgress: Math.round(stats.learningProgress * 100),
            topActions: stats.topActions.slice(0, 5).map(a => ({
              action: a.action,
              count: a.count,
              avgReward: Math.round(a.avgReward * 100) / 100,
            })),
            lastTrainingTime: stats.lastTrainingTime,
          },
        };
      }

      case 'samsung_tv_learn_get_preferences': {
        const learner = getLearner();
        const prefs = learner.getPreferences();

        return {
          success: true,
          data: {
            userId: prefs.userId,
            favoriteGenres: prefs.favoriteGenres,
            favoriteTypes: prefs.favoriteTypes,
            preferredDuration: prefs.preferredDuration,
            preferredTimeSlots: prefs.preferredTimeSlots,
            dislikedGenres: prefs.dislikedGenres,
            watchedCount: prefs.watchedContentIds.length,
            lastUpdated: prefs.lastUpdated,
          },
        };
      }

      case 'samsung_tv_learn_train': {
        const learner = getLearner();
        const batchSize = typeof args.batchSize === 'number' ? args.batchSize : 32;
        learner.experienceReplay(batchSize);

        return {
          success: true,
          data: {
            trained: true,
            batchSize,
            newStats: learner.getStats(),
          },
        };
      }

      case 'samsung_tv_learn_save': {
        const learner = getLearner();
        const persistence = getPersistence();
        persistence.saveModel(learner);

        return {
          success: true,
          data: {
            saved: true,
            path: persistence.getDataDir(),
          },
        };
      }

      case 'samsung_tv_learn_load': {
        const learner = getLearner();
        const persistence = getPersistence();
        const loaded = persistence.loadModel(learner);

        return {
          success: loaded,
          data: loaded ? { loaded: true } : undefined,
          error: loaded ? undefined : 'No saved model found',
        };
      }

      case 'samsung_tv_learn_clear': {
        const persistence = getPersistence();
        persistence.clearAll();
        learner = new PreferenceLearningSystem(); // Reset

        return { success: true, data: { cleared: true } };
      }

      case 'samsung_tv_learn_storage_stats': {
        const persistence = getPersistence();
        const stats = persistence.getStorageStats();

        return {
          success: true,
          data: {
            modelExists: stats.modelExists,
            modelAgeHours: stats.modelAge ? Math.round(stats.modelAge * 10) / 10 : null,
            contentCount: stats.contentCount,
            sessionCount: stats.sessionCount,
            totalSizeKB: Math.round(stats.totalSize / 1024),
          },
        };
      }

      case 'samsung_tv_smart_launch': {
        const client = getSmartClient();
        if (!client) {
          return { success: false, error: 'No TV configured. Run samsung_tv_discover and samsung_tv_connect first.' };
        }

        const content: ContentMetadata = {
          id: String(args.contentId),
          title: String(args.title),
          type: args.type as ContentType,
          genres: (args.genres as string[] || []) as Genre[],
          appId: typeof args.appId === 'string' ? args.appId : undefined,
          actors: [],
          directors: [],
          keywords: [],
        };

        client.addContent(content);
        const result = await client.launchContent(content);

        return {
          success: result.success,
          data: result.success ? { launched: true, contentId: content.id } : undefined,
          error: result.error,
        };
      }

      case 'samsung_tv_smart_end_session': {
        const client = getSmartClient();
        if (!client) {
          return { success: false, error: 'No TV configured' };
        }

        const userRating = typeof args.userRating === 'number' ? args.userRating : undefined;
        const session = client.endCurrentSession(userRating);

        if (!session) {
          return { success: false, error: 'No active session to end' };
        }

        return {
          success: true,
          data: {
            sessionId: session.id,
            contentId: session.contentId,
            watchDuration: Math.round(session.watchDuration),
            completionRate: Math.round(session.completionRate * 100),
          },
        };
      }

      default:
        return { success: false, error: `Unknown learning tool: ${toolName}` };
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error';
    return { success: false, error: message };
  }
}
