/**
 * Crawl endpoint wrapper
 */

import { HttpClient } from '../utils/http';
import { WebSocketClient } from '../utils/websocket';
import {
  CrawlOptions,
  CrawlResponse,
  CrawlStatus,
  StreamOptions,
  APIResponse,
  CrawlerConfig,
} from '../types';
import { ValidationError } from '../utils/errors';

export class CrawlEndpoint {
  private wsClient: WebSocketClient;

  constructor(
    private http: HttpClient,
    private config: CrawlerConfig
  ) {
    this.wsClient = new WebSocketClient(config);
  }

  /**
   * Start a multi-page crawl
   */
  async start(url: string, options?: CrawlOptions): Promise<CrawlResponse> {
    this.validateUrl(url);

    const response = await this.http.post<APIResponse<CrawlResponse>>(
      '/crawl',
      {
        url,
        ...options,
      }
    );

    if (!response.success || !response.data) {
      throw new ValidationError(
        response.error || 'Crawl start failed',
        response
      );
    }

    return response.data;
  }

  /**
   * Get crawl status and results
   */
  async getStatus(crawlId: string): Promise<CrawlStatus> {
    if (!crawlId) {
      throw new ValidationError('Crawl ID is required');
    }

    const response = await this.http.get<APIResponse<CrawlStatus>>(
      `/crawl/${crawlId}`
    );

    if (!response.success || !response.data) {
      throw new ValidationError(
        response.error || 'Failed to get crawl status',
        response
      );
    }

    return response.data;
  }

  /**
   * Cancel a running crawl
   */
  async cancel(crawlId: string): Promise<void> {
    if (!crawlId) {
      throw new ValidationError('Crawl ID is required');
    }

    const response = await this.http.delete<APIResponse<void>>(
      `/crawl/${crawlId}`
    );

    if (!response.success) {
      throw new ValidationError(
        response.error || 'Failed to cancel crawl',
        response
      );
    }
  }

  /**
   * Stream crawl results in real-time via WebSocket
   */
  async stream(
    crawlId: string,
    options: StreamOptions
  ): Promise<void> {
    if (!crawlId) {
      throw new ValidationError('Crawl ID is required');
    }

    await this.wsClient.connect(crawlId, options);
  }

  /**
   * Start crawl and wait for completion
   */
  async crawlAndWait(
    url: string,
    options?: CrawlOptions & { pollInterval?: number }
  ): Promise<CrawlStatus> {
    const { pollInterval = 2000, ...crawlOptions } = options || {};

    // Start crawl
    const crawl = await this.start(url, crawlOptions);

    // Poll for completion
    while (true) {
      const status = await this.getStatus(crawl.id);

      if (status.status === 'completed') {
        return status;
      }

      if (status.status === 'failed') {
        throw new ValidationError('Crawl failed');
      }

      // Wait before next poll
      await this.sleep(pollInterval);
    }
  }

  /**
   * Close WebSocket connection
   */
  closeStream(): void {
    this.wsClient.close();
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

  /**
   * Sleep helper
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
