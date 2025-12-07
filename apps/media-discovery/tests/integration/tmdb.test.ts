/**
 * TMDB API Integration Tests
 *
 * Tests real API calls to The Movie Database
 * Requires NEXT_PUBLIC_TMDB_ACCESS_TOKEN environment variable
 */

import { describe, it, expect, beforeAll } from 'vitest';
import {
  searchMulti,
  getTrending,
  getPopularMovies,
  getPopularTVShows,
  getMovieDetails,
  getTVShowDetails,
  getSimilarMovies,
  getSimilarTVShows,
  getGenres,
  discoverMovies,
  discoverTVShows,
  tmdb,
} from '@/lib/tmdb';

// Skip all tests if no API key
const describeWithAPI = process.env.NEXT_PUBLIC_TMDB_ACCESS_TOKEN
  ? describe
  : describe.skip;

describeWithAPI('TMDB API Integration', () => {
  beforeAll(() => {
    expect(tmdb).not.toBeNull();
  });

  describe('Search', () => {
    it('should search for "Inception" and return relevant results', async () => {
      const { results, totalResults } = await searchMulti('Inception');

      expect(results.length).toBeGreaterThan(0);
      expect(totalResults).toBeGreaterThan(0);

      // Should find the movie "Inception" (2010)
      const inception = results.find(r =>
        r.title.toLowerCase().includes('inception') && r.mediaType === 'movie'
      );
      expect(inception).toBeDefined();
      expect(inception?.voteAverage).toBeGreaterThan(7);
    });

    it('should search for "Breaking Bad" TV series', async () => {
      const { results } = await searchMulti('Breaking Bad');

      expect(results.length).toBeGreaterThan(0);

      const breakingBad = results.find(r =>
        r.title.toLowerCase().includes('breaking bad') && r.mediaType === 'tv'
      );
      expect(breakingBad).toBeDefined();
      expect(breakingBad?.voteAverage).toBeGreaterThan(8);
    });

    it('should filter results by media type', async () => {
      const { results } = await searchMulti('Star Wars', { mediaType: 'movie' });

      expect(results.length).toBeGreaterThan(0);
      expect(results.every(r => r.mediaType === 'movie')).toBe(true);
    });

    it('should handle empty search results gracefully', async () => {
      const { results } = await searchMulti('xyznonexistentmovie12345');

      expect(results).toEqual([]);
    });
  });

  describe('Trending', () => {
    it('should return trending content for the week', async () => {
      const trending = await getTrending('all', 'week');

      expect(trending.length).toBeGreaterThan(0);
      expect(trending[0]).toHaveProperty('id');
      expect(trending[0]).toHaveProperty('title');
      expect(trending[0]).toHaveProperty('mediaType');
    });

    it('should return trending movies only', async () => {
      const trending = await getTrending('movie', 'day');

      expect(trending.length).toBeGreaterThan(0);
      expect(trending.every(t => t.mediaType === 'movie')).toBe(true);
    });

    it('should return trending TV shows only', async () => {
      const trending = await getTrending('tv', 'week');

      expect(trending.length).toBeGreaterThan(0);
      expect(trending.every(t => t.mediaType === 'tv')).toBe(true);
    });
  });

  describe('Popular Content', () => {
    it('should return popular movies', async () => {
      const { results, totalPages } = await getPopularMovies();

      expect(results.length).toBeGreaterThan(0);
      expect(totalPages).toBeGreaterThan(0);
      expect(results[0].mediaType).toBe('movie');
    });

    it('should return popular TV shows', async () => {
      const { results, totalPages } = await getPopularTVShows();

      expect(results.length).toBeGreaterThan(0);
      expect(totalPages).toBeGreaterThan(0);
      expect(results[0].mediaType).toBe('tv');
    });
  });

  describe('Content Details', () => {
    it('should get movie details for The Shawshank Redemption (ID: 278)', async () => {
      const movie = await getMovieDetails(278);

      expect(movie.id).toBe(278);
      expect(movie.title).toBe('The Shawshank Redemption');
      expect(movie.mediaType).toBe('movie');
      expect(movie.voteAverage).toBeGreaterThan(8);
      expect(movie.overview).toBeTruthy();
    });

    it('should get TV show details for Game of Thrones (ID: 1399)', async () => {
      const show = await getTVShowDetails(1399);

      expect(show.id).toBe(1399);
      expect(show.name).toBe('Game of Thrones');
      expect(show.mediaType).toBe('tv');
      expect(show.numberOfSeasons).toBeGreaterThan(0);
    });
  });

  describe('Similar Content', () => {
    it('should get movies similar to The Dark Knight (ID: 155)', async () => {
      const similar = await getSimilarMovies(155);

      expect(similar.length).toBeGreaterThan(0);
      expect(similar[0].mediaType).toBe('movie');
    });

    it('should get TV shows similar to Stranger Things (ID: 66732)', async () => {
      const similar = await getSimilarTVShows(66732);

      expect(similar.length).toBeGreaterThan(0);
      expect(similar[0].mediaType).toBe('tv');
    });
  });

  describe('Genres', () => {
    it('should return movie and TV genres', async () => {
      const { movies, tv } = await getGenres();

      expect(movies.length).toBeGreaterThan(0);
      expect(tv.length).toBeGreaterThan(0);

      // Common genres should exist
      expect(movies.some(g => g.name === 'Action')).toBe(true);
      expect(movies.some(g => g.name === 'Comedy')).toBe(true);
      expect(tv.some(g => g.name === 'Drama')).toBe(true);
    });
  });

  describe('Discover', () => {
    it('should discover movies by genre (Action = 28)', async () => {
      const { results } = await discoverMovies({ genres: [28] });

      expect(results.length).toBeGreaterThan(0);
      expect(results.every(r => r.genreIds.includes(28))).toBe(true);
    });

    it('should discover high-rated movies (8+)', async () => {
      const { results } = await discoverMovies({ ratingMin: 8 });

      expect(results.length).toBeGreaterThan(0);
      expect(results.every(r => r.voteAverage >= 8)).toBe(true);
    });

    it('should discover TV shows by genre (Sci-Fi & Fantasy = 10765)', async () => {
      const { results } = await discoverTVShows({ genres: [10765] });

      expect(results.length).toBeGreaterThan(0);
    });

    it('should discover movies from specific year range', async () => {
      const { results } = await discoverMovies({ yearMin: 2020, yearMax: 2024 });

      expect(results.length).toBeGreaterThan(0);

      // Verify all results are within year range
      for (const movie of results) {
        const year = new Date(movie.releaseDate).getFullYear();
        expect(year).toBeGreaterThanOrEqual(2020);
        expect(year).toBeLessThanOrEqual(2024);
      }
    });
  });
});

// Performance test
describeWithAPI('TMDB API Performance', () => {
  it('should respond within 3 seconds for search queries', async () => {
    const start = Date.now();
    await searchMulti('Avengers');
    const duration = Date.now() - start;

    expect(duration).toBeLessThan(3000);
    console.log(`Search latency: ${duration}ms`);
  });

  it('should handle concurrent requests', async () => {
    const start = Date.now();

    const results = await Promise.all([
      getTrending('movie', 'week'),
      getPopularMovies(),
      getGenres(),
    ]);

    const duration = Date.now() - start;

    expect(results[0].length).toBeGreaterThan(0);
    expect(results[1].results.length).toBeGreaterThan(0);
    expect(results[2].movies.length).toBeGreaterThan(0);

    console.log(`Concurrent requests latency: ${duration}ms`);
    expect(duration).toBeLessThan(5000);
  });
});
