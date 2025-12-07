/**
 * API endpoint to test metadata search improvements
 * Test queries:
 * 1. GET /api/test-metadata?query=Something Richard Gere played
 * 2. GET /api/test-metadata?query=an Oscar winner movie
 */

import { NextRequest, NextResponse } from 'next/server';
import { parseSearchQuery, semanticSearch } from '@/lib/natural-language-search';

export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const query = searchParams.get('query');

  if (!query) {
    return NextResponse.json({
      error: 'Missing query parameter',
      usage: 'GET /api/test-metadata?query=Something Richard Gere played',
    }, { status: 400 });
  }

  try {
    console.log(`\n🔍 Testing metadata search: "${query}"`);

    // Parse the query
    const parsedQuery = await parseSearchQuery(query);

    console.log('📊 Parsed Query:');
    console.log('- Detected Person:', parsedQuery.metadata?.detectedPerson || 'none');
    console.log('- Detected Award:', parsedQuery.metadata?.detectedAward || 'none');
    console.log('- Has Specific Intent:', parsedQuery.metadata?.hasSpecificIntent);

    // Perform search
    const results = await semanticSearch(query, undefined, { includeStreaming: false });

    console.log(`✅ Found ${results.length} results`);

    // Analyze results
    const hasFallback = results.some(r =>
      r.matchReasons.includes('Popular & Trending') ||
      r.matchReasons.includes('Highly rated & Popular')
    );

    const hasPersonMatches = results.some(r =>
      r.matchReasons.some(reason => reason.toLowerCase().includes('starred') || reason.toLowerCase().includes('worked'))
    );

    const hasAwardMatches = results.some(r =>
      r.matchReasons.some(reason => reason.toLowerCase().includes('award') || reason.toLowerCase().includes('acclaimed'))
    );

    return NextResponse.json({
      query,
      parsed: {
        detectedPerson: parsedQuery.metadata?.detectedPerson,
        detectedAward: parsedQuery.metadata?.detectedAward,
        hasSpecificIntent: parsedQuery.metadata?.hasSpecificIntent,
        genres: parsedQuery.intent?.genres,
        keywords: parsedQuery.intent?.keywords,
      },
      results: {
        count: results.length,
        usingFallback: hasFallback,
        hasPersonMatches,
        hasAwardMatches,
        top5: results.slice(0, 5).map(r => ({
          title: r.content.title,
          year: r.content.releaseDate?.split('-')[0],
          rating: r.content.voteAverage,
          relevanceScore: r.relevanceScore,
          matchReasons: r.matchReasons,
        })),
      },
      validation: {
        personDetectionWorking: parsedQuery.metadata?.detectedPerson !== undefined || parsedQuery.metadata?.detectedAward !== undefined,
        resultsNotEmpty: results.length > 0,
        notUsingFallback: !hasFallback,
        hasRelevantMatches: hasPersonMatches || hasAwardMatches,
        overallSuccess: (
          (parsedQuery.metadata?.detectedPerson !== undefined || parsedQuery.metadata?.detectedAward !== undefined) &&
          results.length > 0 &&
          !hasFallback &&
          (hasPersonMatches || hasAwardMatches)
        ),
      },
    });

  } catch (error) {
    console.error('Test failed:', error);
    return NextResponse.json({
      error: 'Test failed',
      message: error instanceof Error ? error.message : String(error),
    }, { status: 500 });
  }
}
