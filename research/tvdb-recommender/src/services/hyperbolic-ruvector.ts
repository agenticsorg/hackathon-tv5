/**
 * Hyperbolic RuVector Self-Learning Service
 *
 * Implements advanced RuVector capabilities:
 * - Hyperbolic embeddings for hierarchical content relationships
 * - Poincaré ball model for genre hierarchies
 * - Q-Learning with experience replay
 * - Self-optimizing embedding strategies
 * - Continuous learning from user interactions
 */

import { Pool } from 'pg';
import pino from 'pino';

const logger = pino({ name: 'hyperbolic-ruvector' });

interface HyperbolicConfig {
  curvature: number;      // Negative curvature for hyperbolic space (-1 to -0.1)
  dimension: number;      // Embedding dimension
  learningRate: number;   // Q-learning rate
  discountFactor: number; // Future reward discount
  explorationRate: number;// Epsilon for exploration
  burnIn: number;         // Episodes before exploitation
}

interface LearningEpisode {
  state: string;          // Content features as state
  action: string;         // Embedding strategy used
  reward: number;         // User engagement reward
  nextState: string;      // Resulting state
  done: boolean;
}

interface ContentHierarchy {
  id: string;
  level: number;          // 0=root, 1=genre, 2=subgenre, 3=content
  parent: string | null;
  embedding: Float32Array;
  hyperbolicEmbedding?: Float32Array; // Poincaré ball embedding
}

export class HyperbolicRuVectorService {
  private pool: Pool;
  private config: HyperbolicConfig;
  private replayBuffer: LearningEpisode[] = [];
  private qTable: Map<string, Map<string, number>> = new Map();
  private episode: number = 0;

  // Embedding strategies to learn
  private readonly ACTIONS = [
    'euclidean_standard',
    'hyperbolic_hierarchy',
    'hybrid_adaptive',
    'genre_weighted',
    'temporal_decay',
    'popularity_boosted'
  ];

  constructor(config?: Partial<HyperbolicConfig>) {
    this.config = {
      curvature: config?.curvature || -1.0,
      dimension: config?.dimension || 384,
      learningRate: config?.learningRate || 0.1,
      discountFactor: config?.discountFactor || 0.95,
      explorationRate: config?.explorationRate || 0.3,
      burnIn: config?.burnIn || 100
    };

    this.pool = new Pool({
      connectionString: process.env.DATABASE_URL ||
        'postgresql://tvdb:tvdb_recommender_2024@localhost:5432/tvdb_recommender',
      max: 10
    });
  }

  /**
   * Initialize hyperbolic space and RuVector learning
   */
  async initialize(): Promise<void> {
    const client = await this.pool.connect();
    try {
      // Enable RuVector hyperbolic mode
      await this.enableHyperbolicMode(client);

      // Load existing Q-table from database
      await this.loadQTable(client);

      // Initialize experience replay buffer
      await this.loadReplayBuffer(client);

      logger.info({
        curvature: this.config.curvature,
        dimension: this.config.dimension,
        episode: this.episode
      }, 'Hyperbolic RuVector initialized');

    } finally {
      client.release();
    }
  }

  /**
   * Enable hyperbolic embedding mode in RuVector
   */
  private async enableHyperbolicMode(client: any): Promise<void> {
    try {
      // Configure RuVector for hyperbolic space
      await client.query(`
        SELECT ruvector_configure($1::jsonb)
      `, [JSON.stringify({
        mode: 'hyperbolic',
        curvature: this.config.curvature,
        distance_metric: 'poincare',
        learning: {
          enabled: true,
          algorithm: 'q_learning_er', // Q-learning with experience replay
          learning_rate: this.config.learningRate,
          discount_factor: this.config.discountFactor,
          replay_buffer_size: 10000,
          batch_size: 32,
          target_update_frequency: 100
        },
        optimization: {
          simd_enabled: true,
          cache_enabled: true,
          index_type: 'hnsw_hyperbolic'
        }
      })]);

      logger.info('RuVector hyperbolic mode enabled');
    } catch (error: any) {
      // Fallback if hyperbolic mode not available
      logger.warn({ error: error.message }, 'Hyperbolic mode not available, using euclidean fallback');

      // Use standard learning mode
      await client.query(`
        SELECT ruvector_enable_learning('content', $1::jsonb)
      `, [JSON.stringify({
        algorithm: 'q_learning',
        learning_rate: this.config.learningRate,
        reward_decay: this.config.discountFactor
      })]);
    }
  }

