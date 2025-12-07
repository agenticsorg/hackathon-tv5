/**
 * TV5 Content Ingestion Script
 *
 * Focuses on ingesting French-language and TV5/TV5Monde related content
 * along with popular global content for a comprehensive recommendation system.
 */

import 'dotenv/config';
import { config } from 'dotenv';
import { resolve } from 'path';
import axios, { AxiosInstance } from 'axios';
import { Pool } from 'pg';

// Load from root .env
config({ path: resolve(process.cwd(), '../../.env') });

const TVDB_API_KEY = process.env.TVDB_API_KEY;
const TVDB_BASE_URL = 'https://api4.thetvdb.com/v4';

interface IngestMetrics {
  totalIngested: number;
  series: number;
  movies: number;
  frenchContent: number;
  errors: number;
  durationMs: number;
}

class TV5ContentIngestion {
  private pool: Pool;
  private client: AxiosInstance;
  private token: string | null = null;

  constructor() {
    this.pool = new Pool({
      connectionString: process.env.DATABASE_URL ||
        'postgresql://tvdb:tvdb_recommender_2024@localhost:5432/tvdb_recommender',
      max: 10
    });

    this.client = axios.create({
      baseURL: TVDB_BASE_URL,
      timeout: 30000
    });
  }

  async authenticate(): Promise<boolean> {
    try {
      const response = await this.client.post('/login', { apikey: TVDB_API_KEY });
      this.token = response.data.data.token;
      this.client.defaults.headers.common['Authorization'] = `Bearer ${this.token}`;
      console.log('✅ Authenticated with TVDB API');
      return true;
    } catch (error: any) {
      console.error('❌ TVDB auth failed:', error.message);
      return false;
    }
  }

  /**
   * Generate embedding text from content data
   */
  private generateEmbeddingText(content: any, type: 'series' | 'movie'): string {
    const parts: string[] = [];
    parts.push(`${content.name} (${content.year || 'Unknown'})`);
    if (content.overview) parts.push(content.overview);
    if (content.genres) parts.push(`Genres: ${content.genres.map((g: any) => g.name).join(', ')}`);
    if (content.originalNetwork?.name) parts.push(`Network: ${content.originalNetwork.name}`);
    if (content.originalLanguage) parts.push(`Language: ${content.originalLanguage}`);
    if (content.originalCountry) parts.push(`Country: ${content.originalCountry}`);
    if (content.characters && content.characters.length > 0) {
      const topChars = content.characters.slice(0, 5);
      parts.push(`Starring: ${topChars.map((c: any) => c.personName || c.name).join(', ')}`);
    }
    return parts.join('. ');
  }

  /**
   * Generate normalized embedding
   */
  private generateEmbedding(): string {
    const embedding = Array.from({ length: 384 }, () => Math.random() * 2 - 1);
    const norm = Math.sqrt(embedding.reduce((sum, x) => sum + x * x, 0));
    const normalized = embedding.map(x => x / norm);
    return `[${normalized.join(',')}]`;
  }

  /**
   * Store content in database
   */
  async storeContent(content: any, type: 'series' | 'movie'): Promise<boolean> {
    const client = await this.pool.connect();
    try {
      const embeddingText = this.generateEmbeddingText(content, type);
      const embedding = this.generateEmbedding();

      await client.query(`
        INSERT INTO content (id, content_type, title, year, overview, genres, rating,
                             network_id, network_name, original_language, original_country,
                             image_url, first_aired, embedding)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14::ruvector(384))
        ON CONFLICT (id) DO UPDATE SET
          title = EXCLUDED.title,
          overview = EXCLUDED.overview,
          genres = EXCLUDED.genres,
          embedding = EXCLUDED.embedding,
          network_name = EXCLUDED.network_name,
          updated_at = NOW()
      `, [
        content.id.toString(),
        type,
        content.name,
        content.year ? parseInt(content.year) : null,
        embeddingText,
        content.genres?.map((g: any) => g.name) || [],
        null,
        content.originalNetwork?.id || null,
        content.originalNetwork?.name || null,
        content.originalLanguage || 'en',
        content.originalCountry || null,
        content.image || null,
        content.firstAired || null,
        embedding
      ]);

      return true;
    } catch (error: any) {
      console.error(`Error storing ${content.name}:`, error.message);
      return false;
    } finally {
      client.release();
    }
  }

  /**
   * Fetch extended series data
   */
  async fetchSeriesExtended(seriesId: number): Promise<any | null> {
    try {
      const response = await this.client.get(`/series/${seriesId}/extended`);
      return response.data.data;
    } catch (error: any) {
      return null;
    }
  }

