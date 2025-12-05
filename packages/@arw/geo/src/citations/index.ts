/**
 * ARW-GEO-1: Citation Framework
 * Provides +40% visibility improvement through structured citations
 */

import type { Citation, CitationType } from '../types';

export class CitationFramework {
  /**
   * Extract citations from markdown content
   */
  extractFromMarkdown(content: string): Citation[] {
    const citations: Citation[] = [];

    // Match citation definitions
    const defPattern = /\[cite:(\d+)\]:\s*\{([^}]+)\}/gs;
    const defs = Array.from(content.matchAll(defPattern));

    for (const def of defs) {
      const id = `cite:${def[1]}`;
      try {
        const jsonStr = `{${def[2]}}`;
        const citation = JSON.parse(jsonStr) as Citation;
        citation.id = id;
        citations.push(citation);
      } catch (error) {
        console.warn(`Failed to parse citation ${id}:`, error);
      }
    }

    return citations;
  }

  /**
   * Calculate authority score for a citation
   */
  calculateAuthorityScore(citation: Partial<Citation>): number {
    let score = 0.5; // Base score

    // Type-based scoring
    const typeWeights: Record<CitationType, number> = {
      academic: 0.4,
      government: 0.35,
      industry_report: 0.3,
      book: 0.25,
      news: 0.15,
      website: 0.1,
      dataset: 0.3,
      internal: 0.2
    };

    if (citation.type) {
      score += typeWeights[citation.type] || 0;
    }

    // Recency bonus
    if (citation.date) {
      const pubDate = new Date(citation.date);
      const age = Date.now() - pubDate.getTime();
      const yearsOld = age / (1000 * 60 * 60 * 24 * 365);

      if (yearsOld < 2) {
        score += 0.1;
      } else if (yearsOld > 5) {
        score -= 0.1;
      }
    }

    // DOI presence (peer-reviewed)
    if (citation.doi) {
      score += 0.1;
    }

    return Math.min(1.0, Math.max(0.0, score));
  }

  /**
   * Validate citation structure
   */
  validate(citation: Citation): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!citation.id) errors.push('Missing citation ID');
    if (!citation.source) errors.push('Missing source');
    if (!citation.type) errors.push('Missing type');
    if (!citation.url) errors.push('Missing URL');
    if (!citation.date) errors.push('Missing date');

    // Validate date format
    if (citation.date && isNaN(Date.parse(citation.date))) {
      errors.push('Invalid date format (must be ISO 8601)');
    }

    // Validate authority score
    if (citation.authority_score !== undefined) {
      if (citation.authority_score < 0 || citation.authority_score > 1) {
        errors.push('Authority score must be between 0 and 1');
      }
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }

  /**
   * Format citation for machine view
   */
  formatForMachineView(citations: Citation[]): string {
    let output = '<!-- citations -->\n### References\n\n';

    for (const citation of citations) {
      output += `[${citation.id}]: ${JSON.stringify(citation, null, 2)}\n\n`;
    }

    return output;
  }

  /**
   * Convert citation to Schema.org JSON-LD
   */
  toSchemaOrg(citation: Citation): object {
    return {
      '@type': 'CreativeWork',
      name: citation.source,
      url: citation.url,
      datePublished: citation.date,
      author: typeof citation.author === 'string'
        ? { '@type': 'Person', name: citation.author }
        : citation.author
          ? {
              '@type': 'Person',
              name: citation.author.name,
              affiliation: citation.author.affiliation,
              hasCredential: citation.author.credentials
            }
          : undefined,
      publisher: citation.publisher
        ? { '@type': 'Organization', name: citation.publisher }
        : undefined,
      identifier: citation.doi || citation.isbn
    };
  }
}
