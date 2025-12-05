/**
 * Content Discovery MCP Tools
 *
 * Provides AI agents with tools to discover, search, and explore content
 * from TMDb for Samsung TV recommendations
 */

import { TMDbClient, createTMDbClient } from './tmdb-client.js';
import { PreferenceLearningSystem } from '../learning/preference-learning.js';
import { MCPToolResult } from '../lib/types.js';
import { ContentMetadata, Genre } from '../learning/types.js';

// Shared instances
let tmdbClient: TMDbClient | null = null;
let learningSystem: PreferenceLearningSystem | null = null;

/**
 * Initialize the content discovery system
 */
export function initContentDiscovery(apiKey?: string, learner?: PreferenceLearningSystem): void {
  if (apiKey || process.env.TMDB_API_KEY) {
    tmdbClient = createTMDbClient(apiKey);
  }
  if (learner) {
    learningSystem = learner;
  }
}

/**
 * Get or create TMDb client
 */
function getClient(): TMDbClient {
  if (!tmdbClient) {
    tmdbClient = createTMDbClient();
  }
  return tmdbClient;
}

/**
 * Get or create learning system
 */
function getLearner(): PreferenceLearningSystem {
  if (!learningSystem) {
    learningSystem = new PreferenceLearningSystem();
  }
  return learningSystem;
}

// Genre name to TMDb ID mapping
const GENRE_TO_TMDB: Record<Genre, number> = {
  action: 28,
  adventure: 12,
  animation: 16,
  comedy: 35,
  crime: 80,
  documentary: 99,
  drama: 18,
  family: 10751,
  fantasy: 14,
  history: 36,
  horror: 27,
  music: 10402,
  mystery: 9648,
  romance: 10749,
  science_fiction: 878,
  thriller: 53,
  war: 10752,
  western: 37,
  reality: 10764,
  sports: 28, // Map to action
  news: 10763,
};

