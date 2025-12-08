/**
 * Learning Persistence Layer
 *
 * Stores learned models locally using file system or IndexedDB (browser)
 * Enables on-device learning that persists across sessions
 */

import { existsSync, readFileSync, writeFileSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { PreferenceLearningSystem } from './preference-learning.js';
import { ContentMetadata, UserPreference, ViewingSession, LearningStats } from './types.js';

const DEFAULT_DATA_DIR = '.samsung-tv-learning';
const MODEL_FILE = 'model.json';
const CONTENT_FILE = 'content-library.json';
const SESSIONS_FILE = 'sessions.json';

interface StoredModel {
  version: string;
  timestamp: string;
  model: ReturnType<PreferenceLearningSystem['exportModel']>;
}

interface StoredContent {
  version: string;
  timestamp: string;
  content: ContentMetadata[];
}

interface StoredSessions {
  version: string;
  timestamp: string;
  sessions: ViewingSession[];
}

/**
 * Learning Persistence Manager
 * Handles saving and loading of learned models
 */
export class LearningPersistence {
  private dataDir: string;
  private modelPath: string;
  private contentPath: string;
  private sessionsPath: string;

  constructor(dataDir?: string) {
    // Use home directory for persistence
    const homeDir = process.env.HOME || process.env.USERPROFILE || '.';
    this.dataDir = dataDir || join(homeDir, DEFAULT_DATA_DIR);
    this.modelPath = join(this.dataDir, MODEL_FILE);
    this.contentPath = join(this.dataDir, CONTENT_FILE);
    this.sessionsPath = join(this.dataDir, SESSIONS_FILE);

    // Ensure data directory exists
    this.ensureDataDir();
  }

  private ensureDataDir(): void {
    if (!existsSync(this.dataDir)) {
      mkdirSync(this.dataDir, { recursive: true });
    }
  }

  /**
   * Save the learned model to disk
   */
  saveModel(learner: PreferenceLearningSystem): void {
    const stored: StoredModel = {
      version: '1.0.0',
      timestamp: new Date().toISOString(),
      model: learner.exportModel(),
    };

    writeFileSync(this.modelPath, JSON.stringify(stored, null, 2), 'utf-8');
  }

  /**
   * Load learned model from disk
   */
  loadModel(learner: PreferenceLearningSystem): boolean {
    if (!existsSync(this.modelPath)) {
      return false;
    }

    try {
      const data = readFileSync(this.modelPath, 'utf-8');
      const stored: StoredModel = JSON.parse(data);
      learner.importModel(stored.model);
      return true;
    } catch (error) {
      console.error('Failed to load model:', error);
      return false;
    }
  }

  /**
   * Save content library to disk
   */
  saveContentLibrary(content: ContentMetadata[]): void {
    const stored: StoredContent = {
      version: '1.0.0',
      timestamp: new Date().toISOString(),
      content,
    };

    writeFileSync(this.contentPath, JSON.stringify(stored, null, 2), 'utf-8');
  }

  /**
   * Load content library from disk
   */
  loadContentLibrary(): ContentMetadata[] | null {
    if (!existsSync(this.contentPath)) {
      return null;
    }

    try {
      const data = readFileSync(this.contentPath, 'utf-8');
      const stored: StoredContent = JSON.parse(data);
      return stored.content;
    } catch (error) {
      console.error('Failed to load content library:', error);
      return null;
    }
  }

  /**
   * Save viewing sessions to disk
   */
  saveSessions(sessions: ViewingSession[]): void {
    const stored: StoredSessions = {
      version: '1.0.0',
      timestamp: new Date().toISOString(),
      sessions,
    };

    writeFileSync(this.sessionsPath, JSON.stringify(stored, null, 2), 'utf-8');
  }

  /**
   * Load viewing sessions from disk
   */
  loadSessions(): ViewingSession[] | null {
    if (!existsSync(this.sessionsPath)) {
      return null;
    }

    try {
      const data = readFileSync(this.sessionsPath, 'utf-8');
      const stored: StoredSessions = JSON.parse(data);
      return stored.sessions;
    } catch (error) {
      console.error('Failed to load sessions:', error);
      return null;
    }
  }

  /**
   * Check if model exists
   */
  hasModel(): boolean {
    return existsSync(this.modelPath);
  }

  /**
   * Get model age in hours
   */
  getModelAge(): number | null {
    if (!existsSync(this.modelPath)) {
      return null;
    }

    try {
      const data = readFileSync(this.modelPath, 'utf-8');
      const stored: StoredModel = JSON.parse(data);
      const modelTime = new Date(stored.timestamp).getTime();
      const now = Date.now();
      return (now - modelTime) / (1000 * 60 * 60);
    } catch {
      return null;
    }
  }

  /**
   * Clear all stored data
   */
  clearAll(): void {
    if (existsSync(this.modelPath)) {
      const fs = require('fs');
      fs.unlinkSync(this.modelPath);
    }
    if (existsSync(this.contentPath)) {
      const fs = require('fs');
      fs.unlinkSync(this.contentPath);
    }
    if (existsSync(this.sessionsPath)) {
      const fs = require('fs');
      fs.unlinkSync(this.sessionsPath);
    }
  }

  /**
   * Get storage stats
   */
  getStorageStats(): {
    modelExists: boolean;
    modelAge: number | null;
    contentCount: number;
    sessionCount: number;
    totalSize: number;
  } {
    let totalSize = 0;
    let contentCount = 0;
    let sessionCount = 0;

    if (existsSync(this.modelPath)) {
      const stats = require('fs').statSync(this.modelPath);
      totalSize += stats.size;
    }

    if (existsSync(this.contentPath)) {
      const stats = require('fs').statSync(this.contentPath);
      totalSize += stats.size;
      try {
        const content = this.loadContentLibrary();
        contentCount = content?.length || 0;
      } catch {}
    }

    if (existsSync(this.sessionsPath)) {
      const stats = require('fs').statSync(this.sessionsPath);
      totalSize += stats.size;
      try {
        const sessions = this.loadSessions();
        sessionCount = sessions?.length || 0;
      } catch {}
    }

    return {
      modelExists: this.hasModel(),
      modelAge: this.getModelAge(),
      contentCount,
      sessionCount,
      totalSize,
    };
  }

  /**
   * Get data directory path
   */
  getDataDir(): string {
    return this.dataDir;
  }
}

/**
 * Browser-compatible IndexedDB persistence
 * (For future browser/WebAssembly usage)
 * Note: This class is only usable in browser environments with IndexedDB support
 */
export class IndexedDBPersistence {
  private dbName: string;
  private dbVersion: number;
  private db: unknown = null;

  constructor(dbName: string = 'samsung-tv-learning', version: number = 1) {
    this.dbName = dbName;
    this.dbVersion = version;
  }

  async init(): Promise<void> {
    // Check if we're in a browser environment with IndexedDB
    if (typeof globalThis !== 'undefined' && 'indexedDB' in globalThis) {
      const idb = (globalThis as Record<string, unknown>).indexedDB as {
        open: (name: string, version: number) => {
          onerror: (() => void) | null;
          onsuccess: (() => void) | null;
          onupgradeneeded: ((event: { target: { result: unknown } }) => void) | null;
          result: unknown;
          error: Error | null;
        };
      };

      return new Promise((resolve, reject) => {
        const request = idb.open(this.dbName, this.dbVersion);

        request.onerror = () => reject(request.error);
        request.onsuccess = () => {
          this.db = request.result;
          resolve();
        };

        request.onupgradeneeded = (event: { target: { result: unknown } }) => {
          const db = event.target.result as {
            objectStoreNames: { contains: (name: string) => boolean };
            createObjectStore: (name: string, options: { keyPath: string }) => void;
          };

          // Create object stores
          if (!db.objectStoreNames.contains('model')) {
            db.createObjectStore('model', { keyPath: 'id' });
          }
          if (!db.objectStoreNames.contains('content')) {
            db.createObjectStore('content', { keyPath: 'id' });
          }
          if (!db.objectStoreNames.contains('sessions')) {
            db.createObjectStore('sessions', { keyPath: 'id' });
          }
        };
      });
    }
    throw new Error('IndexedDB not available in this environment');
  }

  async saveModel(learner: PreferenceLearningSystem): Promise<void> {
    if (!this.db) await this.init();

    const db = this.db as {
      transaction: (store: string, mode: string) => {
        objectStore: (name: string) => {
          put: (data: unknown) => {
            onsuccess: (() => void) | null;
            onerror: (() => void) | null;
            error: Error | null;
          };
        };
      };
    };

    const tx = db.transaction('model', 'readwrite');
    const store = tx.objectStore('model');

    await new Promise<void>((resolve, reject) => {
      const request = store.put({
        id: 'main',
        timestamp: new Date().toISOString(),
        model: learner.exportModel(),
      });
      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  async loadModel(learner: PreferenceLearningSystem): Promise<boolean> {
    if (!this.db) await this.init();

    const db = this.db as {
      transaction: (store: string, mode: string) => {
        objectStore: (name: string) => {
          get: (id: string) => {
            onsuccess: (() => void) | null;
            onerror: (() => void) | null;
            result: { model: ReturnType<PreferenceLearningSystem['exportModel']> } | null;
            error: Error | null;
          };
        };
      };
    };

    const tx = db.transaction('model', 'readonly');
    const store = tx.objectStore('model');

    return new Promise((resolve, reject) => {
      const request = store.get('main');
      request.onsuccess = () => {
        if (request.result) {
          learner.importModel(request.result.model);
          resolve(true);
        } else {
          resolve(false);
        }
      };
      request.onerror = () => reject(request.error);
    });
  }

  async close(): Promise<void> {
    if (this.db) {
      const db = this.db as { close: () => void };
      db.close();
      this.db = null;
    }
  }
}
