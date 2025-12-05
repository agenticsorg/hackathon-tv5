/**
 * Unit tests for QueueManager
 * Tests queue management, concurrency control, and job scheduling
 */

import { describe, it, expect, beforeEach } from '@jest/globals';

/**
 * Mock QueueManager for testing
 */
class QueueManager {
  private queue: any[] = [];
  private processing = 0;
  private maxConcurrency: number;

  constructor(maxConcurrency: number = 5) {
    this.maxConcurrency = maxConcurrency;
  }

  async enqueue(job: any): Promise<string> {
    const id = `job-${Date.now()}-${Math.random()}`;
    this.queue.push({ id, job, status: 'queued' });
    return id;
  }

  async dequeue(): Promise<any | null> {
    if (this.queue.length === 0) return null;
    const item = this.queue.shift();
    if (item) item.status = 'processing';
    return item;
  }

  async process(): Promise<void> {
    while (this.processing < this.maxConcurrency && this.queue.length > 0) {
      const item = await this.dequeue();
      if (item) {
        this.processing++;
        // Simulate async processing
        setTimeout(() => {
          this.processing--;
        }, 100);
      }
    }
  }

  getQueueLength(): number {
    return this.queue.length;
  }

  getProcessingCount(): number {
    return this.processing;
  }
}

describe('QueueManager', () => {
  let queueManager: QueueManager;

  beforeEach(() => {
    queueManager = new QueueManager(3);
  });

  describe('Queue Operations', () => {
    it('should enqueue jobs', async () => {
      const id = await queueManager.enqueue({ url: 'https://example.com' });
      expect(id).toMatch(/job-/);
      expect(queueManager.getQueueLength()).toBe(1);
    });

    it('should dequeue jobs in FIFO order', async () => {
      await queueManager.enqueue({ order: 1 });
      await queueManager.enqueue({ order: 2 });

      const first = await queueManager.dequeue();
      expect(first.job.order).toBe(1);

      const second = await queueManager.dequeue();
      expect(second.job.order).toBe(2);
    });

    it('should return null when queue is empty', async () => {
      const item = await queueManager.dequeue();
      expect(item).toBeNull();
    });

    it('should track queue length', async () => {
      expect(queueManager.getQueueLength()).toBe(0);

      await queueManager.enqueue({ url: 'test1' });
      await queueManager.enqueue({ url: 'test2' });
      expect(queueManager.getQueueLength()).toBe(2);

      await queueManager.dequeue();
      expect(queueManager.getQueueLength()).toBe(1);
    });
  });

  describe('Concurrency Control', () => {
    it('should respect max concurrency limit', async () => {
      await queueManager.enqueue({ url: 'test1' });
      await queueManager.enqueue({ url: 'test2' });
      await queueManager.enqueue({ url: 'test3' });
      await queueManager.enqueue({ url: 'test4' });

      await queueManager.process();

      expect(queueManager.getProcessingCount()).toBeLessThanOrEqual(3);
    });

    it('should process jobs concurrently', async () => {
      const jobs = Array(10).fill(null).map((_, i) => ({ url: `test${i}` }));

      for (const job of jobs) {
        await queueManager.enqueue(job);
      }

      await queueManager.process();
      expect(queueManager.getProcessingCount()).toBeGreaterThan(0);
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty queue', async () => {
      await queueManager.process();
      expect(queueManager.getProcessingCount()).toBe(0);
    });

    it('should handle single job', async () => {
      await queueManager.enqueue({ url: 'single' });
      await queueManager.process();
      expect(queueManager.getQueueLength()).toBe(0);
    });

    it('should handle many jobs', async () => {
      for (let i = 0; i < 100; i++) {
        await queueManager.enqueue({ url: `test${i}` });
      }
      expect(queueManager.getQueueLength()).toBe(100);
    });
  });
});
