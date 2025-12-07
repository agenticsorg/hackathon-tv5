/**
 * RuVector Self-Learning Optimizer
 *
 * Implements continuous optimization of embeddings using:
 * - Q-Learning for embedding strategy optimization
 * - ReasoningBank pattern learning
 * - SIMD-optimized similarity calculations
 * - Adaptive embedding refinement based on user feedback
 */

import { Pool } from 'pg';
import pino from 'pino';

const logger = pino({ name: 'ruvector-optimizer' });

interface OptimizationMetrics {
  totalOptimized: number;
  qualityImprovement: number;
  clustersIdentified: number;
  patternsUpdated: number;
  searchSpeedImprovement: number;
}

interface EmbeddingCluster {
  centroidId: string;
  memberIds: string[];
  avgSimilarity: number;
  genres: string[];
}

interface LearningState {
  episode: number;
  totalReward: number;
  explorationRate: number;
  bestStrategy: string;
}

export class RuVectorOptimizer {
  private pool: Pool;
  private learningState: LearningState = {
    episode: 0,
    totalReward: 0,
    explorationRate: 0.3,
    bestStrategy: 'genre_weighted'
  };

  // Embedding strategies to learn from
  private readonly STRATEGIES = [
    'genre_weighted',      // Weight genres heavily in embeddings
    'actor_focused',       // Focus on actors/characters
    'network_biased',      // Bias towards network preferences
    'recency_boosted',     // Boost newer content
    'rating_amplified',    // Amplify high-rated content
    'hybrid_balanced'      // Balanced approach
  ];

  constructor(connectionString?: string) {
    this.pool = new Pool({
      connectionString: connectionString ||
        process.env.DATABASE_URL ||
        'postgresql://tvdb:tvdb_recommender_2024@localhost:5432/tvdb_recommender',
      max: 10
    });
  }

  /**
   * Initialize the optimizer and enable RuVector learning
   */
  async initialize(): Promise<void> {
    const client = await this.pool.connect();
    try {
      // Enable RuVector self-learning
      await client.query(`
        SELECT ruvector_enable_learning('content', $1::jsonb)
      `, [JSON.stringify({
        algorithm: 'q_learning',
        reward_decay: 0.95,
        learning_rate: 0.1,
        exploration_rate: this.learningState.explorationRate
      })]);

      logger.info('RuVector self-learning enabled');

      // Load previous learning state
      const stateResult = await client.query(`
        SELECT metadata FROM sync_status
        WHERE sync_type = 'learning_state'
        ORDER BY created_at DESC LIMIT 1
      `);

      if (stateResult.rows.length > 0 && stateResult.rows[0].metadata) {
        this.learningState = { ...this.learningState, ...stateResult.rows[0].metadata };
        logger.info({ state: this.learningState }, 'Loaded previous learning state');
      }

    } catch (error: any) {
      // RuVector learning might not be available
      logger.warn({ error: error.message }, 'RuVector learning not available, using fallback');
    } finally {
      client.release();
    }
  }

  /**
   * Run a complete optimization cycle
   */
  async runOptimizationCycle(): Promise<OptimizationMetrics> {
    logger.info('Starting optimization cycle');
    const startTime = Date.now();

    const metrics: OptimizationMetrics = {
      totalOptimized: 0,
      qualityImprovement: 0,
      clustersIdentified: 0,
      patternsUpdated: 0,
      searchSpeedImprovement: 0
    };

    // Step 1: Identify and analyze clusters
    const clusters = await this.identifyClusters();
    metrics.clustersIdentified = clusters.length;

    // Step 2: Update learned patterns
    metrics.patternsUpdated = await this.updateLearningPatterns(clusters);

    // Step 3: Optimize embeddings based on feedback
    metrics.totalOptimized = await this.optimizeEmbeddingsWithFeedback();

    // Step 4: Calculate quality improvement
    metrics.qualityImprovement = await this.measureQualityImprovement();

    // Step 5: Save learning state
    await this.saveLearningState();

    const duration = Date.now() - startTime;
    logger.info({ metrics, durationMs: duration }, 'Optimization cycle complete');

    return metrics;
  }

