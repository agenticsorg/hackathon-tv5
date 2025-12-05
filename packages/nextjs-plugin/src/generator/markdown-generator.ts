/**
 * Markdown Generator Module
 * Generates ARW-compliant .llm.md files from config and chunks
 */

import type { ARWConfig } from './ast-parser';
import type { ChunkData } from './html-processor';

export class MarkdownGenerator {
  /**
   * Generate markdown content from ARW config and extracted chunks
   */
  generate(config: ARWConfig, chunks: ChunkData[]): string {
    const lines: string[] = [];

    // Add ARW metadata comment (no page title)
    lines.push('<!-- ARW Metadata');
    lines.push(`priority: ${config.priority || 'normal'}`);
    lines.push(`purpose: ${config.purpose || 'general'}`);
    lines.push('-->');
    lines.push('');

    // Sort chunks by order
    const sortedChunks = [...chunks].sort((a, b) => a.order - b.order);

    // Add each chunk
    for (const chunk of sortedChunks) {
      this.addChunk(lines, chunk);
    }

    return lines.join('\n');
  }

  /**
   * Clean markdown content by removing emojis and special characters
   */
  private cleanContent(content: string): string {
    let cleaned = content;

    // Remove emojis (Unicode ranges for common emoji)
    cleaned = cleaned.replace(/[\u{1F600}-\u{1F64F}]/gu, ''); // Emoticons
    cleaned = cleaned.replace(/[\u{1F300}-\u{1F5FF}]/gu, ''); // Misc Symbols and Pictographs
    cleaned = cleaned.replace(/[\u{1F680}-\u{1F6FF}]/gu, ''); // Transport and Map
    cleaned = cleaned.replace(/[\u{1F1E0}-\u{1F1FF}]/gu, ''); // Flags
    cleaned = cleaned.replace(/[\u{2600}-\u{26FF}]/gu, '');   // Misc symbols
    cleaned = cleaned.replace(/[\u{2700}-\u{27BF}]/gu, '');   // Dingbats
    cleaned = cleaned.replace(/[\u{FE00}-\u{FE0F}]/gu, '');   // Variation Selectors
    cleaned = cleaned.replace(/[\u{1F900}-\u{1F9FF}]/gu, ''); // Supplemental Symbols and Pictographs
    cleaned = cleaned.replace(/[\u{1FA00}-\u{1FA6F}]/gu, ''); // Chess Symbols
    cleaned = cleaned.replace(/[\u{1FA70}-\u{1FAFF}]/gu, ''); // Symbols and Pictographs Extended-A

    // Convert special bullets to standard markdown
    cleaned = cleaned.replace(/[•●◦▪▫]/g, '-');

    // Convert check marks to text
    cleaned = cleaned.replace(/[✓✔☑]/g, '[x]');
    cleaned = cleaned.replace(/[✗✘☒]/g, '[ ]');

    // Remove zero-width spaces and other invisible characters
    cleaned = cleaned.replace(/[\u200B-\u200D\uFEFF]/g, '');

    // Remove empty bullet points (bullets with only whitespace)
    cleaned = cleaned.replace(/^-\s*$/gm, '');

    // Clean up multiple spaces
    cleaned = cleaned.replace(/  +/g, ' ');

    // Remove multiple consecutive blank lines
    cleaned = cleaned.replace(/\n\n\n+/g, '\n\n');

    // Remove empty lines at start
    cleaned = cleaned.replace(/^\s+/, '');

    return cleaned.trim();
  }

  /**
   * Remove duplicate consecutive headings and clean up bullets
   */
  private removeDuplicateHeadings(content: string): string {
    const lines = content.split('\n');
    const result: string[] = [];
    let lastHeadingText: string | null = null;

    for (const line of lines) {
      const trimmed = line.trim();

      // Check if line is a heading (any level)
      const headingMatch = trimmed.match(/^(#{1,6})\s+(.+)$/);
      if (headingMatch) {
        const headingText = headingMatch[2].trim();

        // Skip if same heading text as previous (regardless of level)
        if (headingText === lastHeadingText) {
          continue;
        }

        lastHeadingText = headingText;
        result.push(line);
      } else {
        // Not a heading - reset tracking and add line
        if (trimmed.length > 0) {
          lastHeadingText = null;
        }

        // Fix double dashes in bullets
        let fixedLine = line;
        if (trimmed.match(/^-\s+-\s+/)) {
          // Convert "- - " to "- "
          fixedLine = line.replace(/^(\s*)-\s+-\s+/, '$1- ');
        }

        result.push(fixedLine);
      }
    }

    return result.join('\n');
  }

  /**
   * Add a chunk to the markdown lines
   */
  private addChunk(lines: string[], chunk: ChunkData): void {
    // Add chunk comment
    lines.push(`<!-- chunk: ${chunk.id} -->`);

    // Clean and deduplicate content first
    let content = '';
    if (chunk.content) {
      content = this.cleanContent(chunk.content);
      content = this.removeDuplicateHeadings(content);
    }

    // Check if content already contains the chunk heading (at any level)
    const cleanedHeading = chunk.heading ? this.cleanContent(chunk.heading) : '';
    const headingInContent = cleanedHeading && content.match(
      new RegExp(`^#{1,6}\\s+${cleanedHeading.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`, 'm')
    );

    // Add heading if present and not already in content
    if (cleanedHeading && !headingInContent) {
      lines.push(`## ${cleanedHeading}`);
      lines.push('');
    }

    // Add content
    if (content) {
      lines.push(content);
      lines.push('');
    }
  }

  /**
   * Generate markdown from config only (no HTML chunks)
   * Useful for pages without data-chunk-id attributes
   */
  generateFromConfigOnly(config: ARWConfig): string {
    const lines: string[] = [];

    // Add ARW metadata comment (no page title)
    lines.push('<!-- ARW Metadata');
    lines.push(`priority: ${config.priority || 'normal'}`);
    lines.push(`purpose: ${config.purpose || 'general'}`);
    lines.push('-->');
    lines.push('');

    // Add chunks from config if available
    if (config.chunks && config.chunks.length > 0) {
      for (const chunk of config.chunks) {
        lines.push(`<!-- chunk: ${chunk.id} -->`);

        if (chunk.heading) {
          const cleanedHeading = this.cleanContent(chunk.heading);
          lines.push(`## ${cleanedHeading}`);
          lines.push('');
        }

        if (chunk.description) {
          const cleanedDesc = this.cleanContent(chunk.description);
          lines.push(cleanedDesc);
          lines.push('');
        }
      }
    }

    return lines.join('\n');
  }
}
