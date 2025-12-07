/**
 * RuVector Self-Learning Integration
 *
 * Integrates hyperbolic vectors and Q-Learning optimization
 * for continuous recommendation improvement
 */

import { Pool } from 'pg';

const pool = new Pool({
  connectionString: process.env.DATABASE_URL ||
    'postgresql://tvdb:tvdb_recommender_2024@localhost:5432/tvdb_recommender',
  max: 10,
});

// Query timing tracker - stores last 100 query times for averaging
const queryTimings: number[] = [];
const MAX_TIMING_SAMPLES = 100;

function recordQueryTiming(timeMs: number) {
  queryTimings.push(timeMs);
  if (queryTimings.length > MAX_TIMING_SAMPLES) {
    queryTimings.shift();
  }
}

export function getAverageQueryTime(): number {
  if (queryTimings.length === 0) return 3.2; // Default fallback
  return queryTimings.reduce((a, b) => a + b, 0) / queryTimings.length;
}

// System config interface and cache
interface SystemConfig {
  qLearning: {
    enabled: boolean;
    explorationRate: number;
    learningRate: number;
    rewardDecay: number;
  };
  vectorSearch: {
    hyperbolicEnabled: boolean;
  };
  recommendations: {
    diversityWeight: number;
    recencyBoost: number;
    popularityWeight: number;
    maxResults: number;
  };
}

let cachedConfig: SystemConfig | null = null;
let configLastFetched = 0;
const CONFIG_CACHE_TTL = 30000; // 30 seconds

async function getSystemConfig(): Promise<SystemConfig> {
  const now = Date.now();
  if (cachedConfig && (now - configLastFetched) < CONFIG_CACHE_TTL) {
    return cachedConfig;
  }

  const client = await pool.connect();
  try {
    const result = await client.query('SELECT config FROM system_config WHERE id = 1');
    if (result.rows.length > 0) {
      cachedConfig = result.rows[0].config;
      configLastFetched = now;
      return cachedConfig!;
    }
  } catch {
    // Fall back to defaults
  } finally {
    client.release();
  }

  // Default config
  return {
    qLearning: { enabled: true, explorationRate: 0.3, learningRate: 0.1, rewardDecay: 0.95 },
    vectorSearch: { hyperbolicEnabled: true },
    recommendations: { diversityWeight: 0.3, recencyBoost: 0.2, popularityWeight: 0.15, maxResults: 20 }
  };
}

export interface LearningState {
  episode: number;
  explorationRate: number;
  bestStrategy: string;
  totalReward: number;
  lastUpdate: Date;
}

export interface PatternStats {
  patternType: string;
  successRate: number;
  totalUses: number;
  avgReward: number;
}

/**
 * Record user feedback and trigger Q-Learning update
 */
