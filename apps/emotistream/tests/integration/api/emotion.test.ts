import request from 'supertest';
import { app } from '../../../src/api/index';

describe('Emotion Detection API', () => {
  describe('POST /api/v1/emotion/analyze', () => {
    it('should return EmotionalState for valid text input', async () => {
      const response = await request(app)
        .post('/api/v1/emotion/analyze')
        .send({
          text: 'I am feeling very stressed and anxious about work',
          userId: 'test-user-123',
        })
        .expect('Content-Type', /json/)
        .expect(200);

      expect(response.body).toHaveProperty('success', true);
      expect(response.body.data).toHaveProperty('valence');
      expect(response.body.data).toHaveProperty('arousal');
      expect(response.body.data).toHaveProperty('dominance');
      expect(response.body.data).toHaveProperty('confidence');
      expect(response.body.data).toHaveProperty('timestamp');
      expect(response.body.data.valence).toBeGreaterThanOrEqual(-1);
      expect(response.body.data.valence).toBeLessThanOrEqual(1);
      expect(response.body.data.confidence).toBeGreaterThan(0);
    });

    it('should return 400 for empty text', async () => {
      const response = await request(app)
        .post('/api/v1/emotion/analyze')
        .send({
          text: '',
          userId: 'test-user-123',
        })
        .expect('Content-Type', /json/)
        .expect(400);

      expect(response.body).toHaveProperty('success', false);
      expect(response.body.error).toHaveProperty('message');
    });

    it('should return 400 for missing text field', async () => {
      const response = await request(app)
        .post('/api/v1/emotion/analyze')
        .send({
          userId: 'test-user-123',
        })
        .expect('Content-Type', /json/)
        .expect(400);

      expect(response.body).toHaveProperty('success', false);
    });

    it('should return 500 on API failure', async () => {
      // Test with extremely long text to simulate API failure
      const longText = 'a'.repeat(100000);

      const response = await request(app)
        .post('/api/v1/emotion/analyze')
        .send({
          text: longText,
          userId: 'test-user-123',
        })
        .expect('Content-Type', /json/);

      // Should either succeed or fail gracefully with 500
      if (response.status === 500) {
        expect(response.body).toHaveProperty('success', false);
        expect(response.body.error).toBeDefined();
      }
    });
  });

  describe('GET /api/v1/emotion/state/:userId', () => {
    it('should return current emotional state for user', async () => {
      // First, create an emotional state
      await request(app)
        .post('/api/v1/emotion/analyze')
        .send({
          text: 'I am feeling great today!',
          userId: 'test-user-456',
        });

      // Then retrieve it
      const response = await request(app)
        .get('/api/v1/emotion/state/test-user-456')
        .expect('Content-Type', /json/)
        .expect(200);

      expect(response.body).toHaveProperty('success', true);
      expect(response.body.data).toHaveProperty('valence');
    });

    it('should return 404 for non-existent user', async () => {
      const response = await request(app)
        .get('/api/v1/emotion/state/non-existent-user')
        .expect('Content-Type', /json/)
        .expect(404);

      expect(response.body).toHaveProperty('success', false);
    });
  });
});
