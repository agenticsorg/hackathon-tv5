/**
 * Watch Party API
 * POST /api/watch-party - Create and search for a watch party
 *
 * Merges preferences from multiple users and finds content everyone will enjoy
 */

import { NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import { applyRateLimit, getRateLimitHeaders } from '@/lib/rate-limit';
import {
  createWatchParty,
  mergePreferences,
  scoreResultsForParty,
  generatePartySearchQuery,
  type PartyMember,
} from '@/lib/watch-party';
import { semanticSearch } from '@/lib/natural-language-search';
import { trackEvent } from '@/lib/analytics';

// Member schema
const PartyMemberSchema = z.object({
  userId: z.string(),
  name: z.string(),
  preferences: z.object({
    favoriteGenres: z.array(z.number()),
    dislikedGenres: z.array(z.number()).default([]),
    moodPreferences: z.array(z.string()).optional(),
    recentlyWatched: z.array(z.number()).optional(),
  }),
});

// Request schema
const WatchPartyRequestSchema = z.object({
  members: z.array(PartyMemberSchema).min(2).max(10),
  partyName: z.string().optional(),
  customQuery: z.string().optional(), // Optional additional query constraints
  region: z.string().optional(),
  includeStreaming: z.boolean().optional(),
  limit: z.number().min(1).max(50).optional(),
});

export async function POST(request: NextRequest) {
  // Check rate limit (use recommendations limit - higher for group feature)
  const rateLimitResponse = applyRateLimit(request, 'recommendations');
  if (rateLimitResponse) {
    return rateLimitResponse;
  }

  const sessionId = request.headers.get('x-session-id') || 'anonymous';

  try {
    const body = await request.json();
    const {
      members,
      partyName,
      customQuery,
      region = 'US',
      includeStreaming = true,
      limit = 20,
    } = WatchPartyRequestSchema.parse(body);

    // Track watch party creation
    trackEvent(sessionId, 'watch_party_created', {
      memberCount: members.length,
      hasCustomQuery: !!customQuery,
    });

    // Create watch party
    const party = createWatchParty(members as PartyMember[], partyName);

    // Merge preferences
    const mergedPrefs = mergePreferences(party);

    // Generate search query from merged preferences
    const generatedQuery = generatePartySearchQuery(mergedPrefs);
    const searchQuery = customQuery
      ? `${customQuery} - ${generatedQuery}`
      : generatedQuery;

    // Perform search with merged genre preferences
    const results = await semanticSearch(
      searchQuery,
      mergedPrefs.genres.slice(0, 5).map(g => g.genreId),
      {
        includeStreaming,
        region,
      }
    );

    // Score results for the party
    const partyResults = scoreResultsForParty(results, mergedPrefs, party);

    // Get rate limit headers
    const rateLimitHeaders = getRateLimitHeaders(request, 'recommendations');

    return NextResponse.json({
      success: true,
      partyId: party.partyId,
      partyName: party.name,
      memberCount: members.length,
      query: searchQuery,
      mergedPreferences: {
        topGenres: mergedPrefs.genres.slice(0, 5).map(g => ({
          genreId: g.genreId,
          score: Math.round(g.score * 100) / 100,
          supporters: g.supportingMembers.length,
        })),
        excludedGenres: mergedPrefs.excludeGenres,
        moods: mergedPrefs.moods,
        conflicts: mergedPrefs.conflicts.length,
      },
      results: partyResults.slice(0, limit),
      totalResults: partyResults.length,
      conflicts: mergedPrefs.conflicts,
    }, {
      headers: rateLimitHeaders,
    });
  } catch (error) {
    console.error('Watch party error:', error);

    if (error instanceof z.ZodError) {
      return NextResponse.json(
        { success: false, error: 'Invalid request', details: error.errors },
        { status: 400 }
      );
    }

    return NextResponse.json(
      { success: false, error: 'Watch party search failed' },
      { status: 500 }
    );
  }
}

// GET endpoint for party info (would need persistence for real use)
export async function GET(request: NextRequest) {
  return NextResponse.json({
    success: true,
    info: {
      description: 'Watch Party API - Find content everyone will enjoy',
      usage: 'POST with members array containing userId, name, and preferences',
      features: [
        'Democratic genre voting',
        'Veto system for strong dislikes',
        'Fairness tracking across sessions',
        'Streaming availability filtering',
      ],
      limits: {
        minMembers: 2,
        maxMembers: 10,
        maxResults: 50,
      },
    },
  });
}
