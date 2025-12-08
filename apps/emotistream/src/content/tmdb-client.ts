/**
 * TMDB Client - Lightweight client for fetching movie/TV content
 *
 * Uses TMDB API v3 with Bearer token authentication.
 * Free tier: ~40 requests/second, unlimited daily.
 */

const TMDB_BASE_URL = 'https://api.themoviedb.org/3';
const TMDB_IMAGE_BASE = 'https://image.tmdb.org/t/p';

// Get token from environment
const getToken = (): string | null => {
  return process.env.TMDB_ACCESS_TOKEN || null;
};

/**
 * TMDB API response types
 */
export interface TMDBMovie {
  id: number;
  title: string;
  overview: string;
  poster_path: string | null;
  backdrop_path: string | null;
  release_date: string;
  vote_average: number;
  vote_count: number;
  popularity: number;
  genre_ids: number[];
  adult: boolean;
  original_language: string;
  runtime?: number;
}

export interface TMDBTVShow {
  id: number;
  name: string;
  overview: string;
  poster_path: string | null;
  backdrop_path: string | null;
  first_air_date: string;
  vote_average: number;
  vote_count: number;
  popularity: number;
  genre_ids: number[];
  origin_country: string[];
  original_language: string;
  episode_run_time?: number[];
}

export interface TMDBGenre {
  id: number;
  name: string;
}

interface TMDBResponse<T> {
  page: number;
  results: T[];
  total_pages: number;
  total_results: number;
}

/**
 * Genre ID to name mapping (TMDB standard)
 */
export const MOVIE_GENRES: Record<number, string> = {
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
  878: 'sci-fi',
  10770: 'tv-movie',
  53: 'thriller',
  10752: 'war',
  37: 'western',
};

export const TV_GENRES: Record<number, string> = {
  10759: 'action-adventure',
  16: 'animation',
  35: 'comedy',
  80: 'crime',
  99: 'documentary',
  18: 'drama',
  10751: 'family',
  10762: 'kids',
  9648: 'mystery',
  10763: 'news',
  10764: 'reality',
  10765: 'sci-fi-fantasy',
  10766: 'soap',
  10767: 'talk',
  10768: 'war-politics',
  37: 'western',
};

/**
 * Make authenticated request to TMDB API
 */
async function tmdbFetch<T>(endpoint: string, params: Record<string, string> = {}): Promise<T> {
  const token = getToken();
  if (!token) {
    throw new Error('TMDB_ACCESS_TOKEN not configured. Add it to your .env file.');
  }

  const url = new URL(`${TMDB_BASE_URL}${endpoint}`);
  Object.entries(params).forEach(([key, value]) => {
    url.searchParams.append(key, value);
  });

  const response = await fetch(url.toString(), {
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
  });

  if (!response.ok) {
    throw new Error(`TMDB API error: ${response.status} ${response.statusText}`);
  }

  return response.json() as Promise<T>;
}

/**
 * Get image URL from TMDB path
 */
export function getImageUrl(path: string | null, size: 'w92' | 'w154' | 'w185' | 'w342' | 'w500' | 'w780' | 'original' = 'w500'): string | null {
  if (!path) return null;
  return `${TMDB_IMAGE_BASE}/${size}${path}`;
}

/**
 * Get trending movies and TV shows
 */
export async function getTrending(
  mediaType: 'movie' | 'tv' | 'all' = 'all',
  timeWindow: 'day' | 'week' = 'week'
): Promise<(TMDBMovie | TMDBTVShow)[]> {
  const response = await tmdbFetch<TMDBResponse<TMDBMovie | TMDBTVShow>>(
    `/trending/${mediaType}/${timeWindow}`
  );
  return response.results;
}

/**
 * Get popular movies
 */
export async function getPopularMovies(page = 1): Promise<{ results: TMDBMovie[]; totalPages: number }> {
  const response = await tmdbFetch<TMDBResponse<TMDBMovie>>(
    '/movie/popular',
    { page: page.toString() }
  );
  return {
    results: response.results,
    totalPages: response.total_pages,
  };
}

/**
 * Get popular TV shows
 */
export async function getPopularTVShows(page = 1): Promise<{ results: TMDBTVShow[]; totalPages: number }> {
  const response = await tmdbFetch<TMDBResponse<TMDBTVShow>>(
    '/tv/popular',
    { page: page.toString() }
  );
  return {
    results: response.results,
    totalPages: response.total_pages,
  };
}

