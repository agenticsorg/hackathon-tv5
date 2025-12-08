import { describe, it, expect, beforeEach, vi } from 'vitest';
import { TMDbClient } from '../src/content/tmdb-client.js';
import { DISCOVERY_TOOLS } from '../src/content/discovery-tools.js';

// Mock fetch for TMDb API tests
const mockFetch = vi.fn();
global.fetch = mockFetch;

describe('TMDb Client', () => {
  let client: TMDbClient;

  beforeEach(() => {
    client = new TMDbClient({ apiKey: 'test-api-key' });
    mockFetch.mockReset();
  });

  describe('configuration', () => {
    it('should create client with API key', () => {
      expect(client).toBeInstanceOf(TMDbClient);
    });

    it('should use default language and region', () => {
      const client2 = new TMDbClient({ apiKey: 'key', language: 'es-ES', region: 'ES' });
      expect(client2).toBeInstanceOf(TMDbClient);
    });
  });

  describe('searchMovies', () => {
    it('should search movies with query', async () => {
      const mockResponse = {
        page: 1,
        results: [
          {
            id: 12345,
            title: 'Test Movie',
            original_title: 'Test Movie',
            overview: 'A test movie description',
            poster_path: '/test.jpg',
            backdrop_path: '/backdrop.jpg',
            release_date: '2024-01-15',
            genre_ids: [28, 12], // action, adventure
            vote_average: 8.5,
            vote_count: 1000,
            popularity: 75.5,
            adult: false,
          },
        ],
        total_pages: 1,
        total_results: 1,
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockResponse),
      });

      const result = await client.searchMovies('Test');

      expect(result.results).toHaveLength(1);
      expect(result.results[0].title).toBe('Test Movie');
      expect(result.results[0].genre_ids).toContain(28);
    });

    it('should handle API errors', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 401,
        statusText: 'Unauthorized',
      });

      await expect(client.searchMovies('Test')).rejects.toThrow('TMDb API error');
    });
  });

  describe('movieToContentMetadata', () => {
    it('should convert TMDb movie to ContentMetadata', async () => {
      const movie = {
        id: 12345,
        title: 'Action Movie',
        original_title: 'Action Movie',
        overview: 'An exciting action movie',
        poster_path: '/poster.jpg',
        backdrop_path: '/backdrop.jpg',
        release_date: '2024-06-15',
        genre_ids: [28, 12], // action, adventure
        vote_average: 8.0,
        vote_count: 500,
        popularity: 65,
        adult: false,
      };

      const content = await client.movieToContentMetadata(movie);

      expect(content.id).toBe('tmdb-movie-12345');
      expect(content.title).toBe('Action Movie');
      expect(content.type).toBe('movie');
      expect(content.genres).toContain('action');
      expect(content.genres).toContain('adventure');
      expect(content.rating).toBe(8.0);
      expect(content.releaseYear).toBe(2024);
      expect(content.description).toBe('An exciting action movie');
      expect(content.posterUrl).toContain('/poster.jpg');
    });

    it('should handle missing optional fields', async () => {
      const movie = {
        id: 99999,
        title: 'Minimal Movie',
        original_title: 'Minimal Movie',
        overview: '',
        poster_path: null,
        backdrop_path: null,
        release_date: '',
        genre_ids: [],
        vote_average: 0,
        vote_count: 0,
        popularity: 0,
        adult: false,
      };

      const content = await client.movieToContentMetadata(movie);

      expect(content.id).toBe('tmdb-movie-99999');
      expect(content.title).toBe('Minimal Movie');
      expect(content.genres).toHaveLength(0);
      expect(content.posterUrl).toBeUndefined();
    });
  });

  describe('tvShowToContentMetadata', () => {
    it('should convert TMDb TV show to ContentMetadata', async () => {
      const show = {
        id: 54321,
        name: 'Test Show',
        original_name: 'Test Show',
        overview: 'A great TV series',
        poster_path: '/show-poster.jpg',
        backdrop_path: '/show-backdrop.jpg',
        first_air_date: '2023-03-10',
        genre_ids: [18, 9648], // drama, mystery
        vote_average: 9.0,
        vote_count: 2000,
        popularity: 85,
        episode_run_time: [45, 50],
      };

      const content = await client.tvShowToContentMetadata(show);

      expect(content.id).toBe('tmdb-tv-54321');
      expect(content.title).toBe('Test Show');
      expect(content.type).toBe('tv_show');
      expect(content.genres).toContain('drama');
      expect(content.genres).toContain('mystery');
      expect(content.rating).toBe(9.0);
      expect(content.releaseYear).toBe(2023);
    });
  });

  describe('caching', () => {
    it('should cache API responses', async () => {
      const mockResponse = {
        page: 1,
        results: [{ id: 1, title: 'Cached Movie' }],
        total_pages: 1,
        total_results: 1,
      };

      mockFetch.mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockResponse),
      });

      // First call
      await client.searchMovies('Cache Test');
      // Second call (should use cache)
      await client.searchMovies('Cache Test');

      // Should only call fetch once due to caching
      expect(mockFetch).toHaveBeenCalledTimes(1);
    });

    it('should clear cache', async () => {
      const mockResponse = {
        page: 1,
        results: [],
        total_pages: 0,
        total_results: 0,
      };

      mockFetch.mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockResponse),
      });

      await client.searchMovies('Clear Test');
      client.clearCache();
      await client.searchMovies('Clear Test');

      // Should call twice after cache clear
      expect(mockFetch).toHaveBeenCalledTimes(2);
    });
  });

  describe('getImageUrl', () => {
    it('should generate image URL with default size', () => {
      const url = TMDbClient.getImageUrl('/test.jpg');
      expect(url).toBe('https://image.tmdb.org/t/p/w500/test.jpg');
    });

    it('should generate image URL with custom size', () => {
      const url = TMDbClient.getImageUrl('/test.jpg', 'original');
      expect(url).toBe('https://image.tmdb.org/t/p/original/test.jpg');
    });

    it('should return null for null path', () => {
      const url = TMDbClient.getImageUrl(null);
      expect(url).toBeNull();
    });
  });
});

