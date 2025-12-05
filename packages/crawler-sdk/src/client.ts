/**
 * Main Crawler SDK Client
 */

import { HttpClient } from './utils/http';
import { ScrapeEndpoint, CrawlEndpoint, MapEndpoint, BatchEndpoint } from './endpoints';
import { ARWDiscovery, MachineView } from './arw';
import {
  CrawlerConfig,
  ScrapeOptions,
  ScrapeResult,
  CrawlOptions,
  CrawlResponse,
  CrawlStatus,
  MapOptions,
  MapResult,
  BatchOperation,
  BatchResult,
  StreamOptions,
  ARWMetadata,
  MachineViewData,
} from './types';
import { ValidationError } from './utils/errors';

/**
 * Main SDK Client for Agent Ready Web Crawler
 *
 * @example
 * ```typescript
 * const client = new CrawlerClient({
 *   apiKey: 'your-api-key',
 *   baseUrl: 'https://api.example.com'
 * });
 *
 * // Scrape a single page
 * const result = await client.scrape('https://example.com');
 *
 * // Start a crawl
 * const crawl = await client.crawl('https://example.com', {
 *   maxDepth: 3,
 *   arwDiscovery: true
 * });
 * ```
 */
export class CrawlerClient {
  private http: HttpClient;
  private scrapeEndpoint: ScrapeEndpoint;
  private crawlEndpoint: CrawlEndpoint;
  private mapEndpoint: MapEndpoint;
  private batchEndpoint: BatchEndpoint;
  private arwDiscovery: ARWDiscovery;
  private machineView: MachineView;

  constructor(config: CrawlerConfig = {}) {
    // Initialize HTTP client
    this.http = new HttpClient(config);

    // Initialize endpoints
    this.scrapeEndpoint = new ScrapeEndpoint(this.http);
    this.crawlEndpoint = new CrawlEndpoint(this.http, config);
    this.mapEndpoint = new MapEndpoint(this.http);
    this.batchEndpoint = new BatchEndpoint(this.http);

    // Initialize ARW utilities
    this.arwDiscovery = new ARWDiscovery(this.http);
    this.machineView = new MachineView(this.http);
  }

  // ==================== Scraping Methods ====================

  /**
   * Scrape a single URL
   *
   * @param url - The URL to scrape
   * @param options - Scraping options
   * @returns Scrape result with content and metadata
   *
   * @example
   * ```typescript
   * const result = await client.scrape('https://example.com', {
   *   formats: ['markdown', 'html'],
   *   onlyMainContent: true
   * });
   * console.log(result.markdown);
   * ```
   */
  async scrape(url: string, options?: ScrapeOptions): Promise<ScrapeResult> {
    return this.scrapeEndpoint.scrape(url, options);
  }

  /**
   * Scrape multiple URLs concurrently
   *
   * @param urls - Array of URLs to scrape
   * @param options - Scraping options
   * @returns Array of scrape results
   *
   * @example
   * ```typescript
   * const results = await client.scrapeMany([
   *   'https://example.com/page1',
   *   'https://example.com/page2'
   * ]);
   * ```
   */
  async scrapeMany(
    urls: string[],
    options?: ScrapeOptions
  ): Promise<ScrapeResult[]> {
    return this.scrapeEndpoint.scrapeMany(urls, options);
  }

  // ==================== Crawling Methods ====================

  /**
   * Start a multi-page crawl
   *
   * @param url - The starting URL
   * @param options - Crawl options
   * @returns Crawl response with ID and status
   *
   * @example
   * ```typescript
   * const crawl = await client.crawl('https://example.com', {
   *   maxDepth: 3,
   *   limit: 100,
   *   machineView: true
   * });
   * console.log('Crawl ID:', crawl.id);
   * ```
   */
  async crawl(url: string, options?: CrawlOptions): Promise<CrawlResponse> {
    return this.crawlEndpoint.start(url, options);
  }

  /**
   * Get crawl status and results
   *
   * @param crawlId - The crawl ID
   * @returns Current crawl status with completed results
   *
   * @example
   * ```typescript
   * const status = await client.getCrawlStatus(crawl.id);
   * console.log(`Progress: ${status.completed}/${status.total}`);
   * ```
   */
  async getCrawlStatus(crawlId: string): Promise<CrawlStatus> {
    return this.crawlEndpoint.getStatus(crawlId);
  }

  /**
   * Cancel a running crawl
   *
   * @param crawlId - The crawl ID
   *
   * @example
   * ```typescript
   * await client.cancelCrawl(crawl.id);
   * ```
   */
  async cancelCrawl(crawlId: string): Promise<void> {
    return this.crawlEndpoint.cancel(crawlId);
  }

