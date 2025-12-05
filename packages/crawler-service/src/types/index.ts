import { z } from 'zod';

// Core Types
export interface CrawlOptions {
  url: string;
  maxDepth?: number;
  maxPages?: number;
  respectRobotsTxt?: boolean;
  followSitemap?: boolean;
  extractionMode?: 'html' | 'text' | 'markdown' | 'machine-view';
  userAgent?: string;
  timeout?: number;
  waitFor?: string | number;
  screenshot?: boolean;
  headers?: Record<string, string>;
  cookies?: Array<{ name: string; value: string; domain?: string }>;
}

export interface ScrapeOptions {
  url: string;
  extractionMode?: 'html' | 'text' | 'markdown' | 'machine-view';
  userAgent?: string;
  timeout?: number;
  waitFor?: string | number;
  screenshot?: boolean;
  headers?: Record<string, string>;
  cookies?: Array<{ name: string; value: string; domain?: string }>;
}

export interface CrawlResult {
  url: string;
  title?: string;
  content: string;
  metadata?: Record<string, any>;
  links?: string[];
  timestamp: Date;
  statusCode?: number;
  contentType?: string;
  machineView?: MachineView;
}

export interface MachineView {
  title: string;
  description?: string;
  mainContent: string;
  sections: Section[];
  links: Link[];
  metadata: Metadata;
  schema?: any;
}

export interface Section {
  type: 'heading' | 'paragraph' | 'list' | 'code' | 'table';
  content: string;
  level?: number;
  items?: string[];
}

export interface Link {
  href: string;
  text: string;
  rel?: string;
  type?: 'internal' | 'external';
}

export interface Metadata {
  title?: string;
  description?: string;
  author?: string;
  publishedDate?: string;
  modifiedDate?: string;
  keywords?: string[];
  language?: string;
  canonicalUrl?: string;
  ogData?: Record<string, string>;
}

export interface ARWDiscoveryResult {
  llmsTxt?: LlmsTxtData;
  robotsTxt?: RobotsTxtData;
  sitemap?: SitemapData;
}

export interface LlmsTxtData {
  url: string;
  content: string;
  sections: Array<{
    title: string;
    content: string;
    links?: string[];
  }>;
  metadata?: Record<string, any>;
  parsedAt: Date;
}

export interface RobotsTxtData {
  url: string;
  content: string;
  rules: Array<{
    userAgent: string;
    allow?: string[];
    disallow?: string[];
  }>;
  sitemaps?: string[];
  crawlDelay?: number;
  parsedAt: Date;
}

export interface SitemapData {
  url: string;
  urls: Array<{
    loc: string;
    lastmod?: string;
    changefreq?: string;
    priority?: number;
  }>;
  parsedAt: Date;
}

export interface CrawlJob {
  id: string;
  url: string;
  options: CrawlOptions;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  progress: number;
  results?: CrawlResult[];
  error?: string;
  createdAt: Date;
  updatedAt: Date;
  completedAt?: Date;
}

export interface BatchJob {
  id: string;
  urls: string[];
  options: CrawlOptions;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  progress: number;
  results: Record<string, CrawlResult>;
  errors: Record<string, string>;
  createdAt: Date;
  updatedAt: Date;
  completedAt?: Date;
}

// Zod Schemas for Validation
export const ScrapeRequestSchema = z.object({
  url: z.string().url(),
  extractionMode: z.enum(['html', 'text', 'markdown', 'machine-view']).optional(),
  userAgent: z.string().optional(),
  timeout: z.number().positive().optional(),
  waitFor: z.union([z.string(), z.number()]).optional(),
  screenshot: z.boolean().optional(),
  headers: z.record(z.string()).optional(),
  cookies: z.array(z.object({
    name: z.string(),
    value: z.string(),
    domain: z.string().optional()
  })).optional()
});

export const CrawlRequestSchema = ScrapeRequestSchema.extend({
  maxDepth: z.number().int().positive().max(10).optional(),
  maxPages: z.number().int().positive().max(1000).optional(),
  respectRobotsTxt: z.boolean().optional(),
  followSitemap: z.boolean().optional()
});

export const BatchRequestSchema = z.object({
  urls: z.array(z.string().url()).min(1).max(100),
  options: CrawlRequestSchema.optional()
});

export const MapRequestSchema = z.object({
  url: z.string().url(),
  maxDepth: z.number().int().positive().max(5).optional(),
  respectRobotsTxt: z.boolean().optional()
});

// Error Types
export class CrawlerError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode: number = 500,
    public details?: any
  ) {
    super(message);
    this.name = 'CrawlerError';
  }
}

export class ValidationError extends CrawlerError {
  constructor(message: string, details?: any) {
    super(message, 'VALIDATION_ERROR', 400, details);
    this.name = 'ValidationError';
  }
}

export class NetworkError extends CrawlerError {
  constructor(message: string, details?: any) {
    super(message, 'NETWORK_ERROR', 503, details);
    this.name = 'NetworkError';
  }
}

export class RateLimitError extends CrawlerError {
  constructor(message: string, details?: any) {
    super(message, 'RATE_LIMIT_ERROR', 429, details);
    this.name = 'RateLimitError';
  }
}
