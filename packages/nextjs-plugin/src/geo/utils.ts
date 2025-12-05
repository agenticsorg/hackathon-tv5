/**
 * GEO Utilities for Next.js Plugin
 */

import type { GEOConfig } from './types';

export const defaultGEOConfig: Partial<GEOConfig> = {
  enabled: false,
  profile: 'ARW-2.2',

  analysis: {
    extractCitations: true,
    extractStatistics: true,
    extractQuotations: true,
    calculateQuality: true,
    extractEntities: true,
    useLLM: false,
  },

  output: {
    generateReports: true,
    reportFormat: 'json',
    reportDir: './output/geo-reports',
    includeInLLMFiles: true,
  },

  dashboard: {
    enabled: true,
    route: '/dev/geo',
    devOnly: true,
  },
};

export function mergeGEOConfig(userConfig: Partial<GEOConfig> = {}): GEOConfig {
  return {
    enabled: userConfig.enabled ?? false,
    profile: userConfig.profile ?? defaultGEOConfig.profile!,
    domain: userConfig.domain,

    analysis: {
      ...defaultGEOConfig.analysis,
      ...userConfig.analysis,
    },

    llm: userConfig.llm,

    output: {
      ...defaultGEOConfig.output,
      ...userConfig.output,
    },

    dashboard: {
      ...defaultGEOConfig.dashboard,
      ...userConfig.dashboard,
    },
  };
}

/**
 * Calculate estimated visibility improvement based on GEO profile
 */
export function calculateVisibilityBoost(profile: 'ARW-2.1' | 'ARW-2.2'): number {
  if (profile === 'ARW-2.1') {
    // Foundation GEO
    return 170; // +140-170%
  } else {
    // Advanced GEO
    return 290; // +230-290%
  }
}

/**
 * Get GEO profile description
 */
export function getProfileDescription(profile: 'ARW-2.1' | 'ARW-2.2'): string {
  if (profile === 'ARW-2.1') {
    return 'Foundation GEO: Citations, Statistics, Quotations, Domain Optimization';
  } else {
    return 'Advanced GEO: All Foundation features + Quality Signals, Entity Enrichment, Semantic Clustering';
  }
}
