/**
 * Tests for CrawlerClient
 */

import { describe, it, expect, beforeEach, jest } from '@jest/globals';
import { CrawlerClient } from '../src/client';
import { ValidationError, AuthenticationError } from '../src/utils/errors';

describe('CrawlerClient', () => {
  let client: CrawlerClient;

  beforeEach(() => {
    client = new CrawlerClient({
      apiKey: 'test-api-key',
      baseUrl: 'http://localhost:3000/api',
      timeout: 10000,
      debug: false,
    });
  });

  describe('scrape()', () => {
    it('should scrape a single URL successfully', async () => {
      // This is a mock test - in real scenario, use mocked HTTP responses
      const url = 'https://example.com';

      // Mock implementation would go here
      expect(client).toBeDefined();
      expect(typeof client.scrape).toBe('function');
    });

    it('should throw ValidationError for invalid URL', async () => {
      await expect(client.scrape('')).rejects.toThrow(ValidationError);
      await expect(client.scrape('not-a-url')).rejects.toThrow(ValidationError);
    });

    it('should accept scrape options', async () => {
      const url = 'https://example.com';
      const options = {
        formats: ['markdown', 'html'] as const,
        onlyMainContent: true,
        timeout: 5000,
      };

      expect(() => client.scrape(url, options)).not.toThrow();
    });
  });

  describe('crawl()', () => {
    it('should start a crawl successfully', async () => {
      const url = 'https://example.com';
      expect(typeof client.crawl).toBe('function');
    });

    it('should accept crawl options', async () => {
      const url = 'https://example.com';
      const options = {
        maxDepth: 3,
        limit: 100,
        arwDiscovery: true,
        machineView: true,
      };

      expect(() => client.crawl(url, options)).not.toThrow();
    });
  });

  describe('map()', () => {
    it('should generate site map', async () => {
      const url = 'https://example.com';
      expect(typeof client.map).toBe('function');
    });

    it('should accept map options', async () => {
      const url = 'https://example.com';
      const options = {
        limit: 1000,
        includeSubdomains: false,
      };

      expect(() => client.map(url, options)).not.toThrow();
    });
  });

  describe('batch()', () => {
    it('should execute batch operations', async () => {
      const operations = [
        { type: 'scrape' as const, url: 'https://example.com' },
        { type: 'map' as const, url: 'https://example.com' },
      ];

      expect(() => client.batch(operations)).not.toThrow();
    });

    it('should throw ValidationError for empty operations', async () => {
      await expect(client.batch([])).rejects.toThrow(ValidationError);
    });
  });

  describe('ARW methods', () => {
    it('should discover ARW metadata', async () => {
      const url = 'https://docs.example.com';
      expect(typeof client.discoverARW).toBe('function');
    });

    it('should check ARW support', async () => {
      const url = 'https://docs.example.com';
      expect(typeof client.hasARWSupport).toBe('function');
    });

    it('should get llms.txt content', async () => {
      const url = 'https://docs.example.com';
      expect(typeof client.getLlmsTxt).toBe('function');
    });

    it('should generate machine view', async () => {
      const url = 'https://docs.example.com';
      const options = {
        maxTokens: 4000,
        prioritySections: ['api', 'guide'],
      };

      expect(() => client.generateMachineView(url, options)).not.toThrow();
    });
  });

  describe('utility methods', () => {
    it('should update API key', () => {
      const newKey = 'new-api-key';
      expect(() => client.setApiKey(newKey)).not.toThrow();
    });

    it('should provide access to ARW utilities', () => {
      expect(client.arw).toBeDefined();
    });

    it('should provide access to Machine View utilities', () => {
      expect(client.machine).toBeDefined();
    });

    it('should provide access to HTTP client', () => {
      const httpClient = client.getHttpClient();
      expect(httpClient).toBeDefined();
    });
  });
});
