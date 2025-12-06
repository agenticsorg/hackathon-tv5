/**
 * Q-Table Persistence Tests
 * Verify Q-values persist across restarts
 */

import { QTable } from '../../src/rl/q-table';
import { QTableStore } from '../../src/persistence/q-table-store';
import * as fs from 'fs';
import * as path from 'path';

describe('QTable Persistence', () => {
  const testDataDir = path.join(process.cwd(), 'data');
  const testQTableFile = path.join(testDataDir, 'qtable.json');

  beforeEach(() => {
    // Clean up test data before each test
    if (fs.existsSync(testQTableFile)) {
      fs.unlinkSync(testQTableFile);
    }
  });

  afterEach(() => {
    // Clean up test data after each test
    if (fs.existsSync(testQTableFile)) {
      fs.unlinkSync(testQTableFile);
    }
  });

  test('Q-values persist across QTable instances', async () => {
    const stateHash = 'state-stressed-anxious';
    const contentId = 'content-123';
    const qValue = 0.75;

    // First instance: set Q-value
    const qTable1 = new QTable();
    await qTable1.setQValue(stateHash, contentId, qValue);
    qTable1.flush(); // Force write to disk

    // Wait for file write
    await new Promise(resolve => setTimeout(resolve, 100));

    // Second instance: should load existing data
    const qTable2 = new QTable();
    const retrieved = await qTable2.getQValue(stateHash, contentId);

    expect(retrieved).toBe(qValue);
  });

  test('Q-learning update rule works correctly', async () => {
    const qTable = new QTable();

    const state1 = 'state-sad';
    const state2 = 'state-happy';
    const contentId = 'content-uplifting';

    // Initial Q-value should be 0
    const initialQ = await qTable.getQValue(state1, contentId);
    expect(initialQ).toBe(0);

    // Update with positive reward
    await qTable.updateQValue(
      state1,
      contentId,
      1.0, // High reward
      state2, // Next state
      0.1, // Learning rate
      0.9 // Discount factor
    );

    // Q-value should increase
    const updatedQ = await qTable.getQValue(state1, contentId);
    expect(updatedQ).toBeGreaterThan(0);
    expect(updatedQ).toBeLessThanOrEqual(1.0);
  });

  test('Visit count increments on updates', async () => {
    const qTable = new QTable();
    const stateHash = 'state-test';
    const contentId = 'content-test';

    // First update
    await qTable.setQValue(stateHash, contentId, 0.5);
    let entry = await qTable.get(stateHash, contentId);
    expect(entry?.visitCount).toBe(1);

    // Second update
    await qTable.setQValue(stateHash, contentId, 0.6);
    entry = await qTable.get(stateHash, contentId);
    expect(entry?.visitCount).toBe(2);

    // Third update
    await qTable.setQValue(stateHash, contentId, 0.7);
    entry = await qTable.get(stateHash, contentId);
    expect(entry?.visitCount).toBe(3);
  });

  test('Epsilon-greedy action selection balances exploration/exploitation', async () => {
    const qTable = new QTable();
    const stateHash = 'state-neutral';

    // Set Q-values for different content
    await qTable.setQValue(stateHash, 'content-high', 0.9);
    await qTable.setQValue(stateHash, 'content-medium', 0.5);
    await qTable.setQValue(stateHash, 'content-low', 0.1);

    const candidates = ['content-high', 'content-medium', 'content-low'];

    // Exploitation (epsilon = 0): should always pick highest Q-value
    const exploitActions = await qTable.selectActions(stateHash, candidates, 0.0);
    expect(exploitActions[0]).toBe('content-high');

    // Exploration (epsilon = 1): should be random (can't test deterministically)
    const exploreActions = await qTable.selectActions(stateHash, candidates, 1.0);
    expect(exploreActions).toHaveLength(3);
    expect(new Set(exploreActions).size).toBe(3); // All unique
  });

  test('getBestAction returns highest Q-value', async () => {
    const qTable = new QTable();
    const stateHash = 'state-test';

    await qTable.setQValue(stateHash, 'content-a', 0.3);
    await qTable.setQValue(stateHash, 'content-b', 0.7);
    await qTable.setQValue(stateHash, 'content-c', 0.5);

    const best = await qTable.getBestAction(stateHash);
    expect(best?.contentId).toBe('content-b');
    expect(best?.qValue).toBe(0.7);
  });

  test('Statistics calculation is accurate', async () => {
    const qTable = new QTable();

    await qTable.setQValue('state-1', 'content-a', 0.5);
    await qTable.setQValue('state-1', 'content-b', 0.7);
    await qTable.setQValue('state-2', 'content-a', 0.3);

    const stats = await qTable.getStats();

    expect(stats.totalEntries).toBe(3);
    expect(stats.totalStates).toBe(2);
    expect(stats.avgVisitCount).toBe(1); // Each visited once
    expect(stats.avgQValue).toBeCloseTo((0.5 + 0.7 + 0.3) / 3);
  });

  test('Clear removes all Q-values', async () => {
    const qTable = new QTable();

    await qTable.setQValue('state-1', 'content-a', 0.5);
    await qTable.setQValue('state-2', 'content-b', 0.7);

    let stats = await qTable.getStats();
    expect(stats.totalEntries).toBe(2);

    await qTable.clear();

    stats = await qTable.getStats();
    expect(stats.totalEntries).toBe(0);
  });

  test('File is created in data directory', async () => {
    const qTable = new QTable();
    await qTable.setQValue('state-test', 'content-test', 0.5);
    qTable.flush();

    // Wait for debounced write
    await new Promise(resolve => setTimeout(resolve, 1500));

    expect(fs.existsSync(testQTableFile)).toBe(true);

    // Verify JSON format
    const content = fs.readFileSync(testQTableFile, 'utf-8');
    const parsed = JSON.parse(content);
    expect(parsed['state-test:content-test']).toBeDefined();
    expect(parsed['state-test:content-test'].qValue).toBe(0.5);
  });
});
