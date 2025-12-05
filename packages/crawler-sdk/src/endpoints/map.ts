/**
 * Map endpoint wrapper
 */

import { HttpClient } from '../utils/http';
import { MapOptions, MapResult, APIResponse } from '../types';
import { ValidationError } from '../utils/errors';

export class MapEndpoint {
  constructor(private http: HttpClient) {}

  /**
   * Generate a site map
   */
  async generate(url: string, options?: MapOptions): Promise<MapResult> {
    this.validateUrl(url);

    const response = await this.http.post<APIResponse<MapResult>>(
      '/map',
      {
        url,
        ...options,
      }
    );

    if (!response.success || !response.data) {
      throw new ValidationError(
        response.error || 'Map generation failed',
        response
      );
    }

    return response.data;
  }

  /**
   * Validate URL format
   */
  private validateUrl(url: string): void {
    if (!url || typeof url !== 'string') {
      throw new ValidationError('URL must be a non-empty string');
    }

    try {
      new URL(url);
    } catch {
      throw new ValidationError(`Invalid URL format: ${url}`);
    }
  }
}
