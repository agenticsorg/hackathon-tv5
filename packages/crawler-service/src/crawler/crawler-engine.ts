import { ARWDiscovery } from '../discovery/arw-discovery.js';
import { ContentExtractor } from '../extractors/content-extractor.js';
import { MachineViewTransformer } from '../transformers/machine-view-transformer.js';
import { CacheManager } from '../cache/cache-manager.js';
import { RateLimiter } from '../utils/rate-limiter.js';
import { createChildLogger } from '../utils/logger.js';
import { normalizeUrl, isSameDomain, extractDomain } from '../utils/url-helpers.js';
import { config } from '../utils/config.js';
import type { CrawlOptions, ScrapeOptions, CrawlResult } from '../types/index.js';

const logger = createChildLogger('crawler-engine');

export class CrawlerEngine {
  private discovery: ARWDiscovery;
  private extractor: ContentExtractor;
  private transformer: MachineViewTransformer;
  private cache: CacheManager;
  private rateLimiter: RateLimiter;
  private visited: Set<string> = new Set();

  constructor() {
    this.discovery = new ARWDiscovery(config.crawler.userAgent);
    this.extractor = new ContentExtractor(config.crawler.userAgent);
    this.transformer = new MachineViewTransformer();
    this.cache = new CacheManager({ useRedis: true });
    this.rateLimiter = new RateLimiter({
      maxRequests: config.crawler.maxConcurrentRequests,
      windowMs: config.rateLimit.windowMs,
      delayBetweenRequests: 1000 // 1 second between requests
    });

    logger.info('Crawler engine initialized');
  }

  /**
   * Scrape a single page
   */
  async scrape(url: string, options: ScrapeOptions = { url }): Promise<CrawlResult> {
    const normalizedUrl = normalizeUrl(url);
    const domain = extractDomain(url);

    logger.info({ url: normalizedUrl }, 'Starting scrape');

    // Check cache
    const cacheKey = `scrape:${normalizedUrl}:${options.extractionMode || 'html'}`;
    const cached = await this.cache.get<CrawlResult>(cacheKey);
    if (cached) {
      logger.info({ url: normalizedUrl }, 'Returning cached result');
      return cached;
    }

    // Wait for rate limit
    await this.rateLimiter.waitForSlot(domain);

    try {
      // Extract content
      const result = await this.extractor.extract(normalizedUrl, options);

      // Transform to machine view if requested
      if (options.extractionMode === 'machine-view') {
        result.machineView = this.transformer.transform(result);
      }

      // Cache result
      await this.cache.set(cacheKey, result, config.cache.ttl);

      logger.info({ url: normalizedUrl }, 'Scrape completed successfully');

      return result;
    } catch (error) {
      logger.error({ url: normalizedUrl, error }, 'Scrape failed');
      throw error;
    }
  }

  /**
   * Crawl multiple pages starting from a URL
   */
  async crawl(url: string, options: CrawlOptions = { url }): Promise<CrawlResult[]> {
    const normalizedUrl = normalizeUrl(url);
    const domain = extractDomain(url);

    logger.info({ url: normalizedUrl, options }, 'Starting crawl');

    this.visited.clear();
    const results: CrawlResult[] = [];
    const queue: Array<{ url: string; depth: number }> = [{ url: normalizedUrl, depth: 0 }];

    const maxDepth = options.maxDepth || config.crawler.maxDepth;
    const maxPages = options.maxPages || config.crawler.maxPages;

    // Discover ARW resources
    let arwData;
    if (options.respectRobotsTxt || options.followSitemap) {
      arwData = await this.discovery.discover(normalizedUrl);
      logger.info({ url: normalizedUrl, arwData: Object.keys(arwData) }, 'ARW discovery completed');
    }

    // Add sitemap URLs if requested
    if (options.followSitemap && arwData?.sitemap) {
      for (const sitemapUrl of arwData.sitemap.urls) {
        if (isSameDomain(sitemapUrl.loc, normalizedUrl)) {
          queue.push({ url: sitemapUrl.loc, depth: 1 });
        }
      }
      logger.info({ count: queue.length }, 'Added sitemap URLs to queue');
    }

    // Process queue
    while (queue.length > 0 && results.length < maxPages) {
      const { url: currentUrl, depth } = queue.shift()!;
      const normalizedCurrentUrl = normalizeUrl(currentUrl);

      // Skip if already visited
      if (this.visited.has(normalizedCurrentUrl)) {
        continue;
      }

      // Skip if exceeds max depth
      if (depth > maxDepth) {
        continue;
      }

      // Check robots.txt if enabled
      if (options.respectRobotsTxt && arwData?.robotsTxt) {
        if (!this.discovery.isAllowed(currentUrl, arwData.robotsTxt)) {
          logger.debug({ url: currentUrl }, 'Skipping URL (disallowed by robots.txt)');
          continue;
        }
      }

      // Mark as visited
      this.visited.add(normalizedCurrentUrl);

      // Wait for rate limit
      await this.rateLimiter.waitForSlot(domain);

      try {
        // Scrape page
        const result = await this.scrape(currentUrl, {
          ...options,
          url: currentUrl
        });

        results.push(result);

        logger.info(
          { url: currentUrl, depth, totalResults: results.length },
          'Page crawled successfully'
        );

        // Add links to queue if not at max depth
        if (depth < maxDepth && result.links) {
          for (const link of result.links) {
            // Only follow links on the same domain
            if (isSameDomain(link, normalizedUrl)) {
              const normalizedLink = normalizeUrl(link);
              if (!this.visited.has(normalizedLink)) {
                queue.push({ url: link, depth: depth + 1 });
              }
            }
          }
        }
      } catch (error) {
        logger.error({ url: currentUrl, error }, 'Failed to crawl page');
      }
    }

    logger.info({ url: normalizedUrl, totalPages: results.length }, 'Crawl completed');

    return results;
  }

