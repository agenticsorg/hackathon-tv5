/**
 * Analytics API
 * GET /api/analytics - Get search quality metrics
 *
 * Provides real-time and historical search analytics
 */

import { NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import {
  getSearchMetrics,
  getRealTimeMetrics,
  getSessionAnalytics,
  exportAnalytics,
} from '@/lib/analytics';

// Query params schema
const AnalyticsQuerySchema = z.object({
  period: z.enum(['hour', 'day', 'week']).optional(),
  sessionId: z.string().optional(),
  export: z.enum(['true', 'false']).optional(),
});

export async function GET(request: NextRequest) {
  try {
    const searchParams = request.nextUrl.searchParams;
    const params = AnalyticsQuerySchema.parse({
      period: searchParams.get('period') || undefined,
      sessionId: searchParams.get('sessionId') || undefined,
      export: searchParams.get('export') || undefined,
    });

    // Session-specific analytics
    if (params.sessionId) {
      const sessionData = getSessionAnalytics(params.sessionId);
      return NextResponse.json({
        success: true,
        type: 'session',
        sessionId: params.sessionId,
        data: sessionData,
      });
    }

    // Export raw data (admin only in production)
    if (params.export === 'true') {
      const endTime = new Date();
      const startTime = new Date(endTime.getTime() - 24 * 60 * 60 * 1000);
      const events = exportAnalytics(startTime, endTime);

      return NextResponse.json({
        success: true,
        type: 'export',
        period: '24h',
        eventCount: events.length,
        events: events.slice(0, 1000), // Limit export size
      });
    }

    // Real-time metrics (default)
    if (!params.period || params.period === 'hour') {
      const metrics = getRealTimeMetrics();
      return NextResponse.json({
        success: true,
        type: 'realtime',
        metrics,
      });
    }

    // Historical metrics
    const endTime = new Date();
    let startTime: Date;

    switch (params.period) {
      case 'day':
        startTime = new Date(endTime.getTime() - 24 * 60 * 60 * 1000);
        break;
      case 'week':
        startTime = new Date(endTime.getTime() - 7 * 24 * 60 * 60 * 1000);
        break;
      default:
        startTime = new Date(endTime.getTime() - 60 * 60 * 1000);
    }

    const metrics = getSearchMetrics(startTime, endTime, params.period);

    return NextResponse.json({
      success: true,
      type: 'historical',
      metrics,
    });
  } catch (error) {
    console.error('Analytics error:', error);

    if (error instanceof z.ZodError) {
      return NextResponse.json(
        { success: false, error: 'Invalid parameters', details: error.errors },
        { status: 400 }
      );
    }

    return NextResponse.json(
      { success: false, error: 'Failed to get analytics' },
      { status: 500 }
    );
  }
}
