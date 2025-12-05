/**
 * Unit tests for retry logic
 * Tests exponential backoff and retry strategies
 */

import { describe, it, expect, jest } from '@jest/globals';

class RetryHelper {
  async retry<T>(
    fn: () => Promise<T>,
    options: { maxAttempts?: number; backoff?: number } = {}
  ): Promise<T> {
    const maxAttempts = options.maxAttempts ?? 3;
    const backoff = options.backoff ?? 1000;

    let lastError: Error | undefined;

    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        return await fn();
      } catch (error) {
        lastError = error as Error;
        if (attempt < maxAttempts) {
          await new Promise(resolve =>
            setTimeout(resolve, backoff * Math.pow(2, attempt - 1))
          );
        }
      }
    }

    throw lastError;
  }
}

describe('RetryHelper', () => {
  let retryHelper: RetryHelper;

  beforeEach(() => {
    retryHelper = new RetryHelper();
  });

  describe('Retry Logic', () => {
    it('should succeed on first attempt', async () => {
      const fn = jest.fn(async () => 'success');

      const result = await retryHelper.retry(fn);

      expect(result).toBe('success');
      expect(fn).toHaveBeenCalledTimes(1);
    });

    it('should retry on failure', async () => {
      let attempts = 0;
      const fn = jest.fn(async () => {
        attempts++;
        if (attempts < 3) throw new Error('Fail');
        return 'success';
      });

      const result = await retryHelper.retry(fn);

      expect(result).toBe('success');
      expect(fn).toHaveBeenCalledTimes(3);
    });

    it('should respect maxAttempts', async () => {
      const fn = jest.fn(async () => {
        throw new Error('Always fails');
      });

      await expect(
        retryHelper.retry(fn, { maxAttempts: 2 })
      ).rejects.toThrow('Always fails');

      expect(fn).toHaveBeenCalledTimes(2);
    });

    it('should use exponential backoff', async () => {
      const fn = jest.fn(async () => {
        throw new Error('Fail');
      });

      const start = Date.now();
      await expect(
        retryHelper.retry(fn, { maxAttempts: 3, backoff: 100 })
      ).rejects.toThrow();
      const duration = Date.now() - start;

      // Should wait ~100ms, ~200ms = ~300ms total
      expect(duration).toBeGreaterThan(200);
    });
  });
});
