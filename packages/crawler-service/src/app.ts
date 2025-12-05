import express, { Express } from 'express';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import { CrawlerEngine } from './crawler/crawler-engine.js';
import { QueueManager } from './queue/queue-manager.js';
import { CrawlerController } from './api/controllers/crawler-controller.js';
import { createCrawlerRoutes } from './api/routes/crawler-routes.js';
import { errorHandler } from './api/middleware/error-handler.js';
import { requestLogger } from './api/middleware/request-logger.js';
import { rateLimitMiddleware } from './api/middleware/rate-limit.js';
import { config } from './utils/config.js';
import { createChildLogger } from './utils/logger.js';

const logger = createChildLogger('app');

export class CrawlerService {
  private app: Express;
  private crawler: CrawlerEngine;
  private queueManager: QueueManager;

  constructor() {
    this.app = express();
    this.crawler = new CrawlerEngine();
    this.queueManager = new QueueManager();

    this.setupMiddleware();
    this.setupRoutes();
    this.setupWorkers();
    this.setupErrorHandling();
  }

  /**
   * Setup middleware
   */
  private setupMiddleware(): void {
    // Security
    this.app.use(helmet());
    this.app.use(cors({ origin: config.api.corsOrigin }));

    // Performance
    this.app.use(compression());

    // Body parsing
    this.app.use(express.json({ limit: '10mb' }));
    this.app.use(express.urlencoded({ extended: true, limit: '10mb' }));

    // Logging
    this.app.use(requestLogger);

    // Rate limiting
    this.app.use(rateLimitMiddleware);

    logger.info('Middleware configured');
  }

  /**
   * Setup routes
   */
  private setupRoutes(): void {
    const controller = new CrawlerController(this.crawler, this.queueManager);
    const routes = createCrawlerRoutes(controller);

    // API routes
    this.app.use(config.api.prefix, routes);

    // Root endpoint
    this.app.get('/', (req, res) => {
      res.json({
        name: '@agent-ready-web/crawler-service',
        version: '0.1.0',
        description: 'Production-ready web crawler API service with ARW discovery',
        endpoints: {
          scrape: `POST ${config.api.prefix}/scrape`,
          crawl: `POST ${config.api.prefix}/crawl`,
          crawlStatus: `GET ${config.api.prefix}/crawl/:id`,
          map: `POST ${config.api.prefix}/map`,
          batch: `POST ${config.api.prefix}/batch`,
          discover: `POST ${config.api.prefix}/discover`,
          stats: `GET ${config.api.prefix}/stats`,
          health: `GET ${config.api.prefix}/health`
        }
      });
    });

    logger.info({ prefix: config.api.prefix }, 'Routes configured');
  }

  /**
   * Setup queue workers
   */
  private setupWorkers(): void {
    // Crawl worker
    this.queueManager.startCrawlWorker(async (job) => {
      const { id, url, options } = job.data;

      try {
        logger.info({ jobId: id, url }, 'Processing crawl job');

        const jobData = await this.queueManager.getJob(id);
        if (jobData) {
          jobData.status = 'processing';
        }

        const results = await this.crawler.crawl(url, options);

        this.queueManager.completeJob(id, results);

        logger.info({ jobId: id, resultCount: results.length }, 'Crawl job completed');
      } catch (error: any) {
        logger.error({ jobId: id, error }, 'Crawl job failed');
        this.queueManager.failJob(id, error.message);
        throw error;
      }
    });

    // Batch worker
    this.queueManager.startBatchWorker(async (job) => {
      const { id, urls, options } = job.data;

      try {
        logger.info({ jobId: id, urlCount: urls.length }, 'Processing batch job');

        const jobData = await this.queueManager.getJob(id);
        if (jobData) {
          jobData.status = 'processing';
        }

        const results = await this.crawler.batchScrape(urls, options);

        this.queueManager.completeJob(id, results);

        logger.info({ jobId: id, successCount: Object.keys(results).length }, 'Batch job completed');
      } catch (error: any) {
        logger.error({ jobId: id, error }, 'Batch job failed');
        this.queueManager.failJob(id, error.message);
        throw error;
      }
    });

    logger.info('Workers started');
  }

  /**
   * Setup error handling
   */
  private setupErrorHandling(): void {
    // 404 handler
    this.app.use((req, res) => {
      res.status(404).json({
        success: false,
        error: {
          code: 'NOT_FOUND',
          message: 'Endpoint not found'
        }
      });
    });

    // Error handler
    this.app.use(errorHandler);

    logger.info('Error handling configured');
  }

  /**
   * Start server
   */
  async start(): Promise<void> {
    return new Promise((resolve) => {
      this.app.listen(config.port, config.host, () => {
        logger.info(
          {
            host: config.host,
            port: config.port,
            nodeEnv: config.nodeEnv,
            apiPrefix: config.api.prefix
          },
          'Server started'
        );
        resolve();
      });
    });
  }

  /**
   * Stop server
   */
  async stop(): Promise<void> {
    await this.crawler.close();
    await this.queueManager.close();
    logger.info('Server stopped');
  }

  /**
   * Get Express app
   */
  getApp(): Express {
    return this.app;
  }
}
