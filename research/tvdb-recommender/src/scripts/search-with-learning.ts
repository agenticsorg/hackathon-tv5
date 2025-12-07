/**
 * Search with Learning - Demonstrates self-learning recommendation system
 * Performs searches, records feedback, and shows learning metrics
 */

import 'dotenv/config';
import { config } from 'dotenv';
import { resolve } from 'path';
import { Pool } from 'pg';
import axios from 'axios';

// Load from root .env
config({ path: resolve(process.cwd(), '../../.env') });

const TVDB_API_KEY = process.env.TVDB_API_KEY;
const TVDB_BASE_URL = 'https://api4.thetvdb.com/v4';

interface SearchMetrics {
  searchQuery: string;
  tvdbResults: number;
  embeddingsGenerated: number;
  similarContentFound: number;
  patternsUsed: string[];
  learningUpdates: number;
  searchTimeMs: number;
  embeddingTimeMs: number;
  recommendationTimeMs: number;
  totalTimeMs: number;
}

interface LearningMetrics {
  totalPatterns: number;
  patternStats: { type: string; successRate: number; uses: number }[];
  totalFeedback: number;
  positiveRate: number;
  avgReward: number;
  contentCount: number;
  usersCount: number;
}

class SearchWithLearning {
  private pool: Pool;
  private tvdbToken: string | null = null;

  constructor() {
    this.pool = new Pool({
      connectionString: process.env.DATABASE_URL ||
        'postgresql://tvdb:tvdb_recommender_2024@localhost:5432/tvdb_recommender',
      max: 10
    });
  }

  /**
   * Authenticate with TVDB
   */
  async authenticate(): Promise<void> {
    if (!TVDB_API_KEY || TVDB_API_KEY === 'your-tvdb-api-key-here') {
      console.log('⚠️  TVDB_API_KEY not set - using mock data for demonstration');
      return;
    }

    try {
      const response = await axios.post(`${TVDB_BASE_URL}/login`, {
        apikey: TVDB_API_KEY
      });
      this.tvdbToken = response.data.data.token;
      console.log('✅ Authenticated with TVDB API');
    } catch (error: any) {
      console.log('⚠️  TVDB auth failed:', error.message, '- using mock data');
    }
  }

  /**
   * Search TVDB API
   */
  async searchTVDB(query: string): Promise<any[]> {
    if (!this.tvdbToken) {
      // Return mock data if not authenticated
      return this.getMockSearchResults(query);
    }

    try {
      const response = await axios.get(`${TVDB_BASE_URL}/search`, {
        params: { query, limit: 10 },
        headers: { Authorization: `Bearer ${this.tvdbToken}` }
      });
      return response.data.data || [];
    } catch (error) {
      console.log('⚠️  Search failed, using mock data');
      return this.getMockSearchResults(query);
    }
  }

  /**
   * Generate mock search results
   */
  private getMockSearchResults(query: string): any[] {
    const mockShows = [
      { id: '81189', name: 'Breaking Bad', year: '2008', type: 'series',
        overview: 'A chemistry teacher diagnosed with terminal cancer partners with a former student to manufacture crystal methamphetamine.',
        genres: ['Drama', 'Crime', 'Thriller'] },
      { id: '121361', name: 'Game of Thrones', year: '2011', type: 'series',
        overview: 'Nine noble families fight for control over the lands of Westeros.',
        genres: ['Drama', 'Fantasy', 'Adventure'] },
      { id: '305288', name: 'Stranger Things', year: '2016', type: 'series',
        overview: 'When a young boy disappears, his mother and friends uncover supernatural forces.',
        genres: ['Drama', 'Sci-Fi', 'Horror'] },
      { id: '73739', name: 'The Office', year: '2005', type: 'series',
        overview: 'A mockumentary sitcom about office employees at the Scranton branch of Dunder Mifflin.',
        genres: ['Comedy'] },
      { id: '79349', name: 'Dexter', year: '2006', type: 'series',
        overview: 'A forensic blood spatter analyst moonlights as a serial killer who only targets murderers.',
        genres: ['Drama', 'Crime', 'Mystery'] }
    ];

    const queryLower = query.toLowerCase();
    return mockShows.filter(s =>
      s.name.toLowerCase().includes(queryLower) ||
      s.overview.toLowerCase().includes(queryLower) ||
      s.genres.some(g => g.toLowerCase().includes(queryLower))
    ).slice(0, 5);
  }

