/**
 * @arw/geo - Main entry point
 * Generative Engine Optimization for Agent-Ready Web
 */

import type { GEOConfig, GEOMetadata } from './types';
import { CitationFramework } from './citations';
import { StatisticsEnhancement } from './statistics';
import { QuotationSystem } from './quotations';
import { ContentQualitySignals } from './quality';
import { EntityEnrichment } from './entities';
import { SemanticClustering } from './clustering';
import { DomainOptimization } from './domain';
import { LLMIntegration } from './llm';

export class GEOOptimizer {
  public readonly citations: CitationFramework;
  public readonly statistics: StatisticsEnhancement;
  public readonly quotations: QuotationSystem;
  public readonly quality: ContentQualitySignals;
  public readonly entities: EntityEnrichment;
  public readonly clustering: SemanticClustering;
  public readonly domain: DomainOptimization;
  public readonly llm?: LLMIntegration;

  private config: GEOConfig;

  constructor(config: GEOConfig) {
    this.config = config;

    // Initialize all modules
    this.citations = new CitationFramework();
    this.statistics = new StatisticsEnhancement();
    this.quotations = new QuotationSystem();
    this.quality = new ContentQualitySignals();
    this.entities = new EntityEnrichment();
    this.clustering = new SemanticClustering();
    this.domain = new DomainOptimization();

    // Initialize LLM integration if configured
    if (config.llm) {
      this.llm = new LLMIntegration(config.llm);
    }
  }

  /**
   * Analyze content and generate comprehensive GEO metadata
   */
  async analyze(content: string, options?: {
    extractCitations?: boolean;
    extractStatistics?: boolean;
    extractQuotations?: boolean;
    calculateQuality?: boolean;
    extractEntities?: boolean;
    useLLM?: boolean;
  }): Promise<{
    citations?: any[];
    statistics?: any[];
    quotations?: any[];
    quality?: any;
    entities?: any[];
    profile: string;
  }> {
    const result: any = {
      profile: this.config.profile
    };

    // Extract citations
    if (options?.extractCitations !== false) {
      result.citations = this.llm && options?.useLLM
        ? await this.llm.enhanceCitations(content)
        : this.citations.extractFromMarkdown(content);
    }

    // Extract statistics
    if (options?.extractStatistics !== false) {
      result.statistics = this.statistics.extractFromMarkdown(content);
    }

    // Extract quotations
    if (options?.extractQuotations !== false) {
      result.quotations = this.llm && options?.useLLM
        ? await this.llm.extractQuotations(content)
        : this.quotations.extractFromMarkdown(content);
    }

    // Calculate quality
    if (options?.calculateQuality !== false) {
      result.quality = this.llm && options?.useLLM
        ? await this.llm.analyzeQuality(content)
        : this.quality.analyzeQuality(content);
    }

    // Extract entities
    if (options?.extractEntities !== false) {
      result.entities = this.llm && options?.useLLM
        ? await this.llm.extractEntities(content)
        : this.entities.extractFromMarkdown(content);
    }

    return result;
  }

  /**
   * Generate GEO metadata manifest
   */
  generateManifest(options: {
    version?: string;
    contentPages: Array<{
      url: string;
      machineView: string;
      citationsCount?: number;
      statisticsCount?: number;
      quotationsCount?: number;
    }>;
  }): GEOMetadata {
    return {
      version: options.version || '0.2',
      profile: this.config.profile,
      geo_enhancements: {
        citations_enabled: this.config.enabledFeatures?.citations !== false,
        statistics_enabled: this.config.enabledFeatures?.statistics !== false,
        quotations_enabled: this.config.enabledFeatures?.quotations !== false,
        quality_signals: this.config.profile === 'ARW-2.2',
        entity_enrichment: this.config.profile === 'ARW-2.2',
        semantic_clustering: this.config.profile === 'ARW-2.2'
      },
      content_pages: options.contentPages.map(page => ({
        url: page.url,
        machine_view: page.machineView,
        geo_metadata: {
          citations_count: page.citationsCount,
          statistics_count: page.statisticsCount,
          quotations_count: page.quotationsCount
        }
      }))
    };
  }

  /**
   * Get optimization recommendations for domain
   */
  getRecommendations(domain?: string): {
    priorities: string[];
    schemas: string[];
    estimatedImpact: string;
  } {
    const domainType = domain || this.config.domain || 'saas';
    const profile = this.domain.getProfile(domainType as any);

    return {
      priorities: profile?.geo_priorities || [],
      schemas: profile?.recommended_schemas || [],
      estimatedImpact: this.config.profile === 'ARW-2.2'
        ? '+230-290% visibility'
        : '+140-170% visibility'
    };
  }
}

// Re-export types
export * from './types';
export { CitationFramework } from './citations';
export { StatisticsEnhancement } from './statistics';
export { QuotationSystem } from './quotations';
export { ContentQualitySignals } from './quality';
export { EntityEnrichment } from './entities';
export { SemanticClustering } from './clustering';
export { DomainOptimization } from './domain';
export { LLMIntegration } from './llm';