  /**
   * Transform Euclidean embedding to Poincaré ball (hyperbolic space)
   */
  euclideanToPoincare(euclidean: Float32Array): Float32Array {
    const K = Math.abs(this.config.curvature);
    const sqrtK = Math.sqrt(K);

    // Calculate Euclidean norm
    let norm = 0;
    for (let i = 0; i < euclidean.length; i++) {
      norm += euclidean[i] * euclidean[i];
    }
    norm = Math.sqrt(norm);

    // Exponential map from tangent space to Poincaré ball
    // x_poincare = tanh(sqrt(K) * ||v||) * v / (sqrt(K) * ||v||)
    if (norm < 1e-10) {
      return new Float32Array(euclidean.length);
    }

    const scale = Math.tanh(sqrtK * norm) / (sqrtK * norm);
    const poincare = new Float32Array(euclidean.length);

    for (let i = 0; i < euclidean.length; i++) {
      poincare[i] = scale * euclidean[i];
    }

    return poincare;
  }

  /**
   * Calculate Poincaré distance between two hyperbolic embeddings
   */
  poincareDistance(u: Float32Array, v: Float32Array): number {
    const K = Math.abs(this.config.curvature);

    // ||u - v||^2
    let diffNormSq = 0;
    for (let i = 0; i < u.length; i++) {
      const diff = u[i] - v[i];
      diffNormSq += diff * diff;
    }

    // ||u||^2
    let uNormSq = 0;
    for (let i = 0; i < u.length; i++) {
      uNormSq += u[i] * u[i];
    }

    // ||v||^2
    let vNormSq = 0;
    for (let i = 0; i < v.length; i++) {
      vNormSq += v[i] * v[i];
    }

    // Poincaré distance formula
    const numerator = diffNormSq;
    const denominator = (1 - uNormSq) * (1 - vNormSq);

    if (denominator <= 0) {
      return Infinity;
    }

    const delta = 1 + 2 * K * numerator / denominator;
    return (1 / Math.sqrt(K)) * Math.acosh(Math.max(1, delta));
  }

  /**
   * Select action using epsilon-greedy policy
   */
  selectAction(state: string): string {
    // Exploration: random action
    if (this.episode < this.config.burnIn || Math.random() < this.config.explorationRate) {
      return this.ACTIONS[Math.floor(Math.random() * this.ACTIONS.length)];
    }

    // Exploitation: best known action
    const stateQ = this.qTable.get(state);
    if (!stateQ || stateQ.size === 0) {
      return this.ACTIONS[0]; // Default action
    }

    let bestAction = this.ACTIONS[0];
    let bestValue = -Infinity;

    for (const [action, value] of stateQ) {
      if (value > bestValue) {
        bestValue = value;
        bestAction = action;
      }
    }

    return bestAction;
  }

  /**
   * Update Q-value using Q-learning update rule
   */
  updateQ(episode: LearningEpisode): void {
    const { state, action, reward, nextState, done } = episode;

    // Initialize state if not exists
    if (!this.qTable.has(state)) {
      this.qTable.set(state, new Map());
    }

    const stateQ = this.qTable.get(state)!;
    const currentQ = stateQ.get(action) || 0;

    // Calculate target Q-value
    let targetQ: number;
    if (done) {
      targetQ = reward;
    } else {
      const nextStateQ = this.qTable.get(nextState);
      const maxNextQ = nextStateQ ? Math.max(...nextStateQ.values(), 0) : 0;
      targetQ = reward + this.config.discountFactor * maxNextQ;
    }

    // Q-learning update
    const newQ = currentQ + this.config.learningRate * (targetQ - currentQ);
    stateQ.set(action, newQ);

    // Decay exploration rate
    this.config.explorationRate = Math.max(0.05, this.config.explorationRate * 0.999);
    this.episode++;
  }

