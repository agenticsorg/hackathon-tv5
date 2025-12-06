/**
 * Pattern Learner for Metadata Enrichment
 *
 * Learns from enrichment results and provides intelligent suggestions
 * based on historical patterns and success rates.
 */

import { AgentDBClient } from './agentdb-client';
import {
  MediaMetadata,
  EnrichmentResult
} from '../types';

/**
 * Enrichment suggestion based on learned patterns
 */
export interface EnrichmentSuggestion {
  approach: string;
  model: string;
  confidence: number; // 0-1 confidence score
  expectedQuality: number; // Expected quality based on historical data
  estimatedLatency: number; // Estimated latency in ms
  estimatedTokens: number; // Estimated token usage
  reason: string; // Explanation for the suggestion
  similarPatterns: number; // Number of similar historical patterns
}

/**
 * Learning configuration
 */
export interface LearningConfig {
  minPatterns?: number; // Minimum patterns before making suggestions (default: 5)
  minQuality?: number; // Minimum quality threshold (default: 0.7)
  similarityThreshold?: number; // Minimum similarity for pattern matching (default: 0.6)
  topK?: number; // Number of top suggestions to return (default: 3)
}

/**
 * PatternLearner - Learns from enrichment results and suggests optimal approaches
 *
 * Features:
 * - Learn from successful and failed enrichment attempts
 * - Suggest best enrichment approaches for new content
 * - Track success rates and quality metrics
 * - Adapt recommendations based on content type and context
 */
export class PatternLearner {
  private client: AgentDBClient;
  private config: Required<LearningConfig>;

  constructor(client: AgentDBClient, config: LearningConfig = {}) {
    this.client = client;
    this.config = {
      minPatterns: config.minPatterns || 5,
      minQuality: config.minQuality || 0.7,
      similarityThreshold: config.similarityThreshold || 0.6,
      topK: config.topK || 3
    };
  }

  /**
   * Learn from an enrichment result
   * Stores the pattern for future recommendations
   *
   * @param result - Enrichment result to learn from
   */
  public async learn(result: EnrichmentResult): Promise<void> {
    const success = result.quality >= this.config.minQuality;

    // Generate semantic embedding (placeholder - integrate with actual embedding service)
    const embedding = this.generateEmbedding(result.partialMetadata);

    this.client.storeEnrichmentPattern(
      result.partialMetadata,
      result.enrichedMetadata,
      success,
      {
        assetId: result.assetId,
        approach: result.approach,
        model: result.model,
        quality: result.quality,
        latencyMs: result.latencyMs,
        tokensUsed: result.tokensUsed,
        fieldsEnriched: result.fieldsEnriched,
        embedding
      }
    );
  }

  /**
   * Suggest optimal enrichment approaches for given metadata
   *
   * @param metadata - Media metadata to enrich
   * @returns Array of enrichment suggestions ranked by confidence
   */
  public suggest(metadata: Partial<MediaMetadata>): EnrichmentSuggestion[] {
    // Get similar patterns from historical data
    const similarPatterns = this.client.retrieveSimilarPatterns(
      metadata,
      50, // Get more patterns for better analysis
      this.config.minQuality
    );

    // Filter by similarity threshold
    const relevantPatterns = similarPatterns.filter(
      p => p.similarity >= this.config.similarityThreshold
    );

    // Not enough data to make suggestions
    if (relevantPatterns.length < this.config.minPatterns) {
      return this.getDefaultSuggestions(metadata);
    }

    // Group patterns by approach
    const approachGroups = new Map<string, typeof relevantPatterns>();
    for (const pattern of relevantPatterns) {
      const approach = pattern.pattern.approach;
      if (!approachGroups.has(approach)) {
        approachGroups.set(approach, []);
      }
      approachGroups.get(approach)!.push(pattern);
    }

    // Generate suggestions for each approach
    const suggestions: EnrichmentSuggestion[] = [];

    for (const [approach, patterns] of approachGroups.entries()) {
      const stats = this.analyzePatterns(patterns);

      // Get the most common model used for this approach
      const models = patterns.map(p => p.pattern.model);
      const modelCounts = new Map<string, number>();
      for (const model of models) {
        modelCounts.set(model, (modelCounts.get(model) || 0) + 1);
      }
      const bestModel = Array.from(modelCounts.entries())
        .sort((a, b) => b[1] - a[1])[0][0];

      suggestions.push({
        approach,
        model: bestModel,
        confidence: this.calculateConfidence(patterns, stats),
        expectedQuality: stats.avgQuality,
        estimatedLatency: stats.avgLatency,
        estimatedTokens: stats.avgTokens,
        reason: this.generateReason(approach, patterns.length, stats),
        similarPatterns: patterns.length
      });
    }

    // Sort by confidence and return top K
    return suggestions
      .sort((a, b) => b.confidence - a.confidence)
      .slice(0, this.config.topK);
  }

