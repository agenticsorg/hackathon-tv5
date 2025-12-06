/**
 * User Profile Store Persistence Tests
 * Verify user RL profiles persist correctly
 */

import { UserProfileStore } from '../../src/persistence/user-profile-store';
import * as fs from 'fs';
import * as path from 'path';

describe('UserProfileStore Persistence', () => {
  const testDataDir = path.join(process.cwd(), 'data');
  const testFile = path.join(testDataDir, 'user-profiles.json');
  let store: UserProfileStore;

  beforeEach(() => {
    if (fs.existsSync(testFile)) {
      fs.unlinkSync(testFile);
    }
    store = new UserProfileStore();
  });

  afterEach(() => {
    if (fs.existsSync(testFile)) {
      fs.unlinkSync(testFile);
    }
  });

  test('Create new user profile with defaults', async () => {
    const profile = await store.create('user-123');

    expect(profile.userId).toBe('user-123');
    expect(profile.explorationRate).toBe(0.3); // Default 30%
    expect(profile.totalExperiences).toBe(0);
    expect(profile.policyVersion).toBe(1);
  });

  test('Get or create returns existing profile', async () => {
    const created = await store.create('user-123');
    const retrieved = await store.getOrCreate('user-123');

    expect(retrieved.userId).toBe(created.userId);
    expect(retrieved.explorationRate).toBe(created.explorationRate);
  });

  test('Get or create creates new if not exists', async () => {
    const profile = await store.getOrCreate('user-new');

    expect(profile.userId).toBe('user-new');
    expect(profile.explorationRate).toBe(0.3);
  });

  test('Update exploration rate', async () => {
    await store.create('user-123');

    await store.updateExplorationRate('user-123', 0.5);

    const profile = await store.get('user-123');
    expect(profile?.explorationRate).toBe(0.5);
  });

  test('Exploration rate clamped to valid range', async () => {
    await store.create('user-123');

    // Too high
    await store.updateExplorationRate('user-123', 1.5);
    let profile = await store.get('user-123');
    expect(profile?.explorationRate).toBe(1.0);

    // Too low
    await store.updateExplorationRate('user-123', -0.1);
    profile = await store.get('user-123');
    expect(profile?.explorationRate).toBe(0.01);
  });

  test('Increment experiences increases count and decays exploration', async () => {
    await store.create('user-123');

    const initialProfile = await store.get('user-123');
    const initialExploration = initialProfile!.explorationRate;

    await store.incrementExperiences('user-123');

    const updatedProfile = await store.get('user-123');
    expect(updatedProfile!.totalExperiences).toBe(1);
    expect(updatedProfile!.explorationRate).toBeLessThan(initialExploration);
    expect(updatedProfile!.explorationRate).toBeGreaterThanOrEqual(0.05); // Min exploration
  });

  test('Exploration rate decays over multiple experiences', async () => {
    await store.create('user-123');

    // Simulate many experiences
    for (let i = 0; i < 100; i++) {
      await store.incrementExperiences('user-123');
    }

    const profile = await store.get('user-123');
    expect(profile!.totalExperiences).toBe(100);
    expect(profile!.explorationRate).toBeGreaterThanOrEqual(0.05); // Should hit minimum
    expect(profile!.explorationRate).toBeLessThan(0.3); // Should be less than initial
  });

  test('Increment policy version', async () => {
    await store.create('user-123');

    await store.incrementPolicyVersion('user-123');
    let profile = await store.get('user-123');
    expect(profile!.policyVersion).toBe(2);

    await store.incrementPolicyVersion('user-123');
    profile = await store.get('user-123');
    expect(profile!.policyVersion).toBe(3);
  });

  test('Reset exploration rate', async () => {
    await store.create('user-123');

    // Decay exploration
    for (let i = 0; i < 50; i++) {
      await store.incrementExperiences('user-123');
    }

    let profile = await store.get('user-123');
    expect(profile!.explorationRate).toBeLessThan(0.3);

    // Reset
    await store.resetExploration('user-123');
    profile = await store.get('user-123');
    expect(profile!.explorationRate).toBe(0.3);
  });

  test('Global statistics calculation', async () => {
    await store.create('user-1');
    await store.incrementExperiences('user-1');
    await store.incrementExperiences('user-1');
    await store.incrementExperiences('user-1');

    await store.create('user-2');
    await store.incrementExperiences('user-2');

    await store.create('user-3');

    const stats = await store.getGlobalStats();

    expect(stats.totalUsers).toBe(3);
    expect(stats.avgExperiences).toBeCloseTo((3 + 1 + 0) / 3);
    expect(stats.mostExperiencedUser?.userId).toBe('user-1');
    expect(stats.mostExperiencedUser?.count).toBe(3);
  });

  test('Global statistics handles empty store', async () => {
    const stats = await store.getGlobalStats();

    expect(stats.totalUsers).toBe(0);
    expect(stats.avgExplorationRate).toBe(0);
    expect(stats.avgExperiences).toBe(0);
    expect(stats.mostExperiencedUser).toBeNull();
  });

  test('Delete user profile', async () => {
    await store.create('user-123');

    const deleted = await store.delete('user-123');
    expect(deleted).toBe(true);

    const profile = await store.get('user-123');
    expect(profile).toBeNull();
  });

  test('Profiles persist across store instances', async () => {
    const store1 = new UserProfileStore();
    await store1.create('user-persistent');
    await store1.incrementExperiences('user-persistent');
    store1.flush();

    await new Promise(resolve => setTimeout(resolve, 1500));

    const store2 = new UserProfileStore();
    const profile = await store2.get('user-persistent');

    expect(profile).toBeDefined();
    expect(profile?.totalExperiences).toBe(1);
    expect(profile?.explorationRate).toBeLessThan(0.3);
  });

  test('Last updated timestamp changes on modifications', async () => {
    await store.create('user-123');
    const profile1 = await store.get('user-123');
    const timestamp1 = profile1!.lastUpdated;

    await new Promise(resolve => setTimeout(resolve, 50));

    await store.incrementExperiences('user-123');
    const profile2 = await store.get('user-123');
    const timestamp2 = profile2!.lastUpdated;

    expect(timestamp2).toBeGreaterThan(timestamp1);
  });
});