  /**
   * Store episode in replay buffer and learn
   */
  async learn(episode: LearningEpisode): Promise<void> {
    // Add to replay buffer
    this.replayBuffer.push(episode);
    if (this.replayBuffer.length > 10000) {
      this.replayBuffer.shift(); // Remove oldest
    }

    // Update Q-value for this episode
    this.updateQ(episode);

    // Experience replay: sample random batch and learn
    if (this.replayBuffer.length >= 32) {
      const batch = this.sampleBatch(32);
      for (const ep of batch) {
        this.updateQ(ep);
      }
    }

    // Persist learning state periodically
    if (this.episode % 100 === 0) {
      await this.persistLearningState();
    }
  }

  /**
   * Sample random batch from replay buffer
   */
  private sampleBatch(size: number): LearningEpisode[] {
    const batch: LearningEpisode[] = [];
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
   * Generate optimized embedding using learned strategy
   */
  async generateOptimizedEmbedding(
    content: { title: string; genres: string[]; overview: string; year?: number },
    baseEmbedding: Float32Array
  ): Promise<{ embedding: Float32Array; strategy: string }> {
    // Create state representation
    const state = this.createState(content);

    // Select action based on learned policy
    const action = this.selectAction(state);

    // Apply selected embedding strategy
    let embedding: Float32Array;

    switch (action) {
      case 'hyperbolic_hierarchy':
        embedding = this.applyHyperbolicTransform(baseEmbedding, content);
        break;
      case 'genre_weighted':
        embedding = this.applyGenreWeighting(baseEmbedding, content.genres);
        break;
      case 'temporal_decay':
        embedding = this.applyTemporalDecay(baseEmbedding, content.year);
        break;
      case 'popularity_boosted':
        embedding = this.applyPopularityBoost(baseEmbedding);
        break;
      case 'hybrid_adaptive':
        embedding = this.applyHybridStrategy(baseEmbedding, content);
        break;
      default:
        embedding = baseEmbedding;
    }

    return { embedding, strategy: action };
  }

  /**
   * Create state representation from content features
   */
  private createState(content: { genres: string[]; year?: number }): string {
    const genreKey = content.genres.slice(0, 2).sort().join('_') || 'unknown';
    const decade = content.year ? Math.floor(content.year / 10) * 10 : 2020;
    return `${genreKey}_${decade}`;
  }

  /**
   * Apply hyperbolic transformation for hierarchical relationships
   */
  private applyHyperbolicTransform(
    embedding: Float32Array,
    content: { genres: string[] }
  ): Float32Array {
    // Transform to Poincaré ball
    const hyperbolic = this.euclideanToPoincare(embedding);

    // Apply hierarchy-aware scaling based on genre depth
    const genreDepth = content.genres.length;
    const scale = 1 / (1 + 0.1 * genreDepth); // Closer to origin = more general

    const result = new Float32Array(hyperbolic.length);
    for (let i = 0; i < hyperbolic.length; i++) {
      result[i] = hyperbolic[i] * scale;
    }

    return result;
  }

  /**
   * Apply genre-based weighting
   */
  private applyGenreWeighting(embedding: Float32Array, genres: string[]): Float32Array {
    const result = new Float32Array(embedding.length);

    // Hash genres to consistent dimension indices
    const genreIndices = genres.map(g => Math.abs(this.hashString(g)) % embedding.length);

    for (let i = 0; i < embedding.length; i++) {
      if (genreIndices.includes(i)) {
        result[i] = embedding[i] * 1.5; // Boost genre-related dimensions
      } else {
        result[i] = embedding[i];
      }
    }

    return this.normalize(result);
  }

  /**
   * Apply temporal decay for recency
   */
  private applyTemporalDecay(embedding: Float32Array, year?: number): Float32Array {
    const currentYear = new Date().getFullYear();
    const contentYear = year || currentYear;
    const age = currentYear - contentYear;
    const decay = Math.exp(-0.05 * age); // Exponential decay

    const result = new Float32Array(embedding.length);
    for (let i = 0; i < embedding.length; i++) {
      result[i] = embedding[i] * (0.5 + 0.5 * decay);
    }

    return result;
  }

  /**
   * Apply popularity boost
   */
  private applyPopularityBoost(embedding: Float32Array): Float32Array {
    // Amplify magnitude slightly
    const result = new Float32Array(embedding.length);
    for (let i = 0; i < embedding.length; i++) {
      result[i] = embedding[i] * 1.1;
    }
    return this.normalize(result);
  }

  /**
   * Apply hybrid adaptive strategy
   */
  private applyHybridStrategy(
    embedding: Float32Array,
    content: { genres: string[]; year?: number }
  ): Float32Array {
    // Combine multiple strategies
    const hyperbolic = this.applyHyperbolicTransform(embedding, content);
    const genreWeighted = this.applyGenreWeighting(embedding, content.genres);
    const temporal = this.applyTemporalDecay(embedding, content.year);

    // Average the strategies
    const result = new Float32Array(embedding.length);
    for (let i = 0; i < embedding.length; i++) {
      result[i] = (hyperbolic[i] + genreWeighted[i] + temporal[i]) / 3;
    }

    return this.normalize(result);
  }

  /**
   * Normalize vector to unit length
   */
  private normalize(vector: Float32Array): Float32Array {
    let norm = 0;
    for (let i = 0; i < vector.length; i++) {
      norm += vector[i] * vector[i];
    }
    norm = Math.sqrt(norm);

    if (norm < 1e-10) return vector;

    const result = new Float32Array(vector.length);
    for (let i = 0; i < vector.length; i++) {
      result[i] = vector[i] / norm;
    }
    return result;
  }

  /**
   * Simple string hash function
   */
  private hashString(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      hash = ((hash << 5) - hash) + str.charCodeAt(i);
      hash = hash & hash;
    }
    return hash;
  }

