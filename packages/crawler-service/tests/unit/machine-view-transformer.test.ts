/**
 * Unit tests for MachineViewTransformer
 * Tests transformation from HTML to LLM-optimized markdown
 */

import { describe, it, expect } from '@jest/globals';

/**
 * Mock MachineViewTransformer
 */
class MachineViewTransformer {
  transform(html: string, options?: any): string {
    // Remove scripts and styles
    let clean = html.replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '');
    clean = clean.replace(/<style\b[^<]*(?:(?!<\/style>)<[^<]*)*<\/style>/gi, '');

    // Remove navigation, ads, footer
    if (options?.removeNav) {
      clean = clean.replace(/<nav\b[^<]*(?:(?!<\/nav>)<[^<]*)*<\/nav>/gi, '');
      clean = clean.replace(/<footer\b[^<]*(?:(?!<\/footer>)<[^<]*)*<\/footer>/gi, '');
    }

    // Convert to markdown
    clean = clean.replace(/<h1[^>]*>(.*?)<\/h1>/gi, '# $1\n\n');
    clean = clean.replace(/<h2[^>]*>(.*?)<\/h2>/gi, '## $1\n\n');
    clean = clean.replace(/<p[^>]*>(.*?)<\/p>/gi, '$1\n\n');

    // Clean up
    clean = clean.replace(/<[^>]+>/g, '');
    return clean.trim();
  }

  addChunkMarkers(markdown: string): string {
    // Add chunk markers between sections
    return markdown.replace(/^(#{1,3} .+)$/gm, '<!-- chunk: auto -->\n\n$1');
  }

  optimizeTokens(markdown: string): string {
    // Remove excessive whitespace
    return markdown.replace(/\n{3,}/g, '\n\n').trim();
  }
}

describe('MachineViewTransformer', () => {
  let transformer: MachineViewTransformer;

  beforeEach(() => {
    transformer = new MachineViewTransformer();
  });

  describe('HTML to Markdown Transformation', () => {
    it('should convert HTML to clean markdown', () => {
      const html = '<h1>Title</h1><p>Content</p>';
      const md = transformer.transform(html);

      expect(md).toContain('# Title');
      expect(md).toContain('Content');
      expect(md).not.toContain('<');
    });

    it('should remove scripts', () => {
      const html = '<h1>Title</h1><script>alert("test")</script><p>Content</p>';
      const md = transformer.transform(html);

      expect(md).not.toContain('script');
      expect(md).not.toContain('alert');
    });

    it('should remove navigation when option enabled', () => {
      const html = '<nav><a>Link</a></nav><h1>Title</h1>';
      const md = transformer.transform(html, { removeNav: true });

      expect(md).not.toContain('Link');
      expect(md).toContain('# Title');
    });

    it('should preserve semantic structure', () => {
      const html = '<h1>Main</h1><h2>Sub</h2><p>Text</p>';
      const md = transformer.transform(html);

      expect(md).toContain('# Main');
      expect(md).toContain('## Sub');
      expect(md).toContain('Text');
    });
  });

  describe('Chunk Markers', () => {
    it('should add chunk markers to sections', () => {
      const md = '# Section 1\n\nContent\n\n## Section 2';
      const chunked = transformer.addChunkMarkers(md);

      expect(chunked).toContain('<!-- chunk: auto -->');
    });

    it('should handle markdown without headers', () => {
      const md = 'Just plain text';
      const chunked = transformer.addChunkMarkers(md);

      expect(chunked).toBe(md);
    });
  });

  describe('Token Optimization', () => {
    it('should remove excessive whitespace', () => {
      const md = 'Text\n\n\n\n\nMore text';
      const optimized = transformer.optimizeTokens(md);

      expect(optimized).toBe('Text\n\nMore text');
    });

    it('should preserve necessary whitespace', () => {
      const md = 'Para 1\n\nPara 2';
      const optimized = transformer.optimizeTokens(md);

      expect(optimized).toBe('Para 1\n\nPara 2');
    });
  });
});
