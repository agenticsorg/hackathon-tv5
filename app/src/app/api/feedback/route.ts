import { NextRequest, NextResponse } from 'next/server';
import { recordFeedbackWithLearning } from '@/lib/ruvector-learning';

// Force dynamic rendering - don't try to pre-render during build
export const dynamic = 'force-dynamic';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { contentId, wasSuccessful, mood, searchQuery } = body;

    if (!contentId || wasSuccessful === undefined) {
      return NextResponse.json(
        { error: 'contentId and wasSuccessful required' },
        { status: 400 }
      );
    }

    // Use Q-Learning enhanced feedback recording
    const result = await recordFeedbackWithLearning(contentId, wasSuccessful, {
      mood,
      searchQuery
    });

    return NextResponse.json({
      success: true,
      message: 'Feedback recorded with Q-Learning update',
      reward: result.reward,
      patternUpdated: result.patternUpdated,
      learning: {
        algorithm: 'Q-Learning',
        explorationRate: 0.3,
        rewardDecay: 0.95
      },
      timestamp: new Date().toISOString(),
    });
  } catch (error: any) {
    console.error('Feedback error:', error);
    return NextResponse.json({ error: error.message }, { status: 500 });
  }
}
