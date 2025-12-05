/**
 * Health Check API
 * GET /api/health
 *
 * Returns service health status for load balancers and monitoring
 */

import { NextResponse } from 'next/server';
import { tmdb } from '@/lib/tmdb';
import { isVectorDbAvailable, getVectorCount } from '@/lib/vector-search';

interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  version: string;
  timestamp: string;
  services: {
    name: string;
    status: 'up' | 'down' | 'degraded';
    latency_ms?: number;
    error?: string;
  }[];
  uptime_seconds: number;
}

const startTime = Date.now();

export async function GET() {
  const services: HealthStatus['services'] = [];
  let overallStatus: HealthStatus['status'] = 'healthy';

  // Check TMDB API
  try {
    const start = Date.now();
    if (tmdb) {
      await tmdb.trending.trending('movie', 'day');
      services.push({
        name: 'tmdb',
        status: 'up',
        latency_ms: Date.now() - start,
      });
    } else {
      services.push({
        name: 'tmdb',
        status: 'down',
        error: 'Not configured',
      });
      overallStatus = 'degraded';
    }
  } catch (error) {
    services.push({
      name: 'tmdb',
      status: 'down',
      error: error instanceof Error ? error.message : 'Unknown error',
    });
    overallStatus = 'degraded';
  }

  // Check RuVector (embedded vector database)
  try {
    const start = Date.now();
    const isAvailable = await isVectorDbAvailable();

    if (isAvailable) {
      const vectorCount = await getVectorCount();
      services.push({
        name: 'ruvector',
        status: 'up',
        latency_ms: Date.now() - start,
      });
    } else {
      services.push({
        name: 'ruvector',
        status: 'down',
        error: 'Database not available',
      });
      // RuVector being down is acceptable for basic functionality
    }
  } catch {
    services.push({
      name: 'ruvector',
      status: 'down',
      error: 'Not available',
    });
    // RuVector is optional, don't degrade status
  }

  // Check OpenAI API (for embeddings)
  const hasOpenAI = !!process.env.OPENAI_API_KEY;
  services.push({
    name: 'openai',
    status: hasOpenAI ? 'up' : 'down',
    error: hasOpenAI ? undefined : 'Not configured',
  });

  // Determine overall status
  const criticalDown = services.some(
    s => s.name === 'tmdb' && s.status === 'down'
  );
  if (criticalDown) {
    overallStatus = 'unhealthy';
  }

  const healthStatus: HealthStatus = {
    status: overallStatus,
    version: process.env.npm_package_version || '0.1.0',
    timestamp: new Date().toISOString(),
    services,
    uptime_seconds: Math.floor((Date.now() - startTime) / 1000),
  };

  const statusCode = overallStatus === 'unhealthy' ? 503 : 200;

  return NextResponse.json(healthStatus, { status: statusCode });
}
