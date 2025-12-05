/**
 * Natural Language Search API
 * POST /api/search
 *
 * Accepts natural language queries and returns semantically matched content
 */

import { NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import { semanticSearch, parseSearchQuery, explainRecommendation } from '@/lib/natural-language-search';

// Request schema
const SearchRequestSchema = z.object({
  query: z.string().min(1).max(500),
  filters: z.object({
    mediaType: z.enum(['movie', 'tv', 'all']).optional(),
    genres: z.array(z.number()).optional(),
    yearRange: z.object({
      min: z.number().optional(),
      max: z.number().optional(),
    }).optional(),
    ratingMin: z.number().min(0).max(10).optional(),
  }).optional(),
  preferences: z.array(z.number()).optional(), // User's preferred genre IDs
  explain: z.boolean().optional(), // Whether to include AI explanations
  limit: z.number().min(1).max(50).optional(),
});

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { query, filters, preferences, explain, limit = 20 } = SearchRequestSchema.parse(body);

    // Perform semantic search
    const results = await semanticSearch(query, preferences);

    // Apply limit
    const limitedResults = results.slice(0, limit);

    // Add explanations if requested
    let finalResults = limitedResults;
    if (explain) {
      finalResults = await Promise.all(
        limitedResults.map(async (result) => ({
          ...result,
          explanation: await explainRecommendation(
            result.content,
            query,
            result.matchReasons
          ),
        }))
      );
    }

    // Parse query to return intent (for debugging/UI)
    const parsedQuery = await parseSearchQuery(query);

    return NextResponse.json({
      success: true,
      query: query,
      intent: parsedQuery.intent,
      results: finalResults,
      totalResults: results.length,
    });
  } catch (error) {
    console.error('Search error:', error);

    if (error instanceof z.ZodError) {
      return NextResponse.json(
        { success: false, error: 'Invalid request', details: error.errors },
        { status: 400 }
      );
    }

    return NextResponse.json(
      { success: false, error: 'Search failed' },
      { status: 500 }
    );
  }
}

// GET endpoint for simple text searches
export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const query = searchParams.get('q');

  if (!query) {
    return NextResponse.json(
      { success: false, error: 'Query parameter "q" is required' },
      { status: 400 }
    );
  }

  try {
    const results = await semanticSearch(query);

    return NextResponse.json({
      success: true,
      query,
      results: results.slice(0, 20),
      totalResults: results.length,
    });
  } catch (error) {
    console.error('Search error:', error);
    return NextResponse.json(
      { success: false, error: 'Search failed' },
      { status: 500 }
    );
  }
}
