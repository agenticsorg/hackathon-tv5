/**
 * Type definitions for ARW Next.js Plugin
 */

export interface ARWPolicies {
  training?: {
    allowed: boolean;
    note?: string;
  };
  inference?: {
    allowed: boolean;
    restrictions?: string[];
  };
  attribution?: {
    required: boolean;
    format?: 'link' | 'text';
    template?: string;
  };
  rate_limit?: {
    authenticated?: string;
    unauthenticated?: string;
  };
}

export interface ARWManifestConfig {
  siteName: string;
  description?: string;
  homepage: string;
  contact: string;
  policies?: ARWPolicies;
}

export interface ARWGenerationOptions {
  format?: 'markdown' | 'toon';
  chunkStrategy?: 'semantic' | 'size';
  includePatterns?: string[];
  excludePatterns?: string[];
  buildDir?: string;
  sourceDir?: string;
  cleanStale?: boolean;
}

export interface ARWRuntimeOptions {
  enableClientGeneration?: boolean;
  cacheStrategy?: 'memory' | 'filesystem';
  ttl?: number;
}

export interface ARWGEOConfig {
  enabled: boolean;
  profile: 'ARW-2.1' | 'ARW-2.2';
  domain?: string;
  analysis?: {
    extractCitations?: boolean;
    extractStatistics?: boolean;
    extractQuotations?: boolean;
    calculateQuality?: boolean;
    extractEntities?: boolean;
    useLLM?: boolean;
  };
  llm?: {
    provider: 'anthropic' | 'openai';
    model?: string;
    apiKey?: string;
  };
  output?: {
    generateReports?: boolean;
    reportFormat?: 'json' | 'html' | 'markdown';
    reportDir?: string;
    includeInLLMFiles?: boolean;
  };
  dashboard?: {
    enabled?: boolean;
    route?: string;
    devOnly?: boolean;
  };
}

export interface ARWNextConfig {
  /** Enable auto-generation of machine views */
  autoGenerate?: boolean;

  /** Output directory for ARW files */
  outputDir?: string;

  /** Enable watch mode in development */
  watch?: boolean;

  /** ARW manifest configuration */
  manifest?: ARWManifestConfig;

  /** Machine view generation options */
  generation?: ARWGenerationOptions;

  /** Runtime options */
  runtime?: ARWRuntimeOptions;

  /** GEO (Generative Engine Optimization) configuration */
  geo?: ARWGEOConfig;
}

export interface ARWChunk {
  id: string;
  heading?: string;
  description?: string;
  url_fragment?: string;
}

export interface PageMetadata {
  title?: string;
  description?: string;
  purpose?: string;
  priority?: 'low' | 'normal' | 'high';
  chunks?: ARWChunk[];
}

export interface ARWManifest {
  version: string;
  profile: string;
  site: {
    name: string;
    description?: string;
    homepage: string;
    contact: string;
  };
  content: ARWContentItem[];
  policies?: ARWPolicies;
}

export interface ARWContentItem {
  url: string;
  machine_view: string;
  purpose?: string;
  priority?: string;
  chunks?: ARWChunk[];
}

export interface MachineView {
  content: string;
  metadata: PageMetadata;
}

export interface ARWHeadProps {
  priority?: 'low' | 'normal' | 'high';
  machineViewPath?: string;
}

export interface ARWProviderProps {
  children: React.ReactNode;
  config?: {
    priority?: 'low' | 'normal' | 'high';
    purpose?: string;
  };
}

export interface UseARWReturn {
  manifest: ARWManifest | undefined;
  loading: boolean;
  error: Error | null;
}