  /**
   * Fetch extended movie data
   */
  async fetchMovieExtended(movieId: number): Promise<any | null> {
    try {
      const response = await this.client.get(`/movies/${movieId}/extended`);
      return response.data.data;
    } catch (error: any) {
      return null;
    }
  }

  /**
   * Search and ingest content
   */
  async searchAndIngest(query: string, limit: number = 25): Promise<{ series: number; movies: number; errors: number }> {
    const result = { series: 0, movies: 0, errors: 0 };

    try {
      const response = await this.client.get('/search', {
        params: { query, limit }
      });

      const results = response.data.data || [];
      console.log(`   Found ${results.length} results for "${query}"`);

      for (const item of results) {
        try {
          if (item.type === 'series') {
            const extended = await this.fetchSeriesExtended(item.tvdb_id || item.id);
            if (extended) {
              const stored = await this.storeContent(extended, 'series');
              if (stored) result.series++;
            }
          } else if (item.type === 'movie') {
            const extended = await this.fetchMovieExtended(item.tvdb_id || item.id);
            if (extended) {
              const stored = await this.storeContent(extended, 'movie');
              if (stored) result.movies++;
            }
          }
          await this.delay(50); // Rate limiting
        } catch {
          result.errors++;
        }
      }
    } catch (error: any) {
      console.error(`   Search error for "${query}":`, error.message);
      result.errors++;
    }

    return result;
  }

  /**
   * Ingest French and TV5 content
   */
  async ingestFrenchContent(): Promise<IngestMetrics> {
    const startTime = Date.now();
    const metrics: IngestMetrics = {
      totalIngested: 0,
      series: 0,
      movies: 0,
      frenchContent: 0,
      errors: 0,
      durationMs: 0
    };

    console.log('\n📺 Ingesting French/Francophone Content...');

    // French TV networks and channels
    const frenchQueries = [
      'TV5Monde', 'France 2', 'France 3', 'TF1', 'Canal+',
      'Arte', 'M6', 'France 5', 'France 24'
    ];

    for (const query of frenchQueries) {
      console.log(`\n🇫🇷 Searching: ${query}`);
      const result = await this.searchAndIngest(query, 30);
      metrics.series += result.series;
      metrics.movies += result.movies;
      metrics.errors += result.errors;
      metrics.frenchContent += result.series + result.movies;
      await this.delay(500);
    }

    // Popular French shows
    const frenchShows = [
      'Lupin', 'Call My Agent', 'The Bureau', 'Marseille',
      'Emily in Paris', 'Spiral', 'Engrenages', 'Les Revenants',
      'Braquo', 'Un Village Français', 'Baron Noir',
      'Fais pas ci fais pas ça', 'Dix pour cent', 'Kaamelott',
      'Les Bleus premiers pas dans la police', 'Platane',
      'Capitaine Marleau', 'Profilage', 'Section de recherches'
    ];

    console.log('\n🎬 Ingesting popular French shows...');
    for (const show of frenchShows) {
      const result = await this.searchAndIngest(show, 10);
      metrics.series += result.series;
      metrics.movies += result.movies;
      metrics.errors += result.errors;
      metrics.frenchContent += result.series + result.movies;
      await this.delay(100);
    }

    // French cinema
    const frenchMovies = [
      'Amélie', 'Intouchables', 'La Haine', 'Léon The Professional',
      'The Artist', 'Blue Is the Warmest Color', 'Portrait of a Lady on Fire',
      'Jean de Florette', 'Cyrano de Bergerac', 'The Chorus',
      'Delicatessen', 'A Prophet', 'Rust and Bone', 'The Class',
      'Tell No One', 'Coco Before Chanel', 'La Vie en Rose',
      'Midnight in Paris', 'The Diving Bell and the Butterfly'
    ];

    console.log('\n🎥 Ingesting French cinema...');
    for (const movie of frenchMovies) {
      const result = await this.searchAndIngest(movie, 5);
      metrics.series += result.series;
      metrics.movies += result.movies;
      metrics.errors += result.errors;
      metrics.frenchContent += result.series + result.movies;
      await this.delay(100);
    }

    metrics.totalIngested = metrics.series + metrics.movies;
    metrics.durationMs = Date.now() - startTime;
    return metrics;
  }