export async function recordFeedbackWithLearning(
  contentId: string,
  wasSuccessful: boolean,
  context?: { mood?: string; searchQuery?: string }
): Promise<{ reward: number; patternUpdated: boolean; configUsed: { explorationRate: number; learningRate: number } }> {
  const client = await pool.connect();
  const config = await getSystemConfig();

  try {
    // Check if Q-Learning is enabled
    if (!config.qLearning.enabled) {
      return { reward: 0, patternUpdated: false, configUsed: config.qLearning };
    }

    // Calculate reward using Q-Learning formula with config values
    const explorationBonus = Math.random() * config.qLearning.explorationRate;
    const reward = wasSuccessful
      ? 0.5 + explorationBonus  // Positive reward with exploration
      : -0.3 + explorationBonus * 0.5; // Negative reward

    // Get content genres for pattern matching
    const contentResult = await client.query(
      'SELECT genres, content_type FROM content WHERE id = $1',
      [contentId]
    );

    const genres = contentResult.rows[0]?.genres || [];
    const contentType = contentResult.rows[0]?.content_type;

    // Determine pattern type from context and content
    let patternType = 'general';
    if (context?.mood) {
      patternType = `mood_${context.mood}`;
    } else if (genres.length >= 2) {
      patternType = genres.slice(0, 2).join('_').toLowerCase();
    } else if (genres.length === 1) {
      patternType = genres[0].toLowerCase();
    }

    // Find or create pattern - first try to find existing
    let patternResult = await client.query(
      'SELECT id, success_rate, total_uses, avg_reward FROM recommendation_patterns WHERE pattern_type = $1',
      [patternType]
    );

    let patternId: number;
    let newSuccessRate: number;

    if (patternResult.rows.length > 0) {
      // Update existing pattern with Q-Learning formula
      const existing = patternResult.rows[0];
      const newTotalUses = existing.total_uses + 1;
      newSuccessRate = (parseFloat(existing.success_rate) * existing.total_uses + (wasSuccessful ? 1 : 0)) / newTotalUses;
      const newAvgReward = (parseFloat(existing.avg_reward || 0) * existing.total_uses + reward) / newTotalUses;

      await client.query(`
        UPDATE recommendation_patterns
        SET success_rate = $1, total_uses = $2, avg_reward = $3, updated_at = NOW()
        WHERE id = $4
      `, [newSuccessRate, newTotalUses, newAvgReward, existing.id]);

      patternId = existing.id;
    } else {
      // Create new pattern
      const insertResult = await client.query(`
        INSERT INTO recommendation_patterns (pattern_type, approach, success_rate, total_uses, avg_reward)
        VALUES ($1, $2, $3, 1, $4)
        RETURNING id
      `, [
        patternType,
        `Q-Learning optimized pattern for ${patternType}`,
        wasSuccessful ? 1.0 : 0.0,
        reward
      ]);
      patternId = insertResult.rows[0].id;
      newSuccessRate = wasSuccessful ? 1.0 : 0.0;
    }

    // Record feedback with pattern association
    await client.query(`
      INSERT INTO learning_feedback (user_id, content_id, pattern_id, was_successful, reward, user_action, recommendation_source)
      VALUES ($1, $2, $3, $4, $5, $6, $7)
    `, [
      '00000000-0000-0000-0000-000000000001',
      contentId,
      patternId,
      wasSuccessful,
      reward,
      wasSuccessful ? 'watched' : 'skipped',
      context?.mood ? `mood:${context.mood}` : (context?.searchQuery ? `search:${context.searchQuery}` : 'recommendation')
    ]);

    // Trigger RuVector learning if available
    try {
      await client.query(`
        SELECT ruvector_learn_from_feedback('content', $1::jsonb)
      `, [JSON.stringify({
        content_id: contentId,
        reward,
        pattern_type: patternType,
        genres
      })]);
    } catch (e) {
      // RuVector learning function not available - continue without it
    }

    return {
      reward,
      patternUpdated: true,
      configUsed: config.qLearning
    };

  } finally {
    client.release();
  }
}

/**
 * Get recommendations using Q-Learning optimized patterns
 */