  /**
   * Record user feedback for learning
   */
  async recordFeedback(
    contentId: string,
    strategy: string,
    engagement: 'watched' | 'clicked' | 'skipped' | 'rated',
    rating?: number
  ): Promise<void> {
    const client = await this.pool.connect();

    try {
      // Get content features for state
      const contentResult = await client.query(`
        SELECT genres, year FROM content WHERE id = $1
      `, [contentId]);

      if (contentResult.rows.length === 0) return;

      const content = contentResult.rows[0];
      const state = this.createState({
        genres: content.genres || [],
        year: content.year
      });

      // Calculate reward based on engagement
      let reward: number;
      switch (engagement) {
        case 'watched':
          reward = 1.0;
          break;
        case 'rated':
          reward = rating ? (rating - 5) / 5 : 0.5; // Normalize to [-1, 1]
          break;
        case 'clicked':
          reward = 0.3;
          break;
        case 'skipped':
          reward = -0.5;
          break;
        default:
          reward = 0;
      }

      // Create next state (could be based on what user did next)
      const nextState = state; // Simplified

      // Learn from this episode
      await this.learn({
        state,
        action: strategy,
        reward,
        nextState,
        done: engagement === 'watched' || engagement === 'rated'
      });

      // Store in database
      await client.query(`
        INSERT INTO learning_feedback (
          user_id, content_id, pattern_id, was_successful, reward, user_action
        ) VALUES (
          '00000000-0000-0000-0000-000000000000',
          $1, NULL, $2, $3, $4
        )
      `, [contentId, reward > 0, reward, engagement]);

      logger.info({
        contentId,
        strategy,
        engagement,
        reward,
        episode: this.episode
      }, 'Recorded feedback and learned');

    } finally {
      client.release();
    }
  }

