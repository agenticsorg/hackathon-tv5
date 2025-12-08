/**
 * PostgreSQL-based Feedback Store
 *
 * Replaces the in-memory/JSON file storage with PostgreSQL persistence.
 */

import { query, queryOne, queryAll, withTransaction } from './postgres-client.js';
import { FeedbackSubmission, StoredFeedback } from '../types/feedback.js';
import { EmotionalState } from '../emotion/types.js';

export class PostgresFeedbackStore {
  /**
   * Save feedback to database
   */
  async saveFeedback(
    submission: FeedbackSubmission,
    reward: number,
    qValueBefore: number,
    qValueAfter: number
  ): Promise<StoredFeedback> {
    const result = await queryOne<StoredFeedback>(
      `INSERT INTO feedback (
        user_id, content_id, content_title, session_id,
        emotion_before_valence, emotion_before_arousal, emotion_before_stress,
        emotion_before_primary, emotion_before_confidence,
        emotion_after_valence, emotion_after_arousal, emotion_after_stress,
        emotion_after_primary, emotion_after_confidence,
        star_rating, completed, watch_duration_ms, total_duration_ms,
        reward, q_value_before, q_value_after
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
      RETURNING *`,
      [
        submission.userId,
        submission.contentId,
        submission.contentTitle,
        submission.sessionId,
        submission.emotionBefore.valence,
        submission.emotionBefore.arousal,
        submission.emotionBefore.stressLevel,
        submission.emotionBefore.primaryEmotion,
        submission.emotionBefore.confidence,
        submission.emotionAfter.valence,
        submission.emotionAfter.arousal,
        submission.emotionAfter.stressLevel,
        submission.emotionAfter.primaryEmotion,
        submission.emotionAfter.confidence,
        submission.starRating,
        submission.completed,
        submission.watchDuration,
        submission.totalDuration,
        reward,
        qValueBefore,
        qValueAfter,
      ]
    );

    return this.mapRowToStoredFeedback(result!);
  }

  /**
   * Get user feedback history
   */
  async getUserFeedback(userId: string, limit = 100): Promise<StoredFeedback[]> {
    const rows = await queryAll<any>(
      `SELECT f.*, c.title as content_title
       FROM feedback f
       LEFT JOIN content c ON f.content_id = c.id
       WHERE f.user_id = $1
       ORDER BY f.created_at DESC
       LIMIT $2`,
      [userId, limit]
    );

    return rows.map((row) => this.mapRowToStoredFeedback(row));
  }

  /**
   * Get feedback for specific content
   */
  async getContentFeedback(contentId: string): Promise<StoredFeedback[]> {
    const rows = await queryAll<any>(
      `SELECT f.*, c.title as content_title
       FROM feedback f
       LEFT JOIN content c ON f.content_id = c.id
       WHERE f.content_id = $1
       ORDER BY f.created_at DESC`,
      [contentId]
    );

    return rows.map((row) => this.mapRowToStoredFeedback(row));
  }

  /**
   * Get user's feedback count
   */
  async getUserFeedbackCount(userId: string): Promise<number> {
    const result = await queryOne<{ count: string }>(
      'SELECT COUNT(*) as count FROM feedback WHERE user_id = $1',
      [userId]
    );
    return parseInt(result?.count || '0');
  }

  /**
   * Get average reward for user
   */
  async getUserAverageReward(userId: string): Promise<number> {
    const result = await queryOne<{ avg: string }>(
      'SELECT AVG(reward) as avg FROM feedback WHERE user_id = $1',
      [userId]
    );
    return parseFloat(result?.avg || '0');
  }

  /**
   * Get recent rewards for user
   */
  async getUserRecentRewards(userId: string, limit = 10): Promise<number[]> {
    const rows = await queryAll<{ reward: number }>(
      'SELECT reward FROM feedback WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2',
      [userId, limit]
    );
    return rows.map((r) => r.reward);
  }

