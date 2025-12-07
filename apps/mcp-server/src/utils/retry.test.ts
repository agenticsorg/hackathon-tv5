/**
 * Tests for Request Timeout and Retry Utility
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
  fetchWithRetry,
  setCacheEntry,
  getCacheEntry,
  isCacheFresh,
  clearCache,
  getCacheSize,
  type RetryConfig,
  type RetryError,
} from './retry';

// Mock global fetch
global.fetch = vi.fn();

describe('retry utility', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    clearCache();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  describe('fetchWithRetry', () => {
    it('should successfully fetch on first attempt', async () => {
      const mockResponse = new Response(JSON.stringify({ data: 'test' }), {
        status: 200,
      });

      (global.fetch as any).mockResolvedValueOnce(mockResponse);

      const promise = fetchWithRetry('http://test.com/api', { method: 'GET' });

      // Fast-forward timers to resolve any pending timeouts
      await vi.runAllTimersAsync();

      const response = await promise;

      expect(response.status).toBe(200);
      expect(global.fetch).toHaveBeenCalledTimes(1);
    });

    it('should timeout after configured duration', async () => {
      // Mock a slow response that takes 200ms
      (global.fetch as any).mockImplementation(
        () =>
          new Promise(resolve =>
            setTimeout(
              () =>
                resolve(
                  new Response(JSON.stringify({ data: 'test' }), { status: 200 })
                ),
              200
            )
          )
      );

      const config: RetryConfig = {
        timeout: 100,
        maxRetries: 0,
        baseDelay: 50,
      };

      const promise = fetchWithRetry('http://test.com/api', {}, config);

      // Fast-forward past the timeout
      await vi.runAllTimersAsync();

      await expect(promise).rejects.toThrow('Request timeout');
    });

    it('should retry with exponential backoff', async () => {
      const mockResponse = new Response(JSON.stringify({ data: 'test' }), {
        status: 200,
      });

      // Fail twice, succeed on third attempt
      (global.fetch as any)
        .mockRejectedValueOnce(new Error('Network error'))
        .mockRejectedValueOnce(new Error('Network error'))
        .mockResolvedValueOnce(mockResponse);

      const config: RetryConfig = {
        timeout: 1000,
        maxRetries: 2,
        baseDelay: 50,
      };

      const promise = fetchWithRetry('http://test.com/api', {}, config);

      // Fast-forward all timers
      await vi.runAllTimersAsync();

      const response = await promise;

      expect(response.status).toBe(200);
      expect(global.fetch).toHaveBeenCalledTimes(3);
    });

    it('should calculate correct exponential backoff delays', async () => {
      const delays: number[] = [];
      let resolveTimeout: (() => void) | null = null;

      (global.fetch as any).mockImplementation(
        () =>
          new Promise((resolve, reject) => {
            resolveTimeout = () => reject(new Error('Network error'));
          })
      );

      const originalSetTimeout = global.setTimeout;
      vi.spyOn(global, 'setTimeout').mockImplementation(((fn: any, delay: number) => {
        if (delay > 10) {
          // Only track retry delays (not timeout delays)
          delays.push(delay);
        }
        return originalSetTimeout(fn, 0); // Execute immediately for testing
      }) as any);

      const config: RetryConfig = {
        timeout: 100,
        maxRetries: 2,
        baseDelay: 50,
      };

      const promise = fetchWithRetry('http://test.com/api', {}, config);

      // Trigger all timeouts
      for (let i = 0; i < 3; i++) {
        if (resolveTimeout) resolveTimeout();
        await vi.runAllTimersAsync();
      }

      await expect(promise).rejects.toThrow();

      // Verify exponential backoff: 50ms, 100ms (with jitter)
      expect(delays.length).toBeGreaterThanOrEqual(2);
      expect(delays[0]).toBeGreaterThanOrEqual(50);
      expect(delays[0]).toBeLessThanOrEqual(60); // 50 + 10ms jitter
      expect(delays[1]).toBeGreaterThanOrEqual(100);
      expect(delays[1]).toBeLessThanOrEqual(110); // 100 + 10ms jitter
    });

    it('should throw MaxRetriesError after exhausting retries', async () => {
      (global.fetch as any).mockRejectedValue(new Error('Network error'));

      const config: RetryConfig = {
        timeout: 1000,
        maxRetries: 2,
        baseDelay: 50,
      };

      const promise = fetchWithRetry('http://test.com/api', {}, config);

      await vi.runAllTimersAsync();

      await expect(promise).rejects.toMatchObject({
        name: 'MaxRetriesError',
        attempts: 3,
        retryAfter: 30,
      });

      expect(global.fetch).toHaveBeenCalledTimes(3);
    });

    it('should include retry attempt count in error', async () => {
      (global.fetch as any).mockRejectedValue(new Error('Network error'));

      const config: RetryConfig = {
        timeout: 1000,
        maxRetries: 1,
        baseDelay: 50,
      };

      const promise = fetchWithRetry('http://test.com/api', {}, config);

      await vi.runAllTimersAsync();

      try {
        await promise;
        expect.fail('Should have thrown error');
      } catch (error) {
        const retryError = error as RetryError;
        expect(retryError.attempts).toBe(2);
        expect(retryError.message).toContain('after 2 attempts');
      }
    });
  });

  describe('cache fallback', () => {
    it('should cache successful GET responses', async () => {
      const testData = { data: 'test' };
      const mockResponse = new Response(JSON.stringify(testData), {
        status: 200,
      });

      (global.fetch as any).mockResolvedValueOnce(mockResponse);

      const promise = fetchWithRetry('http://test.com/api', { method: 'GET' });

      await vi.runAllTimersAsync();

      await promise;

      // Allow async cache operation to complete
      await new Promise(resolve => setTimeout(resolve, 0));

      const cached = getCacheEntry('http://test.com/api');
      expect(cached).toEqual(testData);
    });

    it('should return cached data as fallback when all retries fail', async () => {
      const testData = { data: 'cached' };
      const url = 'http://test.com/api';

      // Populate cache
      setCacheEntry(url, testData, 300);

      // Make all requests fail
      (global.fetch as any).mockRejectedValue(new Error('Network error'));

      const config: RetryConfig = {
        timeout: 1000,
        maxRetries: 2,
        baseDelay: 50,
      };

      const promise = fetchWithRetry(url, {}, config);

      await vi.runAllTimersAsync();

      const response = await promise;

      expect(response.status).toBe(200);
      expect(response.headers.get('X-Cache-Status')).toBe('STALE');

      const data = await response.json();
      expect(data).toEqual(testData);
    });

    it('should check cache freshness correctly', () => {
      const url = 'http://test.com/api';

      setCacheEntry(url, { data: 'test' }, 10); // 10 second TTL

      expect(isCacheFresh(url)).toBe(true);

      // Fast-forward 11 seconds
      vi.advanceTimersByTime(11000);

      expect(isCacheFresh(url)).toBe(false);

      // Data should still be retrievable (for stale fallback)
      expect(getCacheEntry(url)).not.toBeNull();
    });

    it('should clear cache on demand', () => {
      setCacheEntry('http://test1.com', { data: 'test1' });
      setCacheEntry('http://test2.com', { data: 'test2' });

      expect(getCacheSize()).toBe(2);

      clearCache();

      expect(getCacheSize()).toBe(0);
      expect(getCacheEntry('http://test1.com')).toBeNull();
    });
  });

  describe('timeout behavior', () => {
    it('should abort request on timeout', async () => {
      let abortSignal: AbortSignal | undefined;

      (global.fetch as any).mockImplementation((_url: string, options: RequestInit) => {
        abortSignal = options.signal;
        return new Promise(() => {}); // Never resolves
      });

      const config: RetryConfig = {
        timeout: 100,
        maxRetries: 0,
        baseDelay: 50,
      };

      const promise = fetchWithRetry('http://test.com/api', {}, config);

      // Fast-forward past timeout
      await vi.advanceTimersByTimeAsync(150);

      expect(abortSignal?.aborted).toBe(true);

      await expect(promise).rejects.toMatchObject({
        name: 'TimeoutError',
        retryAfter: 5,
      });
    });

    it('should respect custom timeout values', async () => {
      let timeoutDuration = 0;

      (global.fetch as any).mockImplementation(
        () =>
          new Promise(resolve =>
            setTimeout(() => {
              timeoutDuration = Date.now();
              resolve(new Response('OK'));
            }, 500)
          )
      );

      const config: RetryConfig = {
        timeout: 200,
        maxRetries: 0,
        baseDelay: 50,
      };

      const promise = fetchWithRetry('http://test.com/api', {}, config);

      // Should timeout before 500ms response
      await vi.runAllTimersAsync();

      await expect(promise).rejects.toThrow('Request timeout');
    });
  });

  describe('retry configuration', () => {
    it('should use default configuration when not provided', async () => {
      const mockResponse = new Response(JSON.stringify({ data: 'test' }), {
        status: 200,
      });

      (global.fetch as any).mockResolvedValueOnce(mockResponse);

      const promise = fetchWithRetry('http://test.com/api');

      await vi.runAllTimersAsync();

      const response = await promise;

      expect(response.status).toBe(200);
    });

    it('should merge partial configuration with defaults', async () => {
      const mockResponse = new Response(JSON.stringify({ data: 'test' }), {
        status: 200,
      });

      (global.fetch as any)
        .mockRejectedValueOnce(new Error('Network error'))
        .mockResolvedValueOnce(mockResponse);

      // Only override timeout, use defaults for retries
      const promise = fetchWithRetry('http://test.com/api', {}, { timeout: 200 });

      await vi.runAllTimersAsync();

      const response = await promise;

      expect(response.status).toBe(200);
      expect(global.fetch).toHaveBeenCalledTimes(2); // Default maxRetries is 2
    });
  });

  describe('error handling', () => {
    it('should preserve original error in RetryError cause', async () => {
      const originalError = new Error('Original network error');
      (global.fetch as any).mockRejectedValue(originalError);

      const config: RetryConfig = {
        timeout: 1000,
        maxRetries: 1,
        baseDelay: 50,
      };

      const promise = fetchWithRetry('http://test.com/api', {}, config);

      await vi.runAllTimersAsync();

      try {
        await promise;
        expect.fail('Should have thrown error');
      } catch (error) {
        const retryError = error as RetryError;
        expect(retryError.cause).toBe(originalError);
      }
    });

    it('should handle non-Error exceptions', async () => {
      (global.fetch as any).mockRejectedValue('String error');

      const config: RetryConfig = {
        timeout: 1000,
        maxRetries: 0,
        baseDelay: 50,
      };

      const promise = fetchWithRetry('http://test.com/api', {}, config);

      await vi.runAllTimersAsync();

      await expect(promise).rejects.toThrow();
    });
  });

  describe('structured logging', () => {
    it('should log retry attempts with structured data', async () => {
      const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      (global.fetch as any)
        .mockRejectedValueOnce(new Error('Network error'))
        .mockResolvedValueOnce(new Response('OK'));

      const config: RetryConfig = {
        timeout: 1000,
        maxRetries: 1,
        baseDelay: 50,
      };

      const promise = fetchWithRetry('http://test.com/api', {}, config);

      await vi.runAllTimersAsync();

      await promise;

      expect(consoleWarnSpy).toHaveBeenCalled();

      const logCall = consoleWarnSpy.mock.calls[0];
      expect(logCall[0]).toBe('[Retry]');

      const logData = JSON.parse(logCall[1]);
      expect(logData).toMatchObject({
        url: 'http://test.com/api',
        attempt: 1,
        maxRetries: 2,
      });

      consoleWarnSpy.mockRestore();
    });
  });
});
