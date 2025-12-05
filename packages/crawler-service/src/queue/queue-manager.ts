import { Queue, Worker, Job, QueueEvents } from 'bullmq';
import Redis from 'ioredis';
import { config } from '../utils/config.js';
import { createChildLogger } from '../utils/logger.js';
import type { CrawlOptions, CrawlJob, BatchJob, CrawlResult } from '../types/index.js';

const logger = createChildLogger('queue-manager');

export interface JobData {
  id: string;
  url: string;
  options: CrawlOptions;
}

export interface BatchJobData {
  id: string;
  urls: string[];
  options: CrawlOptions;
}

export class QueueManager {
  private connection: Redis;
  private crawlQueue: Queue;
  private batchQueue: Queue;
  private crawlWorker?: Worker;
  private batchWorker?: Worker;
  private queueEvents: QueueEvents;
  private jobs: Map<string, CrawlJob | BatchJob> = new Map();

  constructor() {
    // Create Redis connection
    this.connection = new Redis({
      host: config.redis.host,
      port: config.redis.port,
      password: config.redis.password,
      db: config.redis.db,
      maxRetriesPerRequest: null
    });

    // Create queues
    this.crawlQueue = new Queue('crawl', { connection: this.connection });
    this.batchQueue = new Queue('batch', { connection: this.connection });

    // Create queue events
    this.queueEvents = new QueueEvents('crawl', { connection: this.connection });

    this.setupEventListeners();

    logger.info('Queue manager initialized');
  }

  /**
   * Setup event listeners
   */
  private setupEventListeners(): void {
    this.queueEvents.on('completed', ({ jobId }) => {
      logger.info({ jobId }, 'Job completed');
    });

    this.queueEvents.on('failed', ({ jobId, failedReason }) => {
      logger.error({ jobId, failedReason }, 'Job failed');
    });

    this.queueEvents.on('progress', ({ jobId, data }) => {
      logger.debug({ jobId, progress: data }, 'Job progress');
    });
  }

  /**
   * Add crawl job to queue
   */
  async addCrawlJob(url: string, options: CrawlOptions = { url }): Promise<string> {
    const jobId = `crawl-${Date.now()}-${Math.random().toString(36).substring(7)}`;

    const job: CrawlJob = {
      id: jobId,
      url,
      options,
      status: 'pending',
      progress: 0,
      createdAt: new Date(),
      updatedAt: new Date()
    };

    this.jobs.set(jobId, job);

    await this.crawlQueue.add(
      'crawl',
      { id: jobId, url, options } as JobData,
      {
        jobId,
        attempts: 3,
        backoff: {
          type: 'exponential',
          delay: 2000
        }
      }
    );

    logger.info({ jobId, url }, 'Crawl job added to queue');

    return jobId;
  }

  /**
   * Add batch job to queue
   */
  async addBatchJob(urls: string[], options: CrawlOptions): Promise<string> {
    const jobId = `batch-${Date.now()}-${Math.random().toString(36).substring(7)}`;

    const job: BatchJob = {
      id: jobId,
      urls,
      options,
      status: 'pending',
      progress: 0,
      results: {},
      errors: {},
      createdAt: new Date(),
      updatedAt: new Date()
    };

    this.jobs.set(jobId, job);

    await this.batchQueue.add(
      'batch',
      { id: jobId, urls, options } as BatchJobData,
      {
        jobId,
        attempts: 3,
        backoff: {
          type: 'exponential',
          delay: 2000
        }
      }
    );

    logger.info({ jobId, urlCount: urls.length }, 'Batch job added to queue');

    return jobId;
  }

  /**
   * Get job status
   */
  async getJob(jobId: string): Promise<CrawlJob | BatchJob | null> {
    return this.jobs.get(jobId) || null;
  }

  /**
   * Update job progress
   */
  updateJobProgress(jobId: string, progress: number): void {
    const job = this.jobs.get(jobId);
    if (job) {
      job.progress = progress;
      job.updatedAt = new Date();
    }
  }

  /**
   * Complete job
   */
  completeJob(jobId: string, results: CrawlResult[] | Record<string, CrawlResult>): void {
    const job = this.jobs.get(jobId);
    if (job) {
      job.status = 'completed';
      job.progress = 100;
      job.completedAt = new Date();
      job.updatedAt = new Date();

      if ('results' in job && Array.isArray(results)) {
        job.results = results as CrawlResult[];
      } else if ('results' in job && typeof results === 'object') {
        job.results = results as Record<string, CrawlResult>;
      }
    }
  }

  /**
   * Fail job
   */
  failJob(jobId: string, error: string): void {
    const job = this.jobs.get(jobId);
    if (job) {
      job.status = 'failed';
      job.error = error;
      job.completedAt = new Date();
      job.updatedAt = new Date();
    }
  }

  /**
   * Start crawl worker
   */
  startCrawlWorker(processor: (job: Job<JobData>) => Promise<void>): void {
    this.crawlWorker = new Worker('crawl', processor, {
      connection: this.connection,
      concurrency: config.crawler.maxConcurrentRequests
    });

    this.crawlWorker.on('completed', (job) => {
      logger.info({ jobId: job.id }, 'Crawl worker completed job');
    });

    this.crawlWorker.on('failed', (job, err) => {
      logger.error({ jobId: job?.id, error: err }, 'Crawl worker failed job');
    });

    logger.info('Crawl worker started');
  }

  /**
   * Start batch worker
   */
  startBatchWorker(processor: (job: Job<BatchJobData>) => Promise<void>): void {
    this.batchWorker = new Worker('batch', processor, {
      connection: this.connection,
      concurrency: 1 // Process one batch at a time
    });

    this.batchWorker.on('completed', (job) => {
      logger.info({ jobId: job.id }, 'Batch worker completed job');
    });

    this.batchWorker.on('failed', (job, err) => {
      logger.error({ jobId: job?.id, error: err }, 'Batch worker failed job');
    });

    logger.info('Batch worker started');
  }

  /**
   * Get queue statistics
   */
  async getStats(): Promise<{
    crawl: { waiting: number; active: number; completed: number; failed: number };
    batch: { waiting: number; active: number; completed: number; failed: number };
  }> {
    const [crawlWaiting, crawlActive, crawlCompleted, crawlFailed] = await Promise.all([
      this.crawlQueue.getWaitingCount(),
      this.crawlQueue.getActiveCount(),
      this.crawlQueue.getCompletedCount(),
      this.crawlQueue.getFailedCount()
    ]);

    const [batchWaiting, batchActive, batchCompleted, batchFailed] = await Promise.all([
      this.batchQueue.getWaitingCount(),
      this.batchQueue.getActiveCount(),
      this.batchQueue.getCompletedCount(),
      this.batchQueue.getFailedCount()
    ]);

    return {
      crawl: {
        waiting: crawlWaiting,
        active: crawlActive,
        completed: crawlCompleted,
        failed: crawlFailed
      },
      batch: {
        waiting: batchWaiting,
        active: batchActive,
        completed: batchCompleted,
        failed: batchFailed
      }
    };
  }

  /**
   * Close queue connections
   */
  async close(): Promise<void> {
    if (this.crawlWorker) {
      await this.crawlWorker.close();
    }
    if (this.batchWorker) {
      await this.batchWorker.close();
    }
    await this.crawlQueue.close();
    await this.batchQueue.close();
    await this.queueEvents.close();
    await this.connection.quit();

    logger.info('Queue manager closed');
  }
}