/**
 * Get top rated movies
 */
export async function getTopRatedMovies(page = 1): Promise<{ results: TMDBMovie[]; totalPages: number }> {
  const response = await tmdbFetch<TMDBResponse<TMDBMovie>>(
    '/movie/top_rated',
    { page: page.toString() }
  );
  return {
    results: response.results,
    totalPages: response.total_pages,
  };
}

/**
 * Get top rated TV shows
 */
export async function getTopRatedTVShows(page = 1): Promise<{ results: TMDBTVShow[]; totalPages: number }> {
  const response = await tmdbFetch<TMDBResponse<TMDBTVShow>>(
    '/tv/top_rated',
    { page: page.toString() }
  );
  return {
    results: response.results,
    totalPages: response.total_pages,
  };
}

/**
 * Search for movies
 */
export async function searchMovies(query: string, page = 1): Promise<{ results: TMDBMovie[]; totalPages: number }> {
  const response = await tmdbFetch<TMDBResponse<TMDBMovie>>(
    '/search/movie',
    { query, page: page.toString() }
  );
  return {
    results: response.results,
    totalPages: response.total_pages,
  };
}

/**
 * Search for TV shows
 */
export async function searchTVShows(query: string, page = 1): Promise<{ results: TMDBTVShow[]; totalPages: number }> {
  const response = await tmdbFetch<TMDBResponse<TMDBTVShow>>(
    '/search/tv',
    { query, page: page.toString() }
  );
  return {
    results: response.results,
    totalPages: response.total_pages,
  };
}

/**
 * Discover movies with filters
 */
export async function discoverMovies(options: {
  genres?: number[];
  yearMin?: number;
  yearMax?: number;
  ratingMin?: number;
  sortBy?: string;
  page?: number;
} = {}): Promise<{ results: TMDBMovie[]; totalPages: number }> {
  const params: Record<string, string> = {
    page: (options.page || 1).toString(),
    sort_by: options.sortBy || 'popularity.desc',
  };

  if (options.genres?.length) {
    params.with_genres = options.genres.join(',');
  }
  if (options.yearMin) {
    params['primary_release_date.gte'] = `${options.yearMin}-01-01`;
  }
  if (options.yearMax) {
    params['primary_release_date.lte'] = `${options.yearMax}-12-31`;
  }
  if (options.ratingMin) {
    params['vote_average.gte'] = options.ratingMin.toString();
  }

  const response = await tmdbFetch<TMDBResponse<TMDBMovie>>('/discover/movie', params);
  return {
    results: response.results,
    totalPages: response.total_pages,
  };
}

/**
 * Discover TV shows with filters
 */
export async function discoverTVShows(options: {
  genres?: number[];
  yearMin?: number;
  yearMax?: number;
  ratingMin?: number;
  sortBy?: string;
  page?: number;
} = {}): Promise<{ results: TMDBTVShow[]; totalPages: number }> {
  const params: Record<string, string> = {
    page: (options.page || 1).toString(),
    sort_by: options.sortBy || 'popularity.desc',
  };

  if (options.genres?.length) {
    params.with_genres = options.genres.join(',');
  }
  if (options.yearMin) {
    params['first_air_date.gte'] = `${options.yearMin}-01-01`;
  }
  if (options.yearMax) {
    params['first_air_date.lte'] = `${options.yearMax}-12-31`;
  }
  if (options.ratingMin) {
    params['vote_average.gte'] = options.ratingMin.toString();
  }

  const response = await tmdbFetch<TMDBResponse<TMDBTVShow>>('/discover/tv', params);
  return {
    results: response.results,
    totalPages: response.total_pages,
  };
}

/**
 * Get movie details
 */
export async function getMovieDetails(id: number): Promise<TMDBMovie & { runtime: number; genres: TMDBGenre[] }> {
  return tmdbFetch(`/movie/${id}`);
}

/**
 * Get TV show details
 */
export async function getTVShowDetails(id: number): Promise<TMDBTVShow & { episode_run_time: number[]; genres: TMDBGenre[] }> {
  return tmdbFetch(`/tv/${id}`);
}

/**
 * Check if TMDB is configured
 */
export function isTMDBConfigured(): boolean {
  return !!getToken();
}
