/**
 * TV Show Details API
 * GET /api/tv/[id]
 *
 * Returns full TV show details with credits, videos, similar, and recommendations
 */

import { NextRequest, NextResponse } from 'next/server';
import { getFullTVShowDetails } from '@/lib/tmdb';

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params;
    const showId = parseInt(id, 10);

    if (isNaN(showId)) {
      return NextResponse.json(
        { success: false, error: 'Invalid TV show ID' },
        { status: 400 }
      );
    }

    const data = await getFullTVShowDetails(showId);

    return NextResponse.json({
      success: true,
      ...data,
    });
  } catch (error) {
    console.error('TV show details error:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to fetch TV show details' },
      { status: 500 }
    );
  }
}
