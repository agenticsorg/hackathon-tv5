import { Pool } from 'pg';

const pool = new Pool({
  connectionString: process.env.DATABASE_URL ||
    'postgresql://tvdb:tvdb_recommender_2024@localhost:5432/tvdb_recommender',
  max: 10,
});

export interface Content {
  id: string;
  title: string;
  year: number;
  content_type: 'series' | 'movie';
  genres: string[];
  overview: string;
  rating: number;
  network_name: string;
  original_language: string;
  image_url: string;
  similarity?: number;
}

export interface SearchResult {
  id: string;
  title: string;
  year: number;
  genres: string[];
  overview: string;
  similarity: number;
  matchReason: string;
}

export interface LearningStats {
  totalContent: number;
  totalSeries: number;
  totalMovies: number;
  totalPatterns: number;
  avgSuccessRate: number;
  totalFeedback: number;
  topPatterns: Array<{ type: string; rate: number; uses: number }>;
  languageDistribution: Array<{ language: string; count: number }>;
}

export async function searchContent(query: string, limit = 20): Promise<Content[]> {
  const client = await pool.connect();
  try {
    // Extract keywords from natural language query
    const keywords = query.toLowerCase()
      .replace(/something|like|for|the|and|but|with|can|watch|tonight|night|my|good|great|best|series|movie|show/gi, '')
      .split(/\s+/)
      .filter(w => w.length > 2);

    // Map natural language to genres/filters
    const genreMap: Record<string, string[]> = {
      'fun': ['Comedy'],
      'funny': ['Comedy'],
      'date': ['Romance', 'Comedy'],
      'romantic': ['Romance'],
      'scary': ['Horror', 'Thriller'],
      'horror': ['Horror'],
      'action': ['Action'],
      'exciting': ['Action', 'Thriller', 'Adventure'],
      'drama': ['Drama'],
      'french': [],  // language filter
      'kids': ['Animation', 'Family'],
      'children': ['Animation', 'Family'],
      'family': ['Family', 'Animation'],
      'documentary': ['Documentary'],
      'crime': ['Crime'],
      'thriller': ['Thriller'],
      'breaking': ['Crime', 'Drama'],
      'bad': ['Crime', 'Drama'],
      'trending': [],  // sort by year
    };

    let genres: string[] = [];
    let languageFilter: string | null = null;
    let sortByYear = false;

    for (const keyword of keywords) {
      if (keyword === 'french' || keyword === 'france' || keyword === 'fra') {
        languageFilter = 'fra';
      } else if (keyword === 'trending' || keyword === 'new' || keyword === 'latest') {
        sortByYear = true;
      } else if (genreMap[keyword]) {
        genres.push(...genreMap[keyword]);
      }
    }

    // Build query conditions (allow NULL embeddings for initial content)
    let conditions: string[] = ["title NOT LIKE 'Test Content%'"];
    let params: any[] = [];
    let paramIdx = 1;

    // CONTENT SAFETY - Check if this is a kids/family search
    const isKidsSearch = keywords.some(k => ['kids', 'children', 'child', 'family', 'animated', 'cartoon'].includes(k));

    if (isKidsSearch) {
      // Strict filtering for kids/family searches - no R-rated content
      conditions.push(`NOT (LOWER(title) LIKE '%sausage party%')`);
      conditions.push(`content_rating IN ('G', 'PG') OR content_rating IS NULL`);
      conditions.push(`NOT (genres && ARRAY['Horror', 'Crime', 'Thriller', 'War', 'Adult']::text[])`);
      conditions.push(`NOT (LOWER(overview) LIKE '%kill%' OR LOWER(overview) LIKE '%murder%' OR LOWER(overview) LIKE '%sex%' OR LOWER(overview) LIKE '%violence%')`);
    }
    // For adult searches, allow R-rated content but still exclude known problematic content when searching animation
    const isAnimationSearch = keywords.some(k => ['animation', 'anime', 'cartoon'].includes(k));
    if (isAnimationSearch && !isKidsSearch) {
      // Exclude R-rated animation masquerading as kids content (like Sausage Party)
      conditions.push(`NOT (content_rating = 'R' AND genres && ARRAY['Animation']::text[] AND LOWER(title) LIKE '%sausage party%')`);
    }

    // Text search on title/overview with FUZZY matching using pg_trgm
    const searchTerms = keywords.filter(k => !genreMap[k] && k !== 'french' && k !== 'trending');
    let fuzzyTermForOrder: string | null = null;
    if (searchTerms.length > 0) {
      const searchPattern = searchTerms.map(t => `%${t}%`).join('%');
      const fuzzyTerm = searchTerms.join(' ');
      fuzzyTermForOrder = fuzzyTerm.toLowerCase();

      // Use trigram similarity for fuzzy matching (handles "starwars" -> "star wars")
      // Combined with LIKE for exact substring matches
      conditions.push(`(
        LOWER(title) LIKE $${paramIdx}
        OR LOWER(overview) LIKE $${paramIdx}
        OR similarity(LOWER(title), $${paramIdx + 1}) > 0.3
        OR LOWER(REPLACE(title, ' ', '')) LIKE $${paramIdx + 2}
      )`);
      params.push(`%${searchPattern}%`);  // For LIKE
      params.push(fuzzyTerm.toLowerCase());  // For trigram similarity
      params.push(`%${fuzzyTerm.toLowerCase().replace(/\s+/g, '')}%`);  // For no-space match
      paramIdx += 3;
    }

    // Genre filter
    if (genres.length > 0) {
      conditions.push(`genres && $${paramIdx}::text[]`);
      params.push(Array.from(new Set(genres)));
      paramIdx++;
    }

    // Language filter
    if (languageFilter) {
      conditions.push(`(original_language = $${paramIdx} OR overview ILIKE '%french%' OR overview ILIKE '%france%')`);
      params.push(languageFilter);
      paramIdx++;
    }

    // Order by similarity when we have search terms, otherwise by year
    let orderBy: string;
    if (fuzzyTermForOrder) {
      // Add the fuzzy term for ORDER BY similarity calculation
      params.push(fuzzyTermForOrder);
      const similarityParamIdx = paramIdx;
      paramIdx++;
      orderBy = `similarity(LOWER(title), $${similarityParamIdx}) DESC, year DESC NULLS LAST, rating DESC NULLS LAST`;
    } else if (sortByYear) {
      orderBy = 'year DESC NULLS LAST, rating DESC NULLS LAST';
    } else {
      orderBy = `CASE WHEN title LIKE 'Test Content%' THEN 1 ELSE 0 END, year DESC NULLS LAST, rating DESC NULLS LAST`;
    }

    // Add limit as the final parameter
    params.push(limit);
    const limitParamIdx = paramIdx;

    const result = await client.query(`
      SELECT id, title, year, content_type, genres, overview, rating,
             network_name, original_language, image_url
      FROM content
      WHERE ${conditions.join(' AND ')}
      ORDER BY ${orderBy}
      LIMIT $${limitParamIdx}
    `, params);

    return result.rows;
  } finally {
    client.release();
  }
}

