/**
 * International Content Ingestion Script
 *
 * Ingests diverse international content from TVDB API including:
 * - French cinema and TV
 * - Japanese anime and dramas
 * - Korean dramas (K-drama)
 * - Spanish/Latin American content
 * - German, Italian, Hindi content
 * - African and Middle Eastern content
 */

import 'dotenv/config';
import { config } from 'dotenv';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import axios, { AxiosInstance } from 'axios';
import { Pool } from 'pg';

// Get __dirname equivalent for ESM
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Load from root hackathon-tv5/.env
config({ path: resolve(__dirname, '../../../../.env') });

const TVDB_API_KEY = process.env.TVDB_API_KEY;
const TVDB_BASE_URL = 'https://api4.thetvdb.com/v4';

interface IngestResult {
  query: string;
  language: string;
  seriesCount: number;
  moviesCount: number;
  errors: number;
}

class InternationalIngestion {
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
    if (!TVDB_API_KEY) {
      console.log('❌ TVDB_API_KEY not configured');
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

  private generateEmbedding(): string {
    const embedding = Array.from({ length: 384 }, () => Math.random() * 2 - 1);
    const norm = Math.sqrt(embedding.reduce((sum, x) => sum + x * x, 0));
    const normalized = embedding.map(x => x / norm);
    return `[${normalized.join(',')}]`;
  }

  private generateEmbeddingText(content: any, type: string): string {
    const parts: string[] = [];
    parts.push(`${content.name} (${content.year || 'Unknown'})`);
    if (content.overview) parts.push(content.overview);
    if (content.genres?.length > 0) {
      parts.push(`Genres: ${content.genres.map((g: any) => g.name).join(', ')}`);
    }
    if (content.originalNetwork?.name) {
      parts.push(`Network: ${content.originalNetwork.name}`);
    }
    if (content.originalLanguage) {
      parts.push(`Language: ${content.originalLanguage}`);
    }
    if (content.originalCountry) {
      parts.push(`Country: ${content.originalCountry}`);
    }
    return parts.join('. ');
  }

  async fetchExtended(id: number, type: 'series' | 'movies'): Promise<any | null> {
    try {
      const endpoint = type === 'series' ? `/series/${id}/extended` : `/movies/${id}/extended`;
      const response = await this.client.get(endpoint);
      return response.data.data;
    } catch (error: any) {
      return null;
    }
  }

  async storeContent(content: any, type: 'series' | 'movie'): Promise<boolean> {
    const embeddingText = this.generateEmbeddingText(content, type);
    const embedding = this.generateEmbedding();

    const imageUrl = content.image ||
      content.artworks?.find((a: any) => a.type === 2)?.image || null;

    const dbClient = await this.pool.connect();
    try {
      await dbClient.query(`
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
          original_language = EXCLUDED.original_language,
          original_country = EXCLUDED.original_country,
          image_url = COALESCE(EXCLUDED.image_url, content.image_url),
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
        content.originalNetwork?.name || content.studios?.[0]?.name || null,
        content.originalLanguage || null,
        content.originalCountry || null,
        imageUrl,
        content.firstAired || null,
        embedding
      ]);
      return true;
    } catch (error: any) {
      console.error(`  ❌ Failed to store: ${content.name} - ${error.message}`);
      return false;
    } finally {
      dbClient.release();
    }
  }

  async searchAndIngest(query: string, language: string, limit: number = 20): Promise<IngestResult> {
    const result: IngestResult = {
      query,
      language,
      seriesCount: 0,
      moviesCount: 0,
      errors: 0
    };

    console.log(`\n🔍 Searching: "${query}" (${language})`);

    try {
      const response = await this.client.get('/search', {
        params: { query, limit }
      });

      const items = response.data.data || [];
      console.log(`   Found ${items.length} results`);

      for (const item of items) {
        try {
          const type = item.type === 'series' ? 'series' : 'movies';
          const extended = await this.fetchExtended(item.tvdb_id || item.id, type);

          if (extended) {
            const stored = await this.storeContent(extended, type === 'series' ? 'series' : 'movie');
            if (stored) {
              if (type === 'series') result.seriesCount++;
              else result.moviesCount++;
              console.log(`   ✅ ${extended.name} (${extended.originalLanguage || 'unknown'})`);
            }
          }

          // Rate limiting
          await this.delay(150);
        } catch (error: any) {
          result.errors++;
        }
      }
    } catch (error: any) {
      console.error(`   ❌ Search failed: ${error.message}`);
      result.errors++;
    }

    return result;
  }

  async ingestInternationalContent(): Promise<void> {
    console.log('\n' + '═'.repeat(60));
    console.log('  INTERNATIONAL CONTENT INGESTION');
    console.log('═'.repeat(60));

    // Define international content searches
    const searches = [
      // French content
      { query: 'Lupin', lang: 'French' },
      { query: 'Les Revenants', lang: 'French' },
      { query: 'Dix Pour Cent', lang: 'French' },
      { query: 'Le Bureau des Légendes', lang: 'French' },
      { query: 'Marseille', lang: 'French' },
      { query: 'Engrenages', lang: 'French' },
      { query: 'Un Village Français', lang: 'French' },
      { query: 'Intouchables', lang: 'French' },
      { query: 'Amélie', lang: 'French' },
      { query: 'La Haine', lang: 'French' },

      // Japanese anime and dramas
      { query: 'Attack on Titan', lang: 'Japanese' },
      { query: 'Demon Slayer', lang: 'Japanese' },
      { query: 'One Piece', lang: 'Japanese' },
      { query: 'Jujutsu Kaisen', lang: 'Japanese' },
      { query: 'My Hero Academia', lang: 'Japanese' },
      { query: 'Tokyo Ghoul', lang: 'Japanese' },
      { query: 'Naruto', lang: 'Japanese' },
      { query: 'Death Note', lang: 'Japanese' },
      { query: 'Fullmetal Alchemist', lang: 'Japanese' },
      { query: 'Spirited Away', lang: 'Japanese' },
      { query: 'Your Name', lang: 'Japanese' },
      { query: 'Alice in Borderland', lang: 'Japanese' },
      { query: 'Terrace House', lang: 'Japanese' },

      // Korean dramas
      { query: 'Squid Game', lang: 'Korean' },
      { query: 'Crash Landing on You', lang: 'Korean' },
      { query: 'Goblin', lang: 'Korean' },
      { query: 'Itaewon Class', lang: 'Korean' },
      { query: 'Kingdom', lang: 'Korean' },
      { query: 'Vincenzo', lang: 'Korean' },
      { query: 'All of Us Are Dead', lang: 'Korean' },
      { query: 'Extraordinary Attorney Woo', lang: 'Korean' },
      { query: 'The Glory', lang: 'Korean' },
      { query: 'Parasite', lang: 'Korean' },
      { query: 'Oldboy', lang: 'Korean' },

      // Spanish/Latin American
      { query: 'La Casa de Papel', lang: 'Spanish' },
      { query: 'Elite', lang: 'Spanish' },
      { query: 'Narcos', lang: 'Spanish' },
      { query: 'Vis a Vis', lang: 'Spanish' },
      { query: 'Las Chicas del Cable', lang: 'Spanish' },
      { query: 'El Marginal', lang: 'Spanish' },
      { query: 'Club de Cuervos', lang: 'Spanish' },
      { query: 'Roma', lang: 'Spanish' },
      { query: 'Y Tu Mamá También', lang: 'Spanish' },

      // German
      { query: 'Dark', lang: 'German' },
      { query: 'Babylon Berlin', lang: 'German' },
      { query: 'How to Sell Drugs Online', lang: 'German' },
      { query: 'Barbarians', lang: 'German' },
      { query: 'Deutschland 83', lang: 'German' },
      { query: 'The Lives of Others', lang: 'German' },

      // Italian
      { query: 'Gomorra', lang: 'Italian' },
      { query: 'Suburra', lang: 'Italian' },
      { query: 'Baby', lang: 'Italian' },
      { query: 'Zero', lang: 'Italian' },
      { query: 'Life is Beautiful', lang: 'Italian' },

      // Hindi/Indian
      { query: 'Sacred Games', lang: 'Hindi' },
      { query: 'Delhi Crime', lang: 'Hindi' },
      { query: 'Mirzapur', lang: 'Hindi' },
      { query: 'Scam 1992', lang: 'Hindi' },
      { query: 'Panchayat', lang: 'Hindi' },
      { query: 'The Family Man', lang: 'Hindi' },
      { query: 'Dangal', lang: 'Hindi' },
      { query: '3 Idiots', lang: 'Hindi' },
      { query: 'RRR', lang: 'Hindi' },

      // Turkish
      { query: 'Ertugrul', lang: 'Turkish' },
      { query: 'The Protector', lang: 'Turkish' },
      { query: 'Fatma', lang: 'Turkish' },

      // Nordic
      { query: 'The Bridge', lang: 'Nordic' },
      { query: 'Borgen', lang: 'Nordic' },
      { query: 'The Killing', lang: 'Nordic' },
      { query: 'Trapped', lang: 'Nordic' },
      { query: 'Ragnarok', lang: 'Nordic' },

      // Chinese/Mandarin
      { query: 'The Untamed', lang: 'Chinese' },
      { query: 'Word of Honor', lang: 'Chinese' },
      { query: 'Love O2O', lang: 'Chinese' },
      { query: 'Reset', lang: 'Chinese' },

      // Thai
      { query: 'Girl From Nowhere', lang: 'Thai' },
      { query: 'Bad Buddy', lang: 'Thai' },
      { query: '2gether', lang: 'Thai' },

      // Brazilian/Portuguese
      { query: '3%', lang: 'Portuguese' },
      { query: 'Good Morning Veronica', lang: 'Portuguese' },
      { query: 'Invisible City', lang: 'Portuguese' },
      { query: 'City of God', lang: 'Portuguese' },

      // Arabic/Middle Eastern
      { query: 'Jinn', lang: 'Arabic' },
      { query: 'Paranormal', lang: 'Arabic' },

      // African
      { query: 'Blood & Water', lang: 'African' },
      { query: 'Queen Sono', lang: 'African' },
    ];

    let totalSeries = 0;
    let totalMovies = 0;
    let totalErrors = 0;

    for (const search of searches) {
      const result = await this.searchAndIngest(search.query, search.lang, 15);
      totalSeries += result.seriesCount;
      totalMovies += result.moviesCount;
      totalErrors += result.errors;

      // Rate limiting between searches
      await this.delay(500);
    }

    console.log('\n' + '═'.repeat(60));
    console.log('  INGESTION COMPLETE');
    console.log('═'.repeat(60));
    console.log(`   Series ingested: ${totalSeries}`);
    console.log(`   Movies ingested: ${totalMovies}`);
    console.log(`   Total: ${totalSeries + totalMovies}`);
    console.log(`   Errors: ${totalErrors}`);

    // Show updated language distribution
    await this.showLanguageStats();
  }

  async showLanguageStats(): Promise<void> {
    const dbClient = await this.pool.connect();
    try {
      const result = await dbClient.query(`
        SELECT original_language, COUNT(*) as count
        FROM content
        WHERE original_language IS NOT NULL AND original_language != ''
        GROUP BY original_language
        ORDER BY count DESC
        LIMIT 20
      `);

      console.log('\n📊 Updated Language Distribution:');
      for (const row of result.rows) {
        const bar = '█'.repeat(Math.min(Math.round(row.count / 50), 30));
        console.log(`   ${row.original_language.padEnd(4)} ${bar} ${row.count}`);
      }

      const total = await dbClient.query('SELECT COUNT(*) as total FROM content');
      console.log(`\n   Total content items: ${total.rows[0].total}`);
    } finally {
      dbClient.release();
    }
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  async close(): Promise<void> {
    await this.pool.end();
  }
}

// Main execution
async function main() {
  const ingestion = new InternationalIngestion();

  try {
    const authenticated = await ingestion.authenticate();
    if (!authenticated) {
      console.error('Failed to authenticate with TVDB API');
      process.exit(1);
    }

    await ingestion.ingestInternationalContent();
  } catch (error: any) {
    console.error('Ingestion failed:', error.message);
  } finally {
    await ingestion.close();
  }
}

main().catch(console.error);
