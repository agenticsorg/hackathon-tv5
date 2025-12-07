/**
 * Configuration loader
 * Loads configuration from root .env file
 */

import { config } from 'dotenv';
import { resolve } from 'path';
import { AppConfig } from '../types/index.js';

// Load from root .env file
config({ path: resolve(process.cwd(), '../../.env') });

// Also try local .env as fallback
config({ path: resolve(process.cwd(), '.env') });

export function loadConfig(): AppConfig {
  return {
    tvdb: {
      apiKey: process.env.TVDB_API_KEY || '',
      pin: process.env.TVDB_PIN,
      baseUrl: process.env.TVDB_BASE_URL || 'https://api4.thetvdb.com/v4',
      cacheTtlSeconds: parseInt(process.env.TVDB_CACHE_TTL || '3600'),
      rateLimitPerMinute: parseInt(process.env.TVDB_RATE_LIMIT || '100')
    },
    database: {
      url: process.env.DATABASE_URL || 'postgresql://localhost:5432/tvdb_recommender',
      poolSize: parseInt(process.env.DB_POOL_SIZE || '10'),
      ssl: process.env.DB_SSL === 'true'
    },
    agentdb: {
      path: process.env.AGENTDB_PATH || './data/agent-memory.db',
      backend: (process.env.AGENTDB_BACKEND as any) || 'auto',
      embeddingModel: process.env.EMBEDDING_MODEL || 'Xenova/all-MiniLM-L6-v2',
      embeddingDimension: parseInt(process.env.EMBEDDING_DIMENSION || '384')
    },
    embedding: {
      model: process.env.EMBEDDING_MODEL || 'Xenova/all-MiniLM-L6-v2',
      dimension: parseInt(process.env.EMBEDDING_DIMENSION || '384'),
      batchSize: parseInt(process.env.EMBEDDING_BATCH_SIZE || '32'),
      cacheEnabled: process.env.EMBEDDING_CACHE !== 'false'
    },
    recommendation: {
      defaultLimit: parseInt(process.env.REC_DEFAULT_LIMIT || '20'),
      maxLimit: parseInt(process.env.REC_MAX_LIMIT || '100'),
      similarityThreshold: parseFloat(process.env.REC_SIMILARITY_THRESHOLD || '0.3'),
      diversityFactor: parseFloat(process.env.REC_DIVERSITY_FACTOR || '0.2'),
      coldStartStrategy: (process.env.REC_COLD_START_STRATEGY as any) || 'trending'
    },
    learning: {
      enabled: process.env.LEARNING_ENABLED !== 'false',
      minSamplesForTraining: parseInt(process.env.LEARNING_MIN_SAMPLES || '100'),
      consolidationSchedule: process.env.LEARNING_CONSOLIDATION_SCHEDULE || '0 3 * * *',
      gnnEnabled: process.env.LEARNING_GNN_ENABLED === 'true',
      rewardDecayFactor: parseFloat(process.env.LEARNING_REWARD_DECAY || '0.95')
    }
  };
}

export function validateConfig(config: AppConfig): void {
  if (!config.tvdb.apiKey) {
    console.warn('Warning: TVDB_API_KEY not set. Set it in the root .env file.');
  }
}

export default loadConfig;