  /**
   * Generate embedding for text (simplified - random for demo)
   */
  private generateEmbedding(): string {
    const embedding = Array.from({ length: 384 }, () => Math.random());
    return `[${embedding.join(',')}]`;
  }

  /**
   * Store content with embedding
   */
  async storeContent(item: any): Promise<void> {
    const client = await this.pool.connect();
    try {
      await client.query(`
        INSERT INTO content (id, content_type, title, year, overview, genres, rating, embedding)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8::ruvector(384))
        ON CONFLICT (id) DO UPDATE SET
          title = EXCLUDED.title,
          overview = EXCLUDED.overview,
          embedding = EXCLUDED.embedding,
          updated_at = NOW()
      `, [
        item.id,
        item.type || 'series',
        item.name,
        parseInt(item.year) || 2020,
        item.overview || '',
        item.genres || [],
        (Math.random() * 3 + 7).toFixed(1), // Random rating 7-10
        this.generateEmbedding()
      ]);
    } finally {
      client.release();
    }
  }

  /**
   * Find similar content using vector search
   */
  async findSimilar(contentId: string, limit: number = 5): Promise<any[]> {
    const client = await this.pool.connect();
    try {
      const result = await client.query(`
        SELECT c.id, c.title, c.genres, c.rating,
               1 - ruvector_cosine_distance(c.embedding,
                   (SELECT embedding FROM content WHERE id = $1)) as similarity
        FROM content c
        WHERE c.id != $1 AND c.embedding IS NOT NULL
        ORDER BY ruvector_cosine_distance(c.embedding,
                 (SELECT embedding FROM content WHERE id = $1))
        LIMIT $2
      `, [contentId, limit]);
      return result.rows;
    } finally {
      client.release();
    }
  }

  /**
   * Select best recommendation pattern
   */
  async selectPattern(): Promise<{ id: number; type: string; approach: string }> {
    const client = await this.pool.connect();
    try {
      const result = await client.query(`
        SELECT id, pattern_type, approach, success_rate
        FROM recommendation_patterns
        ORDER BY success_rate DESC, random()
        LIMIT 1
      `);
      return result.rows[0] ? {
        id: result.rows[0].id,
        type: result.rows[0].pattern_type,
        approach: result.rows[0].approach
      } : { id: 1, type: 'similar_content', approach: 'Vector similarity' };
    } finally {
      client.release();
    }
  }

  /**
   * Record learning feedback
   */
  async recordFeedback(patternId: number, contentId: string, success: boolean): Promise<void> {
    const client = await this.pool.connect();
    try {
      // Ensure user exists
      await client.query(`
        INSERT INTO users (id, email)
        VALUES ('00000000-0000-0000-0000-000000000001', 'demo@test.com')
        ON CONFLICT DO NOTHING
      `);

      // Record feedback
      const reward = success ? (Math.random() * 0.5 + 0.5) : (Math.random() * 0.5 - 0.5);
      await client.query(`
        INSERT INTO learning_feedback (user_id, content_id, pattern_id, was_successful, reward, user_action)
        VALUES ($1, $2, $3, $4, $5, $6)
      `, [
        '00000000-0000-0000-0000-000000000001',
        contentId,
        patternId,
        success,
        reward,
        success ? 'watched' : 'skipped'
      ]);

      // Update pattern success rate
      await client.query(`
        UPDATE recommendation_patterns
        SET success_rate = (
          SELECT COALESCE(AVG(CASE WHEN was_successful THEN 1.0 ELSE 0.0 END), success_rate)
          FROM learning_feedback
          WHERE pattern_id = $1
        ),
        total_uses = total_uses + 1,
        avg_reward = (
          SELECT COALESCE(AVG(reward), avg_reward)
          FROM learning_feedback
          WHERE pattern_id = $1
        ),
        updated_at = NOW()
        WHERE id = $1
      `, [patternId]);
    } finally {
      client.release();
    }
  }

