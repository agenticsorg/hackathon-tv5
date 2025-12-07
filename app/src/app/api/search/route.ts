import { NextRequest, NextResponse } from 'next/server';
import { searchContent } from '@/lib/db';

// Force dynamic rendering - don't try to pre-render during build
export const dynamic = 'force-dynamic';

export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const query = searchParams.get('q');
  const limit = parseInt(searchParams.get('limit') || '20');

  if (!query) {
    return NextResponse.json({ error: 'Query parameter required' }, { status: 400 });
  }

  try {
    const results = await searchContent(query, limit);
    return NextResponse.json({
      query,
      results,
      count: results.length,
      timestamp: new Date().toISOString(),
    });
  } catch (error: any) {
    console.error('Search error:', error);
    return NextResponse.json({ error: error.message }, { status: 500 });
  }
}
