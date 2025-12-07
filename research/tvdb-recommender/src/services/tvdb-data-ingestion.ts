/**
 * TVDB Data Ingestion Service
 *
 * Ingests rich content from TVDB API for training the self-learning system.
 * Uses extended endpoints for comprehensive metadata that improves embeddings.
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

interface TVDBContent {
  id: number;
  name: string;
  year?: string;
  overview?: string;
  image?: string;
  status?: { name: string };
  originalNetwork?: { name: string; id: number };
  genres?: Array<{ name: string }>;
  characters?: Array<{ name: string; personName: string }>;
  artworks?: Array<{ type: number; image: string }>;
  translations?: { nameTranslations?: string[]; overviewTranslations?: string[] };
  firstAired?: string;
  lastAired?: string;
  originalLanguage?: string;
  originalCountry?: string;
  averageRuntime?: number;
  episodes?: Array<{ seasonNumber: number; number: number; name: string }>;
  // Movie-specific
  studios?: Array<{ name: string }>;
  productionCountries?: Array<{ name: string }>;
  boxOffice?: string;
  budget?: string;
  runtime?: number;
}

interface StreamingAvailability {
  contentId: string;
  platforms: Array<{
    name: string;
    country: string;
    type: 'subscription' | 'rent' | 'buy';
    deepLink?: string;
  }>;
}

interface IngestMetrics {
  seriesIngested: number;
  moviesIngested: number;
  embeddingsGenerated: number;
  patternsLearned: number;
  errorsEncountered: number;
  durationMs: number;
}

export class TVDBDataIngestion {
  private pool: Pool;
  private client: AxiosInstance;
  private token: string | null = null;
  private lastSyncTimestamp: number = 0;

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

  /**
   * Authenticate with TVDB API
   */
  async authenticate(): Promise<boolean> {
    if (!TVDB_API_KEY || TVDB_API_KEY === 'your-tvdb-api-key-here') {
      console.log('⚠️  TVDB_API_KEY not configured - using demo mode');
      return false;
    }

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
   * Generate rich text for embedding from extended content data
   * This creates a comprehensive text representation for better semantic search
   */
  private generateEmbeddingText(content: TVDBContent, type: 'series' | 'movie'): string {
    const parts: string[] = [];

    // Title with year for uniqueness
    parts.push(`${content.name} (${content.year || 'Unknown'})`);

    // Overview is the most important for semantic matching
    if (content.overview) {
      parts.push(content.overview);
    }

    // Genres are crucial for recommendations
    if (content.genres && content.genres.length > 0) {
      parts.push(`Genres: ${content.genres.map(g => g.name).join(', ')}`);
    }

    // Network/Studio helps identify similar content sources
    if (content.originalNetwork?.name) {
      parts.push(`Network: ${content.originalNetwork.name}`);
    }
    if (content.studios && content.studios.length > 0) {
      parts.push(`Studios: ${content.studios.map(s => s.name).join(', ')}`);
    }

    // Characters and actors for "if you liked actor X" recommendations
    if (content.characters && content.characters.length > 0) {
      const topCharacters = content.characters.slice(0, 5);
      parts.push(`Starring: ${topCharacters.map(c => c.personName || c.name).join(', ')}`);
    }

    // Runtime helps match viewing time preferences
    if (content.averageRuntime) {
      parts.push(`Runtime: ${content.averageRuntime} minutes per episode`);
    }
    if (content.runtime) {
      parts.push(`Runtime: ${content.runtime} minutes`);
    }

    // Status indicates if it's ongoing or complete
    if (content.status?.name) {
      parts.push(`Status: ${content.status.name}`);
    }

    // Country/Language for regional preferences
    if (content.originalCountry) {
      parts.push(`Country: ${content.originalCountry}`);
    }

    return parts.join('. ');
  }

  /**
   * Generate embedding vector (simplified - in production use Transformers.js)
   */
  private generateEmbedding(): string {
    // In production, use actual embedding model
    const embedding = Array.from({ length: 384 }, () => Math.random() * 2 - 1);
    // Normalize
    const norm = Math.sqrt(embedding.reduce((sum, x) => sum + x * x, 0));
    const normalized = embedding.map(x => x / norm);
    return `[${normalized.join(',')}]`;
  }

  /**
   * Fetch extended series data
   */
  async fetchSeriesExtended(seriesId: number): Promise<TVDBContent | null> {
    try {
      const response = await this.client.get(`/series/${seriesId}/extended`, {
        params: { meta: 'episodes' }
      });
      return response.data.data;
    } catch (error: any) {
      console.error(`Failed to fetch series ${seriesId}:`, error.message);
      return null;
    }
  }

  /**
   * Fetch extended movie data
   */
  async fetchMovieExtended(movieId: number): Promise<TVDBContent | null> {
    try {
      const response = await this.client.get(`/movies/${movieId}/extended`);
      return response.data.data;
    } catch (error: any) {
      console.error(`Failed to fetch movie ${movieId}:`, error.message);
      return null;
    }
  }

  /**
   * Search and ingest content by query
   */
  async searchAndIngest(query: string, limit: number = 25): Promise<IngestMetrics> {
    const startTime = Date.now();
    const metrics: IngestMetrics = {
      seriesIngested: 0,
      moviesIngested: 0,
      embeddingsGenerated: 0,
      patternsLearned: 0,
      errorsEncountered: 0,
      durationMs: 0
    };

    console.log(`\n🔍 Searching TVDB for: "${query}"`);

    try {
      const response = await this.client.get('/search', {
        params: { query, limit }
      });

      const results = response.data.data || [];
      console.log(`📺 Found ${results.length} results`);

      for (const item of results) {
        try {
          if (item.type === 'series') {
            const extended = await this.fetchSeriesExtended(item.tvdb_id || item.id);
            if (extended) {
              await this.storeContent(extended, 'series');
              metrics.seriesIngested++;
              metrics.embeddingsGenerated++;
            }
          } else if (item.type === 'movie') {
            const extended = await this.fetchMovieExtended(item.tvdb_id || item.id);
            if (extended) {
              await this.storeContent(extended, 'movie');
              metrics.moviesIngested++;
              metrics.embeddingsGenerated++;
            }
          }

          // Rate limiting - TVDB has strict limits
          await this.delay(100);
        } catch (error: any) {
          console.error(`Error ingesting ${item.name}:`, error.message);
          metrics.errorsEncountered++;
        }
      }
    } catch (error: any) {
      console.error('Search failed:', error.message);
      metrics.errorsEncountered++;
    }

    metrics.durationMs = Date.now() - startTime;
    return metrics;
  }

  /**
   * Extract best image URLs from content
   * TVDB artwork types: 1=Banner, 2=Poster, 3=Background, 6=Icon, 7=Clearart, etc.
   */
  private extractImageUrls(content: TVDBContent): { imageUrl: string | null; thumbnailUrl: string | null } {
    let imageUrl = content.image || null;
    let thumbnailUrl = null;

    // Try to get poster (type 2) or banner (type 1) from artworks
    if (content.artworks && content.artworks.length > 0) {
      // Prefer poster (type 2) for main image
      const poster = content.artworks.find(a => a.type === 2);
      if (poster?.image) {
        imageUrl = poster.image;
      }

      // Use smaller artwork or generate thumbnail URL
      // TVDB images can be resized by modifying the URL path
      if (imageUrl) {
        // TVDB provides thumbnail versions at artworks4.thetvdb.com
        thumbnailUrl = imageUrl.replace('artworks.thetvdb.com', 'artworks4.thetvdb.com');
      }
    }

    return { imageUrl, thumbnailUrl };
  }

  /**
   * Store content with rich embedding
   */
  async storeContent(content: TVDBContent, type: 'series' | 'movie'): Promise<void> {
    const embeddingText = this.generateEmbeddingText(content, type);
    const embedding = this.generateEmbedding();
    const { imageUrl, thumbnailUrl } = this.extractImageUrls(content);

    const client = await this.pool.connect();
    try {
      await client.query(`
        INSERT INTO content (id, content_type, title, year, overview, genres, rating,
                             network_id, network_name, original_language, original_country,
                             image_url, thumbnail_url, first_aired, embedding)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15::ruvector(384))
        ON CONFLICT (id) DO UPDATE SET
          title = EXCLUDED.title,
          overview = EXCLUDED.overview,
          genres = EXCLUDED.genres,
          embedding = EXCLUDED.embedding,
          network_name = EXCLUDED.network_name,
          image_url = COALESCE(EXCLUDED.image_url, content.image_url),
          thumbnail_url = COALESCE(EXCLUDED.thumbnail_url, content.thumbnail_url),
          updated_at = NOW()
      `, [
        content.id.toString(),
        type,
        content.name,
        content.year ? parseInt(content.year) : null,
        embeddingText, // Store rich text for transparency
        content.genres?.map(g => g.name) || [],
        null, // TVDB doesn't provide ratings directly
        content.originalNetwork?.id || null,
        content.originalNetwork?.name || null,
        content.originalLanguage || 'en',
        content.originalCountry || null,
        imageUrl,
        thumbnailUrl,
        content.firstAired || null,
        embedding
      ]);

      console.log(`  ✅ Stored: ${content.name} (${type})${imageUrl ? ' 🖼️' : ''}`);
    } finally {
      client.release();
    }
  }

  /**
   * Ingest updates since last sync (continuous learning)
   */
  async ingestUpdates(): Promise<IngestMetrics> {
    const startTime = Date.now();
    const metrics: IngestMetrics = {
      seriesIngested: 0,
      moviesIngested: 0,
      embeddingsGenerated: 0,
      patternsLearned: 0,
      errorsEncountered: 0,
      durationMs: 0
    };

    // Get last sync timestamp from database
    const client = await this.pool.connect();
    try {
      const result = await client.query(`
        SELECT last_sync_timestamp FROM sync_status
        WHERE sync_type = 'content'
        ORDER BY created_at DESC LIMIT 1
      `);

      if (result.rows.length > 0) {
        this.lastSyncTimestamp = parseInt(result.rows[0].last_sync_timestamp);
      } else {
        // First sync - use 7 days ago
        this.lastSyncTimestamp = Math.floor(Date.now() / 1000) - (7 * 24 * 60 * 60);
      }
    } finally {
      client.release();
    }

    console.log(`\n🔄 Fetching updates since ${new Date(this.lastSyncTimestamp * 1000).toISOString()}`);

    try {
      // Fetch series updates
      const seriesUpdates = await this.client.get('/updates', {
        params: { since: this.lastSyncTimestamp, type: 'series' }
      });

      const seriesIds = seriesUpdates.data.data || [];
      console.log(`📺 Found ${seriesIds.length} updated series`);

      for (const item of seriesIds.slice(0, 50)) { // Limit for demo
        const extended = await this.fetchSeriesExtended(item.recordId || item.id);
        if (extended) {
          await this.storeContent(extended, 'series');
          metrics.seriesIngested++;
          metrics.embeddingsGenerated++;
        }
        await this.delay(100);
      }

      // Fetch movie updates
      const movieUpdates = await this.client.get('/updates', {
        params: { since: this.lastSyncTimestamp, type: 'movies' }
      });

      const movieIds = movieUpdates.data.data || [];
      console.log(`🎬 Found ${movieIds.length} updated movies`);

      for (const item of movieIds.slice(0, 50)) {
        const extended = await this.fetchMovieExtended(item.recordId || item.id);
        if (extended) {
          await this.storeContent(extended, 'movie');
          metrics.moviesIngested++;
          metrics.embeddingsGenerated++;
        }
        await this.delay(100);
      }

      // Record sync
      await this.recordSync('content', metrics.seriesIngested + metrics.moviesIngested);

    } catch (error: any) {
      console.error('Update fetch failed:', error.message);
      metrics.errorsEncountered++;
    }

    metrics.durationMs = Date.now() - startTime;
    return metrics;
  }

  /**
   * Record sync status
   */
  private async recordSync(syncType: string, itemsSynced: number): Promise<void> {
    const client = await this.pool.connect();
    try {
      await client.query(`
        INSERT INTO sync_status (sync_type, last_sync_timestamp, items_synced, status)
        VALUES ($1, $2, $3, 'completed')
      `, [syncType, Math.floor(Date.now() / 1000), itemsSynced]);
    } finally {
      client.release();
    }
  }

  /**
   * Ingest popular content for cold start
   */
  async ingestPopularContent(): Promise<IngestMetrics> {
    const categories = [
      'drama', 'comedy', 'thriller', 'sci-fi', 'action',
      'romance', 'horror', 'documentary', 'animation', 'crime'
    ];

    let totalMetrics: IngestMetrics = {
      seriesIngested: 0,
      moviesIngested: 0,
      embeddingsGenerated: 0,
      patternsLearned: 0,
      errorsEncountered: 0,
      durationMs: 0
    };

    console.log('\n📊 Ingesting popular content across genres...');

    for (const category of categories) {
      const metrics = await this.searchAndIngest(category, 10);
      totalMetrics.seriesIngested += metrics.seriesIngested;
      totalMetrics.moviesIngested += metrics.moviesIngested;
      totalMetrics.embeddingsGenerated += metrics.embeddingsGenerated;
      totalMetrics.errorsEncountered += metrics.errorsEncountered;
      totalMetrics.durationMs += metrics.durationMs;

      // Rate limiting between categories
      await this.delay(1000);
    }

    return totalMetrics;
  }

  /**
   * Learn patterns from ingested content
   */
  async learnContentPatterns(): Promise<number> {
    console.log('\n🧠 Learning patterns from ingested content...');

    const client = await this.pool.connect();
    try {
      // Analyze genre distributions
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

      // Analyze network distributions
      const networkStats = await client.query(`
        SELECT network_name, COUNT(*) as count
        FROM content
        WHERE network_name IS NOT NULL
        GROUP BY network_name
        ORDER BY count DESC
        LIMIT 10
      `);

      console.log('\n📺 Network Distribution:');
      for (const row of networkStats.rows.slice(0, 5)) {
        console.log(`   ${row.network_name}: ${row.count} items`);
      }

      // Create learned patterns
      let patternsCreated = 0;

      // Genre-based patterns
      for (const row of genreStats.rows.slice(0, 5)) {
        await client.query(`
          INSERT INTO recommendation_patterns (pattern_type, approach, success_rate)
          VALUES ($1, $2, 0.7)
          ON CONFLICT DO NOTHING
        `, [`genre_${row.genre.toLowerCase()}`, `Recommend ${row.genre} content based on user genre affinity`]);
        patternsCreated++;
      }

      // Network-based patterns
      for (const row of networkStats.rows.slice(0, 3)) {
        if (row.network_name) {
          await client.query(`
            INSERT INTO recommendation_patterns (pattern_type, approach, success_rate)
            VALUES ($1, $2, 0.65)
            ON CONFLICT DO NOTHING
          `, [`network_${row.network_name.toLowerCase().replace(/\s+/g, '_')}`,
              `Recommend content from ${row.network_name} for users who watch this network`]);
          patternsCreated++;
        }
      }

      console.log(`\n✅ Created ${patternsCreated} learned patterns`);
      return patternsCreated;

    } finally {
      client.release();
    }
  }

  /**
   * Get ingestion statistics
   */
  async getStats(): Promise<any> {
    const client = await this.pool.connect();
    try {
      const [contentStats, patternStats, syncStats] = await Promise.all([
        client.query(`
          SELECT
            content_type,
            COUNT(*) as count,
            COUNT(CASE WHEN embedding IS NOT NULL THEN 1 END) as with_embeddings
          FROM content
          GROUP BY content_type
        `),
        client.query(`
          SELECT COUNT(*) as total, AVG(success_rate) as avg_success
          FROM recommendation_patterns
        `),
        client.query(`
          SELECT sync_type, last_sync_timestamp, items_synced
          FROM sync_status
          ORDER BY created_at DESC
          LIMIT 5
        `)
      ]);

      return {
        content: contentStats.rows,
        patterns: patternStats.rows[0],
        recentSyncs: syncStats.rows
      };
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

// CLI execution
async function main() {
  const ingestion = new TVDBDataIngestion();

  console.log('\n' + '═'.repeat(60));
  console.log('  TVDB Data Ingestion for Self-Learning System');
  console.log('═'.repeat(60));

  try {
    const authenticated = await ingestion.authenticate();

    if (authenticated) {
      // Ingest popular content
      const metrics = await ingestion.ingestPopularContent();

      console.log('\n' + '═'.repeat(60));
      console.log('  INGESTION SUMMARY');
      console.log('═'.repeat(60));
      console.log(`   Series Ingested: ${metrics.seriesIngested}`);
      console.log(`   Movies Ingested: ${metrics.moviesIngested}`);
      console.log(`   Embeddings: ${metrics.embeddingsGenerated}`);
      console.log(`   Errors: ${metrics.errorsEncountered}`);
      console.log(`   Duration: ${(metrics.durationMs / 1000).toFixed(1)}s`);

      // Learn patterns
      const patternsLearned = await ingestion.learnContentPatterns();
      console.log(`   Patterns Learned: ${patternsLearned}`);

    } else {
      console.log('\n⚠️  Running in demo mode without TVDB API');
      console.log('   Set TVDB_API_KEY in .env to enable real data ingestion');
    }

    // Show final stats
    const stats = await ingestion.getStats();
    console.log('\n📊 Database Statistics:');
    console.log(JSON.stringify(stats, null, 2));

  } finally {
    await ingestion.close();
  }
}

// Export for use as module
export { main as runIngestion };

// Run if called directly
import { fileURLToPath } from 'url';
const __filename = fileURLToPath(import.meta.url);
if (process.argv[1] === __filename) {
  main().catch(console.error);
}
