/**
 * Natural Language Search Service
 * Converts user prompts into semantic search queries using AI
 */

import { generateObject, generateText } from 'ai';
import { openai } from '@ai-sdk/openai';
import { google } from '@ai-sdk/google';
import { z } from 'zod';
import type { SemanticSearchQuery, SearchIntent, SearchFilters, MediaContent, SearchResult, StreamingInfo } from '@/types/media';
import { searchMulti, getSimilarMovies, getSimilarTVShows, discoverMovies, discoverTVShows, searchPerson, getPersonCredits, getTrending, getRecentReleases } from './tmdb';
import { searchByEmbedding, getContentEmbedding, calculateSimilarity } from './vector-search';
import {
  getBatchStreamingAvailability,
  parseStreamingPreference,
  filterByStreamingService,
  formatStreamingProviders,
  getStreamingBadge,
  isStreamingNow,
  type StreamingAvailability,
} from './streaming-availability';
import {
  getIntentCache,
  getSearchResultCache,
  createSearchCacheKey,
} from './cache';
import {
  trackSearchInitiated,
  trackSearchCompleted,
} from './analytics';

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

// Multi-tier cache for intent parsing (L1: in-memory, L2: Redis if available)
const intentCacheInstance = getIntentCache();
const searchResultCacheInstance = getSearchResultCache();

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

