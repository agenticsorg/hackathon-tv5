/**
 * TMDB API Integration
 * Provides typed access to The Movie Database API
 */

import { TMDB } from 'tmdb-ts';
import type { Movie, TVShow, MediaContent, Genre, SearchFilters } from '@/types/media';

// Environment validation
const TMDB_ACCESS_TOKEN = process.env.NEXT_PUBLIC_TMDB_ACCESS_TOKEN;

if (!TMDB_ACCESS_TOKEN) {
  console.warn('TMDB_ACCESS_TOKEN is not defined. TMDB API calls will fail.');
}

// Initialize TMDB client
export const tmdb = TMDB_ACCESS_TOKEN ? new TMDB(TMDB_ACCESS_TOKEN) : null;

/**
 * Image URL helpers
 */
export const TMDB_IMAGE_BASE = 'https://image.tmdb.org/t/p';

export const getImageUrl = (path: string | null, size: 'w500' | 'w780' | 'original' = 'w500'): string | null => {
  if (!path) return null;
  return `${TMDB_IMAGE_BASE}/${size}${path}`;
};

export const getPosterUrl = (path: string | null): string | null => getImageUrl(path, 'w500');
export const getBackdropUrl = (path: string | null): string | null => getImageUrl(path, 'w780');

/**
 * Search for movies and TV shows
 */
export async function searchMulti(
  query: string,
  filters?: SearchFilters,
  page = 1
): Promise<{ results: MediaContent[]; totalPages: number; totalResults: number }> {
  if (!tmdb) throw new Error('TMDB client not initialized');

  const response = await tmdb.search.multi({ query, page });

  // Filter and transform results
  let results: MediaContent[] = response.results
    .filter(item => item.media_type === 'movie' || item.media_type === 'tv')
    .map(item => transformToMediaContent(item));

  // Apply filters
  if (filters) {
    results = applyFilters(results, filters);
  }

  return {
    results,
    totalPages: response.total_pages,
    totalResults: response.total_results,
  };
}

/**
 * Get trending content
 */
export async function getTrending(
  mediaType: 'movie' | 'tv' | 'all' = 'all',
  timeWindow: 'day' | 'week' = 'week'
): Promise<MediaContent[]> {
  if (!tmdb) throw new Error('TMDB client not initialized');

  const response = await tmdb.trending.trending(mediaType, timeWindow);

  return response.results.map(item => transformToMediaContent(item));
}

/**
 * Get popular movies
 */
export async function getPopularMovies(page = 1): Promise<{ results: Movie[]; totalPages: number }> {
  if (!tmdb) throw new Error('TMDB client not initialized');

  const response = await tmdb.movies.popular({ page });

  return {
    results: response.results.map(movie => transformToMovie(movie)),
    totalPages: response.total_pages,
  };
}

/**
 * Get popular TV shows
 */
export async function getPopularTVShows(page = 1): Promise<{ results: TVShow[]; totalPages: number }> {
  if (!tmdb) throw new Error('TMDB client not initialized');

  const response = await tmdb.tvShows.popular({ page });

  return {
    results: response.results.map(show => transformToTVShow(show)),
    totalPages: response.total_pages,
  };
}

/**
 * Get movie details (basic)
 */
export async function getMovieDetails(id: number): Promise<Movie> {
  if (!tmdb) throw new Error('TMDB client not initialized');

  const movie = await tmdb.movies.details(id);
  return transformToMovie(movie);
}

/**
 * Get full movie details with appended data (credits, videos, similar, recommendations)
 */
