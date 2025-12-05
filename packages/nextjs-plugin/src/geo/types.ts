/**
 * GEO Configuration Types for Next.js Plugin
 */

export interface GEOConfig {
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

export interface GEOAnalysisResult {
  profile: string;
  citations: Citation[];
  statistics: Statistic[];
  quotations: Quotation[];
  quality: QualityMetrics;
  entities: Entity[];
  timestamp: string;
}

export interface Citation {
  url: string;
  title?: string;
  domain: string;
  authoritative?: boolean;
}

export interface Statistic {
  value: string;
  context: string;
  type: 'percentage' | 'currency' | 'number' | 'other';
}

export interface Quotation {
  text: string;
  source?: string;
  author?: string;
  credentials?: string;
}

export interface QualityMetrics {
  overall: number;
  readability: number;
  credibility: number;
  freshness?: number;
}

export interface Entity {
  name: string;
  type: string;
  count: number;
  wikidataId?: string;
  relationships?: string[];
}

export interface GEOReport {
  timestamp: string;
  summary: {
    totalPages: number;
    avgCitations: number;
    avgStatistics: number;
    avgQuality: number;
    totalEntities: number;
  };
  pages: PageAnalysis[];
  recommendations: string[];
}

export interface PageAnalysis {
  path: string;
  url: string;
  analysis: GEOAnalysisResult;
  visibilityImprovement: number;
  issues: string[];
  suggestions: string[];
}
