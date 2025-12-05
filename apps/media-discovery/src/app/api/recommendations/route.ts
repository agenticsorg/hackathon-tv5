/**
 * Personalized Recommendations API
 * GET /api/recommendations
 * POST /api/recommendations
 *
 * Returns personalized content recommendations based on user preferences
 */

import { NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import { getSimilarMovies, getSimilarTVShows, discoverMovies, discoverTVShows, getTrending } from '@/lib/tmdb';
import { findSimilarContent } from '@/lib/vector-search';
import type { MediaContent, Recommendation } from '@/types/media';

// Request schema for POST
const RecommendationsRequestSchema = z.object({
  userId: z.string().optional(),
  basedOn: z.object({
    contentId: z.number(),
    mediaType: z.enum(['movie', 'tv']),
  }).optional(),
  preferences: z.object({
    genres: z.array(z.number()).optional(),
    likedContentIds: z.array(z.number()).optional(),
    dislikedContentIds: z.array(z.number()).optional(),
    preferenceVector: z.array(z.number()).optional(),
  }).optional(),
  limit: z.number().min(1).max(50).optional(),
});

/**
 * GET - Simple recommendations based on query params
 */
export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const contentId = searchParams.get('contentId') ? parseInt(searchParams.get('contentId')!, 10) : undefined;
  const mediaType = searchParams.get('mediaType') as 'movie' | 'tv' | undefined;
  const limit = parseInt(searchParams.get('limit') || '10', 10);

  try {
    let recommendations: Recommendation[] = [];

    if (contentId && mediaType) {
      // Content-based recommendations
      const similar = mediaType === 'movie'
        ? await getSimilarMovies(contentId, 1)
        : await getSimilarTVShows(contentId, 1);

      recommendations = similar.slice(0, limit).map((content, index) => ({
        content,
        score: 1 - (index * 0.05), // Decreasing score
        reasons: [`Similar to content you're viewing`],
        basedOn: {
          type: 'similar' as const,
          references: [contentId.toString()],
        },
      }));
    } else {
      // Default: trending content
      const trending = await getTrending('all', 'week');
      recommendations = trending.slice(0, limit).map((content, index) => ({
        content,
        score: 1 - (index * 0.02),
        reasons: ['Trending this week'],
        basedOn: {
          type: 'trending' as const,
          references: [],
        },
      }));
    }

    return NextResponse.json({
      success: true,
      recommendations,
    });
  } catch (error) {
    console.error('Recommendations error:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to get recommendations' },
      { status: 500 }
    );
  }
}

/**
 * POST - Advanced personalized recommendations
 */
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { basedOn, preferences, limit = 20 } = RecommendationsRequestSchema.parse(body);

    const recommendations: Recommendation[] = [];
    const seen = new Set<string>();

    // Strategy 1: Similar content (if basedOn provided)
    if (basedOn) {
      try {
        // TMDB similar
        const tmdbSimilar = basedOn.mediaType === 'movie'
          ? await getSimilarMovies(basedOn.contentId)
          : await getSimilarTVShows(basedOn.contentId);

        for (const content of tmdbSimilar.slice(0, 8)) {
          const key = `${content.mediaType}-${content.id}`;
          if (!seen.has(key)) {
            seen.add(key);
            recommendations.push({
              content,
              score: 0.9,
              reasons: ['Similar to content you enjoyed'],
              basedOn: { type: 'similar', references: [basedOn.contentId.toString()] },
            });
          }
        }

        // Vector-based similar
        try {
          const vectorSimilar = await findSimilarContent(basedOn.contentId, basedOn.mediaType, 10);
          for (const { content, score } of vectorSimilar) {
            const key = `${content.mediaType}-${content.id}`;
            if (!seen.has(key)) {
              seen.add(key);
              recommendations.push({
                content,
                score: score * 0.85,
                reasons: ['Semantically similar'],
                basedOn: { type: 'similar', references: [basedOn.contentId.toString()] },
              });
            }
          }
        } catch {
          // Vector search not available
        }

      } catch (error) {
        console.error('Similar content search failed:', error);
      }
    }

    // Strategy 2: Genre-based discovery (if preferences provided)
    if (preferences?.genres?.length) {
      try {
        const [movieRecs, tvRecs] = await Promise.all([
          discoverMovies({ genres: preferences.genres, ratingMin: 7 }),
          discoverTVShows({ genres: preferences.genres, ratingMin: 7 }),
        ]);

        for (const content of [...movieRecs.results, ...tvRecs.results].slice(0, 10)) {
          const key = `${content.mediaType}-${content.id}`;
          if (!seen.has(key)) {
            seen.add(key);
            recommendations.push({
              content,
              score: 0.75,
              reasons: ['Matches your favorite genres'],
              basedOn: { type: 'genre', references: preferences.genres.map(String) },
            });
          }
        }
      } catch (error) {
        console.error('Genre discovery failed:', error);
      }
    }

    // Strategy 3: Fill with trending (if not enough recommendations)
    if (recommendations.length < limit) {
      try {
        const trending = await getTrending('all', 'week');
        for (const content of trending) {
          const key = `${content.mediaType}-${content.id}`;
          if (!seen.has(key) && recommendations.length < limit) {
            seen.add(key);
            recommendations.push({
              content,
              score: 0.5 - (recommendations.length * 0.01),
              reasons: ['Trending now'],
              basedOn: { type: 'trending', references: [] },
            });
          }
        }
      } catch (error) {
        console.error('Trending fetch failed:', error);
      }
    }

    // Sort by score and limit
    const sortedRecommendations = recommendations
      .sort((a, b) => b.score - a.score)
      .slice(0, limit);

    return NextResponse.json({
      success: true,
      recommendations: sortedRecommendations,
      strategies: {
        similar: sortedRecommendations.filter(r => r.basedOn?.type === 'similar').length,
        genre: sortedRecommendations.filter(r => r.basedOn?.type === 'genre').length,
        trending: sortedRecommendations.filter(r => r.basedOn?.type === 'trending').length,
      },
    });
  } catch (error) {
    console.error('Recommendations error:', error);

    if (error instanceof z.ZodError) {
      return NextResponse.json(
        { success: false, error: 'Invalid request', details: error.errors },
        { status: 400 }
      );
    }

    return NextResponse.json(
      { success: false, error: 'Failed to get recommendations' },
      { status: 500 }
    );
  }
}
