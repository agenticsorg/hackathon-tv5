/**
 * TMDB Catalog - Fetches and converts TMDB content to EmotiStream format
 *
 * Converts real movie/TV data from TMDB API into ContentMetadata
 * that EmotiStream's recommendation engine can use.
 */

import {
  TMDBMovie,
  TMDBTVShow,
  MOVIE_GENRES,
  TV_GENRES,
  getImageUrl,
  getTrending,
  getPopularMovies,
  getPopularTVShows,
  getTopRatedMovies,
  getTopRatedTVShows,
  discoverMovies,
  discoverTVShows,
  isTMDBConfigured,
} from './tmdb-client.js';
import { ContentMetadata } from './types.js';

/**
 * Emotional tags derived from genres
 * Maps TMDB genres to emotional descriptors for the recommendation engine
 */
const GENRE_TO_EMOTIONAL_TAGS: Record<string, string[]> = {
  'action': ['intense', 'exciting', 'adrenaline', 'thrilling'],
  'adventure': ['exciting', 'uplifting', 'wonder', 'escapism'],
  'animation': ['fun', 'imaginative', 'colorful', 'whimsical'],
  'comedy': ['funny', 'lighthearted', 'feel-good', 'amusing'],
  'crime': ['tense', 'suspenseful', 'dark', 'gripping'],
  'documentary': ['educational', 'thought-provoking', 'informative', 'eye-opening'],
  'drama': ['emotional', 'moving', 'thought-provoking', 'intense'],
  'family': ['heartwarming', 'fun', 'wholesome', 'uplifting'],
  'fantasy': ['magical', 'escapism', 'wonder', 'imaginative'],
  'history': ['educational', 'thought-provoking', 'dramatic', 'inspiring'],
  'horror': ['scary', 'tense', 'thrilling', 'dark'],
  'music': ['uplifting', 'emotional', 'inspiring', 'energizing'],
  'mystery': ['intriguing', 'suspenseful', 'thought-provoking', 'engaging'],
  'romance': ['romantic', 'emotional', 'heartwarming', 'feel-good'],
  'sci-fi': ['thought-provoking', 'imaginative', 'exciting', 'wonder'],
  'thriller': ['tense', 'suspenseful', 'gripping', 'intense'],
  'war': ['intense', 'emotional', 'dramatic', 'thought-provoking'],
  'western': ['adventurous', 'dramatic', 'nostalgic', 'rugged'],
  // TV-specific
  'action-adventure': ['exciting', 'thrilling', 'adventurous', 'intense'],
  'kids': ['fun', 'colorful', 'educational', 'lighthearted'],
  'reality': ['engaging', 'dramatic', 'entertaining', 'relatable'],
  'sci-fi-fantasy': ['imaginative', 'wonder', 'escapism', 'exciting'],
  'talk': ['informative', 'entertaining', 'engaging', 'conversational'],
  'war-politics': ['intense', 'thought-provoking', 'dramatic', 'tense'],
};

/**
 * Convert TMDB movie to ContentMetadata
 */
function movieToContentMetadata(movie: TMDBMovie): ContentMetadata {
  const genreIds = movie.genre_ids || [];
  const genres = genreIds
    .map(id => MOVIE_GENRES[id])
    .filter(Boolean);

  const tags = genres
    .flatMap(genre => GENRE_TO_EMOTIONAL_TAGS[genre] || [])
    .filter((tag, idx, arr) => arr.indexOf(tag) === idx) // unique
    .slice(0, 6);

  // Add rating-based tags
  if (movie.vote_average >= 8) tags.push('critically-acclaimed');
  if (movie.popularity > 100) tags.push('popular');

  return {
    contentId: `tmdb_movie_${movie.id}`,
    title: movie.title,
    description: movie.overview || 'No description available.',
    platform: 'tmdb',
    genres,
    category: 'movie',
    tags,
    duration: movie.runtime || 120, // default 2 hours if unknown
    tmdbId: movie.id,
    posterUrl: getImageUrl(movie.poster_path, 'w500'),
    backdropUrl: getImageUrl(movie.backdrop_path, 'w780'),
    releaseDate: movie.release_date,
    rating: movie.vote_average,
    popularity: movie.popularity,
  };
}

