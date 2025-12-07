/**
 * MCP Server Configuration
 */

import dotenv from 'dotenv';

dotenv.config();

export const config = {
  nodeEnv: process.env.NODE_ENV || 'development',
  port: parseInt(process.env.PORT || '3000', 10),
  transport: process.env.TRANSPORT || 'stdio',

  services: {
    discovery: process.env.DISCOVERY_SERVICE_URL || 'http://localhost:4001',
    content: process.env.CONTENT_SERVICE_URL || 'http://localhost:4002',
    recommendation: process.env.RECOMMENDATION_SERVICE_URL || 'http://localhost:4003',
    user: process.env.USER_SERVICE_URL || 'http://localhost:4004',
  },

  auth: {
    jwtSecret: process.env.JWT_SECRET || 'dev-secret',
    jwtPublicKeyUrl: process.env.JWT_PUBLIC_KEY_URL || '',
  },

  rateLimit: {
    unauthenticated: parseInt(process.env.RATE_LIMIT_UNAUTHENTICATED || '10', 10),
    authenticated: parseInt(process.env.RATE_LIMIT_AUTHENTICATED || '1000', 10),
    windowMs: parseInt(process.env.RATE_LIMIT_WINDOW_MS || '900000', 10), // 15 minutes
  },

  cache: {
    ttlSeconds: parseInt(process.env.CACHE_TTL_SECONDS || '300', 10),
  },

  logging: {
    level: process.env.LOG_LEVEL || 'info',
  },

  cors: {
    allowedOrigins: (process.env.ALLOWED_ORIGINS || 'http://localhost:3000').split(','),
  },

  apiVersion: process.env.API_VERSION || '1.0.0',
};

export const isDevelopment = config.nodeEnv === 'development';
export const isProduction = config.nodeEnv === 'production';
