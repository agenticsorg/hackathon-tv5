/**
 * Agent Ready Web Crawler SDK
 *
 * A comprehensive TypeScript SDK for interacting with the Agent Ready Web crawler service.
 *
 * @packageDocumentation
 */

// Main client
export { CrawlerClient } from './client';
export { CrawlerClient as default } from './client';

// Type definitions
export * from './types';

// Utilities
export * from './utils';

// Endpoints
export * from './endpoints';

// ARW utilities
export * from './arw';

/**
 * SDK version
 */
export const VERSION = '1.0.0';
