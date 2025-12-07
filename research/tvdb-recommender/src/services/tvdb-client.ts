/**
 * TheTVDB API Client
 * Handles authentication, caching, and all API interactions
 */

import axios, { AxiosInstance, AxiosError } from 'axios';
import NodeCache from 'node-cache';
import pino from 'pino';
import {
  TVDBAuthResponse,
  TVDBSearchResult,
  SeriesBaseRecord,
  SeriesExtendedRecord,
  EpisodeBaseRecord,
  MovieBaseRecord,
  TVDBConfig,
  Genre,
  Artwork,
  Character
} from '../types/index.js';

const logger = pino({ name: 'tvdb-client' });

interface TVDBApiResponse<T> {
  status: 'success' | 'failure';
  data: T;
  links?: {
    prev: string | null;
    self: string;
    next: string | null;
    total_items: number;
    page_size: number;
  };
}

export class TVDBClient {
  private client: AxiosInstance;
  private cache: NodeCache;
  private token: string | null = null;
  private tokenExpiry: Date | null = null;
  private config: TVDBConfig;

  constructor(config: TVDBConfig) {
    this.config = config;
    this.cache = new NodeCache({
      stdTTL: config.cacheTtlSeconds,
      checkperiod: 120,
      useClones: false
    });

    this.client = axios.create({
      baseURL: config.baseUrl,
      timeout: 30000,
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json'
      }
    });

