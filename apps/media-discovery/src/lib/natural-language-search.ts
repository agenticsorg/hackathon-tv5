/**
 * Natural Language Search Service
 * Converts user prompts into semantic search queries using AI
 */

import { generateObject, generateText } from 'ai';
import { openai } from '@ai-sdk/openai';
import { google } from '@ai-sdk/google';
import { z } from 'zod';
import type { SemanticSearchQuery, SearchIntent, SearchFilters, MediaContent, SearchResult } from '@/types/media';
import { searchMulti, getSimilarMovies, getSimilarTVShows, discoverMovies, discoverTVShows } from './tmdb';
import { searchByEmbedding, getContentEmbedding, calculateSimilarity } from './vector-search';

// Schema for parsed search intent
const SearchIntentSchema = z.object({
  mood: z.array(z.string()).optional().describe('Emotional tone or feeling (e.g., "exciting", "heartwarming", "dark")'),
  themes: z.array(z.string()).optional().describe('Story themes (e.g., "redemption", "coming-of-age", "survival")'),
  pacing: z.enum(['slow', 'medium', 'fast']).optional().describe('Preferred pacing of the content'),
  era: z.string().optional().describe('Time period setting (e.g., "1980s", "modern", "futuristic")'),
  setting: z.array(z.string()).optional().describe('Physical setting (e.g., "space", "urban", "underwater")'),
  similar_to: z.array(z.string()).optional().describe('Similar movies/shows mentioned by user'),
  avoid: z.array(z.string()).optional().describe('Elements to avoid (e.g., "gore", "jump scares")'),
  genres: z.array(z.string()).optional().describe('Inferred genres from the query'),
  keywords: z.array(z.string()).optional().describe('Key search terms extracted'),
  mediaType: z.enum(['movie', 'tv', 'all']).optional().describe('Preferred media type'),
});

const SearchFiltersSchema = z.object({
  mediaType: z.enum(['movie', 'tv', 'all']).optional(),
  genres: z.array(z.number()).optional(),
  yearRange: z.object({
    min: z.number().optional(),
    max: z.number().optional(),
  }).optional(),
  ratingMin: z.number().optional(),
});

// Server-side intent cache (survives across requests in same process)
const intentCache = new Map<string, { result: SemanticSearchQuery; timestamp: number }>();
const INTENT_CACHE_TTL_MS = 10 * 60 * 1000; // 10 minute TTL

// Genre mapping for common genre names to TMDB IDs
const GENRE_MAP: Record<string, { movie: number; tv: number }> = {
  action: { movie: 28, tv: 10759 },
  adventure: { movie: 12, tv: 10759 },
  animation: { movie: 16, tv: 16 },
  comedy: { movie: 35, tv: 35 },
  crime: { movie: 80, tv: 80 },
  documentary: { movie: 99, tv: 99 },
  drama: { movie: 18, tv: 18 },
  family: { movie: 10751, tv: 10751 },
  fantasy: { movie: 14, tv: 10765 },
  horror: { movie: 27, tv: 9648 },
  mystery: { movie: 9648, tv: 9648 },
  romance: { movie: 10749, tv: 10749 },
  'sci-fi': { movie: 878, tv: 10765 },
  'science fiction': { movie: 878, tv: 10765 },
  thriller: { movie: 53, tv: 53 },
  war: { movie: 10752, tv: 10768 },
  western: { movie: 37, tv: 37 },
};

/**
 * Parse natural language query into structured search intent
 */
export async function parseSearchQuery(query: string): Promise<SemanticSearchQuery> {
  // Normalize for cache key
  const cacheKey = query.toLowerCase().trim();

  // Check cache first
  const cached = intentCache.get(cacheKey);
  if (cached && Date.now() - cached.timestamp < INTENT_CACHE_TTL_MS) {
    console.log(`📦 Intent cache hit for: "${query.slice(0, 30)}..."`);
    return cached.result;
  }

  try {
    console.log(`🧠 AI parsing intent for: "${query.slice(0, 30)}..."`);
    // Use AI to extract intent from natural language
    const { object: intent } = await generateObject({
      model: openai('gpt-4o-mini'),
      schema: SearchIntentSchema,
      prompt: `Analyze this movie/TV show search query and extract the user's intent:

Query: "${query}"

Extract:
- Mood: What emotional experience are they seeking?
- Themes: What story themes are they interested in?
- Pacing: Do they want something slow-burn, fast-paced, or in between?
- Era: What time period setting do they prefer?
- Setting: Where should the story take place?
- Similar to: Are they referencing specific movies/shows?
- Avoid: What elements should be avoided?
- Genres: What genres fit this description?
- Keywords: What are the key search terms?
- Media type: Do they specifically want movies or TV shows?

Be specific and extract as much relevant information as possible.`,
    });

    // Convert genre names to IDs
    const genreIds = intent.genres?.flatMap(genre => {
      const normalized = genre.toLowerCase();
      const mapping = GENRE_MAP[normalized];
      if (mapping) {
        return [mapping.movie, mapping.tv].filter((v, i, a) => a.indexOf(v) === i);
      }
      return [];
    }) || [];

    const result: SemanticSearchQuery = {
      query,
      intent: intent as SearchIntent,
      filters: {
        mediaType: intent.mediaType || 'all',
        genres: genreIds.length > 0 ? genreIds : undefined,
      },
    };

    // Cache the result
    intentCache.set(cacheKey, { result, timestamp: Date.now() });

    return result;
  } catch (error) {
    console.error('Failed to parse search query with AI:', error);
    // Fallback to basic query
    return { query };
  }
}