  /**
   * Start crawl and wait for completion
   *
   * @param url - The starting URL
   * @param options - Crawl options with polling interval
   * @returns Final crawl status with all results
   *
   * @example
   * ```typescript
   * const results = await client.crawlAndWait('https://example.com', {
   *   maxDepth: 2,
   *   pollInterval: 3000 // Check every 3 seconds
   * });
   * console.log('Total pages:', results.data.length);
   * ```
   */
  async crawlAndWait(
    url: string,
    options?: CrawlOptions & { pollInterval?: number }
  ): Promise<CrawlStatus> {
    return this.crawlEndpoint.crawlAndWait(url, options);
  }

  /**
   * Stream crawl results in real-time via WebSocket
   *
   * @param crawlId - The crawl ID
   * @param options - Stream event handlers
   *
   * @example
   * ```typescript
   * await client.streamCrawl(crawl.id, {
   *   onStatus: (status) => console.log('Progress:', status.completed),
   *   onResult: (result) => console.log('Page:', result.url),
   *   onComplete: () => console.log('Crawl complete!')
   * });
   * ```
   */
  async streamCrawl(
    crawlId: string,
    options: StreamOptions
  ): Promise<void> {
    return this.crawlEndpoint.stream(crawlId, options);
  }

  /**
   * Close WebSocket stream connection
   */
  closeStream(): void {
    this.crawlEndpoint.closeStream();
  }

  // ==================== Mapping Methods ====================

  /**
   * Generate a site map
   *
   * @param url - The site URL
   * @param options - Map generation options
   * @returns Map result with discovered links
   *
   * @example
   * ```typescript
   * const map = await client.map('https://example.com', {
   *   includeSubdomains: false,
   *   limit: 1000
   * });
   * console.log('Found links:', map.links.length);
   * ```
   */
  async map(url: string, options?: MapOptions): Promise<MapResult> {
    return this.mapEndpoint.generate(url, options);
  }

  // ==================== Batch Methods ====================

  /**
   * Execute multiple operations in batch
   *
   * @param operations - Array of batch operations
   * @returns Batch result with all operation results
   *
   * @example
   * ```typescript
   * const result = await client.batch([
   *   { type: 'scrape', url: 'https://example.com' },
   *   { type: 'map', url: 'https://example.com' }
   * ]);
   * ```
   */
  async batch(operations: BatchOperation[]): Promise<BatchResult> {
    return this.batchEndpoint.execute(operations);
  }

  // ==================== ARW Methods ====================

  /**
   * Discover ARW llms.txt file
   *
   * @param url - The URL to check
   * @returns ARW metadata including llms.txt content
   *
   * @example
   * ```typescript
   * const arw = await client.discoverARW('https://docs.example.com');
   * if (arw.discovered) {
   *   console.log('llms.txt URL:', arw.llmsTxtUrl);
   * }
   * ```
   */
  async discoverARW(url: string): Promise<ARWMetadata> {
    return this.arwDiscovery.discover(url);
  }

  /**
   * Check if URL has ARW support
   *
   * @param url - The URL to check
   * @returns True if ARW is supported
   *
   * @example
   * ```typescript
   * if (await client.hasARWSupport('https://docs.example.com')) {
   *   console.log('This site supports ARW!');
   * }
   * ```
   */
  async hasARWSupport(url: string): Promise<boolean> {
    return this.arwDiscovery.hasSupport(url);
  }

  /**
   * Get llms.txt content
   *
   * @param url - The URL to check
   * @returns llms.txt content or null
   *
   * @example
   * ```typescript
   * const content = await client.getLlmsTxt('https://docs.example.com');
   * if (content) {
   *   console.log(content);
   * }
   * ```
   */
  async getLlmsTxt(url: string): Promise<string | null> {
    return this.arwDiscovery.getLlmsTxt(url);
  }

  /**
   * Generate machine-optimized view
   *
   * @param url - The URL to process
   * @param options - Machine view options
   * @returns Optimized machine view data
   *
   * @example
   * ```typescript
   * const view = await client.generateMachineView('https://example.com', {
   *   maxTokens: 4000,
   *   prioritySections: ['introduction', 'api']
   * });
   * ```
   */
  async generateMachineView(
    url: string,
    options?: {
      maxTokens?: number;
      prioritySections?: string[];
    }
  ): Promise<MachineViewData> {
    return this.machineView.generate(url, options);
  }

  // ==================== Utility Methods ====================

  /**
   * Update API key
   *
   * @param apiKey - New API key
   */
  setApiKey(apiKey: string): void {
    this.http.setApiKey(apiKey);
  }

  /**
   * Get access to ARW Discovery utilities
   */
  get arw(): ARWDiscovery {
    return this.arwDiscovery;
  }

  /**
   * Get access to Machine View utilities
   */
  get machine(): MachineView {
    return this.machineView;
  }

  /**
   * Get the underlying HTTP client (advanced usage)
   */
  getHttpClient(): HttpClient {
    return this.http;
  }
}

/**
 * Export default client
 */
export default CrawlerClient;