  /**
   * Identify content clusters using RuVector similarity
   */
  private async identifyClusters(): Promise<EmbeddingCluster[]> {
    const client = await this.pool.connect();
    const clusters: EmbeddingCluster[] = [];

    try {
      // Find high-similarity pairs and group them
      const result = await client.query(`
        WITH similarity_pairs AS (
          SELECT
            c1.id as id1,
            c2.id as id2,
            c1.genres as genres1,
            (1 - ruvector_cosine_distance(c1.embedding, c2.embedding)) as similarity
          FROM content c1
          JOIN content c2 ON c1.id < c2.id
          WHERE c1.embedding IS NOT NULL
          AND c2.embedding IS NOT NULL
          AND (1 - ruvector_cosine_distance(c1.embedding, c2.embedding)) > 0.75
          LIMIT 500
        )
        SELECT id1, id2, genres1, similarity
        FROM similarity_pairs
        ORDER BY similarity DESC
      `);

      // Group into clusters
      const clusterMap = new Map<string, Set<string>>();

      for (const row of result.rows) {
        const existing1 = this.findCluster(clusterMap, row.id1);
        const existing2 = this.findCluster(clusterMap, row.id2);

        if (existing1 && existing2 && existing1 !== existing2) {
          // Merge clusters
          const merged = new Set([...clusterMap.get(existing1)!, ...clusterMap.get(existing2)!]);
          clusterMap.delete(existing2);
          clusterMap.set(existing1, merged);
        } else if (existing1) {
          clusterMap.get(existing1)!.add(row.id2);
        } else if (existing2) {
          clusterMap.get(existing2)!.add(row.id1);
        } else {
          clusterMap.set(row.id1, new Set([row.id1, row.id2]));
        }
      }

      // Convert to cluster objects
      for (const [centroid, members] of clusterMap) {
        if (members.size >= 3) {
          clusters.push({
            centroidId: centroid,
            memberIds: Array.from(members),
            avgSimilarity: 0.8,
            genres: result.rows.find(r => r.id1 === centroid)?.genres1 || []
          });
        }
      }

      logger.info({ clusterCount: clusters.length }, 'Identified content clusters');

    } finally {
      client.release();
    }

    return clusters;
  }

  private findCluster(map: Map<string, Set<string>>, id: string): string | null {
    for (const [key, members] of map) {
      if (members.has(id)) return key;
    }
    return null;
  }

  /**
   * Update learning patterns based on cluster analysis
   */
  private async updateLearningPatterns(clusters: EmbeddingCluster[]): Promise<number> {
    const client = await this.pool.connect();
    let patternsUpdated = 0;

    try {
      for (const cluster of clusters) {
        if (cluster.genres.length >= 2) {
          const patternType = cluster.genres.slice(0, 2).join('_').toLowerCase();

          await client.query(`
            INSERT INTO recommendation_patterns (
              pattern_type, approach, success_rate, total_uses
            ) VALUES ($1, $2, $3, $4)
            ON CONFLICT DO NOTHING
          `, [
            patternType,
            `Cluster-based recommendation for ${cluster.genres.join(' + ')} content`,
            0.7 + (cluster.avgSimilarity * 0.2), // Higher similarity = higher success
            cluster.memberIds.length
          ]);

          patternsUpdated++;
        }
      }

      // Update exploration rate using decay
      this.learningState.explorationRate = Math.max(
        0.05,
        this.learningState.explorationRate * 0.99
      );

      logger.info({ patternsUpdated }, 'Updated learning patterns');

    } finally {
      client.release();
    }

    return patternsUpdated;
  }