export async function getFullMovieDetails(id: number): Promise<{
  movie: Movie;
  credits: { cast: CastMember[]; crew: CrewMember[] };
  videos: Video[];
  similar: Movie[];
  recommendations: Movie[];
  genres: Genre[];
}> {
  if (!tmdb) throw new Error('TMDB client not initialized');

  const movie = await tmdb.movies.details(id, [
    'credits',
    'videos',
    'similar',
    'recommendations',
  ] as any);

  return {
    movie: transformToMovie(movie),
    credits: {
      cast: ((movie as any).credits?.cast || []).slice(0, 20).map((c: any) => ({
        id: c.id,
        name: c.name,
        character: c.character,
        profilePath: c.profile_path,
        order: c.order,
      })),
      crew: ((movie as any).credits?.crew || []).slice(0, 10).map((c: any) => ({
        id: c.id,
        name: c.name,
        job: c.job,
        department: c.department,
        profilePath: c.profile_path,
      })),
    },
    videos: ((movie as any).videos?.results || []).map((v: any) => ({
      id: v.id,
      key: v.key,
      name: v.name,
      site: v.site,
      type: v.type,
    })),
    similar: ((movie as any).similar?.results || []).slice(0, 12).map((m: any) => transformToMovie(m)),
    recommendations: ((movie as any).recommendations?.results || []).slice(0, 12).map((m: any) => transformToMovie(m)),
    genres: (movie as any).genres || [],
  };
}

/**
 * Get TV show details (basic)
 */
export async function getTVShowDetails(id: number): Promise<TVShow> {
  if (!tmdb) throw new Error('TMDB client not initialized');

  const show = await tmdb.tvShows.details(id);
  return transformToTVShow(show);
}

/**
 * Get full TV show details with appended data
 */
export async function getFullTVShowDetails(id: number): Promise<{
  show: TVShow;
  credits: { cast: CastMember[]; crew: CrewMember[] };
  videos: Video[];
  similar: TVShow[];
  recommendations: TVShow[];
  genres: Genre[];
}> {
  if (!tmdb) throw new Error('TMDB client not initialized');

  const show = await tmdb.tvShows.details(id, [
    'credits',
    'videos',
    'similar',
    'recommendations',
  ] as any);

  return {
    show: transformToTVShow(show),
    credits: {
      cast: ((show as any).credits?.cast || []).slice(0, 20).map((c: any) => ({
        id: c.id,
        name: c.name,
        character: c.character,
        profilePath: c.profile_path,
        order: c.order,
      })),
      crew: ((show as any).credits?.crew || []).slice(0, 10).map((c: any) => ({
        id: c.id,
        name: c.name,
        job: c.job,
        department: c.department,
        profilePath: c.profile_path,
      })),
    },
    videos: ((show as any).videos?.results || []).map((v: any) => ({
      id: v.id,
      key: v.key,
      name: v.name,
      site: v.site,
      type: v.type,
    })),
    similar: ((show as any).similar?.results || []).slice(0, 12).map((s: any) => transformToTVShow(s)),
    recommendations: ((show as any).recommendations?.results || []).slice(0, 12).map((s: any) => transformToTVShow(s)),
    genres: (show as any).genres || [],
  };
}

// Types for credits and videos
export interface CastMember {
  id: number;
  name: string;
  character: string;
  profilePath: string | null;
  order: number;
}

export interface CrewMember {
  id: number;
  name: string;
  job: string;
  department: string;
  profilePath: string | null;
}

export interface Video {
  id: string;
  key: string;
  name: string;
  site: string;
  type: string;
}

/**
 * Get similar movies
 */
export async function getSimilarMovies(id: number, page = 1): Promise<Movie[]> {
  if (!tmdb) throw new Error('TMDB client not initialized');

  const response = await tmdb.movies.similar(id, { page });
  return response.results.map(movie => transformToMovie(movie));
}

/**
 * Get similar TV shows
 */
export async function getSimilarTVShows(id: number, page = 1): Promise<TVShow[]> {
  if (!tmdb) throw new Error('TMDB client not initialized');

  const response = await tmdb.tvShows.similar(id, { page });
  return response.results.map(show => transformToTVShow(show));
}

/**
 * Get all genres
 */
export async function getGenres(): Promise<{ movies: Genre[]; tv: Genre[] }> {
  if (!tmdb) throw new Error('TMDB client not initialized');

  const [movieGenres, tvGenres] = await Promise.all([
    tmdb.genres.movies(),
    tmdb.genres.tvShows(),
  ]);

  return {
    movies: movieGenres.genres,
    tv: tvGenres.genres,
  };
}

/**
 * Discover movies with filters
 */
