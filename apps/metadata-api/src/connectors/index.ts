/**
 * Platform Connectors Export
 *
 * Central export point for all platform-specific connectors
 */

// Export all types from the types module
export * from './types.js';

// Export base connector
export * from './base.js';

// Export platform-specific connectors
export { FASTMRSSConnector, default as DefaultFASTConnector } from './fast-mrss.js';
export { NetflixIMFConnector } from './netflix-imf.js';
export { AmazonMECConnector } from './amazon-mec.js';

// Export FAST-specific types
export type {
  LinearScheduleMetadata,
  AdBreakMetadata,
  FASTContentSpec,
  FASTMRSSItem
} from './fast-mrss.js';