  /**
   * Map database row to StoredFeedback object
   */
  private mapRowToStoredFeedback(row: any): StoredFeedback {
    const emotionBefore: EmotionalState = {
      valence: row.emotion_before_valence,
      arousal: row.emotion_before_arousal,
      stressLevel: row.emotion_before_stress,
      primaryEmotion: row.emotion_before_primary || 'neutral',
      emotionVector: new Float32Array(8),
      confidence: row.emotion_before_confidence,
      timestamp: new Date(row.created_at).getTime(),
    };

    const emotionAfter: EmotionalState = {
      valence: row.emotion_after_valence,
      arousal: row.emotion_after_arousal,
      stressLevel: row.emotion_after_stress,
      primaryEmotion: row.emotion_after_primary || 'neutral',
      emotionVector: new Float32Array(8),
      confidence: row.emotion_after_confidence,
      timestamp: new Date(row.created_at).getTime(),
    };

    return {
      feedbackId: row.id,
      userId: row.user_id,
      contentId: row.content_id,
      contentTitle: row.content_title || row.content_id,
      sessionId: row.session_id || '',
      emotionBefore,
      emotionAfter,
      starRating: row.star_rating,
      completed: row.completed,
      watchDuration: row.watch_duration_ms,
      totalDuration: row.total_duration_ms,
      reward: row.reward,
      qValueBefore: row.q_value_before,
      qValueAfter: row.q_value_after,
      processed: true,
      timestamp: new Date(row.created_at),
    };
  }
}

/**
 * PostgreSQL-based Q-Value Store for RL Policy
 */
export class PostgresQValueStore {
  /**
   * Get Q-value for state-action pair
   */
  async getQValue(userId: string, stateKey: string, contentId: string): Promise<number> {
    const result = await queryOne<{ q_value: number }>(
      'SELECT q_value FROM q_values WHERE user_id = $1 AND state_key = $2 AND content_id = $3',
      [userId, stateKey, contentId]
    );
    return result?.q_value ?? 0;
  }

  /**
   * Set Q-value for state-action pair
   */
  async setQValue(
    userId: string,
    stateKey: string,
    contentId: string,
    qValue: number
  ): Promise<void> {
    await query(
      `INSERT INTO q_values (user_id, state_key, content_id, q_value, visit_count, last_updated)
       VALUES ($1, $2, $3, $4, 1, NOW())
       ON CONFLICT (user_id, state_key, content_id)
       DO UPDATE SET q_value = $4, visit_count = q_values.visit_count + 1, last_updated = NOW()`,
      [userId, stateKey, contentId, qValue]
    );
  }

  /**
   * Get all Q-values for a user's state
   */
  async getStateQValues(userId: string, stateKey: string): Promise<Map<string, number>> {
    const rows = await queryAll<{ content_id: string; q_value: number }>(
      'SELECT content_id, q_value FROM q_values WHERE user_id = $1 AND state_key = $2',
      [userId, stateKey]
    );

    const map = new Map<string, number>();
    for (const row of rows) {
      map.set(row.content_id, row.q_value);
    }
    return map;
  }

  /**
   * Get best action (highest Q-value) for a state
   */
  async getBestAction(userId: string, stateKey: string): Promise<string | null> {
    const result = await queryOne<{ content_id: string }>(
      `SELECT content_id FROM q_values
       WHERE user_id = $1 AND state_key = $2
       ORDER BY q_value DESC
       LIMIT 1`,
      [userId, stateKey]
    );
    return result?.content_id ?? null;
  }

  /**
   * Get total visit count for user
   */
  async getTotalVisits(userId: string): Promise<number> {
    const result = await queryOne<{ total: string }>(
      'SELECT SUM(visit_count) as total FROM q_values WHERE user_id = $1',
      [userId]
    );
    return parseInt(result?.total || '0');
  }
}

/**
 * PostgreSQL-based User Store
 */
export class PostgresUserStore {
  /**
   * Create a new user
   */
  async createUser(
    email: string,
    passwordHash: string,
    displayName?: string
  ): Promise<{ id: string; email: string; displayName: string | null }> {
    const result = await queryOne<any>(
      `INSERT INTO users (email, password_hash, display_name)
       VALUES ($1, $2, $3)
       RETURNING id, email, display_name`,
      [email, passwordHash, displayName]
    );

    return {
      id: result.id,
      email: result.email,
      displayName: result.display_name,
    };
  }

