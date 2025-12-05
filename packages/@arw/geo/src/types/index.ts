/**
 * Core types for ARW GEO enhancements
 * Based on ARW GEO Technical Design v1.0
 */

// === Citation Types (ARW-GEO-1) ===

export type CitationType =
  | 'academic'
  | 'industry_report'
  | 'government'
  | 'news'
  | 'book'
  | 'website'
  | 'dataset'
  | 'internal';

export interface Author {
  name: string;
  affiliation?: string;
  credentials?: string[];
}

export interface Citation {
  id: string;
  source: string;
  type: CitationType;
  url: string;
  date: string; // ISO 8601
  author?: string | Author;
  publisher?: string;
  authority_score?: number; // 0-1
  verification_status?: 'verified' | 'unverified' | 'external';
  doi?: string;
  isbn?: string;
  accessed_date?: string;
  excerpt?: string;
}

// === Statistics Types (ARW-GEO-2) ===

export type StatisticType =
  | 'market_metric'
  | 'performance_metric'
  | 'demographic'
  | 'financial'
  | 'comparative_metrics'
  | 'time_series'
  | 'forecast';

export interface GrowthRate {
  value: number;
  unit: 'percent' | 'absolute';
  period: 'YoY' | 'MoM' | 'QoQ';
  date_range: [string, string];
}

export interface Comparison {
  baseline: {
    value: number;
    label: string;
  };
  comparisons: Array<{
    value: number;
    label: string;
    difference: number;
    difference_percent: number;
  }>;
}

export interface Statistic {
  id: string;
  type: StatisticType;
  value: number | string;
  unit: string;
  date: string;
  date_range?: [string, string];
  source: string;
  source_url?: string;
  confidence?: number; // 0-1
  methodology?: string;
  sample_size?: number;
  margin_of_error?: number;
  growth_rate?: GrowthRate;
  comparison?: Comparison;
  visualization_url?: string;
}

// === Quotation Types (ARW-GEO-3) ===

export type QuotationType =
  | 'expert_opinion'
  | 'case_study'
  | 'research_finding'
  | 'executive_insight'
  | 'technical_detail'
  | 'statistical_claim';

export interface Speaker {
  name: string;
  title: string;
  affiliation: string;
  credentials?: string[];
  authority_score?: number; // 0-1
  linkedin?: string;
  twitter?: string;
  company_size?: string;
  industry?: string;
  verified?: boolean;
}

export interface Quotation {
  id: string;
  text: string;
  speaker: Speaker;
  context?: string;
  date: string;
  source: string;
  source_url?: string;
  type: QuotationType;
  verified?: boolean;
  metrics_mentioned?: string[];
}

// === Quality Signals Types (ARW-GEO-4) ===

export interface ReadabilityMetrics {
  flesch_kincaid_grade: number;
  gunning_fog: number;
  smog_index: number;
  reading_ease: number;
  average_sentence_length: number;
  complex_words_percent: number;
}

export interface EEATSignals {
  experience_signals: string[];
  expertise_signals: string[];
  authoritativeness_signals: string[];
  trustworthiness_signals: string[];
  overall_score?: number; // 0-1
}

export interface Publication {
  title: string;
  type: 'book' | 'paper' | 'article' | 'report';
  date: string;
  url?: string;
  citations?: number;
}

export interface AuthorCredentials {
  name: string;
  title: string;
  credentials: string[];
  affiliation: string;
  bio_url?: string;
  verified: boolean;
  publications?: Publication[];
  expertise_areas?: string[];
}

export type ContentType =
  | 'research'
  | 'tutorial'
  | 'analysis'
  | 'news'
  | 'opinion'
  | 'case_study'
  | 'documentation';

export type TechnicalLevel =
  | 'beginner'
  | 'intermediate'
  | 'advanced'
  | 'academic';

export interface QualityMetadata {
  fluency_score: number; // 0-1
  readability: ReadabilityMetrics;
  eeat: EEATSignals;
  author: AuthorCredentials;
  last_reviewed?: string;
  fact_checked?: boolean;
  peer_reviewed?: boolean;
  content_type?: ContentType;
  technical_level?: TechnicalLevel;
}

// === Entity Types (ARW-GEO-5) ===

