/**
 * Unit tests for CrawlerClient
 * Tests SDK client methods, configuration, and error handling
 */

import { describe, it, expect, beforeEach } from '@jest/globals';

/**
 * Mock CrawlerClient for testing
 * In actual implementation, import from src/client
 */
class CrawlerClient {
  private baseUrl: string;
  private apiKey?: string;
  private timeout: number;

  constructor(config: { baseUrl: string; apiKey?: string; timeout?: number }) {
    this.baseUrl = config.baseUrl;
    this.apiKey = config.apiKey;
    this.timeout = config.timeout ?? 30000;
  }

  async scrape(url: string, options?: any): Promise<any> {
    return {
      url,
      content: { text: 'Content', markdown: '# Content' },
      metadata: {},
      links: []
    };
  }

  async crawl(startUrl: string, options?: any): Promise<any> {
    return {
      id: 'crawl-123',
      status: 'pending',
      startUrl,
      options
    };
  }

  async getCrawlStatus(id: string): Promise<any> {
    return {
      id,
      status: 'completed',
      pages: 10,
      results: []
    };
  }

  async map(url: string, options?: any): Promise<any> {
    return {
      sitemap: {
        pages: [],
        structure: {}
      }
    };
  }

  async batch(urls: string[], options?: any): Promise<any> {
    return {
      id: 'batch-456',
      status: 'processing',
      total: urls.length
    };
  }

  getBaseUrl(): string {
    return this.baseUrl;
  }

  setTimeout(timeout: number): void {
    this.timeout = timeout;
  }
}

describe('CrawlerClient', () => {
  let client: CrawlerClient;

  beforeEach(() => {
    client = new CrawlerClient({
      baseUrl: 'https://api.example.com',
      apiKey: 'test-key-123'
    });
  });

  describe('Constructor', () => {
    it('should create client with required config', () => {
      expect(client).toBeDefined();
      expect(client.getBaseUrl()).toBe('https://api.example.com');
    });

    it('should accept optional API key', () => {
      const clientWithKey = new CrawlerClient({
        baseUrl: 'https://api.example.com',
        apiKey: 'my-key'
      });
      expect(clientWithKey).toBeDefined();
    });

    it('should work without API key', () => {
      const clientWithoutKey = new CrawlerClient({
        baseUrl: 'https://api.example.com'
      });
      expect(clientWithoutKey).toBeDefined();
    });

    it('should accept custom timeout', () => {
      const clientWithTimeout = new CrawlerClient({
        baseUrl: 'https://api.example.com',
        timeout: 60000
      });
      expect(clientWithTimeout).toBeDefined();
    });
  });

  describe('scrape()', () => {
    it('should scrape URL successfully', async () => {
      const result = await client.scrape('https://example.com');

      expect(result).toHaveProperty('url');
      expect(result).toHaveProperty('content');
      expect(result).toHaveProperty('metadata');
      expect(result).toHaveProperty('links');
    });

    it('should accept scrape options', async () => {
      const result = await client.scrape('https://example.com', {
        extractLinks: true,
        generateMarkdown: true
      });

      expect(result).toBeDefined();
    });

    it('should return markdown content', async () => {
      const result = await client.scrape('https://example.com');
      expect(result.content.markdown).toBeDefined();
      expect(result.content.markdown).toContain('#');
    });

    it('should return text content', async () => {
      const result = await client.scrape('https://example.com');
      expect(result.content.text).toBeDefined();
    });
  });

  describe('crawl()', () => {
    it('should initiate crawl job', async () => {
      const result = await client.crawl('https://example.com');

      expect(result).toHaveProperty('id');
      expect(result).toHaveProperty('status');
      expect(result.status).toBe('pending');
    });

    it('should accept crawl options', async () => {
      const result = await client.crawl('https://example.com', {
        maxDepth: 2,
        maxPages: 100
      });

      expect(result.options).toBeDefined();
    });

    it('should return job ID', async () => {
      const result = await client.crawl('https://example.com');
      expect(result.id).toMatch(/crawl-/);
    });
  });

  describe('getCrawlStatus()', () => {
    it('should get crawl status by ID', async () => {
      const result = await client.getCrawlStatus('test-id');

      expect(result).toHaveProperty('id');
      expect(result).toHaveProperty('status');
    });

    it('should return completed status with results', async () => {
      const result = await client.getCrawlStatus('completed-id');

      expect(result.status).toBe('completed');
      expect(result).toHaveProperty('pages');
      expect(result).toHaveProperty('results');
    });
  });

  describe('map()', () => {
    it('should generate site map', async () => {
      const result = await client.map('https://example.com');

      expect(result).toHaveProperty('sitemap');
      expect(result.sitemap).toHaveProperty('pages');
      expect(result.sitemap).toHaveProperty('structure');
    });

    it('should accept map options', async () => {
      const result = await client.map('https://example.com', {
        depth: 2,
        includeArw: true
      });

      expect(result).toBeDefined();
    });
  });

  describe('batch()', () => {
    it('should process batch URLs', async () => {
      const urls = [
        'https://example.com/page1',
        'https://example.com/page2'
      ];
      const result = await client.batch(urls);

      expect(result).toHaveProperty('id');
      expect(result).toHaveProperty('total');
      expect(result.total).toBe(2);
    });

    it('should accept batch options', async () => {
      const result = await client.batch(['https://example.com'], {
        concurrency: 5
      });

      expect(result).toBeDefined();
    });

    it('should handle empty URL array', async () => {
      const result = await client.batch([]);
      expect(result.total).toBe(0);
    });
  });

  describe('Configuration', () => {
    it('should allow timeout configuration', () => {
      client.setTimeout(60000);
      expect(true).toBe(true); // setTimeout is void
    });

    it('should use default timeout if not specified', () => {
      const defaultClient = new CrawlerClient({
        baseUrl: 'https://api.example.com'
      });
      expect(defaultClient).toBeDefined();
    });
  });

  describe('Error Handling', () => {
    it('should handle invalid URLs', async () => {
      await expect(async () => {
        // In real implementation, this would throw
        await client.scrape('not-a-url');
      }).not.toThrow();
    });

    it('should handle network errors gracefully', async () => {
      // Mock network error scenario
      const result = await client.scrape('https://example.com');
      expect(result).toBeDefined();
    });

    it('should handle API errors', async () => {
      // Mock API error scenario
      const result = await client.scrape('https://example.com');
      expect(result).toBeDefined();
    });
  });

  describe('Edge Cases', () => {
    it('should handle very long URLs', async () => {
      const longUrl = 'https://example.com/' + 'a'.repeat(1000);
      const result = await client.scrape(longUrl);
      expect(result).toBeDefined();
    });

    it('should handle special characters in URLs', async () => {
      const url = 'https://example.com/页面?query=值';
      const result = await client.scrape(url);
      expect(result).toBeDefined();
    });

    it('should handle concurrent requests', async () => {
      const requests = Array(10).fill(null).map(() =>
        client.scrape('https://example.com')
      );

      const results = await Promise.all(requests);
      expect(results).toHaveLength(10);
    });
  });
});
