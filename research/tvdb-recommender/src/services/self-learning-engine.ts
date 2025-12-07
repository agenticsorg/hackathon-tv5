/**
 * Self-Learning Recommendation Engine
 *
 * Implements advanced self-learning capabilities:
 * - ReasoningBank pattern recognition
 * - Q-Learning with experience replay
 * - Multi-armed bandit exploration
 * - Contextual bandits for personalization
 * - Continuous model improvement
 */

import { Pool } from 'pg';
import pino from 'pino';

const logger = pino({ name: 'self-learning-engine' });

// ============================================================================
// TYPES
// ============================================================================

interface LearningConfig {
  learningRate: number;           // Alpha: how fast to learn
  discountFactor: number;         // Gamma: future reward importance
  explorationRate: number;        // Epsilon: exploration vs exploitation
  explorationDecay: number;       // How fast to reduce exploration
  minExplorationRate: number;     // Minimum exploration rate
  replayBufferSize: number;       // Max experiences to store
  batchSize: number;              // Batch size for replay learning
  targetUpdateFrequency: number;  // How often to update target Q
}

interface Experience {
  state: string;
  action: string;
  reward: number;
  nextState: string;
  done: boolean;
  context: {
    userId?: string;
    contentId: string;
    timestamp: number;
    mood?: string;
    genres?: string[];
  };
}

interface ReasoningPattern {
  id: string;
  condition: string;
  action: string;
  successRate: number;
  confidence: number;
  sampleCount: number;
  lastUpdated: Date;
}

interface LearningStats {
  totalEpisodes: number;
  avgReward: number;
  explorationRate: number;
  qTableSize: number;
  patternCount: number;
  topPatterns: ReasoningPattern[];
  recentPerformance: number[];
}

// ============================================================================
// SELF-LEARNING ENGINE
// ============================================================================

export class SelfLearningEngine {
  private pool: Pool;
  private config: LearningConfig;
  private qTable: Map<string, Map<string, number>> = new Map();
  private replayBuffer: Experience[] = [];
  private patterns: Map<string, ReasoningPattern> = new Map();
  private episode: number = 0;
  private rewardHistory: number[] = [];

  // Available actions (recommendation strategies)
  private readonly ACTIONS = [
    'content_based',           // Recommend based on content similarity
    'collaborative',           // Recommend based on similar users
    'genre_weighted',          // Weight by user's preferred genres
    'recency_boosted',         // Prefer newer content
    'popularity_boosted',      // Prefer popular content
    'diversity_enhanced',      // Increase variety in recommendations
    'mood_matched',            // Match content to user's mood
    'binge_optimized',         // Optimize for series binge-watching
    'discovery_mode',          // Surface hidden gems
    'trending_focus'           // Focus on trending content
  ];

  constructor(config?: Partial<LearningConfig>) {
    this.config = {
      learningRate: config?.learningRate ?? 0.1,
      discountFactor: config?.discountFactor ?? 0.95,
      explorationRate: config?.explorationRate ?? 0.3,
      explorationDecay: config?.explorationDecay ?? 0.995,
      minExplorationRate: config?.minExplorationRate ?? 0.05,
      replayBufferSize: config?.replayBufferSize ?? 10000,
      batchSize: config?.batchSize ?? 32,
      targetUpdateFrequency: config?.targetUpdateFrequency ?? 100
    };

    this.pool = new Pool({
      connectionString: process.env.DATABASE_URL ||
        'postgresql://tvdb:tvdb_recommender_2024@localhost:5432/tvdb_recommender',
      max: 10
    });
  }

  /**
   * Initialize the learning engine
   */
  async initialize(): Promise<void> {
    logger.info('Initializing self-learning engine');

    const client = await this.pool.connect();
    try {
      // Create learning tables
      await this.createLearningTables(client);

      // Load existing Q-table
      await this.loadQTable(client);

      // Load reasoning patterns
      await this.loadPatterns(client);

      // Load replay buffer
      await this.loadReplayBuffer(client);

      logger.info({
        qTableSize: this.qTable.size,
        patternCount: this.patterns.size,
        replayBufferSize: this.replayBuffer.length,
        episode: this.episode
      }, 'Self-learning engine initialized');

    } finally {
      client.release();
    }
  }

