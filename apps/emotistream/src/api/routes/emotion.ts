import { Router, Request, Response, NextFunction } from 'express';
import { emotionRateLimiter } from '../middleware/rate-limiter.js';
import { ValidationError, ApiResponse } from '../middleware/error-handler.js';
import { EmotionalState } from '../../types/index.js';
import { getServices } from '../../services/index.js';
import { GeminiClient } from '../../emotion/gemini-client.js';

const router = Router();

// Lazy initialization - create client on first request after env is loaded
let geminiClient: GeminiClient | null = null;
function getGeminiClient(): GeminiClient {
  if (!geminiClient) {
    geminiClient = new GeminiClient();
  }
  return geminiClient;
}

/**
 * POST /api/v1/emotion/analyze
 * Analyze text input for emotional state
 *
 * Request body:
 * {
 *   userId: string;
 *   text: string;
 * }
 *
 * Response:
 * {
 *   success: true,
 *   data: {
 *     state: EmotionalState;
 *     desired: DesiredState;
 *   }
 * }
 */
router.post(
  '/analyze',
  emotionRateLimiter,
  async (req: Request, res: Response<ApiResponse<any>>, next: NextFunction) => {
    try {
      const { userId, text } = req.body;

      // Validate request
      if (!userId || typeof userId !== 'string') {
        throw new ValidationError('userId is required and must be a string');
      }

      if (!text || typeof text !== 'string') {
        throw new ValidationError('text is required and must be a string');
      }

      if (text.trim().length < 10) {
        throw new ValidationError('text must be at least 10 characters');
      }

      if (text.length > 1000) {
        throw new ValidationError('text must be less than 1000 characters');
      }

      // Use EmotionDetector with Gemini fallback
      const services = getServices();
      let emotionResult;
      let usedGemini = false;

      // Try Gemini first if available
      const gemini = getGeminiClient();
      if (gemini.isAvailable()) {
        try {
          const geminiResult = await gemini.analyzeEmotion(text);
          emotionResult = {
            valence: geminiResult.valence,
            arousal: geminiResult.arousal,
            stressLevel: geminiResult.stress,
            primaryEmotion: geminiResult.dominantEmotion,
            emotionVector: new Float32Array([
              geminiResult.plutchikEmotions.joy,
              geminiResult.plutchikEmotions.trust,
              geminiResult.plutchikEmotions.fear,
              geminiResult.plutchikEmotions.surprise,
              geminiResult.plutchikEmotions.sadness,
              geminiResult.plutchikEmotions.disgust,
              geminiResult.plutchikEmotions.anger,
              geminiResult.plutchikEmotions.anticipation,
            ]),
            confidence: geminiResult.confidence,
            timestamp: Date.now(),
          };
          usedGemini = true;
        } catch (error) {
          console.warn('Gemini API failed, using local detector:', error);
        }
      }

      // Fall back to local detector
      if (!emotionResult) {
        const localResult = await services.emotionDetector.analyzeText(text);
        emotionResult = {
          valence: localResult.currentState.valence,
          arousal: localResult.currentState.arousal,
          stressLevel: localResult.currentState.stressLevel,
          primaryEmotion: localResult.currentState.primaryEmotion,
          emotionVector: localResult.currentState.emotionVector,
          confidence: localResult.currentState.confidence,
          timestamp: localResult.currentState.timestamp,
        };
      }

      const state: EmotionalState = {
        valence: emotionResult.valence,
        arousal: emotionResult.arousal,
        stressLevel: emotionResult.stressLevel,
        primaryEmotion: emotionResult.primaryEmotion,
        emotionVector: emotionResult.emotionVector,
        confidence: emotionResult.confidence,
        timestamp: emotionResult.timestamp,
      };

      const desired = {
        targetValence: state.valence < 0 ? 0.5 : state.valence,
        targetArousal: state.stressLevel > 0.5 ? -0.2 : state.arousal,
        targetStress: Math.max(0.1, state.stressLevel - 0.4),
        intensity: state.stressLevel > 0.7 ? 'high' as const : 'moderate' as const,
        reasoning: `Analyzed with ${usedGemini ? 'Gemini AI' : 'local detector'}. ${state.stressLevel > 0.5 ? 'High stress detected, suggesting calming content.' : 'Recommending content aligned with current mood.'}`,
      };

      res.json({
        success: true,
        data: {
          userId,
          inputText: text,
          state,
          desired,
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
 * GET /api/v1/emotion/history/:userId
 * Get emotional state history for a user
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
