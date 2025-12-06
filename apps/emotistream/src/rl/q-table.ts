import { QTableEntry } from './types';

export class QTable {
  private table: Map<string, QTableEntry>;

  constructor() {
    this.table = new Map();
  }

  async get(stateHash: string, contentId: string): Promise<QTableEntry | null> {
    const key = this.buildKey(stateHash, contentId);
    return this.table.get(key) || null;
  }

  async set(entry: QTableEntry): Promise<void> {
    const key = this.buildKey(entry.stateHash, entry.contentId);
    this.table.set(key, entry);
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
