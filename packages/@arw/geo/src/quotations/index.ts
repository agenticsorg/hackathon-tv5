/**
 * ARW-GEO-3: Quotation System
 * Provides +40% visibility improvement through expert quotations
 */

import type { Quotation, Speaker } from '../types';

export class QuotationSystem {
  /**
   * Extract quotations from markdown content
   */
  extractFromMarkdown(content: string): Quotation[] {
    const quotations: Quotation[] = [];

    // Match quotation blocks
    const quotePattern = /<!-- quote:([\w-]+) -->\s*>\s*"([^"]+)"\s*```quotation\s*(.*?)\s*```/gs;
    const matches = Array.from(content.matchAll(quotePattern));

    for (const match of matches) {
      const [, quoteId, text, metadataJson] = match;
      try {
        const metadata = JSON.parse(metadataJson) as Quotation;
        metadata.id = `quote:${quoteId}`;
        metadata.text = text;
        quotations.push(metadata);
      } catch (error) {
        console.warn(`Failed to parse quotation ${quoteId}:`, error);
      }
    }

    return quotations;
  }

  /**
   * Calculate speaker authority score
   */
  calculateSpeakerAuthority(speaker: Speaker): number {
    let score = 0.5; // Base score

    // Credentials
    if (speaker.credentials) {
      if (speaker.credentials.some(c => c.includes('PhD'))) score += 0.2;
      if (speaker.credentials.some(c => c.includes('Professor'))) score += 0.15;
      if (speaker.credentials.some(c => /\d+\s*years?/.test(c))) score += 0.1;
    }

    // Title authority
    if (speaker.title.match(/Chief|Director|VP|Head/i)) score += 0.15;
    if (speaker.title.match(/CEO|President|Founder/i)) score += 0.1;

    // Verification
    if (speaker.verified) score += 0.1;

    return Math.min(1.0, score);
  }

  /**
   * Validate quotation structure
   */
  validate(quotation: Quotation): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!quotation.id) errors.push('Missing quotation ID');
    if (!quotation.text) errors.push('Missing quotation text');
    if (!quotation.speaker) errors.push('Missing speaker information');
    if (!quotation.source) errors.push('Missing source');
    if (!quotation.date) errors.push('Missing date');
    if (!quotation.type) errors.push('Missing quotation type');

    // Validate speaker
    if (quotation.speaker) {
      if (!quotation.speaker.name) errors.push('Missing speaker name');
      if (!quotation.speaker.title) errors.push('Missing speaker title');
      if (!quotation.speaker.affiliation) errors.push('Missing speaker affiliation');
    }

    // Validate date format
    if (quotation.date && isNaN(Date.parse(quotation.date))) {
      errors.push('Invalid date format (must be ISO 8601)');
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }

  /**
   * Format quotation for machine view
   */
  formatForMachineView(quotations: Quotation[]): string {
    let output = '';

    for (const quote of quotations) {
      output += `<!-- ${quote.id} -->\n`;
      output += `> "${quote.text}"\n\n`;
      output += '```quotation\n';
      output += JSON.stringify(quote, null, 2);
      output += '\n```\n\n';
    }

    return output;
  }

  /**
   * Convert quotation to Schema.org
   */
  toSchemaOrg(quotation: Quotation): object {
    return {
      '@type': 'Quotation',
      text: quotation.text,
      spokenByCharacter: {
        '@type': 'Person',
        name: quotation.speaker.name,
        jobTitle: quotation.speaker.title,
        worksFor: {
          '@type': 'Organization',
          name: quotation.speaker.affiliation
        },
        hasCredential: quotation.speaker.credentials?.map(c => ({
          '@type': 'EducationalOccupationalCredential',
          credentialCategory: c
        }))
      },
      datePublished: quotation.date,
      isPartOf: {
        '@type': 'Event',
        name: quotation.source
      }
    };
  }

  /**
   * Extract direct quotes from text
   */
  extractFromText(text: string): Array<{
    quote: string;
    context: string;
    position: number;
  }> {
    const quotes: Array<{ quote: string; context: string; position: number }> = [];

    // Match quoted text
    const quotePattern = /"([^"]{20,500})"/g;
    const matches = Array.from(text.matchAll(quotePattern));

    for (const match of matches) {
      const quote = match[1];
      const position = match.index || 0;

      // Get context (100 chars before and after)
      const context = text.substring(
        Math.max(0, position - 100),
        Math.min(text.length, position + match[0].length + 100)
      ).trim();

      quotes.push({ quote, context, position });
    }

    return quotes;
  }
}
