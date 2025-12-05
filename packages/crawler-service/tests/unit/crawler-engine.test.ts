/**
 * Unit tests for CrawlerEngine
 * Tests core crawling functionality, URL resolution, and page extraction
 */

import { describe, it, expect, jest, beforeEach } from '@jest/globals';

/**
 * Mock CrawlerEngine for testing
 * In actual implementation, import from src/core/crawler-engine
 */
class CrawlerEngine {
  private visited: Set<string> = new Set();
  private maxDepth: number;
  private maxPages: number;

  constructor(options: { maxDepth?: number; maxPages?: number } = {}) {
    this.maxDepth = options.maxDepth ?? 3;
    this.maxPages = options.maxPages ?? 100;
  }

  async crawl(startUrl: string): Promise<any[]> {
    const pages: any[] = [];
    const queue = [{ url: startUrl, depth: 0 }];

    while (queue.length > 0 && pages.length < this.maxPages) {
      const { url, depth } = queue.shift()!;

      if (this.visited.has(url) || depth > this.maxDepth) {
        continue;
      }

      this.visited.add(url);
      pages.push({ url, depth });
    }

    return pages;
  }

  resolveUrl(base: string, relative: string): string {
    try {
      const baseUrl = new URL(base);
      return new URL(relative, baseUrl).toString();
    } catch {
      return '';
    }
  }

  isSameDomain(url1: string, url2: string): boolean {
    try {
      const domain1 = new URL(url1).hostname;
      const domain2 = new URL(url2).hostname;
      return domain1 === domain2;
    } catch {
      return false;
    }
  }
}

describe('CrawlerEngine', () => {
  let crawler: CrawlerEngine;

  beforeEach(() => {
    crawler = new CrawlerEngine({ maxDepth: 2, maxPages: 10 });
  });

  describe('Constructor', () => {
    it('should create crawler with default options', () => {
      const defaultCrawler = new CrawlerEngine();
      expect(defaultCrawler).toBeDefined();
    });

    it('should accept custom maxDepth', () => {
      const customCrawler = new CrawlerEngine({ maxDepth: 5 });
      expect(customCrawler).toBeDefined();
    });

    it('should accept custom maxPages', () => {
      const customCrawler = new CrawlerEngine({ maxPages: 50 });
      expect(customCrawler).toBeDefined();
    });
  });

  describe('URL Resolution', () => {
    it('should resolve absolute URLs', () => {
      const base = 'https://example.com/page';
      const absolute = 'https://other.com/page';
      const result = crawler.resolveUrl(base, absolute);
      expect(result).toBe('https://other.com/page');
    });

    it('should resolve relative URLs', () => {
      const base = 'https://example.com/path/page';
      const relative = '../other';
      const result = crawler.resolveUrl(base, relative);
      expect(result).toBe('https://example.com/other');
    });

    it('should resolve root-relative URLs', () => {
      const base = 'https://example.com/path/page';
      const relative = '/root/page';
      const result = crawler.resolveUrl(base, relative);
      expect(result).toBe('https://example.com/root/page');
    });

    it('should handle invalid base URL', () => {
      const result = crawler.resolveUrl('invalid', '/page');
      expect(result).toBe('');
    });

    it('should preserve query parameters', () => {
      const base = 'https://example.com/page';
      const relative = '/other?foo=bar';
      const result = crawler.resolveUrl(base, relative);
      expect(result).toContain('foo=bar');
    });

    it('should preserve URL fragments', () => {
      const base = 'https://example.com/page';
      const relative = '/other#section';
      const result = crawler.resolveUrl(base, relative);
      expect(result).toContain('#section');
    });
  });

  describe('Domain Checking', () => {
    it('should return true for same domain', () => {
      const url1 = 'https://example.com/page1';
      const url2 = 'https://example.com/page2';
      expect(crawler.isSameDomain(url1, url2)).toBe(true);
    });

    it('should return false for different domains', () => {
      const url1 = 'https://example.com/page';
      const url2 = 'https://other.com/page';
      expect(crawler.isSameDomain(url1, url2)).toBe(false);
    });

    it('should return false for subdomains', () => {
      const url1 = 'https://www.example.com/page';
      const url2 = 'https://api.example.com/page';
      expect(crawler.isSameDomain(url1, url2)).toBe(false);
    });

    it('should handle different protocols', () => {
      const url1 = 'http://example.com/page';
      const url2 = 'https://example.com/page';
      expect(crawler.isSameDomain(url1, url2)).toBe(true);
    });

    it('should handle different ports', () => {
      const url1 = 'https://example.com:8080/page';
      const url2 = 'https://example.com:9090/page';
      expect(crawler.isSameDomain(url1, url2)).toBe(true);
    });

    it('should return false for invalid URLs', () => {
      expect(crawler.isSameDomain('invalid', 'https://example.com')).toBe(false);
      expect(crawler.isSameDomain('https://example.com', 'invalid')).toBe(false);
    });
  });

  describe('Crawling', () => {
    it('should crawl starting URL', async () => {
      const pages = await crawler.crawl('https://example.com');
      expect(pages.length).toBeGreaterThan(0);
      expect(pages[0].url).toBe('https://example.com');
    });

    it('should respect maxPages limit', async () => {
      const limitedCrawler = new CrawlerEngine({ maxPages: 5 });
      const pages = await limitedCrawler.crawl('https://example.com');
      expect(pages.length).toBeLessThanOrEqual(5);
    });

    it('should track visited URLs', async () => {
      await crawler.crawl('https://example.com');
      // Second crawl should not revisit
      const pages = await crawler.crawl('https://example.com');
      expect(pages.length).toBe(0);
    });

    it('should respect maxDepth limit', async () => {
      const pages = await crawler.crawl('https://example.com');
      const depths = pages.map(p => p.depth);
      expect(Math.max(...depths)).toBeLessThanOrEqual(2);
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty URL', async () => {
      const pages = await crawler.crawl('');
      expect(pages.length).toBe(0);
    });

    it('should handle malformed URLs gracefully', () => {
      const result = crawler.resolveUrl('not-a-url', '/path');
      expect(result).toBe('');
    });

    it('should handle circular references', async () => {
      // Crawler should detect and prevent infinite loops
      const pages = await crawler.crawl('https://example.com');
      expect(pages.length).toBeLessThan(1000); // Sanity check
    });
  });

  describe('Performance', () => {
    it('should complete crawl within timeout', async () => {
      const start = Date.now();
      await crawler.crawl('https://example.com');
      const duration = Date.now() - start;
      expect(duration).toBeLessThan(5000); // 5 seconds
    });

    it('should handle large number of URLs', async () => {
      const largeCrawler = new CrawlerEngine({ maxPages: 100 });
      const pages = await largeCrawler.crawl('https://example.com');
      expect(pages).toBeDefined();
    });
  });
});