  /**
   * Get learning metrics
   */
  async getLearningMetrics(): Promise<LearningMetrics> {
    const client = await this.pool.connect();
    try {
      const [patterns, feedback, content, users] = await Promise.all([
        client.query(`
          SELECT pattern_type, success_rate, total_uses, avg_reward
          FROM recommendation_patterns
          ORDER BY success_rate DESC
        `),
        client.query(`
          SELECT
            COUNT(*) as total,
            SUM(CASE WHEN was_successful THEN 1 ELSE 0 END) as positive,
            AVG(reward) as avg_reward
          FROM learning_feedback
        `),
        client.query('SELECT COUNT(*) FROM content'),
        client.query('SELECT COUNT(*) FROM users')
      ]);

      const feedbackRow = feedback.rows[0];
      const total = parseInt(feedbackRow.total) || 0;
      const positive = parseInt(feedbackRow.positive) || 0;

      return {
        totalPatterns: patterns.rows.length,
        patternStats: patterns.rows.map(p => ({
          type: p.pattern_type,
          successRate: parseFloat(p.success_rate),
          uses: parseInt(p.total_uses)
        })),
        totalFeedback: total,
        positiveRate: total > 0 ? positive / total : 0,
        avgReward: parseFloat(feedbackRow.avg_reward) || 0,
        contentCount: parseInt(content.rows[0].count),
        usersCount: parseInt(users.rows[0].count)
      };
    } finally {
      client.release();
    }
  }

  /**
   * Perform search with learning
   */
  async searchWithLearning(query: string): Promise<SearchMetrics> {
    const startTime = Date.now();
    const metrics: Partial<SearchMetrics> = {
      searchQuery: query,
      patternsUsed: [],
      learningUpdates: 0
    };

    console.log(`\n🔍 Searching for: "${query}"`);
    console.log('─'.repeat(50));

    // 1. Search TVDB
    const searchStart = Date.now();
    const results = await this.searchTVDB(query);
    metrics.searchTimeMs = Date.now() - searchStart;
    metrics.tvdbResults = results.length;
    console.log(`📺 Found ${results.length} results from TVDB (${metrics.searchTimeMs}ms)`);

    // 2. Generate embeddings and store
    const embeddingStart = Date.now();
    for (const item of results) {
      await this.storeContent(item);
    }
    metrics.embeddingTimeMs = Date.now() - embeddingStart;
    metrics.embeddingsGenerated = results.length;
    console.log(`🧠 Generated ${results.length} embeddings (${metrics.embeddingTimeMs}ms)`);

    // 3. Select pattern and find recommendations
    const recStart = Date.now();
    const pattern = await this.selectPattern();
    metrics.patternsUsed!.push(pattern.type);
    console.log(`📊 Using pattern: "${pattern.type}" - ${pattern.approach}`);

    let similarCount = 0;
    if (results.length > 0) {
      const similar = await this.findSimilar(results[0].id);
      similarCount = similar.length;
      console.log(`\n🎯 Similar content to "${results[0].name}":`);
      similar.forEach((s, i) => {
        console.log(`   ${i + 1}. ${s.title} (similarity: ${(parseFloat(s.similarity) * 100).toFixed(1)}%)`);
      });
    }
    metrics.recommendationTimeMs = Date.now() - recStart;
    metrics.similarContentFound = similarCount;

    // 4. Simulate user feedback and learning
    console.log(`\n📝 Recording learning feedback...`);
    for (const item of results.slice(0, 3)) {
      const success = Math.random() > 0.3; // 70% positive rate
      await this.recordFeedback(pattern.id, item.id, success);
      metrics.learningUpdates!++;
    }
    console.log(`   ✅ Recorded ${metrics.learningUpdates} feedback entries`);

    metrics.totalTimeMs = Date.now() - startTime;

    return metrics as SearchMetrics;
  }