// Mood-to-genre mapping for natural language understanding without AI
// Expanded with 50+ keywords to handle diverse voice queries
const MOOD_MAP: Record<string, { genres: string[]; themes?: string[]; pacing?: 'slow' | 'medium' | 'fast' }> = {
  // Excitement & Energy
  'exciting': { genres: ['action', 'thriller', 'adventure'], pacing: 'fast' },
  'thrilling': { genres: ['thriller', 'action', 'mystery'], pacing: 'fast' },
  'adrenaline': { genres: ['action', 'thriller'], pacing: 'fast' },
  'intense': { genres: ['thriller', 'drama', 'action'], pacing: 'fast' },
  'edge of my seat': { genres: ['thriller', 'horror', 'action'] },

  // Comfort & Warmth
  'cozy': { genres: ['comedy', 'romance', 'family'], pacing: 'slow', themes: ['heartwarming', 'comfort'] },
  'comforting': { genres: ['family', 'comedy', 'romance'], pacing: 'slow' },
  'feel good': { genres: ['comedy', 'family', 'romance'], themes: ['uplifting', 'heartwarming'] },
  'heartwarming': { genres: ['family', 'romance', 'drama'], themes: ['heartwarming'] },
  'uplifting': { genres: ['family', 'comedy', 'drama'], themes: ['uplifting', 'inspiring'] },

  // Darkness & Mystery
  'dark': { genres: ['horror', 'thriller', 'mystery'], themes: ['dark', 'psychological'] },
  'scary': { genres: ['horror', 'thriller'] },
  'creepy': { genres: ['horror', 'mystery', 'thriller'] },
  'suspenseful': { genres: ['thriller', 'mystery', 'crime'], pacing: 'medium' },
  'mysterious': { genres: ['mystery', 'thriller', 'sci-fi'] },
  'psychological': { genres: ['thriller', 'drama', 'mystery'], themes: ['psychological'] },

  // Romance & Relationships
  'romantic': { genres: ['romance', 'comedy', 'drama'] },
  'love': { genres: ['romance', 'drama'] },
  'relationship': { genres: ['romance', 'drama'], themes: ['relationships'] },

  // Humor
  'funny': { genres: ['comedy'] },
  'hilarious': { genres: ['comedy'] },
  'laugh': { genres: ['comedy'] },
  'lighthearted': { genres: ['comedy', 'family'], pacing: 'medium' },

  // Intellectual & Thought-provoking
  'thought provoking': { genres: ['drama', 'sci-fi', 'mystery'], pacing: 'slow' },
  'intelligent': { genres: ['drama', 'mystery', 'thriller'], pacing: 'slow' },
  'cerebral': { genres: ['drama', 'sci-fi', 'thriller'], pacing: 'slow' },
  'mind bending': { genres: ['sci-fi', 'thriller', 'mystery'], themes: ['complex'] },

  // Action & Adventure
  'action packed': { genres: ['action', 'adventure'], pacing: 'fast' },
  'explosive': { genres: ['action', 'sci-fi'], pacing: 'fast' },
  'adventurous': { genres: ['adventure', 'action', 'fantasy'], pacing: 'medium' },

  // Emotional
  'emotional': { genres: ['drama', 'romance'], themes: ['emotional'] },
  'sad': { genres: ['drama'], themes: ['emotional', 'tragic'] },
  'crying': { genres: ['drama', 'romance'], themes: ['emotional'] },
  'inspiring': { genres: ['drama', 'family'], themes: ['inspiring', 'uplifting'] },

  // Sci-Fi & Fantasy
  'futuristic': { genres: ['sci-fi'], themes: ['futuristic'] },
  'space': { genres: ['sci-fi', 'adventure'], themes: ['space'] },
  'magical': { genres: ['fantasy', 'family'], themes: ['magic'] },
  'epic': { genres: ['fantasy', 'adventure', 'action'], pacing: 'medium' },

  // NEW: Positive/Quality Descriptors (for "cool", "awesome", "interesting", etc.)
  'cool': { genres: ['action', 'sci-fi', 'thriller'], pacing: 'fast', themes: ['modern', 'stylish'] },
  'awesome': { genres: ['action', 'adventure', 'sci-fi'], pacing: 'fast', themes: ['spectacular'] },
  'amazing': { genres: ['adventure', 'fantasy', 'sci-fi'], pacing: 'medium', themes: ['spectacular'] },
  'fantastic': { genres: ['fantasy', 'adventure', 'sci-fi'], pacing: 'medium' },
  'great': { genres: ['action', 'comedy', 'drama'], pacing: 'medium' },
  'good': { genres: ['comedy', 'drama', 'family'], pacing: 'medium' },
  'excellent': { genres: ['drama', 'thriller', 'action'], pacing: 'medium' },
  'wonderful': { genres: ['family', 'romance', 'comedy'], themes: ['heartwarming'] },
  'brilliant': { genres: ['drama', 'mystery', 'thriller'], themes: ['intelligent'] },
  'incredible': { genres: ['action', 'adventure', 'sci-fi'], pacing: 'fast' },
  'outstanding': { genres: ['drama', 'thriller', 'action'], pacing: 'medium' },
  'superb': { genres: ['drama', 'mystery', 'thriller'], pacing: 'medium' },
  'terrific': { genres: ['comedy', 'action', 'adventure'], pacing: 'fast' },
  'marvelous': { genres: ['fantasy', 'adventure', 'family'], pacing: 'medium' },
  'spectacular': { genres: ['action', 'sci-fi', 'adventure'], pacing: 'fast', themes: ['spectacular'] },

  // NEW: Interest/Engagement
  'interesting': { genres: ['mystery', 'thriller', 'sci-fi'], themes: ['complex', 'intriguing'] },
  'intriguing': { genres: ['mystery', 'thriller', 'drama'], themes: ['complex'] },
  'compelling': { genres: ['drama', 'thriller', 'mystery'], themes: ['engaging'] },
  'engaging': { genres: ['drama', 'thriller', 'comedy'], pacing: 'medium' },
  'captivating': { genres: ['drama', 'romance', 'mystery'], pacing: 'medium' },
  'gripping': { genres: ['thriller', 'action', 'mystery'], pacing: 'fast' },
  'absorbing': { genres: ['drama', 'mystery', 'sci-fi'], pacing: 'slow' },
  'mesmerizing': { genres: ['drama', 'fantasy', 'sci-fi'], pacing: 'slow' },
  'fascinating': { genres: ['documentary', 'sci-fi', 'mystery'], themes: ['educational'] },

  // NEW: Energy Levels
  'energetic': { genres: ['action', 'comedy', 'adventure'], pacing: 'fast' },
  'wild': { genres: ['action', 'thriller', 'comedy'], pacing: 'fast' },
  'crazy': { genres: ['comedy', 'action', 'thriller'], pacing: 'fast' },
  'insane': { genres: ['action', 'thriller', 'horror'], pacing: 'fast' },
  'calm': { genres: ['drama', 'romance', 'family'], pacing: 'slow' },
  'peaceful': { genres: ['drama', 'family', 'documentary'], pacing: 'slow' },
  'relaxing': { genres: ['family', 'comedy', 'romance'], pacing: 'slow' },
  'chill': { genres: ['comedy', 'romance', 'family'], pacing: 'slow' },

  // NEW: Generic Entertainment Requests
  'entertaining': { genres: ['comedy', 'action', 'adventure'], pacing: 'medium' },
  'fun': { genres: ['comedy', 'adventure', 'family'], pacing: 'medium' },
  'enjoyable': { genres: ['comedy', 'family', 'romance'], pacing: 'medium' },
  'pleasurable': { genres: ['romance', 'comedy', 'family'], pacing: 'medium' },
  'delightful': { genres: ['comedy', 'family', 'romance'], themes: ['heartwarming'] },

  // NEW: Mood States
  'bored': { genres: ['action', 'comedy', 'thriller'], pacing: 'fast', themes: ['exciting'] },
  'tired': { genres: ['comedy', 'family', 'romance'], pacing: 'slow' },
  'stressed': { genres: ['comedy', 'family', 'romance'], pacing: 'slow', themes: ['relaxing'] },
  'happy': { genres: ['comedy', 'romance', 'family'], themes: ['uplifting'] },
  'curious': { genres: ['mystery', 'sci-fi', 'documentary'], themes: ['intriguing'] },

  // NEW: Novelty/Discovery
  'fresh': { genres: ['comedy', 'drama', 'sci-fi'], pacing: 'medium', themes: ['modern'] },
  'new': { genres: ['action', 'comedy', 'sci-fi'], pacing: 'medium', themes: ['modern'] },
  'different': { genres: ['sci-fi', 'mystery', 'thriller'], themes: ['unique'] },
  'unique': { genres: ['sci-fi', 'fantasy', 'mystery'], themes: ['unique'] },
  'original': { genres: ['sci-fi', 'drama', 'mystery'], themes: ['unique'] },
  'innovative': { genres: ['sci-fi', 'thriller', 'mystery'], themes: ['unique'] },
  'creative': { genres: ['fantasy', 'sci-fi', 'comedy'], themes: ['unique'] },
};

