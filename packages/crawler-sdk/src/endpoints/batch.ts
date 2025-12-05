/**
 * Batch operations endpoint wrapper
 */

import { HttpClient } from '../utils/http';
import { BatchOperation, BatchResult, APIResponse } from '../types';
import { ValidationError } from '../utils/errors';

export class BatchEndpoint {
  constructor(private http: HttpClient) {}

  /**
   * Execute multiple operations in batch
   */
  async execute(operations: BatchOperation[]): Promise<BatchResult> {
    if (!operations || !operations.length) {
      throw new ValidationError('Operations array cannot be empty');
    }

    // Validate all operations
    operations.forEach((op, index) => {
      this.validateOperation(op, index);
    });

    const response = await this.http.post<APIResponse<BatchResult>>(
      '/batch',
      { operations }
    );

    if (!response.success || !response.data) {
      throw new ValidationError(
        response.error || 'Batch execution failed',
        response
      );
    }

    return response.data;
  }

  /**
   * Validate a single batch operation
   */
  private validateOperation(op: BatchOperation, index: number): void {
    if (!op.type) {
      throw new ValidationError(
        `Operation at index ${index}: type is required`
      );
    }

    if (!['scrape', 'crawl', 'map'].includes(op.type)) {
      throw new ValidationError(
        `Operation at index ${index}: invalid type '${op.type}'`
      );
    }

    if (!op.url || typeof op.url !== 'string') {
      throw new ValidationError(
        `Operation at index ${index}: URL must be a non-empty string`
      );
    }

    try {
      new URL(op.url);
    } catch {
      throw new ValidationError(
        `Operation at index ${index}: invalid URL format '${op.url}'`
      );
    }
  }
}
