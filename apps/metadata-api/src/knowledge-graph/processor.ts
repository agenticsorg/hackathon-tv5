/**
 * Knowledge Graph Processor
 *
 * Transforms TMDB movie rows into knowledge graph nodes and edges.
 * Handles JSON parsing, data validation, and relationship extraction.
 */

import { logger } from '../utils/logger';
import {
  TMDBMovieRow,
  MovieNode,
  GenreNode,
  ProductionCompanyNode,
  CountryNode,
  LanguageNode,
  KeywordNode,
  MovieGenreEdge,
  MovieCompanyEdge,
  MovieCountryEdge,
  MovieLanguageEdge,
  MovieKeywordEdge,
  Hyperedge,
  DistributionStatus,
} from './schema';

/**
 * Parsed TMDB JSON structures
 */
interface TMDBGenre {
  id: number;
  name: string;
}

interface TMDBCompany {
  id: number;
  name: string;
  logo_path?: string;
  origin_country?: string;
}

interface TMDBCountry {
  iso_3166_1: string;
  name: string;
}

interface TMDBLanguage {
  iso_639_1: string;
  name: string;
  english_name?: string;
}

interface TMDBKeyword {
  id: number;
  name: string;
}

/**
 * Processing result for a single movie
 */
export interface ProcessedMovie {
  movie: MovieNode;
  genres: GenreNode[];
  companies: ProductionCompanyNode[];
  countries: CountryNode[];
  languages: LanguageNode[];
  keywords: KeywordNode[];
  edges: Hyperedge[];
}

/**
 * Batch processing statistics
 */
export interface ProcessingStats {
  totalRows: number;
  successfulMovies: number;
  failedRows: number;
  totalGenres: number;
  totalCompanies: number;
  totalCountries: number;
  totalLanguages: number;
  totalKeywords: number;
  totalEdges: number;
  processingTimeMs: number;
}

/**
 * Generate a stable ID from a string (simple hash)
 */
function generateStableId(str: string): string {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32bit integer
  }
  return Math.abs(hash).toString();
}

/**
 * Parse comma-separated genres into TMDBGenre format
 * Handles format like: "Action, Science Fiction, Adventure"
 */
function parseCommaSeparatedGenres(genreString: string): TMDBGenre[] {
  if (!genreString || genreString.trim() === '') {
    return [];
  }

  return genreString
    .split(',')
    .map(g => g.trim())
    .filter(g => g.length > 0)
    .map(name => ({
      id: parseInt(generateStableId(name.toLowerCase()), 10),
      name,
    }));
}

/**
 * Parse comma-separated keywords into TMDBKeyword format
 * Handles format like: "time travel, hero, villain"
 */
function parseCommaSeparatedKeywords(keywordString: string): TMDBKeyword[] {
  if (!keywordString || keywordString.trim() === '') {
    return [];
  }

  return keywordString
    .split(',')
    .map(k => k.trim())
    .filter(k => k.length > 0)
    .map(name => ({
      id: parseInt(generateStableId(name.toLowerCase()), 10),
      name,
    }));
}

/**
 * Safe JSON parse with fallback
 */