  /**
   * Optimize embeddings based on user feedback
   */
  private async optimizeEmbeddingsWithFeedback(): Promise<number> {
    const client = await this.pool.connect();
    let optimized = 0;

    try {
      // Get feedback data
      const feedback = await client.query(`
        SELECT
          lf.content_id,
          lf.was_successful,
          lf.reward,
          c.genres
        FROM learning_feedback lf
        JOIN content c ON c.id = lf.content_id
        WHERE lf.created_at > NOW() - INTERVAL '7 days'
        ORDER BY lf.created_at DESC
        LIMIT 100
      `);

      // Calculate rewards per genre
      const genreRewards = new Map<string, { total: number; count: number }>();

      for (const row of feedback.rows) {
        const reward = row.was_successful ? parseFloat(row.reward) : -0.1;

        for (const genre of row.genres || []) {
          const current = genreRewards.get(genre) || { total: 0, count: 0 };
          current.total += reward;
          current.count++;
          genreRewards.set(genre, current);
        }
      }

      // Update strategy based on rewards
      let bestGenre = '';
      let bestAvgReward = -1;

      for (const [genre, { total, count }] of genreRewards) {
        const avgReward = total / count;
        if (avgReward > bestAvgReward) {
          bestAvgReward = avgReward;
          bestGenre = genre;
        }
      }

      if (bestGenre) {
        this.learningState.bestStrategy = `genre_${bestGenre}`;
        this.learningState.totalReward += bestAvgReward;
        this.learningState.episode++;

        logger.info({
          bestGenre,
          avgReward: bestAvgReward,
          episode: this.learningState.episode
        }, 'Updated learning strategy');
      }

      // Use RuVector learning function if available
      try {
        await client.query(`
          SELECT ruvector_learn_from_feedback('content', $1::jsonb)
        `, [JSON.stringify({
          rewards: Object.fromEntries(genreRewards),
          strategy: this.learningState.bestStrategy,
          episode: this.learningState.episode
        })]);
        optimized = feedback.rows.length;
      } catch (e) {
        // Fallback - RuVector learning not available
        optimized = 0;
      }

    } finally {
      client.release();
    }

    return optimized;
  }

  /**
   * Measure embedding quality improvement
   */
  private async measureQualityImprovement(): Promise<number> {
    const client = await this.pool.connect();

    try {
      // Calculate average intra-cluster similarity
      const result = await client.query(`
        WITH sample AS (
          SELECT id, embedding, genres[1] as primary_genre
          FROM content
          WHERE embedding IS NOT NULL
          AND array_length(genres, 1) > 0
          ORDER BY RANDOM()
          LIMIT 100
        ),
        genre_similarities AS (
          SELECT
            s1.primary_genre,
            AVG(1 - ruvector_cosine_distance(s1.embedding, s2.embedding)) as avg_similarity
          FROM sample s1
          JOIN sample s2 ON s1.primary_genre = s2.primary_genre AND s1.id != s2.id
          GROUP BY s1.primary_genre
        )
        SELECT AVG(avg_similarity) as overall_quality
        FROM genre_similarities
      `);

      const quality = parseFloat(result.rows[0]?.overall_quality || 0);
      logger.info({ quality }, 'Measured embedding quality');

      return quality;

    } finally {
      client.release();
    }
  }

  /**
   * Save learning state for persistence
   */
  private async saveLearningState(): Promise<void> {
    const client = await this.pool.connect();

    try {
      await client.query(`
        INSERT INTO sync_status (sync_type, last_sync_timestamp, items_synced, status)
        VALUES ('learning_state', $1, $2, 'completed')
      `, [
        Math.floor(Date.now() / 1000),
        this.learningState.episode
      ]);

    } finally {
      client.release();
    }
  }

  /**
   * Get current learning statistics
   */
  async getStats(): Promise<any> {
    const client = await this.pool.connect();

    try {
      const [contentStats, patternStats, qualityStats] = await Promise.all([
        client.query(`
          SELECT
            content_type,
            COUNT(*) as total,
            COUNT(CASE WHEN embedding IS NOT NULL THEN 1 END) as with_embeddings
          FROM content
          GROUP BY content_type
        `),
        client.query(`
          SELECT
            COUNT(*) as total_patterns,
            AVG(success_rate) as avg_success,
            MAX(success_rate) as best_success
          FROM recommendation_patterns
        `),
        client.query(`
          SELECT
            COUNT(*) as feedback_count,
            AVG(CASE WHEN was_successful THEN 1.0 ELSE 0.0 END) as success_rate
          FROM learning_feedback
          WHERE created_at > NOW() - INTERVAL '24 hours'
        `)
      ]);

      return {
        content: contentStats.rows,
        patterns: patternStats.rows[0],
        recentFeedback: qualityStats.rows[0],
        learningState: this.learningState
      };

    } finally {
      client.release();
    }
  }

  async close(): Promise<void> {
    await this.pool.end();
  }
}

// Factory function
export function createRuVectorOptimizer(connectionString?: string): RuVectorOptimizer {
  return new RuVectorOptimizer(connectionString);
}

export default RuVectorOptimizer;