export type EntityType =
  | 'Person'
  | 'Organization'
  | 'Product'
  | 'Place'
  | 'Concept'
  | 'Event'
  | 'CreativeWork';

export interface Entity {
  id: string;
  type: EntityType;
  name: string;
  wikidata_id?: string;
  dbpedia_url?: string;
  schema_type: string;
  properties: Record<string, any>;
  related_entities?: string[];
  aliases?: string[];
  description?: string;
  confidence?: number; // 0-1
}

export interface EntityRelationship {
  source_id: string;
  target_id: string;
  relationship_type: string;
  strength?: number; // 0-1
}

export interface EntityGraph {
  entities: Entity[];
  relationships: EntityRelationship[];
}

// === Clustering Types (ARW-GEO-6) ===

export interface Topic {
  id: string;
  name: string;
  parent?: string;
  children?: string[];
  page_count: number;
  authority_score?: number; // 0-1
  description?: string;
  synonyms?: string[];
}

export interface ContentCluster {
  id: string;
  name: string;
  topic_id: string;
  pages: string[];
  pillar_page?: string;
  cohesion_score: number; // 0-1
  created_date: string;
  last_updated: string;
}

export type RelationshipType =
  | 'explains_concept'
  | 'prerequisite'
  | 'next_step'
  | 'comparison'
  | 'case_study'
  | 'alternative'
  | 'deep_dive';

export type TopicalDepth =
  | 'overview'
  | 'introductory'
  | 'intermediate'
  | 'advanced'
  | 'comprehensive';

export interface RelatedPage {
  url: string;
  relationship: RelationshipType;
  relevance: number; // 0-1
  title?: string;
}

export interface SemanticMetadata {
  primary_topic: string;
  secondary_topics?: string[];
  topic_hierarchy: string[];
  content_cluster?: string;
  related_pages: RelatedPage[];
  semantic_keywords: {
    primary: string[];
    secondary: string[];
    entities: string[];
  };
  topical_depth: TopicalDepth;
  content_type: string;
}

// === Domain Types (ARW-GEO-7) ===

export type DomainType =
  | 'ecommerce'
  | 'saas'
  | 'media_publishing'
  | 'documentation'
  | 'education'
  | 'finance'
  | 'healthcare'
  | 'real_estate'
  | 'travel'
  | 'food_beverage'
  | 'local_business'
  | 'marketplace';

export interface DomainClassification {
  primary: DomainType;
  secondary?: DomainType[];
  confidence: number; // 0-1
  industries?: string[];
  business_model?: string;
}

export interface DomainOptimizationProfile {
  domain: DomainType;
  recommended_schemas: string[];
  required_metadata: string[];
  optional_metadata: string[];
  geo_priorities: string[];
  examples: string[];
}

export interface DomainMetadata {
  domain_classification: DomainClassification;
  domain_specific_patterns: Record<string, any>;
  optimization_recommendations: string[];
}

// === GEO Metadata ===

export interface GEOEnhancements {
  citations_enabled?: boolean;
  statistics_enabled?: boolean;
  quotations_enabled?: boolean;
  quality_signals?: boolean;
  entity_enrichment?: boolean;
  semantic_clustering?: boolean;
  domain_optimization?: boolean;
}

export interface GEOMetadata {
  version: string;
  profile: 'ARW-2.1' | 'ARW-2.2';
  geo_enhancements: GEOEnhancements;
  content_pages: Array<{
    url: string;
    machine_view: string;
    geo_metadata?: {
      citations_count?: number;
      statistics_count?: number;
      quotations_count?: number;
      quality_score?: number;
      entities_count?: number;
      authority_score?: number;
    };
  }>;
}

// === Config Types ===

export interface LLMConfig {
  provider: 'anthropic' | 'openai';
  apiKey: string;
  model: string;
  temperature?: number;
  maxTokens?: number;
}

export interface GEOConfig {
  profile: 'ARW-2.1' | 'ARW-2.2';
  domain?: DomainType;
  llm?: LLMConfig;
  enabledFeatures?: {
    citations?: boolean;
    statistics?: boolean;
    quotations?: boolean;
    quality?: boolean;
    entities?: boolean;
    clustering?: boolean;
  };
}
