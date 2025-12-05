/**
 * ARW Discovery utilities for llms.txt detection
 */

import { HttpClient } from '../utils/http';
import { ARWMetadata, APIResponse } from '../types';
import { ValidationError } from '../utils/errors';

export class ARWDiscovery {
  constructor(private http: HttpClient) {}

  /**
   * Discover ARW llms.txt file for a URL
   */
  async discover(url: string): Promise<ARWMetadata> {
    this.validateUrl(url);

    const response = await this.http.post<APIResponse<ARWMetadata>>(
      '/arw/discover',
      { url }
    );

    if (!response.success || !response.data) {
      throw new ValidationError(
        response.error || 'ARW discovery failed',
        response
      );
    }

    return response.data;
  }

  /**
   * Check if a URL has ARW support
   */
  async hasSupport(url: string): Promise<boolean> {
    try {
      const metadata = await this.discover(url);
      return metadata.discovered;
    } catch {
      return false;
    }
  }

  /**
   * Get llms.txt content
   */
  async getLlmsTxt(url: string): Promise<string | null> {
    const metadata = await this.discover(url);

    if (!metadata.discovered || !metadata.llmsTxtContent) {
      return null;
    }

    return metadata.llmsTxtContent;
  }

  /**
   * Parse llms.txt content into structured sections
   */
  parseLlmsTxt(content: string): Array<{
    title: string;
    url: string;
    description?: string;
  }> {
    const sections: Array<{
      title: string;
      url: string;
      description?: string;
    }> = [];

    const lines = content.split('\n');
    let currentSection: {
      title: string;
      url: string;
      description?: string;
    } | null = null;

    for (const line of lines) {
      const trimmed = line.trim();

      // Skip empty lines and comments
      if (!trimmed || trimmed.startsWith('#')) {
        continue;
      }

      // Section header (markdown link format)
      const linkMatch = trimmed.match(/^\[([^\]]+)\]\(([^)]+)\)$/);
      if (linkMatch) {
        if (currentSection) {
          sections.push(currentSection);
        }
        currentSection = {
          title: linkMatch[1],
          url: linkMatch[2],
        };
        continue;
      }

      // Description line (starts with -)
      if (trimmed.startsWith('-') && currentSection) {
        const description = trimmed.substring(1).trim();
        if (!currentSection.description) {
          currentSection.description = description;
        } else {
          currentSection.description += ' ' + description;
        }
      }
    }

    // Add last section
    if (currentSection) {
      sections.push(currentSection);
    }

    return sections;
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
 * Standalone discovery helper functions
 */
export class ARWHelper {
  /**
   * Generate potential llms.txt URLs for a given URL
   */
  static generateLlmsTxtUrls(url: string): string[] {
    const parsedUrl = new URL(url);
    const baseUrl = `${parsedUrl.protocol}//${parsedUrl.host}`;

    return [
      `${baseUrl}/.well-known/llms.txt`, // Standard location
      `${baseUrl}/llms.txt`,              // Root location
      `${baseUrl}/.well-known/ai-plugin.json`, // Alternative
    ];
  }

  /**
   * Extract domain from URL
   */
  static extractDomain(url: string): string {
    const parsedUrl = new URL(url);
    return parsedUrl.hostname;
  }

  /**
   * Check if URL is likely to have ARW support based on common patterns
   */
  static likelyHasSupport(url: string): boolean {
    const domain = this.extractDomain(url);

    // Common domains/patterns known to support ARW
    const supportedPatterns = [
      /docs?\./i,
      /api\./i,
      /developer\./i,
      /guide\./i,
      /reference\./i,
    ];

    return supportedPatterns.some(pattern => pattern.test(domain));
  }

  /**
   * Normalize llms.txt URL
   */
  static normalizeLlmsTxtUrl(baseUrl: string, llmsTxtPath?: string): string {
    if (!llmsTxtPath) {
      return `${baseUrl}/.well-known/llms.txt`;
    }

    if (llmsTxtPath.startsWith('http')) {
      return llmsTxtPath;
    }

    const base = baseUrl.endsWith('/') ? baseUrl.slice(0, -1) : baseUrl;
    const path = llmsTxtPath.startsWith('/') ? llmsTxtPath : `/${llmsTxtPath}`;

    return `${base}${path}`;
  }
}
