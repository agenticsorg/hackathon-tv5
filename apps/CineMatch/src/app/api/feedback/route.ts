import { NextRequest, NextResponse } from 'next/server';
import { recordFeedback } from '@/lib/vector-search';
import type { UserPreferences } from '@/store/match-store';

export async function POST(req: NextRequest) {
    try {
        const body = await req.json();
        const { userId, preferences, totalLikes, genreWeights, item, reward } = body;

        if (!userId || !preferences || !item || reward === undefined) {
            return NextResponse.json({ error: 'Missing required parameters' }, { status: 400 });
        }

        await recordFeedback(
            userId,
            preferences as UserPreferences,
            totalLikes || 0,
            genreWeights || {},
            item,
            reward
        );

        return NextResponse.json({ success: true });
    } catch (error) {
        console.error('Feedback API Error:', error);
        return NextResponse.json({ error: 'Internal Server Error' }, { status: 500 });
    }
}
