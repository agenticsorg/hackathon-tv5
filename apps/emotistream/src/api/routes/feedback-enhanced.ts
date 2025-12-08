/**
 * Enhanced Feedback API Routes
 *
 * Extended feedback submission with full tracking and analytics integration.
 */

import { Router, Request, Response } from 'express';
import { z } from 'zod';
import { FeedbackSubmission, EmotionComparison } from '../../types/feedback.js';
import { FeedbackStore } from '../../persistence/feedback-store.js';
import { RewardCalculator } from '../../services/reward-calculator.js';
import { WatchTracker } from '../../services/watch-tracker.js';
import { apiResponse } from '../middleware/response.js';
import { ValidationError, NotFoundError } from '../../utils/errors.js';

const router = Router();

// Initialize services
const feedbackStore = new FeedbackStore();
const rewardCalculator = new RewardCalculator();

// Helper function
function getRewardMessage(reward: number): string {
  if (reward > 0.7) {
    return '🎉 Excellent choice! You felt significantly better!';
  } else if (reward > 0.4) {
    return '👍 Great! This moved you closer to your goal!';
  } else if (reward > 0) {
    return '✓ Good choice. You made some progress!';
  } else if (reward > -0.3) {
    return '🤔 This was okay, but might not have been ideal.';
  } else {
    return '💭 Let\'s try something different next time.';
  }
}
const watchTracker = new WatchTracker();

feedbackStore.initialize().catch(console.error);

// Validation schema
const feedbackSchema = z.object({
  userId: z.string().min(1),
  contentId: z.string().min(1),
  contentTitle: z.string().min(1),
  sessionId: z.string().min(1),

  emotionBefore: z.object({
    valence: z.number().min(-1).max(1),
    arousal: z.number().min(-1).max(1),
    stress: z.number().min(0).max(1),
  }),

  emotionAfter: z.object({
    valence: z.number().min(-1).max(1),
    arousal: z.number().min(-1).max(1),
    stress: z.number().min(0).max(1),
  }),

  desiredState: z.object({
    valence: z.number().min(-1).max(1),
    arousal: z.number().min(-1).max(1),
    stress: z.number().min(0).max(1),
  }),

  starRating: z.number().min(1).max(5),
  completed: z.boolean(),

  totalDuration: z.number().min(0), // Content length in ms
});

/**
 * POST /api/v1/feedback/submit
 * Submit comprehensive feedback after watching content
 */
router.post('/submit', async (req: Request, res: Response) => {
  try {
    const data = feedbackSchema.parse(req.body);

    // Get watch session
    const session = watchTracker.getSession(data.sessionId);

    // End session if not already ended
    if (!session.endTime) {
      watchTracker.endSession(data.sessionId, data.completed);
    }

    // Build submission
    const submission: FeedbackSubmission = {
      userId: data.userId,
      contentId: data.contentId,
      contentTitle: data.contentTitle,
      sessionId: data.sessionId,
      emotionBefore: {
        valence: data.emotionBefore.valence,
        arousal: data.emotionBefore.arousal,
        stressLevel: data.emotionBefore.stress,
        primaryEmotion: 'neutral',
        emotionVector: new Float32Array(8),
        confidence: 0.8,
        timestamp: session.startTime.getTime(),
      },
      emotionAfter: {
        valence: data.emotionAfter.valence,
        arousal: data.emotionAfter.arousal,
        stressLevel: data.emotionAfter.stress,
        primaryEmotion: 'neutral',
        emotionVector: new Float32Array(8),
        confidence: 0.8,
        timestamp: (session.endTime || new Date()).getTime(),
      },
      starRating: data.starRating,
      completed: data.completed,
      watchDuration: session.duration,
      totalDuration: data.totalDuration,
      timestamp: new Date(),
    };

    // Calculate reward
    const rewardCalc = rewardCalculator.calculate(
      submission.emotionBefore,
      submission.emotionAfter,
      {
        valence: data.desiredState.valence,
        arousal: data.desiredState.arousal,
        stressLevel: data.desiredState.stress,
        primaryEmotion: 'neutral',
        emotionVector: new Float32Array(8),
        confidence: 0.8,
        timestamp: Date.now(),
      },
      submission.completed,
      submission.starRating,
      submission.watchDuration,
      submission.totalDuration
    );

    // Store feedback (with placeholder Q-values)
    const record = await feedbackStore.saveFeedback(
      submission,
      rewardCalc.reward,
      0.5, // qValueBefore - would come from RL policy
      0.5 + rewardCalc.reward * 0.1, // qValueAfter - simplified
    );

    // Build emotion comparison
    const comparison: EmotionComparison = {
      before: submission.emotionBefore,
      after: submission.emotionAfter,
      delta: {
        valence: submission.emotionAfter.valence - submission.emotionBefore.valence,
        arousal: submission.emotionAfter.arousal - submission.emotionBefore.arousal,
        stress: submission.emotionAfter.stressLevel - submission.emotionBefore.stressLevel,
      },
      improvement: (rewardCalc.reward + 1) / 2, // Map [-1,1] to [0,1]
    };

    res.status(201).json(apiResponse({
      feedbackId: record.feedbackId,
      reward: {
        value: rewardCalc.reward,
        components: rewardCalc.components,
        explanation: rewardCalc.explanation,
      },
      emotionComparison: comparison,
      message: getRewardMessage(rewardCalc.reward),
      confetti: rewardCalc.reward > 0.7, // Show confetti for high rewards
    }));
  } catch (error) {
    if (error instanceof z.ZodError) {
      throw new ValidationError('Invalid request data', error.errors);
    }
    throw error;
  }
});

