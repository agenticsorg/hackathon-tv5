/* tslint:disable */
/* eslint-disable */
/**
 * Initialize logging for WASM
 */
export function init(): void;
/**
 * Log a message to the browser console
 */
export function log(message: string): void;
/**
 * WASM wrapper for ReasoningBank
 */
export class ReasoningBankWasm {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Create a new ReasoningBank instance
   */
  constructor(db_name?: string | null);
  /**
   * Store a reasoning pattern
   */
  storePattern(pattern_json: string): Promise<string>;
  /**
   * Retrieve a pattern by ID
   */
  getPattern(id: string): Promise<string>;
  /**
   * Search patterns by category
   */
  searchByCategory(category: string, limit: number): Promise<string>;
  /**
   * Find similar patterns
   */
  findSimilar(task_description: string, task_category: string, top_k: number): Promise<string>;
  /**
   * Get storage statistics
   */
  getStats(): Promise<string>;
}
