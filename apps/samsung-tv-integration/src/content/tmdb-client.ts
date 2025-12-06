/**
 * TMDb (The Movie Database) API Client
 *
 * Fetches movie and TV show metadata for the learning system
 * API docs: https://developer.themoviedb.org/reference
 */

import { ContentMetadata, Genre, ContentType } from '../learning/types.js';

const TMDB_BASE_URL = 'https://api.themoviedb.org/3';
const TMDB_IMAGE_BASE = 'https://image.tmdb.org/t/p';

// TMDb genre ID to our Genre type mapping
const GENRE_MAP: Record<number, Genre> = {
  28: 'action',
  12: 'adventure',
  16: 'animation',
  35: 'comedy',
  80: 'crime',
  99: 'documentary',
  18: 'drama',
  10751: 'family',
  14: 'fantasy',
  36: 'history',
  27: 'horror',
  10402: 'music',
  9648: 'mystery',
  10749: 'romance',
  878: 'science_fiction',
  10770: 'drama', // TV Movie
  53: 'thriller',
  10752: 'war',
  37: 'western',
  // TV genres
  10759: 'action', // Action & Adventure
  10762: 'family', // Kids
  10763: 'news',
  10764: 'reality',
  10765: 'science_fiction', // Sci-Fi & Fantasy
  10766: 'drama', // Soap
  10767: 'drama', // Talk
  10768: 'war', // War & Politics
};

// Streaming app mapping for deep linking
const STREAMING_PROVIDERS: Record<number, { appId: string; appName: string }> = {
  8: { appId: '111299001912', appName: 'Netflix' },
  9: { appId: '111299000410', appName: 'Amazon Prime Video' },
  337: { appId: '3201601007250', appName: 'Disney+' },
  1899: { appId: '3201606009684', appName: 'HBO Max' },
  15: { appId: '3201512006963', appName: 'Hulu' },
  386: { appId: '3201611010983', appName: 'Peacock' },
  531: { appId: '3201601007670', appName: 'Paramount+' },
  350: { appId: '3201608010191', appName: 'Apple TV+' },
};

export interface TMDbConfig {
  apiKey: string;
  language?: string;
  region?: string;
  includeAdult?: boolean;
}

export interface TMDbMovie {
  id: number;
  title: string;
  original_title: string;
  overview: string;
  poster_path: string | null;
  backdrop_path: string | null;
  release_date: string;
  genre_ids: number[];
  vote_average: number;
  vote_count: number;
  popularity: number;
  adult: boolean;
  runtime?: number;
}

export interface TMDbTVShow {
  id: number;
  name: string;
  original_name: string;
  overview: string;
  poster_path: string | null;
  backdrop_path: string | null;
  first_air_date: string;
  genre_ids: number[];
  vote_average: number;
  vote_count: number;
  popularity: number;
  episode_run_time?: number[];
}

export interface TMDbSearchResult<T> {
  page: number;
  results: T[];
  total_pages: number;
  total_results: number;
}

export interface TMDbCredits {
  cast: Array<{ id: number; name: string; character: string; order: number }>;
  crew: Array<{ id: number; name: string; job: string; department: string }>;
}

export interface TMDbWatchProviders {
  results: Record<string, {
    link: string;
    flatrate?: Array<{ provider_id: number; provider_name: string; logo_path: string }>;
    rent?: Array<{ provider_id: number; provider_name: string; logo_path: string }>;
    buy?: Array<{ provider_id: number; provider_name: string; logo_path: string }>;
  }>;
}

/**
 * TMDb API Client for content discovery
 */
export class TMDbClient {
  private apiKey: string;
  private language: string;
  private region: string;
  private includeAdult: boolean;
  private cache: Map<string, { data: unknown; timestamp: number }> = new Map();
  private cacheTTL: number = 5 * 60 * 1000; // 5 minutes

  constructor(config: TMDbConfig) {
    this.apiKey = config.apiKey;
    this.language = config.language || 'en-US';
    this.region = config.region || 'US';
    this.includeAdult = config.includeAdult || false;
  }

  /**
   * Make API request with caching
   */
  private async request<T>(endpoint: string, params: Record<string, string> = {}): Promise<T> {
    const url = new URL(`${TMDB_BASE_URL}${endpoint}`);
    url.searchParams.set('api_key', this.apiKey);
    url.searchParams.set('language', this.language);

    for (const [key, value] of Object.entries(params)) {
      url.searchParams.set(key, value);
    }

    const cacheKey = url.toString();
    const cached = this.cache.get(cacheKey);
    if (cached && Date.now() - cached.timestamp < this.cacheTTL) {
      return cached.data as T;
    }

    const response = await fetch(url.toString());
    if (!response.ok) {
      throw new Error(`TMDb API error: ${response.status} ${response.statusText}`);
    }

    const data = await response.json() as T;
    this.cache.set(cacheKey, { data, timestamp: Date.now() });
    return data;
  }

