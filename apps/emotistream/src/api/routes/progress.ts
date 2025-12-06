/**
 * Progress Analytics API Routes
 *
 * Routes for learning progress and analytics.
 */

import { Router, Request, Response } from 'express';
import { z } from 'zod';
import { ProgressAnalytics } from '../../services/progress-analytics.js';
import { getFeedbackStore } from '../../persistence/index.js';
import { apiResponse } from '../middleware/response.js';
import { NotFoundError } from '../../utils/errors.js';

const router = Router();
const progressAnalytics = new ProgressAnalytics();

// Get shared feedback store instance
const feedbackStore = getFeedbackStore();

// Helper functions
function classifyQuadrant(emotion: any): string {
  if (emotion.valence > 0 && emotion.arousal > 0) return 'Excited';
  if (emotion.valence > 0 && emotion.arousal < 0) return 'Calm';
  if (emotion.valence < 0 && emotion.arousal < 0) return 'Sad';
  if (emotion.valence < 0 && emotion.arousal > 0) return 'Stressed';
  return 'Neutral';
}

function average(values: number[]): number {
  if (values.length === 0) return 0;
  return values.reduce((sum, v) => sum + v, 0) / values.length;
}

function calculateTrendLine(rewards: number[]): number[] {
  const windowSize = Math.max(3, Math.floor(rewards.length / 10));
  const trendLine: number[] = [];

  for (let i = 0; i < rewards.length; i++) {
    const start = Math.max(0, i - windowSize + 1);
    const window = rewards.slice(start, i + 1);
    trendLine.push(average(window));
  }

  return trendLine;
}

function getStageDescription(stage: string): string {
  switch (stage) {
    case 'exploring':
      return 'Still learning your preferences';
    case 'learning':
      return 'Building a solid understanding';
    case 'confident':
      return 'Knows your preferences well';
    default:
      return 'Unknown stage';
  }
}

function getNextMilestone(experiences: number): { count: number; description: string } | null {
  if (experiences < 5) {
    return { count: 5, description: 'Complete 5 experiences to see initial trends' };
  }
  if (experiences < 20) {
    return { count: 20, description: 'Reach 20 experiences for personalized recommendations' };
  }
  if (experiences < 50) {
    return { count: 50, description: 'Complete 50 experiences for advanced insights' };
  }
  return null;
}

function getProgressColor(score: number): string {
  if (score < 30) return '#fbbf24'; // yellow - exploring
  if (score < 70) return '#3b82f6'; // blue - learning
  return '#10b981'; // green - confident
}

function getProgressLabel(score: number): string {
  if (score < 30) return 'Still exploring';
  if (score < 70) return 'Learning preferences';
  return 'Confident predictions';
}

/**
 * GET /api/v1/progress/:userId
 * Get comprehensive learning progress for user
 */
router.get('/:userId', async (req: Request, res: Response) => {
  try {
    const { userId } = req.params;

    const feedbackHistory = await feedbackStore.getUserFeedback(userId);
    const progress = progressAnalytics.calculateProgress(userId, feedbackHistory);

    res.json(apiResponse({
      progress: {
        // Overview
        totalExperiences: progress.totalExperiences,
        completedContent: progress.completedContent,
        completionRate: progress.totalExperiences > 0
          ? (progress.completedContent / progress.totalExperiences * 100).toFixed(1)
          : '0',

        // Rewards
        averageReward: progress.averageReward.toFixed(3),
        rewardTrend: progress.rewardTrend,
        recentRewards: progress.recentRewards,

        // Exploration
        explorationRate: (progress.explorationRate * 100).toFixed(1) + '%',
        explorationCount: progress.explorationCount,
        exploitationCount: progress.exploitationCount,

        // Convergence
        convergence: {
          score: progress.convergenceScore,
          stage: progress.convergenceStage,
          percentage: Math.round(progress.convergenceScore),
        },

        // Summary
        summary: {
          level: progress.convergenceStage,
          description: getStageDescription(progress.convergenceStage),
          nextMilestone: getNextMilestone(progress.totalExperiences),
        },

        timestamp: progress.timestamp,
      },
    }));
  } catch (error) {
    throw error;
  }
});

/**
 * GET /api/v1/progress/:userId/convergence
 * Get detailed convergence analysis
 */
router.get('/:userId/convergence', async (req: Request, res: Response) => {
  try {
    const { userId } = req.params;

    const feedbackHistory = await feedbackStore.getUserFeedback(userId);
    const convergence = progressAnalytics.analyzeConvergence(feedbackHistory);

    res.json(apiResponse({
      convergence: {
        score: Math.round(convergence.score),
        stage: convergence.stage,
        explanation: convergence.explanation,
        metrics: {
          qValueStability: (convergence.metrics.qValueStability * 100).toFixed(1) + '%',
          rewardVariance: convergence.metrics.rewardVariance.toFixed(3),
          explorationRate: (convergence.metrics.explorationRate * 100).toFixed(1) + '%',
          policyChanges: convergence.metrics.policyChanges,
        },
        recommendations: convergence.recommendations,
        progressBar: {
          percentage: Math.round(convergence.score),
          color: getProgressColor(convergence.score),
          label: getProgressLabel(convergence.score),
        },
      },
    }));
  } catch (error) {
    throw error;
  }
});

/**
 * GET /api/v1/progress/:userId/journey
 * Get emotional journey visualization data
 */