  /**
   * Find user by email
   */
  async findByEmail(email: string): Promise<{
    id: string;
    email: string;
    passwordHash: string;
    displayName: string | null;
  } | null> {
    const result = await queryOne<any>(
      'SELECT id, email, password_hash, display_name FROM users WHERE email = $1',
      [email]
    );

    if (!result) return null;

    return {
      id: result.id,
      email: result.email,
      passwordHash: result.password_hash,
      displayName: result.display_name,
    };
  }

  /**
   * Find user by ID
   */
  async findById(id: string): Promise<{
    id: string;
    email: string;
    displayName: string | null;
  } | null> {
    const result = await queryOne<any>(
      'SELECT id, email, display_name FROM users WHERE id = $1',
      [id]
    );

    if (!result) return null;

    return {
      id: result.id,
      email: result.email,
      displayName: result.display_name,
    };
  }

  /**
   * Check if email exists
   */
  async emailExists(email: string): Promise<boolean> {
    const result = await queryOne<{ exists: boolean }>(
      'SELECT EXISTS(SELECT 1 FROM users WHERE email = $1) as exists',
      [email]
    );
    return result?.exists ?? false;
  }
}

/**
 * PostgreSQL-based Content Store
 */
export class PostgresContentStore {
  /**
   * Get all content
   */
  async getAllContent(): Promise<any[]> {
    return queryAll(
      'SELECT id, title, description, category, duration_minutes, emotional_profile, tags FROM content'
    );
  }

  /**
   * Get content by ID
   */
  async getContentById(id: string): Promise<any | null> {
    return queryOne(
      'SELECT id, title, description, category, duration_minutes, emotional_profile, tags FROM content WHERE id = $1',
      [id]
    );
  }

  /**
   * Get content by category
   */
  async getContentByCategory(category: string): Promise<any[]> {
    return queryAll(
      'SELECT id, title, description, category, duration_minutes, emotional_profile, tags FROM content WHERE category = $1',
      [category]
    );
  }
}

/**
 * PostgreSQL-based Recommendation History Store
 */
export class PostgresRecommendationHistoryStore {
  /**
   * Save recommendation to history
   */
  async saveRecommendation(
    userId: string,
    recommendation: {
      contentId: string;
      title: string;
      qValue: number;
      similarityScore: number;
      combinedScore: number;
      isExploration: boolean;
      reasoning: string;
    },
    state: {
      valence: number;
      arousal: number;
      stress: number;
    }
  ): Promise<void> {
    await query(
      `INSERT INTO recommendation_history (
        user_id, content_id, content_title, q_value, similarity_score,
        combined_score, is_exploration, reasoning,
        state_valence, state_arousal, state_stress
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)`,
      [
        userId,
        recommendation.contentId,
        recommendation.title,
        recommendation.qValue,
        recommendation.similarityScore,
        recommendation.combinedScore,
        recommendation.isExploration,
        recommendation.reasoning,
        state.valence,
        state.arousal,
        state.stress,
      ]
    );
  }

  /**
   * Get recommendation history for user
   */
  async getUserHistory(
    userId: string,
    limit = 50
  ): Promise<
    Array<{
      id: string;
      contentId: string;
      contentTitle: string;
      qValue: number;
      similarityScore: number;
      combinedScore: number;
      isExploration: boolean;
      reasoning: string;
      stateValence: number;
      stateArousal: number;
      stateStress: number;
      createdAt: Date;
    }>
  > {
    const rows = await queryAll<{
      id: string;
      content_id: string;
      content_title: string;
      q_value: number;
      similarity_score: number;
      combined_score: number;
      is_exploration: boolean;
      reasoning: string;
      state_valence: number;
      state_arousal: number;
      state_stress: number;
      created_at: Date;
    }>(
      `SELECT id, content_id, content_title, q_value, similarity_score,
              combined_score, is_exploration, reasoning,
              state_valence, state_arousal, state_stress, created_at
       FROM recommendation_history
       WHERE user_id = $1
       ORDER BY created_at DESC
       LIMIT $2`,
      [userId, limit]
    );

    return rows.map((row) => ({
      id: row.id,
      contentId: row.content_id,
      contentTitle: row.content_title,
      qValue: row.q_value,
      similarityScore: row.similarity_score,
      combinedScore: row.combined_score,
      isExploration: row.is_exploration,
      reasoning: row.reasoning,
      stateValence: row.state_valence,
      stateArousal: row.state_arousal,
      stateStress: row.state_stress,
      createdAt: new Date(row.created_at),
    }));
  }

