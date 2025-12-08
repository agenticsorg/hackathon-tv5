/**
 * Cache Layer Tests
 *
 * Tests multi-tier caching functionality
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import {
  MultiTierCache,
  getIntentCache,
  getSearchResultCache,
  getStreamingCache,
  createSearchCacheKey,
  getCacheStats,
} from '@/lib/cache';
import type { SemanticSearchQuery } from '@/types/media';

describe('MultiTierCache', () => {
  let cache: MultiTierCache<string>;

  beforeEach(() => {
    cache = new MultiTierCache<string>({
      prefix: 'test',
      l1MaxSize: 10,
      l1TtlMs: 1000, // 1 second for testing
    });
  });

  afterEach(() => {
    cache.dispose();
  });

  describe('L1 Cache (In-Memory)', () => {
    it('should store and retrieve values', async () => {
      await cache.set('key1', 'value1');
      const result = await cache.get('key1');

      expect(result).toBe('value1');
    });

    it('should return null for missing keys', async () => {
      const result = await cache.get('nonexistent');

      expect(result).toBeNull();
    });

    it('should evict old entries when at capacity', async () => {
      // Fill cache to capacity
      for (let i = 0; i < 10; i++) {
        await cache.set(`key${i}`, `value${i}`);
      }

      // Add one more (should evict oldest)
      await cache.set('key10', 'value10');

      // First key should be evicted
      const first = await cache.get('key0');
      const last = await cache.get('key10');

      expect(first).toBeNull();
      expect(last).toBe('value10');
    });

    it('should respect TTL', async () => {
      await cache.set('expiring', 'value');

      // Should exist immediately
      expect(await cache.get('expiring')).toBe('value');

      // Wait for TTL to expire
      await new Promise(resolve => setTimeout(resolve, 1100));

      // Should be gone
      expect(await cache.get('expiring')).toBeNull();
    });

    it('should delete specific keys', async () => {
      await cache.set('toDelete', 'value');
      expect(await cache.get('toDelete')).toBe('value');

      await cache.delete('toDelete');
      expect(await cache.get('toDelete')).toBeNull();
    });

    it('should clear all entries', async () => {
      await cache.set('a', '1');
      await cache.set('b', '2');

      await cache.clear();

      expect(await cache.get('a')).toBeNull();
      expect(await cache.get('b')).toBeNull();
    });

    it('should provide cache stats', async () => {
      await cache.set('stat1', 'value1');
      await cache.set('stat2', 'value2');

      // Hit stat1 twice
      await cache.get('stat1');
      await cache.get('stat1');

      const stats = cache.stats();

      expect(stats.l1.size).toBe(2);
      expect(stats.l1.maxSize).toBe(10);
      expect(stats.l2Connected).toBe(false); // No Redis configured
    });
  });

  describe('Object Caching', () => {
    let objectCache: MultiTierCache<{ name: string; value: number }>;

    beforeEach(() => {
      objectCache = new MultiTierCache({
        prefix: 'obj',
        l1MaxSize: 10,
        l1TtlMs: 5000,
      });
    });

    afterEach(() => {
      objectCache.dispose();
    });

    it('should cache and retrieve objects', async () => {
      const obj = { name: 'test', value: 42 };
      await objectCache.set('obj1', obj);

      const result = await objectCache.get('obj1');

      expect(result).toEqual(obj);
    });

    it('should handle complex nested objects', async () => {
      const complex = {
        name: 'complex',
        value: 100,
        nested: { deep: { value: 'found' } },
      } as any;

      await objectCache.set('complex', complex);
      const result = await objectCache.get('complex');

      expect(result?.name).toBe('complex');
      // Note: nested properties beyond type won't be type-checked
    });
  });
});

describe('Cache Key Generation', () => {
  it('should create deterministic keys for same inputs', () => {
    const key1 = createSearchCacheKey('test query', [1, 2], { region: 'US' });
    const key2 = createSearchCacheKey('test query', [1, 2], { region: 'US' });

    expect(key1).toBe(key2);
  });

  it('should normalize query case', () => {
    const key1 = createSearchCacheKey('Test Query', [1], { region: 'US' });
    const key2 = createSearchCacheKey('test query', [1], { region: 'US' });

    expect(key1).toBe(key2);
  });

  it('should create different keys for different preferences', () => {
    const key1 = createSearchCacheKey('query', [1, 2], { region: 'US' });
    const key2 = createSearchCacheKey('query', [2, 3], { region: 'US' });

    expect(key1).not.toBe(key2);
  });

  it('should create different keys for different regions', () => {
    const key1 = createSearchCacheKey('query', [], { region: 'US' });
    const key2 = createSearchCacheKey('query', [], { region: 'GB' });

    expect(key1).not.toBe(key2);
  });

  it('should handle empty preferences', () => {
    const key = createSearchCacheKey('query', undefined);

    expect(key).toBeTruthy();
    expect(key).toContain('query');
  });
});

describe('Singleton Cache Instances', () => {
  it('should return same intent cache instance', () => {
    const cache1 = getIntentCache();
    const cache2 = getIntentCache();

    expect(cache1).toBe(cache2);
  });

  it('should return same search result cache instance', () => {
    const cache1 = getSearchResultCache();
    const cache2 = getSearchResultCache();

    expect(cache1).toBe(cache2);
  });

  it('should return same streaming cache instance', () => {
    const cache1 = getStreamingCache();
    const cache2 = getStreamingCache();

    expect(cache1).toBe(cache2);
  });

  it('should provide aggregate stats', () => {
    const stats = getCacheStats();

    expect(stats).toHaveProperty('intent');
    expect(stats).toHaveProperty('search');
    expect(stats).toHaveProperty('streaming');

    expect(stats.intent.l1).toBeDefined();
    expect(stats.search.l1).toBeDefined();
    expect(stats.streaming.l1).toBeDefined();
  });
});

describe('Intent Cache', () => {
  it('should cache SemanticSearchQuery objects', async () => {
    const intentCache = getIntentCache();

    const query: SemanticSearchQuery = {
      query: 'action movies',
      intent: {
        mood: ['exciting'],
        themes: ['adventure'],
        genres: ['action'],
      },
      filters: {
        mediaType: 'movie',
        genres: [28],
      },
    };

    await intentCache.set('action movies', query);
    const result = await intentCache.get('action movies');

    expect(result?.query).toBe('action movies');
    expect(result?.intent?.mood).toContain('exciting');
    expect(result?.filters?.genres).toContain(28);
  });
});
