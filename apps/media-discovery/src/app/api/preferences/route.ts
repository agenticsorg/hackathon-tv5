/**
 * User Preferences API
 * GET/POST/DELETE /api/preferences
 *
 * Manages user preferences and feedback
 */

import { NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import {
  getUserPreferences,
  saveUserPreferences,
  recordWatch,
  recordFeedback,
  updateGenrePreferences,
  exportUserData,
  deleteUserData,
} from '@/lib/preferences';

// Request schemas
const RecordWatchSchema = z.object({
  userId: z.string(),
  contentId: z.number(),
  mediaType: z.enum(['movie', 'tv']),
  progress: z.number().min(0).max(1),
  rating: z.number().min(1).max(10).optional(),
});

const RecordFeedbackSchema = z.object({
  userId: z.string(),
  contentId: z.number(),
  feedback: z.enum(['like', 'dislike']),
  genres: z.array(z.number()).optional(),
});

const UpdateGenresSchema = z.object({
  userId: z.string(),
  genres: z.array(z.number()),
});

/**
 * GET - Retrieve user preferences
 */
export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const userId = searchParams.get('userId');

  if (!userId) {
    return NextResponse.json(
      { success: false, error: 'userId is required' },
      { status: 400 }
    );
  }

  try {
    const prefs = await getUserPreferences(userId);

    return NextResponse.json({
      success: true,
      preferences: {
        userId: prefs.userId,
        favoriteGenres: prefs.favoriteGenres,
        likedCount: prefs.likedContent.length,
        dislikedCount: prefs.dislikedContent.length,
        watchHistoryCount: prefs.watchHistory.length,
        updatedAt: prefs.updatedAt,
      },
    });
  } catch (error) {
    console.error('Error fetching preferences:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to fetch preferences' },
      { status: 500 }
    );
  }
}

/**
 * POST - Record user actions (watch, feedback, genre updates)
 */
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const action = body.action;

    switch (action) {
      case 'watch': {
        const data = RecordWatchSchema.parse(body);
        await recordWatch(
          data.userId,
          data.contentId,
          data.mediaType,
          data.progress,
          data.rating
        );

        // Update genre preferences based on watch behavior
        if (data.progress >= 0.5) {
          const content = {
            id: data.contentId,
            mediaType: data.mediaType,
            genreIds: body.genres || [],
            title: '',
            overview: '',
            posterPath: null,
            backdropPath: null,
            releaseDate: '',
            voteAverage: 0,
            voteCount: 0,
            popularity: 0,
          };
          await updateGenrePreferences(
            data.userId,
            content,
            data.progress >= 0.9 ? 'complete' : 'watch',
            data.rating
          );
        }

        return NextResponse.json({ success: true, action: 'watch_recorded' });
      }

      case 'feedback': {
        const data = RecordFeedbackSchema.parse(body);
        await recordFeedback(data.userId, data.contentId, data.feedback);

        // Update genre preferences based on feedback
        if (data.genres?.length) {
          const content = {
            id: data.contentId,
            mediaType: 'movie' as const,
            genreIds: data.genres,
            title: '',
            overview: '',
            posterPath: null,
            backdropPath: null,
            releaseDate: '',
            voteAverage: 0,
            voteCount: 0,
            popularity: 0,
          };
          await updateGenrePreferences(data.userId, content, data.feedback);
        }

        return NextResponse.json({ success: true, action: 'feedback_recorded' });
      }

      case 'update_genres': {
        const data = UpdateGenresSchema.parse(body);
        const prefs = await getUserPreferences(data.userId);
        prefs.favoriteGenres = data.genres;
        await saveUserPreferences(prefs);

        return NextResponse.json({ success: true, action: 'genres_updated' });
      }

      default:
        return NextResponse.json(
          { success: false, error: 'Invalid action' },
          { status: 400 }
        );
    }
  } catch (error) {
    console.error('Error processing preference action:', error);

    if (error instanceof z.ZodError) {
      return NextResponse.json(
        { success: false, error: 'Invalid request', details: error.errors },
        { status: 400 }
      );
    }

    return NextResponse.json(
      { success: false, error: 'Failed to process action' },
      { status: 500 }
    );
  }
}

/**
 * DELETE - Delete user data (GDPR compliance)
 */
export async function DELETE(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const userId = searchParams.get('userId');
  const exportData = searchParams.get('export') === 'true';

  if (!userId) {
    return NextResponse.json(
      { success: false, error: 'userId is required' },
      { status: 400 }
    );
  }

  try {
    let exportedData = null;

    if (exportData) {
      exportedData = await exportUserData(userId);
    }

    await deleteUserData(userId);

    return NextResponse.json({
      success: true,
      message: 'User data deleted',
      exportedData,
    });
  } catch (error) {
    console.error('Error deleting user data:', error);
    return NextResponse.json(
      { success: false, error: 'Failed to delete user data' },
      { status: 500 }
    );
  }
}