/**
 * Perform semantic search combining TMDB and vector search
 */
export async function semanticSearch(
  query: string,
  userPreferences?: number[]
): Promise<SearchResult[]> {
  // Parse the natural language query
  const semanticQuery = await parseSearchQuery(query);

  // Parallel search strategies
  const [tmdbResults, vectorResults] = await Promise.all([
    performTMDBSearch(semanticQuery),
    performVectorSearch(semanticQuery),
  ]);

  // Merge and rank results
  const mergedResults = mergeAndRankResults(tmdbResults, vectorResults, semanticQuery, userPreferences);

  return mergedResults;
}

/**
 * Perform TMDB-based search with intent understanding
 */
async function performTMDBSearch(query: SemanticSearchQuery): Promise<SearchResult[]> {
  const results: SearchResult[] = [];

  // Text search - prioritize direct title matches
  if (query.query) {
    const { results: textResults } = await searchMulti(query.query, query.filters);

    // Check if the query looks like a specific title (contains proper nouns or is in similar_to)
    const queryLower = query.query.toLowerCase();
    const isLikelyTitleSearch = query.intent?.similar_to?.some(
      ref => ref.toLowerCase() === queryLower || queryLower.includes(ref.toLowerCase())
    );

    results.push(...textResults.map((content, index) => {
      // Give highest score to exact/close title matches
      const titleLower = content.title.toLowerCase();
      const isExactMatch = titleLower === queryLower;
      const isCloseMatch = titleLower.includes(queryLower) || queryLower.includes(titleLower);

      let score = 0.8;
      if (isExactMatch) {
        score = 1.0; // Perfect match
      } else if (isCloseMatch) {
        score = 0.95; // Close match
      } else if (index < 3) {
        score = 0.85; // Top TMDB results
      }

      return {
        content,
        relevanceScore: score,
        matchReasons: isExactMatch || isCloseMatch ? ['Title match'] : ['Text match'],
      };
    }));
  }

  // Similar content search if references found (but DON'T overshadow direct matches)
  if (query.intent?.similar_to?.length) {
    // Search for the referenced content first
    for (const ref of query.intent.similar_to.slice(0, 3)) {
      const { results: refResults } = await searchMulti(ref);
      if (refResults.length > 0) {
        const firstMatch = refResults[0];

        // Add the referenced content itself with high score
        const alreadyExists = results.some(r =>
          r.content.id === firstMatch.id && r.content.mediaType === firstMatch.mediaType
        );
        if (!alreadyExists) {
          results.push({
            content: firstMatch,
            relevanceScore: 0.98, // Very high, but below exact title match
            matchReasons: ['Referenced title'],
          });
        }

        // Then get similar content
        const similar = firstMatch.mediaType === 'movie'
          ? await getSimilarMovies(firstMatch.id)
          : await getSimilarTVShows(firstMatch.id);

        results.push(...similar.map(content => ({
          content,
          relevanceScore: 0.75, // Lower than direct matches
          matchReasons: [`Similar to "${ref}"`],
        })));
      }
    }
  }

  // Discovery-based search with filters
  if (query.filters?.genres?.length) {
    const movieGenres = query.filters.genres.filter(id => id < 10000);
    const tvGenres = query.filters.genres.filter(id => id >= 10000 || GENRE_MAP.action?.tv === id);

    if (query.filters.mediaType !== 'tv' && movieGenres.length > 0) {
      const { results: discoveredMovies } = await discoverMovies({
        genres: movieGenres,
        ratingMin: query.filters.ratingMin,
      });
      results.push(...discoveredMovies.map(content => ({
        content,
        relevanceScore: 0.7,
        matchReasons: ['Genre match'],
      })));
    }

    if (query.filters.mediaType !== 'movie' && tvGenres.length > 0) {
      const { results: discoveredShows } = await discoverTVShows({
        genres: tvGenres,
        ratingMin: query.filters.ratingMin,
      });
      results.push(...discoveredShows.map(content => ({
        content,
        relevanceScore: 0.7,
        matchReasons: ['Genre match'],
      })));
    }
  }

  return results;
}

