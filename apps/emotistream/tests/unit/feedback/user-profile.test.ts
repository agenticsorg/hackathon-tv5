/**
 * UserProfileManager Unit Tests
 */

import { UserProfileManager } from '../../../src/feedback/user-profile';
import { UserProfile } from '../../../src/feedback/types';

const mockAgentDB = {
  get: jest.fn(),
  set: jest.fn(),
};

describe('UserProfileManager', () => {
  let manager: UserProfileManager;

  beforeEach(() => {
    jest.clearAllMocks();
    manager = new UserProfileManager(mockAgentDB as any);
  });

  describe('update', () => {
    it('should create new profile for first-time user', async () => {
      mockAgentDB.get.mockResolvedValue(null);
      mockAgentDB.set.mockResolvedValue(true);

      const result = await manager.update('user-123', 0.8);

      expect(result).toBe(true);
      expect(mockAgentDB.set).toHaveBeenCalledWith(
        'user:user-123:profile',
        expect.objectContaining({
          userId: 'user-123',
          totalExperiences: 1,
          explorationRate: expect.any(Number),
        })
      );
    });

    it('should update existing profile', async () => {
      const existingProfile: UserProfile = {
        userId: 'user-123',
        totalExperiences: 10,
        avgReward: 0.5,
        explorationRate: 0.2,
        preferredGenres: ['comedy'],
        learningProgress: 50,
      };

      mockAgentDB.get.mockResolvedValue(existingProfile);
      mockAgentDB.set.mockResolvedValue(true);

      await manager.update('user-123', 0.7);

      expect(mockAgentDB.set).toHaveBeenCalledWith(
        'user:user-123:profile',
        expect.objectContaining({
          totalExperiences: 11,
          avgReward: expect.any(Number),
        })
      );
    });

    it('should decay exploration rate with each update', async () => {
      const existingProfile: UserProfile = {
        userId: 'user-123',
        totalExperiences: 5,
        avgReward: 0.6,
        explorationRate: 0.25,
        preferredGenres: [],
        learningProgress: 30,
      };

      mockAgentDB.get.mockResolvedValue(existingProfile);
      mockAgentDB.set.mockResolvedValue(true);

      await manager.update('user-123', 0.8);

      const savedProfile = (mockAgentDB.set as jest.Mock).mock.calls[0][1];
      expect(savedProfile.explorationRate).toBeLessThan(0.25);
      expect(savedProfile.explorationRate).toBeGreaterThanOrEqual(0.05);
    });

    it('should update average reward using exponential moving average', async () => {
      const existingProfile: UserProfile = {
        userId: 'user-123',
        totalExperiences: 10,
        avgReward: 0.5,
        explorationRate: 0.15,
        preferredGenres: [],
        learningProgress: 50,
      };

      mockAgentDB.get.mockResolvedValue(existingProfile);
      mockAgentDB.set.mockResolvedValue(true);

      await manager.update('user-123', 0.9);

      const savedProfile = (mockAgentDB.set as jest.Mock).mock.calls[0][1];
      // EMA: 0.1 * 0.9 + 0.9 * 0.5 = 0.09 + 0.45 = 0.54
      expect(savedProfile.avgReward).toBeCloseTo(0.54, 2);
    });

    it('should calculate learning progress', async () => {
      const existingProfile: UserProfile = {
        userId: 'user-123',
        totalExperiences: 50,
        avgReward: 0.6,
        explorationRate: 0.1,
        preferredGenres: [],
        learningProgress: 60,
      };

      mockAgentDB.get.mockResolvedValue(existingProfile);
      mockAgentDB.set.mockResolvedValue(true);

      await manager.update('user-123', 0.7);

      const savedProfile = (mockAgentDB.set as jest.Mock).mock.calls[0][1];
      expect(savedProfile.learningProgress).toBeGreaterThan(0);
      expect(savedProfile.learningProgress).toBeLessThanOrEqual(100);
    });
  });

  describe('get', () => {
    it('should retrieve user profile', async () => {
      const profile: UserProfile = {
        userId: 'user-123',
        totalExperiences: 25,
        avgReward: 0.65,
        explorationRate: 0.12,
        preferredGenres: ['drama', 'comedy'],
        learningProgress: 70,
      };

      mockAgentDB.get.mockResolvedValue(profile);

      const result = await manager.get('user-123');

      expect(result).toEqual(profile);
      expect(mockAgentDB.get).toHaveBeenCalledWith('user:user-123:profile');
    });

    it('should return null for non-existent user', async () => {
      mockAgentDB.get.mockResolvedValue(null);

      const result = await manager.get('user-999');

      expect(result).toBeNull();
    });
  });
});