/**
 * GET /api/v1/feedback/:feedbackId
 * Get specific feedback record
 */
router.get('/:feedbackId', async (req: Request, res: Response) => {
  try {
    const { feedbackId } = req.params;

    const feedback = feedbackStore.getFeedback(feedbackId);

    if (!feedback) {
      throw new NotFoundError('Feedback', feedbackId);
    }

    res.json(apiResponse({
      feedback: {
        feedbackId: feedback.feedbackId,
        userId: feedback.userId,
        contentId: feedback.contentId,
        contentTitle: feedback.contentTitle,
        timestamp: feedback.timestamp,
        emotionBefore: feedback.emotionBefore,
        emotionAfter: feedback.emotionAfter,
        delta: {
          valence: feedback.emotionAfter.valence - feedback.emotionBefore.valence,
          arousal: feedback.emotionAfter.arousal - feedback.emotionBefore.arousal,
          stress: feedback.emotionAfter.stressLevel - feedback.emotionBefore.stressLevel,
        },
        reward: feedback.reward,
        starRating: feedback.starRating,
        completed: feedback.completed,
        watchDuration: feedback.watchDuration,
        completionPercentage: (feedback.watchDuration / feedback.totalDuration * 100).toFixed(1),
      },
    }));
  } catch (error) {
    throw error;
  }
});

/**
 * GET /api/v1/feedback/user/:userId/recent
 * Get user's recent feedback
 */
router.get('/user/:userId/recent', async (req: Request, res: Response) => {
  try {
    const { userId } = req.params;
    const { limit = '10' } = req.query;

    const limitNum = Math.min(50, Math.max(1, Number(limit)));
    const feedback = feedbackStore.getUserRecentFeedback(userId, limitNum);

    res.json(apiResponse({
      feedback: feedback.map(f => ({
        feedbackId: f.feedbackId,
        contentTitle: f.contentTitle,
        timestamp: f.timestamp,
        reward: f.reward,
        completed: f.completed,
        starRating: f.starRating,
      })),
      total: feedbackStore.countUserFeedback(userId),
    }));
  } catch (error) {
    throw error;
  }
});

/**
 * GET /api/v1/feedback/content/:contentId
 * Get feedback for specific content
 */
router.get('/content/:contentId', async (req: Request, res: Response) => {
  try {
    const { contentId } = req.params;

    const feedback = feedbackStore.getContentFeedback(contentId);

    // Calculate aggregate statistics
    const avgReward = feedback.reduce((sum, f) => sum + f.reward, 0) / feedback.length || 0;
    const avgRating = feedback.reduce((sum, f) => sum + f.starRating, 0) / feedback.length || 0;
    const completionRate = feedback.filter(f => f.completed).length / feedback.length || 0;

    res.json(apiResponse({
      contentId,
      statistics: {
        totalFeedback: feedback.length,
        averageReward: avgReward.toFixed(3),
        averageRating: avgRating.toFixed(1),
        completionRate: (completionRate * 100).toFixed(0) + '%',
      },
      recentFeedback: feedback.slice(0, 10).map(f => ({
        feedbackId: f.feedbackId,
        userId: f.userId,
        timestamp: f.timestamp,
        reward: f.reward,
        starRating: f.starRating,
      })),
    }));
  } catch (error) {
    throw error;
  }
});

export default router;
