/**
 * ARW-GEO-2: Statistics Enhancement
 * Provides +40% visibility improvement through structured statistical data
 */

import type { Statistic } from '../types';

export class StatisticsEnhancement {
  /**
   * Extract statistics from markdown content
   */
  extractFromMarkdown(content: string): Statistic[] {
    const statistics: Statistic[] = [];

    // Match statistics blocks
    const statPattern = /```statistics\s*\n(.*?)\n```/gs;
    const matches = Array.from(content.matchAll(statPattern));

    for (const match of matches) {
      try {
        const stat = JSON.parse(match[1]) as Statistic;
        statistics.push(stat);
      } catch (error) {
        console.warn('Failed to parse statistic:', error);
      }
    }

    return statistics;
  }

  /**
   * Calculate freshness score based on date
   */
  calculateFreshnessScore(stat: Partial<Statistic>): number {
    if (!stat.date) return 0.5;

    const statDate = new Date(stat.date);
    const age = Date.now() - statDate.getTime();
    const daysOld = age / (1000 * 60 * 60 * 24);

    if (daysOld < 30) return 1.0;
    if (daysOld < 90) return 0.9;
    if (daysOld < 180) return 0.7;
    if (daysOld < 365) return 0.5;
    return 0.3;
  }

  /**
   * Validate statistic structure
   */
  validate(stat: Statistic): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!stat.id) errors.push('Missing statistic ID');
    if (!stat.type) errors.push('Missing type');
    if (stat.value === undefined || stat.value === null) {
      errors.push('Missing value');
    }
    if (!stat.unit) errors.push('Missing unit');
    if (!stat.date) errors.push('Missing date');
    if (!stat.source) errors.push('Missing source');

    // Validate date format
    if (stat.date && isNaN(Date.parse(stat.date))) {
      errors.push('Invalid date format (must be ISO 8601)');
    }

    // Validate confidence score
    if (stat.confidence !== undefined) {
      if (stat.confidence < 0 || stat.confidence > 1) {
        errors.push('Confidence must be between 0 and 1');
      }
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }

  /**
   * Format statistic for machine view
   */
  formatForMachineView(statistics: Statistic[]): string {
    let output = '';

    for (const stat of statistics) {
      output += `<!-- stat:${stat.id} -->\n`;
      output += `**${stat.id}**\n\n`;
      output += '```statistics\n';
      output += JSON.stringify(stat, null, 2);
      output += '\n```\n\n';
    }

    return output;
  }

  /**
   * Convert statistic to Schema.org Dataset
   */
  toSchemaOrg(stat: Statistic): object {
    return {
      '@type': 'Dataset',
      name: stat.id,
      description: `Statistical value: ${stat.value} ${stat.unit}`,
      temporalCoverage: stat.date,
      variableMeasured: stat.type,
      measurementTechnique: stat.methodology,
      citation: {
        '@type': 'CreativeWork',
        name: stat.source,
        url: stat.source_url
      }
    };
  }

  /**
   * Extract numerical statistics from text using NLP
   */
  extractFromText(text: string): Array<{
    value: number;
    unit: string;
    context: string;
  }> {
    const results: Array<{ value: number; unit: string; context: string }> = [];

    // Pattern for numbers with units
    const pattern = /(\d+(?:,\d{3})*(?:\.\d+)?)\s*(%|percent|billion|million|thousand|USD|dollars?|euros?|pounds?|hours?|days?|years?|months?)/gi;
    const matches = Array.from(text.matchAll(pattern));

    for (const match of matches) {
      const value = parseFloat(match[1].replace(/,/g, ''));
      const unit = match[2].toLowerCase();
      const startIndex = match.index || 0;

      // Get context (50 chars before and after)
      const context = text.substring(
        Math.max(0, startIndex - 50),
        Math.min(text.length, startIndex + match[0].length + 50)
      ).trim();

      results.push({ value, unit, context });
    }

    return results;
  }
}
