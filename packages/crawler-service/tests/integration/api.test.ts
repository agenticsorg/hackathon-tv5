import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import request from 'supertest';
import { CrawlerService } from '../../src/app.js';

describe('API Integration Tests', () => {
  let service: CrawlerService;
  let app: any;

  beforeAll(async () => {
    service = new CrawlerService();
    app = service.getApp();
  });

  afterAll(async () => {
    await service.stop();
  });

  describe('GET /', () => {
    it('should return service information', async () => {
      const response = await request(app).get('/');

      expect(response.status).toBe(200);
      expect(response.body).toHaveProperty('name');
      expect(response.body).toHaveProperty('version');
      expect(response.body).toHaveProperty('endpoints');
    });
  });

  describe('GET /api/v1/health', () => {
    it('should return health status', async () => {
      const response = await request(app).get('/api/v1/health');

      expect(response.status).toBe(200);
      expect(response.body).toHaveProperty('status', 'ok');
      expect(response.body).toHaveProperty('timestamp');
    });
  });

  describe('POST /api/v1/scrape', () => {
    it('should scrape a URL', async () => {
      const response = await request(app)
        .post('/api/v1/scrape')
        .send({ url: 'https://example.com' });

      expect(response.status).toBe(200);
      expect(response.body.success).toBe(true);
      expect(response.body.data).toHaveProperty('url');
      expect(response.body.data).toHaveProperty('content');
    });

    it('should return validation error for invalid URL', async () => {
      const response = await request(app)
        .post('/api/v1/scrape')
        .send({ url: 'not-a-url' });

      expect(response.status).toBe(400);
      expect(response.body.success).toBe(false);
      expect(response.body.error.code).toBe('VALIDATION_ERROR');
    });
  });

  describe('POST /api/v1/crawl', () => {
    it('should create a crawl job', async () => {
      const response = await request(app)
        .post('/api/v1/crawl')
        .send({ url: 'https://example.com', maxDepth: 2 });

      expect(response.status).toBe(200);
      expect(response.body.success).toBe(true);
      expect(response.body.data).toHaveProperty('jobId');
      expect(response.body.data.status).toBe('pending');
    });
  });

  describe('GET /api/v1/stats', () => {
    it('should return service statistics', async () => {
      const response = await request(app).get('/api/v1/stats');

      expect(response.status).toBe(200);
      expect(response.body.success).toBe(true);
      expect(response.body.data).toHaveProperty('queue');
      expect(response.body.data).toHaveProperty('cache');
    });
  });
});
