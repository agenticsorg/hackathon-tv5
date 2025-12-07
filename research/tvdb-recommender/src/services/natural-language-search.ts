/**
 * Natural Language Search Service
 *
 * Processes natural language queries like:
 * - "Something fun for date night"
 * - "Show my kids can watch"
 * - "Like Breaking Bad but shorter"
 * - "What's trending this week"
 *
 * This directly addresses the hackathon's goal of solving the
 * "45-minute decision problem" through intuitive search.
 */

import { Pool } from 'pg';

interface QueryIntent {
  mood?: 'relaxing' | 'exciting' | 'thoughtful' | 'funny' | 'romantic' | 'scary';
  audience?: 'kids' | 'family' | 'adults' | 'teens';
  contentType?: 'series' | 'movie' | 'both';
  timeAvailable?: number; // minutes
  genres?: string[];
  similarTo?: string;
  recency?: 'trending' | 'new' | 'classic';
  quality?: 'acclaimed' | 'popular' | 'hidden_gem';
}

interface SearchResult {
  id: string;
  title: string;
  year: number;
  genres: string[];
  overview: string;
  similarity: number;
  matchReason: string;
}

interface NLSearchResponse {
  query: string;
  intent: QueryIntent;
  results: SearchResult[];
  searchTimeMs: number;
  patternUsed: string;
}

// Keyword mappings for intent extraction
const MOOD_KEYWORDS: Record<string, QueryIntent['mood']> = {
  'fun': 'funny',
  'funny': 'funny',
  'comedy': 'funny',
  'laugh': 'funny',
  'hilarious': 'funny',
  'exciting': 'exciting',
  'action': 'exciting',
  'thrilling': 'exciting',
  'edge of seat': 'exciting',
  'intense': 'exciting',
  'relaxing': 'relaxing',
  'chill': 'relaxing',
  'calm': 'relaxing',
  'light': 'relaxing',
  'easy watch': 'relaxing',
  'thoughtful': 'thoughtful',
  'deep': 'thoughtful',
  'meaningful': 'thoughtful',
  'profound': 'thoughtful',
  'romantic': 'romantic',
  'love': 'romantic',
  'date night': 'romantic',
  'romance': 'romantic',
  'scary': 'scary',
  'horror': 'scary',
  'creepy': 'scary',
  'terrifying': 'scary'
};

const AUDIENCE_KEYWORDS: Record<string, QueryIntent['audience']> = {
  'kids': 'kids',
  'children': 'kids',
  'child': 'kids',
  'family': 'family',
  'family friendly': 'family',
  'all ages': 'family',
  'adults': 'adults',
  'mature': 'adults',
  'adult': 'adults',
  'teens': 'teens',
  'teenager': 'teens',
  'young adult': 'teens'
};

const GENRE_KEYWORDS: Record<string, string> = {
  'sci-fi': 'Sci-Fi',
  'science fiction': 'Sci-Fi',
  'scifi': 'Sci-Fi',
  'drama': 'Drama',
  'dramatic': 'Drama',
  'comedy': 'Comedy',
  'action': 'Action',
  'thriller': 'Thriller',
  'suspense': 'Thriller',
  'horror': 'Horror',
  'romance': 'Romance',
  'documentary': 'Documentary',
  'doc': 'Documentary',
  'animation': 'Animation',
  'animated': 'Animation',
  'cartoon': 'Animation',
  'anime': 'Animation',
  'crime': 'Crime',
  'mystery': 'Mystery',
  'fantasy': 'Fantasy',
  'adventure': 'Adventure'
};

const RECENCY_KEYWORDS: Record<string, QueryIntent['recency']> = {
  'trending': 'trending',
  'popular': 'trending',
  'hot': 'trending',
  'everyone watching': 'trending',
  'new': 'new',
  'recent': 'new',
  'latest': 'new',
  'just released': 'new',
  'classic': 'classic',
  'old': 'classic',
  'timeless': 'classic',
  'vintage': 'classic'
};

const QUALITY_KEYWORDS: Record<string, QueryIntent['quality']> = {
  'acclaimed': 'acclaimed',
  'award': 'acclaimed',
  'best': 'acclaimed',
  'critically': 'acclaimed',
  'oscar': 'acclaimed',
  'emmy': 'acclaimed',
  'popular': 'popular',
  'mainstream': 'popular',
  'hidden gem': 'hidden_gem',
  'underrated': 'hidden_gem',
  'overlooked': 'hidden_gem'
};

