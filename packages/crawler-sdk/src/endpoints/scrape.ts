/**
 * Scrape endpoint wrapper
 */

import { HttpClient } from '../utils/http';
import { ScrapeOptions, ScrapeResult, APIResponse } from '../types';
import { ValidationError } from '../utils/errors';

export class ScrapeEndpoint {
  constructor(private http: HttpClient) {}

  /**
   * Scrape a single URL
   */
  async scrape(url: string, options?: ScrapeOptions): Promise<ScrapeResult> {
    this.validateUrl(url);

    const response = await this.http.post<APIResponse<ScrapeResult>>(
      '/scrape',
      {
        url,
        ...options,
      }
    );

    if (!response.success || !response.data) {
      throw new ValidationError(
        response.error || 'Scrape failed',
        response
      );
    }

    return response.data;
  }

  /**
   * Scrape multiple URLs concurrently
   */
  async scrapeMany(
    urls: string[],
    options?: ScrapeOptions
  ): Promise<ScrapeResult[]> {
    if (!urls.length) {
      throw new ValidationError('URLs array cannot be empty');
    }

    urls.forEach(url => this.validateUrl(url));

    const response = await this.http.post<APIResponse<ScrapeResult[]>>(
      '/scrape/batch',
      {
        urls,
        ...options,
      }
    );

    if (!response.success || !response.data) {
      throw new ValidationError(
        response.error || 'Batch scrape failed',
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