/**
 * Perform vector similarity search
 */
async function performVectorSearch(query: SemanticSearchQuery): Promise<SearchResult[]> {
  try {
    // Get embedding for the query
    const queryEmbedding = await getContentEmbedding(query.query);
    if (!queryEmbedding) return [];

    // Search vector database
    const vectorResults = await searchByEmbedding(queryEmbedding, 20);

    return vectorResults.map(result => ({
      content: result.content,
      relevanceScore: result.score,
      matchReasons: ['Semantic similarity'],
      similarityScore: result.score,
    }));
  } catch (error) {
    console.error('Vector search failed:', error);
    return [];
  }
}

/**
 * Merge and rank results from multiple sources
 */
function mergeAndRankResults(
  tmdbResults: SearchResult[],
  vectorResults: SearchResult[],
  query: SemanticSearchQuery,
  userPreferences?: number[]
): SearchResult[] {
  // Combine results, deduplicating by ID
  const resultMap = new Map<string, SearchResult>();

  // Process TMDB results
  for (const result of tmdbResults) {
    const key = `${result.content.mediaType}-${result.content.id}`;
    const existing = resultMap.get(key);
    if (existing) {
      existing.relevanceScore = Math.max(existing.relevanceScore, result.relevanceScore);
      existing.matchReasons.push(...result.matchReasons);
    } else {
      resultMap.set(key, { ...result });
    }
  }

  // Process vector results with higher weight
  for (const result of vectorResults) {
    const key = `${result.content.mediaType}-${result.content.id}`;
    const existing = resultMap.get(key);
    if (existing) {
      // Boost score significantly for vector matches
      existing.relevanceScore = Math.min(1, existing.relevanceScore + (result.similarityScore || 0) * 0.5);
      existing.matchReasons.push(...result.matchReasons);
      existing.similarityScore = result.similarityScore;
    } else {
      resultMap.set(key, { ...result, relevanceScore: (result.similarityScore || 0.5) * 0.8 });
    }
  }

  // Convert to array and apply additional scoring
  let results = Array.from(resultMap.values());

  // Apply intent-based scoring
  if (query.intent) {
    results = results.map(result => {
      let boost = 0;

      // Boost for matching themes (map themes to potential genre matches)
      if (query.intent?.themes?.length) {
        const themeGenres = query.intent.themes
          .map((t: string) => GENRE_MAP[t.toLowerCase()])
          .filter(Boolean);
        if (themeGenres.some((g: { movie?: number; tv?: number }) =>
          g?.movie && result.content.genreIds.includes(g.movie)
        )) {
          boost += 0.1;
          result.matchReasons.push('Theme match');
        }
      }

      // Popularity boost for highly rated content
      if (result.content.voteAverage >= 7.5 && result.content.voteCount > 1000) {
        boost += 0.05;
      }

      return {
        ...result,
        relevanceScore: Math.min(1, result.relevanceScore + boost),
      };
    });
  }

  // Apply user preference boost
  if (userPreferences?.length) {
    results = results.map(result => {
      const genreMatch = result.content.genreIds.some(id => userPreferences.includes(id));
      if (genreMatch) {
        return {
          ...result,
          relevanceScore: Math.min(1, result.relevanceScore + 0.1),
          matchReasons: [...result.matchReasons, 'Matches your preferences'],
        };
      }
      return result;
    });
  }

  // Sort by relevance and deduplicate match reasons
  return results
    .map(r => ({
      ...r,
      matchReasons: [...new Set(r.matchReasons)],
    }))
    .sort((a, b) => b.relevanceScore - a.relevanceScore)
    .slice(0, 50);
}

/**
 * Generate a natural language explanation for why content was recommended
 */
export async function explainRecommendation(
  content: MediaContent,
  userQuery: string,
  matchReasons: string[]
): Promise<string> {
  try {
    const { text } = await generateText({
      model: openai('gpt-4o-mini'),
      prompt: `Generate a brief, engaging explanation for why "${content.title}" was recommended to a user who searched for: "${userQuery}"

Match reasons: ${matchReasons.join(', ')}
Content overview: ${content.overview}
Rating: ${content.voteAverage}/10

Write a 1-2 sentence explanation that's conversational and highlights why this is a good match for their search. Don't mention the rating unless it's relevant.`,
    });

    return text;
  } catch (error) {
    // Fallback to simple explanation
    return matchReasons[0] || 'Based on your search';
  }
}
