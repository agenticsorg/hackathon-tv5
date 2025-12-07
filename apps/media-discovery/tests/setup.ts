/**
 * Vitest Test Setup
 *
 * Configures environment for integration tests with real APIs
 */

import { config } from 'dotenv';
import { resolve } from 'path';

// Load environment variables from .env file
config({ path: resolve(__dirname, '../.env') });

// Fallback to .env.example values for CI (will skip tests requiring real keys)
if (!process.env.NEXT_PUBLIC_TMDB_ACCESS_TOKEN) {
  console.warn('⚠️  TMDB_ACCESS_TOKEN not set. Some tests will be skipped.');
}

if (!process.env.OPENAI_API_KEY) {
  console.warn('⚠️  OPENAI_API_KEY not set. AI-dependent tests will be skipped.');
}

// Global test utilities
declare global {
  var skipIfNoTMDB: () => boolean;
  var skipIfNoOpenAI: () => boolean;
}

globalThis.skipIfNoTMDB = () => !process.env.NEXT_PUBLIC_TMDB_ACCESS_TOKEN;
globalThis.skipIfNoOpenAI = () => !process.env.OPENAI_API_KEY;

// Increase timeout for API calls
if (typeof jest !== 'undefined') {
  jest.setTimeout(30000);
}
