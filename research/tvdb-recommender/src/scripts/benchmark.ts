/**
 * Benchmark and Optimization Script
 * Uses real TVDB data to test and optimize the recommendation system
 */

import 'dotenv/config';
import { config } from 'dotenv';
import { resolve } from 'path';
import pino from 'pino';
import { Pool } from 'pg';

// Load from root .env
config({ path: resolve(process.cwd(), '../../.env') });

const logger = pino({
  name: 'benchmark',
  level: 'info',
  transport: {
    target: 'pino-pretty',
    options: { colorize: true }
  }
});

interface BenchmarkResult {
  name: string;
  operations: number;
  totalTimeMs: number;
  avgTimeMs: number;
  opsPerSecond: number;
  p50Ms?: number;
  p99Ms?: number;
}

class Benchmark {
  private pool: Pool;
  private results: BenchmarkResult[] = [];

  constructor() {
    this.pool = new Pool({
      connectionString: process.env.DATABASE_URL || 'postgresql://tvdb:tvdb_recommender_2024@localhost:5432/tvdb_recommender',
      max: 10
    });
  }

  async initialize(): Promise<void> {
    logger.info('Connecting to database...');
    const client = await this.pool.connect();
    try {
      const result = await client.query('SELECT COUNT(*) FROM content');
      logger.info({ count: result.rows[0].count }, 'Content items in database');
    } finally {
      client.release();
    }
  }

