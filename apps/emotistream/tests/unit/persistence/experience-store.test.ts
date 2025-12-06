/**
 * Experience Store Persistence Tests
 * Verify experience replay buffer persists correctly
 */

import { ExperienceStore } from '../../src/persistence/experience-store';
import { Experience } from '../../src/rl/types';
import * as fs from 'fs';
import * as path from 'path';

describe('ExperienceStore Persistence', () => {
  const testDataDir = path.join(process.cwd(), 'data');
  const testFile = path.join(testDataDir, 'experiences.json');
  let store: ExperienceStore;

  beforeEach(() => {
    if (fs.existsSync(testFile)) {
      fs.unlinkSync(testFile);
    }
    store = new ExperienceStore();
  });

  afterEach(() => {
    if (fs.existsSync(testFile)) {
      fs.unlinkSync(testFile);
    }
  });

  const createMockExperience = (userId: string, reward: number, completed: boolean = true): Experience => ({
    id: `exp-${Date.now()}-${Math.random()}`,
    userId,
    stateHash: 'state-test',
    contentId: 'content-test',
    reward,
    nextStateHash: 'state-next',
    timestamp: Date.now(),
    completed,
    watchDuration: completed ? 1800 : 900
  });

  test('Add and retrieve experience', async () => {
    const experience = createMockExperience('user-123', 0.8);
    await store.add(experience);

    const retrieved = await store.get(experience.id);
    expect(retrieved).toEqual(experience);
  });

  test('Get experiences by user', async () => {
    await store.add(createMockExperience('user-1', 0.5));
    await store.add(createMockExperience('user-1', 0.7));
    await store.add(createMockExperience('user-2', 0.3));

    const user1Exps = await store.getByUser('user-1');
    expect(user1Exps).toHaveLength(2);
    expect(user1Exps.every(e => e.userId === 'user-1')).toBe(true);
  });

  test('Get recent experiences returns latest first', async () => {
    const exp1 = createMockExperience('user-1', 0.5);
    await new Promise(resolve => setTimeout(resolve, 10));
    const exp2 = createMockExperience('user-1', 0.7);
    await new Promise(resolve => setTimeout(resolve, 10));
    const exp3 = createMockExperience('user-1', 0.9);

    await store.add(exp1);
    await store.add(exp2);
    await store.add(exp3);

    const recent = await store.getRecent('user-1', 2);
    expect(recent).toHaveLength(2);
    expect(recent[0].id).toBe(exp3.id); // Most recent first
    expect(recent[1].id).toBe(exp2.id);
  });

  test('Get completed experiences only', async () => {
    await store.add(createMockExperience('user-1', 0.5, true));
    await store.add(createMockExperience('user-1', 0.3, false));
    await store.add(createMockExperience('user-1', 0.7, true));

    const completed = await store.getCompleted('user-1');
    expect(completed).toHaveLength(2);
    expect(completed.every(e => e.completed)).toBe(true);
  });

  test('Get high reward experiences', async () => {
    await store.add(createMockExperience('user-1', 0.3));
    await store.add(createMockExperience('user-1', 0.7));
    await store.add(createMockExperience('user-1', 0.9));

    const highReward = await store.getHighReward('user-1', 0.6);
    expect(highReward).toHaveLength(2);
    expect(highReward.every(e => e.reward >= 0.6)).toBe(true);
  });

  test('Sample returns random subset', async () => {
    for (let i = 0; i < 10; i++) {
      await store.add(createMockExperience('user-1', Math.random()));
    }

    const sample = await store.sample('user-1', 5);
    expect(sample).toHaveLength(5);

    // Verify all are unique
    const ids = sample.map(e => e.id);
    expect(new Set(ids).size).toBe(5);
  });

  test('Cleanup removes old experiences', async () => {
    const oldExp = createMockExperience('user-1', 0.5);
    oldExp.timestamp = Date.now() - 10000; // 10 seconds ago

    const recentExp = createMockExperience('user-1', 0.7);

    await store.add(oldExp);
    await store.add(recentExp);

    // Clean up experiences older than 5 seconds
    const deletedCount = await store.cleanup(5000);

    expect(deletedCount).toBe(1);

    const remaining = await store.getByUser('user-1');
    expect(remaining).toHaveLength(1);
    expect(remaining[0].id).toBe(recentExp.id);
  });

  test('Statistics calculation', async () => {
    await store.add(createMockExperience('user-1', 0.5, true));
    await store.add(createMockExperience('user-1', 0.7, true));
    await store.add(createMockExperience('user-1', 0.3, false));

    const stats = await store.getStats('user-1');

    expect(stats.total).toBe(3);
    expect(stats.completed).toBe(2);
    expect(stats.completionRate).toBeCloseTo(2 / 3);
    expect(stats.avgReward).toBeCloseTo((0.5 + 0.7 + 0.3) / 3);
    expect(stats.avgWatchDuration).toBe(1800); // Average of completed only
  });

  test('Experiences persist across store instances', async () => {
    const exp = createMockExperience('user-1', 0.8);

    const store1 = new ExperienceStore();
    await store1.add(exp);
    store1.flush();

    await new Promise(resolve => setTimeout(resolve, 1500));

    const store2 = new ExperienceStore();
    const retrieved = await store2.get(exp.id);

    expect(retrieved).toBeDefined();
    expect(retrieved?.reward).toBe(0.8);
  });
});