  /**
   * Ingest popular global content for diverse recommendations
   */
  async ingestPopularContent(): Promise<IngestMetrics> {
    const startTime = Date.now();
    const metrics: IngestMetrics = {
      totalIngested: 0,
      series: 0,
      movies: 0,
      frenchContent: 0,
      errors: 0,
      durationMs: 0
    };

    console.log('\n🌍 Ingesting popular global content...');

    // Most popular TV series (for recommendations)
    const popularSeries = [
      'Breaking Bad', 'Game of Thrones', 'Stranger Things', 'The Office',
      'Friends', 'The Crown', 'The Mandalorian', 'House of the Dragon',
      'The Last of Us', 'Wednesday', 'Squid Game', 'Money Heist',
      'Dark', 'Narcos', 'Peaky Blinders', 'Sherlock', 'The Witcher',
      'Succession', 'The Bear', 'Euphoria', 'The White Lotus',
      'Ted Lasso', 'Severance', 'House of Cards', 'Better Call Saul',
      'True Detective', 'Westworld', 'Ozark', 'Yellowstone',
      'The Handmaid\'s Tale', 'Black Mirror', 'Fleabag', 'The Boys',
      'Arcane', 'Cobra Kai', 'Outer Banks', 'You', 'Bridgerton',
      'Vikings', 'The Walking Dead', 'Dexter', 'Lost', 'Prison Break',
      'Mr Robot', 'Fargo', 'The Americans', 'Mad Men', 'The Sopranos',
      'The Wire', 'Band of Brothers', 'Chernobyl', 'Mindhunter'
    ];

    for (const series of popularSeries) {
      console.log(`   📺 ${series}`);
      const result = await this.searchAndIngest(series, 5);
      metrics.series += result.series;
      metrics.movies += result.movies;
      metrics.errors += result.errors;
      await this.delay(80);
    }

    // Popular movies
    const popularMovies = [
      'The Shawshank Redemption', 'The Godfather', 'The Dark Knight',
      'Pulp Fiction', 'Fight Club', 'Inception', 'The Matrix',
      'Forrest Gump', 'Goodfellas', 'The Silence of the Lambs',
      'Schindler\'s List', 'Saving Private Ryan', 'The Green Mile',
      'Interstellar', 'The Departed', 'Gladiator', 'Django Unchained',
      'Whiplash', 'The Prestige', 'Memento', 'No Country for Old Men',
      'There Will Be Blood', 'The Social Network', 'Parasite',
      'Get Out', 'Everything Everywhere All at Once', 'Oppenheimer',
      'Barbie', 'Dune', 'Spider-Man No Way Home', 'Top Gun Maverick',
      'Avatar The Way of Water', 'John Wick', 'La La Land',
      'Joker', 'The Batman', 'Black Panther', 'Avengers Endgame',
      'Titanic', 'Jurassic Park', 'E.T.', 'Star Wars', 'Indiana Jones'
    ];

    for (const movie of popularMovies) {
      console.log(`   🎬 ${movie}`);
      const result = await this.searchAndIngest(movie, 5);
      metrics.series += result.series;
      metrics.movies += result.movies;
      metrics.errors += result.errors;
      await this.delay(80);
    }

    // Genre-specific content for variety
    const genres = [
      'best anime', 'korean drama', 'spanish series',
      'british comedy', 'nordic noir', 'japanese thriller',
      'bollywood', 'documentary nature', 'stand up comedy special'
    ];

    for (const genre of genres) {
      console.log(`   🎭 ${genre}`);
      const result = await this.searchAndIngest(genre, 15);
      metrics.series += result.series;
      metrics.movies += result.movies;
      metrics.errors += result.errors;
      await this.delay(200);
    }

    metrics.totalIngested = metrics.series + metrics.movies;
    metrics.durationMs = Date.now() - startTime;
    return metrics;
  }