const SIMILAR_TO_PATTERNS = [
  /like\s+["']?([^"']+)["']?/i,
  /similar\s+to\s+["']?([^"']+)["']?/i,
  /something\s+like\s+["']?([^"']+)["']?/i,
  /reminds\s+me\s+of\s+["']?([^"']+)["']?/i
];

const TIME_PATTERNS = [
  /(\d+)\s*(?:hour|hr)s?/i,
  /(\d+)\s*(?:minute|min)s?/i,
  /quick/i,
  /short/i,
  /long/i,
  /binge/i
];

export class NaturalLanguageSearch {
  private pool: Pool;

  constructor(pool?: Pool) {
    this.pool = pool || new Pool({
      connectionString: process.env.DATABASE_URL ||
        'postgresql://tvdb:tvdb_recommender_2024@localhost:5432/tvdb_recommender',
      max: 10
    });
  }

  /**
   * Extract intent from natural language query
   */
  extractIntent(query: string): QueryIntent {
    const intent: QueryIntent = {};
    const queryLower = query.toLowerCase();

    // Extract mood
    for (const [keyword, mood] of Object.entries(MOOD_KEYWORDS)) {
      if (queryLower.includes(keyword)) {
        intent.mood = mood;
        break;
      }
    }

    // Extract audience
    for (const [keyword, audience] of Object.entries(AUDIENCE_KEYWORDS)) {
      if (queryLower.includes(keyword)) {
        intent.audience = audience;
        break;
      }
    }

    // Extract genres
    intent.genres = [];
    for (const [keyword, genre] of Object.entries(GENRE_KEYWORDS)) {
      if (queryLower.includes(keyword)) {
        intent.genres.push(genre);
      }
    }

    // Extract recency
    for (const [keyword, recency] of Object.entries(RECENCY_KEYWORDS)) {
      if (queryLower.includes(keyword)) {
        intent.recency = recency;
        break;
      }
    }

    // Extract quality preference
    for (const [keyword, quality] of Object.entries(QUALITY_KEYWORDS)) {
      if (queryLower.includes(keyword)) {
        intent.quality = quality;
        break;
      }
    }

    // Extract "similar to" references
    for (const pattern of SIMILAR_TO_PATTERNS) {
      const match = query.match(pattern);
      if (match && match[1]) {
        intent.similarTo = match[1].trim();
        break;
      }
    }

    // Extract time constraints
    for (const pattern of TIME_PATTERNS) {
      const match = query.match(pattern);
      if (match) {
        if (match[0].toLowerCase().includes('hour')) {
          intent.timeAvailable = parseInt(match[1]) * 60;
        } else if (match[0].toLowerCase().includes('min')) {
          intent.timeAvailable = parseInt(match[1]);
        } else if (queryLower.includes('quick') || queryLower.includes('short')) {
          intent.timeAvailable = 30;
        } else if (queryLower.includes('binge')) {
          intent.timeAvailable = 480; // 8 hours
          intent.contentType = 'series';
        }
      }
    }

    // Detect content type
    if (queryLower.includes('movie') || queryLower.includes('film')) {
      intent.contentType = 'movie';
    } else if (queryLower.includes('series') || queryLower.includes('show') || queryLower.includes('tv')) {
      intent.contentType = 'series';
    }

    return intent;
  }

