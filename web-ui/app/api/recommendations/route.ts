// Next.js API route to proxy requests to Python FastAPI backend
// This allows the frontend to call our Python agents

import { NextRequest, NextResponse } from 'next/server';

interface SearchRequest {
  query: string;
  context?: {
    viewing?: string;
    energy?: string;
    duration?: string;
  };
  filters?: Record<string, any>;
}

const API_BASE_URL = process.env.PYTHON_API_URL || 'http://localhost:8000';

export async function POST(request: NextRequest) {
  try {
    const body: SearchRequest = await request.json();

    // Validate request
    if (!body.query || body.query.trim() === '') {
      return NextResponse.json(
        { error: 'Query is required' },
        { status: 400 }
      );
    }

    // Call Python FastAPI backend
    const response = await fetch(`${API_BASE_URL}/api/recommendations`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      const errorData = await response.json();
      return NextResponse.json(
        { error: errorData.detail || 'Failed to get recommendations' },
        { status: response.status }
      );
    }

    const data = await response.json();

    return NextResponse.json(data, {
      headers: {
        'Cache-Control': 'no-store, max-age=0',
      },
    });
  } catch (error: any) {
    console.error('API route error:', error);

    // Check if Python backend is running
    if (error.code === 'ECONNREFUSED') {
      return NextResponse.json(
        {
          error: 'Python API backend is not running. Please start it with: cd api && python3 main.py',
          fallback: true,
        },
        { status: 503 }
      );
    }

    return NextResponse.json(
      { error: error.message || 'Internal server error' },
      { status: 500 }
    );
  }
}

// Health check endpoint
export async function GET() {
  try {
    const response = await fetch(`${API_BASE_URL}/health`);
    const data = await response.json();

    return NextResponse.json({
      status: 'healthy',
      python_backend: data,
      timestamp: new Date().toISOString(),
    });
  } catch (error: any) {
    return NextResponse.json(
      {
        status: 'unhealthy',
        error: 'Python backend not reachable',
        message: 'Start with: cd api && python3 main.py',
      },
      { status: 503 }
    );
  }
}
