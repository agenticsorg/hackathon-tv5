import { Router, Request, Response, NextFunction } from 'express';
import { ValidationError, ApiResponse, InternalError } from '../middleware/error-handler.js';
import { FeedbackRequest, FeedbackResponse } from '../../feedback/types.js';
import { EmotionalState } from '../../emotion/types.js';
import { getServices } from '../../services/index.js';
import { EmotionalExperience } from '../../rl/types.js';
import { getFeedbackStore } from '../../persistence/index.js';
import { FeedbackSubmission } from '../../types/feedback.js';

const router = Router();

// Get shared feedback store instance
const feedbackStore = getFeedbackStore();

// Store for tracking user sessions (stateBeforeViewing for reward calculation)
const userSessionStore = new Map<string, { stateBefore: any; desiredState: any; contentId: string }>();

/**
 * POST /api/v1/feedback
 * Submit post-viewing feedback
 *
 * Request body:
 * {
 *   userId: string;
 *   contentId: string;
 *   actualPostState: EmotionalState;
 *   watchDuration: number;
 *   completed: boolean;
 *   explicitRating?: number;
 * }
 *
 * Response:
 * {
 *   success: true,
 *   data: {
 *     reward: number;
 *     policyUpdated: boolean;
 *     newQValue: number;
 *     learningProgress: LearningProgress;
 *   }
 * }
 */
router.post(
  '/',
  async (req: Request, res: Response<ApiResponse<FeedbackResponse>>, next: NextFunction) => {
    try {
      const feedbackRequest: FeedbackRequest = req.body;

      // Validate request
      if (!feedbackRequest.userId || typeof feedbackRequest.userId !== 'string') {
        throw new ValidationError('userId is required and must be a string');
      }

      if (!feedbackRequest.contentId || typeof feedbackRequest.contentId !== 'string') {
        throw new ValidationError('contentId is required and must be a string');
      }

      if (!feedbackRequest.actualPostState || typeof feedbackRequest.actualPostState !== 'object') {
        throw new ValidationError('actualPostState is required and must be an EmotionalState object');
      }

      if (typeof feedbackRequest.watchDuration !== 'number' || feedbackRequest.watchDuration < 0) {
        throw new ValidationError('watchDuration must be a positive number');
      }

      if (typeof feedbackRequest.completed !== 'boolean') {
        throw new ValidationError('completed must be a boolean');
      }

      // Validate optional explicitRating
      if (feedbackRequest.explicitRating !== undefined) {
        const rating = feedbackRequest.explicitRating;
        if (typeof rating !== 'number' || rating < 1 || rating > 5) {
          throw new ValidationError('explicitRating must be between 1 and 5');
        }
      }

      // Use real FeedbackProcessor and RLPolicyEngine
      const services = getServices();

      // Build state before viewing (use stored session or construct from actualPostState)
      const sessionKey = `${feedbackRequest.userId}:${feedbackRequest.contentId}`;
      const session = userSessionStore.get(sessionKey);

      // Default state before (will be estimated if no session exists)
      const stateBefore = session?.stateBefore ?? {
        valence: feedbackRequest.actualPostState.valence * 0.5,
        arousal: feedbackRequest.actualPostState.arousal * 0.8,
        stress: feedbackRequest.actualPostState.stressLevel ?? 0.5,
        confidence: 0.6,
      };

      const desiredState = session?.desiredState ?? {
        targetValence: 0.5,
        targetArousal: -0.2,
        targetStress: 0.2,
        intensity: 'moderate' as const,
        reasoning: 'Default desired state for emotional homeostasis',
      };

      // Process feedback using FeedbackProcessor
      const feedbackResult = services.feedbackProcessor.process(
        feedbackRequest,
        {
          valence: stateBefore.valence,
          arousal: stateBefore.arousal,
          stressLevel: stateBefore.stress,
          primaryEmotion: 'joy' as const, // Default to 'joy' - closest to neutral positive
          emotionVector: new Float32Array(8),
          confidence: stateBefore.confidence,
          timestamp: Date.now() - feedbackRequest.watchDuration * 60000,
        },
        desiredState
      );

      // Update RL policy using RLPolicyEngine
      // Build experience object matching rl/types.ts EmotionalExperience
      const experience: EmotionalExperience = {
        stateBefore: {
          valence: stateBefore.valence,
          arousal: stateBefore.arousal,
          stress: stateBefore.stress,
          confidence: stateBefore.confidence,
        },
        stateAfter: {
          valence: feedbackRequest.actualPostState.valence,
          arousal: feedbackRequest.actualPostState.arousal,
          stress: feedbackRequest.actualPostState.stressLevel ?? 0.3,
          confidence: 0.8,
        },
        contentId: feedbackRequest.contentId,
        reward: feedbackResult.reward,
        desiredState: {
          valence: desiredState.targetValence,
          arousal: desiredState.targetArousal,
          confidence: 0.7,
        },
      };

      const policyUpdate = await services.policyEngine.updatePolicy(
        feedbackRequest.userId,
        experience
      );

      // Persist feedback to store for progress tracking
      const submission: FeedbackSubmission = {
        userId: feedbackRequest.userId,
        contentId: feedbackRequest.contentId,
        contentTitle: feedbackRequest.contentId, // Use contentId as title fallback
        sessionId: sessionKey,
        emotionBefore: {
          valence: stateBefore.valence,
          arousal: stateBefore.arousal,
          stressLevel: stateBefore.stress,
          primaryEmotion: 'neutral',
          emotionVector: new Float32Array(8),
          confidence: stateBefore.confidence,
          timestamp: Date.now() - feedbackRequest.watchDuration * 60000,
        },
        emotionAfter: {
          valence: feedbackRequest.actualPostState.valence,
          arousal: feedbackRequest.actualPostState.arousal,
          stressLevel: feedbackRequest.actualPostState.stressLevel ?? 0.3,
          primaryEmotion: 'neutral',
          emotionVector: new Float32Array(8),
          confidence: 0.8,
          timestamp: Date.now(),
        },
        starRating: feedbackRequest.explicitRating ?? 3,
        completed: feedbackRequest.completed,
        watchDuration: feedbackRequest.watchDuration * 60000, // Convert to ms
        totalDuration: feedbackRequest.watchDuration * 60000 * 1.2, // Estimate total
        timestamp: new Date(),
      };

      await feedbackStore.saveFeedback(
        submission,
        feedbackResult.reward,
        0.5, // qValueBefore placeholder
        policyUpdate.newQValue
      );

      // Clean up session
      userSessionStore.delete(sessionKey);

      const response: FeedbackResponse = {
        reward: feedbackResult.reward,
        policyUpdated: true,
        newQValue: policyUpdate.newQValue,
        learningProgress: {
          totalExperiences: feedbackResult.learningProgress.totalExperiences,
          avgReward: feedbackResult.learningProgress.avgReward,
          explorationRate: services.getExplorationRate(),
          convergenceScore: feedbackResult.learningProgress.convergenceScore,
        },
      };

      res.json({
        success: true,
        data: response,
        error: null,
        timestamp: new Date().toISOString(),
      });
    } catch (error) {
      next(error);
    }
  }
);