function safeJsonParse<T>(jsonString: string | undefined, fallback: T[] = []): T[] {
  if (!jsonString || jsonString.trim() === '' || jsonString === '[]') {
    return fallback;
  }

  try {
    // Handle single-quoted JSON (common in TMDB data)
    const cleaned = jsonString
      .replace(/'/g, '"')
      .replace(/None/g, 'null')
      .replace(/True/g, 'true')
      .replace(/False/g, 'false');

    return JSON.parse(cleaned);
  } catch (error) {
    // Try to extract basic info from malformed JSON
    try {
      // Match pattern like: {'id': 28, 'name': 'Action'}
      const matches = jsonString.matchAll(/\{[^}]+\}/g);
      const results: any[] = [];

      for (const match of matches) {
        const cleaned = match[0]
          .replace(/'/g, '"')
          .replace(/None/g, 'null');
        try {
          results.push(JSON.parse(cleaned));
        } catch {
          // Skip malformed objects
        }
      }

      return results as T[];
    } catch {
      return fallback;
    }
  }
}

/**
 * Parse genres field - handles both JSON and comma-separated formats
 */
function parseGenres(genreField: string | undefined): TMDBGenre[] {
  if (!genreField || genreField.trim() === '' || genreField === '[]') {
    return [];
  }

  // Check if it looks like JSON (starts with [ or {)
  const trimmed = genreField.trim();
  if (trimmed.startsWith('[') || trimmed.startsWith('{')) {
    const parsed = safeJsonParse<TMDBGenre>(genreField);
    if (parsed.length > 0) {
      return parsed;
    }
  }

  // Fall back to comma-separated parsing
  return parseCommaSeparatedGenres(genreField);
}

/**
 * Parse keywords field - handles both JSON and comma-separated formats
 */
function parseKeywords(keywordField: string | undefined): TMDBKeyword[] {
  if (!keywordField || keywordField.trim() === '' || keywordField === '[]') {
    return [];
  }

  // Check if it looks like JSON (starts with [ or {)
  const trimmed = keywordField.trim();
  if (trimmed.startsWith('[') || trimmed.startsWith('{')) {
    const parsed = safeJsonParse<TMDBKeyword>(keywordField);
    if (parsed.length > 0) {
      return parsed;
    }
  }

  // Fall back to comma-separated parsing
  return parseCommaSeparatedKeywords(keywordField);
}

/**
 * Parse numeric value with fallback
 */
function parseNumber(value: string | undefined, fallback: number = 0): number {
  if (!value || value.trim() === '') return fallback;
  const parsed = parseFloat(value);
  return isNaN(parsed) ? fallback : parsed;
}

/**
 * Generate unique edge ID
 */
function generateEdgeId(type: string, ...ids: string[]): string {
  return `${type}:${ids.join('-')}`;
}

/**
 * Get ISO 639-1 to BCP-47 mapping
 */
function getBcp47Tag(iso639: string): string {
  const bcp47Map: Record<string, string> = {
    en: 'en-US',
    es: 'es-ES',
    fr: 'fr-FR',
    de: 'de-DE',
    it: 'it-IT',
    pt: 'pt-PT',
    ja: 'ja-JP',
    ko: 'ko-KR',
    zh: 'zh-CN',
    hi: 'hi-IN',
    ar: 'ar-SA',
    ru: 'ru-RU',
    // Add more mappings as needed
  };

  return bcp47Map[iso639] || `${iso639}-${iso639.toUpperCase()}`;
}

/**
 * Knowledge Graph Processor Class
 */
export class KnowledgeGraphProcessor {
  private genreCache: Map<string, GenreNode> = new Map();
  private companyCache: Map<string, ProductionCompanyNode> = new Map();
  private countryCache: Map<string, CountryNode> = new Map();
  private languageCache: Map<string, LanguageNode> = new Map();
  private keywordCache: Map<string, KeywordNode> = new Map();

  constructor() {
    logger.info('Knowledge Graph Processor initialized');
  }

  /**
   * Process a single TMDB row into graph nodes and edges
   */
  processRow(row: TMDBMovieRow): ProcessedMovie | null {
    try {
      const now = new Date().toISOString();
      const movieId = row.id;

      if (!movieId || !row.title) {
        logger.warn('Skipping row with missing ID or title');
        return null;
      }

      // Parse JSON fields (genres and keywords also support comma-separated format)
      const genres = parseGenres(row.genres);
      const companies = safeJsonParse<TMDBCompany>(row.production_companies);
      const countries = safeJsonParse<TMDBCountry>(row.production_countries);
      const languages = safeJsonParse<TMDBLanguage>(row.spoken_languages);
      const keywords = parseKeywords(row.keywords);

      // Create movie node
      const movieNode: MovieNode = {
        id: movieId,
        type: 'movie',
        title: row.title,
        originalTitle: row.original_title || undefined,
        overview: row.overview || '',
        tagline: row.tagline || undefined,
        adult: row.adult === 'True' || row.adult === 'true' || row.adult === '1',
        status: row.status || 'Unknown',
        releaseDate: row.release_date || undefined,
        year: row.release_date ? parseInt(row.release_date.substring(0, 4), 10) : undefined,
        runtime: parseNumber(row.runtime) || undefined,
        budget: parseNumber(row.budget) || undefined,
        revenue: parseNumber(row.revenue) || undefined,
        voteAverage: parseNumber(row.vote_average),
        voteCount: parseNumber(row.vote_count),
        popularity: parseNumber(row.popularity),
        posterPath: row.poster_path || undefined,
        backdropPath: row.backdrop_path || undefined,
        homepage: row.homepage || undefined,
        imdbId: row.imdb_id || undefined,
        distributionStatus: 'pending' as DistributionStatus,
        createdAt: now,
        updatedAt: now,
      };

      // Process related entities and edges
      const genreNodes: GenreNode[] = [];
      const companyNodes: ProductionCompanyNode[] = [];
      const countryNodes: CountryNode[] = [];
      const languageNodes: LanguageNode[] = [];
      const keywordNodes: KeywordNode[] = [];
      const edges: Hyperedge[] = [];

      // Process genres
      genres.forEach((g, index) => {
        const genreId = String(g.id);
        let genreNode = this.genreCache.get(genreId);

        if (!genreNode) {
          genreNode = {
            id: genreId,
            type: 'genre',
            name: g.name,
            movieCount: 0,
            createdAt: now,
            updatedAt: now,
          };
          this.genreCache.set(genreId, genreNode);
        }

        genreNode.movieCount++;
        genreNode.updatedAt = now;
        genreNodes.push(genreNode);

        // Create edge
        const edge: MovieGenreEdge = {
          id: generateEdgeId('HAS_GENRE', movieId, genreId),
          type: 'HAS_GENRE',
          movieId,
          genreId,
          primary: index === 0, // First genre is primary
          createdAt: now,
          updatedAt: now,
        };
        edges.push(edge);
      });

      // Process production companies
      companies.forEach((c) => {
        const companyId = String(c.id);
        let companyNode = this.companyCache.get(companyId);

        if (!companyNode) {
          companyNode = {
            id: companyId,
            type: 'production_company',
            name: c.name,
            originCountry: c.origin_country || undefined,
            logoPath: c.logo_path || undefined,
            movieCount: 0,
            createdAt: now,
            updatedAt: now,
          };
          this.companyCache.set(companyId, companyNode);
        }

        companyNode.movieCount++;
        companyNode.updatedAt = now;
        companyNodes.push(companyNode);

        const edge: MovieCompanyEdge = {
          id: generateEdgeId('PRODUCED_BY', movieId, companyId),
          type: 'PRODUCED_BY',
          movieId,
          companyId,
          role: 'producer',
          createdAt: now,
          updatedAt: now,
        };
        edges.push(edge);
      });

      // Process countries
      countries.forEach((c) => {
        const countryId = c.iso_3166_1;
        let countryNode = this.countryCache.get(countryId);

        if (!countryNode) {
          countryNode = {
            id: countryId,
            type: 'country',
            name: c.name,
            iso31661: c.iso_3166_1,
            movieCount: 0,
            createdAt: now,
            updatedAt: now,
          };
          this.countryCache.set(countryId, countryNode);
        }

        countryNode.movieCount++;
        countryNode.updatedAt = now;
        countryNodes.push(countryNode);

        const edge: MovieCountryEdge = {
          id: generateEdgeId('PRODUCED_IN', movieId, countryId),
          type: 'PRODUCED_IN',
          movieId,
          countryId,
          createdAt: now,
          updatedAt: now,
        };
        edges.push(edge);
      });

      // Process languages
      languages.forEach((l, index) => {
        const languageId = l.iso_639_1;
        let languageNode = this.languageCache.get(languageId);

        if (!languageNode) {
          languageNode = {
            id: languageId,
            type: 'language',
            name: l.name,
            iso6391: l.iso_639_1,
            englishName: l.english_name || undefined,
            bcp47Tag: getBcp47Tag(l.iso_639_1),
            movieCount: 0,
            createdAt: now,
            updatedAt: now,
          };
          this.languageCache.set(languageId, languageNode);
        }

        languageNode.movieCount++;
        languageNode.updatedAt = now;
        languageNodes.push(languageNode);

        const edge: MovieLanguageEdge = {
          id: generateEdgeId('SPOKEN_IN', movieId, languageId),
          type: 'SPOKEN_IN',
          movieId,
          languageId,
          primary: index === 0,
          createdAt: now,
          updatedAt: now,
        };
        edges.push(edge);
      });

      // Process keywords
      keywords.forEach((k) => {
        const keywordId = String(k.id);
        let keywordNode = this.keywordCache.get(keywordId);

        if (!keywordNode) {
          keywordNode = {
            id: keywordId,
            type: 'keyword',
            name: k.name,
            movieCount: 0,
            createdAt: now,
            updatedAt: now,
          };
          this.keywordCache.set(keywordId, keywordNode);
        }

        keywordNode.movieCount++;
        keywordNode.updatedAt = now;
        keywordNodes.push(keywordNode);

        const edge: MovieKeywordEdge = {
          id: generateEdgeId('HAS_KEYWORD', movieId, keywordId),
          type: 'HAS_KEYWORD',
          movieId,
          keywordId,
          createdAt: now,
          updatedAt: now,
        };
        edges.push(edge);
      });

      return {
        movie: movieNode,
        genres: genreNodes,
        companies: companyNodes,
        countries: countryNodes,
        languages: languageNodes,
        keywords: keywordNodes,
        edges,
      };
    } catch (error) {
      logger.error('Failed to process row', {
        movieId: row.id,
        title: row.title,
        error: error instanceof Error ? error.message : 'Unknown error',
      });
      return null;
    }
  }

  /**
   * Process batch of TMDB rows
   */
  processBatch(rows: TMDBMovieRow[]): {
    movies: ProcessedMovie[];
    stats: Partial<ProcessingStats>;
  } {
    const startTime = Date.now();
    const movies: ProcessedMovie[] = [];
    let failedRows = 0;

    for (const row of rows) {
      const result = this.processRow(row);
      if (result) {
        movies.push(result);
      } else {
        failedRows++;
      }
    }

    return {
      movies,
      stats: {
        totalRows: rows.length,
        successfulMovies: movies.length,
        failedRows,
        processingTimeMs: Date.now() - startTime,
      },
    };
  }

  /**
   * Get aggregated entity caches
   */
  getEntityCaches(): {
    genres: Map<string, GenreNode>;
    companies: Map<string, ProductionCompanyNode>;
    countries: Map<string, CountryNode>;
    languages: Map<string, LanguageNode>;
    keywords: Map<string, KeywordNode>;
  } {
    return {
      genres: this.genreCache,
      companies: this.companyCache,
      countries: this.countryCache,
      languages: this.languageCache,
      keywords: this.keywordCache,
    };
  }

  /**
   * Clear all caches
   */
  clearCaches(): void {
    this.genreCache.clear();
    this.companyCache.clear();
    this.countryCache.clear();
    this.languageCache.clear();
    this.keywordCache.clear();
  }

  /**
   * Get cache statistics
   */
  getCacheStats(): {
    genres: number;
    companies: number;
    countries: number;
    languages: number;
    keywords: number;
  } {
    return {
      genres: this.genreCache.size,
      companies: this.companyCache.size,
      countries: this.countryCache.size,
      languages: this.languageCache.size,
      keywords: this.keywordCache.size,
    };
  }
}

/**
 * Singleton instance
 */
let processorInstance: KnowledgeGraphProcessor | null = null;

export function getProcessor(): KnowledgeGraphProcessor {
  if (!processorInstance) {
    processorInstance = new KnowledgeGraphProcessor();
  }
  return processorInstance;
}
