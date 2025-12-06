import { Router, Request, Response, NextFunction } from 'express';
import { ValidationError, ApiResponse, InternalError } from '../middleware/error-handler';
import { FeedbackRequest, FeedbackResponse, EmotionalState } from '../../types';

const router = Router();

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

      // TODO: Integrate with FeedbackProcessor and RLPolicyEngine
      // For now, return mock response
      const mockResponse: FeedbackResponse = {
        reward: 0.75,
        policyUpdated: true,
        newQValue: 0.82,
        learningProgress: {
          totalExperiences: 15,
          avgReward: 0.68,
          explorationRate: 0.12,
          convergenceScore: 0.45,
        },
      };

      res.json({
        success: true,
        data: mockResponse,
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