// MCP Tool Definitions
export const DISCOVERY_TOOLS = [
  {
    name: 'content_search',
    description: 'Search for movies and TV shows by title or keywords. Returns content with metadata for recommendations.',
    inputSchema: {
      type: 'object',
      properties: {
        query: {
          type: 'string',
          description: 'Search query (title, keywords, etc.)',
        },
        type: {
          type: 'string',
          enum: ['movie', 'tv', 'all'],
          description: 'Content type to search (default: all)',
        },
        page: {
          type: 'number',
          description: 'Page number for pagination (default: 1)',
        },
        addToLibrary: {
          type: 'boolean',
          description: 'Add results to learning library (default: true)',
        },
      },
      required: ['query'],
    },
  },
  {
    name: 'content_trending',
    description: 'Get trending movies and TV shows. Great for discovering popular content.',
    inputSchema: {
      type: 'object',
      properties: {
        type: {
          type: 'string',
          enum: ['movie', 'tv', 'all'],
          description: 'Content type (default: all)',
        },
        timeWindow: {
          type: 'string',
          enum: ['day', 'week'],
          description: 'Time window for trending (default: week)',
        },
        addToLibrary: {
          type: 'boolean',
          description: 'Add results to learning library (default: true)',
        },
      },
    },
  },
  {
    name: 'content_popular',
    description: 'Get popular movies or TV shows based on overall popularity.',
    inputSchema: {
      type: 'object',
      properties: {
        type: {
          type: 'string',
          enum: ['movie', 'tv'],
          description: 'Content type (default: movie)',
        },
        page: {
          type: 'number',
          description: 'Page number (default: 1)',
        },
        addToLibrary: {
          type: 'boolean',
          description: 'Add results to learning library (default: true)',
        },
      },
    },
  },
  {
    name: 'content_top_rated',
    description: 'Get top rated movies or TV shows by user ratings.',
    inputSchema: {
      type: 'object',
      properties: {
        type: {
          type: 'string',
          enum: ['movie', 'tv'],
          description: 'Content type (default: movie)',
        },
        page: {
          type: 'number',
          description: 'Page number (default: 1)',
        },
        addToLibrary: {
          type: 'boolean',
          description: 'Add results to learning library (default: true)',
        },
      },
    },
  },
  {
    name: 'content_discover',
    description: 'Discover content with filters like genre, rating, and year. Powerful for targeted recommendations.',
    inputSchema: {
      type: 'object',
      properties: {
        type: {
          type: 'string',
          enum: ['movie', 'tv'],
          description: 'Content type (default: movie)',
        },
        genres: {
          type: 'array',
          items: { type: 'string' },
          description: 'Filter by genres (e.g., ["action", "comedy"])',
        },
        minRating: {
          type: 'number',
          description: 'Minimum rating (0-10)',
        },
        maxRating: {
          type: 'number',
          description: 'Maximum rating (0-10)',
        },
        minYear: {
          type: 'number',
          description: 'Minimum release year',
        },
        maxYear: {
          type: 'number',
          description: 'Maximum release year',
        },
        sortBy: {
          type: 'string',
          enum: ['popularity.desc', 'popularity.asc', 'vote_average.desc', 'vote_average.asc', 'release_date.desc', 'release_date.asc'],
          description: 'Sort order (default: popularity.desc)',
        },
        page: {
          type: 'number',
          description: 'Page number (default: 1)',
        },
        addToLibrary: {
          type: 'boolean',
          description: 'Add results to learning library (default: true)',
        },
      },
    },
  },
  {
    name: 'content_details',
    description: 'Get detailed information about a specific movie or TV show including cast, streaming availability.',
    inputSchema: {
      type: 'object',
      properties: {
        contentId: {
          type: 'string',
          description: 'Content ID (e.g., "tmdb-movie-12345" or just the TMDb ID)',
        },
        type: {
          type: 'string',
          enum: ['movie', 'tv'],
          description: 'Content type (required if using raw TMDb ID)',
        },
        addToLibrary: {
          type: 'boolean',
          description: 'Add to learning library (default: true)',
        },
      },
      required: ['contentId'],
    },
  },
  {
    name: 'content_similar',
    description: 'Get content similar to a specific movie or TV show.',
    inputSchema: {
      type: 'object',
      properties: {
        contentId: {
          type: 'string',
          description: 'Content ID to find similar content for',
        },
        type: {
          type: 'string',
          enum: ['movie', 'tv'],
          description: 'Content type',
        },
        page: {
          type: 'number',
          description: 'Page number (default: 1)',
        },
        addToLibrary: {
          type: 'boolean',
          description: 'Add results to learning library (default: true)',
        },
      },
      required: ['contentId'],
    },
  },
  {
    name: 'content_recommendations',
    description: 'Get TMDb recommendations based on a specific movie or TV show.',
    inputSchema: {
      type: 'object',
      properties: {
        contentId: {
          type: 'string',
          description: 'Content ID to get recommendations for',
        },
        type: {
          type: 'string',
          enum: ['movie', 'tv'],
          description: 'Content type',
        },
        page: {
          type: 'number',
          description: 'Page number (default: 1)',
        },
        addToLibrary: {
          type: 'boolean',
          description: 'Add results to learning library (default: true)',
        },
      },
      required: ['contentId'],
    },
  },
  {
    name: 'content_now_playing',
    description: 'Get movies currently in theaters.',
    inputSchema: {
      type: 'object',
      properties: {
        page: {
          type: 'number',
          description: 'Page number (default: 1)',
        },
        addToLibrary: {
          type: 'boolean',
          description: 'Add results to learning library (default: true)',
        },
      },
    },
  },
  {
    name: 'content_upcoming',
    description: 'Get upcoming movie releases.',
    inputSchema: {
      type: 'object',
      properties: {
        page: {
          type: 'number',
          description: 'Page number (default: 1)',
        },
        addToLibrary: {
          type: 'boolean',
          description: 'Add results to learning library (default: true)',
        },
      },
    },
  },
  {
    name: 'content_personalized',
    description: 'Get personalized content recommendations based on learned user preferences.',
    inputSchema: {
      type: 'object',
      properties: {
        count: {
          type: 'number',
          description: 'Number of recommendations (default: 10)',
        },
        includeDetails: {
          type: 'boolean',
          description: 'Include full details with cast and streaming (default: false)',
        },
      },
    },
  },
  {
    name: 'content_for_mood',
    description: 'Get content recommendations based on mood or viewing context.',
    inputSchema: {
      type: 'object',
      properties: {
        mood: {
          type: 'string',
          enum: ['relaxing', 'exciting', 'romantic', 'scary', 'funny', 'thoughtful', 'family', 'nostalgic'],
          description: 'Desired viewing mood',
        },
        duration: {
          type: 'string',
          enum: ['short', 'medium', 'long'],
          description: 'Preferred duration (short: <90min, medium: 90-150min, long: >150min)',
        },
        count: {
          type: 'number',
          description: 'Number of recommendations (default: 5)',
        },
      },
      required: ['mood'],
    },
  },
];