  /**
   * Search for movies
   */
  async searchMovies(query: string, page: number = 1): Promise<TMDbSearchResult<TMDbMovie>> {
    return this.request<TMDbSearchResult<TMDbMovie>>('/search/movie', {
      query,
      page: String(page),
      include_adult: String(this.includeAdult),
      region: this.region,
    });
  }

  /**
   * Search for TV shows
   */
  async searchTVShows(query: string, page: number = 1): Promise<TMDbSearchResult<TMDbTVShow>> {
    return this.request<TMDbSearchResult<TMDbTVShow>>('/search/tv', {
      query,
      page: String(page),
      include_adult: String(this.includeAdult),
    });
  }

  /**
   * Multi-search (movies, TV shows, people)
   */
  async multiSearch(query: string, page: number = 1): Promise<TMDbSearchResult<TMDbMovie | TMDbTVShow>> {
    return this.request<TMDbSearchResult<TMDbMovie | TMDbTVShow>>('/search/multi', {
      query,
      page: String(page),
      include_adult: String(this.includeAdult),
      region: this.region,
    });
  }

  /**
   * Get trending content
   */
  async getTrending(
    mediaType: 'movie' | 'tv' | 'all' = 'all',
    timeWindow: 'day' | 'week' = 'week'
  ): Promise<TMDbSearchResult<TMDbMovie | TMDbTVShow>> {
    return this.request<TMDbSearchResult<TMDbMovie | TMDbTVShow>>(
      `/trending/${mediaType}/${timeWindow}`
    );
  }

  /**
   * Get popular movies
   */
  async getPopularMovies(page: number = 1): Promise<TMDbSearchResult<TMDbMovie>> {
    return this.request<TMDbSearchResult<TMDbMovie>>('/movie/popular', {
      page: String(page),
      region: this.region,
    });
  }

  /**
   * Get popular TV shows
   */
  async getPopularTVShows(page: number = 1): Promise<TMDbSearchResult<TMDbTVShow>> {
    return this.request<TMDbSearchResult<TMDbTVShow>>('/tv/popular', {
      page: String(page),
    });
  }

  /**
   * Get top rated movies
   */
  async getTopRatedMovies(page: number = 1): Promise<TMDbSearchResult<TMDbMovie>> {
    return this.request<TMDbSearchResult<TMDbMovie>>('/movie/top_rated', {
      page: String(page),
      region: this.region,
    });
  }

  /**
   * Get top rated TV shows
   */
  async getTopRatedTVShows(page: number = 1): Promise<TMDbSearchResult<TMDbTVShow>> {
    return this.request<TMDbSearchResult<TMDbTVShow>>('/tv/top_rated', {
      page: String(page),
    });
  }

  /**
   * Get now playing movies
   */
  async getNowPlayingMovies(page: number = 1): Promise<TMDbSearchResult<TMDbMovie>> {
    return this.request<TMDbSearchResult<TMDbMovie>>('/movie/now_playing', {
      page: String(page),
      region: this.region,
    });
  }

  /**
   * Get upcoming movies
   */
  async getUpcomingMovies(page: number = 1): Promise<TMDbSearchResult<TMDbMovie>> {
    return this.request<TMDbSearchResult<TMDbMovie>>('/movie/upcoming', {
      page: String(page),
      region: this.region,
    });
  }

  /**
   * Get movie details
   */
  async getMovieDetails(movieId: number): Promise<TMDbMovie & { runtime: number; genres: Array<{ id: number; name: string }> }> {
    return this.request(`/movie/${movieId}`);
  }

  /**
   * Get TV show details
   */
  async getTVShowDetails(tvId: number): Promise<TMDbTVShow & { episode_run_time: number[]; genres: Array<{ id: number; name: string }> }> {
    return this.request(`/tv/${tvId}`);
  }

  /**
   * Get movie credits (cast & crew)
   */
  async getMovieCredits(movieId: number): Promise<TMDbCredits> {
    return this.request(`/movie/${movieId}/credits`);
  }

  /**
   * Get TV show credits
   */
  async getTVShowCredits(tvId: number): Promise<TMDbCredits> {
    return this.request(`/tv/${tvId}/credits`);
  }

