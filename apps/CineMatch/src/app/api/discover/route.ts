/**
 * Content Discovery API
 * GET /api/discover
 *
 * Returns trending and popular content for browsing
 */

import { NextRequest, NextResponse } from 'next/server';
import {
  getTrending,
  getPopularMovies,
  getPopularTVShows,
  discoverMovies,
  discoverTVShows,
} from '@/lib/tmdb';

export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const category = searchParams.get('category') || 'trending';
  const mediaType = searchParams.get('type') as 'movie' | 'tv' | 'all' || 'all';
  const page = parseInt(searchParams.get('page') || '1', 10);
  const genres = searchParams.get('genres')?.split(',').map(Number).filter(Boolean);
  const yearMin = searchParams.get('yearMin') ? parseInt(searchParams.get('yearMin')!, 10) : undefined;
  const yearMax = searchParams.get('yearMax') ? parseInt(searchParams.get('yearMax')!, 10) : undefined;
  const ratingMin = searchParams.get('ratingMin') ? parseFloat(searchParams.get('ratingMin')!) : undefined;

  try {
    let results;
    let totalPages = 1;

    switch (category) {
      case 'trending':
        results = await getTrending(mediaType, 'week');
        break;

      case 'popular':
        if (mediaType === 'movie') {
          const movies = await getPopularMovies(page);
          results = movies.results;
          totalPages = movies.totalPages;
        } else if (mediaType === 'tv') {
          const shows = await getPopularTVShows(page);
          results = shows.results;
          totalPages = shows.totalPages;
        } else {
          const [movies, shows] = await Promise.all([
            getPopularMovies(page),
            getPopularTVShows(page),
          ]);
          results = [...movies.results, ...shows.results]
            .sort((a, b) => b.popularity - a.popularity);
          totalPages = Math.max(movies.totalPages, shows.totalPages);
        }
        break;

      case 'discover':
        const discoverOptions = { genres, yearMin, yearMax, ratingMin, page };
        if (mediaType === 'movie') {
          const movies = await discoverMovies(discoverOptions);
          results = movies.results;
          totalPages = movies.totalPages;
        } else if (mediaType === 'tv') {
          const shows = await discoverTVShows(discoverOptions);
          results = shows.results;
          totalPages = shows.totalPages;
        } else {
          const [movies, shows] = await Promise.all([
            discoverMovies(discoverOptions),
            discoverTVShows(discoverOptions),
          ]);
          results = [...movies.results, ...shows.results]
            .sort((a, b) => b.popularity - a.popularity);
          totalPages = Math.max(movies.totalPages, shows.totalPages);
        }
        break;

      default:
        return NextResponse.json(
          { success: false, error: 'Invalid category' },
          { status: 400 }
        );
    }

    return NextResponse.json({
      success: true,
      category,
      mediaType,
      page,
      totalPages,
      results,
    });
  } catch (error) {
    console.error('Discovery error:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to fetch content' },
      { status: 500 }
    );
  }
}