  /**
   * Get approach-specific statistics
   *
   * @param approach - Approach name
   * @returns Statistics for the approach
   */
  public getApproachStats(approach: string) {
    return this.client.getApproachStats(approach);
  }

  /**
   * Get overall learning statistics
   *
   * @returns Learning statistics
   */
  public getStats() {
    return this.client.getStats();
  }

  /**
   * Analyze patterns to extract statistics
   */
  private analyzePatterns(patterns: Array<{ pattern: any; similarity: number }>) {
    const qualities = patterns.map(p => p.pattern.quality);
    const latencies = patterns.map(p => p.pattern.latencyMs);
    const tokens = patterns.map(p => p.pattern.tokensUsed);
    const similarities = patterns.map(p => p.similarity);

    return {
      avgQuality: this.average(qualities),
      avgLatency: this.average(latencies),
      avgTokens: this.average(tokens),
      avgSimilarity: this.average(similarities),
      count: patterns.length
    };
  }

  /**
   * Calculate confidence score for a suggestion
   * Based on pattern count, average similarity, and quality consistency
   */
  private calculateConfidence(
    patterns: Array<{ pattern: any; similarity: number }>,
    stats: { avgQuality: number; avgSimilarity: number; count: number }
  ): number {
    // More patterns = higher confidence (up to 0.3)
    const countScore = Math.min(stats.count / 20, 1.0) * 0.3;

    // Higher average similarity = higher confidence (up to 0.4)
    const similarityScore = stats.avgSimilarity * 0.4;

    // Higher quality = higher confidence (up to 0.3)
    const qualityScore = stats.avgQuality * 0.3;

    return Math.min(countScore + similarityScore + qualityScore, 1.0);
  }

  /**
   * Generate human-readable reason for suggestion
   */
  private generateReason(approach: string, patternCount: number, stats: any): string {
    const qualityPct = Math.round(stats.avgQuality * 100);
    return `Based on ${patternCount} similar enrichments, this approach achieved ${qualityPct}% average quality with ${Math.round(stats.avgLatency)}ms latency.`;
  }

  /**
   * Get default suggestions when not enough data is available
   */
  private getDefaultSuggestions(metadata: Partial<MediaMetadata>): EnrichmentSuggestion[] {
    return [
      {
        approach: 'default_enrichment',
        model: 'gemini-2.0',
        confidence: 0.5,
        expectedQuality: 0.75,
        estimatedLatency: 1500,
        estimatedTokens: 300,
        reason: 'Default approach - insufficient historical data for personalized recommendation',
        similarPatterns: 0
      },
      {
        approach: 'genre_specialist',
        model: 'claude-3.5',
        confidence: 0.4,
        expectedQuality: 0.8,
        estimatedLatency: 2000,
        estimatedTokens: 400,
        reason: 'Genre-specific enrichment with detailed analysis',
        similarPatterns: 0
      }
    ];
  }

  /**
   * Generate semantic embedding for metadata (placeholder)
   * In production, integrate with actual embedding service (e.g., Vertex AI)
   */
  private generateEmbedding(metadata: Partial<MediaMetadata>): number[] {
    // Placeholder: Generate simple embedding based on metadata features
    // In production, use Vertex AI Embeddings or similar service
    const embedding = new Array(384).fill(0);

    // Hash-based pseudo-embedding (replace with actual embedding service)
    const features = [
      metadata.type || '',
      ...(metadata.genres || []),
      ...(metadata.keywords || [])
    ].join(' ');

    for (let i = 0; i < features.length; i++) {
      const charCode = features.charCodeAt(i);
      const idx = (charCode * (i + 1)) % 384;
      embedding[idx] += charCode / 1000;
    }

    // Normalize
    const norm = Math.sqrt(embedding.reduce((sum, val) => sum + val * val, 0));
    return norm > 0 ? embedding.map(val => val / norm) : embedding;
  }

  /**
   * Calculate average of array
   */
  private average(arr: number[]): number {
    return arr.length > 0 ? arr.reduce((sum, val) => sum + val, 0) / arr.length : 0;
  }
}

export default PatternLearner;