/**
 * GET /api/v1/feedback/progress/:userId
 * Get learning progress for a user
 */
router.get(
  '/progress/:userId',
  async (req: Request, res: Response<ApiResponse<any>>, next: NextFunction) => {
    try {
      const { userId } = req.params;

      if (!userId) {
        throw new ValidationError('userId is required');
      }

      // TODO: Implement progress retrieval from RLPolicyEngine
      const mockProgress = {
        userId,
        totalExperiences: 15,
        avgReward: 0.68,
        explorationRate: 0.12,
        convergenceScore: 0.45,
        recentRewards: [0.75, 0.82, 0.65, 0.71, 0.88],
      };

      res.json({
        success: true,
        data: mockProgress,
        error: null,
        timestamp: new Date().toISOString(),
      });
    } catch (error) {
      next(error);
    }
  }
);

/**
 * GET /api/v1/feedback/experiences/:userId
 * Get feedback experiences for a user
 */
router.get(
  '/experiences/:userId',
  async (req: Request, res: Response<ApiResponse<any>>, next: NextFunction) => {
    try {
      const { userId } = req.params;
      const limit = parseInt(req.query.limit as string) || 10;

      if (!userId) {
        throw new ValidationError('userId is required');
      }

      if (limit < 1 || limit > 100) {
        throw new ValidationError('limit must be between 1 and 100');
      }

      // TODO: Implement experience retrieval
      res.json({
        success: true,
        data: {
          userId,
          experiences: [],
          count: 0,
        },
        error: null,
        timestamp: new Date().toISOString(),
      });
    } catch (error) {
      next(error);
    }
  }
);

export default router;
