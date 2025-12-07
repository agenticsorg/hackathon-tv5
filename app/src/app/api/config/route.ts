import { NextRequest, NextResponse } from 'next/server';
import { Pool } from 'pg';

const pool = new Pool({
  connectionString: process.env.DATABASE_URL ||
    'postgresql://tvdb:tvdb_recommender_2024@localhost:5432/tvdb_recommender',
  max: 5,
});

export interface SystemConfig {
  qLearning: {
    enabled: boolean;
    explorationRate: number;
    learningRate: number;
    rewardDecay: number;
    minExploration: number;
    explorationDecay: number;
  };
  vectorSearch: {
    hyperbolicEnabled: boolean;
    dimensions: number;
    distanceMetric: string;
    curvature: number;
    indexType: string;
  };
  recommendations: {
    diversityWeight: number;
    recencyBoost: number;
    popularityWeight: number;
    personalizedWeight: number;
    maxResults: number;
  };
  performance: {
    batchSize: number;
    cacheEnabled: boolean;
    parallelQueries: number;
    precomputeEmbeddings: boolean;
  };
}

const defaultConfig: SystemConfig = {
  qLearning: {
    enabled: true,
    explorationRate: 0.3,
    learningRate: 0.1,
    rewardDecay: 0.95,
    minExploration: 0.05,
    explorationDecay: 0.995,
  },
  vectorSearch: {
    hyperbolicEnabled: true,
    dimensions: 384,
    distanceMetric: 'cosine',
    curvature: -1.0,
    indexType: 'hnsw',
  },
  recommendations: {
    diversityWeight: 0.3,
    recencyBoost: 0.2,
    popularityWeight: 0.15,
    personalizedWeight: 0.7,
    maxResults: 20,
  },
  performance: {
    batchSize: 100,
    cacheEnabled: true,
    parallelQueries: 4,
    precomputeEmbeddings: true,
  },
};

// GET - Retrieve current config
export async function GET() {
  const client = await pool.connect();

  try {
    const result = await client.query(
      'SELECT config, updated_at FROM system_config WHERE id = 1'
    );

    if (result.rows.length === 0) {
      return NextResponse.json({
        config: defaultConfig,
        updated_at: new Date().toISOString(),
        source: 'default'
      });
    }

    return NextResponse.json({
      config: result.rows[0].config,
      updated_at: result.rows[0].updated_at,
      source: 'database'
    });
  } catch (error) {
    console.error('Failed to fetch config:', error);
    return NextResponse.json({
      config: defaultConfig,
      updated_at: new Date().toISOString(),
      source: 'fallback',
      error: 'Database error'
    });
  } finally {
    client.release();
  }
}

// POST - Update config
export async function POST(request: NextRequest) {
  const client = await pool.connect();

  try {
    const body = await request.json();
    const newConfig = body.config as SystemConfig;

    // Validate config structure
    if (!newConfig || !newConfig.qLearning || !newConfig.vectorSearch ||
        !newConfig.recommendations || !newConfig.performance) {
      return NextResponse.json(
        { error: 'Invalid config structure' },
        { status: 400 }
      );
    }

    // Validate numeric ranges
    const validations = [
      { value: newConfig.qLearning.explorationRate, min: 0, max: 1, name: 'explorationRate' },
      { value: newConfig.qLearning.learningRate, min: 0, max: 1, name: 'learningRate' },
      { value: newConfig.qLearning.rewardDecay, min: 0, max: 1, name: 'rewardDecay' },
      { value: newConfig.recommendations.diversityWeight, min: 0, max: 1, name: 'diversityWeight' },
      { value: newConfig.recommendations.recencyBoost, min: 0, max: 1, name: 'recencyBoost' },
      { value: newConfig.recommendations.maxResults, min: 1, max: 100, name: 'maxResults' },
    ];

    for (const v of validations) {
      if (v.value < v.min || v.value > v.max) {
        return NextResponse.json(
          { error: `${v.name} must be between ${v.min} and ${v.max}` },
          { status: 400 }
        );
      }
    }

    // Upsert config
    await client.query(`
      INSERT INTO system_config (id, config, updated_at)
      VALUES (1, $1, NOW())
      ON CONFLICT (id) DO UPDATE SET
        config = $1,
        updated_at = NOW()
    `, [JSON.stringify(newConfig)]);

    return NextResponse.json({
      success: true,
      config: newConfig,
      updated_at: new Date().toISOString()
    });
  } catch (error) {
    console.error('Failed to save config:', error);
    return NextResponse.json(
      { error: 'Failed to save configuration' },
      { status: 500 }
    );
  } finally {
    client.release();
  }
}

