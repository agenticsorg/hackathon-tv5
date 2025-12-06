/**
 * Progress Analytics Service
 *
 * Analyzes user learning progress, convergence, and emotional journey.
 */

import {
  LearningProgress,
  ConvergenceAnalysis,
  EmotionalJourneyPoint,
  ContentPerformance,
  FeedbackRecord,
} from '../types/feedback.js';
import { EmotionalState } from '../types/index.js';

export class ProgressAnalytics {
  /**
   * Calculate comprehensive learning progress
   */
  calculateProgress(
    userId: string,
    feedbackHistory: FeedbackRecord[]
  ): LearningProgress {
    const userFeedback = feedbackHistory.filter(f => f.userId === userId);

    if (userFeedback.length === 0) {
      return this.getEmptyProgress(userId);
    }

    // Sort by timestamp
    userFeedback.sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime());

    const totalExperiences = userFeedback.length;
    const completedContent = userFeedback.filter(f => f.completed).length;

    // Reward statistics
    const rewards = userFeedback.map(f => f.reward);
    const averageReward = this.average(rewards);
    const recentRewards = rewards.slice(-10);
    const rewardTrend = this.calculateTrend(rewards);

    // Exploration metrics (would come from RL policy in real system)
    const explorationRate = Math.max(0.1, 0.3 - totalExperiences * 0.01);

    // Convergence
    const convergenceAnalysis = this.analyzeConvergence(userFeedback);

    // Emotional journey
    const emotionalJourney = this.buildEmotionalJourney(userFeedback);

    // Content performance
    const contentPerformance = this.analyzeContentPerformance(userFeedback);

