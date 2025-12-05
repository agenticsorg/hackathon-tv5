/**
 * Tests for Markdown Generator
 */

import { describe, it, expect } from 'vitest';
import { MarkdownGenerator } from '../../src/generator/markdown-generator';
import type { ARWConfig } from '../../src/generator/ast-parser';
import type { ChunkData } from '../../src/generator/html-processor';

describe('MarkdownGenerator', () => {
  const generator = new MarkdownGenerator();

  it('should generate markdown with title and metadata', () => {
    const config: ARWConfig = {
      title: 'Test Page',
      description: 'A test page',
      priority: 'high',
      purpose: 'testing'
    };

    const chunks: ChunkData[] = [];

    const markdown = generator.generate(config, chunks);

    expect(markdown).toContain('# Test Page');
    expect(markdown).toContain('A test page');
    expect(markdown).toContain('<!-- ARW Metadata');
    expect(markdown).toContain('priority: high');
    expect(markdown).toContain('purpose: testing');
  });

  it('should include chunks in correct order', () => {
    const config: ARWConfig = {
      title: 'Test Page'
    };

    const chunks: ChunkData[] = [
      { id: 'intro', heading: 'Introduction', content: 'Intro text', order: 0 },
      { id: 'content', heading: 'Content', content: 'Main content', order: 1 }
    ];

    const markdown = generator.generate(config, chunks);

    expect(markdown).toContain('<!-- chunk: intro -->');
    expect(markdown).toContain('## Introduction');
    expect(markdown).toContain('Intro text');
    expect(markdown).toContain('<!-- chunk: content -->');
    expect(markdown).toContain('## Content');
    expect(markdown).toContain('Main content');

    // Check order
    const introIndex = markdown.indexOf('<!-- chunk: intro -->');
    const contentIndex = markdown.indexOf('<!-- chunk: content -->');
    expect(introIndex).toBeLessThan(contentIndex);
  });

  it('should handle chunks without headings', () => {
    const config: ARWConfig = {
      title: 'Test Page'
    };

    const chunks: ChunkData[] = [
      { id: 'footer', content: 'Footer text', order: 0 }
    ];

    const markdown = generator.generate(config, chunks);

    expect(markdown).toContain('<!-- chunk: footer -->');
    expect(markdown).toContain('Footer text');
    expect(markdown).not.toContain('##');
  });

  it('should use default values for missing fields', () => {
    const config: ARWConfig = {};
    const chunks: ChunkData[] = [];

    const markdown = generator.generate(config, chunks);

    expect(markdown).toContain('# Page');
    expect(markdown).toContain('priority: normal');
    expect(markdown).toContain('purpose: general');
  });

  it('should generate from config only when no chunks', () => {
    const config: ARWConfig = {
      title: 'Config Only',
      description: 'Generated from config',
      priority: 'low',
      chunks: [
        { id: 'section1', heading: 'Section 1', description: 'First section' },
        { id: 'section2', heading: 'Section 2' }
      ]
    };

    const markdown = generator.generateFromConfigOnly(config);

    expect(markdown).toContain('# Config Only');
    expect(markdown).toContain('Generated from config');
    expect(markdown).toContain('<!-- chunk: section1 -->');
    expect(markdown).toContain('## Section 1');
    expect(markdown).toContain('First section');
    expect(markdown).toContain('<!-- chunk: section2 -->');
  });
});