/**
 * Handle content discovery tool calls
 */
export async function handleDiscoveryToolCall(
  toolName: string,
  args: Record<string, unknown>
): Promise<MCPToolResult> {
  try {
    const client = getClient();
    const learner = getLearner();
    const addToLibrary = args.addToLibrary !== false;

    switch (toolName) {
      case 'content_search': {
        const query = args.query as string;
        const type = (args.type as string) || 'all';
        const page = (args.page as number) || 1;

        let results: ContentMetadata[] = [];

        if (type === 'movie' || type === 'all') {
          const movies = await client.searchMovies(query, page);
          const movieContent = await Promise.all(
            movies.results.slice(0, 10).map(m => client.movieToContentMetadata(m))
          );
          results.push(...movieContent);
        }

        if (type === 'tv' || type === 'all') {
          const shows = await client.searchTVShows(query, page);
          const showContent = await Promise.all(
            shows.results.slice(0, 10).map(s => client.tvShowToContentMetadata(s))
          );
          results.push(...showContent);
        }

        if (addToLibrary) {
          learner.addContents(results);
        }

        return {
          success: true,
          data: {
            query,
            count: results.length,
            results: results.map(r => ({
              id: r.id,
              title: r.title,
              type: r.type,
              genres: r.genres,
              rating: r.rating,
              year: r.releaseYear,
              description: r.description?.slice(0, 200),
              posterUrl: r.posterUrl,
              appName: r.appName,
            })),
          },
        };
      }

      case 'content_trending': {
        const type = (args.type as 'movie' | 'tv' | 'all') || 'all';
        const timeWindow = (args.timeWindow as 'day' | 'week') || 'week';

        const trending = await client.getTrending(type, timeWindow);
        const results: ContentMetadata[] = await Promise.all(
          trending.results.slice(0, 20).map(async item => {
            if ('title' in item) {
              return client.movieToContentMetadata(item);
            } else {
              return client.tvShowToContentMetadata(item);
            }
          })
        );

        if (addToLibrary) {
          learner.addContents(results);
        }

        return {
          success: true,
          data: {
            timeWindow,
            count: results.length,
            results: results.map(r => ({
              id: r.id,
              title: r.title,
              type: r.type,
              genres: r.genres,
              rating: r.rating,
              popularity: r.popularity,
              appName: r.appName,
            })),
          },
        };
      }

      case 'content_popular': {
        const type = (args.type as 'movie' | 'tv') || 'movie';
        const page = (args.page as number) || 1;

        let results: ContentMetadata[];
        if (type === 'movie') {
          const popular = await client.getPopularMovies(page);
          results = await Promise.all(popular.results.map(m => client.movieToContentMetadata(m)));
        } else {
          const popular = await client.getPopularTVShows(page);
          results = await Promise.all(popular.results.map(s => client.tvShowToContentMetadata(s)));
        }

        if (addToLibrary) {
          learner.addContents(results);
        }

        return {
          success: true,
          data: {
            type,
            page,
            count: results.length,
            results: results.map(r => ({
              id: r.id,
              title: r.title,
              type: r.type,
              genres: r.genres,
              rating: r.rating,
              popularity: r.popularity,
            })),
          },
        };
      }

      case 'content_top_rated': {
        const type = (args.type as 'movie' | 'tv') || 'movie';
        const page = (args.page as number) || 1;

        let results: ContentMetadata[];
        if (type === 'movie') {
          const topRated = await client.getTopRatedMovies(page);
          results = await Promise.all(topRated.results.map(m => client.movieToContentMetadata(m)));
        } else {
          const topRated = await client.getTopRatedTVShows(page);
          results = await Promise.all(topRated.results.map(s => client.tvShowToContentMetadata(s)));
        }

        if (addToLibrary) {
          learner.addContents(results);
        }

        return {
          success: true,
          data: {
            type,
            page,
            count: results.length,
            results: results.map(r => ({
              id: r.id,
              title: r.title,
              type: r.type,
              genres: r.genres,
              rating: r.rating,
            })),
          },
        };
      }

      case 'content_discover': {
        const type = (args.type as 'movie' | 'tv') || 'movie';
        const genres = args.genres as string[] | undefined;
        const page = (args.page as number) || 1;

        const genreIds = genres?.map(g => GENRE_TO_TMDB[g as Genre]).filter(Boolean);

        const filters = {
          genres: genreIds,
          minRating: args.minRating as number | undefined,
          maxRating: args.maxRating as number | undefined,
          minYear: args.minYear as number | undefined,
          maxYear: args.maxYear as number | undefined,
          sortBy: args.sortBy as string | undefined,
          page,
        };

        let results: ContentMetadata[];
        if (type === 'movie') {
          const discovered = await client.discoverMovies(filters);
          results = await Promise.all(discovered.results.map(m => client.movieToContentMetadata(m)));
        } else {
          const discovered = await client.discoverTVShows(filters);
          results = await Promise.all(discovered.results.map(s => client.tvShowToContentMetadata(s)));
        }

        if (addToLibrary) {
          learner.addContents(results);
        }

        return {
          success: true,
          data: {
            type,
            filters: { requestedGenres: genres, ...filters },
            count: results.length,
            results: results.map(r => ({
              id: r.id,
              title: r.title,
              type: r.type,
              genres: r.genres,
              rating: r.rating,
              year: r.releaseYear,
            })),
          },
        };
      }

      case 'content_details': {
        const contentId = args.contentId as string;
        let type = args.type as 'movie' | 'tv' | undefined;
        let tmdbId: number;

        // Parse content ID
        if (contentId.startsWith('tmdb-movie-')) {
          tmdbId = parseInt(contentId.replace('tmdb-movie-', ''));
          type = 'movie';
        } else if (contentId.startsWith('tmdb-tv-')) {
          tmdbId = parseInt(contentId.replace('tmdb-tv-', ''));
          type = 'tv';
        } else {
          tmdbId = parseInt(contentId);
          if (!type) {
            return { success: false, error: 'Type required for raw TMDb ID' };
          }
        }

        let content: ContentMetadata;
        if (type === 'movie') {
          const details = await client.getMovieDetails(tmdbId);
          content = await client.movieToContentMetadata(details, true);
        } else {
          const details = await client.getTVShowDetails(tmdbId);
          content = await client.tvShowToContentMetadata(details, true);
        }

        if (addToLibrary) {
          learner.addContent(content);
        }

        return {
          success: true,
          data: content,
        };
      }

      case 'content_similar': {
        const contentId = args.contentId as string;
        let type = args.type as 'movie' | 'tv' | undefined;
        const page = (args.page as number) || 1;
        let tmdbId: number;

        if (contentId.startsWith('tmdb-movie-')) {
          tmdbId = parseInt(contentId.replace('tmdb-movie-', ''));
          type = 'movie';
        } else if (contentId.startsWith('tmdb-tv-')) {
          tmdbId = parseInt(contentId.replace('tmdb-tv-', ''));
          type = 'tv';
        } else {
          tmdbId = parseInt(contentId);
          if (!type) {
            return { success: false, error: 'Type required for raw TMDb ID' };
          }
        }

        let results: ContentMetadata[];
        if (type === 'movie') {
          const similar = await client.getSimilarMovies(tmdbId, page);
          results = await Promise.all(similar.results.map(m => client.movieToContentMetadata(m)));
        } else {
          const similar = await client.getSimilarTVShows(tmdbId, page);
          results = await Promise.all(similar.results.map(s => client.tvShowToContentMetadata(s)));
        }

        if (addToLibrary) {
          learner.addContents(results);
        }

        return {
          success: true,
          data: {
            basedOn: contentId,
            count: results.length,
            results: results.map(r => ({
              id: r.id,
              title: r.title,
              type: r.type,
              genres: r.genres,
              rating: r.rating,
            })),
          },
        };
      }

      case 'content_recommendations': {
        const contentId = args.contentId as string;
        let type = args.type as 'movie' | 'tv' | undefined;
        const page = (args.page as number) || 1;
        let tmdbId: number;

        if (contentId.startsWith('tmdb-movie-')) {
          tmdbId = parseInt(contentId.replace('tmdb-movie-', ''));
          type = 'movie';
        } else if (contentId.startsWith('tmdb-tv-')) {
          tmdbId = parseInt(contentId.replace('tmdb-tv-', ''));
          type = 'tv';
        } else {
          tmdbId = parseInt(contentId);
          if (!type) {
            return { success: false, error: 'Type required for raw TMDb ID' };
          }
        }

        let results: ContentMetadata[];
        if (type === 'movie') {
          const recs = await client.getMovieRecommendations(tmdbId, page);
          results = await Promise.all(recs.results.map(m => client.movieToContentMetadata(m)));
        } else {
          const recs = await client.getTVShowRecommendations(tmdbId, page);
          results = await Promise.all(recs.results.map(s => client.tvShowToContentMetadata(s)));
        }

        if (addToLibrary) {
          learner.addContents(results);
        }

        return {
          success: true,
          data: {
            basedOn: contentId,
            count: results.length,
            results: results.map(r => ({
              id: r.id,
              title: r.title,
              type: r.type,
              genres: r.genres,
              rating: r.rating,
            })),
          },
        };
      }

      case 'content_now_playing': {
        const page = (args.page as number) || 1;
        const nowPlaying = await client.getNowPlayingMovies(page);
        const results = await Promise.all(nowPlaying.results.map(m => client.movieToContentMetadata(m)));

        if (addToLibrary) {
          learner.addContents(results);
        }

        return {
          success: true,
          data: {
            page,
            count: results.length,
            results: results.map(r => ({
              id: r.id,
              title: r.title,
              genres: r.genres,
              rating: r.rating,
              year: r.releaseYear,
            })),
          },
        };
      }

      case 'content_upcoming': {
        const page = (args.page as number) || 1;
        const upcoming = await client.getUpcomingMovies(page);
        const results = await Promise.all(upcoming.results.map(m => client.movieToContentMetadata(m)));

        if (addToLibrary) {
          learner.addContents(results);
        }

        return {
          success: true,
          data: {
            page,
            count: results.length,
            results: results.map(r => ({
              id: r.id,
              title: r.title,
              genres: r.genres,
              rating: r.rating,
              year: r.releaseYear,
            })),
          },
        };
      }

      case 'content_personalized': {
        const count = (args.count as number) || 10;
        const includeDetails = args.includeDetails === true;

        // Get recommendations from learning system
        const recommendations = learner.getRecommendations(count);

        // Optionally enhance with full details
        let results = recommendations;
        if (includeDetails && recommendations.length > 0) {
          // Fetch additional details for top recommendations
          // This would need the content to have been fetched with details already
        }

        return {
          success: true,
          data: {
            count: results.length,
            learningStats: learner.getStats(),
            recommendations: results,
          },
        };
      }

      case 'content_for_mood': {
        const mood = args.mood as string;
        const duration = args.duration as string | undefined;
        const count = (args.count as number) || 5;

        // Map mood to genres and filters
        const moodGenres: Record<string, Genre[]> = {
          relaxing: ['comedy', 'family', 'romance', 'animation'],
          exciting: ['action', 'adventure', 'thriller', 'science_fiction'],
          romantic: ['romance', 'drama', 'comedy'],
          scary: ['horror', 'thriller', 'mystery'],
          funny: ['comedy', 'animation', 'family'],
          thoughtful: ['drama', 'documentary', 'history', 'mystery'],
          family: ['family', 'animation', 'comedy', 'adventure'],
          nostalgic: ['drama', 'romance', 'family'],
        };

        const genres = moodGenres[mood] || ['drama', 'comedy'];
        const genreIds = genres.map(g => GENRE_TO_TMDB[g]).filter(Boolean);

        // Duration filters
        let minRuntime: number | undefined;
        let maxRuntime: number | undefined;
        if (duration === 'short') {
          maxRuntime = 90;
        } else if (duration === 'medium') {
          minRuntime = 90;
          maxRuntime = 150;
        } else if (duration === 'long') {
          minRuntime = 150;
        }

        const discovered = await client.discoverMovies({
          genres: genreIds.slice(0, 2),
          minRating: 6,
          sortBy: 'vote_average.desc',
          page: 1,
        });

        let results = await Promise.all(
          discovered.results
            .filter(m => {
              if (!m.runtime) return true;
              if (minRuntime && m.runtime < minRuntime) return false;
              if (maxRuntime && m.runtime > maxRuntime) return false;
              return true;
            })
            .slice(0, count)
            .map(m => client.movieToContentMetadata(m))
        );

        if (addToLibrary) {
          learner.addContents(results);
        }

        return {
          success: true,
          data: {
            mood,
            duration,
            suggestedGenres: genres,
            count: results.length,
            results: results.map(r => ({
              id: r.id,
              title: r.title,
              type: r.type,
              genres: r.genres,
              rating: r.rating,
              duration: r.duration,
              description: r.description?.slice(0, 150),
            })),
          },
        };
      }

      default:
        return { success: false, error: `Unknown tool: ${toolName}` };
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error';
    return { success: false, error: message };
  }
}

/**
 * Get all content discovery tools
 */
export function getDiscoveryTools() {
  return DISCOVERY_TOOLS;
}
