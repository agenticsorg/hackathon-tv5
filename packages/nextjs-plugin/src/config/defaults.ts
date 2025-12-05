/**
 * Default configuration values for ARW Next.js Plugin
 */

import type { ARWNextConfig, ARWPolicies } from '../types';

export const defaultPolicies: ARWPolicies = {
  training: {
    allowed: false,
    note: 'Content not licensed for model training'
  },
  inference: {
    allowed: true,
    restrictions: ['attribution_required']
  },
  attribution: {
    required: true,
    format: 'link'
  }
};

export const defaultConfig: Required<ARWNextConfig> = {
  autoGenerate: false,
  outputDir: 'public',
  watch: process.env.NODE_ENV === 'development',
  manifest: {
    siteName: '',
    homepage: '',
    contact: '',
    policies: defaultPolicies
  },
  generation: {
    format: 'markdown',
    chunkStrategy: 'semantic',
    includePatterns: ['**/*.tsx', '**/*.jsx'],
    excludePatterns: ['node_modules/**', '.next/**', 'dist/**']
  },
  runtime: {
    enableClientGeneration: false,
    cacheStrategy: 'memory',
    ttl: 3600000 // 1 hour
  }
};

export function mergeConfig(userConfig: ARWNextConfig): Required<ARWNextConfig> {
  return {
    ...defaultConfig,
    ...userConfig,
    manifest: {
      ...defaultConfig.manifest,
      ...userConfig.manifest,
      policies: {
        ...defaultConfig.manifest.policies,
        ...userConfig.manifest?.policies
      }
    },
    generation: {
      ...defaultConfig.generation,
      ...userConfig.generation
    },
    runtime: {
      ...defaultConfig.runtime,
      ...userConfig.runtime
    }
  };
}
