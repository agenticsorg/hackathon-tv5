import { Router, Request, Response, NextFunction } from 'express';
import { recommendRateLimiter } from '../middleware/rate-limiter.js';
import { ValidationError, ApiResponse } from '../middleware/error-handler.js';
import { EmotionalState, DesiredState, Recommendation } from '../../types/index.js';
import { getServices } from '../../services/index.js';

const router = Router();

/**
 * POST /api/v1/recommend
 * Get content recommendations based on emotional state
 *
 * Request body:
 * {
 *   userId: string;
 *   currentState: EmotionalState;
 *   desiredState: DesiredState;
 *   limit?: number;
 * }
 *
 * Response:
 * {
 *   success: true,
 *   data: {
 *     recommendations: Recommendation[];
 *     explorationRate: number;
 *   }
 * }
 */
router.post(
  '/',
  recommendRateLimiter,
  async (req: Request, res: Response<ApiResponse<any>>, next: NextFunction) => {
    try {
      const { userId, currentState, desiredState, limit = 5 } = req.body;

      // Validate request
      if (!userId || typeof userId !== 'string') {
        throw new ValidationError('userId is required and must be a string');
      }

      if (!currentState || typeof currentState !== 'object') {
        throw new ValidationError('currentState is required and must be an EmotionalState object');
      }

      if (!desiredState || typeof desiredState !== 'object') {
        throw new ValidationError('desiredState is required and must be a DesiredState object');
      }

      // Validate limit
      const numLimit = parseInt(limit as string);
      if (isNaN(numLimit) || numLimit < 1 || numLimit > 20) {
        throw new ValidationError('limit must be between 1 and 20');
      }

      // Use real RecommendationEngine
      const services = getServices();
      const recommendations = await services.recommendationEngine.recommend(
        userId,
        {
          valence: currentState.valence,
          arousal: currentState.arousal,
          stress: currentState.stressLevel ?? currentState.stress ?? 0.5,
        },
        numLimit
      );

      // Map to API response format
      const apiRecommendations = recommendations.map((rec) => ({
        contentId: rec.contentId,
        title: rec.title,
        qValue: rec.qValue,
        similarityScore: rec.similarityScore,
        combinedScore: rec.combinedScore,
        predictedOutcome: {
          expectedValence: rec.predictedOutcome.expectedValence,
          expectedArousal: rec.predictedOutcome.expectedArousal,
          expectedStress: rec.predictedOutcome.expectedStress,
          confidence: rec.predictedOutcome.confidence,
        },
        reasoning: rec.reasoning,
        isExploration: rec.isExploration,
      }));

      const explorationRate = services.getExplorationRate();

      res.json({
        success: true,
        data: {
          userId,
          recommendations: apiRecommendations,
          explorationRate,
          timestamp: Date.now(),
        },
        error: null,
        timestamp: new Date().toISOString(),
      });
    } catch (error) {
      next(error);
    }
  }
);

/**
 * GET /api/v1/recommend/history/:userId
 * Get recommendation history for a user
 */
router.get(
  '/history/:userId',
  async (req: Request, res: Response<ApiResponse<any>>, next: NextFunction) => {
    try {
      const { userId } = req.params;
      const limit = parseInt(req.query.limit as string) || 10;

      if (!userId) {
        throw new ValidationError('userId is required');
      }

      // TODO: Implement history retrieval
      res.json({
        success: true,
        data: {
          userId,
          history: [],
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