export async function getRecommendations(mood?: string, contentType?: string, limit = 20): Promise<Content[]> {
  const client = await pool.connect();
  try {
    let whereConditions = [
      'embedding IS NOT NULL',
      // CONTENT SAFETY - Exclude R-rated and known adult content
      `NOT (LOWER(title) LIKE '%sausage party%')`,
      `(content_rating != 'R' OR content_rating IS NULL)`
    ];
    const params: any[] = [];
    let paramIndex = 1;

    if (contentType && contentType !== 'both') {
      whereConditions.push(`content_type = $${paramIndex++}`);
      params.push(contentType);
    }

    if (mood) {
      const moodGenres: Record<string, string[]> = {
        funny: ['Comedy'],
        exciting: ['Action', 'Thriller', 'Adventure'],
        romantic: ['Romance', 'Drama'],
        scary: ['Horror', 'Thriller'],
        thoughtful: ['Drama', 'Documentary'],
        relaxing: ['Comedy', 'Animation'],
      };

      const genres = moodGenres[mood] || [];
      if (genres.length > 0) {
        whereConditions.push(`genres && $${paramIndex++}::text[]`);
        params.push(genres);
      }

      // Extra safety for relaxing (Animation included) - exclude violent content
      if (mood === 'relaxing') {
        whereConditions.push(`NOT (LOWER(overview) LIKE '%kill%' OR LOWER(overview) LIKE '%murder%')`);
      }
    }

    params.push(limit);

    const result = await client.query(`
      SELECT id, title, year, content_type, genres, overview, rating,
             network_name, original_language, image_url
      FROM content
      WHERE ${whereConditions.join(' AND ')}
      ORDER BY
        CASE WHEN title LIKE 'Test Content%' THEN 1 ELSE 0 END,
        year DESC NULLS LAST,
        rating DESC NULLS LAST
      LIMIT $${paramIndex}
    `, params);

    return result.rows;
  } finally {
    client.release();
  }
}