  /**
   * Generate site map
   */
  async generateSiteMap(
    url: string,
    options: { maxDepth?: number; respectRobotsTxt?: boolean } = {}
  ): Promise<Array<{ url: string; title?: string; depth: number }>> {
    const normalizedUrl = normalizeUrl(url);
    const domain = extractDomain(url);

    logger.info({ url: normalizedUrl }, 'Generating site map');

    const siteMap: Array<{ url: string; title?: string; depth: number }> = [];
    const visited = new Set<string>();
    const queue: Array<{ url: string; depth: number }> = [{ url: normalizedUrl, depth: 0 }];

    const maxDepth = options.maxDepth || 3;

    // Discover ARW resources
    let arwData;
    if (options.respectRobotsTxt) {
      arwData = await this.discovery.discover(normalizedUrl);
    }

    while (queue.length > 0) {
      const { url: currentUrl, depth } = queue.shift()!;
      const normalizedCurrentUrl = normalizeUrl(currentUrl);

      if (visited.has(normalizedCurrentUrl) || depth > maxDepth) {
        continue;
      }

      // Check robots.txt
      if (options.respectRobotsTxt && arwData?.robotsTxt) {
        if (!this.discovery.isAllowed(currentUrl, arwData.robotsTxt)) {
          continue;
        }
      }

      visited.add(normalizedCurrentUrl);

      // Wait for rate limit
      await this.rateLimiter.waitForSlot(domain);

      try {
        const result = await this.scrape(currentUrl, {
          url: currentUrl,
          extractionMode: 'text'
        });

        siteMap.push({
          url: currentUrl,
          title: result.title,
          depth
        });

        // Add links to queue
        if (depth < maxDepth && result.links) {
          for (const link of result.links) {
            if (isSameDomain(link, normalizedUrl)) {
              const normalizedLink = normalizeUrl(link);
              if (!visited.has(normalizedLink)) {
                queue.push({ url: link, depth: depth + 1 });
              }
            }
          }
        }
      } catch (error) {
        logger.error({ url: currentUrl, error }, 'Failed to process URL for site map');
      }
    }

    logger.info({ url: normalizedUrl, totalPages: siteMap.length }, 'Site map generated');

    return siteMap;
  }

  /**
   * Batch scrape multiple URLs
   */
  async batchScrape(
    urls: string[],
    options: ScrapeOptions = { url: '' }
  ): Promise<Record<string, CrawlResult>> {
    logger.info({ urlCount: urls.length }, 'Starting batch scrape');

    const results: Record<string, CrawlResult> = {};

    for (const url of urls) {
      try {
        const result = await this.scrape(url, { ...options, url });
        results[url] = result;
      } catch (error) {
        logger.error({ url, error }, 'Batch scrape failed for URL');
      }
    }

    logger.info({ urlCount: urls.length, successCount: Object.keys(results).length }, 'Batch scrape completed');

    return results;
  }

  /**
   * Discover ARW resources
   */
  async discoverARW(url: string) {
    return this.discovery.discover(url);
  }

  /**
   * Get cache statistics
   */
  getCacheStats() {
    return this.cache.getStats();
  }

  /**
   * Clear cache
   */
  async clearCache(): Promise<void> {
    await this.cache.clear();
  }

  /**
   * Close all connections
   */
  async close(): Promise<void> {
    await this.extractor.close();
    await this.cache.close();
    logger.info('Crawler engine closed');
  }
}