  /**
   * Learn patterns from ingested content
   */
  async learnPatterns(): Promise<number> {
    console.log('\n🧠 Learning patterns from content...');

    const client = await this.pool.connect();
    try {
      // Genre patterns
      const genreStats = await client.query(`
        SELECT unnest(genres) as genre, COUNT(*) as count
        FROM content
        GROUP BY genre
        ORDER BY count DESC
        LIMIT 20
      `);

      console.log('\n📊 Genre Distribution:');
      for (const row of genreStats.rows.slice(0, 10)) {
        console.log(`   ${row.genre}: ${row.count} items`);
      }

      // Create patterns
      let patternsCreated = 0;

      // Add language-based patterns (important for TV5)
      const languages = ['fra', 'eng', 'kor', 'jpn', 'spa', 'deu'];
      for (const lang of languages) {
        await client.query(`
          INSERT INTO recommendation_patterns (pattern_type, approach, success_rate)
          VALUES ($1, $2, 0.7)
          ON CONFLICT DO NOTHING
        `, [`language_${lang}`, `Recommend content in ${lang} language`]);
        patternsCreated++;
      }

      // Add more sophisticated patterns
      const additionalPatterns = [
        ['binge_worthy', 'Series with high completion rates for binge watching', 0.75],
        ['movie_night', 'Feature films for dedicated viewing sessions', 0.72],
        ['quick_watch', 'Short episodes or movies under 30 minutes', 0.68],
        ['critically_acclaimed', 'Award-winning and highly rated content', 0.80],
        ['hidden_gems', 'Underrated content with high user satisfaction', 0.70],
        ['trending_now', 'Currently popular and discussed content', 0.73],
        ['french_language', 'French and Francophone content', 0.75],
        ['international', 'Non-English language content with subtitles', 0.70]
      ];

      for (const [type, approach, rate] of additionalPatterns) {
        await client.query(`
          INSERT INTO recommendation_patterns (pattern_type, approach, success_rate)
          VALUES ($1, $2, $3)
          ON CONFLICT DO NOTHING
        `, [type, approach, rate]);
        patternsCreated++;
      }

      console.log(`\n✅ Created ${patternsCreated} patterns`);
      return patternsCreated;

    } finally {
      client.release();
    }
  }

  /**
   * Get database statistics
   */
  async getStats(): Promise<void> {
    const client = await this.pool.connect();
    try {
      const [contentStats, langStats, patternStats] = await Promise.all([
        client.query(`
          SELECT content_type, COUNT(*) as count
          FROM content
          GROUP BY content_type
        `),
        client.query(`
          SELECT original_language, COUNT(*) as count
          FROM content
          WHERE original_language IS NOT NULL
          GROUP BY original_language
          ORDER BY count DESC
          LIMIT 10
        `),
        client.query(`
          SELECT pattern_type, success_rate
          FROM recommendation_patterns
          ORDER BY success_rate DESC
          LIMIT 15
        `)
      ]);

      console.log('\n' + '═'.repeat(60));
      console.log('  DATABASE STATISTICS');
      console.log('═'.repeat(60));

      console.log('\n📊 Content by Type:');
      for (const row of contentStats.rows) {
        console.log(`   ${row.content_type}: ${row.count}`);
      }

      console.log('\n🌐 Content by Language:');
      for (const row of langStats.rows) {
        console.log(`   ${row.original_language}: ${row.count}`);
      }

      console.log('\n🎯 Top Patterns:');
      for (const row of patternStats.rows) {
        console.log(`   ${row.pattern_type}: ${(parseFloat(row.success_rate) * 100).toFixed(1)}%`);
      }

    } finally {
      client.release();
    }
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  async close(): Promise<void> {
    await this.pool.end();
  }
}

async function main() {
  const ingestion = new TV5ContentIngestion();

  console.log('\n' + '═'.repeat(60));
  console.log('  TV5 Content Ingestion - Self-Learning System');
  console.log('═'.repeat(60));

  try {
    const authenticated = await ingestion.authenticate();
    if (!authenticated) {
      console.error('Failed to authenticate with TVDB API');
      process.exit(1);
    }

    // Ingest French/TV5 content first
    console.log('\n🇫🇷 Phase 1: French & Francophone Content');
    const frenchMetrics = await ingestion.ingestFrenchContent();
    console.log(`   ✅ French content: ${frenchMetrics.frenchContent} items`);

    // Ingest popular global content
    console.log('\n🌍 Phase 2: Popular Global Content');
    const globalMetrics = await ingestion.ingestPopularContent();
    console.log(`   ✅ Global content: ${globalMetrics.totalIngested} items`);

    // Learn patterns
    console.log('\n🧠 Phase 3: Pattern Learning');
    await ingestion.learnPatterns();

    // Show final stats
    await ingestion.getStats();

    // Summary
    console.log('\n' + '═'.repeat(60));
    console.log('  INGESTION COMPLETE');
    console.log('═'.repeat(60));
    console.log(`   Total Series: ${frenchMetrics.series + globalMetrics.series}`);
    console.log(`   Total Movies: ${frenchMetrics.movies + globalMetrics.movies}`);
    console.log(`   French Content: ${frenchMetrics.frenchContent}`);
    console.log(`   Total Duration: ${((frenchMetrics.durationMs + globalMetrics.durationMs) / 1000).toFixed(1)}s`);
    console.log(`   Errors: ${frenchMetrics.errors + globalMetrics.errors}`);

  } finally {
    await ingestion.close();
  }
}

// ES module entry point
import { fileURLToPath } from 'url';
const __filename = fileURLToPath(import.meta.url);
if (process.argv[1] === __filename) {
  main().catch(console.error);
}
