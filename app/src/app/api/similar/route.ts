import { NextRequest, NextResponse } from 'next/server';
import { getSimilarWithHyperbolic } from '@/lib/ruvector-learning';

// Force dynamic rendering - don't try to pre-render during build
export const dynamic = 'force-dynamic';

export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const id = searchParams.get('id');
  const limit = parseInt(searchParams.get('limit') || '10');

  if (!id) {
    return NextResponse.json({ error: 'Content ID required' }, { status: 400 });
  }

  try {
    // Use hyperbolic vector distance for better hierarchical similarity
    const results = await getSimilarWithHyperbolic(id, limit);

    return NextResponse.json({
      referenceId: id,
      results,
      count: results.length,
      vectorSpace: {
        type: 'hyperbolic', // Falls back to cosine if not available
        model: 'poincare_ball',
        dimensions: 384
      },
      timestamp: new Date().toISOString(),
    });
  } catch (error: any) {
    console.error('Similar content error:', error);
    return NextResponse.json({ error: error.message }, { status: 500 });
  }
}