  /**
   * Create learning infrastructure tables
   */
  private async createLearningTables(client: any): Promise<void> {
    await client.query(`
      CREATE TABLE IF NOT EXISTS q_table (
        state VARCHAR(200) NOT NULL,
        action VARCHAR(50) NOT NULL,
        value FLOAT NOT NULL DEFAULT 0,
        updates INTEGER NOT NULL DEFAULT 0,
        last_updated TIMESTAMP DEFAULT NOW(),
        PRIMARY KEY (state, action)
      )
    `);

    await client.query(`
      CREATE TABLE IF NOT EXISTS experience_replay (
        id SERIAL PRIMARY KEY,
        state VARCHAR(200) NOT NULL,
        action VARCHAR(50) NOT NULL,
        reward FLOAT NOT NULL,
        next_state VARCHAR(200) NOT NULL,
        done BOOLEAN DEFAULT FALSE,
        context JSONB,
        created_at TIMESTAMP DEFAULT NOW()
      )
    `);

    await client.query(`
      CREATE INDEX IF NOT EXISTS idx_experience_time ON experience_replay(created_at DESC)
    `);

    await client.query(`
      CREATE TABLE IF NOT EXISTS reasoning_patterns (
        id VARCHAR(100) PRIMARY KEY,
        condition TEXT NOT NULL,
        action VARCHAR(50) NOT NULL,
        success_rate FLOAT DEFAULT 0.5,
        confidence FLOAT DEFAULT 0.5,
        sample_count INTEGER DEFAULT 0,
        embedding ruvector(384),
        last_updated TIMESTAMP DEFAULT NOW()
      )
    `);

    await client.query(`
      CREATE TABLE IF NOT EXISTS learning_metrics (
        id SERIAL PRIMARY KEY,
        episode INTEGER NOT NULL,
        avg_reward FLOAT,
        exploration_rate FLOAT,
        q_table_size INTEGER,
        pattern_count INTEGER,
        created_at TIMESTAMP DEFAULT NOW()
      )
    `);
  }

  /**
   * Load Q-table from database
   */
  private async loadQTable(client: any): Promise<void> {
    const result = await client.query(`
      SELECT state, action, value FROM q_table
    `);

    for (const row of result.rows) {
      if (!this.qTable.has(row.state)) {
        this.qTable.set(row.state, new Map());
      }
      this.qTable.get(row.state)!.set(row.action, row.value);
    }

    // Also load from recommendation_patterns for compatibility
    const patterns = await client.query(`
      SELECT pattern_type, approach, success_rate, total_uses
      FROM recommendation_patterns
    `);

    for (const row of patterns.rows) {
      if (!this.qTable.has(row.pattern_type)) {
        this.qTable.set(row.pattern_type, new Map());
      }
      // Map approach to action
      const action = this.mapApproachToAction(row.approach);
      this.qTable.get(row.pattern_type)!.set(action, row.success_rate * (row.total_uses || 1));
    }

    this.episode = this.qTable.size;
  }

  /**
   * Map legacy approach strings to actions
   */
  private mapApproachToAction(approach: string): string {
    const approachLower = approach.toLowerCase();
    if (approachLower.includes('genre')) return 'genre_weighted';
    if (approachLower.includes('similar')) return 'content_based';
    if (approachLower.includes('popular')) return 'popularity_boosted';
    if (approachLower.includes('trend')) return 'trending_focus';
    if (approachLower.includes('cold')) return 'popularity_boosted';
    return 'content_based';
  }

  /**
   * Load reasoning patterns
   */
  private async loadPatterns(client: any): Promise<void> {
    const result = await client.query(`
      SELECT id, condition, action, success_rate, confidence, sample_count, last_updated
      FROM reasoning_patterns
    `);

    for (const row of result.rows) {
      this.patterns.set(row.id, {
        id: row.id,
        condition: row.condition,
        action: row.action,
        successRate: parseFloat(row.success_rate),
        confidence: parseFloat(row.confidence),
        sampleCount: row.sample_count,
        lastUpdated: row.last_updated
      });
    }
  }

  /**
   * Load replay buffer from database
   */
  private async loadReplayBuffer(client: any): Promise<void> {
    const result = await client.query(`
      SELECT state, action, reward, next_state, done, context
      FROM experience_replay
      ORDER BY created_at DESC
      LIMIT $1
    `, [this.config.replayBufferSize]);

    this.replayBuffer = result.rows.map(row => ({
      state: row.state,
      action: row.action,
      reward: parseFloat(row.reward),
      nextState: row.next_state,
      done: row.done,
      context: row.context || {}
    }));
  }