/**
 * Parse natural language query into structured search intent
 * Now with robust fallback mood detection that doesn't require API keys
 */
export async function parseSearchQuery(query: string): Promise<SemanticSearchQuery> {
  // Normalize for cache key
  const cacheKey = query.toLowerCase().trim();

  // Check multi-tier cache first (L1 in-memory, L2 Redis)
  const cached = await intentCacheInstance.get(cacheKey);
  if (cached) {
    console.log(`📦 Intent cache hit for: "${query.slice(0, 30)}..."`);
    return cached;
  }

  const queryLower = query.toLowerCase();
  let intent: SearchIntent = {};
  let usedFallback = false;

  // Detect person names (actors, directors) - comprehensive patterns
  const personPatterns = [
    // Classic patterns: "starring [Name]", "by [Name]", "featuring [Name]"
    /\b(with|starring|by|featuring|from)\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)\b/i,

    // "[Name] movie/film/show/series"
    /\b([A-Z][a-z]+\s+[A-Z][a-z]+)\s+(movie|film|show|series|movies|films|shows)/i,

    // "actor/director [Name]"
    /\b(actor|director|writer|producer)\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)\b/i,

    // "Something [Name] played/acted/directed/wrote/produced"
    /\b(something|anything|what)\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)\s+(played|acted|directed|wrote|produced|made|starred\s+in|was\s+in)\b/i,

    // "movies/films/shows [Name] played/acted/directed/was in"
    /\b(movies|films|shows|series)\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)\s+(played|acted|directed|wrote|produced|made|starred\s+in|was\s+in|acted\s+in)\b/i,

    // "directed by [Name]", "written by [Name]", "produced by [Name]"
    /\b(directed|written|produced|created)\s+by\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)\b/i,

    // "[Name] directed/wrote/produced"
    /\b([A-Z][a-z]+\s+[A-Z][a-z]+)\s+(directed|wrote|produced|created|made)\b/i,
  ];

  let detectedPerson: string | undefined;
  for (const pattern of personPatterns) {
    const match = query.match(pattern);
    if (match && match.length > 1) {
      // Extract the name from the match groups
      // The name is typically in group 2, but we need to find the capitalized name
      for (let i = 1; i < match.length; i++) {
        const candidate = match[i]?.trim();
        // Check if this looks like a person name (starts with capital, has at least 2 words)
        if (candidate && /^[A-Z][a-z]+\s+[A-Z][a-z]+/.test(candidate)) {
          detectedPerson = candidate;
          break;
        }
      }
      if (detectedPerson) break;
    }
  }

  // Detect trending/recency queries
  const isTrendingQuery = /\b(trending|popular|hot|what'?s new|latest|recent|just released|new release)\b/i.test(query);
  const isRecentQuery = /\b(latest|recent|new|2024|2023|just released|came out)\b/i.test(query);

  // Detect platform queries (Netflix, etc.)
  const platformMatch = query.match(/\b(netflix|hulu|disney|prime|hbo|apple\s*tv)\b/i);
  const detectedPlatform = platformMatch ? platformMatch[1] : undefined;

  // Detect award queries (Oscar, Emmy, Golden Globe, etc.)
  const awardPatterns = [
    /\b(oscar|academy\s+award)s?\s+(winner|winning|nominated|nomination)\b/i,
    /\b(oscar|academy\s+award)s?\s+for\s+best\b/i,
    /\b(emmy|golden\s+globe|bafta|sag|screen\s+actors\s+guild)\s+(winner|winning|award|nominated|nomination)\b/i,
    /\bwon\s+(an?\s+)?(oscar|academy\s+award|emmy|golden\s+globe|bafta|sag)\b/i,
    /\b(oscar|academy\s+award|emmy|golden\s+globe|bafta|sag)[\s-]winning\b/i,
  ];

  let detectedAward: string | undefined;
  for (const pattern of awardPatterns) {
    const match = query.match(pattern);
    if (match) {
      // Extract award type from the match
      const awardMatch = match[0].match(/\b(oscar|academy\s+award|emmy|golden\s+globe|bafta|sag)\b/i);
      if (awardMatch) {
        detectedAward = awardMatch[0];
        break;
      }
    }
  }

  // Try AI-powered parsing first (if API keys are available)
  if (process.env.OPENAI_API_KEY && process.env.OPENAI_API_KEY.length > 10) {
    try {
      console.log(`🧠 AI parsing intent for: "${query.slice(0, 30)}..."`);
      const { object: aiIntent } = await generateObject({
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
      intent = aiIntent as SearchIntent;
    } catch (error) {
      console.log('⚠️  AI parsing failed, using fallback mood detection');
      usedFallback = true;
    }
  } else {
    console.log('💡 Using fallback mood detection (no API key configured)');
    usedFallback = true;
  }

  // Fallback: Pattern-based mood and genre detection
  if (usedFallback || !intent.genres || intent.genres.length === 0) {
    const detectedMoods: string[] = [];
    const detectedGenres: string[] = [];
    const detectedThemes: string[] = [];
    let detectedPacing: 'slow' | 'medium' | 'fast' | undefined;

    // Check for mood keywords
    for (const [mood, config] of Object.entries(MOOD_MAP)) {
      if (queryLower.includes(mood)) {
        detectedMoods.push(mood);
        detectedGenres.push(...config.genres);
        if (config.themes) {
          detectedThemes.push(...config.themes);
        }
        if (config.pacing && !detectedPacing) {
          detectedPacing = config.pacing;
        }
      }
    }

    // Check for direct genre mentions
    for (const [genre] of Object.entries(GENRE_MAP)) {
      if (queryLower.includes(genre)) {
        detectedGenres.push(genre);
      }
    }

    // Special patterns
    if (queryLower.match(/\b(rom-?com|romantic comedy)\b/)) {
      detectedGenres.push('romance', 'comedy');
      detectedMoods.push('romantic');
    }
    if (queryLower.match(/\b(sci-?fi|science fiction)\b/)) {
      detectedGenres.push('sci-fi');
    }
    if (queryLower.match(/\b(tonight|streaming|watch now)\b/)) {
      intent.keywords = [...(intent.keywords || []), 'streaming now'];
    }

    // Update intent with detected values
    if (detectedMoods.length > 0) {
      intent.mood = [...new Set([...(intent.mood || []), ...detectedMoods])];
    }
    if (detectedGenres.length > 0) {
      intent.genres = [...new Set([...(intent.genres || []), ...detectedGenres])];
    }
    if (detectedThemes.length > 0) {
      intent.themes = [...new Set([...(intent.themes || []), ...detectedThemes])];
    }
    if (detectedPacing) {
      intent.pacing = detectedPacing;
    }
  }

  // Convert genre names to IDs
  const genreIds = intent.genres?.flatMap(genre => {
    const normalized = genre.toLowerCase();
    const mapping = GENRE_MAP[normalized];
    if (mapping) {
      return [mapping.movie, mapping.tv].filter((v, i, a) => a.indexOf(v) === i);
    }
    return [];
  }) || [];

  // Add detected person, trending flags, platform, and awards to keywords
  const keywords: string[] = [...(intent.keywords || [])];
  if (detectedPerson) {
    keywords.push(`person:${detectedPerson}`);
  }
  if (isTrendingQuery) {
    keywords.push('trending');
  }
  if (isRecentQuery) {
    keywords.push('recent');
  }
  if (detectedPlatform) {
    keywords.push(`platform:${detectedPlatform}`);
  }
  if (detectedAward) {
    keywords.push(`award:${detectedAward}`);
  }

  const result: SemanticSearchQuery = {
    query,
    intent: Object.keys(intent).length > 0 ? intent : undefined,
    filters: {
      mediaType: intent.mediaType || 'all',
      genres: genreIds.length > 0 ? genreIds : undefined,
    },
    metadata: {
      detectedPerson,
      isTrending: isTrendingQuery,
      isRecent: isRecentQuery,
      platform: detectedPlatform,
      detectedAward,
      hasSpecificIntent: genreIds.length > 0 || detectedPerson !== undefined || detectedAward !== undefined,
    },
  };

  // Update keywords in intent
  if (keywords.length > 0) {
    if (!result.intent) result.intent = {};
    result.intent.keywords = keywords;
  }

  // Cache the result in multi-tier cache
  await intentCacheInstance.set(cacheKey, result);

  console.log(`✅ Parsed intent: ${intent.mood?.join(', ') || 'none'} → genres: ${intent.genres?.join(', ') || 'none'} | person: ${detectedPerson || 'none'} | award: ${detectedAward || 'none'} | trending: ${isTrendingQuery}`);

  return result;
}

/**
 * Perform semantic search combining TMDB and vector search
 * Now includes streaming availability data
 */
export async function semanticSearch(
  query: string,
  userPreferences?: number[],
  options?: {
    includeStreaming?: boolean;
    region?: string;
    filterByService?: number[];
    sessionId?: string;
  }
): Promise<SearchResult[]> {
  const startTime = Date.now();
  const sessionId = options?.sessionId || 'anonymous';

  // Check search result cache
  const cacheKey = createSearchCacheKey(query, userPreferences, options);
  const cachedResults = await searchResultCacheInstance.get(cacheKey);
  if (cachedResults) {
    console.log(`📦 Search result cache hit for: "${query.slice(0, 30)}..."`);
    trackSearchCompleted(sessionId, query, {
      count: cachedResults.length,
      latencyMs: Date.now() - startTime,
      cacheHit: true,
      topIds: cachedResults.slice(0, 5).map(r => r.content.id),
    });
    return cachedResults;
  }

  // Track search initiation
  trackSearchInitiated(sessionId, query, {
    streamingService: options?.filterByService?.length ? 'specified' : undefined,
  });

  // Parse the natural language query
  const semanticQuery = await parseSearchQuery(query);

  // Check if user mentioned a specific streaming service
  const streamingPref = parseStreamingPreference(query);
  const filterServices = options?.filterByService || streamingPref.serviceIds;

  // Parallel search strategies
  const [tmdbResults, vectorResults] = await Promise.all([
    performTMDBSearch(semanticQuery),
    performVectorSearch(semanticQuery),
  ]);

  // Merge and rank results
  let mergedResults = mergeAndRankResults(tmdbResults, vectorResults, semanticQuery, userPreferences);

  // Enrich with streaming availability if requested (default: true for top results)
  const includeStreaming = options?.includeStreaming ?? true;
  if (includeStreaming && mergedResults.length > 0) {
    // Get streaming data for top 20 results
    const topResults = mergedResults.slice(0, 20);
    const streamingData = await getBatchStreamingAvailability(
      topResults.map(r => ({ id: r.content.id, mediaType: r.content.mediaType })),
      options?.region || 'US'
    );

    // Enrich results with streaming info
    mergedResults = mergedResults.map(result => {
      const key = `${result.content.mediaType}-${result.content.id}`;
      const availability = streamingData.get(key);

      if (availability) {
        const providers: StreamingInfo[] = [
          ...availability.providers.flatrate.map(p => ({
            provider: p.provider.name,
            providerLogo: p.provider.logoPath,
            availabilityType: 'flatrate' as const,
          })),
          ...availability.providers.free.map(p => ({
            provider: p.provider.name,
            providerLogo: p.provider.logoPath,
            availabilityType: 'free' as const,
          })),
        ];

        return {
          ...result,
          streaming: {
            isAvailable: isStreamingNow(availability) || availability.providers.free.length > 0,
            providers,
            formattedText: formatStreamingProviders(availability),
            badge: getStreamingBadge(availability),
          },
        };
      }
      return result;
    });

    // If user specified a streaming service, filter and boost those results
    if (filterServices.length > 0) {
      mergedResults = mergedResults.map(result => {
        const key = `${result.content.mediaType}-${result.content.id}`;
        const availability = streamingData.get(key);

        if (availability && filterByStreamingService(availability, filterServices)) {
          // Boost score for matching streaming service
          return {
            ...result,
            relevanceScore: Math.min(1, result.relevanceScore + 0.15),
            matchReasons: [
              ...result.matchReasons,
              `Available on ${streamingPref.serviceName || 'your streaming service'}`
            ],
          };
        }
        return result;
      });

      // Re-sort by updated relevance
      mergedResults.sort((a, b) => b.relevanceScore - a.relevanceScore);
    }
  }

  // Cache the results
  await searchResultCacheInstance.set(cacheKey, mergedResults);

  // Track search completion
  trackSearchCompleted(sessionId, query, {
    count: mergedResults.length,
    latencyMs: Date.now() - startTime,
    cacheHit: false,
    topIds: mergedResults.slice(0, 5).map(r => r.content.id),
  }, {
    intents: semanticQuery.intent?.themes || [],
  });

  return mergedResults;
}

/**
 * Perform TMDB-based search with intent understanding
 */
async function performTMDBSearch(query: SemanticSearchQuery): Promise<SearchResult[]> {
  const results: SearchResult[] = [];

  // 1. PERSON SEARCH - Check if query is about a specific person (actor/director)
  const detectedPerson = query.metadata?.detectedPerson;
  if (detectedPerson) {
    try {
      const { results: personResults } = await searchPerson(detectedPerson);

      if (personResults.length > 0) {
        const topPerson = personResults[0];
        console.log(`🎭 Found person: ${topPerson.name} (ID: ${topPerson.id}), fetching full credits...`);

        // Get comprehensive credits (all movies/shows they were in)
        const credits = await getPersonCredits(topPerson.id);

        // Filter cast roles to prioritize major appearances (high vote count and popularity)
        // This filters out minor guest appearances and focuses on main roles

        // Detect if user wants movies or TV shows specifically
        const queryLower = query.query.toLowerCase();
        const wantsMovies = /\b(movie|film|cinema)\b/.test(queryLower);
        const wantsTV = /\b(tv|show|series|episode)\b/.test(queryLower);

        const majorCastRoles = credits.cast
          .filter(content => {
            // Require minimum vote count to filter out obscure/minor appearances
            const hasSignificantVotes = content.voteCount >= 100;
            // Require reasonable popularity score
            const isPopular = content.popularity >= 5;
            // Require decent rating (filters out very poorly rated content)
            const isWellRated = content.voteAverage >= 5.0;

            // If user specifically wants movies, filter out TV shows
            if (wantsMovies && !wantsTV && content.mediaType === 'tv') {
              return false;
            }

            // If user specifically wants TV, filter out movies
            if (wantsTV && !wantsMovies && content.mediaType === 'movie') {
              return false;
            }

            return hasSignificantVotes && isPopular && isWellRated;
          })
          .slice(0, 20); // Top 20 major roles

        // Filter crew roles similarly (for directors, producers, etc.)
        const majorCrewRoles = credits.crew
          .filter(content => {
            const hasSignificantVotes = content.voteCount >= 50;
            const isPopular = content.popularity >= 3;
            return hasSignificantVotes && isPopular;
          })
          .slice(0, 10); // Top 10 crew roles

        // Combine cast and crew, prioritizing cast roles with variable relevance scores
        const allWork = [
          ...majorCastRoles.map((content, index) => ({
            content,
            relevanceScore: 0.95 - (index * 0.01), // Slightly decay for lower positions
            matchReasons: [`${topPerson.name} starred in this`],
          })),
          ...majorCrewRoles.map((content, index) => ({
            content,
            relevanceScore: 0.88 - (index * 0.01),
            matchReasons: [`${topPerson.name} worked on this`],
          })),
        ];

        console.log(`✅ Found ${allWork.length} major credits for ${topPerson.name} (filtered from ${credits.cast.length + credits.crew.length} total)`);
        results.push(...allWork);

        // Fallback: If filtering was too aggressive and we have no results, relax the filters
        if (results.length === 0 && (credits.cast.length > 0 || credits.crew.length > 0)) {
          console.log(`⚠️ Relaxing filters for ${topPerson.name}...`);
          const relaxedWork = [
            ...credits.cast.slice(0, 15).map((content, index) => ({
              content,
              relevanceScore: 0.92 - (index * 0.02),
              matchReasons: [`${topPerson.name} appeared in this`],
            })),
            ...credits.crew.slice(0, 8).map((content, index) => ({
              content,
              relevanceScore: 0.85 - (index * 0.02),
              matchReasons: [`${topPerson.name} worked on this`],
            })),
          ];
          results.push(...relaxedWork);
          console.log(`✅ Added ${relaxedWork.length} results with relaxed filters`);
        }

        // Final fallback: If credits API fails entirely, use knownFor
        if (results.length === 0) {
          console.log(`⚠️ Using fallback knownFor data`);
          results.push(...topPerson.knownFor.map(content => ({
            content,
            relevanceScore: 0.92,
            matchReasons: [`Features ${topPerson.name}`],
          })));
        }
      } else {
        console.log(`⚠️ No person found matching: ${detectedPerson}`);
      }
    } catch (error) {
      console.error('Person search failed:', error);
    }
  }

  // 2. AWARD-WINNING CONTENT - Handle Oscar, Emmy, etc. queries
  if (query.metadata?.detectedAward) {
    try {
      console.log(`🏆 Searching for award-winning content: ${query.metadata.detectedAward}...`);

      // For Oscar/Academy Awards, search for highly acclaimed films
      const isOscar = /oscar|academy\s+award/i.test(query.metadata.detectedAward);
      const isEmmy = /emmy/i.test(query.metadata.detectedAward);
      const isGoldenGlobe = /golden\s+globe/i.test(query.metadata.detectedAward);
      const isSAG = /sag|screen\s+actors\s+guild/i.test(query.metadata.detectedAward);
      const isBAFTA = /bafta/i.test(query.metadata.detectedAward);

      // Search for highly rated content with strict filters
      // Award winners typically have: high ratings, many votes, and high popularity
      const [awardMovies, awardShows] = await Promise.all([
        // Movies with very high ratings and significant vote counts (likely award winners)
        (isOscar || isGoldenGlobe || isSAG || isBAFTA || !isEmmy) ? discoverMovies({
          sortBy: 'vote_average.desc',
          ratingMin: 7.8, // Higher threshold for awards
          page: 1,
        }).catch(() => ({ results: [] })) : Promise.resolve({ results: [] }),

        // TV shows with very high ratings (for Emmy queries)
        (isEmmy || isGoldenGlobe) ? discoverTVShows({
          sortBy: 'vote_average.desc',
          ratingMin: 8.2, // Higher threshold for TV awards
          page: 1,
        }).catch(() => ({ results: [] })) : Promise.resolve({ results: [] }),
      ]);

      // Filter movies/shows to only include those with substantial vote counts
      // This further filters out obscure content and focuses on well-known acclaimed works
      const qualityMovies = awardMovies.results.filter(
        content => content.voteCount >= 100 && content.popularity >= 5
      );
      const qualityShows = awardShows.results.filter(
        content => content.voteCount >= 50 && content.popularity >= 5
      );

      // Add award-winning content with high relevance scores
      if (isOscar && qualityMovies.length > 0) {
        // For Oscars, prioritize critically acclaimed films with high vote counts
        results.push(...qualityMovies.slice(0, 15).map((content, index) => ({
          content,
          relevanceScore: 0.93 - (index * 0.01),
          matchReasons: ['Critically acclaimed film / Likely Oscar contender'],
        })));
        console.log(`🏆 Added ${Math.min(15, qualityMovies.length)} Oscar-caliber films`);
      }

      if (isEmmy && qualityShows.length > 0) {
        // For Emmy, prioritize acclaimed TV series
        results.push(...qualityShows.slice(0, 15).map((content, index) => ({
          content,
          relevanceScore: 0.93 - (index * 0.01),
          matchReasons: ['Critically acclaimed series / Likely Emmy contender'],
        })));
        console.log(`📺 Added ${Math.min(15, qualityShows.length)} Emmy-caliber shows`);
      }

      if (isGoldenGlobe) {
        // Golden Globes cover both film and TV
        const combinedResults = [
          ...qualityMovies.slice(0, 10).map((content, index) => ({
            content,
            relevanceScore: 0.92 - (index * 0.01),
            matchReasons: ['Critically acclaimed film / Golden Globe caliber'],
          })),
          ...qualityShows.slice(0, 8).map((content, index) => ({
            content,
            relevanceScore: 0.91 - (index * 0.01),
            matchReasons: ['Acclaimed series / Golden Globe caliber'],
          })),
        ];
        results.push(...combinedResults);
        console.log(`🌟 Added ${combinedResults.length} Golden Globe-caliber content`);
      }

      if ((isSAG || isBAFTA) && qualityMovies.length > 0) {
        // SAG and BAFTA are primarily film awards
        results.push(...qualityMovies.slice(0, 12).map((content, index) => ({
          content,
          relevanceScore: 0.91 - (index * 0.01),
          matchReasons: [`Critically acclaimed film / ${isSAG ? 'SAG' : 'BAFTA'} caliber`],
        })));
      }

      // Generic award query (if no specific award matched)
      if (!isOscar && !isEmmy && !isGoldenGlobe && !isSAG && !isBAFTA) {
        const combinedResults = [
          ...qualityMovies.slice(0, 10).map((content, index) => ({
            content,
            relevanceScore: 0.90 - (index * 0.01),
            matchReasons: ['Critically acclaimed / Award-worthy film'],
          })),
          ...qualityShows.slice(0, 5).map((content, index) => ({
            content,
            relevanceScore: 0.88 - (index * 0.01),
            matchReasons: ['Critically acclaimed / Award-worthy series'],
          })),
        ];
        results.push(...combinedResults);
      }

      console.log(`✅ Added ${results.length} award-caliber content results`);
    } catch (error) {
      console.error('Award content search failed:', error);
    }
  }

  // 3. TRENDING/RECENT CONTENT - Handle "what's new", "trending", etc.
  if (query.metadata?.isTrending) {
    try {
      const trendingContent = await getTrending('all', 'week');
      console.log(`📈 Adding trending content (${trendingContent.length} items)...`);

      results.push(...trendingContent.slice(0, 15).map((content, index) => ({
        content,
        relevanceScore: 0.88 - (index * 0.02),
        matchReasons: ['Currently trending'],
      })));
    } catch (error) {
      console.error('Trending search failed:', error);
    }
  }

  if (query.metadata?.isRecent) {
    try {
      const [recentMovies, recentShows] = await Promise.all([
        getRecentReleases('movie'),
        getRecentReleases('tv'),
      ]);
      console.log(`🆕 Adding recent releases...`);

      const mediaType = query.filters?.mediaType;
      if (mediaType !== 'tv') {
        results.push(...recentMovies.results.slice(0, 10).map((content, index) => ({
          content,
          relevanceScore: 0.86 - (index * 0.02),
          matchReasons: ['Recent release'],
        })));
      }
      if (mediaType !== 'movie') {
        results.push(...recentShows.results.slice(0, 10).map((content, index) => ({
          content,
          relevanceScore: 0.86 - (index * 0.02),
          matchReasons: ['Recent release'],
        })));
      }
    } catch (error) {
      console.error('Recent releases search failed:', error);
    }
  }

  // 4. TEXT SEARCH - Traditional title/keyword search
  if (query.query && !detectedPerson) { // Skip if we already did person search
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

  // 5. SIMILAR CONTENT SEARCH - If references found (but DON'T overshadow direct matches)
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

  // 6. DISCOVERY-BASED SEARCH - Genre/filter based discovery
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

  // 7. FALLBACK STRATEGY - If no results yet, return trending/popular content
  // This ensures ZERO empty results for vague queries like "show me something cool"
  if (results.length === 0) {
    console.log(`⚠️ No results found, applying fallback strategy...`);

    try {
      // Return a mix of trending and popular content
      const [trending, popularMovies, popularShows] = await Promise.all([
        getTrending('all', 'week').catch(() => []),
        discoverMovies({ sortBy: 'popularity.desc', ratingMin: 7.0 }).catch(() => ({ results: [] })),
        discoverTVShows({ sortBy: 'popularity.desc', ratingMin: 7.0 }).catch(() => ({ results: [] })),
      ]);

      // Add trending content
      results.push(...trending.slice(0, 10).map((content, index) => ({
        content,
        relevanceScore: 0.75 - (index * 0.02),
        matchReasons: ['Popular & Trending'],
      })));

      // Add popular movies
      const mediaType = query.filters?.mediaType;
      if (mediaType !== 'tv') {
        results.push(...popularMovies.results.slice(0, 8).map((content, index) => ({
          content,
          relevanceScore: 0.72 - (index * 0.02),
          matchReasons: ['Highly rated & Popular'],
        })));
      }

      // Add popular shows
      if (mediaType !== 'movie') {
        results.push(...popularShows.results.slice(0, 8).map((content, index) => ({
          content,
          relevanceScore: 0.72 - (index * 0.02),
          matchReasons: ['Highly rated & Popular'],
        })));
      }

      console.log(`✅ Fallback added ${results.length} results`);
    } catch (error) {
      console.error('Fallback strategy failed:', error);
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

  // Apply content freshness scoring
  // Boost newer content based on release date
  const wantsNew = /\b(new|recent|latest|2024|2023|just released|came out)\b/i.test(query.query);
  results = results.map(result => {
    const releaseDate = new Date(result.content.releaseDate);
    const now = new Date();
    const daysSinceRelease = Math.floor((now.getTime() - releaseDate.getTime()) / (1000 * 60 * 60 * 24));

    // Calculate recency boost using exponential decay
    // More aggressive boost if user explicitly wants new content
    let recencyBoost = 0;

    if (daysSinceRelease < 0) {
      // Future release - slight boost for anticipation
      recencyBoost = 0.02;
    } else if (daysSinceRelease <= 30) {
      // Less than 1 month - very fresh
      recencyBoost = wantsNew ? 0.15 : 0.08;
      result.matchReasons.push('New release');
    } else if (daysSinceRelease <= 90) {
      // 1-3 months - still fresh
      recencyBoost = wantsNew ? 0.10 : 0.05;
      result.matchReasons.push('Recent release');
    } else if (daysSinceRelease <= 365) {
      // Within a year
      recencyBoost = wantsNew ? 0.05 : 0.02;
    } else {
      // Classic content - no penalty, just no boost
      // (some users specifically want classic content)
      recencyBoost = 0;
    }

    // Additional boost for trending recent content
    if (daysSinceRelease <= 90 && result.content.popularity > 100) {
      recencyBoost += 0.03;
      if (!result.matchReasons.includes('Trending')) {
        result.matchReasons.push('Trending');
      }
    }

    return {
      ...result,
      relevanceScore: Math.min(1, result.relevanceScore + recencyBoost),
    };
  });

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