export async function getSimilarContent(contentId: string, limit = 10): Promise<Content[]> {
  const client = await pool.connect();
  try {
    const result = await client.query(`
      SELECT c.id, c.title, c.year, c.content_type, c.genres, c.overview, c.rating,
             c.network_name, c.original_language, c.image_url,
             1 - ruvector_cosine_distance(c.embedding,
                 (SELECT embedding FROM content WHERE id = $1)) as similarity
      FROM content c
      WHERE c.id != $1
        AND c.embedding IS NOT NULL
        AND NOT (LOWER(c.title) LIKE '%sausage party%')
        AND (c.content_rating != 'R' OR c.content_rating IS NULL)
      ORDER BY ruvector_cosine_distance(c.embedding,
               (SELECT embedding FROM content WHERE id = $1))
      LIMIT $2
    `, [contentId, limit]);

    return result.rows;
  } finally {
    client.release();
  }
}

export async function getLearningStats(): Promise<LearningStats> {
  const client = await pool.connect();
  try {
    const [contentStats, patternStats, feedbackStats, languageStats] = await Promise.all([
      client.query(`
        SELECT
          COUNT(*) as total,
          SUM(CASE WHEN content_type = 'series' THEN 1 ELSE 0 END) as series,
          SUM(CASE WHEN content_type = 'movie' THEN 1 ELSE 0 END) as movies
        FROM content
      `),
      client.query(`
        SELECT pattern_type, success_rate, total_uses
        FROM recommendation_patterns
        ORDER BY success_rate DESC
        LIMIT 10
      `),
      client.query(`
        SELECT
          COUNT(*) as total,
          AVG(CASE WHEN was_successful THEN 1.0 ELSE 0.0 END) as success_rate
        FROM learning_feedback
      `),
      client.query(`
        SELECT original_language as language, COUNT(*) as count
        FROM content
        WHERE original_language IS NOT NULL
        GROUP BY original_language
        ORDER BY count DESC
        LIMIT 10
      `),
    ]);

    const content = contentStats.rows[0];
    const feedback = feedbackStats.rows[0];

    return {
      totalContent: parseInt(content.total),
      totalSeries: parseInt(content.series),
      totalMovies: parseInt(content.movies),
      totalPatterns: patternStats.rows.length,
      avgSuccessRate: parseFloat(feedback.success_rate) || 0,
      totalFeedback: parseInt(feedback.total) || 0,
      topPatterns: patternStats.rows.map(p => ({
        type: p.pattern_type,
        rate: parseFloat(p.success_rate),
        uses: parseInt(p.total_uses),
      })),
      languageDistribution: languageStats.rows.map(l => ({
        language: l.language,
        count: parseInt(l.count),
      })),
    };
  } finally {
    client.release();
  }
}

export async function recordFeedback(
  contentId: string,
  wasSuccessful: boolean,
  patternId?: number
): Promise<void> {
  const client = await pool.connect();
  try {
    await client.query(`
      INSERT INTO users (id, email)
      VALUES ('00000000-0000-0000-0000-000000000001', 'web@user.com')
      ON CONFLICT DO NOTHING
    `);

    const reward = wasSuccessful ? (0.5 + Math.random() * 0.5) : (-0.5 + Math.random() * 0.5);

    await client.query(`
      INSERT INTO learning_feedback (user_id, content_id, pattern_id, was_successful, reward, user_action)
      VALUES ($1, $2, $3, $4, $5, $6)
    `, [
      '00000000-0000-0000-0000-000000000001',
      contentId,
      patternId || 1,
      wasSuccessful,
      reward,
      wasSuccessful ? 'watched' : 'skipped',
    ]);

    if (patternId) {
      await client.query(`
        UPDATE recommendation_patterns
        SET success_rate = (
          SELECT AVG(CASE WHEN was_successful THEN 1.0 ELSE 0.0 END)
          FROM learning_feedback
          WHERE pattern_id = $1
        ),
        total_uses = total_uses + 1,
        updated_at = NOW()
        WHERE id = $1
      `, [patternId]);
    }
  } finally {
    client.release();
  }
}

export default pool;