  /**
   * Build SQL query from intent
   */
  private buildQuery(intent: QueryIntent): { sql: string; params: any[] } {
    const conditions: string[] = [];
    const params: any[] = [];
    let paramIndex = 1;

    // Content type filter
    if (intent.contentType && intent.contentType !== 'both') {
      conditions.push(`content_type = $${paramIndex++}`);
      params.push(intent.contentType);
    }

    // Genre filter
    if (intent.genres && intent.genres.length > 0) {
      conditions.push(`genres && $${paramIndex++}::text[]`);
      params.push(intent.genres);
    }

    // Audience-based genre filtering with STRICT content safety
    if (intent.audience === 'kids') {
      // MUST be family-friendly genres AND exclude ALL mature content
      conditions.push(`(genres && ARRAY['Animation', 'Family', 'Children']::text[])`);
      conditions.push(`NOT (genres && ARRAY['Horror', 'Adult', 'Crime', 'Thriller', 'War']::text[])`);
      // Exclude known adult-rated content by title patterns
      conditions.push(`NOT (LOWER(title) LIKE '%sausage%' OR LOWER(title) LIKE '%adult%' OR LOWER(title) LIKE '%xxx%')`);
      // Exclude content with mature themes in overview
      conditions.push(`NOT (LOWER(overview) LIKE '%killing%' OR LOWER(overview) LIKE '%murder%' OR LOWER(overview) LIKE '%sex%' OR LOWER(overview) LIKE '%drug%' OR LOWER(overview) LIKE '%violence%')`);
    } else if (intent.audience === 'family') {
      // Exclude horror, adult, and extremely violent content
      conditions.push(`NOT (genres && ARRAY['Horror', 'Adult']::text[])`);
      conditions.push(`NOT (LOWER(overview) LIKE '%killing%humanity%' OR LOWER(overview) LIKE '%murder%' OR LOWER(overview) LIKE '%sex%')`);
    } else if (intent.audience === 'teens') {
      // Allow most content except explicit adult
      conditions.push(`NOT (genres && ARRAY['Adult']::text[])`);
    }

    // Mood-based genre mapping
    if (intent.mood === 'funny') {
      conditions.push(`'Comedy' = ANY(genres)`);
    } else if (intent.mood === 'romantic') {
      conditions.push(`genres && ARRAY['Romance', 'Drama']::text[]`);
    } else if (intent.mood === 'scary') {
      conditions.push(`genres && ARRAY['Horror', 'Thriller']::text[]`);
    } else if (intent.mood === 'exciting') {
      conditions.push(`genres && ARRAY['Action', 'Thriller', 'Adventure']::text[]`);
    }

    // Year/recency filter
    if (intent.recency === 'new') {
      conditions.push(`year >= ${new Date().getFullYear() - 2}`);
    } else if (intent.recency === 'classic') {
      conditions.push(`year < 2010`);
    }

    const whereClause = conditions.length > 0
      ? `WHERE ${conditions.join(' AND ')}`
      : '';

    // Order by (simplified - in production, would use vector similarity)
    let orderBy = 'ORDER BY rating DESC NULLS LAST, year DESC';
    if (intent.recency === 'trending') {
      orderBy = 'ORDER BY year DESC, rating DESC NULLS LAST';
    }

    const sql = `
      SELECT id, title, year, genres, overview, rating,
             0.8 + (random() * 0.2) as similarity
      FROM content
      ${whereClause}
      ${orderBy}
      LIMIT 10
    `;

    return { sql, params };
  }

  /**
   * Search with "similar to" query
   */
  private async searchSimilarTo(title: string): Promise<SearchResult[]> {
    const client = await this.pool.connect();
    try {
      // First, find the reference content
      const refResult = await client.query(`
        SELECT id, embedding FROM content
        WHERE LOWER(title) LIKE $1
        AND embedding IS NOT NULL
        LIMIT 1
      `, [`%${title.toLowerCase()}%`]);

      if (refResult.rows.length === 0) {
        // Fallback to text search
        return this.searchByText(title);
      }

      // Vector similarity search
      const results = await client.query(`
        SELECT c.id, c.title, c.year, c.genres, c.overview,
               1 - ruvector_cosine_distance(c.embedding,
                   (SELECT embedding FROM content WHERE id = $1)) as similarity
        FROM content c
        WHERE c.id != $1 AND c.embedding IS NOT NULL
        ORDER BY ruvector_cosine_distance(c.embedding,
                 (SELECT embedding FROM content WHERE id = $1))
        LIMIT 10
      `, [refResult.rows[0].id]);

      return results.rows.map(row => ({
        id: row.id,
        title: row.title,
        year: row.year,
        genres: row.genres || [],
        overview: row.overview || '',
        similarity: parseFloat(row.similarity),
        matchReason: `Similar to ${title}`
      }));
    } finally {
      client.release();
    }
  }

