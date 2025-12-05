import { createChildLogger } from './logger.js';

const logger = createChildLogger('rate-limiter');

export interface RateLimiterOptions {
  maxRequests: number;
  windowMs: number;
  delayBetweenRequests?: number;
}

export class RateLimiter {
  private requests: Map<string, number[]> = new Map();
  private maxRequests: number;
  private windowMs: number;
  private delayBetweenRequests: number;

  constructor(options: RateLimiterOptions) {
    this.maxRequests = options.maxRequests;
    this.windowMs = options.windowMs;
    this.delayBetweenRequests = options.delayBetweenRequests || 0;
  }

  /**
   * Check if request is allowed
   */
  async checkLimit(key: string): Promise<boolean> {
    const now = Date.now();
    const windowStart = now - this.windowMs;

    // Get existing requests for this key
    let timestamps = this.requests.get(key) || [];

    // Remove expired timestamps
    timestamps = timestamps.filter(ts => ts > windowStart);

    // Check if limit exceeded
    if (timestamps.length >= this.maxRequests) {
      logger.debug({ key, count: timestamps.length }, 'Rate limit exceeded');
      return false;
    }

    // Add new timestamp
    timestamps.push(now);
    this.requests.set(key, timestamps);

    return true;
  }

  /**
   * Wait until request is allowed
   */
  async waitForSlot(key: string): Promise<void> {
    while (!(await this.checkLimit(key))) {
      // Calculate wait time
      const timestamps = this.requests.get(key) || [];
      const oldestTimestamp = Math.min(...timestamps);
      const waitTime = oldestTimestamp + this.windowMs - Date.now();

      if (waitTime > 0) {
        logger.debug({ key, waitTime }, 'Waiting for rate limit slot');
        await this.sleep(waitTime);
      }
    }

    // Add delay between requests if configured
    if (this.delayBetweenRequests > 0) {
      await this.sleep(this.delayBetweenRequests);
    }
  }

  /**
   * Get remaining requests for key
   */
  getRemainingRequests(key: string): number {
    const now = Date.now();
    const windowStart = now - this.windowMs;

    let timestamps = this.requests.get(key) || [];
    timestamps = timestamps.filter(ts => ts > windowStart);

    return Math.max(0, this.maxRequests - timestamps.length);
  }

  /**
   * Get time until next available slot
   */
  getTimeUntilReset(key: string): number {
    const timestamps = this.requests.get(key) || [];
    if (timestamps.length === 0) {
      return 0;
    }

    const oldestTimestamp = Math.min(...timestamps);
    const resetTime = oldestTimestamp + this.windowMs;

    return Math.max(0, resetTime - Date.now());
  }

  /**
   * Clear rate limit data for key
   */
  clear(key: string): void {
    this.requests.delete(key);
  }

  /**
   * Clear all rate limit data
   */
  clearAll(): void {
    this.requests.clear();
  }

  /**
   * Sleep helper
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Cleanup old entries periodically
   */
  startCleanup(intervalMs: number = 60000): NodeJS.Timer {
    return setInterval(() => {
      const now = Date.now();
      const windowStart = now - this.windowMs;

      for (const [key, timestamps] of this.requests.entries()) {
        const filtered = timestamps.filter(ts => ts > windowStart);
        if (filtered.length === 0) {
          this.requests.delete(key);
        } else {
          this.requests.set(key, filtered);
        }
      }

      logger.debug({ keysCount: this.requests.size }, 'Rate limiter cleanup completed');
    }, intervalMs);
  }
}