  /**
   * Create state representation from context
   */
  createState(context: {
    genres?: string[];
    mood?: string;
    timeOfDay?: string;
    dayOfWeek?: string;
    userSegment?: string;
    contentType?: string;
  }): string {
    const parts: string[] = [];

    if (context.genres && context.genres.length > 0) {
      parts.push(context.genres.slice(0, 2).sort().join('_'));
    }
    if (context.mood) {
      parts.push(`mood:${context.mood}`);
    }
    if (context.userSegment) {
      parts.push(`seg:${context.userSegment}`);
    }
    if (context.contentType) {
      parts.push(`type:${context.contentType}`);
    }

    return parts.join('|') || 'default';
  }

  /**
   * Select action using epsilon-greedy with UCB exploration bonus
   */
  selectAction(state: string): { action: string; isExploration: boolean } {
    // Epsilon-greedy exploration
    if (Math.random() < this.config.explorationRate) {
      const action = this.ACTIONS[Math.floor(Math.random() * this.ACTIONS.length)];
      return { action, isExploration: true };
    }

    // Exploitation: select best action
    const stateQ = this.qTable.get(state);
    if (!stateQ || stateQ.size === 0) {
      // No knowledge for this state, explore
      const action = this.ACTIONS[Math.floor(Math.random() * this.ACTIONS.length)];
      return { action, isExploration: true };
    }

    // Find action with highest Q-value + UCB bonus
    let bestAction = this.ACTIONS[0];
    let bestValue = -Infinity;
    const totalVisits = Array.from(stateQ.values()).reduce((a, b) => a + Math.abs(b), 1);

    for (const [action, qValue] of stateQ) {
      // UCB exploration bonus
      const visits = Math.abs(qValue) + 1;
      const ucbBonus = Math.sqrt(2 * Math.log(totalVisits) / visits);
      const adjustedValue = qValue + 0.1 * ucbBonus;

      if (adjustedValue > bestValue) {
        bestValue = adjustedValue;
        bestAction = action;
      }
    }

    return { action: bestAction, isExploration: false };
  }

  /**
   * Update Q-value using Q-learning update rule
   */
  private updateQValue(state: string, action: string, reward: number, nextState: string, done: boolean): void {
    // Initialize state if needed
    if (!this.qTable.has(state)) {
      this.qTable.set(state, new Map());
    }

    const stateQ = this.qTable.get(state)!;
    const currentQ = stateQ.get(action) || 0;

    // Calculate target Q
    let targetQ: number;
    if (done) {
      targetQ = reward;
    } else {
      const nextStateQ = this.qTable.get(nextState);
      const maxNextQ = nextStateQ
        ? Math.max(...Array.from(nextStateQ.values()), 0)
        : 0;
      targetQ = reward + this.config.discountFactor * maxNextQ;
    }

    // Q-learning update
    const newQ = currentQ + this.config.learningRate * (targetQ - currentQ);
    stateQ.set(action, newQ);
  }

  /**
   * Record experience and learn from it
   */
  async learn(experience: Experience): Promise<void> {
    // Add to replay buffer
    this.replayBuffer.push(experience);
    if (this.replayBuffer.length > this.config.replayBufferSize) {
      this.replayBuffer.shift();
    }

    // Update Q-value for this experience
    this.updateQValue(
      experience.state,
      experience.action,
      experience.reward,
      experience.nextState,
      experience.done
    );

    // Experience replay: learn from random batch
    if (this.replayBuffer.length >= this.config.batchSize) {
      await this.experienceReplay();
    }

    // Decay exploration rate
    this.config.explorationRate = Math.max(
      this.config.minExplorationRate,
      this.config.explorationRate * this.config.explorationDecay
    );

    // Track reward
    this.rewardHistory.push(experience.reward);
    if (this.rewardHistory.length > 100) {
      this.rewardHistory.shift();
    }

    this.episode++;

    // Persist periodically
    if (this.episode % 50 === 0) {
      await this.persist();
    }

    // Update reasoning patterns
    await this.updateReasoningPattern(experience);

    logger.debug({
      episode: this.episode,
      state: experience.state,
      action: experience.action,
      reward: experience.reward,
      explorationRate: this.config.explorationRate
    }, 'Learned from experience');
  }

  /**
   * Learn from random batch of experiences (experience replay)
   */
  private async experienceReplay(): Promise<void> {
    const batch = this.sampleBatch(this.config.batchSize);

    for (const exp of batch) {
      this.updateQValue(exp.state, exp.action, exp.reward, exp.nextState, exp.done);
    }
  }

  /**
   * Sample random batch from replay buffer
   */
  private sampleBatch(size: number): Experience[] {
    const batch: Experience[] = [];
    const indices = new Set<number>();

    while (indices.size < Math.min(size, this.replayBuffer.length)) {
      indices.add(Math.floor(Math.random() * this.replayBuffer.length));
    }

    for (const idx of indices) {
      batch.push(this.replayBuffer[idx]);
    }

    return batch;
  }

