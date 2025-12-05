import { Request, Response } from 'express';
import { CrawlerEngine } from '../../crawler/crawler-engine.js';
import { QueueManager } from '../../queue/queue-manager.js';
import { createChildLogger } from '../../utils/logger.js';
import { ValidationError, CrawlerError } from '../../types/index.js';
import {
  ScrapeRequestSchema,
  CrawlRequestSchema,
  BatchRequestSchema,
  MapRequestSchema
} from '../../types/index.js';

const logger = createChildLogger('crawler-controller');

export class CrawlerController {
  private crawler: CrawlerEngine;
  private queueManager: QueueManager;

  constructor(crawler: CrawlerEngine, queueManager: QueueManager) {
    this.crawler = crawler;
    this.queueManager = queueManager;
  }

  /**
   * POST /api/v1/scrape - Scrape single page
   */
  scrape = async (req: Request, res: Response): Promise<void> => {
    try {
      const validated = ScrapeRequestSchema.parse(req.body);

      logger.info({ url: validated.url }, 'Scrape request received');

      const result = await this.crawler.scrape(validated.url, validated);

      res.json({
        success: true,
        data: result
      });
    } catch (error) {
      this.handleError(error, res);
    }
  };

  /**
   * POST /api/v1/crawl - Start crawl job
   */
  crawl = async (req: Request, res: Response): Promise<void> => {
    try {
      const validated = CrawlRequestSchema.parse(req.body);

      logger.info({ url: validated.url }, 'Crawl request received');

      const jobId = await this.queueManager.addCrawlJob(validated.url, validated);

      res.json({
        success: true,
        data: {
          jobId,
          status: 'pending',
          message: 'Crawl job created successfully'
        }
      });
    } catch (error) {
      this.handleError(error, res);
    }
  };

  /**
   * GET /api/v1/crawl/:id - Get crawl job status
   */
  getCrawlStatus = async (req: Request, res: Response): Promise<void> => {
    try {
      const { id } = req.params;

      logger.info({ jobId: id }, 'Crawl status request received');

      const job = await this.queueManager.getJob(id);

      if (!job) {
        res.status(404).json({
          success: false,
          error: {
            code: 'JOB_NOT_FOUND',
            message: 'Job not found'
          }
        });
        return;
      }

      res.json({
        success: true,
        data: job
      });
    } catch (error) {
      this.handleError(error, res);
    }
  };

  /**
   * POST /api/v1/map - Generate site map
   */
  generateMap = async (req: Request, res: Response): Promise<void> => {
    try {
      const validated = MapRequestSchema.parse(req.body);

      logger.info({ url: validated.url }, 'Site map request received');

      const siteMap = await this.crawler.generateSiteMap(validated.url, {
        maxDepth: validated.maxDepth,
        respectRobotsTxt: validated.respectRobotsTxt
      });

      res.json({
        success: true,
        data: {
          url: validated.url,
          pages: siteMap,
          totalPages: siteMap.length
        }
      });
    } catch (error) {
      this.handleError(error, res);
    }
  };

  /**
   * POST /api/v1/batch - Batch scrape
   */
  batchScrape = async (req: Request, res: Response): Promise<void> => {
    try {
      const validated = BatchRequestSchema.parse(req.body);

      logger.info({ urlCount: validated.urls.length }, 'Batch scrape request received');

      const jobId = await this.queueManager.addBatchJob(
        validated.urls,
        validated.options || { url: '' }
      );

      res.json({
        success: true,
        data: {
          jobId,
          status: 'pending',
          urlCount: validated.urls.length,
          message: 'Batch job created successfully'
        }
      });
    } catch (error) {
      this.handleError(error, res);
    }
  };

  /**
   * POST /api/v1/discover - Discover ARW resources
   */
  discoverARW = async (req: Request, res: Response): Promise<void> => {
    try {
      const { url } = req.body;

      if (!url) {
        throw new ValidationError('URL is required');
      }

      logger.info({ url }, 'ARW discovery request received');

      const discovery = await this.crawler.discoverARW(url);

      res.json({
        success: true,
        data: discovery
      });
    } catch (error) {
      this.handleError(error, res);
    }
  };

  /**
   * GET /api/v1/stats - Get service statistics
   */
  getStats = async (req: Request, res: Response): Promise<void> => {
    try {
      const queueStats = await this.queueManager.getStats();
      const cacheStats = this.crawler.getCacheStats();

      res.json({
        success: true,
        data: {
          queue: queueStats,
          cache: cacheStats
        }
      });
    } catch (error) {
      this.handleError(error, res);
    }
  };

  /**
   * DELETE /api/v1/cache - Clear cache
   */
  clearCache = async (req: Request, res: Response): Promise<void> => {
    try {
      await this.crawler.clearCache();

      res.json({
        success: true,
        message: 'Cache cleared successfully'
      });
    } catch (error) {
      this.handleError(error, res);
    }
  };

  /**
   * Handle errors
   */
  private handleError(error: any, res: Response): void {
    logger.error({ error }, 'Request error');

    if (error instanceof CrawlerError) {
      res.status(error.statusCode).json({
        success: false,
        error: {
          code: error.code,
          message: error.message,
          details: error.details
        }
      });
      return;
    }

    if (error.name === 'ZodError') {
      res.status(400).json({
        success: false,
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Invalid request data',
          details: error.errors
        }
      });
      return;
    }

    res.status(500).json({
      success: false,
      error: {
        code: 'INTERNAL_ERROR',
        message: 'An unexpected error occurred'
      }
    });
  }
}
