import { load } from 'cheerio';
import { createChildLogger } from '../utils/logger.js';
import type { MachineView, Section, Link, Metadata, CrawlResult } from '../types/index.js';

const logger = createChildLogger('machine-view-transformer');

/**
 * Transforms HTML content into machine-optimized view for AI consumption
 */
export class MachineViewTransformer {
  /**
   * Transform crawl result into machine view
   */
  transform(result: CrawlResult): MachineView {
    logger.info({ url: result.url }, 'Transforming to machine view');

    const $ = load(result.content);

    // Remove noise
    this.removeNoise($);

    // Extract structured data
    const title = this.extractTitle($);
    const description = this.extractDescription($);
    const mainContent = this.extractMainContent($);
    const sections = this.extractSections($);
    const links = this.extractLinks($, result.url);
    const metadata = this.buildMetadata($, result);

    const machineView: MachineView = {
      title,
      description,
      mainContent,
      sections,
      links,
      metadata,
      schema: this.extractSchema($)
    };

    logger.info({ url: result.url, sectionCount: sections.length }, 'Machine view created');

    return machineView;
  }

  /**
   * Remove non-content elements
   */
  private removeNoise($: any): void {
    $(
      'script, style, nav, footer, aside, .ad, .advertisement, ' +
      '.social-share, .comments, .cookie-banner, iframe, noscript'
    ).remove();
  }

  /**
   * Extract title
   */
  private extractTitle($: any): string {
    // Priority: h1 > title > og:title
    return (
      $('h1').first().text().trim() ||
      $('title').text().trim() ||
      $('meta[property="og:title"]').attr('content') ||
      'Untitled'
    );
  }

  /**
   * Extract description
   */
  private extractDescription($: any): string | undefined {
    return (
      $('meta[name="description"]').attr('content') ||
      $('meta[property="og:description"]').attr('content') ||
      undefined
    );
  }

  /**
   * Extract main content
   */
  private extractMainContent($: any): string {
    // Try to find main content area
    const selectors = [
      'main',
      'article',
      '[role="main"]',
      '.main-content',
      '.content',
      '#content',
      '.post-content',
      '.article-content'
    ];

    for (const selector of selectors) {
      const element = $(selector).first();
      if (element.length > 0) {
        return element.text().replace(/\s+/g, ' ').trim();
      }
    }

    // Fallback to body
    return $('body').text().replace(/\s+/g, ' ').trim();
  }

  /**
   * Extract structured sections
   */
  private extractSections($: any): Section[] {
    const sections: Section[] = [];

    // Extract headings with their content
    $('h1, h2, h3, h4, h5, h6').each((_, elem) => {
      const $elem = $(elem);
      const level = parseInt(elem.name.substring(1), 10);
      const content = $elem.text().trim();

      if (content) {
        sections.push({
          type: 'heading',
          content,
          level
        });
      }
    });

    // Extract paragraphs
    $('p').each((_, elem) => {
      const content = $(elem).text().trim();
      if (content && content.length > 20) { // Filter short paragraphs
        sections.push({
          type: 'paragraph',
          content
        });
      }
    });

    // Extract lists
    $('ul, ol').each((_, elem) => {
      const $elem = $(elem);
      const items: string[] = [];

      $elem.find('li').each((_, li) => {
        const text = $(li).text().trim();
        if (text) {
          items.push(text);
        }
      });

      if (items.length > 0) {
        sections.push({
          type: 'list',
          content: items.join('\n'),
          items
        });
      }
    });

    // Extract code blocks
    $('pre, code').each((_, elem) => {
      const content = $(elem).text().trim();
      if (content && content.length > 10) {
        sections.push({
          type: 'code',
          content
        });
      }
    });

    // Extract tables
    $('table').each((_, elem) => {
      const $table = $(elem);
      const rows: string[] = [];

      $table.find('tr').each((_, tr) => {
        const cells: string[] = [];
        $(tr).find('th, td').each((_, cell) => {
          cells.push($(cell).text().trim());
        });
        if (cells.length > 0) {
          rows.push(cells.join(' | '));
        }
      });

      if (rows.length > 0) {
        sections.push({
          type: 'table',
          content: rows.join('\n')
        });
      }
    });

    return sections;
  }

