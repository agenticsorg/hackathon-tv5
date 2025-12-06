import { QTableEntry } from './types.js';
import { FileStore } from '../persistence/file-store.js';

export class QTable {
  private table: Map<string, QTableEntry>;
  private store: FileStore<QTableEntry>;
  private readonly PERSISTENCE_FILE = 'qtable.json';

  constructor() {
    this.table = new Map();
    this.store = new FileStore<QTableEntry>(this.PERSISTENCE_FILE);
    this.loadFromStore();
  }

  /**
   * Load Q-table entries from persistent storage
   */
  private loadFromStore(): void {
    for (const [key, entry] of this.store.entries()) {
      this.table.set(key, entry);
    }
    console.log(`📊 Loaded ${this.table.size} Q-table entries from storage`);
  }

  /**
   * Force save to disk (useful for shutdown)
   */
  flush(): void {
    this.store.flush();
  }

  async get(stateHash: string, contentId: string): Promise<QTableEntry | null> {
    const key = this.buildKey(stateHash, contentId);
    return this.table.get(key) || null;
  }

  async set(entry: QTableEntry): Promise<void> {
    const key = this.buildKey(entry.stateHash, entry.contentId);
    this.table.set(key, entry);
    // Persist to file store
    this.store.set(key, entry);
  }

  async updateQValue(stateHash: string, contentId: string, newValue: number): Promise<void> {
    const existing = await this.get(stateHash, contentId);

    if (existing) {
      existing.qValue = newValue;
      existing.visitCount++;
      existing.lastUpdated = Date.now();
      await this.set(existing);
    } else {
      const newEntry: QTableEntry = {
        stateHash,
        contentId,
        qValue: newValue,
        visitCount: 1,
        lastUpdated: Date.now()
      };
      await this.set(newEntry);
    }
  }

  async getStateActions(stateHash: string): Promise<QTableEntry[]> {
    const entries: QTableEntry[] = [];

    for (const [key, entry] of this.table.entries()) {
      if (entry.stateHash === stateHash) {
        entries.push(entry);
      }
    }

    return entries;
  }

  private buildKey(stateHash: string, contentId: string): string {
    return `${stateHash}:${contentId}`;
  }
}