  /**
   * Get recommendation count for user
   */
  async getUserRecommendationCount(userId: string): Promise<number> {
    const result = await queryOne<{ count: string }>(
      'SELECT COUNT(*) as count FROM recommendation_history WHERE user_id = $1',
      [userId]
    );
    return parseInt(result?.count || '0');
  }
}

/**
 * PostgreSQL-based Emotion History Store
 */
export class PostgresEmotionHistoryStore {
  /**
   * Save emotion analysis to history
   */
  async saveAnalysis(
    userId: string,
    inputText: string,
    state: {
      valence: number;
      arousal: number;
      stressLevel: number;
      primaryEmotion: string;
      confidence: number;
    }
  ): Promise<string> {
    const result = await queryOne<{ id: string }>(
      `INSERT INTO emotion_analyses (
        user_id, input_text, valence, arousal, stress_level,
        primary_emotion, confidence
      ) VALUES ($1, $2, $3, $4, $5, $6, $7)
      RETURNING id`,
      [
        userId,
        inputText,
        state.valence,
        state.arousal,
        state.stressLevel,
        state.primaryEmotion,
        state.confidence,
      ]
    );
    return result?.id || '';
  }

  /**
   * Get emotion history for user
   */
  async getUserHistory(
    userId: string,
    limit = 50
  ): Promise<
    Array<{
      id: string;
      inputText: string;
      valence: number;
      arousal: number;
      stressLevel: number;
      primaryEmotion: string;
      confidence: number;
      createdAt: Date;
    }>
  > {
    const rows = await queryAll<{
      id: string;
      input_text: string;
      valence: number;
      arousal: number;
      stress_level: number;
      primary_emotion: string;
      confidence: number;
      created_at: Date;
    }>(
      `SELECT id, input_text, valence, arousal, stress_level,
              primary_emotion, confidence, created_at
       FROM emotion_analyses
       WHERE user_id = $1
       ORDER BY created_at DESC
       LIMIT $2`,
      [userId, limit]
    );

    return rows.map((row) => ({
      id: row.id,
      inputText: row.input_text,
      valence: row.valence,
      arousal: row.arousal,
      stressLevel: row.stress_level,
      primaryEmotion: row.primary_emotion,
      confidence: row.confidence,
      createdAt: new Date(row.created_at),
    }));
  }

  /**
   * Get emotion analysis count for user
   */
  async getUserAnalysisCount(userId: string): Promise<number> {
    const result = await queryOne<{ count: string }>(
      'SELECT COUNT(*) as count FROM emotion_analyses WHERE user_id = $1',
      [userId]
    );
    return parseInt(result?.count || '0');
  }
}

// Singleton instances
let feedbackStore: PostgresFeedbackStore | null = null;
let qValueStore: PostgresQValueStore | null = null;
let userStore: PostgresUserStore | null = null;
let contentStore: PostgresContentStore | null = null;
let recommendationHistoryStore: PostgresRecommendationHistoryStore | null = null;
let emotionHistoryStore: PostgresEmotionHistoryStore | null = null;

export function getPostgresFeedbackStore(): PostgresFeedbackStore {
  if (!feedbackStore) {
    feedbackStore = new PostgresFeedbackStore();
  }
  return feedbackStore;
}

export function getPostgresQValueStore(): PostgresQValueStore {
  if (!qValueStore) {
    qValueStore = new PostgresQValueStore();
  }
  return qValueStore;
}

export function getPostgresUserStore(): PostgresUserStore {
  if (!userStore) {
    userStore = new PostgresUserStore();
  }
  return userStore;
}

export function getPostgresContentStore(): PostgresContentStore {
  if (!contentStore) {
    contentStore = new PostgresContentStore();
  }
  return contentStore;
}

export function getPostgresRecommendationHistoryStore(): PostgresRecommendationHistoryStore {
  if (!recommendationHistoryStore) {
    recommendationHistoryStore = new PostgresRecommendationHistoryStore();
  }
  return recommendationHistoryStore;
}

export function getPostgresEmotionHistoryStore(): PostgresEmotionHistoryStore {
  if (!emotionHistoryStore) {
    emotionHistoryStore = new PostgresEmotionHistoryStore();
  }
  return emotionHistoryStore;
}
