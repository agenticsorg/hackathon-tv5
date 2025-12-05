/**
 * Machine View utilities for content optimization
 */

import { HttpClient } from '../utils/http';
import {
  MachineViewData,
  MachineViewSection,
  ScrapeResult,
  APIResponse,
} from '../types';
import { ValidationError } from '../utils/errors';

export class MachineView {
  constructor(private http: HttpClient) {}

  /**
   * Generate machine-optimized view of content
   */
  async generate(
    url: string,
    options?: {
      maxTokens?: number;
      prioritySections?: string[];
    }
  ): Promise<MachineViewData> {
    this.validateUrl(url);

    const response = await this.http.post<APIResponse<MachineViewData>>(
      '/arw/machine-view',
      {
        url,
        ...options,
      }
    );

    if (!response.success || !response.data) {
      throw new ValidationError(
        response.error || 'Machine view generation failed',
        response
      );
    }

    return response.data;
  }

  /**
   * Extract structured sections from scrape result
   */
  extractSections(result: ScrapeResult): MachineViewSection[] {
    const sections: MachineViewSection[] = [];

    if (!result.markdown) {
      return sections;
    }

    // Split by markdown headers
    const lines = result.markdown.split('\n');
    let currentSection: MachineViewSection | null = null;
    let currentContent: string[] = [];

    for (const line of lines) {
      // Check for header
      const headerMatch = line.match(/^(#{1,6})\s+(.+)$/);

      if (headerMatch) {
        // Save previous section
        if (currentSection && currentContent.length > 0) {
          currentSection.content = currentContent.join('\n').trim();
          sections.push(currentSection);
        }

        // Start new section
        const level = headerMatch[1].length;
        const title = headerMatch[2];

        currentSection = {
          type: `heading-${level}`,
          content: '',
          priority: this.calculatePriority(level, title),
          metadata: {
            level,
            title,
          },
        };
        currentContent = [];
      } else if (currentSection) {
        currentContent.push(line);
      }
    }

    // Add last section
    if (currentSection && currentContent.length > 0) {
      currentSection.content = currentContent.join('\n').trim();
      sections.push(currentSection);
    }

    return sections;
  }

  /**
   * Optimize sections for token budget
   */
  optimizeForTokens(
    sections: MachineViewSection[],
    maxTokens: number
  ): MachineViewSection[] {
    // Rough estimate: 1 token ≈ 4 characters
    const maxChars = maxTokens * 4;

    // Sort by priority (higher first)
    const sorted = [...sections].sort(
      (a, b) => (b.priority || 0) - (a.priority || 0)
    );

    const optimized: MachineViewSection[] = [];
    let currentChars = 0;

    for (const section of sorted) {
      const sectionChars = section.content.length;

      if (currentChars + sectionChars <= maxChars) {
        optimized.push(section);
        currentChars += sectionChars;
      } else if (currentChars < maxChars) {
        // Truncate last section to fit
        const remaining = maxChars - currentChars;
        optimized.push({
          ...section,
          content: section.content.substring(0, remaining) + '...',
        });
        break;
      }
    }

    // Restore original order
    return optimized.sort((a, b) => {
      const aIndex = sections.indexOf(sections.find(s => s === a) || sections[0]);
      const bIndex = sections.indexOf(sections.find(s => s === b) || sections[0]);
      return aIndex - bIndex;
    });
  }

  /**
   * Convert sections to markdown
   */
  toMarkdown(sections: MachineViewSection[]): string {
    return sections
      .map((section) => {
        const level = section.metadata?.level || 1;
        const title = section.metadata?.title || 'Section';
        const header = '#'.repeat(level) + ' ' + title;
        return `${header}\n\n${section.content}`;
      })
      .join('\n\n');
  }

  /**
   * Convert sections to JSON
   */
  toJSON(sections: MachineViewSection[]): string {
    return JSON.stringify(
      sections.map((section) => ({
        type: section.type,
        title: section.metadata?.title,
        content: section.content,
        priority: section.priority,
      })),
      null,
      2
    );
  }

  /**
   * Calculate section priority based on heading level and title
   */
  private calculatePriority(level: number, title: string): number {
    let priority = 10 - level; // Higher levels get higher priority

    // Boost priority for important keywords
    const importantKeywords = [
      'introduction',
      'overview',
      'getting started',
      'api',
      'reference',
      'guide',
    ];

    const lowerTitle = title.toLowerCase();
    if (importantKeywords.some((keyword) => lowerTitle.includes(keyword))) {
      priority += 5;
    }

    return priority;
  }

  /**
   * Validate URL format
   */
  private validateUrl(url: string): void {
    if (!url || typeof url !== 'string') {
      throw new ValidationError('URL must be a non-empty string');
    }

    try {
      new URL(url);
    } catch {
      throw new ValidationError(`Invalid URL format: ${url}`);
    }
  }
}

/**
 * Content extraction helpers
 */
export class ContentExtractor {
  /**
   * Extract code blocks from markdown
   */
  static extractCodeBlocks(markdown: string): Array<{
    language: string;
    code: string;
  }> {
    const codeBlocks: Array<{ language: string; code: string }> = [];
    const regex = /```(\w+)?\n([\s\S]*?)```/g;
    let match;

    while ((match = regex.exec(markdown)) !== null) {
      codeBlocks.push({
        language: match[1] || 'plaintext',
        code: match[2].trim(),
      });
    }

    return codeBlocks;
  }

  /**
   * Extract links from markdown
   */
  static extractLinks(markdown: string): Array<{
    text: string;
    url: string;
  }> {
    const links: Array<{ text: string; url: string }> = [];
    const regex = /\[([^\]]+)\]\(([^)]+)\)/g;
    let match;

    while ((match = regex.exec(markdown)) !== null) {
      links.push({
        text: match[1],
        url: match[2],
      });
    }

    return links;
  }

  /**
   * Extract tables from markdown
   */
  static extractTables(markdown: string): string[] {
    const tables: string[] = [];
    const lines = markdown.split('\n');
    let currentTable: string[] = [];
    let inTable = false;

    for (const line of lines) {
      const isTableRow = line.trim().startsWith('|');

      if (isTableRow) {
        inTable = true;
        currentTable.push(line);
      } else if (inTable) {
        tables.push(currentTable.join('\n'));
        currentTable = [];
        inTable = false;
      }
    }

    if (currentTable.length > 0) {
      tables.push(currentTable.join('\n'));
    }

    return tables;
  }

  /**
   * Remove markdown formatting
   */
  static stripMarkdown(markdown: string): string {
    return markdown
      .replace(/#+\s/g, '') // Headers
      .replace(/\*\*([^*]+)\*\*/g, '$1') // Bold
      .replace(/\*([^*]+)\*/g, '$1') // Italic
      .replace(/`([^`]+)`/g, '$1') // Inline code
      .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1') // Links
      .replace(/```[\s\S]*?```/g, '') // Code blocks
      .trim();
  }
}
