/**
 * Unit tests for ContentExtractor
 * Tests HTML parsing, content extraction, and machine view generation
 */

import { describe, it, expect, beforeEach } from '@jest/globals';
import {
  BASIC_HTML,
  ARW_COMPLIANT_HTML,
  COMPLEX_HTML,
  MALFORMED_HTML,
  EMPTY_HTML,
  HTML_WITH_SCRIPTS,
  HTML_WITH_METADATA
} from '../fixtures/html-samples';

/**
 * Mock ContentExtractor for testing
 * In actual implementation, import from src/core/content-extractor
 */
class ContentExtractor {
  extractText(html: string): string {
    // Remove scripts and styles
    let clean = html.replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '');
    clean = clean.replace(/<style\b[^<]*(?:(?!<\/style>)<[^<]*)*<\/style>/gi, '');
    // Remove HTML tags
    clean = clean.replace(/<[^>]+>/g, ' ');
    // Normalize whitespace
    return clean.replace(/\s+/g, ' ').trim();
  }

  extractLinks(html: string): string[] {
    const links: string[] = [];
    const regex = /<a[^>]+href=["']([^"']+)["']/gi;
    let match;
    while ((match = regex.exec(html)) !== null) {
      links.push(match[1]);
    }
    return links;
  }

  extractMetadata(html: string): Record<string, string> {
    const metadata: Record<string, string> = {};

    // Extract title
    const titleMatch = html.match(/<title>([^<]+)<\/title>/i);
    if (titleMatch) metadata.title = titleMatch[1];

    // Extract meta tags
    const metaRegex = /<meta[^>]+name=["']([^"']+)["'][^>]+content=["']([^"']+)["']/gi;
    let match;
    while ((match = metaRegex.exec(html)) !== null) {
      metadata[match[1]] = match[2];
    }

    return metadata;
  }

  extractChunks(html: string): Array<{ id: string; content: string }> {
    const chunks: Array<{ id: string; content: string }> = [];
    const regex = /data-chunk-id=["']([^"']+)["'][^>]*>(.*?)<\/[^>]+>/gis;
    let match;
    while ((match = regex.exec(html)) !== null) {
      chunks.push({ id: match[1], content: match[2].trim() });
    }
    return chunks;
  }

  toMarkdown(html: string): string {
    // Simplified markdown conversion
    let md = html;
    md = md.replace(/<h1[^>]*>(.*?)<\/h1>/gi, '# $1\n\n');
    md = md.replace(/<h2[^>]*>(.*?)<\/h2>/gi, '## $1\n\n');
    md = md.replace(/<h3[^>]*>(.*?)<\/h3>/gi, '### $1\n\n');
    md = md.replace(/<p[^>]*>(.*?)<\/p>/gi, '$1\n\n');
    md = md.replace(/<strong[^>]*>(.*?)<\/strong>/gi, '**$1**');
    md = md.replace(/<em[^>]*>(.*?)<\/em>/gi, '*$1*');
    md = md.replace(/<a[^>]+href=["']([^"']+)["'][^>]*>(.*?)<\/a>/gi, '[$2]($1)');
    md = md.replace(/<[^>]+>/g, '');
    return md.trim();
  }
}

