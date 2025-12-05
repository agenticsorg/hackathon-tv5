import { CrawlerService } from './app.js';
import { logger } from './utils/logger.js';

// Export main classes for programmatic use
export { CrawlerService } from './app.js';
export { CrawlerEngine } from './crawler/crawler-engine.js';
export { ARWDiscovery } from './discovery/arw-discovery.js';
export { ContentExtractor } from './extractors/content-extractor.js';
export { MachineViewTransformer } from './transformers/machine-view-transformer.js';
export { QueueManager } from './queue/queue-manager.js';
export { CacheManager } from './cache/cache-manager.js';
export { RateLimiter } from './utils/rate-limiter.js';
export * from './types/index.js';

// Start server if running as main module
if (import.meta.url === `file://${process.argv[1]}`) {
  const service = new CrawlerService();

  // Handle graceful shutdown
  process.on('SIGTERM', async () => {
    logger.info('SIGTERM received, shutting down gracefully');
    await service.stop();
    process.exit(0);
  });

  process.on('SIGINT', async () => {
    logger.info('SIGINT received, shutting down gracefully');
    await service.stop();
    process.exit(0);
  });

  // Handle uncaught errors
  process.on('uncaughtException', (error) => {
    logger.fatal({ error }, 'Uncaught exception');
    process.exit(1);
  });

  process.on('unhandledRejection', (reason, promise) => {
    logger.fatal({ reason, promise }, 'Unhandled rejection');
    process.exit(1);
  });

  // Start service
  service.start().catch((error) => {
    logger.fatal({ error }, 'Failed to start service');
    process.exit(1);
  });
}
