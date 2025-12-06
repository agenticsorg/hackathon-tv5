import request from 'supertest';
import { app } from '../../../src/api/index';

describe('Recommendation API', () => {
  describe('POST /api/v1/recommend', () => {
    it('should return recommendations for user and emotional state', async () => {
      const response = await request(app)
        .post('/api/v1/recommend')
        .send({
          userId: 'test-user-123',
          currentState: {
            valence: -0.6,
            arousal: 0.3,
            dominance: -0.2,
            confidence: 0.85,
          },
          limit: 5,
        })
        .expect('Content-Type', /json/)
        .expect(200);

      expect(response.body).toHaveProperty('success', true);
      expect(response.body.data).toBeInstanceOf(Array);
      expect(response.body.data.length).toBeLessThanOrEqual(5);

      if (response.body.data.length > 0) {
        const recommendation = response.body.data[0];
        expect(recommendation).toHaveProperty('contentId');
        expect(recommendation).toHaveProperty('title');
        expect(recommendation).toHaveProperty('score');
        expect(recommendation).toHaveProperty('reasoning');
      }
    });

    it('should respect limit parameter', async () => {
      const response = await request(app)
        .post('/api/v1/recommend')
        .send({
          userId: 'test-user-123',
          currentState: {
            valence: 0.5,
            arousal: -0.3,
            dominance: 0.1,
            confidence: 0.9,
          },
          limit: 3,
        })
        .expect(200);

      expect(response.body.data.length).toBeLessThanOrEqual(3);
    });

    it('should include reasoning for each recommendation', async () => {
      const response = await request(app)
        .post('/api/v1/recommend')
        .send({
          userId: 'test-user-123',
          currentState: {
            valence: -0.4,
            arousal: 0.2,
            dominance: 0.0,
            confidence: 0.8,
          },
        })
        .expect(200);

      if (response.body.data.length > 0) {
        response.body.data.forEach((rec: any) => {
          expect(rec).toHaveProperty('reasoning');
          expect(typeof rec.reasoning).toBe('string');
          expect(rec.reasoning.length).toBeGreaterThan(0);
        });
      }
    });

    it('should return 400 for invalid emotional state', async () => {
      const response = await request(app)
        .post('/api/v1/recommend')
        .send({
          userId: 'test-user-123',
          currentState: {
            valence: 2.0, // Invalid: should be -1 to 1
            arousal: 0.3,
            dominance: 0.1,
            confidence: 0.9,
          },
        })
        .expect(400);

      expect(response.body).toHaveProperty('success', false);
    });

    it('should return 400 for missing userId', async () => {
      const response = await request(app)
        .post('/api/v1/recommend')
        .send({
          currentState: {
            valence: 0.5,
            arousal: -0.3,
            dominance: 0.1,
            confidence: 0.9,
          },
        })
        .expect(400);

      expect(response.body).toHaveProperty('success', false);
    });
  });

  describe('GET /api/v1/recommend/history/:userId', () => {
    it('should return recommendation history for user', async () => {
      // First, create a recommendation
      await request(app)
        .post('/api/v1/recommend')
        .send({
          userId: 'test-user-history',
          currentState: {
            valence: 0.3,
            arousal: -0.2,
            dominance: 0.0,
            confidence: 0.85,
          },
        });

      // Then retrieve history
      const response = await request(app)
        .get('/api/v1/recommend/history/test-user-history')
        .expect('Content-Type', /json/)
        .expect(200);

      expect(response.body).toHaveProperty('success', true);
      expect(response.body.data).toBeInstanceOf(Array);
    });

    it('should return empty array for user with no history', async () => {
      const response = await request(app)
        .get('/api/v1/recommend/history/new-user-no-history')
        .expect(200);

      expect(response.body.data).toBeInstanceOf(Array);
      expect(response.body.data.length).toBe(0);
    });
  });
});