router.get('/:userId/journey', async (req: Request, res: Response) => {
  try {
    const { userId } = req.params;
    const { limit } = req.query;

    const feedbackHistory = await feedbackStore.getUserFeedback(userId);
    const progress = progressAnalytics.calculateProgress(userId, feedbackHistory);

    let journey = progress.emotionalJourney;

    // Limit results if requested
    if (limit && !isNaN(Number(limit))) {
      journey = journey.slice(-Number(limit));
    }

    res.json(apiResponse({
      journey: journey.map(point => ({
        experienceNumber: point.experienceNumber,
        timestamp: point.timestamp,
        contentId: point.contentId,
        contentTitle: point.contentTitle,
        emotionBefore: {
          valence: point.emotionBefore.valence,
          arousal: point.emotionBefore.arousal,
          stress: point.emotionBefore.stressLevel,
        },
        emotionAfter: {
          valence: point.emotionAfter.valence,
          arousal: point.emotionAfter.arousal,
          stress: point.emotionAfter.stressLevel,
        },
        delta: {
          valence: point.emotionAfter.valence - point.emotionBefore.valence,
          arousal: point.emotionAfter.arousal - point.emotionBefore.arousal,
          stress: point.emotionAfter.stressLevel - point.emotionBefore.stressLevel,
        },
        reward: point.reward,
        completed: point.completed,
        // Quadrant classification
        quadrant: classifyQuadrant(point.emotionAfter),
      })),
      totalPoints: progress.emotionalJourney.length,
    }));
  } catch (error) {
    throw error;
  }
});

/**
 * GET /api/v1/progress/:userId/rewards
 * Get reward timeline data
 */
router.get('/:userId/rewards', async (req: Request, res: Response) => {
  try {
    const { userId } = req.params;

    const feedbackHistory = await feedbackStore.getUserFeedback(userId);

    const timeline = feedbackHistory.map((feedback, index) => ({
      experienceNumber: index + 1,
      timestamp: feedback.timestamp,
      reward: feedback.reward,
      contentTitle: feedback.contentTitle,
      contentId: feedback.contentId,
      completed: feedback.completed,
      starRating: feedback.starRating,
    }));

    // Calculate trend line (simple moving average)
    const trendLine = calculateTrendLine(timeline.map(t => t.reward));

    res.json(apiResponse({
      timeline,
      trendLine,
      statistics: {
        average: average(timeline.map(t => t.reward)),
        highest: Math.max(...timeline.map(t => t.reward)),
        lowest: Math.min(...timeline.map(t => t.reward)),
        recent: timeline.slice(-10).map(t => t.reward),
      },
    }));
  } catch (error) {
    throw error;
  }
});

/**
 * GET /api/v1/progress/:userId/content
 * Get content performance rankings
 */
router.get('/:userId/content', async (req: Request, res: Response) => {
  try {
    const { userId } = req.params;

    const feedbackHistory = await feedbackStore.getUserFeedback(userId);
    const progress = progressAnalytics.calculateProgress(userId, feedbackHistory);

    res.json(apiResponse({
      bestContent: progress.bestContent.map(c => ({
        contentId: c.contentId,
        contentTitle: c.contentTitle,
        timesWatched: c.timesWatched,
        averageReward: c.averageReward.toFixed(3),
        completionRate: (c.completionRate * 100).toFixed(0) + '%',
        averageRating: c.averageRating.toFixed(1),
        lastWatched: c.lastWatched,
      })),
      worstContent: progress.worstContent.map(c => ({
        contentId: c.contentId,
        contentTitle: c.contentTitle,
        timesWatched: c.timesWatched,
        averageReward: c.averageReward.toFixed(3),
        completionRate: (c.completionRate * 100).toFixed(0) + '%',
        averageRating: c.averageRating.toFixed(1),
        lastWatched: c.lastWatched,
      })),
    }));
  } catch (error) {
    throw error;
  }
});

/**
 * GET /api/v1/progress/:userId/experiences
 * Get recent experiences list
 */
router.get('/:userId/experiences', async (req: Request, res: Response) => {
  try {
    const { userId } = req.params;
    const { limit = '10' } = req.query;

    const feedbackHistory = await feedbackStore.getUserFeedback(userId);
    const limitNum = Math.min(50, Math.max(1, Number(limit)));

    const experiences = feedbackHistory
      .slice(-limitNum)
      .reverse()
      .map((feedback, index) => ({
        experienceId: feedback.feedbackId,
        experienceNumber: feedbackHistory.length - index,
        timestamp: feedback.timestamp,
        contentId: feedback.contentId,
        contentTitle: feedback.contentTitle,
        emotionChange: {
          before: feedback.emotionBefore,
          after: feedback.emotionAfter,
          delta: {
            valence: feedback.emotionAfter.valence - feedback.emotionBefore.valence,
            arousal: feedback.emotionAfter.arousal - feedback.emotionBefore.arousal,
            stress: feedback.emotionAfter.stressLevel - feedback.emotionBefore.stressLevel,
          },
          improvement: feedback.reward, // Simplified
        },
        reward: feedback.reward,
        starRating: feedback.starRating,
        completed: feedback.completed,
        watchDuration: feedback.watchDuration,
        completionPercentage: (feedback.watchDuration / feedback.totalDuration * 100).toFixed(0),
      }));

    res.json(apiResponse({
      experiences,
      total: feedbackHistory.length,
      showing: experiences.length,
    }));
  } catch (error) {
    throw error;
  }
});

export default router;