    return {
      userId,
      totalExperiences,
      completedContent,
      averageReward,
      rewardTrend,
      recentRewards,
      explorationRate,
      explorationCount: Math.floor(totalExperiences * explorationRate),
      exploitationCount: Math.floor(totalExperiences * (1 - explorationRate)),
      convergenceScore: convergenceAnalysis.score,
      convergenceStage: convergenceAnalysis.stage,
      emotionalJourney,
      bestContent: contentPerformance.best,
      worstContent: contentPerformance.worst,
      timestamp: new Date(),
    };
  }

  /**
   * Analyze policy convergence
   */
  analyzeConvergence(feedbackHistory: FeedbackRecord[]): ConvergenceAnalysis {
    if (feedbackHistory.length < 5) {
      return {
        score: 0,
        stage: 'exploring',
        explanation: 'Just getting started! Keep watching to help the system learn your preferences.',
        metrics: {
          qValueStability: 0,
          rewardVariance: 1,
          explorationRate: 0.3,
          policyChanges: 0,
        },
        recommendations: [
          'Watch more content to build your profile',
          'Try diverse content types',
          'Provide honest feedback',
        ],
      };
    }

    const rewards = feedbackHistory.map(f => f.reward);
    const qValueChanges = feedbackHistory.map(f => Math.abs(f.qValueAfter - f.qValueBefore));

    // Metrics
    const rewardVariance = this.variance(rewards.slice(-10));
    const qValueStability = 1 - this.average(qValueChanges.slice(-10));
    const recentRewards = rewards.slice(-10);
    const avgRecentReward = this.average(recentRewards);

    // Overall score (0-100)
    let score = 0;
    score += (1 - rewardVariance) * 40; // Low variance = good
    score += qValueStability * 30; // Stable Q-values = good
    score += (avgRecentReward + 1) * 15; // High rewards = good
    score += Math.min(feedbackHistory.length / 50, 1) * 15; // More experience = good

    // Determine stage
    let stage: 'exploring' | 'learning' | 'confident';
    if (score < 30) {
      stage = 'exploring';
    } else if (score < 70) {
      stage = 'learning';
    } else {
      stage = 'confident';
    }

    const explanation = this.generateConvergenceExplanation(score, stage, feedbackHistory.length);
    const recommendations = this.generateRecommendations(score, stage);

    return {
      score,
      stage,
      explanation,
      metrics: {
        qValueStability,
        rewardVariance,
        explorationRate: Math.max(0.1, 0.3 - feedbackHistory.length * 0.01),
        policyChanges: qValueChanges.filter(c => c > 0.1).length,
      },
      recommendations,
    };
  }

  /**
   * Build emotional journey visualization data
   */
  private buildEmotionalJourney(
    feedbackHistory: FeedbackRecord[]
  ): EmotionalJourneyPoint[] {
    return feedbackHistory.map((feedback, index) => ({
      experienceNumber: index + 1,
      timestamp: feedback.timestamp,
      contentId: feedback.contentId,
      contentTitle: feedback.contentTitle,
      emotionBefore: feedback.emotionBefore,
      emotionAfter: feedback.emotionAfter,
      reward: feedback.reward,
      completed: feedback.completed,
    }));
  }

  /**
   * Analyze content performance
   */
  private analyzeContentPerformance(
    feedbackHistory: FeedbackRecord[]
  ): { best: ContentPerformance[]; worst: ContentPerformance[] } {
    const contentMap = new Map<string, {
      contentId: string;
      contentTitle: string;
      rewards: number[];
      completed: number;
      total: number;
      ratings: number[];
      lastWatched: Date;
    }>();

    // Aggregate by content
    for (const feedback of feedbackHistory) {
      const existing = contentMap.get(feedback.contentId);

      if (existing) {
        existing.rewards.push(feedback.reward);
        existing.ratings.push(feedback.starRating);
        existing.total += 1;
        if (feedback.completed) existing.completed += 1;
        if (feedback.timestamp > existing.lastWatched) {
          existing.lastWatched = feedback.timestamp;
        }
      } else {
        contentMap.set(feedback.contentId, {
          contentId: feedback.contentId,
          contentTitle: feedback.contentTitle,
          rewards: [feedback.reward],
          ratings: [feedback.starRating],
          completed: feedback.completed ? 1 : 0,
          total: 1,
          lastWatched: feedback.timestamp,
        });
      }
    }

    // Convert to performance records
    const performances: ContentPerformance[] = Array.from(contentMap.values()).map(data => ({
      contentId: data.contentId,
      contentTitle: data.contentTitle,
      timesWatched: data.total,
      averageReward: this.average(data.rewards),
      completionRate: data.completed / data.total,
      averageRating: this.average(data.ratings),
      lastWatched: data.lastWatched,
    }));

    // Sort by average reward
    performances.sort((a, b) => b.averageReward - a.averageReward);

    return {
      best: performances.slice(0, 5),
      worst: performances.slice(-5).reverse(),
    };
  }

  /**
   * Calculate trend direction
   */
  private calculateTrend(values: number[]): 'improving' | 'stable' | 'declining' {
    if (values.length < 5) return 'stable';

    const recent = values.slice(-10);
    const older = values.slice(-20, -10);

    if (older.length === 0) return 'stable';

    const recentAvg = this.average(recent);
    const olderAvg = this.average(older);

    const diff = recentAvg - olderAvg;

    if (diff > 0.1) return 'improving';
    if (diff < -0.1) return 'declining';
    return 'stable';
  }

  /**
   * Generate convergence explanation
   */
  private generateConvergenceExplanation(
    score: number,
    stage: string,
    experiences: number
  ): string {
    if (stage === 'exploring') {
      return `You're in the exploration phase (${score.toFixed(0)}% confident). The system is learning your preferences through ${experiences} experiences.`;
    } else if (stage === 'learning') {
      return `Good progress! (${score.toFixed(0)}% confident). After ${experiences} experiences, the system is developing a solid understanding of what you enjoy.`;
    } else {
      return `Excellent! (${score.toFixed(0)}% confident). With ${experiences} experiences, the system knows your preferences well and can make reliable recommendations.`;
    }
  }

  /**
   * Generate recommendations for improvement
   */
  private generateRecommendations(score: number, stage: string): string[] {
    if (stage === 'exploring') {
      return [
        'Continue watching to build your profile',
        'Try different content types',
        'Provide detailed feedback after watching',
      ];
    } else if (stage === 'learning') {
      return [
        'Your profile is developing well',
        'Keep completing content you start',
        'Rate content honestly to refine recommendations',
      ];
    } else {
      return [
        'Your preferences are well-established',
        'Explore new genres occasionally to discover surprises',
        'The system will maintain high-quality recommendations',
      ];
    }
  }

  /**
   * Get empty progress for new users
   */
  private getEmptyProgress(userId: string): LearningProgress {
    return {
      userId,
      totalExperiences: 0,
      completedContent: 0,
      averageReward: 0,
      rewardTrend: 'stable',
      recentRewards: [],
      explorationRate: 0.3,
      explorationCount: 0,
      exploitationCount: 0,
      convergenceScore: 0,
      convergenceStage: 'exploring',
      emotionalJourney: [],
      bestContent: [],
      worstContent: [],
      timestamp: new Date(),
    };
  }

  /**
   * Calculate average of number array
   */
  private average(values: number[]): number {
    if (values.length === 0) return 0;
    return values.reduce((sum, v) => sum + v, 0) / values.length;
  }

  /**
   * Calculate variance of number array
   */
  private variance(values: number[]): number {
    if (values.length < 2) return 0;
    const avg = this.average(values);
    const squaredDiffs = values.map(v => (v - avg) ** 2);
    return this.average(squaredDiffs);
  }
}