export async function discoverMovies(
  options: {
    genres?: number[];
    yearMin?: number;
    yearMax?: number;
    ratingMin?: number;
    sortBy?: string;
    page?: number;
  } = {}
): Promise<{ results: Movie[]; totalPages: number }> {
  if (!tmdb) throw new Error('TMDB client not initialized');

  const params: Record<string, string | number> = {
    page: options.page || 1,
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
    params['vote_average.gte'] = options.ratingMin;
  }

  const response = await tmdb.discover.movie(params);

  return {
    results: response.results.map(movie => transformToMovie(movie)),
    totalPages: response.total_pages,
  };
}

/**
 * Discover TV shows with filters
 */
export async function discoverTVShows(
  options: {
    genres?: number[];
    yearMin?: number;
    yearMax?: number;
    ratingMin?: number;
    sortBy?: string;
    page?: number;
  } = {}
): Promise<{ results: TVShow[]; totalPages: number }> {
  if (!tmdb) throw new Error('TMDB client not initialized');

  const params: Record<string, string | number> = {
    page: options.page || 1,
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
    params['vote_average.gte'] = options.ratingMin;
  }

  // Direct fetch as tmdb-ts may not support discover.tv
  const queryParams = new URLSearchParams();
  Object.entries(params).forEach(([key, value]) => {
    queryParams.append(key, String(value));
  });

  const response = await fetch(
    `https://api.themoviedb.org/3/discover/tv?${queryParams.toString()}`,
    {
      headers: {
        Authorization: `Bearer ${TMDB_ACCESS_TOKEN}`,
        'Content-Type': 'application/json',
      },
    }
  );

  if (!response.ok) {
    throw new Error(`TMDB API error: ${response.status}`);
  }

  const data = await response.json();

  return {
    results: data.results.map((show: any) => transformToTVShow(show)),
    totalPages: data.total_pages,
  };
}

// Helper: Transform TMDB response to MediaContent
function transformToMediaContent(item: any): MediaContent {
  const isTV = item.media_type === 'tv' || 'first_air_date' in item;

  return {
    id: item.id,
    title: isTV ? item.name : item.title,
    overview: item.overview || '',
    posterPath: item.poster_path,
    backdropPath: item.backdrop_path,
    releaseDate: isTV ? item.first_air_date : item.release_date,
    voteAverage: item.vote_average,
    voteCount: item.vote_count,
    popularity: item.popularity,
    genreIds: item.genre_ids || (item.genres?.map((g: any) => g.id) ?? []),
    mediaType: isTV ? 'tv' : 'movie',
  };
}

// Helper: Transform to Movie type
function transformToMovie(movie: any): Movie {
  return {
    ...transformToMediaContent({ ...movie, media_type: 'movie' }),
    mediaType: 'movie',
    runtime: movie.runtime,
    budget: movie.budget,
    revenue: movie.revenue,
    tagline: movie.tagline,
    status: movie.status,
  };
}

// Helper: Transform to TVShow type
function transformToTVShow(show: any): TVShow {
  return {
    ...transformToMediaContent({ ...show, media_type: 'tv' }),
    mediaType: 'tv',
    name: show.name,
    firstAirDate: show.first_air_date,
    lastAirDate: show.last_air_date,
    numberOfSeasons: show.number_of_seasons,
    numberOfEpisodes: show.number_of_episodes,
    episodeRunTime: show.episode_run_time,
    status: show.status,
    inProduction: show.in_production,
  };
}

// Helper: Apply search filters
function applyFilters(results: MediaContent[], filters: SearchFilters): MediaContent[] {
  return results.filter(item => {
    // Media type filter
    if (filters.mediaType && filters.mediaType !== 'all' && item.mediaType !== filters.mediaType) {
      return false;
    }

    // Genre filter
    if (filters.genres?.length) {
      const hasMatchingGenre = item.genreIds.some(id => filters.genres!.includes(id));
      if (!hasMatchingGenre) return false;
    }

    // Year range filter
    if (filters.yearRange) {
      const year = new Date(item.releaseDate).getFullYear();
      if (filters.yearRange.min && year < filters.yearRange.min) return false;
      if (filters.yearRange.max && year > filters.yearRange.max) return false;
    }

    // Rating filter
    if (filters.ratingMin && item.voteAverage < filters.ratingMin) {
      return false;
    }

    return true;
  });
}