  /**
   * Update reasoning patterns based on experience
   */
  private async updateReasoningPattern(experience: Experience): Promise<void> {
    const patternId = `${experience.state}:${experience.action}`;

    let pattern = this.patterns.get(patternId);
    if (!pattern) {
      pattern = {
        id: patternId,
        condition: experience.state,
        action: experience.action,
        successRate: 0.5,
        confidence: 0.5,
        sampleCount: 0,
        lastUpdated: new Date()
      };
    }

    // Bayesian update for success rate
    const success = experience.reward > 0 ? 1 : 0;
    pattern.sampleCount++;
    pattern.successRate = pattern.successRate + (success - pattern.successRate) / pattern.sampleCount;

    // Update confidence based on sample size
    pattern.confidence = Math.min(0.99, 1 - 1 / (pattern.sampleCount + 1));
    pattern.lastUpdated = new Date();

    this.patterns.set(patternId, pattern);
  }

  /**
   * Record user feedback for learning
   */
  async recordFeedback(
    contentId: string,
    action: string,
    feedback: 'watched' | 'completed' | 'rated' | 'skipped' | 'dismissed',
    rating?: number,
    context?: { genres?: string[]; mood?: string; userId?: string }
  ): Promise<void> {
    // Calculate reward based on feedback
    let reward: number;
    switch (feedback) {
      case 'completed':
        reward = 1.0;
        break;
      case 'watched':
        reward = 0.7;
        break;
      case 'rated':
        reward = rating ? (rating - 5) / 5 : 0.5; // Normalize to [-1, 1]
        break;
      case 'skipped':
        reward = -0.3;
        break;
      case 'dismissed':
        reward = -0.7;
        break;
      default:
        reward = 0;
    }

    // Create state from context
    const state = this.createState(context || {});

    // Create experience
    const experience: Experience = {
      state,
      action,
      reward,
      nextState: state, // Simplified - could track actual next state
      done: feedback === 'completed' || feedback === 'dismissed',
      context: {
        contentId,
        timestamp: Date.now(),
        ...context
      }
    };

    // Learn from this experience
    await this.learn(experience);

    // Store in database
    const client = await this.pool.connect();
    try {
      await client.query(`
        INSERT INTO learning_feedback (
          user_id, content_id, was_successful, reward, user_action
        ) VALUES ($1, $2, $3, $4, $5)
      `, [
        context?.userId || '00000000-0000-0000-0000-000000000000',
        contentId,
        reward > 0,
        reward,
        feedback
      ]);
    } finally {
      client.release();
    }

    logger.info({
      contentId,
      feedback,
      reward,
      action,
      state
    }, 'Recorded feedback');
  }

  /**
   * Get optimal action for given context
   */
  getOptimalAction(context: {
    genres?: string[];
    mood?: string;
    userSegment?: string;
    contentType?: string;
  }): { action: string; confidence: number; isExploration: boolean } {
    const state = this.createState(context);
    const { action, isExploration } = this.selectAction(state);

    // Calculate confidence
    const stateQ = this.qTable.get(state);
    let confidence = 0.5;
    if (stateQ && stateQ.has(action)) {
      const qValue = stateQ.get(action)!;
      const maxQ = Math.max(...Array.from(stateQ.values()), 1);
      confidence = Math.min(0.99, Math.abs(qValue) / maxQ);
    }

    return { action, confidence, isExploration };
  }

  /**
   * Get best patterns for given context
   */
  getBestPatterns(context: {
    genres?: string[];
    mood?: string;
  }, limit: number = 5): ReasoningPattern[] {
    const state = this.createState(context);
    const matchingPatterns: ReasoningPattern[] = [];

    for (const pattern of this.patterns.values()) {
      // Check if pattern condition matches state
      if (pattern.condition === state || state.includes(pattern.condition)) {
        matchingPatterns.push(pattern);
      }
    }

    // Sort by success rate * confidence
    return matchingPatterns
      .sort((a, b) => (b.successRate * b.confidence) - (a.successRate * a.confidence))
      .slice(0, limit);
  }