describe('Discovery Tools', () => {
  describe('tool definitions', () => {
    it('should have all required tools defined', () => {
      const toolNames = DISCOVERY_TOOLS.map(t => t.name);

      expect(toolNames).toContain('content_search');
      expect(toolNames).toContain('content_trending');
      expect(toolNames).toContain('content_popular');
      expect(toolNames).toContain('content_top_rated');
      expect(toolNames).toContain('content_discover');
      expect(toolNames).toContain('content_details');
      expect(toolNames).toContain('content_similar');
      expect(toolNames).toContain('content_recommendations');
      expect(toolNames).toContain('content_now_playing');
      expect(toolNames).toContain('content_upcoming');
      expect(toolNames).toContain('content_personalized');
      expect(toolNames).toContain('content_for_mood');
    });

    it('should have 12 discovery tools', () => {
      expect(DISCOVERY_TOOLS).toHaveLength(12);
    });

    it('should have valid input schemas', () => {
      for (const tool of DISCOVERY_TOOLS) {
        expect(tool.inputSchema).toBeDefined();
        expect(tool.inputSchema.type).toBe('object');
        expect(tool.inputSchema.properties).toBeDefined();
      }
    });

    it('content_search should require query parameter', () => {
      const searchTool = DISCOVERY_TOOLS.find(t => t.name === 'content_search');
      expect(searchTool?.inputSchema.required).toContain('query');
    });

    it('content_for_mood should require mood parameter', () => {
      const moodTool = DISCOVERY_TOOLS.find(t => t.name === 'content_for_mood');
      expect(moodTool?.inputSchema.required).toContain('mood');
    });

    it('content_details should require contentId parameter', () => {
      const detailsTool = DISCOVERY_TOOLS.find(t => t.name === 'content_details');
      expect(detailsTool?.inputSchema.required).toContain('contentId');
    });
  });

  describe('tool descriptions', () => {
    it('all tools should have descriptions', () => {
      for (const tool of DISCOVERY_TOOLS) {
        expect(tool.description).toBeDefined();
        expect(tool.description.length).toBeGreaterThan(10);
      }
    });
  });
});

describe('Genre Mapping', () => {
  it('should map TMDb action genre', async () => {
    const client = new TMDbClient({ apiKey: 'test' });

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve({
        page: 1,
        results: [{
          id: 1,
          title: 'Action',
          original_title: 'Action',
          overview: '',
          poster_path: null,
          backdrop_path: null,
          release_date: '2024-01-01',
          genre_ids: [28], // TMDb action genre ID
          vote_average: 7,
          vote_count: 100,
          popularity: 50,
          adult: false,
        }],
        total_pages: 1,
        total_results: 1,
      }),
    });

    const result = await client.searchMovies('action');
    const content = await client.movieToContentMetadata(result.results[0]);

    expect(content.genres).toContain('action');
  });

  it('should handle unknown genre IDs gracefully', async () => {
    const client = new TMDbClient({ apiKey: 'test' });

    const movie = {
      id: 1,
      title: 'Unknown Genre',
      original_title: 'Unknown Genre',
      overview: '',
      poster_path: null,
      backdrop_path: null,
      release_date: '',
      genre_ids: [99999], // Unknown genre ID
      vote_average: 0,
      vote_count: 0,
      popularity: 0,
      adult: false,
    };

    const content = await client.movieToContentMetadata(movie);

    // Unknown genres should be filtered out
    expect(content.genres).not.toContain(99999);
  });
});