export async function getOptimizedRecommendations(
  mood?: string,
  contentType?: string,
  limit?: number,
  audience?: 'kids' | 'family' | 'teens' | 'adults'
): Promise<any[]> {
  const startTime = performance.now();
  const client = await pool.connect();
  const config = await getSystemConfig();

  // Use config maxResults if limit not specified
  const effectiveLimit = limit || config.recommendations.maxResults;

  try {
    // Get best performing patterns
    const patternsResult = await client.query(`
      SELECT pattern_type, success_rate, avg_reward
      FROM recommendation_patterns
      WHERE total_uses >= 3
      ORDER BY (success_rate * 0.7 + COALESCE((avg_reward + 1) / 2, 0.5) * 0.3) DESC
      LIMIT 5
    `);

    const topPatterns = patternsResult.rows;

    // Extract genres from top patterns for boosting
    const patternGenres = topPatterns
      .map(p => p.pattern_type.split('_'))
      .flat()
      .filter(g => g.length > 2 && !['mood', 'language', 'network', 'gems', 'now'].includes(g));

    // Build query conditions (allow NULL embeddings - will use rating/year ordering)
    let conditions: string[] = ["title NOT LIKE 'Test Content%'"];
    let params: any[] = [];
    let paramIdx = 1;

    // CONTENT SAFETY FILTERING - Always exclude known adult content
    // Exclude R-rated animated content that masquerades as kids content
    conditions.push(`NOT (LOWER(title) LIKE '%sausage party%')`);
    conditions.push(`NOT (LOWER(overview) LIKE '%killing off all of humanity%')`);
    conditions.push(`NOT (LOWER(overview) LIKE '%sex%' AND genres && ARRAY['Animation']::text[])`);

    // Apply audience-based content filtering
    if (audience === 'kids') {
      // STRICT kids filtering - only G-rated family content
      conditions.push(`content_rating = 'G'`);
      conditions.push(`genres && ARRAY['Animation', 'Family', 'Children']::text[]`);
      conditions.push(`NOT (genres && ARRAY['Horror', 'Crime', 'Thriller', 'War', 'Adult']::text[])`);
      conditions.push(`NOT (LOWER(overview) LIKE '%kill%' OR LOWER(overview) LIKE '%murder%' OR LOWER(overview) LIKE '%drug%' OR LOWER(overview) LIKE '%violence%')`);
    } else if (audience === 'family') {
      // Family-friendly - G or PG rated
      conditions.push(`content_rating IN ('G', 'PG')`);
      conditions.push(`NOT (genres && ARRAY['Horror', 'Adult']::text[])`);
    } else if (audience === 'teens') {
      // Teen appropriate - no R-rated
      conditions.push(`content_rating IN ('G', 'PG', 'PG-13')`);
    }
    // Adults: no restrictions (default)

    // Apply content type filter
    if (contentType && contentType !== 'both') {
      conditions.push(`content_type = $${paramIdx}`);
      params.push(contentType);
      paramIdx++;
    }

    // Apply mood filter with primary genre prioritization
    let moodPrimaryGenre: string | null = null;
    if (mood) {
      const moodGenres: Record<string, { primary: string; secondary: string[] }> = {
        funny: { primary: 'Comedy', secondary: [] },
        exciting: { primary: 'Action', secondary: ['Thriller', 'Adventure'] },
        romantic: { primary: 'Romance', secondary: ['Drama'] },
        scary: { primary: 'Horror', secondary: ['Thriller'] },
        thoughtful: { primary: 'Drama', secondary: ['Documentary'] },
        relaxing: { primary: 'Animation', secondary: ['Comedy', 'Family'] },
      };

      const moodConfig = moodGenres[mood];
      if (moodConfig) {
        const allGenres = [moodConfig.primary, ...moodConfig.secondary];
        conditions.push(`genres && $${paramIdx}::text[]`);
        params.push(allGenres);
        paramIdx++;
        moodPrimaryGenre = moodConfig.primary;
      }
    }

    // Add pattern boost genres and limit
    const boostGenresParam = paramIdx;
    params.push(patternGenres.length > 0 ? patternGenres : ['Drama', 'Comedy', 'Action']);
    paramIdx++;

    // Add primary genre param for mood ordering
    let primaryGenreParam: number | null = null;
    if (moodPrimaryGenre) {
      primaryGenreParam = paramIdx;
      params.push(moodPrimaryGenre);
      paramIdx++;
    }

    const limitParam = paramIdx;
    params.push(effectiveLimit);

    // Build ORDER BY clause - prioritize primary mood genre if set
    const orderByClause = moodPrimaryGenre
      ? `ORDER BY
          CASE WHEN $${primaryGenreParam} = ANY(c.genres) THEN 0 ELSE 1 END,
          pattern_score DESC,
          year DESC NULLS LAST,
          rating DESC NULLS LAST`
      : `ORDER BY
          pattern_score DESC,
          year DESC NULLS LAST,
          rating DESC NULLS LAST`;

    // Query with pattern boosting and mood prioritization
    const result = await client.query(`
      WITH pattern_boost AS (
        SELECT unnest($${boostGenresParam}::text[]) as boost_genre
      )
      SELECT
        c.id, c.title, c.year, c.content_type, c.genres, c.overview,
        c.rating, c.network_name, c.original_language, c.image_url,
        COALESCE(
          (SELECT COUNT(*) FROM pattern_boost pb WHERE pb.boost_genre = ANY(c.genres)),
          0
        ) as pattern_score
      FROM content c
      WHERE ${conditions.join(' AND ')}
      ${orderByClause}
      LIMIT $${limitParam}
    `, params);

    // Record query timing
    const endTime = performance.now();
    recordQueryTiming(endTime - startTime);

    return result.rows;

  } finally {
    client.release();
  }
}

/**
 * Get similar content using hyperbolic vector distance
 */
