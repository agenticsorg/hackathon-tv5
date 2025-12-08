import request from 'supertest';
import { app } from '../../../src/api/index';

describe('Feedback API', () => {
  describe('POST /api/v1/feedback', () => {
    it('should process feedback and return reward', async () => {
      // First, create an emotional state
      const stateResponse = await request(app)
        .post('/api/v1/emotion/analyze')
        .send({
          text: 'I am feeling stressed',
          userId: 'test-user-feedback',
        });

      const emotionalStateId = stateResponse.body.data.id || 'state-123';

      // Submit feedback
      const response = await request(app)
        .post('/api/v1/feedback')
        .send({
          userId: 'test-user-feedback',
          contentId: 'content-relaxation-video',
          emotionalStateId,
          postViewingState: {
            text: 'I feel much more relaxed now',
          },
          viewingDetails: {
            completionRate: 1.0,
            durationSeconds: 1800,
            pauseCount: 1,
            skipCount: 0,
          },
        })
        .expect('Content-Type', /json/)
        .expect(200);

      expect(response.body).toHaveProperty('success', true);
      expect(response.body.data).toHaveProperty('experienceId');
      expect(response.body.data).toHaveProperty('reward');
      expect(response.body.data.reward).toBeGreaterThanOrEqual(-1);
      expect(response.body.data.reward).toBeLessThanOrEqual(1);
    });

    it('should update policy after feedback', async () => {
      const response = await request(app)
        .post('/api/v1/feedback')
        .send({
          userId: 'test-user-policy',
          contentId: 'content-123',
          emotionalStateId: 'state-456',
          postViewingState: {
            explicitRating: 5,
          },
          viewingDetails: {
            completionRate: 0.95,
            durationSeconds: 1200,
          },
        })
        .expect(200);

      expect(response.body.data).toHaveProperty('policyUpdated', true);
      expect(response.body.data).toHaveProperty('qValueBefore');
      expect(response.body.data).toHaveProperty('qValueAfter');
    });

    it('should return learning progress metrics', async () => {
      const response = await request(app)
        .post('/api/v1/feedback')
        .send({
          userId: 'test-user-learning',
          contentId: 'content-789',
          emotionalStateId: 'state-789',
          postViewingState: {
            explicitEmoji: '😊',
          },
          viewingDetails: {
            completionRate: 1.0,
            durationSeconds: 900,
          },
        })
        .expect(200);

      expect(response.body.data).toHaveProperty('emotionalImprovement');
      expect(response.body.data).toHaveProperty('insights');
      expect(response.body.data.insights).toHaveProperty('directionAlignment');
      expect(response.body.data.insights).toHaveProperty('magnitudeScore');
    });

    it('should return 400 for missing required fields', async () => {
      const response = await request(app)
        .post('/api/v1/feedback')
        .send({
          userId: 'test-user',
          contentId: 'content-123',
          // Missing emotionalStateId and postViewingState
        })
        .expect(400);

      expect(response.body).toHaveProperty('success', false);
    });

    it('should return 400 for invalid completion rate', async () => {
      const response = await request(app)
        .post('/api/v1/feedback')
        .send({
          userId: 'test-user',
          contentId: 'content-123',
          emotionalStateId: 'state-123',
          postViewingState: {
            text: 'Great video',
          },
          viewingDetails: {
            completionRate: 1.5, // Invalid: should be 0-1
            durationSeconds: 1000,
          },
        })
        .expect(400);

      expect(response.body).toHaveProperty('success', false);
    });

    it('should accept text, rating, or emoji feedback', async () => {
      // Test text feedback
      const textResponse = await request(app)
        .post('/api/v1/feedback')
        .send({
          userId: 'test-user',
          contentId: 'content-1',
          emotionalStateId: 'state-1',
          postViewingState: {
            text: 'Excellent content',
          },
        })
        .expect(200);

      expect(textResponse.body.data).toHaveProperty('reward');

      // Test rating feedback
      const ratingResponse = await request(app)
        .post('/api/v1/feedback')
        .send({
          userId: 'test-user',
          contentId: 'content-2',
          emotionalStateId: 'state-2',
          postViewingState: {
            explicitRating: 4,
          },
        })
        .expect(200);

      expect(ratingResponse.body.data).toHaveProperty('reward');

      // Test emoji feedback
      const emojiResponse = await request(app)
        .post('/api/v1/feedback')
        .send({
          userId: 'test-user',
          contentId: 'content-3',
          emotionalStateId: 'state-3',
          postViewingState: {
            explicitEmoji: '❤️',
          },
        })
        .expect(200);

      expect(emojiResponse.body.data).toHaveProperty('reward');
    });
  });

  describe('GET /api/v1/feedback/progress/:userId', () => {
    it('should return learning progress for user', async () => {
      const response = await request(app)
        .get('/api/v1/feedback/progress/test-user-progress')
        .expect('Content-Type', /json/)
        .expect(200);

      expect(response.body).toHaveProperty('success', true);
      expect(response.body.data).toHaveProperty('totalExperiences');
      expect(response.body.data).toHaveProperty('avgReward');
      expect(response.body.data).toHaveProperty('learningProgress');
    });

    it('should return 404 for non-existent user', async () => {
      const response = await request(app)
        .get('/api/v1/feedback/progress/non-existent-user')
        .expect(404);

      expect(response.body).toHaveProperty('success', false);
    });
  });
});
