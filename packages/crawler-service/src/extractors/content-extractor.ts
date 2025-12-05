import { load } from 'cheerio';
import { launch, Browser, Page } from 'puppeteer';
import { fetch } from 'undici';
import { createChildLogger } from '../utils/logger.js';
import { config } from '../utils/config.js';
import type { ScrapeOptions, CrawlResult } from '../types/index.js';

const logger = createChildLogger('content-extractor');

export class ContentExtractor {
  private browser: Browser | null = null;
  private userAgent: string;

  constructor(userAgent?: string) {
    this.userAgent = userAgent || config.crawler.userAgent;
  }

  /**
   * Extract content from a URL
   */
  async extract(url: string, options: ScrapeOptions = { url }): Promise<CrawlResult> {
    logger.info({ url, mode: options.extractionMode }, 'Extracting content');

    try {
      const { html, statusCode, contentType } = await this.fetchHtml(url, options);
      const $ = load(html);

      // Extract basic metadata
      const title = $('title').text() || $('h1').first().text() || '';
      const description = $('meta[name="description"]').attr('content') || '';

      // Extract links
      const links = this.extractLinks($, url);

      // Extract content based on mode
      let content = '';
      switch (options.extractionMode) {
        case 'text':
          content = this.extractText($);
          break;
        case 'markdown':
          content = this.extractMarkdown($);
          break;
        case 'html':
        default:
          content = html;
          break;
      }

      // Extract metadata
      const metadata = this.extractMetadata($);

      logger.info({ url, linksFound: links.length }, 'Content extracted successfully');

      return {
        url,
        title,
        content,
        links,
        metadata,
        timestamp: new Date(),
        statusCode,
        contentType
      };
    } catch (error) {
      logger.error({ url, error }, 'Failed to extract content');
      throw error;
    }
  }

  /**
   * Fetch HTML content
   */
  private async fetchHtml(
    url: string,
    options: ScrapeOptions
  ): Promise<{ html: string; statusCode: number; contentType?: string }> {
    // Use Puppeteer if waitFor or screenshot is required
    if (options.waitFor !== undefined || options.screenshot) {
      return this.fetchWithPuppeteer(url, options);
    }

    // Otherwise use fetch for better performance
    const response = await fetch(url, {
      headers: {
        'User-Agent': options.userAgent || this.userAgent,
        ...options.headers
      },
      signal: AbortSignal.timeout(options.timeout || config.crawler.requestTimeout)
    });

    const html = await response.text();

    return {
      html,
      statusCode: response.status,
      contentType: response.headers.get('content-type') || undefined
    };
  }

  /**
   * Fetch HTML using Puppeteer
   */
  private async fetchWithPuppeteer(
    url: string,
    options: ScrapeOptions
  ): Promise<{ html: string; statusCode: number; contentType?: string }> {
    if (!this.browser) {
      this.browser = await launch({
        headless: config.puppeteer.headless,
        args: ['--no-sandbox', '--disable-setuid-sandbox']
      });
    }

    const page = await this.browser.newPage();

    try {
      // Set user agent
      await page.setUserAgent(options.userAgent || this.userAgent);

      // Set custom headers
      if (options.headers) {
        await page.setExtraHTTPHeaders(options.headers);
      }

      // Set cookies
      if (options.cookies) {
        await page.setCookie(...options.cookies.map(c => ({
          name: c.name,
          value: c.value,
          domain: c.domain || new URL(url).hostname
        })));
      }

      // Navigate to page
      const response = await page.goto(url, {
        timeout: options.timeout || config.puppeteer.timeout,
        waitUntil: 'networkidle0'
      });

      // Wait for selector or timeout
      if (options.waitFor) {
        if (typeof options.waitFor === 'string') {
          await page.waitForSelector(options.waitFor, { timeout: 5000 }).catch(() => {
            logger.warn({ url, selector: options.waitFor }, 'Selector not found');
          });
        } else {
          await page.waitForTimeout(options.waitFor);
        }
      }

      // Take screenshot if requested
      if (options.screenshot) {
        await page.screenshot({ path: `screenshot-${Date.now()}.png` });
      }

      const html = await page.content();
      const statusCode = response?.status() || 200;
      const contentType = response?.headers()['content-type'];

      return { html, statusCode, contentType };
    } finally {
      await page.close();
    }
  }

  /**
   * Extract clean text content
   */
  private extractText($: any): string {
    // Remove script and style elements
    $('script, style, nav, footer, aside').remove();

    // Get main content
    const main = $('main, article, [role="main"]').first();
    const content = main.length > 0 ? main : $('body');

    return content
      .text()
      .replace(/\s+/g, ' ')
      .trim();
  }

  /**
   * Extract content as markdown
   */
  private extractMarkdown($: any): string {
    // Remove unwanted elements
    $('script, style, nav, footer, aside').remove();

    const lines: string[] = [];

    // Extract title
    const title = $('h1').first().text();
    if (title) {
      lines.push(`# ${title}`, '');
    }

    // Extract headings and paragraphs
    $('h2, h3, h4, p, ul, ol, pre, blockquote').each((_, elem) => {
      const $elem = $(elem);
      const tag = elem.name;

      switch (tag) {
        case 'h2':
          lines.push(`## ${$elem.text().trim()}`, '');
          break;
        case 'h3':
          lines.push(`### ${$elem.text().trim()}`, '');
          break;
        case 'h4':
          lines.push(`#### ${$elem.text().trim()}`, '');
          break;
        case 'p':
          lines.push($elem.text().trim(), '');
          break;
        case 'ul':
          $elem.find('li').each((_, li) => {
            lines.push(`- ${$(li).text().trim()}`);
          });
          lines.push('');
          break;
        case 'ol':
          $elem.find('li').each((i, li) => {
            lines.push(`${i + 1}. ${$(li).text().trim()}`);
          });
          lines.push('');
          break;
        case 'pre':
          lines.push('```', $elem.text().trim(), '```', '');
          break;
        case 'blockquote':
          lines.push(`> ${$elem.text().trim()}`, '');
          break;
      }
    });

    return lines.join('\n');
  }

  /**
   * Extract links from page
   */
  private extractLinks($: any, baseUrl: string): string[] {
    const links: Set<string> = new Set();

    $('a[href]').each((_, elem) => {
      const href = $(elem).attr('href');
      if (href && !href.startsWith('#') && !href.startsWith('javascript:')) {
        try {
          const absoluteUrl = new URL(href, baseUrl).toString();
          links.add(absoluteUrl);
        } catch {
          // Invalid URL, skip
        }
      }
    });

    return Array.from(links);
  }

  /**
   * Extract metadata
   */
  private extractMetadata($: any): Record<string, any> {
    const metadata: Record<string, any> = {};

    // Standard meta tags
    $('meta').each((_, elem) => {
      const $elem = $(elem);
      const name = $elem.attr('name') || $elem.attr('property');
      const content = $elem.attr('content');

      if (name && content) {
        metadata[name] = content;
      }
    });

    // Extract JSON-LD
    $('script[type="application/ld+json"]').each((_, elem) => {
      try {
        const jsonLd = JSON.parse($(elem).html() || '');
        metadata.jsonLd = jsonLd;
      } catch {
        // Invalid JSON-LD, skip
      }
    });

    return metadata;
  }

  /**
   * Close browser
   */
  async close(): Promise<void> {
    if (this.browser) {
      await this.browser.close();
      this.browser = null;
    }
  }
}