/**
 * Convert TMDB TV show to ContentMetadata
 */
function tvShowToContentMetadata(show: TMDBTVShow): ContentMetadata {
  const genreIds = show.genre_ids || [];
  const genres = genreIds
    .map(id => TV_GENRES[id] || MOVIE_GENRES[id])
    .filter(Boolean);

  const tags = genres
    .flatMap(genre => GENRE_TO_EMOTIONAL_TAGS[genre] || [])
    .filter((tag, idx, arr) => arr.indexOf(tag) === idx)
    .slice(0, 6);

  // Add rating-based tags
  if (show.vote_average >= 8) tags.push('critically-acclaimed');
  if (show.popularity > 100) tags.push('popular');
  tags.push('binge-worthy'); // TV shows are bingeable

  // Estimate episode duration (default 45 min if unknown)
  const episodeDuration = show.episode_run_time?.[0] || 45;

  return {
    contentId: `tmdb_tv_${show.id}`,
    title: show.name,
    description: show.overview || 'No description available.',
    platform: 'tmdb',
    genres,
    category: 'series',
    tags,
    duration: episodeDuration,
    tmdbId: show.id,
    posterUrl: getImageUrl(show.poster_path, 'w500'),
    backdropUrl: getImageUrl(show.backdrop_path, 'w780'),
    releaseDate: show.first_air_date,
    rating: show.vote_average,
    popularity: show.popularity,
  };
}

/**
 * TMDBCatalog - Fetches real content from TMDB
 */
export class TMDBCatalog {
  private cache: Map<string, ContentMetadata> = new Map();
  private lastFetch: number = 0;
  private cacheTTL: number = 30 * 60 * 1000; // 30 minutes

  /**
   * Check if TMDB is available
   */
  isAvailable(): boolean {
    return isTMDBConfigured();
  }

  /**
   * Fetch a diverse catalog of content
   * Returns mix of trending, popular, and top-rated content
   */
  async fetchCatalog(count: number = 100): Promise<ContentMetadata[]> {
    if (!this.isAvailable()) {
      console.warn('TMDB not configured, returning empty catalog');
      return [];
    }

    // Check cache
    if (this.cache.size >= count && Date.now() - this.lastFetch < this.cacheTTL) {
      return Array.from(this.cache.values()).slice(0, count);
    }

    console.log(`Fetching ${count} items from TMDB...`);

    try {
      const itemsPerCategory = Math.ceil(count / 6);
      const pages = Math.ceil(itemsPerCategory / 20); // TMDB returns 20 per page

      // Fetch in parallel for speed
      const [
        trendingData,
        popularMoviesData,
        popularTVData,
        topMoviesData,
        topTVData,
      ] = await Promise.all([
        getTrending('all', 'week'),
        this.fetchMultiplePages(getPopularMovies, pages),
        this.fetchMultiplePages(getPopularTVShows, pages),
        this.fetchMultiplePages(getTopRatedMovies, pages),
        this.fetchMultiplePages(getTopRatedTVShows, pages),
      ]);

      // Convert to ContentMetadata
      const catalog: ContentMetadata[] = [];

      // Process trending (mix of movies and TV)
      for (const item of trendingData) {
        if ('title' in item) {
          catalog.push(movieToContentMetadata(item as TMDBMovie));
        } else if ('name' in item) {
          catalog.push(tvShowToContentMetadata(item as TMDBTVShow));
        }
      }

      // Process movies
      for (const movie of popularMoviesData) {
        catalog.push(movieToContentMetadata(movie));
      }
      for (const movie of topMoviesData) {
        catalog.push(movieToContentMetadata(movie));
      }

      // Process TV shows
      for (const show of popularTVData) {
        catalog.push(tvShowToContentMetadata(show));
      }
      for (const show of topTVData) {
        catalog.push(tvShowToContentMetadata(show));
      }

      // Deduplicate by contentId
      const uniqueCatalog = catalog.filter((item, idx, arr) =>
        arr.findIndex(i => i.contentId === item.contentId) === idx
      );

      // Update cache
      this.cache.clear();
      for (const item of uniqueCatalog) {
        this.cache.set(item.contentId, item);
      }
      this.lastFetch = Date.now();

      console.log(`Fetched ${uniqueCatalog.length} unique items from TMDB`);

      return uniqueCatalog.slice(0, count);
    } catch (error) {
      console.error('Error fetching TMDB catalog:', error);
      // Return cached data if available
      if (this.cache.size > 0) {
        return Array.from(this.cache.values()).slice(0, count);
      }
      return [];
    }
  }