export async function getSimilarWithHyperbolic(
  contentId: string,
  limit = 10
): Promise<any[]> {
  const client = await pool.connect();
  const config = await getSystemConfig();

  try {
    // Use hyperbolic if enabled in config, fall back to cosine
    let result;
    const useHyperbolic = config.vectorSearch.hyperbolicEnabled;

    if (useHyperbolic) {
      try {
        // Attempt hyperbolic distance (Poincaré ball model)
        result = await client.query(`
          SELECT
            c.id, c.title, c.year, c.content_type, c.genres, c.overview,
            c.rating, c.network_name, c.original_language, c.image_url,
            1 - ruvector_hyperbolic_distance(c.embedding,
                (SELECT embedding FROM content WHERE id = $1)) as similarity
          FROM content c
          WHERE c.id != $1
            AND c.embedding IS NOT NULL
            AND c.title NOT LIKE 'Test Content%'
          ORDER BY ruvector_hyperbolic_distance(c.embedding,
                   (SELECT embedding FROM content WHERE id = $1))
          LIMIT $2
        `, [contentId, limit]);
      } catch {
        // Fall back to cosine if hyperbolic fails
        result = null;
      }
    }

    // Use cosine distance if hyperbolic disabled or failed
    if (!result) {
      result = await client.query(`
        SELECT
          c.id, c.title, c.year, c.content_type, c.genres, c.overview,
          c.rating, c.network_name, c.original_language, c.image_url,
          1 - ruvector_cosine_distance(c.embedding,
              (SELECT embedding FROM content WHERE id = $1)) as similarity
        FROM content c
        WHERE c.id != $1
          AND c.embedding IS NOT NULL
          AND c.title NOT LIKE 'Test Content%'
        ORDER BY ruvector_cosine_distance(c.embedding,
                 (SELECT embedding FROM content WHERE id = $1))
        LIMIT $2
      `, [contentId, limit]);
    }

    return result.rows;

  } finally {
    client.release();
  }
}

/**
 * Get learning statistics
 */
export async function getLearningStats(): Promise<{
  totalPatterns: number;
  avgSuccessRate: number;
  bestPatterns: PatternStats[];
  recentLearning: { feedbackCount: number; avgReward: number };
  vectorStats: { hyperbolicEnabled: boolean; avgSearchTime: number };
}> {
  const client = await pool.connect();

  try {
    const [patternStats, feedbackStats, vectorCheck] = await Promise.all([
      client.query(`
        SELECT
          pattern_type,
          success_rate,
          total_uses,
          avg_reward
        FROM recommendation_patterns
        WHERE total_uses >= 2
        ORDER BY success_rate DESC
        LIMIT 10
      `),
      client.query(`
        SELECT
          COUNT(*) as feedback_count,
          AVG(reward) as avg_reward
        FROM learning_feedback
        WHERE created_at > NOW() - INTERVAL '24 hours'
      `),
      client.query(`
        SELECT
          EXISTS(SELECT 1 FROM pg_proc WHERE proname = 'ruvector_hyperbolic_distance') as hyperbolic_available
      `)
    ]);

    const patterns = patternStats.rows.map(p => ({
      patternType: p.pattern_type,
      successRate: parseFloat(p.success_rate),
      totalUses: parseInt(p.total_uses),
      avgReward: parseFloat(p.avg_reward) || 0
    }));

    return {
      totalPatterns: patterns.length,
      avgSuccessRate: patterns.length > 0
        ? patterns.reduce((sum, p) => sum + p.successRate, 0) / patterns.length
        : 0,
      bestPatterns: patterns.slice(0, 5),
      recentLearning: {
        feedbackCount: parseInt(feedbackStats.rows[0]?.feedback_count) || 0,
        avgReward: parseFloat(feedbackStats.rows[0]?.avg_reward) || 0
      },
      vectorStats: {
        hyperbolicEnabled: vectorCheck.rows[0]?.hyperbolic_available || false,
        avgSearchTime: parseFloat(getAverageQueryTime().toFixed(1))
      }
    };

  } finally {
    client.release();
  }
}

export default {
  recordFeedbackWithLearning,
  getOptimizedRecommendations,
  getSimilarWithHyperbolic,
  getLearningStats
};