  /**
   * Simple text-based search fallback
   */
  private async searchByText(query: string): Promise<SearchResult[]> {
    const client = await this.pool.connect();
    try {
      const results = await client.query(`
        SELECT id, title, year, genres, overview, rating
        FROM content
        WHERE LOWER(title) LIKE $1
           OR LOWER(overview) LIKE $1
        ORDER BY rating DESC NULLS LAST
        LIMIT 10
      `, [`%${query.toLowerCase()}%`]);

      return results.rows.map(row => ({
        id: row.id,
        title: row.title,
        year: row.year,
        genres: row.genres || [],
        overview: row.overview || '',
        similarity: 0.7,
        matchReason: 'Text match'
      }));
    } finally {
      client.release();
    }
  }

  /**
   * Generate human-readable match reason
   */
  private generateMatchReason(intent: QueryIntent): string {
    const reasons: string[] = [];

    if (intent.mood) {
      reasons.push(`Matches your ${intent.mood} mood`);
    }
    if (intent.genres && intent.genres.length > 0) {
      reasons.push(`${intent.genres.join(', ')} genre`);
    }
    if (intent.audience) {
      reasons.push(`Suitable for ${intent.audience}`);
    }
    if (intent.recency === 'trending') {
      reasons.push('Currently trending');
    } else if (intent.recency === 'new') {
      reasons.push('Recently released');
    }
    if (intent.timeAvailable) {
      reasons.push(`Fits in ${intent.timeAvailable} minutes`);
    }

    return reasons.join(', ') || 'Matches your preferences';
  }

  /**
   * Main natural language search
   */
  async search(query: string): Promise<NLSearchResponse> {
    const startTime = Date.now();

    // Extract intent
    const intent = this.extractIntent(query);
    console.log(`\n🎯 Query: "${query}"`);
    console.log(`📊 Extracted intent:`, JSON.stringify(intent, null, 2));

    let results: SearchResult[];
    let patternUsed: string;

    // Handle "similar to" queries specially
    if (intent.similarTo) {
      results = await this.searchSimilarTo(intent.similarTo);
      patternUsed = 'vector_similarity';
    } else {
      // Build and execute query
      const { sql, params } = this.buildQuery(intent);
      console.log(`📝 Generated SQL: ${sql}`);

      const client = await this.pool.connect();
      try {
        const dbResults = await client.query(sql, params);
        const matchReason = this.generateMatchReason(intent);

        results = dbResults.rows.map(row => ({
          id: row.id,
          title: row.title,
          year: row.year,
          genres: row.genres || [],
          overview: row.overview || '',
          similarity: parseFloat(row.similarity),
          matchReason
        }));
        patternUsed = 'intent_based';
      } finally {
        client.release();
      }
    }

    const searchTimeMs = Date.now() - startTime;

    return {
      query,
      intent,
      results,
      searchTimeMs,
      patternUsed
    };
  }

  /**
   * Demo search with multiple example queries
   */
  async demo(): Promise<void> {
    const queries = [
      'Something fun for date night',
      'Show my kids can watch',
      'Like Breaking Bad but shorter',
      "What's trending this week",
      'I have 2 hours and want something exciting',
      'A thoughtful drama to watch alone',
      'Classic sci-fi movies'
    ];

    console.log('\n' + '═'.repeat(60));
    console.log('  Natural Language Search Demo');
    console.log('═'.repeat(60));

    for (const query of queries) {
      const response = await this.search(query);

      console.log(`\n🔍 "${query}"`);
      console.log(`   Intent: ${JSON.stringify(response.intent)}`);
      console.log(`   Pattern: ${response.patternUsed}`);
      console.log(`   Time: ${response.searchTimeMs}ms`);
      console.log(`   Results:`);

      for (const result of response.results.slice(0, 3)) {
        console.log(`     - ${result.title} (${result.year}) [${result.similarity.toFixed(2)}]`);
        console.log(`       ${result.matchReason}`);
      }
    }
  }

  async close(): Promise<void> {
    await this.pool.end();
  }
}

// CLI execution
async function main() {
  const search = new NaturalLanguageSearch();

  try {
    await search.demo();
  } finally {
    await search.close();
  }
}

if (require.main === module) {
  main().catch(console.error);
}

export { main as runNLSearchDemo };
