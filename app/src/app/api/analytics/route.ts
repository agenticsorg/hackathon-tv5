import { NextRequest, NextResponse } from 'next/server';
import { Pool } from 'pg';

// Force dynamic rendering - don't try to pre-render during build
export const dynamic = 'force-dynamic';

const pool = new Pool({
  connectionString: process.env.DATABASE_URL ||
    'postgresql://tvdb:tvdb_recommender_2024@localhost:5432/tvdb_recommender',
  max: 10,
});

export async function GET(request: NextRequest) {
  const client = await pool.connect();

  try {
    // Execute all queries in parallel for maximum performance
    const [
      contentStats,
      patternEvolution,
      feedbackTimeline,
      genreNetwork,
      learningMetrics,
      algorithmPerformance,
      contentDistribution,
      recentActivity,
    ] = await Promise.all([
      // Content statistics
      client.query(`
        SELECT
          COUNT(*) as total,
          SUM(CASE WHEN content_type = 'series' THEN 1 ELSE 0 END) as series,
          SUM(CASE WHEN content_type = 'movie' THEN 1 ELSE 0 END) as movies,
          AVG(rating) as avg_rating,
          COUNT(CASE WHEN embedding IS NOT NULL THEN 1 END) as with_embeddings
        FROM content
        WHERE title NOT LIKE 'Test Content%'
      `),

      // Pattern evolution over time (simulated from pattern updates)
      client.query(`
        SELECT
          pattern_type,
          success_rate,
          total_uses,
          COALESCE(avg_reward, 0) as avg_reward,
          updated_at
        FROM recommendation_patterns
        ORDER BY total_uses DESC
        LIMIT 20
      `),

      // Feedback timeline (last 7 days aggregated by hour)
      client.query(`
        SELECT
          DATE_TRUNC('hour', created_at) as hour,
          COUNT(*) as feedback_count,
          SUM(CASE WHEN was_successful THEN 1 ELSE 0 END) as positive,
          AVG(reward) as avg_reward
        FROM learning_feedback
        WHERE created_at > NOW() - INTERVAL '7 days'
        GROUP BY DATE_TRUNC('hour', created_at)
        ORDER BY hour DESC
        LIMIT 168
      `),

      // Genre co-occurrence network for graph visualization
      client.query(`
        WITH genre_pairs AS (
          SELECT
            g1.genre as source,
            g2.genre as target,
            COUNT(*) as weight
          FROM content c,
            LATERAL unnest(c.genres) AS g1(genre),
            LATERAL unnest(c.genres) AS g2(genre)
          WHERE g1.genre < g2.genre
            AND c.title NOT LIKE 'Test Content%'
          GROUP BY g1.genre, g2.genre
          HAVING COUNT(*) >= 5
        )
        SELECT * FROM genre_pairs
        ORDER BY weight DESC
        LIMIT 50
      `),

      // Learning metrics over time
      client.query(`
        SELECT
          pattern_type,
          success_rate,
          total_uses,
          COALESCE(avg_reward, 0) as avg_reward,
          EXTRACT(EPOCH FROM (NOW() - updated_at)) / 3600 as hours_since_update
        FROM recommendation_patterns
        WHERE total_uses >= 2
        ORDER BY success_rate DESC
        LIMIT 15
      `),

      // Algorithm performance comparison
      client.query(`
        SELECT
          COALESCE(recommendation_source, 'recommendation') as algorithm,
          COUNT(*) as total_uses,
          AVG(CASE WHEN was_successful THEN 1.0 ELSE 0.0 END) as success_rate,
          AVG(reward) as avg_reward
        FROM learning_feedback
        GROUP BY recommendation_source
        ORDER BY success_rate DESC
      `),

      // Content distribution by year and type
      client.query(`
        SELECT
          year,
          content_type,
          COUNT(*) as count
        FROM content
        WHERE year IS NOT NULL
          AND year >= 1990
          AND title NOT LIKE 'Test Content%'
        GROUP BY year, content_type
        ORDER BY year DESC
        LIMIT 70
      `),

      // Recent activity stream
      client.query(`
        SELECT
          lf.id,
          lf.was_successful,
          lf.reward,
          lf.user_action,
          lf.created_at,
          c.title,
          c.content_type,
          rp.pattern_type
        FROM learning_feedback lf
        LEFT JOIN content c ON lf.content_id = c.id
        LEFT JOIN recommendation_patterns rp ON lf.pattern_id = rp.id
        ORDER BY lf.created_at DESC
        LIMIT 50
      `),
    ]);

    // Process genre network into nodes and edges for force graph
    const genres = new Set<string>();
    const edges: { source: string; target: string; weight: number }[] = [];

    genreNetwork.rows.forEach((row: any) => {
      genres.add(row.source);
      genres.add(row.target);
      edges.push({
        source: row.source,
        target: row.target,
        weight: parseInt(row.weight),
      });
    });

    const nodes = Array.from(genres).map((genre) => ({
      id: genre,
      name: genre,
      val: edges.filter(e => e.source === genre || e.target === genre)
        .reduce((sum, e) => sum + e.weight, 0),
    }));

    // Calculate performance metrics
    const totalFeedback = feedbackTimeline.rows.reduce(
      (sum: number, row: any) => sum + parseInt(row.feedback_count), 0
    );
    const positiveRate = feedbackTimeline.rows.reduce(
      (sum: number, row: any) => sum + parseInt(row.positive), 0
    ) / Math.max(totalFeedback, 1);

    return NextResponse.json({
      overview: {
        totalContent: parseInt(contentStats.rows[0]?.total) || 0,
        series: parseInt(contentStats.rows[0]?.series) || 0,
        movies: parseInt(contentStats.rows[0]?.movies) || 0,
        avgRating: parseFloat(contentStats.rows[0]?.avg_rating) || 0,
        embeddingCoverage: parseInt(contentStats.rows[0]?.with_embeddings) /
          Math.max(parseInt(contentStats.rows[0]?.total), 1) * 100,
        totalFeedback,
        positiveRate,
      },

      patterns: patternEvolution.rows.map((p: any) => ({
        type: p.pattern_type,
        successRate: parseFloat(p.success_rate),
        uses: parseInt(p.total_uses),
        avgReward: parseFloat(p.avg_reward),
        lastUpdated: p.updated_at,
      })),

      feedbackTimeline: feedbackTimeline.rows.map((f: any) => ({
        hour: f.hour,
        count: parseInt(f.feedback_count),
        positive: parseInt(f.positive),
        avgReward: parseFloat(f.avg_reward) || 0,
      })).reverse(),

      genreNetwork: {
        nodes,
        edges,
      },

      learningMetrics: learningMetrics.rows.map((m: any) => ({
        pattern: m.pattern_type,
        successRate: parseFloat(m.success_rate),
        uses: parseInt(m.total_uses),
        avgReward: parseFloat(m.avg_reward),
        freshness: parseFloat(m.hours_since_update),
      })),

      algorithms: algorithmPerformance.rows.map((a: any) => ({
        name: a.algorithm,
        uses: parseInt(a.total_uses),
        successRate: parseFloat(a.success_rate),
        avgReward: parseFloat(a.avg_reward) || 0,
      })),

      contentByYear: contentDistribution.rows.map((c: any) => ({
        year: c.year,
        type: c.content_type,
        count: parseInt(c.count),
      })),

      recentActivity: recentActivity.rows.map((a: any) => ({
        id: a.id,
        success: a.was_successful,
        reward: parseFloat(a.reward),
        action: a.user_action,
        time: a.created_at,
        title: a.title,
        contentType: a.content_type,
        pattern: a.pattern_type,
      })),

      timestamp: new Date().toISOString(),
    });
  } catch (error: any) {
    console.error('Analytics error:', error);
    return NextResponse.json({ error: error.message }, { status: 500 });
  } finally {
    client.release();
  }
}
