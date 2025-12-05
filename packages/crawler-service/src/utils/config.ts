import { config as dotenvConfig } from 'dotenv';
import { z } from 'zod';

// Load environment variables
dotenvConfig();

// Configuration Schema
const ConfigSchema = z.object({
  // Server
  port: z.number().default(3000),
  host: z.string().default('0.0.0.0'),
  nodeEnv: z.enum(['development', 'production', 'test']).default('development'),

  // Redis
  redis: z.object({
    host: z.string().default('localhost'),
    port: z.number().default(6379),
    password: z.string().optional(),
    db: z.number().default(0)
  }),

  // Crawler
  crawler: z.object({
    maxConcurrentRequests: z.number().default(10),
    requestTimeout: z.number().default(30000),
    userAgent: z.string().default('ARW-Crawler/1.0'),
    maxDepth: z.number().default(3),
    maxPages: z.number().default(100)
  }),

  // Rate Limiting
  rateLimit: z.object({
    windowMs: z.number().default(60000),
    maxRequests: z.number().default(100)
  }),

  // Cache
  cache: z.object({
    ttl: z.number().default(3600),
    maxSize: z.number().default(1000)
  }),

  // Puppeteer
  puppeteer: z.object({
    headless: z.boolean().default(true),
    timeout: z.number().default(30000)
  }),

  // Logging
  logging: z.object({
    level: z.enum(['fatal', 'error', 'warn', 'info', 'debug', 'trace']).default('info'),
    pretty: z.boolean().default(false)
  }),

  // API
  api: z.object({
    prefix: z.string().default('/api/v1'),
    corsOrigin: z.string().default('*')
  })
});

export type Config = z.infer<typeof ConfigSchema>;

// Load and validate configuration
function loadConfig(): Config {
  const rawConfig = {
    port: parseInt(process.env.PORT || '3000', 10),
    host: process.env.HOST || '0.0.0.0',
    nodeEnv: process.env.NODE_ENV || 'development',

    redis: {
      host: process.env.REDIS_HOST || 'localhost',
      port: parseInt(process.env.REDIS_PORT || '6379', 10),
      password: process.env.REDIS_PASSWORD,
      db: parseInt(process.env.REDIS_DB || '0', 10)
    },

    crawler: {
      maxConcurrentRequests: parseInt(process.env.MAX_CONCURRENT_REQUESTS || '10', 10),
      requestTimeout: parseInt(process.env.REQUEST_TIMEOUT || '30000', 10),
      userAgent: process.env.USER_AGENT || 'ARW-Crawler/1.0',
      maxDepth: parseInt(process.env.MAX_DEPTH || '3', 10),
      maxPages: parseInt(process.env.MAX_PAGES || '100', 10)
    },

    rateLimit: {
      windowMs: parseInt(process.env.RATE_LIMIT_WINDOW_MS || '60000', 10),
      maxRequests: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS || '100', 10)
    },

    cache: {
      ttl: parseInt(process.env.CACHE_TTL || '3600', 10),
      maxSize: parseInt(process.env.CACHE_MAX_SIZE || '1000', 10)
    },

    puppeteer: {
      headless: process.env.PUPPETEER_HEADLESS !== 'false',
      timeout: parseInt(process.env.PUPPETEER_TIMEOUT || '30000', 10)
    },

    logging: {
      level: (process.env.LOG_LEVEL || 'info') as any,
      pretty: process.env.LOG_PRETTY === 'true'
    },

    api: {
      prefix: process.env.API_PREFIX || '/api/v1',
      corsOrigin: process.env.CORS_ORIGIN || '*'
    }
  };

  return ConfigSchema.parse(rawConfig);
}

export const config = loadConfig();
