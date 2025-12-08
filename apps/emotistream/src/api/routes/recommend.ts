import { Router, Request, Response, NextFunction } from 'express';
import { recommendRateLimiter } from '../middleware/rate-limiter.js';
import { ValidationError, ApiResponse } from '../middleware/error-handler.js';
import { EmotionalState, DesiredState, Recommendation } from '../../types/index.js';
import { getServices } from '../../services/index.js';
import { sessionStore } from './session-store.js';
import { isUsingPostgres, getRecommendationHistoryStore } from '../../persistence/index.js';
import { createLogger } from '../../utils/logger.js';

const logger = createLogger('RecommendRoute');

const router = Router();

// Type definitions for API responses
interface RecommendationResponse {
  userId: string;
  recommendations: Array<{
    contentId: string;
    title: string;
    qValue: number;
    similarityScore: number;
    combinedScore: number;
    predictedOutcome: {
      expectedValence: number;
      expectedArousal: number;
      expectedStress: number;
      confidence: number;
    };
    reasoning: string;
    isExploration: boolean;
  }>;
  explorationRate: number;
  timestamp: number;
}

interface RecommendationHistoryItem {
  id: string;
  contentId: string;
  contentTitle: string;
  qValue: number;
  similarityScore: number;
  combinedScore: number;
  isExploration: boolean;
  reasoning: string;
  emotionalState: {
    valence: number;
    arousal: number;
    stress: number;
  };
  timestamp: string;
}

interface RecommendationHistoryResponse {
  userId: string;
  history: RecommendationHistoryItem[];
  count: number;
  usingPostgres: boolean;
}

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
  async (req: Request, res: Response<ApiResponse<RecommendationResponse>>, next: NextFunction) => {
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

      // Store session data for each recommendation for later feedback processing
      const currentStateNormalized = {
        valence: currentState.valence,
        arousal: currentState.arousal,
        stress: currentState.stressLevel ?? currentState.stress ?? 0.5,
        confidence: 0.7, // Default confidence for user-provided state
      };

      const desiredStateNormalized = {
        targetValence: desiredState.targetValence ?? 0.5,
        targetArousal: desiredState.targetArousal ?? -0.2,
        targetStress: desiredState.targetStress ?? 0.2,
        intensity: (desiredState.intensity ?? 'moderate') as 'subtle' | 'moderate' | 'significant',
        reasoning: desiredState.reasoning ?? 'User-specified desired emotional state',
      };

      const timestamp = Date.now();

      // Store session for each recommended content
      recommendations.forEach((rec) => {
        const sessionKey = `${userId}:${rec.contentId}`;
        sessionStore.set(sessionKey, {
          stateBefore: currentStateNormalized,
          desiredState: desiredStateNormalized,
          contentId: rec.contentId,
          timestamp,
        });
      });

      // Persist recommendations to history if using PostgreSQL
      if (isUsingPostgres()) {
        const historyStore = getRecommendationHistoryStore();
        for (const rec of recommendations) {
          try {
            await historyStore.saveRecommendation(
              userId,
              {
                contentId: rec.contentId,
                title: rec.title,
                qValue: rec.qValue,
                similarityScore: rec.similarityScore,
                combinedScore: rec.combinedScore,
                isExploration: rec.isExploration,
                reasoning: rec.reasoning,
              },
              currentStateNormalized
            );
          } catch (error) {
            logger.warn('Failed to save recommendation to history', { error, contentId: rec.contentId });
          }
        }
      }

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
  async (req: Request, res: Response<ApiResponse<RecommendationHistoryResponse>>, next: NextFunction) => {
    try {
      const { userId } = req.params;
      const limit = Math.min(Math.max(parseInt(req.query.limit as string) || 50, 1), 100);

      if (!userId) {
        throw new ValidationError('userId is required');
      }

      let history: RecommendationHistoryItem[] = [];
      let count = 0;
      const usingPostgres = isUsingPostgres();

      if (usingPostgres) {
        const historyStore = getRecommendationHistoryStore();
        const dbHistory = await historyStore.getUserHistory(userId, limit);
        count = await historyStore.getUserRecommendationCount(userId);

        history = dbHistory.map((item) => ({
          id: item.id,
          contentId: item.contentId,
          contentTitle: item.contentTitle,
          qValue: item.qValue,
          similarityScore: item.similarityScore,
          combinedScore: item.combinedScore,
          isExploration: item.isExploration,
          reasoning: item.reasoning,
          emotionalState: {
            valence: item.stateValence,
            arousal: item.stateArousal,
            stress: item.stateStress,
          },
          timestamp: item.createdAt.toISOString(),
        }));
      } else {
        logger.debug('Recommendation history requires PostgreSQL persistence');
      }

      res.json({
        success: true,
        data: {
          userId,
          history,
          count,
          usingPostgres,
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