    // Request interceptor for auth
    this.client.interceptors.request.use(async (config) => {
      if (config.url !== '/login') {
        const token = await this.getValidToken();
        config.headers.Authorization = `Bearer ${token}`;
      }
      return config;
    });

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      async (error: AxiosError) => {
        if (error.response?.status === 401) {
          // Token expired, refresh and retry
          this.token = null;
          this.tokenExpiry = null;
          const token = await this.getValidToken();
          if (error.config) {
            error.config.headers.Authorization = `Bearer ${token}`;
            return this.client.request(error.config);
          }
        }
        throw error;
      }
    );
  }

  /**
   * Authenticate and get JWT token
   */
  async login(): Promise<string> {
    try {
      const payload: { apikey: string; pin?: string } = {
        apikey: this.config.apiKey
      };

      if (this.config.pin) {
        payload.pin = this.config.pin;
      }

      const response = await this.client.post<TVDBAuthResponse>('/login', payload);

      if (response.data.status === 'success' && response.data.data?.token) {
        this.token = response.data.data.token;
        // Token valid for 1 month, refresh a day before
        this.tokenExpiry = new Date(Date.now() + 29 * 24 * 60 * 60 * 1000);
        logger.info('TVDB authentication successful');
        return this.token;
      }

      throw new Error(response.data.message || 'Authentication failed');
    } catch (error) {
      logger.error({ error }, 'TVDB authentication failed');
      throw error;
    }
  }

  /**
   * Get valid token, refreshing if necessary
   */
  private async getValidToken(): Promise<string> {
    if (this.token && this.tokenExpiry && this.tokenExpiry > new Date()) {
      return this.token;
    }
    return this.login();
  }

  /**
   * Search for series, movies, or people
   */
  async search(query: string, options: {
    type?: 'series' | 'movie' | 'person' | 'company';
    year?: string;
    limit?: number;
    offset?: number;
  } = {}): Promise<TVDBSearchResult[]> {
    const cacheKey = `search:${query}:${JSON.stringify(options)}`;
    const cached = this.cache.get<TVDBSearchResult[]>(cacheKey);
    if (cached) return cached;

    try {
      const params = new URLSearchParams({ query });
      if (options.type) params.append('type', options.type);
      if (options.year) params.append('year', options.year);
      if (options.limit) params.append('limit', options.limit.toString());
      if (options.offset) params.append('offset', options.offset.toString());

      const response = await this.client.get<TVDBApiResponse<TVDBSearchResult[]>>(
        `/search?${params.toString()}`
      );

      const results = response.data.data || [];
      this.cache.set(cacheKey, results);
      return results;
    } catch (error) {
      logger.error({ error, query }, 'Search failed');
      throw error;
    }
  }

  /**
   * Get series by ID (base record)
   */
  async getSeries(id: number): Promise<SeriesBaseRecord> {
    const cacheKey = `series:${id}`;
    const cached = this.cache.get<SeriesBaseRecord>(cacheKey);
    if (cached) return cached;

    try {
      const response = await this.client.get<TVDBApiResponse<SeriesBaseRecord>>(
        `/series/${id}`
      );

      const series = response.data.data;
      this.cache.set(cacheKey, series);
      return series;
    } catch (error) {
      logger.error({ error, id }, 'Failed to get series');
      throw error;
    }
  }

  /**
   * Get series extended info (includes artworks, characters, etc.)
   */
  async getSeriesExtended(id: number, options: {
    meta?: 'translations' | 'episodes';
    short?: boolean;
  } = {}): Promise<SeriesExtendedRecord> {
    const cacheKey = `series-extended:${id}:${JSON.stringify(options)}`;
    const cached = this.cache.get<SeriesExtendedRecord>(cacheKey);
    if (cached) return cached;

    try {
      const params = new URLSearchParams();
      if (options.meta) params.append('meta', options.meta);
      if (options.short) params.append('short', 'true');

      const url = `/series/${id}/extended${params.toString() ? `?${params.toString()}` : ''}`;
      const response = await this.client.get<TVDBApiResponse<SeriesExtendedRecord>>(url);

      const series = response.data.data;
      this.cache.set(cacheKey, series, 3600); // Cache extended info for 1 hour
      return series;
    } catch (error) {
      logger.error({ error, id }, 'Failed to get series extended');
      throw error;
    }
  }

  /**
   * Get all episodes for a series
   */
  async getSeriesEpisodes(id: number, seasonType: 'default' | 'dvd' | 'absolute' = 'default', options: {
    season?: number;
    episodeNumber?: number;
    airDate?: string;
    page?: number;
  } = {}): Promise<{ episodes: EpisodeBaseRecord[]; series: SeriesBaseRecord }> {
    const cacheKey = `series-episodes:${id}:${seasonType}:${JSON.stringify(options)}`;
    const cached = this.cache.get<{ episodes: EpisodeBaseRecord[]; series: SeriesBaseRecord }>(cacheKey);
    if (cached) return cached;

    try {
      const params = new URLSearchParams();
      if (options.season) params.append('season', options.season.toString());
      if (options.episodeNumber) params.append('episodeNumber', options.episodeNumber.toString());
      if (options.airDate) params.append('airDate', options.airDate);
      if (options.page) params.append('page', options.page.toString());

      const url = `/series/${id}/episodes/${seasonType}${params.toString() ? `?${params.toString()}` : ''}`;
      const response = await this.client.get<TVDBApiResponse<{ episodes: EpisodeBaseRecord[]; series: SeriesBaseRecord }>>(url);

      const data = response.data.data;
      this.cache.set(cacheKey, data);
      return data;
    } catch (error) {
      logger.error({ error, id }, 'Failed to get series episodes');
      throw error;
    }
  }

  /**
   * Get movie by ID
   */
  async getMovie(id: number): Promise<MovieBaseRecord> {
    const cacheKey = `movie:${id}`;
    const cached = this.cache.get<MovieBaseRecord>(cacheKey);
    if (cached) return cached;

    try {
      const response = await this.client.get<TVDBApiResponse<MovieBaseRecord>>(
        `/movies/${id}`
      );

      const movie = response.data.data;
      this.cache.set(cacheKey, movie);
      return movie;
    } catch (error) {
      logger.error({ error, id }, 'Failed to get movie');
      throw error;
    }
  }

  /**
   * Get all genres
   */
  async getGenres(): Promise<Genre[]> {
    const cacheKey = 'genres';
    const cached = this.cache.get<Genre[]>(cacheKey);
    if (cached) return cached;

    try {
      const response = await this.client.get<TVDBApiResponse<Genre[]>>('/genres');
      const genres = response.data.data;
      this.cache.set(cacheKey, genres, 86400 * 7); // Cache for 7 days
      return genres;
    } catch (error) {
      logger.error({ error }, 'Failed to get genres');
      throw error;
    }
  }

  /**
   * Get artwork by ID
   */
  async getArtwork(id: number): Promise<Artwork> {
    const cacheKey = `artwork:${id}`;
    const cached = this.cache.get<Artwork>(cacheKey);
    if (cached) return cached;

    try {
      const response = await this.client.get<TVDBApiResponse<Artwork>>(
        `/artwork/${id}`
      );

      const artwork = response.data.data;
      this.cache.set(cacheKey, artwork, 86400); // Cache for 1 day
      return artwork;
    } catch (error) {
      logger.error({ error, id }, 'Failed to get artwork');
      throw error;
    }
  }

  /**
   * Get updates since a timestamp
   */
  async getUpdates(since: number, options: {
    type?: 'series' | 'movies' | 'episodes';
    action?: 'update' | 'delete';
    page?: number;
  } = {}): Promise<{ id: number; method: string; timestamp: number; entityType: string }[]> {
    try {
      const params = new URLSearchParams({ since: since.toString() });
      if (options.type) params.append('type', options.type);
      if (options.action) params.append('action', options.action);
      if (options.page) params.append('page', options.page.toString());

      const response = await this.client.get<TVDBApiResponse<{ id: number; method: string; timestamp: number; entityType: string }[]>>(
        `/updates?${params.toString()}`
      );

      return response.data.data || [];
    } catch (error) {
      logger.error({ error, since }, 'Failed to get updates');
      throw error;
    }
  }

  /**
   * Get character by ID
   */
  async getCharacter(id: number): Promise<Character> {
    const cacheKey = `character:${id}`;
    const cached = this.cache.get<Character>(cacheKey);
    if (cached) return cached;

    try {
      const response = await this.client.get<TVDBApiResponse<Character>>(
        `/characters/${id}`
      );

      const character = response.data.data;
      this.cache.set(cacheKey, character, 86400); // Cache for 1 day
      return character;
    } catch (error) {
      logger.error({ error, id }, 'Failed to get character');
      throw error;
    }
  }

  /**
   * Search by remote ID (IMDB, TMDB, etc.)
   */
  async searchByRemoteId(remoteId: string): Promise<TVDBSearchResult[]> {
    const cacheKey = `remote:${remoteId}`;
    const cached = this.cache.get<TVDBSearchResult[]>(cacheKey);
    if (cached) return cached;

    try {
      const response = await this.client.get<TVDBApiResponse<TVDBSearchResult[]>>(
        `/search/remoteid/${encodeURIComponent(remoteId)}`
      );

      const results = response.data.data || [];
      this.cache.set(cacheKey, results, 86400); // Cache for 1 day
      return results;
    } catch (error) {
      logger.error({ error, remoteId }, 'Failed to search by remote ID');
      throw error;
    }
  }

  /**
   * Get popular series (trending)
   */
  async getPopularSeries(page: number = 0): Promise<SeriesBaseRecord[]> {
    const cacheKey = `popular-series:${page}`;
    const cached = this.cache.get<SeriesBaseRecord[]>(cacheKey);
    if (cached) return cached;

    try {
      const response = await this.client.get<TVDBApiResponse<SeriesBaseRecord[]>>(
        `/series?page=${page}`
      );

      const series = response.data.data || [];
      this.cache.set(cacheKey, series, 3600); // Cache for 1 hour
      return series;
    } catch (error) {
      logger.error({ error, page }, 'Failed to get popular series');
      throw error;
    }
  }

  /**
   * Clear cache
   */
  clearCache(): void {
    this.cache.flushAll();
    logger.info('Cache cleared');
  }

  /**
   * Get cache stats
   */
  getCacheStats(): { hits: number; misses: number; keys: number } {
    const stats = this.cache.getStats();
    return {
      hits: stats.hits,
      misses: stats.misses,
      keys: this.cache.keys().length
    };
  }
}

// Factory function
export function createTVDBClient(config?: Partial<TVDBConfig>): TVDBClient {
  const fullConfig: TVDBConfig = {
    apiKey: config?.apiKey || process.env.TVDB_API_KEY || '',
    pin: config?.pin || process.env.TVDB_PIN,
    baseUrl: config?.baseUrl || 'https://api4.thetvdb.com/v4',
    cacheTtlSeconds: config?.cacheTtlSeconds || 3600,
    rateLimitPerMinute: config?.rateLimitPerMinute || 100
  };

  if (!fullConfig.apiKey) {
    throw new Error('TVDB API key is required. Set TVDB_API_KEY environment variable.');
  }

  return new TVDBClient(fullConfig);
}

export default TVDBClient;
