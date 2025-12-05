/**
 * Type definitions for Agent Ready Web Crawler SDK
 */

/**
 * SDK Configuration
 */
export interface CrawlerConfig {
  apiKey?: string;
  baseUrl?: string;
  timeout?: number;
  retryAttempts?: number;
  retryDelay?: number;
  debug?: boolean;
}

/**
 * HTTP Headers
 */
export interface Headers {
  [key: string]: string;
}

/**
 * Scraping Options
 */
export interface ScrapeOptions {
  formats?: ('markdown' | 'html' | 'links' | 'screenshot')[];
  headers?: Headers;
  onlyMainContent?: boolean;
  includeTags?: string[];
  excludeTags?: string[];
  waitFor?: number;
  timeout?: number;
  removeBase64Images?: boolean;
  mobile?: boolean;
  skipTlsVerification?: boolean;
  arwDiscovery?: boolean;
}

/**
 * Crawl Options
 */
export interface CrawlOptions extends ScrapeOptions {
  limit?: number;
  maxDepth?: number;
  allowBackwardLinks?: boolean;
  allowExternalLinks?: boolean;
  ignoreSitemap?: boolean;
  includeSubdomains?: boolean;
  webhookUrl?: string;
  machineView?: boolean;
}

/**
 * Map Options
 */
export interface MapOptions {
  search?: string;
  ignoreSitemap?: boolean;
  includeSubdomains?: boolean;
  limit?: number;
}

/**
 * Batch Operation
 */
export interface BatchOperation {
  type: 'scrape' | 'crawl' | 'map';
  url: string;
  options?: ScrapeOptions | CrawlOptions | MapOptions;
}

/**
 * Scrape Result
 */
export interface ScrapeResult {
  success: boolean;
  url: string;
  markdown?: string;
  html?: string;
  links?: string[];
  screenshot?: string;
  metadata?: PageMetadata;
  arw?: ARWMetadata;
  error?: string;
}

/**
 * Page Metadata
 */
export interface PageMetadata {
  title?: string;
  description?: string;
  language?: string;
  keywords?: string;
  robots?: string;
  ogTitle?: string;
  ogDescription?: string;
  ogImage?: string;
  ogUrl?: string;
  sourceURL?: string;
  statusCode?: number;
  pageStatusCode?: number;
  pageError?: string;
}

/**
 * ARW Metadata
 */
export interface ARWMetadata {
  discovered: boolean;
  llmsTxtUrl?: string;
  llmsTxtContent?: string;
  machineView?: MachineViewData;
  discoveryMethod?: 'wellknown' | 'root' | 'metadata' | 'none';
}

/**
 * Machine View Data
 */
export interface MachineViewData {
  sections: MachineViewSection[];
  tokens?: number;
  optimized: boolean;
}

/**
 * Machine View Section
 */
export interface MachineViewSection {
  type: string;
  content: string;
  priority?: number;
  metadata?: Record<string, any>;
}

/**
 * Crawl Status Response
 */
export interface CrawlStatus {
  status: 'scraping' | 'completed' | 'failed';
  completed: number;
  total: number;
  creditsUsed: number;
  expiresAt: string;
  next?: string;
  data?: ScrapeResult[];
}

/**
 * Crawl Response
 */
export interface CrawlResponse {
  id: string;
  url: string;
  status: 'scraping' | 'completed' | 'failed';
}

/**
 * Map Result
 */
export interface MapResult {
  success: boolean;
  links: string[];
  error?: string;
}

/**
 * Batch Result
 */
export interface BatchResult {
  success: boolean;
  results: (ScrapeResult | CrawlResponse | MapResult)[];
  errors: Array<{
    index: number;
    error: string;
  }>;
}

/**
 * WebSocket Message Types
 */
export type WebSocketMessage =
  | { type: 'status'; data: CrawlStatus }
  | { type: 'result'; data: ScrapeResult }
  | { type: 'error'; data: { error: string } }
  | { type: 'complete'; data: { total: number; creditsUsed: number } };

/**
 * Stream Options
 */
export interface StreamOptions {
  onStatus?: (status: CrawlStatus) => void;
  onResult?: (result: ScrapeResult) => void;
  onError?: (error: Error) => void;
  onComplete?: () => void;
}

/**
 * Error Response
 */
export interface ErrorResponse {
  error: string;
  message?: string;
  statusCode?: number;
  details?: any;
}

/**
 * API Response
 */
export interface APIResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}
