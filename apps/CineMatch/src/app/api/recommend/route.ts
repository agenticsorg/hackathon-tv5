import { NextRequest, NextResponse } from 'next/server';
import { recommendForUser } from '@/lib/vector-search';
import { useMatchStore } from '@/store/match-store'; // Note: we can't use hooks in API routes, we expect params in body
import type { UserPreferences } from '@/store/match-store';

export async function POST(req: NextRequest) {
    try {
        const body = await req.json();
        const { userId, preferences, totalLikes, genreWeights, page } = body;

        if (!userId || !preferences) {
            return NextResponse.json({ error: 'Missing required parameters' }, { status: 400 });
        }

        const recommendations = await recommendForUser(
            userId,
            preferences as UserPreferences,
            totalLikes || 0,
            genreWeights || {},
            page || 1
        );

        return NextResponse.json({ results: recommendations });
    } catch (error) {
        console.error('Recommendation API Error:', error);
        return NextResponse.json({ error: 'Internal Server Error' }, { status: 500 });
    }
}