  /**
   * Extract and classify links
   */
  private extractLinks($: any, baseUrl: string): Link[] {
    const links: Link[] = [];
    const baseDomain = new URL(baseUrl).hostname;

    $('a[href]').each((_, elem) => {
      const $elem = $(elem);
      const href = $elem.attr('href');
      const text = $elem.text().trim();
      const rel = $elem.attr('rel');

      if (href && !href.startsWith('#') && !href.startsWith('javascript:')) {
        try {
          const absoluteUrl = new URL(href, baseUrl).toString();
          const linkDomain = new URL(absoluteUrl).hostname;

          links.push({
            href: absoluteUrl,
            text: text || href,
            rel,
            type: linkDomain === baseDomain ? 'internal' : 'external'
          });
        } catch {
          // Invalid URL, skip
        }
      }
    });

    return links;
  }

  /**
   * Build comprehensive metadata
   */
  private buildMetadata($: any, result: CrawlResult): Metadata {
    const metadata: Metadata = {};

    // Basic meta tags
    metadata.title = $('title').text().trim();
    metadata.description = $('meta[name="description"]').attr('content');
    metadata.author = $('meta[name="author"]').attr('content');
    metadata.keywords = $('meta[name="keywords"]').attr('content')?.split(',').map(k => k.trim());
    metadata.language = $('html').attr('lang') || $('meta[name="language"]').attr('content');

    // Dates
    metadata.publishedDate =
      $('meta[property="article:published_time"]').attr('content') ||
      $('meta[name="date"]').attr('content') ||
      $('time[itemprop="datePublished"]').attr('datetime');

    metadata.modifiedDate =
      $('meta[property="article:modified_time"]').attr('content') ||
      $('time[itemprop="dateModified"]').attr('datetime');

    // Canonical URL
    metadata.canonicalUrl = $('link[rel="canonical"]').attr('href');

    // Open Graph data
    const ogData: Record<string, string> = {};
    $('meta[property^="og:"]').each((_, elem) => {
      const property = $(elem).attr('property')!.replace('og:', '');
      const content = $(elem).attr('content');
      if (content) {
        ogData[property] = content;
      }
    });
    if (Object.keys(ogData).length > 0) {
      metadata.ogData = ogData;
    }

    return metadata;
  }

  /**
   * Extract structured data (JSON-LD, microdata)
   */
  private extractSchema($: any): any {
    const schemas: any[] = [];

    // Extract JSON-LD
    $('script[type="application/ld+json"]').each((_, elem) => {
      try {
        const data = JSON.parse($(elem).html() || '');
        schemas.push(data);
      } catch {
        // Invalid JSON-LD, skip
      }
    });

    // Extract microdata (basic support)
    $('[itemscope]').each((_, elem) => {
      const $elem = $(elem);
      const itemType = $elem.attr('itemtype');
      const properties: Record<string, string> = {};

      $elem.find('[itemprop]').each((_, prop) => {
        const name = $(prop).attr('itemprop')!;
        const content = $(prop).attr('content') || $(prop).text().trim();
        properties[name] = content;
      });

      if (itemType && Object.keys(properties).length > 0) {
        schemas.push({
          '@type': itemType,
          ...properties
        });
      }
    });

    return schemas.length > 0 ? schemas : undefined;
  }

  /**
   * Generate machine-readable text summary
   */
  generateSummary(machineView: MachineView, maxLength: number = 500): string {
    const parts: string[] = [];

    // Title
    parts.push(`Title: ${machineView.title}`);

    // Description
    if (machineView.description) {
      parts.push(`Description: ${machineView.description}`);
    }

    // Main sections
    const headings = machineView.sections
      .filter(s => s.type === 'heading')
      .map(s => s.content)
      .slice(0, 5);

    if (headings.length > 0) {
      parts.push(`Sections: ${headings.join(', ')}`);
    }

    // Content preview
    const contentPreview = machineView.mainContent.substring(0, 200);
    parts.push(`Content: ${contentPreview}...`);

    let summary = parts.join('\n');

    // Truncate if needed
    if (summary.length > maxLength) {
      summary = summary.substring(0, maxLength - 3) + '...';
    }

    return summary;
  }
}
