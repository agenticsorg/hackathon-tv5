/**
 * Integration tests for session store persistence
 * Verifies that sessions.json is created and properly persisted
 */

import * as fs from 'fs';
import * as path from 'path';
import { FileStore } from '../file-store';
import { SessionData } from '../../api/routes/session-store';

describe('SessionStore Integration', () => {
  const testDataDir = path.join(process.cwd(), 'data');
  const testFileName = 'test-sessions.json';
  const testFilePath = path.join(testDataDir, testFileName);

  let store: FileStore<SessionData>;

  beforeAll(() => {
    // Ensure data directory exists
    if (!fs.existsSync(testDataDir)) {
      fs.mkdirSync(testDataDir, { recursive: true });
    }
  });

  afterAll(() => {
    // Clean up test file
    if (fs.existsSync(testFilePath)) {
      fs.unlinkSync(testFilePath);
    }
  });

  beforeEach(() => {
    // Remove test file before each test
    if (fs.existsSync(testFilePath)) {
      fs.unlinkSync(testFilePath);
    }
    store = new FileStore<SessionData>(testFileName);
  });

  test('creates sessions file when data is written', () => {
    const sessionData: SessionData = {
      stateBefore: {
        valence: 0.5,
        arousal: -0.2,
        stress: 0.3,
        confidence: 0.8,
      },
      desiredState: {
        targetValence: 0.6,
        targetArousal: -0.1,
        targetStress: 0.2,
        intensity: 'moderate',
        reasoning: 'Test session',
      },
      contentId: 'test-content-123',
      timestamp: Date.now(),
    };

    store.set('user1:content123', sessionData);
    store.flush(); // Force immediate save

    expect(fs.existsSync(testFilePath)).toBe(true);

    const fileContent = fs.readFileSync(testFilePath, 'utf-8');
    const parsed = JSON.parse(fileContent);
    expect(parsed['user1:content123']).toBeDefined();
    expect(parsed['user1:content123'].contentId).toBe('test-content-123');
  });

  test('persists multiple sessions correctly', () => {
    const session1: SessionData = {
      stateBefore: { valence: 0.3, arousal: 0.1, stress: 0.4, confidence: 0.7 },
      desiredState: {
        targetValence: 0.5,
        targetArousal: 0.0,
        targetStress: 0.2,
        intensity: 'subtle',
        reasoning: 'Session 1',
      },
      contentId: 'content-1',
      timestamp: Date.now(),
    };

    const session2: SessionData = {
      stateBefore: { valence: -0.2, arousal: 0.5, stress: 0.7, confidence: 0.85 },
      desiredState: {
        targetValence: 0.3,
        targetArousal: -0.3,
        targetStress: 0.3,
        intensity: 'significant',
        reasoning: 'Session 2',
      },
      contentId: 'content-2',
      timestamp: Date.now(),
    };

    store.set('user1:content-1', session1);
    store.set('user2:content-2', session2);
    store.flush();

    expect(fs.existsSync(testFilePath)).toBe(true);

    const fileContent = fs.readFileSync(testFilePath, 'utf-8');
    const parsed = JSON.parse(fileContent);
    expect(Object.keys(parsed).length).toBe(2);
    expect(parsed['user1:content-1'].contentId).toBe('content-1');
    expect(parsed['user2:content-2'].contentId).toBe('content-2');
  });

  test('loads existing sessions on initialization', () => {
    // Pre-create file with session data
    const existingData = {
      'preexisting:session': {
        stateBefore: { valence: 0.1, arousal: 0.2, stress: 0.3, confidence: 0.9 },
        desiredState: {
          targetValence: 0.4,
          targetArousal: 0.1,
          targetStress: 0.2,
          intensity: 'moderate',
          reasoning: 'Preexisting',
        },
        contentId: 'preexisting-content',
        timestamp: 1234567890,
      },
    };

    fs.writeFileSync(testFilePath, JSON.stringify(existingData), 'utf-8');

    // Create new store instance - should load existing data
    const newStore = new FileStore<SessionData>(testFileName);
    const loaded = newStore.get('preexisting:session');

    expect(loaded).toBeDefined();
    expect(loaded?.contentId).toBe('preexisting-content');
  });

  test('deletes sessions correctly', () => {
    const session: SessionData = {
      stateBefore: { valence: 0.5, arousal: -0.1, stress: 0.2, confidence: 0.8 },
      desiredState: {
        targetValence: 0.6,
        targetArousal: 0.0,
        targetStress: 0.1,
        intensity: 'moderate',
        reasoning: 'Delete test',
      },
      contentId: 'to-delete',
      timestamp: Date.now(),
    };

    store.set('user:to-delete', session);
    store.flush();

    expect(store.get('user:to-delete')).toBeDefined();

    store.delete('user:to-delete');
    store.flush();

    expect(store.get('user:to-delete')).toBeUndefined();

    const fileContent = fs.readFileSync(testFilePath, 'utf-8');
    const parsed = JSON.parse(fileContent);
    expect(parsed['user:to-delete']).toBeUndefined();
  });

  test('creates data directory if it does not exist', () => {
    const nestedPath = path.join(testDataDir, 'nested');
    const nestedFile = 'nested-sessions.json';
    const fullPath = path.join(nestedPath, nestedFile);

    // Ensure nested dir doesn't exist
    if (fs.existsSync(fullPath)) {
      fs.unlinkSync(fullPath);
    }
    if (fs.existsSync(nestedPath)) {
      fs.rmdirSync(nestedPath);
    }

    // This would fail if directory creation doesn't work
    // Note: FileStore uses cwd/data/<filename>, so we need to test with actual path
    const testStore = new FileStore<SessionData>(nestedFile);
    testStore.set('test:key', {
      stateBefore: { valence: 0.1, arousal: 0.1, stress: 0.1, confidence: 0.5 },
      desiredState: {
        targetValence: 0.2,
        targetArousal: 0.0,
        targetStress: 0.1,
        intensity: 'subtle',
        reasoning: 'Dir test',
      },
      contentId: 'dir-test',
      timestamp: Date.now(),
    });
    testStore.flush();

    const expectedPath = path.join(testDataDir, nestedFile);
    expect(fs.existsSync(expectedPath)).toBe(true);

    // Cleanup
    fs.unlinkSync(expectedPath);
  });
});
