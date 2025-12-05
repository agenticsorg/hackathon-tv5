import { fetch } from 'undici';
import robotsParser from 'robots-parser';
import { createChildLogger } from '../utils/logger.js';
import { normalizeUrl, resolveUrl, getBaseUrl } from '../utils/url-helpers.js';
import type { ARWDiscoveryResult, LlmsTxtData, RobotsTxtData, SitemapData } from '../types/index.js';

const logger = createChildLogger('arw-discovery');

export class ARWDiscovery {
  private cache: Map<string, ARWDiscoveryResult> = new Map();
  private userAgent: string;

  constructor(userAgent: string = 'ARW-Crawler/1.0') {
    this.userAgent = userAgent;
  }

  /**
   * Discover ARW resources for a given URL
   */
  async discover(url: string): Promise<ARWDiscoveryResult> {
    const baseUrl = getBaseUrl(url);
    const cacheKey = normalizeUrl(baseUrl);

    // Check cache
    if (this.cache.has(cacheKey)) {
      logger.debug({ url: baseUrl }, 'Using cached discovery result');
      return this.cache.get(cacheKey)!;
    }

    logger.info({ url: baseUrl }, 'Starting ARW discovery');

    const result: ARWDiscoveryResult = {};

    // Discover in parallel
    const [llmsTxt, robotsTxt, sitemap] = await Promise.allSettled([
      this.discoverLlmsTxt(baseUrl),
      this.discoverRobotsTxt(baseUrl),
      this.discoverSitemap(baseUrl)
    ]);

    if (llmsTxt.status === 'fulfilled' && llmsTxt.value) {
      result.llmsTxt = llmsTxt.value;
    } else if (llmsTxt.status === 'rejected') {
      logger.debug({ error: llmsTxt.reason }, 'llms.txt not found or invalid');
    }

    if (robotsTxt.status === 'fulfilled' && robotsTxt.value) {
      result.robotsTxt = robotsTxt.value;
    } else if (robotsTxt.status === 'rejected') {
      logger.debug({ error: robotsTxt.reason }, 'robots.txt not found or invalid');
    }

    if (sitemap.status === 'fulfilled' && sitemap.value) {
      result.sitemap = sitemap.value;
    } else if (sitemap.status === 'rejected') {
      logger.debug({ error: sitemap.reason }, 'Sitemap not found or invalid');
    }

    this.cache.set(cacheKey, result);
    logger.info({ url: baseUrl, found: Object.keys(result) }, 'ARW discovery completed');

    return result;
  }

  /**
   * Discover and parse llms.txt file
   */
  async discoverLlmsTxt(baseUrl: string): Promise<LlmsTxtData | null> {
    const llmsUrl = resolveUrl(baseUrl, '/llms.txt');

    try {
      const response = await fetch(llmsUrl, {
        headers: { 'User-Agent': this.userAgent },
        signal: AbortSignal.timeout(10000)
      });

      if (!response.ok) {
        return null;
      }

      const content = await response.text();
      const sections = this.parseLlmsTxt(content);

      logger.info({ url: llmsUrl }, 'Successfully parsed llms.txt');

      return {
        url: llmsUrl,
        content,
        sections,
        parsedAt: new Date()
      };
    } catch (error) {
      logger.debug({ url: llmsUrl, error }, 'Failed to fetch llms.txt');
      return null;
    }
  }

  /**
   * Parse llms.txt content into structured sections
   */
  private parseLlmsTxt(content: string): Array<{ title: string; content: string; links?: string[] }> {
    const sections: Array<{ title: string; content: string; links?: string[] }> = [];
    const lines = content.split('\n');

    let currentSection: { title: string; content: string; links: string[] } | null = null;

    for (const line of lines) {
      const trimmed = line.trim();

      // Section header (starts with #)
      if (trimmed.startsWith('#')) {
        if (currentSection) {
          sections.push(currentSection);
        }
        currentSection = {
          title: trimmed.replace(/^#+\s*/, ''),
          content: '',
          links: []
        };
      } else if (currentSection) {
        // Extract links
        const linkMatches = trimmed.matchAll(/https?:\/\/[^\s)]+/g);
        for (const match of linkMatches) {
          currentSection.links.push(match[0]);
        }

        currentSection.content += (currentSection.content ? '\n' : '') + trimmed;
      }
    }

    if (currentSection) {
      sections.push(currentSection);
    }

    return sections;
  }

  /**
   * Discover and parse robots.txt file
   */
  async discoverRobotsTxt(baseUrl: string): Promise<RobotsTxtData | null> {
    const robotsUrl = resolveUrl(baseUrl, '/robots.txt');

    try {
      const response = await fetch(robotsUrl, {
        headers: { 'User-Agent': this.userAgent },
        signal: AbortSignal.timeout(10000)
      });

      if (!response.ok) {
        return null;
      }

      const content = await response.text();
      const robots = robotsParser(robotsUrl, content);

      logger.info({ url: robotsUrl }, 'Successfully parsed robots.txt');

      return {
        url: robotsUrl,
        content,
        rules: this.parseRobotsTxtRules(content),
        sitemaps: this.extractSitemaps(content),
        crawlDelay: this.extractCrawlDelay(content, this.userAgent),
        parsedAt: new Date()
      };
    } catch (error) {
      logger.debug({ url: robotsUrl, error }, 'Failed to fetch robots.txt');
      return null;
    }
  }

