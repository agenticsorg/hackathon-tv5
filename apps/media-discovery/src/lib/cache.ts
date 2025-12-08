/**
 * Multi-tier Caching Layer
 *
 * Provides flexible caching with:
 * - L1: In-memory cache (fastest, limited by process memory)
 * - L2: Redis cache (distributed, for production scaling)
 *
 * Falls back gracefully when Redis is unavailable
 */

import type { SemanticSearchQuery, SearchResult } from '@/types/media';

// Cache configuration
interface CacheConfig {
  l1MaxSize: number;      // Max entries in memory
  l1TtlMs: number;        // L1 TTL in milliseconds
  l2TtlSeconds: number;   // L2 TTL in seconds
  redisUrl?: string;      // Redis connection URL
  prefix: string;         // Cache key prefix
}

// Default configuration
const DEFAULT_CONFIG: CacheConfig = {
  l1MaxSize: 1000,
  l1TtlMs: 10 * 60 * 1000,     // 10 minutes
  l2TtlSeconds: 60 * 60,       // 1 hour
  prefix: 'media-discovery',
};

// Cache entry with metadata
interface CacheEntry<T> {
  value: T;
  timestamp: number;
  hits: number;
}

// L1 In-memory cache (LRU-ish with TTL)
class L1Cache<T> {
  private cache: Map<string, CacheEntry<T>> = new Map();
  private maxSize: number;
  private ttlMs: number;

  constructor(maxSize: number, ttlMs: number) {
    this.maxSize = maxSize;
    this.ttlMs = ttlMs;
  }

  get(key: string): T | null {
    const entry = this.cache.get(key);
    if (!entry) return null;

    // Check TTL
    if (Date.now() - entry.timestamp > this.ttlMs) {
      this.cache.delete(key);
      return null;
    }

    // Update hit count and move to end (pseudo-LRU)
    entry.hits++;
    this.cache.delete(key);
    this.cache.set(key, entry);

    return entry.value;
  }

  set(key: string, value: T): void {
    // Evict oldest if at capacity
    if (this.cache.size >= this.maxSize) {
      const oldestKey = this.cache.keys().next().value;
      if (oldestKey) {
        this.cache.delete(oldestKey);
      }
    }

    this.cache.set(key, {
      value,
      timestamp: Date.now(),
      hits: 0,
    });
  }

  delete(key: string): boolean {
    return this.cache.delete(key);
  }

  clear(): void {
    this.cache.clear();
  }

  size(): number {
    return this.cache.size;
  }

  // Cleanup expired entries
  cleanup(): number {
    const now = Date.now();
    let cleaned = 0;

    for (const [key, entry] of this.cache.entries()) {
      if (now - entry.timestamp > this.ttlMs) {
        this.cache.delete(key);
        cleaned++;
      }
    }

    return cleaned;
  }

  // Get cache stats
  stats(): { size: number; maxSize: number; hitRate: number } {
    let totalHits = 0;
    for (const entry of this.cache.values()) {
      totalHits += entry.hits;
    }

    return {
      size: this.cache.size,
      maxSize: this.maxSize,
      hitRate: this.cache.size > 0 ? totalHits / this.cache.size : 0,
    };
  }
}

// Redis client interface (deferred loading)
interface RedisClient {
  get(key: string): Promise<string | null>;
  set(key: string, value: string, options?: { EX?: number }): Promise<void>;
  del(key: string): Promise<void>;
  ping(): Promise<string>;
}

// Multi-tier cache implementation
export class MultiTierCache<T> {
  private l1: L1Cache<T>;
  private redisClient: RedisClient | null = null;
  private redisConnected = false;
  private config: CacheConfig;
  private cleanupInterval: NodeJS.Timeout | null = null;

  constructor(config: Partial<CacheConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.l1 = new L1Cache<T>(this.config.l1MaxSize, this.config.l1TtlMs);

    // Start periodic cleanup
    this.cleanupInterval = setInterval(() => {
      this.l1.cleanup();
    }, this.config.l1TtlMs / 2);

    // Attempt Redis connection if URL provided
    if (this.config.redisUrl) {
      this.initRedis();
    }
  }

  private async initRedis(): Promise<void> {
    if (!this.config.redisUrl) return;

    try {
      // Dynamic import to avoid requiring redis when not used
      const { createClient } = await import('redis');
      const client = createClient({ url: this.config.redisUrl });

      client.on('error', (err: Error) => {
        console.warn('Redis cache error:', err.message);
        this.redisConnected = false;
      });

      client.on('connect', () => {
        console.log('📦 Redis cache connected');
        this.redisConnected = true;
      });

      await client.connect();
      this.redisClient = client as unknown as RedisClient;
    } catch (error) {
      console.warn('Redis cache unavailable, using L1 only:', error);
      this.redisConnected = false;
    }
  }

