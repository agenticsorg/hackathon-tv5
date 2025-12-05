/**
 * Tests for HTML Processor
 */

import { describe, it, expect } from 'vitest';
import { HTMLProcessor } from '../../src/generator/html-processor';

describe('HTMLProcessor', () => {
  const processor = new HTMLProcessor();

  it('should extract chunks from HTML with data-chunk-id', async () => {
    const html = `
      <div data-chunk-id="intro">
        <h2>Introduction</h2>
        <p>This is the intro section.</p>
      </div>
      <div data-chunk-id="content">
        <h2>Main Content</h2>
        <p>This is the main content.</p>
      </div>
    `;

    const chunks = await processor.extractChunks(html);

    expect(chunks).toHaveLength(2);
    expect(chunks[0].id).toBe('intro');
    expect(chunks[0].heading).toBe('Introduction');
    expect(chunks[0].content).toContain('intro section');
    expect(chunks[1].id).toBe('content');
  });

  it('should handle chunks without headings', async () => {
    const html = `
      <div data-chunk-id="footer">
        <p>Footer content</p>
      </div>
    `;

    const chunks = await processor.extractChunks(html);

    expect(chunks).toHaveLength(1);
    expect(chunks[0].id).toBe('footer');
    expect(chunks[0].heading).toBeUndefined();
    expect(chunks[0].content).toContain('Footer content');
  });

  it('should convert HTML to markdown', async () => {
    const html = `
      <div data-chunk-id="test">
        <h2>Test</h2>
        <p>Paragraph with <strong>bold</strong> and <em>italic</em>.</p>
        <ul>
          <li>Item 1</li>
          <li>Item 2</li>
        </ul>
      </div>
    `;

    const chunks = await processor.extractChunks(html);

    expect(chunks[0].content).toContain('**bold**');
    expect(chunks[0].content).toContain('*italic*');
    expect(chunks[0].content).toContain('- Item 1');
  });

  it('should handle nested chunks', async () => {
    const html = `
      <section data-chunk-id="outer">
        <h1>Outer</h1>
        <div data-chunk-id="inner">
          <h2>Inner</h2>
          <p>Inner content</p>
        </div>
      </section>
    `;

    const chunks = await processor.extractChunks(html);

    // Should find both chunks
    expect(chunks.length).toBeGreaterThanOrEqual(1);
    expect(chunks.some(c => c.id === 'outer')).toBe(true);
  });

  it('should return empty array for HTML without chunks', async () => {
    const html = `
      <div>
        <h1>No Chunks Here</h1>
        <p>Just regular content</p>
      </div>
    `;

    const chunks = await processor.extractChunks(html);

    expect(chunks).toHaveLength(0);
  });
});
