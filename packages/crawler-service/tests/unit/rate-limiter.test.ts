import { describe, it, expect, beforeEach } from 'vitest';
import { RateLimiter } from '../../src/utils/rate-limiter.js';

describe('RateLimiter', () => {
  let rateLimiter: RateLimiter;

  beforeEach(() => {
    rateLimiter = new RateLimiter({
      maxRequests: 3,
      windowMs: 1000
    });
  });

  describe('checkLimit', () => {
    it('should allow requests within limit', async () => {
      const key = 'test-key';

      expect(await rateLimiter.checkLimit(key)).toBe(true);
      expect(await rateLimiter.checkLimit(key)).toBe(true);
      expect(await rateLimiter.checkLimit(key)).toBe(true);
    });

    it('should reject requests over limit', async () => {
      const key = 'test-key';

      await rateLimiter.checkLimit(key);
      await rateLimiter.checkLimit(key);
      await rateLimiter.checkLimit(key);

      expect(await rateLimiter.checkLimit(key)).toBe(false);
    });

    it('should reset after window expires', async () => {
      const key = 'test-key';

      await rateLimiter.checkLimit(key);
      await rateLimiter.checkLimit(key);
      await rateLimiter.checkLimit(key);

      // Wait for window to expire
      await new Promise(resolve => setTimeout(resolve, 1100));

      expect(await rateLimiter.checkLimit(key)).toBe(true);
    });
  });

  describe('getRemainingRequests', () => {
    it('should return correct remaining count', async () => {
      const key = 'test-key';

      expect(rateLimiter.getRemainingRequests(key)).toBe(3);

      await rateLimiter.checkLimit(key);
      expect(rateLimiter.getRemainingRequests(key)).toBe(2);

      await rateLimiter.checkLimit(key);
      expect(rateLimiter.getRemainingRequests(key)).toBe(1);
    });
  });

  describe('clear', () => {
    it('should clear rate limit for key', async () => {
      const key = 'test-key';

      await rateLimiter.checkLimit(key);
      await rateLimiter.checkLimit(key);

      rateLimiter.clear(key);

      expect(rateLimiter.getRemainingRequests(key)).toBe(3);
    });
  });
});