describe('ContentExtractor', () => {
  let extractor: ContentExtractor;

  beforeEach(() => {
    extractor = new ContentExtractor();
  });

  describe('Text Extraction', () => {
    it('should extract plain text from HTML', () => {
      const text = extractor.extractText(BASIC_HTML);
      expect(text).toContain('Welcome to Test Page');
      expect(text).toContain('simple test page');
    });

    it('should remove script tags', () => {
      const text = extractor.extractText(HTML_WITH_SCRIPTS);
      expect(text).not.toContain('sensitive');
      expect(text).not.toContain('console.log');
    });

    it('should handle empty HTML', () => {
      const text = extractor.extractText(EMPTY_HTML);
      expect(text).toBeDefined();
    });

    it('should handle malformed HTML', () => {
      const text = extractor.extractText(MALFORMED_HTML);
      expect(text).toContain('Missing closing tags');
    });

    it('should normalize whitespace', () => {
      const html = '<p>Text   with    multiple    spaces</p>';
      const text = extractor.extractText(html);
      expect(text).toBe('Text with multiple spaces');
    });
  });

  describe('Link Extraction', () => {
    it('should extract all links from HTML', () => {
      const links = extractor.extractLinks(BASIC_HTML);
      expect(links).toContain('/page1');
      expect(links).toContain('/page2');
    });

    it('should extract links from complex HTML', () => {
      const links = extractor.extractLinks(COMPLEX_HTML);
      expect(links.length).toBeGreaterThan(0);
      expect(links).toContain('/');
      expect(links).toContain('/about');
    });

    it('should return empty array for HTML without links', () => {
      const html = '<div>No links here</div>';
      const links = extractor.extractLinks(html);
      expect(links).toEqual([]);
    });

    it('should handle links with different quote styles', () => {
      const html = '<a href="link1">Test</a><a href=\'link2\'>Test</a>';
      const links = extractor.extractLinks(html);
      expect(links).toHaveLength(2);
    });
  });

  describe('Metadata Extraction', () => {
    it('should extract title', () => {
      const metadata = extractor.extractMetadata(HTML_WITH_METADATA);
      expect(metadata.title).toBe('Metadata Test');
    });

    it('should extract meta tags', () => {
      const metadata = extractor.extractMetadata(HTML_WITH_METADATA);
      expect(metadata.description).toBe('Test description');
      expect(metadata.keywords).toBe('test, crawler, arw');
    });

    it('should return empty object for HTML without metadata', () => {
      const html = '<html><body>No metadata</body></html>';
      const metadata = extractor.extractMetadata(html);
      expect(Object.keys(metadata)).toHaveLength(0);
    });

    it('should handle missing title', () => {
      const html = '<html><head></head></html>';
      const metadata = extractor.extractMetadata(html);
      expect(metadata.title).toBeUndefined();
    });
  });

  describe('Chunk Extraction (ARW)', () => {
    it('should extract chunks from ARW-compliant HTML', () => {
      const chunks = extractor.extractChunks(ARW_COMPLIANT_HTML);
      expect(chunks).toHaveLength(2);
      expect(chunks[0].id).toBe('overview');
      expect(chunks[1].id).toBe('features');
    });

    it('should return empty array for non-chunked HTML', () => {
      const chunks = extractor.extractChunks(BASIC_HTML);
      expect(chunks).toEqual([]);
    });

    it('should handle nested chunks', () => {
      const html = `
        <section data-chunk-id="parent">
          <div data-chunk-id="child">Content</div>
        </section>
      `;
      const chunks = extractor.extractChunks(html);
      expect(chunks.length).toBeGreaterThan(0);
    });
  });

  describe('Markdown Conversion', () => {
    it('should convert headings to markdown', () => {
      const html = '<h1>Title</h1><h2>Subtitle</h2>';
      const md = extractor.toMarkdown(html);
      expect(md).toContain('# Title');
      expect(md).toContain('## Subtitle');
    });

    it('should convert paragraphs to markdown', () => {
      const html = '<p>Paragraph text</p>';
      const md = extractor.toMarkdown(html);
      expect(md).toContain('Paragraph text');
    });

    it('should convert bold and italic', () => {
      const html = '<p><strong>bold</strong> and <em>italic</em></p>';
      const md = extractor.toMarkdown(html);
      expect(md).toContain('**bold**');
      expect(md).toContain('*italic*');
    });

    it('should convert links to markdown format', () => {
      const html = '<a href="/page">Link text</a>';
      const md = extractor.toMarkdown(html);
      expect(md).toContain('[Link text](/page)');
    });

    it('should handle complex HTML', () => {
      const md = extractor.toMarkdown(COMPLEX_HTML);
      expect(md).toBeDefined();
      expect(md.length).toBeGreaterThan(0);
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty string', () => {
      expect(() => extractor.extractText('')).not.toThrow();
      expect(extractor.extractLinks('')).toEqual([]);
    });

    it('should handle very large HTML', () => {
      const largeHtml = '<div>' + 'a'.repeat(100000) + '</div>';
      const text = extractor.extractText(largeHtml);
      expect(text).toBeDefined();
    });

    it('should handle special characters', () => {
      const html = '<p>&lt;&gt;&amp;&quot;&#39;</p>';
      const text = extractor.extractText(html);
      expect(text).toBeDefined();
    });

    it('should handle unicode characters', () => {
      const html = '<p>Hello 世界 🌍</p>';
      const text = extractor.extractText(html);
      expect(text).toContain('Hello');
    });
  });

  describe('Performance', () => {
    it('should process HTML quickly', () => {
      const start = Date.now();
      extractor.extractText(COMPLEX_HTML);
      const duration = Date.now() - start;
      expect(duration).toBeLessThan(100);
    });

    it('should handle repeated extractions efficiently', () => {
      const start = Date.now();
      for (let i = 0; i < 100; i++) {
        extractor.extractText(BASIC_HTML);
      }
      const duration = Date.now() - start;
      expect(duration).toBeLessThan(1000);
    });
  });
});