  /**
   * Persist learning state to database
   */
  async persist(): Promise<void> {
    const client = await this.pool.connect();
    try {
      await client.query('BEGIN');

      // Save Q-table
      for (const [state, actions] of this.qTable) {
        for (const [action, value] of actions) {
          await client.query(`
            INSERT INTO q_table (state, action, value, updates, last_updated)
            VALUES ($1, $2, $3, 1, NOW())
            ON CONFLICT (state, action) DO UPDATE SET
              value = $3,
              updates = q_table.updates + 1,
              last_updated = NOW()
          `, [state, action, value]);
        }
      }

      // Save reasoning patterns
      for (const pattern of this.patterns.values()) {
        await client.query(`
          INSERT INTO reasoning_patterns (id, condition, action, success_rate, confidence, sample_count, last_updated)
          VALUES ($1, $2, $3, $4, $5, $6, NOW())
          ON CONFLICT (id) DO UPDATE SET
            success_rate = $4,
            confidence = $5,
            sample_count = $6,
            last_updated = NOW()
        `, [pattern.id, pattern.condition, pattern.action, pattern.successRate, pattern.confidence, pattern.sampleCount]);
      }

      // Save metrics
      const avgReward = this.rewardHistory.length > 0
        ? this.rewardHistory.reduce((a, b) => a + b, 0) / this.rewardHistory.length
        : 0;

      await client.query(`
        INSERT INTO learning_metrics (episode, avg_reward, exploration_rate, q_table_size, pattern_count)
        VALUES ($1, $2, $3, $4, $5)
      `, [this.episode, avgReward, this.config.explorationRate, this.qTable.size, this.patterns.size]);

      // Save recent experiences
      for (const exp of this.replayBuffer.slice(-100)) {
        await client.query(`
          INSERT INTO experience_replay (state, action, reward, next_state, done, context)
          VALUES ($1, $2, $3, $4, $5, $6)
          ON CONFLICT DO NOTHING
        `, [exp.state, exp.action, exp.reward, exp.nextState, exp.done, JSON.stringify(exp.context)]);
      }

      await client.query('COMMIT');

      logger.info({
        qTableSize: this.qTable.size,
        patternCount: this.patterns.size,
        episode: this.episode
      }, 'Persisted learning state');

    } catch (error) {
      await client.query('ROLLBACK');
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Get learning statistics
   */
  async getStats(): Promise<LearningStats> {
    const avgReward = this.rewardHistory.length > 0
      ? this.rewardHistory.reduce((a, b) => a + b, 0) / this.rewardHistory.length
      : 0;

    const topPatterns = Array.from(this.patterns.values())
      .sort((a, b) => (b.successRate * b.confidence) - (a.successRate * a.confidence))
      .slice(0, 10);

    return {
      totalEpisodes: this.episode,
      avgReward,
      explorationRate: this.config.explorationRate,
      qTableSize: this.qTable.size,
      patternCount: this.patterns.size,
      topPatterns,
      recentPerformance: this.rewardHistory.slice(-20)
    };
  }

  /**
   * Run optimization cycle
   */
  async optimize(): Promise<{ improved: number; optimized: number }> {
    logger.info('Running optimization cycle');

    let improved = 0;
    let optimized = 0;

    // 1. Prune low-confidence patterns
    for (const [id, pattern] of this.patterns) {
      if (pattern.confidence < 0.3 && pattern.sampleCount < 10) {
        this.patterns.delete(id);
        optimized++;
      }
    }

    // 2. Experience replay optimization
    if (this.replayBuffer.length >= this.config.batchSize) {
      for (let i = 0; i < 5; i++) {
        await this.experienceReplay();
      }
      improved += 5;
    }

    // 3. Pattern consolidation (merge similar patterns)
    const patternGroups = new Map<string, ReasoningPattern[]>();
    for (const pattern of this.patterns.values()) {
      const key = pattern.action;
      if (!patternGroups.has(key)) {
        patternGroups.set(key, []);
      }
      patternGroups.get(key)!.push(pattern);
    }

    // 4. Update exploration rate based on performance
    if (this.rewardHistory.length > 20) {
      const recentAvg = this.rewardHistory.slice(-20).reduce((a, b) => a + b, 0) / 20;
      const olderAvg = this.rewardHistory.slice(-40, -20).reduce((a, b) => a + b, 0) / 20;

      if (recentAvg > olderAvg) {
        // Performance improving, reduce exploration
        this.config.explorationRate *= 0.95;
      } else {
        // Performance declining, increase exploration
        this.config.explorationRate = Math.min(0.5, this.config.explorationRate * 1.1);
      }
      optimized++;
    }

    await this.persist();

    logger.info({ improved, optimized }, 'Optimization cycle complete');
    return { improved, optimized };
  }

  async close(): Promise<void> {
    await this.persist();
    await this.pool.end();
  }
}

// Factory function
export function createSelfLearningEngine(config?: Partial<LearningConfig>): SelfLearningEngine {
  return new SelfLearningEngine(config);
}

export default SelfLearningEngine;
