/**
 * Movie Details API
 * GET /api/movies/[id]
 *
 * Returns full movie details with credits, videos, similar, and recommendations
 */

import { NextRequest, NextResponse } from 'next/server';
import { getFullMovieDetails } from '@/lib/tmdb';

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params;
    const movieId = parseInt(id, 10);

    if (isNaN(movieId)) {
      return NextResponse.json(
        { success: false, error: 'Invalid movie ID' },
        { status: 400 }
      );
    }

    const data = await getFullMovieDetails(movieId);

    return NextResponse.json({
      success: true,
      ...data,
    });
  } catch (error) {
    console.error('Movie details error:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to fetch movie details' },
      { status: 500 }
    );
  }
}