  async close(): Promise<void> {
    await this.pool.end();
  }
}

async function main() {
  const search = new SearchWithLearning();

  console.log('\n' + '═'.repeat(60));
  console.log('  TVDB Self-Learning Recommendation System - Live Demo');
  console.log('═'.repeat(60));

  try {
    await search.authenticate();

    // Perform multiple searches to demonstrate learning
    const queries = ['drama', 'comedy', 'thriller', 'sci-fi', 'crime'];
    const allMetrics: SearchMetrics[] = [];

    for (const query of queries) {
      const metrics = await search.searchWithLearning(query);
      allMetrics.push(metrics);
    }

    // Get final learning metrics
    console.log('\n' + '═'.repeat(60));
    console.log('  LEARNING METRICS');
    console.log('═'.repeat(60));

    const learningMetrics = await search.getLearningMetrics();

    console.log(`\n📊 Database Stats:`);
    console.log(`   Content Items: ${learningMetrics.contentCount}`);
    console.log(`   Users: ${learningMetrics.usersCount}`);
    console.log(`   Total Feedback: ${learningMetrics.totalFeedback}`);

    console.log(`\n🎯 Learning Performance:`);
    console.log(`   Positive Rate: ${(learningMetrics.positiveRate * 100).toFixed(1)}%`);
    console.log(`   Average Reward: ${learningMetrics.avgReward.toFixed(3)}`);

    console.log(`\n📈 Pattern Performance:`);
    console.log('   ┌─────────────────────┬──────────────┬───────┐');
    console.log('   │ Pattern Type        │ Success Rate │ Uses  │');
    console.log('   ├─────────────────────┼──────────────┼───────┤');
    for (const p of learningMetrics.patternStats) {
      console.log(`   │ ${p.type.padEnd(19)} │ ${(p.successRate * 100).toFixed(1).padStart(10)}% │ ${p.uses.toString().padStart(5)} │`);
    }
    console.log('   └─────────────────────┴──────────────┴───────┘');

    // Search performance summary
    console.log('\n' + '═'.repeat(60));
    console.log('  SEARCH PERFORMANCE SUMMARY');
    console.log('═'.repeat(60));

    console.log('\n   ┌────────────┬─────────┬───────────┬──────────┬─────────┐');
    console.log('   │ Query      │ Results │ Search ms │ Embed ms │ Rec ms  │');
    console.log('   ├────────────┼─────────┼───────────┼──────────┼─────────┤');
    for (const m of allMetrics) {
      console.log(`   │ ${m.searchQuery.padEnd(10)} │ ${m.tvdbResults.toString().padStart(7)} │ ${m.searchTimeMs.toString().padStart(9)} │ ${m.embeddingTimeMs.toString().padStart(8)} │ ${m.recommendationTimeMs.toString().padStart(7)} │`);
    }
    console.log('   └────────────┴─────────┴───────────┴──────────┴─────────┘');

    const avgSearch = allMetrics.reduce((a, m) => a + m.searchTimeMs, 0) / allMetrics.length;
    const avgEmbed = allMetrics.reduce((a, m) => a + m.embeddingTimeMs, 0) / allMetrics.length;
    const avgRec = allMetrics.reduce((a, m) => a + m.recommendationTimeMs, 0) / allMetrics.length;
    const avgTotal = allMetrics.reduce((a, m) => a + m.totalTimeMs, 0) / allMetrics.length;

    console.log(`\n   Average Search Time: ${avgSearch.toFixed(1)}ms`);
    console.log(`   Average Embedding Time: ${avgEmbed.toFixed(1)}ms`);
    console.log(`   Average Recommendation Time: ${avgRec.toFixed(1)}ms`);
    console.log(`   Average Total Time: ${avgTotal.toFixed(1)}ms`);

    console.log('\n' + '═'.repeat(60));
    console.log('  ✅ Demo Complete - System is learning from each interaction!');
    console.log('═'.repeat(60) + '\n');

  } catch (error) {
    console.error('Error:', error);
    process.exit(1);
  } finally {
    await search.close();
  }
}

main();