  /**
   * Parse robots.txt rules
   */
  private parseRobotsTxtRules(content: string): Array<{ userAgent: string; allow?: string[]; disallow?: string[] }> {
    const rules: Array<{ userAgent: string; allow?: string[]; disallow?: string[] }> = [];
    const lines = content.split('\n');

    let currentRule: { userAgent: string; allow: string[]; disallow: string[] } | null = null;

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith('#')) continue;

      const [key, value] = trimmed.split(':').map(s => s.trim());

      if (key.toLowerCase() === 'user-agent') {
        if (currentRule) {
          rules.push(currentRule);
        }
        currentRule = { userAgent: value, allow: [], disallow: [] };
      } else if (currentRule) {
        if (key.toLowerCase() === 'allow' && value) {
          currentRule.allow.push(value);
        } else if (key.toLowerCase() === 'disallow' && value) {
          currentRule.disallow.push(value);
        }
      }
    }

    if (currentRule) {
      rules.push(currentRule);
    }

    return rules;
  }

  /**
   * Extract sitemap URLs from robots.txt
   */
  private extractSitemaps(content: string): string[] {
    const sitemaps: string[] = [];
    const lines = content.split('\n');

    for (const line of lines) {
      const trimmed = line.trim();
      if (trimmed.toLowerCase().startsWith('sitemap:')) {
        const url = trimmed.substring(8).trim();
        if (url) {
          sitemaps.push(url);
        }
      }
    }

    return sitemaps;
  }

  /**
   * Extract crawl delay for user agent
   */
  private extractCrawlDelay(content: string, userAgent: string): number | undefined {
    const lines = content.split('\n');
    let isRelevantSection = false;

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith('#')) continue;

      const [key, value] = trimmed.split(':').map(s => s.trim());

      if (key.toLowerCase() === 'user-agent') {
        isRelevantSection = value === '*' || value === userAgent;
      } else if (isRelevantSection && key.toLowerCase() === 'crawl-delay') {
        const delay = parseInt(value, 10);
        if (!isNaN(delay)) {
          return delay * 1000; // Convert to milliseconds
        }
      }
    }

    return undefined;
  }

  /**
   * Discover sitemap.xml
   */
  async discoverSitemap(baseUrl: string): Promise<SitemapData | null> {
    const sitemapUrls = [
      resolveUrl(baseUrl, '/sitemap.xml'),
      resolveUrl(baseUrl, '/sitemap_index.xml'),
      resolveUrl(baseUrl, '/sitemap-index.xml')
    ];

    for (const sitemapUrl of sitemapUrls) {
      try {
        const response = await fetch(sitemapUrl, {
          headers: { 'User-Agent': this.userAgent },
          signal: AbortSignal.timeout(10000)
        });

        if (response.ok) {
          const content = await response.text();
          const urls = this.parseSitemap(content);

          logger.info({ url: sitemapUrl, urlCount: urls.length }, 'Successfully parsed sitemap');

          return {
            url: sitemapUrl,
            urls,
            parsedAt: new Date()
          };
        }
      } catch (error) {
        logger.debug({ url: sitemapUrl, error }, 'Failed to fetch sitemap');
      }
    }

    return null;
  }

  /**
   * Parse sitemap.xml content
   */
  private parseSitemap(content: string): Array<{
    loc: string;
    lastmod?: string;
    changefreq?: string;
    priority?: number;
  }> {
    const urls: Array<{
      loc: string;
      lastmod?: string;
      changefreq?: string;
      priority?: number;
    }> = [];

    // Simple XML parsing without dependencies
    const urlMatches = content.matchAll(/<url>([\s\S]*?)<\/url>/g);

    for (const match of urlMatches) {
      const urlBlock = match[1];

      const locMatch = urlBlock.match(/<loc>(.*?)<\/loc>/);
      const lastmodMatch = urlBlock.match(/<lastmod>(.*?)<\/lastmod>/);
      const changefreqMatch = urlBlock.match(/<changefreq>(.*?)<\/changefreq>/);
      const priorityMatch = urlBlock.match(/<priority>(.*?)<\/priority>/);

      if (locMatch) {
        urls.push({
          loc: locMatch[1].trim(),
          lastmod: lastmodMatch ? lastmodMatch[1].trim() : undefined,
          changefreq: changefreqMatch ? changefreqMatch[1].trim() : undefined,
          priority: priorityMatch ? parseFloat(priorityMatch[1]) : undefined
        });
      }
    }

    return urls;
  }

  /**
   * Check if URL is allowed by robots.txt
   */
  isAllowed(url: string, robotsTxt?: RobotsTxtData): boolean {
    if (!robotsTxt) return true;

    const robots = robotsParser(robotsTxt.url, robotsTxt.content);
    return robots.isAllowed(url, this.userAgent) ?? true;
  }

  /**
   * Clear cache
   */
  clearCache(): void {
    this.cache.clear();
  }
}
