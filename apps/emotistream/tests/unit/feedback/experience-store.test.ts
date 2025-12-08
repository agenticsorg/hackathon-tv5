/**
 * ExperienceStore Unit Tests
 */

import { ExperienceStore } from '../../../src/feedback/experience-store';
import { EmotionalExperience, EmotionalState } from '../../../src/feedback/types';

// Mock AgentDB
const mockAgentDB = {
  set: jest.fn(),
  get: jest.fn(),
  delete: jest.fn(),
  zadd: jest.fn(),
  zcard: jest.fn(),
  zrange: jest.fn(),
  zremrangebyrank: jest.fn(),
  lpush: jest.fn(),
  ltrim: jest.fn(),
};

describe('ExperienceStore', () => {
  let store: ExperienceStore;

  const mockExperience: EmotionalExperience = {
    experienceId: 'exp-123',
    userId: 'user-456',
    contentId: 'content-789',
    stateBeforeId: 'state-before-123',
    stateAfter: {
      valence: 0.5,
      arousal: -0.2,
      dominance: 0.1,
      confidence: 0.8,
      timestamp: new Date(),
    } as EmotionalState,
    desiredState: {
      valence: 0.6,
      arousal: -0.3,
      dominance: 0.2,
      confidence: 1.0,
      timestamp: new Date(),
    } as EmotionalState,
    reward: 0.85,
    qValueBefore: 0.5,
    qValueAfter: 0.585,
    timestamp: new Date(),
    metadata: {
      viewingDetails: {
        completionRate: 0.95,
        durationSeconds: 1800,
      },
    },
  };

  beforeEach(() => {
    jest.clearAllMocks();
    store = new ExperienceStore(mockAgentDB as any);
  });

  describe('store', () => {
    it('should store experience in AgentDB', async () => {
      mockAgentDB.set.mockResolvedValue(true);
      mockAgentDB.zadd.mockResolvedValue(1);
      mockAgentDB.zcard.mockResolvedValue(1);
      mockAgentDB.lpush.mockResolvedValue(1);
      mockAgentDB.ltrim.mockResolvedValue(true);

      const result = await store.store(mockExperience);

      expect(result).toBe(true);
      expect(mockAgentDB.set).toHaveBeenCalledWith(
        `exp:${mockExperience.experienceId}`,
        mockExperience,
        expect.any(Number)
      );
    });

    it('should add to user experience list', async () => {
      mockAgentDB.set.mockResolvedValue(true);
      mockAgentDB.zadd.mockResolvedValue(1);
      mockAgentDB.zcard.mockResolvedValue(1);
      mockAgentDB.lpush.mockResolvedValue(1);
      mockAgentDB.ltrim.mockResolvedValue(true);

      await store.store(mockExperience);

      expect(mockAgentDB.zadd).toHaveBeenCalledWith(
        `user:${mockExperience.userId}:experiences`,
        expect.any(Number),
        mockExperience.experienceId
      );
    });

    it('should add to global replay buffer', async () => {
      mockAgentDB.set.mockResolvedValue(true);
      mockAgentDB.zadd.mockResolvedValue(1);
      mockAgentDB.zcard.mockResolvedValue(1);
      mockAgentDB.lpush.mockResolvedValue(1);
      mockAgentDB.ltrim.mockResolvedValue(true);

      await store.store(mockExperience);

      expect(mockAgentDB.lpush).toHaveBeenCalledWith(
        'global:experience_replay',
        mockExperience.experienceId
      );
      expect(mockAgentDB.ltrim).toHaveBeenCalled();
    });

    it('should handle storage failure gracefully', async () => {
      mockAgentDB.set.mockRejectedValue(new Error('DB error'));

      const result = await store.store(mockExperience);

      expect(result).toBe(false);
    });
  });

  describe('retrieve', () => {
    it('should retrieve experience by ID', async () => {
      mockAgentDB.get.mockResolvedValue(mockExperience);

      const result = await store.retrieve('exp-123');

      expect(result).toEqual(mockExperience);
      expect(mockAgentDB.get).toHaveBeenCalledWith('exp:exp-123');
    });

    it('should return null for non-existent experience', async () => {
      mockAgentDB.get.mockResolvedValue(null);

      const result = await store.retrieve('exp-999');

      expect(result).toBeNull();
    });
  });

  describe('getUserExperiences', () => {
    it('should return user experiences in chronological order', async () => {
      const expIds = ['exp-1', 'exp-2', 'exp-3'];
      mockAgentDB.zrange.mockResolvedValue(expIds);
      mockAgentDB.get.mockImplementation((key: string) => {
        const id = key.split(':')[1];
        return Promise.resolve({ ...mockExperience, experienceId: id });
      });

      const result = await store.getUserExperiences('user-456', 100);

      expect(result).toHaveLength(3);
      expect(mockAgentDB.zrange).toHaveBeenCalledWith(
        'user:user-456:experiences',
        0,
        99
      );
    });

    it('should respect limit parameter', async () => {
      const expIds = ['exp-1', 'exp-2'];
      mockAgentDB.zrange.mockResolvedValue(expIds);
      mockAgentDB.get.mockResolvedValue(mockExperience);

      await store.getUserExperiences('user-456', 50);

      expect(mockAgentDB.zrange).toHaveBeenCalledWith(
        'user:user-456:experiences',
        0,
        49
      );
    });
  });
});
