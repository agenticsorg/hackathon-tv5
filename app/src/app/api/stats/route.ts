import { NextResponse } from 'next/server';
import { getLearningStats as getDbStats } from '@/lib/db';
import { getLearningStats as getRuVectorStats } from '@/lib/ruvector-learning';

// Force dynamic rendering - don't try to pre-render during build
export const dynamic = 'force-dynamic';

export async function GET() {
  try {
    const [dbStats, ruVectorStats] = await Promise.all([
      getDbStats(),
      getRuVectorStats()
    ]);

    return NextResponse.json({
      stats: dbStats,
      learning: {
        qLearning: {
          enabled: true,
          algorithm: 'Q-Learning with ε-greedy exploration',
          explorationRate: 0.3,
          rewardDecay: 0.95,
          learningRate: 0.1
        },
        patterns: {
          total: ruVectorStats.totalPatterns,
          avgSuccessRate: ruVectorStats.avgSuccessRate,
          topPerforming: ruVectorStats.bestPatterns
        },
        recentActivity: ruVectorStats.recentLearning,
        vectorSpace: {
          hyperbolicEnabled: ruVectorStats.vectorStats.hyperbolicEnabled,
          model: 'poincare_ball',
          avgSearchLatency: `${ruVectorStats.vectorStats.avgSearchTime}ms`
        }
      },
      timestamp: new Date().toISOString(),
    });
  } catch (error: any) {
    console.error('Stats error:', error);
    return NextResponse.json({ error: error.message }, { status: 500 });
  }
}