  /**
   * Get movie watch providers
   */
  async getMovieWatchProviders(movieId: number): Promise<TMDbWatchProviders> {
    return this.request(`/movie/${movieId}/watch/providers`);
  }

  /**
   * Get TV show watch providers
   */
  async getTVShowWatchProviders(tvId: number): Promise<TMDbWatchProviders> {
    return this.request(`/tv/${tvId}/watch/providers`);
  }

  /**
   * Get similar movies
   */
  async getSimilarMovies(movieId: number, page: number = 1): Promise<TMDbSearchResult<TMDbMovie>> {
    return this.request<TMDbSearchResult<TMDbMovie>>(`/movie/${movieId}/similar`, {
      page: String(page),
    });
  }

  /**
   * Get similar TV shows
   */
  async getSimilarTVShows(tvId: number, page: number = 1): Promise<TMDbSearchResult<TMDbTVShow>> {
    return this.request<TMDbSearchResult<TMDbTVShow>>(`/tv/${tvId}/similar`, {
      page: String(page),
    });
  }

  /**
   * Get movie recommendations
   */
  async getMovieRecommendations(movieId: number, page: number = 1): Promise<TMDbSearchResult<TMDbMovie>> {
    return this.request<TMDbSearchResult<TMDbMovie>>(`/movie/${movieId}/recommendations`, {
      page: String(page),
    });
  }

  /**
   * Get TV show recommendations
   */
  async getTVShowRecommendations(tvId: number, page: number = 1): Promise<TMDbSearchResult<TMDbTVShow>> {
    return this.request<TMDbSearchResult<TMDbTVShow>>(`/tv/${tvId}/recommendations`, {
      page: String(page),
    });
  }

  /**
   * Discover movies with filters
   */
  async discoverMovies(filters: {
    genres?: number[];
    minRating?: number;
    maxRating?: number;
    minYear?: number;
    maxYear?: number;
    sortBy?: string;
    page?: number;
  } = {}): Promise<TMDbSearchResult<TMDbMovie>> {
    const params: Record<string, string> = {
      page: String(filters.page || 1),
      sort_by: filters.sortBy || 'popularity.desc',
      include_adult: String(this.includeAdult),
      region: this.region,
    };

    if (filters.genres?.length) {
      params.with_genres = filters.genres.join(',');
    }
    if (filters.minRating !== undefined) {
      params['vote_average.gte'] = String(filters.minRating);
    }
    if (filters.maxRating !== undefined) {
      params['vote_average.lte'] = String(filters.maxRating);
    }
    if (filters.minYear !== undefined) {
      params['primary_release_date.gte'] = `${filters.minYear}-01-01`;
    }
    if (filters.maxYear !== undefined) {
      params['primary_release_date.lte'] = `${filters.maxYear}-12-31`;
    }

    return this.request<TMDbSearchResult<TMDbMovie>>('/discover/movie', params);
  }

  /**
   * Discover TV shows with filters
   */
  async discoverTVShows(filters: {
    genres?: number[];
    minRating?: number;
    maxRating?: number;
    minYear?: number;
    maxYear?: number;
    sortBy?: string;
    page?: number;
  } = {}): Promise<TMDbSearchResult<TMDbTVShow>> {
    const params: Record<string, string> = {
      page: String(filters.page || 1),
      sort_by: filters.sortBy || 'popularity.desc',
      include_adult: String(this.includeAdult),
    };

    if (filters.genres?.length) {
      params.with_genres = filters.genres.join(',');
    }
    if (filters.minRating !== undefined) {
      params['vote_average.gte'] = String(filters.minRating);
    }
    if (filters.maxRating !== undefined) {
      params['vote_average.lte'] = String(filters.maxRating);
    }
    if (filters.minYear !== undefined) {
      params['first_air_date.gte'] = `${filters.minYear}-01-01`;
    }
    if (filters.maxYear !== undefined) {
      params['first_air_date.lte'] = `${filters.maxYear}-12-31`;
    }

    return this.request<TMDbSearchResult<TMDbTVShow>>('/discover/tv', params);
  }

