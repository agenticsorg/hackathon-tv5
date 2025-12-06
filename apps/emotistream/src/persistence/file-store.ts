import * as fs from 'fs';
import * as path from 'path';

/**
 * Simple file-based key-value store with debounced persistence
 * @template T - Type of values stored
 */
export class FileStore<T> {
  private data: Map<string, T> = new Map();
  private filePath: string;
  private saveDebounce: NodeJS.Timeout | null = null;
  private readonly DEBOUNCE_MS = 1000;

  constructor(filename: string) {
    this.filePath = path.join(process.cwd(), 'data', filename);
    this.load();
  }

  /**
   * Load data from file on initialization
   */
  private load(): void {
    try {
      if (fs.existsSync(this.filePath)) {
        const content = fs.readFileSync(this.filePath, 'utf-8');
        const parsed = JSON.parse(content);
        this.data = new Map(Object.entries(parsed));
      }
    } catch (e) {
      console.warn(`Could not load store from ${this.filePath}:`, e);
      this.data = new Map();
    }
  }

  /**
   * Debounced save to file (1 second delay to batch writes)
   */
  private save(): void {
    if (this.saveDebounce) {
      clearTimeout(this.saveDebounce);
    }

    this.saveDebounce = setTimeout(() => {
      try {
        const dir = path.dirname(this.filePath);
        if (!fs.existsSync(dir)) {
          fs.mkdirSync(dir, { recursive: true });
        }

        const dataObj = Object.fromEntries(this.data);
        fs.writeFileSync(this.filePath, JSON.stringify(dataObj, null, 2), 'utf-8');
      } catch (e) {
        console.error(`Failed to save store to ${this.filePath}:`, e);
      }
    }, this.DEBOUNCE_MS);
  }

  /**
   * Force immediate save (useful for shutdown)
   */
  public flush(): void {
    if (this.saveDebounce) {
      clearTimeout(this.saveDebounce);
      this.saveDebounce = null;
    }

    try {
      const dir = path.dirname(this.filePath);
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }

      const dataObj = Object.fromEntries(this.data);
      fs.writeFileSync(this.filePath, JSON.stringify(dataObj, null, 2), 'utf-8');
    } catch (e) {
      console.error(`Failed to flush store to ${this.filePath}:`, e);
    }
  }

  get(key: string): T | undefined {
    return this.data.get(key);
  }

  set(key: string, value: T): void {
    this.data.set(key, value);
    this.save();
  }

  delete(key: string): boolean {
    const result = this.data.delete(key);
    if (result) this.save();
    return result;
  }

  entries(): IterableIterator<[string, T]> {
    return this.data.entries();
  }

  values(): IterableIterator<T> {
    return this.data.values();
  }

  keys(): IterableIterator<string> {
    return this.data.keys();
  }

  size(): number {
    return this.data.size;
  }

  clear(): void {
    this.data.clear();
    this.save();
  }
}