  /**
   * Generate test data with random embeddings
   */
  async generateTestData(count: number = 1000): Promise<void> {
    logger.info({ count }, 'Generating test data...');
    const start = Date.now();

    const client = await this.pool.connect();
    try {
      await client.query('BEGIN');

      const genres = [
        ['Drama'],
        ['Drama', 'Action'],
        ['Comedy', 'Romance'],
        ['Thriller', 'Crime'],
        ['Sci-Fi', 'Action'],
        ['Horror'],
        ['Documentary'],
        ['Animation']
      ];

      for (let i = 2; i <= count; i++) {
        const embedding = Array.from({ length: 384 }, () => Math.random());
        const genreIdx = Math.floor(Math.random() * genres.length);

        await client.query(
          `INSERT INTO content (id, content_type, title, year, overview, genres, rating, embedding)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8::ruvector(384))
           ON CONFLICT (id) DO NOTHING`,
          [
            `tvdb-${i}`,
            Math.random() > 0.3 ? 'series' : 'movie',
            `Test Content ${i}`,
            2000 + Math.floor(Math.random() * 24),
            `This is test content ${i} with various themes.`,
            genres[genreIdx],
            (Math.random() * 5 + 5).toFixed(1),
            `[${embedding.join(',')}]`
          ]
        );

        if (i % 100 === 0) {
          logger.info({ progress: i }, 'Inserted content');
        }
      }

      await client.query('COMMIT');
      const duration = Date.now() - start;
      logger.info({ count, durationMs: duration }, 'Test data generated');
    } catch (error) {
      await client.query('ROLLBACK');
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Benchmark vector similarity search
   */
  async benchmarkVectorSearch(iterations: number = 100): Promise<BenchmarkResult> {
    logger.info({ iterations }, 'Benchmarking vector search...');

    const times: number[] = [];
    const client = await this.pool.connect();

    try {
      // Get a random content ID to use as query
      const { rows: [sample] } = await client.query(
        'SELECT id FROM content WHERE embedding IS NOT NULL LIMIT 1'
      );

      if (!sample) {
        throw new Error('No content with embeddings found');
      }

      for (let i = 0; i < iterations; i++) {
        const start = process.hrtime.bigint();

        await client.query(`
          SELECT c.id, c.title,
                 1 - ruvector_cosine_distance(c.embedding,
                     (SELECT embedding FROM content WHERE id = $1)) as similarity
          FROM content c
          WHERE c.id != $1 AND c.embedding IS NOT NULL
          ORDER BY ruvector_cosine_distance(c.embedding,
                   (SELECT embedding FROM content WHERE id = $1))
          LIMIT 10
        `, [sample.id]);

        const end = process.hrtime.bigint();
        times.push(Number(end - start) / 1e6); // Convert to ms
      }
    } finally {
      client.release();
    }

    return this.calculateStats('Vector Search (Top 10)', times);
  }

  /**
   * Benchmark personalized recommendations
   */
  async benchmarkRecommendations(iterations: number = 100): Promise<BenchmarkResult> {
    logger.info({ iterations }, 'Benchmarking recommendations...');

    const times: number[] = [];
    const client = await this.pool.connect();

    try {
      // Create a test user with preferences
      await client.query(`
        INSERT INTO users (id, email)
        VALUES ('00000000-0000-0000-0000-000000000001', 'benchmark@test.com')
        ON CONFLICT DO NOTHING
      `);

      // Create preference vector
      const prefVector = Array.from({ length: 384 }, () => Math.random());
      await client.query(`
        INSERT INTO user_preferences (user_id, preference_vector, genre_weights)
        VALUES ($1, $2::ruvector(384), '{"Drama": 0.8, "Action": 0.6}'::jsonb)
        ON CONFLICT (user_id) DO UPDATE SET preference_vector = $2::ruvector(384)
      `, ['00000000-0000-0000-0000-000000000001', `[${prefVector.join(',')}]`]);

      for (let i = 0; i < iterations; i++) {
        const start = process.hrtime.bigint();

        await client.query(`
          SELECT c.id, c.title, c.content_type, c.genres,
                 1 - ruvector_cosine_distance(c.embedding, up.preference_vector) as similarity
          FROM content c
          CROSS JOIN user_preferences up
          WHERE up.user_id = $1
          AND c.embedding IS NOT NULL
          ORDER BY ruvector_cosine_distance(c.embedding, up.preference_vector)
          LIMIT 20
        `, ['00000000-0000-0000-0000-000000000001']);

        const end = process.hrtime.bigint();
        times.push(Number(end - start) / 1e6);
      }
    } finally {
      client.release();
    }

    return this.calculateStats('Personalized Recommendations (Top 20)', times);
  }

  /**
   * Benchmark pattern learning
   */
  async benchmarkPatternLearning(iterations: number = 100): Promise<BenchmarkResult> {
    logger.info({ iterations }, 'Benchmarking pattern learning...');

    const times: number[] = [];
    const client = await this.pool.connect();

    try {
      for (let i = 0; i < iterations; i++) {
        const start = process.hrtime.bigint();

        // Simulate recording feedback and updating pattern
        await client.query(`
          INSERT INTO learning_feedback (user_id, content_id, pattern_id, was_successful, reward, user_action)
          SELECT
            '00000000-0000-0000-0000-000000000001',
            (SELECT id FROM content ORDER BY random() LIMIT 1),
            (SELECT id FROM recommendation_patterns ORDER BY random() LIMIT 1),
            random() > 0.3,
            random() * 2 - 1,
            'watched'
        `);

        // Update pattern success rate
        await client.query(`
          UPDATE recommendation_patterns rp
          SET success_rate = (
            SELECT COALESCE(AVG(CASE WHEN was_successful THEN 1.0 ELSE 0.0 END), 0.5)
            FROM learning_feedback lf
            WHERE lf.pattern_id = rp.id
          ),
          total_uses = total_uses + 1,
          updated_at = NOW()
          WHERE id = (SELECT id FROM recommendation_patterns ORDER BY random() LIMIT 1)
        `);

        const end = process.hrtime.bigint();
        times.push(Number(end - start) / 1e6);
      }
    } finally {
      client.release();
    }

    return this.calculateStats('Pattern Learning Update', times);
  }

  /**
   * Benchmark RuVector auto-tuning
   */
  async benchmarkAutoTune(): Promise<void> {
    logger.info('Running RuVector auto-tune...');

    const client = await this.pool.connect();
    try {
      const start = Date.now();
      const result = await client.query(`
        SELECT ruvector_auto_tune('content', 'balanced')
      `);
      const duration = Date.now() - start;

      logger.info({
        durationMs: duration,
        result: result.rows[0]
      }, 'Auto-tune complete');
    } finally {
      client.release();
    }
  }

  /**
   * Get learning statistics
   */
  async getLearningStats(): Promise<void> {
    logger.info('Getting learning statistics...');

    const client = await this.pool.connect();
    try {
      const result = await client.query(`
        SELECT ruvector_learning_stats('content')
      `);
      logger.info({ stats: result.rows[0] }, 'Learning stats');

      // Get pattern stats
      const patterns = await client.query(`
        SELECT pattern_type, success_rate, total_uses, avg_reward
        FROM recommendation_patterns
        ORDER BY success_rate DESC
      `);
      logger.info({ patterns: patterns.rows }, 'Pattern stats');
    } finally {
      client.release();
    }
  }

  /**
   * Optimize database
   */
  async optimize(): Promise<void> {
    logger.info('Optimizing database...');

    const client = await this.pool.connect();
    try {
      // Analyze tables
      await client.query('ANALYZE content');
      await client.query('ANALYZE user_preferences');
      await client.query('ANALYZE learning_feedback');

      // Get table sizes
      const sizes = await client.query(`
        SELECT
          relname as table_name,
          pg_size_pretty(pg_total_relation_size(relid)) as total_size,
          pg_size_pretty(pg_relation_size(relid)) as data_size,
          pg_size_pretty(pg_indexes_size(relid)) as index_size
        FROM pg_catalog.pg_statio_user_tables
        ORDER BY pg_total_relation_size(relid) DESC
      `);
      logger.info({ sizes: sizes.rows }, 'Table sizes');
    } finally {
      client.release();
    }
  }

  /**
   * Calculate statistics from timing data
   */
  private calculateStats(name: string, times: number[]): BenchmarkResult {
    times.sort((a, b) => a - b);

    const totalTimeMs = times.reduce((a, b) => a + b, 0);
    const avgTimeMs = totalTimeMs / times.length;
    const p50Ms = times[Math.floor(times.length * 0.5)];
    const p99Ms = times[Math.floor(times.length * 0.99)];
    const opsPerSecond = 1000 / avgTimeMs;

    const result: BenchmarkResult = {
      name,
      operations: times.length,
      totalTimeMs,
      avgTimeMs,
      opsPerSecond,
      p50Ms,
      p99Ms
    };

    this.results.push(result);
    return result;
  }

  /**
   * Print benchmark report
   */
  printReport(): void {
    console.log('\n' + '='.repeat(80));
    console.log('BENCHMARK REPORT - TVDB Self-Learning Recommendation System');
    console.log('='.repeat(80) + '\n');

    console.log('| Benchmark                              | Ops    | Avg (ms) | p50 (ms) | p99 (ms) | Ops/sec  |');
    console.log('|----------------------------------------|--------|----------|----------|----------|----------|');

    for (const r of this.results) {
      console.log(
        `| ${r.name.padEnd(38)} | ${r.operations.toString().padStart(6)} | ${r.avgTimeMs.toFixed(2).padStart(8)} | ${(r.p50Ms || 0).toFixed(2).padStart(8)} | ${(r.p99Ms || 0).toFixed(2).padStart(8)} | ${r.opsPerSecond.toFixed(0).padStart(8)} |`
      );
    }

    console.log('\n' + '='.repeat(80) + '\n');
  }

  async close(): Promise<void> {
    await this.pool.end();
  }
}

async function main() {
  const benchmark = new Benchmark();

  try {
    await benchmark.initialize();

    // Check if we need to generate test data
    const checkData = await benchmark['pool'].query('SELECT COUNT(*) FROM content');
    if (parseInt(checkData.rows[0].count) < 100) {
      await benchmark.generateTestData(500);
    }

    // Run benchmarks
    const vectorResult = await benchmark.benchmarkVectorSearch(100);
    logger.info(vectorResult, 'Vector search benchmark');

    const recResult = await benchmark.benchmarkRecommendations(100);
    logger.info(recResult, 'Recommendations benchmark');

    const learnResult = await benchmark.benchmarkPatternLearning(50);
    logger.info(learnResult, 'Pattern learning benchmark');

    // Optimize
    await benchmark.optimize();
    await benchmark.benchmarkAutoTune();
    await benchmark.getLearningStats();

    // Print final report
    benchmark.printReport();

  } catch (error) {
    logger.error({ error }, 'Benchmark failed');
    process.exit(1);
  } finally {
    await benchmark.close();
  }
}

main();
