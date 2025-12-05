/**
 * HTML Processor Module
 * Extracts chunks from rendered HTML and converts to markdown
 */

import * as cheerio from 'cheerio';
import TurndownService from 'turndown';

export interface ChunkData {
  id: string;
  heading?: string;
  content: string;
  order: number;
}

export class HTMLProcessor {
  private turndown: TurndownService;

  constructor() {
    // Initialize turndown with ARW-friendly settings
    this.turndown = new TurndownService({
      headingStyle: 'atx',
      codeBlockStyle: 'fenced',
      emDelimiter: '*',
      bulletListMarker: '-'
    });

    // Add custom rules for better markdown conversion
    this.setupCustomRules();
  }

  /**
   * Extract chunks from HTML content
   */
  async extractChunks(html: string): Promise<ChunkData[]> {
    const $ = cheerio.load(html);
    const chunks: ChunkData[] = [];

    // Find all elements with data-chunk-id attribute
    $('[data-chunk-id]').each((index, element) => {
      const $el = $(element);
      const chunkId = $el.attr('data-chunk-id');

      if (!chunkId) return;

      // Extract heading from first h1/h2/h3 in chunk
      const heading = $el.find('h1, h2, h3').first().text().trim();

      // Convert chunk HTML to markdown
      const chunkHtml = $el.html() || '';
      const content = this.convertToMarkdown(chunkHtml);

      chunks.push({
        id: chunkId,
        heading: heading || undefined,
        content,
        order: index
      });
    });

    return chunks;
  }

  /**
   * Convert HTML to clean markdown
   */
  private convertToMarkdown(html: string): string {
    try {
      // Use turndown to convert HTML to markdown
      let markdown = this.turndown.turndown(html);

      // Clean up excessive whitespace
      markdown = markdown.replace(/\n{3,}/g, '\n\n');

      // Trim leading/trailing whitespace
      markdown = markdown.trim();

      return markdown;
    } catch (error) {
      console.error('[ARW] Error converting HTML to markdown:', error);
      return html; // Fallback to raw HTML
    }
  }

  /**
   * Setup custom turndown rules for better conversion
   */
  private setupCustomRules(): void {
    // Remove script and style tags
    this.turndown.remove(['script', 'style', 'noscript']);

    // Custom rule for preserving code blocks
    this.turndown.addRule('codeBlock', {
      filter: ['pre'],
      replacement: (content, node) => {
        const code = (node as any).querySelector('code');
        const language = code ? this.getCodeLanguage(code) : '';
        return `\n\`\`\`${language}\n${content.trim()}\n\`\`\`\n`;
      }
    });

    // Custom rule for inline code
    this.turndown.addRule('inlineCode', {
      filter: (node) => {
        return (
          node.nodeName === 'CODE' &&
          node.parentNode?.nodeName !== 'PRE'
        );
      },
      replacement: (content) => {
        return `\`${content}\``;
      }
    });
  }

  /**
   * Extract language from code element class
   */
  private getCodeLanguage(codeElement: any): string {
    const className = codeElement.className || '';
    const match = className.match(/language-(\w+)/);
    return match ? match[1] : '';
  }
}