  /**
   * Convert TMDb movie to ContentMetadata for learning system
   */
  async movieToContentMetadata(movie: TMDbMovie, includeDetails: boolean = false): Promise<ContentMetadata> {
    let runtime = movie.runtime;
    let actors: string[] = [];
    let directors: string[] = [];
    let keywords: string[] = [];
    let appId: string | undefined;
    let appName: string | undefined;

    if (includeDetails) {
      try {
        // Get full details
        const details = await this.getMovieDetails(movie.id);
        runtime = details.runtime;

        // Get credits
        const credits = await this.getMovieCredits(movie.id);
        actors = credits.cast.slice(0, 5).map(c => c.name);
        directors = credits.crew.filter(c => c.job === 'Director').map(c => c.name);

        // Get streaming availability
        const providers = await this.getMovieWatchProviders(movie.id);
        const usProviders = providers.results[this.region];
        if (usProviders?.flatrate?.length) {
          for (const provider of usProviders.flatrate) {
            const mapped = STREAMING_PROVIDERS[provider.provider_id];
            if (mapped) {
              appId = mapped.appId;
              appName = mapped.appName;
              break;
            }
          }
        }
      } catch {
        // Continue without details
      }
    }

    return {
      id: `tmdb-movie-${movie.id}`,
      title: movie.title,
      type: 'movie' as ContentType,
      genres: movie.genre_ids
        .map(id => GENRE_MAP[id])
        .filter((g): g is Genre => g !== undefined),
      duration: runtime || 120,
      rating: movie.vote_average,
      popularity: Math.min(100, movie.popularity),
      releaseYear: movie.release_date ? parseInt(movie.release_date.split('-')[0]) : undefined,
      description: movie.overview,
      posterUrl: movie.poster_path ? `${TMDB_IMAGE_BASE}/w500${movie.poster_path}` : undefined,
      backdropUrl: movie.backdrop_path ? `${TMDB_IMAGE_BASE}/original${movie.backdrop_path}` : undefined,
      actors,
      directors,
      keywords,
      appId,
      appName,
    };
  }

  /**
   * Convert TMDb TV show to ContentMetadata
   */
  async tvShowToContentMetadata(show: TMDbTVShow, includeDetails: boolean = false): Promise<ContentMetadata> {
    let runtime = show.episode_run_time?.[0];
    let actors: string[] = [];
    let directors: string[] = [];
    let keywords: string[] = [];
    let appId: string | undefined;
    let appName: string | undefined;

    if (includeDetails) {
      try {
        const details = await this.getTVShowDetails(show.id);
        runtime = details.episode_run_time?.[0];

        const credits = await this.getTVShowCredits(show.id);
        actors = credits.cast.slice(0, 5).map(c => c.name);
        directors = credits.crew.filter(c => c.job === 'Executive Producer').slice(0, 2).map(c => c.name);

        const providers = await this.getTVShowWatchProviders(show.id);
        const usProviders = providers.results[this.region];
        if (usProviders?.flatrate?.length) {
          for (const provider of usProviders.flatrate) {
            const mapped = STREAMING_PROVIDERS[provider.provider_id];
            if (mapped) {
              appId = mapped.appId;
              appName = mapped.appName;
              break;
            }
          }
        }
      } catch {
        // Continue without details
      }
    }

    return {
      id: `tmdb-tv-${show.id}`,
      title: show.name,
      type: 'tv_show' as ContentType,
      genres: show.genre_ids
        .map(id => GENRE_MAP[id])
        .filter((g): g is Genre => g !== undefined),
      duration: runtime || 45,
      rating: show.vote_average,
      popularity: Math.min(100, show.popularity),
      releaseYear: show.first_air_date ? parseInt(show.first_air_date.split('-')[0]) : undefined,
      description: show.overview,
      posterUrl: show.poster_path ? `${TMDB_IMAGE_BASE}/w500${show.poster_path}` : undefined,
      backdropUrl: show.backdrop_path ? `${TMDB_IMAGE_BASE}/original${show.backdrop_path}` : undefined,
      actors,
      directors,
      keywords,
      appId,
      appName,
    };
  }

  /**
   * Clear the cache
   */
  clearCache(): void {
    this.cache.clear();
  }

  /**
   * Get image URL
   */
  static getImageUrl(path: string | null, size: 'w92' | 'w154' | 'w185' | 'w342' | 'w500' | 'w780' | 'original' = 'w500'): string | null {
    if (!path) return null;
    return `${TMDB_IMAGE_BASE}/${size}${path}`;
  }
}

/**
 * Create TMDb client from environment or config
 */
export function createTMDbClient(apiKey?: string): TMDbClient {
  const key = apiKey || process.env.TMDB_API_KEY;
  if (!key) {
    throw new Error('TMDb API key required. Set TMDB_API_KEY environment variable or pass apiKey parameter.');
  }
  return new TMDbClient({ apiKey: key });
}
