import { Router } from 'express';
import { CrawlerController } from '../controllers/crawler-controller.js';

export function createCrawlerRoutes(controller: CrawlerController): Router {
  const router = Router();

  // Scrape single page
  router.post('/scrape', controller.scrape);

  // Start crawl job
  router.post('/crawl', controller.crawl);

  // Get crawl job status
  router.get('/crawl/:id', controller.getCrawlStatus);

  // Generate site map
  router.post('/map', controller.generateMap);

  // Batch scrape
  router.post('/batch', controller.batchScrape);

  // Discover ARW resources
  router.post('/discover', controller.discoverARW);

  // Get service statistics
  router.get('/stats', controller.getStats);

  // Clear cache
  router.delete('/cache', controller.clearCache);

  // Health check
  router.get('/health', (req, res) => {
    res.json({ status: 'ok', timestamp: new Date().toISOString() });
  });

  return router;
}