  /**
   * Fetch content by genre
   */
  async fetchByGenre(genreId: number, mediaType: 'movie' | 'tv' = 'movie', count: number = 20): Promise<ContentMetadata[]> {
    if (!this.isAvailable()) return [];

    try {
      if (mediaType === 'movie') {
        const { results } = await discoverMovies({ genres: [genreId], ratingMin: 6 });
        return results.slice(0, count).map(movieToContentMetadata);
      } else {
        const { results } = await discoverTVShows({ genres: [genreId], ratingMin: 6 });
        return results.slice(0, count).map(tvShowToContentMetadata);
      }
    } catch (error) {
      console.error('Error fetching by genre:', error);
      return [];
    }
  }

  /**
   * Fetch content for specific emotional needs
   */
  async fetchForMood(mood: 'happy' | 'sad' | 'stressed' | 'bored' | 'anxious', count: number = 20): Promise<ContentMetadata[]> {
    if (!this.isAvailable()) return [];

    // Map moods to genres
    const moodToGenres: Record<string, number[]> = {
      'happy': [35, 10751, 16], // Comedy, Family, Animation
      'sad': [18, 10749], // Drama, Romance (cathartic)
      'stressed': [35, 16, 10402], // Comedy, Animation, Music
      'bored': [28, 12, 878], // Action, Adventure, Sci-Fi
      'anxious': [35, 10751, 99], // Comedy, Family, Documentary
    };

    const genres = moodToGenres[mood] || [35];

    try {
      const [movies, shows] = await Promise.all([
        discoverMovies({ genres, ratingMin: 7 }),
        discoverTVShows({ genres, ratingMin: 7 }),
      ]);

      const catalog = [
        ...movies.results.map(movieToContentMetadata),
        ...shows.results.map(tvShowToContentMetadata),
      ];

      return catalog.slice(0, count);
    } catch (error) {
      console.error('Error fetching for mood:', error);
      return [];
    }
  }

  /**
   * Get a single item from cache or fetch
   */
  async getContent(contentId: string): Promise<ContentMetadata | null> {
    if (this.cache.has(contentId)) {
      return this.cache.get(contentId)!;
    }

    // If not in cache, we'd need to fetch by ID
    // For MVP, just return null if not cached
    return null;
  }

  /**
   * Helper: Fetch multiple pages and flatten results
   */
  private async fetchMultiplePages<T>(
    fetcher: (page: number) => Promise<{ results: T[]; totalPages: number }>,
    pages: number
  ): Promise<T[]> {
    const results: T[] = [];
    for (let page = 1; page <= pages; page++) {
      const { results: pageResults } = await fetcher(page);
      results.push(...pageResults);
    }
    return results;
  }

  /**
   * Clear the cache
   */
  clearCache(): void {
    this.cache.clear();
    this.lastFetch = 0;
  }
}

// Export singleton instance
export const tmdbCatalog = new TMDBCatalog();
