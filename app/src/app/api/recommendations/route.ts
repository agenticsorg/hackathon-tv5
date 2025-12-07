import { NextRequest, NextResponse } from 'next/server';
import { getOptimizedRecommendations, getLearningStats } from '@/lib/ruvector-learning';

// Force dynamic rendering - don't try to pre-render during build
export const dynamic = 'force-dynamic';

export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const mood = searchParams.get('mood') || undefined;
  const contentType = searchParams.get('type') || undefined;
  const limitParam = searchParams.get('limit');
  const limit = limitParam ? parseInt(limitParam) : undefined;  // Use config default if not specified
  const includeStats = searchParams.get('stats') === 'true';
  const audience = searchParams.get('audience') as 'kids' | 'family' | 'teens' | 'adults' | undefined;

  try {
    // Use Q-Learning optimized recommendations with content safety filtering
    const results = await getOptimizedRecommendations(mood, contentType, limit, audience);

    const response: any = {
      mood,
      contentType,
      audience,
      results,
      count: results.length,
      optimization: {
        algorithm: 'Q-Learning',
        patternBoosting: true,
        hyperbolicVectors: false, // Will be true when enabled
        contentSafety: audience ? `${audience}-filtered` : 'none'
      },
      timestamp: new Date().toISOString(),
    };

    // Optionally include learning stats
    if (includeStats) {
      response.learningStats = await getLearningStats();
    }

    return NextResponse.json(response);
  } catch (error: any) {
    console.error('Recommendations error:', error);
    return NextResponse.json({ error: error.message }, { status: 500 });
  }
}