  private buildKey(key: string): string {
    return `${this.config.prefix}:${key}`;
  }

  async get(key: string): Promise<T | null> {
    const fullKey = this.buildKey(key);

    // Try L1 first
    const l1Result = this.l1.get(fullKey);
    if (l1Result !== null) {
      return l1Result;
    }

    // Try L2 if available
    if (this.redisConnected && this.redisClient) {
      try {
        const l2Result = await this.redisClient.get(fullKey);
        if (l2Result) {
          const parsed = JSON.parse(l2Result) as T;
          // Populate L1 for next hit
          this.l1.set(fullKey, parsed);
          return parsed;
        }
      } catch (error) {
        console.warn('Redis get error:', error);
      }
    }

    return null;
  }

  async set(key: string, value: T): Promise<void> {
    const fullKey = this.buildKey(key);

    // Always set in L1
    this.l1.set(fullKey, value);

    // Set in L2 if available
    if (this.redisConnected && this.redisClient) {
      try {
        await this.redisClient.set(
          fullKey,
          JSON.stringify(value),
          { EX: this.config.l2TtlSeconds }
        );
      } catch (error) {
        console.warn('Redis set error:', error);
      }
    }
  }

  async delete(key: string): Promise<void> {
    const fullKey = this.buildKey(key);

    this.l1.delete(fullKey);

    if (this.redisConnected && this.redisClient) {
      try {
        await this.redisClient.del(fullKey);
      } catch (error) {
        console.warn('Redis delete error:', error);
      }
    }
  }

  async clear(): Promise<void> {
    this.l1.clear();
    // Note: Redis clear would require pattern matching, not implemented for safety
  }

  stats(): { l1: ReturnType<L1Cache<T>['stats']>; l2Connected: boolean } {
    return {
      l1: this.l1.stats(),
      l2Connected: this.redisConnected,
    };
  }

  dispose(): void {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
    }
  }
}

// Singleton caches for different data types
let intentCache: MultiTierCache<SemanticSearchQuery> | null = null;
let searchResultCache: MultiTierCache<SearchResult[]> | null = null;
let streamingCache: MultiTierCache<unknown> | null = null;

/**
 * Get or create the intent parsing cache
 */
export function getIntentCache(): MultiTierCache<SemanticSearchQuery> {
  if (!intentCache) {
    intentCache = new MultiTierCache<SemanticSearchQuery>({
      prefix: 'intent',
      l1MaxSize: 500,
      l1TtlMs: 15 * 60 * 1000,     // 15 minutes for intent
      l2TtlSeconds: 2 * 60 * 60,   // 2 hours in Redis
      redisUrl: process.env.REDIS_URL,
    });
  }
  return intentCache;
}

/**
 * Get or create the search results cache
 */
export function getSearchResultCache(): MultiTierCache<SearchResult[]> {
  if (!searchResultCache) {
    searchResultCache = new MultiTierCache<SearchResult[]>({
      prefix: 'search',
      l1MaxSize: 200,
      l1TtlMs: 5 * 60 * 1000,      // 5 minutes for search results
      l2TtlSeconds: 30 * 60,       // 30 minutes in Redis
      redisUrl: process.env.REDIS_URL,
    });
  }
  return searchResultCache;
}

/**
 * Get or create the streaming availability cache
 */
export function getStreamingCache(): MultiTierCache<unknown> {
  if (!streamingCache) {
    streamingCache = new MultiTierCache<unknown>({
      prefix: 'streaming',
      l1MaxSize: 1000,
      l1TtlMs: 60 * 60 * 1000,     // 1 hour for streaming data
      l2TtlSeconds: 6 * 60 * 60,   // 6 hours in Redis
      redisUrl: process.env.REDIS_URL,
    });
  }
  return streamingCache;
}

/**
 * Get all cache stats
 */
export function getCacheStats(): Record<string, ReturnType<MultiTierCache<unknown>['stats']>> {
  return {
    intent: getIntentCache().stats(),
    search: getSearchResultCache().stats(),
    streaming: getStreamingCache().stats(),
  };
}

/**
 * Create a cache key from search parameters
 */
export function createSearchCacheKey(
  query: string,
  preferences?: number[],
  options?: { region?: string; filterByService?: number[] }
): string {
  const normalized = query.toLowerCase().trim();
  const prefKey = preferences?.sort().join(',') || '';
  const region = options?.region || 'US';
  const services = options?.filterByService?.sort().join(',') || '';

  return `${normalized}|${prefKey}|${region}|${services}`;
}