  /**
   * Load Q-table from database
   */
  private async loadQTable(client: any): Promise<void> {
    try {
      const result = await client.query(`
        SELECT pattern_type, approach, success_rate, total_uses
        FROM recommendation_patterns
      `);

      for (const row of result.rows) {
        const state = row.pattern_type;
        const action = row.approach.includes('hyperbolic') ? 'hyperbolic_hierarchy' :
                      row.approach.includes('genre') ? 'genre_weighted' : 'euclidean_standard';

        if (!this.qTable.has(state)) {
          this.qTable.set(state, new Map());
        }

        this.qTable.get(state)!.set(action, row.success_rate * row.total_uses);
      }

      logger.info({ states: this.qTable.size }, 'Loaded Q-table');
    } catch (error) {
      logger.warn('Could not load Q-table');
    }
  }

  /**
   * Load replay buffer from database
   */
  private async loadReplayBuffer(client: any): Promise<void> {
    try {
      const result = await client.query(`
        SELECT
          c.genres,
          c.year,
          lf.reward,
          lf.user_action
        FROM learning_feedback lf
        JOIN content c ON c.id = lf.content_id
        ORDER BY lf.created_at DESC
        LIMIT 1000
      `);

      for (const row of result.rows) {
        const state = this.createState({
          genres: row.genres || [],
          year: row.year
        });

        this.replayBuffer.push({
          state,
          action: 'euclidean_standard',
          reward: parseFloat(row.reward),
          nextState: state,
          done: true
        });
      }

      this.episode = this.replayBuffer.length;
      logger.info({ episodes: this.episode }, 'Loaded replay buffer');
    } catch (error) {
      logger.warn('Could not load replay buffer');
    }
  }

  /**
   * Persist learning state to database
   */
  private async persistLearningState(): Promise<void> {
    const client = await this.pool.connect();
    try {
      // Update patterns with learned Q-values
      for (const [state, actions] of this.qTable) {
        for (const [action, value] of actions) {
          await client.query(`
            INSERT INTO recommendation_patterns (pattern_type, approach, success_rate, total_uses)
            VALUES ($1, $2, $3, 1)
            ON CONFLICT DO NOTHING
          `, [state, `${action} strategy`, Math.min(1, Math.max(0, value / 100))]);
        }
      }

      await client.query(`
        INSERT INTO sync_status (sync_type, last_sync_timestamp, items_synced, status)
        VALUES ('hyperbolic_learning', $1, $2, 'completed')
      `, [Math.floor(Date.now() / 1000), this.episode]);

      logger.info({ episode: this.episode }, 'Persisted learning state');
    } finally {
      client.release();
    }
  }

  /**
   * Get learning statistics
   */
  async getStats(): Promise<{
    episode: number;
    explorationRate: number;
    qTableSize: number;
    replayBufferSize: number;
    bestStrategies: Array<{ state: string; action: string; value: number }>;
  }> {
    const bestStrategies: Array<{ state: string; action: string; value: number }> = [];

    for (const [state, actions] of this.qTable) {
      let bestAction = '';
      let bestValue = -Infinity;

      for (const [action, value] of actions) {
        if (value > bestValue) {
          bestValue = value;
          bestAction = action;
        }
      }

      if (bestAction) {
        bestStrategies.push({ state, action: bestAction, value: bestValue });
      }
    }

    bestStrategies.sort((a, b) => b.value - a.value);

    return {
      episode: this.episode,
      explorationRate: this.config.explorationRate,
      qTableSize: this.qTable.size,
      replayBufferSize: this.replayBuffer.length,
      bestStrategies: bestStrategies.slice(0, 10)
    };
  }

  async close(): Promise<void> {
    await this.persistLearningState();
    await this.pool.end();
  }
}

// Factory function
export function createHyperbolicRuVectorService(config?: Partial<HyperbolicConfig>): HyperbolicRuVectorService {
  return new HyperbolicRuVectorService(config);
}

export default HyperbolicRuVectorService;
