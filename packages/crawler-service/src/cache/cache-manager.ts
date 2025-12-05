import NodeCache from 'node-cache';
import Redis from 'ioredis';
import { config } from '../utils/config.js';
import { createChildLogger } from '../utils/logger.js';

const logger = createChildLogger('cache-manager');

export interface CacheOptions {
  ttl?: number;
  useRedis?: boolean;
}

export class CacheManager {
  private memoryCache: NodeCache;
  private redisCache?: Redis;
  private useRedis: boolean;

  constructor(options: CacheOptions = {}) {
    // Initialize memory cache
    this.memoryCache = new NodeCache({
      stdTTL: options.ttl || config.cache.ttl,
      maxKeys: config.cache.maxSize,
      checkperiod: 120
    });

    // Initialize Redis cache if enabled
    this.useRedis = options.useRedis ?? false;
    if (this.useRedis) {
      this.redisCache = new Redis({
        host: config.redis.host,
        port: config.redis.port,
        password: config.redis.password,
        db: config.redis.db
      });

      this.redisCache.on('connect', () => {
        logger.info('Redis cache connected');
      });

      this.redisCache.on('error', (error) => {
        logger.error({ error }, 'Redis cache error');
        this.useRedis = false; // Fallback to memory cache
      });
    }

    logger.info({ useRedis: this.useRedis }, 'Cache manager initialized');
  }

  /**
   * Get value from cache
   */
  async get<T>(key: string): Promise<T | null> {
    try {
      // Try Redis first if enabled
      if (this.useRedis && this.redisCache) {
        const value = await this.redisCache.get(key);
        if (value) {
          logger.debug({ key }, 'Cache hit (Redis)');
          return JSON.parse(value) as T;
        }
      }

      // Fallback to memory cache
      const value = this.memoryCache.get<T>(key);
      if (value !== undefined) {
        logger.debug({ key }, 'Cache hit (memory)');
        return value;
      }

      logger.debug({ key }, 'Cache miss');
      return null;
    } catch (error) {
      logger.error({ key, error }, 'Cache get error');
      return null;
    }
  }

  /**
   * Set value in cache
   */
  async set<T>(key: string, value: T, ttl?: number): Promise<void> {
    try {
      // Set in Redis if enabled
      if (this.useRedis && this.redisCache) {
        const serialized = JSON.stringify(value);
        if (ttl) {
          await this.redisCache.setex(key, ttl, serialized);
        } else {
          await this.redisCache.set(key, serialized);
        }
      }

      // Always set in memory cache
      this.memoryCache.set(key, value, ttl || config.cache.ttl);

      logger.debug({ key, ttl }, 'Value cached');
    } catch (error) {
      logger.error({ key, error }, 'Cache set error');
    }
  }

  /**
   * Delete value from cache
   */
  async delete(key: string): Promise<void> {
    try {
      if (this.useRedis && this.redisCache) {
        await this.redisCache.del(key);
      }

      this.memoryCache.del(key);

      logger.debug({ key }, 'Value removed from cache');
    } catch (error) {
      logger.error({ key, error }, 'Cache delete error');
    }
  }

  /**
   * Check if key exists in cache
   */
  async has(key: string): Promise<boolean> {
    try {
      if (this.useRedis && this.redisCache) {
        const exists = await this.redisCache.exists(key);
        return exists === 1;
      }

      return this.memoryCache.has(key);
    } catch (error) {
      logger.error({ key, error }, 'Cache has error');
      return false;
    }
  }

  /**
   * Clear all cache
   */
  async clear(): Promise<void> {
    try {
      if (this.useRedis && this.redisCache) {
        await this.redisCache.flushdb();
      }

      this.memoryCache.flushAll();

      logger.info('Cache cleared');
    } catch (error) {
      logger.error({ error }, 'Cache clear error');
    }
  }

  /**
   * Get cache statistics
   */
  getStats(): {
    hits: number;
    misses: number;
    keys: number;
    ksize: number;
    vsize: number;
  } {
    return this.memoryCache.getStats();
  }

  /**
   * Close cache connections
   */
  async close(): Promise<void> {
    if (this.redisCache) {
      await this.redisCache.quit();
    }
    this.memoryCache.close();
    logger.info('Cache manager closed');
  }
}
