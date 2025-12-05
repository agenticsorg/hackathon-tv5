/**
 * Tests for Machine View
 */

import { describe, it, expect } from '@jest/globals';
import { ContentExtractor } from '../../src/arw/machine-view';

describe('ContentExtractor', () => {
  describe('extractCodeBlocks()', () => {
    it('should extract code blocks from markdown', () => {
      const markdown = `
# Example

\`\`\`javascript
console.log('hello');
\`\`\`

\`\`\`python
print('world')
\`\`\`
      `;

      const blocks = ContentExtractor.extractCodeBlocks(markdown);
      expect(blocks).toHaveLength(2);
      expect(blocks[0].language).toBe('javascript');
      expect(blocks[0].code).toContain('console.log');
      expect(blocks[1].language).toBe('python');
      expect(blocks[1].code).toContain('print');
    });
  });

  describe('extractLinks()', () => {
    it('should extract links from markdown', () => {
      const markdown = `
[Example Link](https://example.com)
[Another Link](https://another.com)
      `;

      const links = ContentExtractor.extractLinks(markdown);
      expect(links).toHaveLength(2);
      expect(links[0].text).toBe('Example Link');
      expect(links[0].url).toBe('https://example.com');
    });
  });

  describe('extractTables()', () => {
    it('should extract tables from markdown', () => {
      const markdown = `
| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |
| Value 3  | Value 4  |
      `;

      const tables = ContentExtractor.extractTables(markdown);
      expect(tables).toHaveLength(1);
      expect(tables[0]).toContain('Column 1');
    });
  });

  describe('stripMarkdown()', () => {
    it('should remove markdown formatting', () => {
      const markdown = '# Heading\n**bold** and *italic* and `code`';
      const plain = ContentExtractor.stripMarkdown(markdown);

      expect(plain).not.toContain('#');
      expect(plain).not.toContain('**');
      expect(plain).not.toContain('*');
      expect(plain).not.toContain('`');
      expect(plain).toContain('Heading');
      expect(plain).toContain('bold');
    });
  });
});
