/**
 * Example Integration: AgentDB Learning with Metadata Enrichment
 *
 * Demonstrates how to integrate pattern learning into the metadata enrichment workflow
 */

import { AgentDBClient, PatternLearner } from './index';
import { MetadataService } from '../services/MetadataService';
import { EnrichmentRequest, EnrichmentResult } from '../types';

/**
 * Example: Enrichment with Learning Integration
 */
export async function enrichWithLearning() {
  // Initialize AgentDB client and pattern learner
  const agentDB = new AgentDBClient({
    // Uses default path: mondweep/.swarm/memory.db
    embeddingDimension: 384,
    vectorSearchEnabled: true
  });

  const learner = new PatternLearner(agentDB, {
    minPatterns: 5,
    minQuality: 0.7,
    similarityThreshold: 0.6,
    topK: 3
  });

  // Initialize metadata service
  const metadataService = new MetadataService();

  // Example: Get suggestions before enrichment
  const metadata = await metadataService.getById('asset-001');
  if (!metadata) {
    throw new Error('Asset not found');
  }

  console.log('\n🔍 Getting enrichment suggestions...');
  const suggestions = learner.suggest(metadata);

  console.log(`\n📊 Found ${suggestions.length} suggestions:`);
  suggestions.forEach((suggestion, idx) => {
    console.log(`\n${idx + 1}. Approach: ${suggestion.approach}`);
    console.log(`   Model: ${suggestion.model}`);
    console.log(`   Confidence: ${(suggestion.confidence * 100).toFixed(1)}%`);
    console.log(`   Expected Quality: ${(suggestion.expectedQuality * 100).toFixed(1)}%`);
    console.log(`   Estimated Latency: ${suggestion.estimatedLatency}ms`);
    console.log(`   Reason: ${suggestion.reason}`);
  });

  // Use top suggestion or fallback to default
  const bestSuggestion = suggestions[0];
  const enrichmentRequest: EnrichmentRequest = {
    assetId: metadata.id,
    approach: bestSuggestion?.approach || 'default_enrichment',
    model: bestSuggestion?.model as any || 'gemini-2.0'
  };

  console.log('\n⚡ Performing enrichment...');
  const result = await metadataService.enrich(enrichmentRequest);

  console.log('\n✅ Enrichment completed:');
  console.log(`   Quality: ${(result.quality * 100).toFixed(1)}%`);
  console.log(`   Latency: ${result.latencyMs}ms`);
  console.log(`   Tokens: ${result.tokensUsed}`);
  console.log(`   Fields enriched: ${result.fieldsEnriched.join(', ')}`);

  // Learn from the enrichment result
  console.log('\n🧠 Learning from enrichment result...');
  await learner.learn(result);

  // Get updated statistics
  const stats = learner.getStats();
  console.log('\n📈 Learning Statistics:');
  console.log(`   Total Patterns: ${stats.totalPatterns}`);
  console.log(`   Successful Patterns: ${stats.successfulPatterns}`);
  console.log(`   Unique Approaches: ${stats.uniqueApproaches}`);
  console.log(`   Average Quality: ${(stats.avgQuality * 100).toFixed(1)}%`);

  // Get approach-specific stats
  if (bestSuggestion) {
    const approachStats = learner.getApproachStats(bestSuggestion.approach);
    console.log(`\n📊 Stats for "${bestSuggestion.approach}":`);
    console.log(`   Total Attempts: ${approachStats.totalAttempts}`);
    console.log(`   Success Rate: ${(approachStats.successRate * 100).toFixed(1)}%`);
    console.log(`   Avg Quality: ${(approachStats.avgQuality * 100).toFixed(1)}%`);
    console.log(`   Avg Latency: ${approachStats.avgLatency.toFixed(0)}ms`);
    console.log(`   Avg Tokens: ${approachStats.avgTokens.toFixed(0)}`);
  }

  // Clean up
  agentDB.close();

  return result;
}

/**
 * Example: Batch Learning from Multiple Enrichments
 */
export async function batchLearningExample() {
  const agentDB = new AgentDBClient();
  const learner = new PatternLearner(agentDB);
  const metadataService = new MetadataService();

  console.log('\n🚀 Batch Learning Example\n');

  // Simulate multiple enrichments
  const assetIds = ['asset-001', 'asset-002'];
  const approaches = ['genre_specialist', 'mood_analyzer', 'default_enrichment'];

  for (const assetId of assetIds) {
    const metadata = await metadataService.getById(assetId);
    if (!metadata) continue;

    console.log(`\n📄 Processing ${metadata.title}...`);

    // Try different approaches and learn from each
    for (const approach of approaches) {
      const request: EnrichmentRequest = {
        assetId,
        approach,
        model: 'gemini-2.0'
      };

      const result = await metadataService.enrich(request);
      await learner.learn(result);

      console.log(`   ✓ Learned from ${approach}: quality=${(result.quality * 100).toFixed(0)}%`);
    }
  }

  // After learning, get optimized suggestions
  const testMetadata = await metadataService.getById('asset-001');
  if (testMetadata) {
    console.log('\n🎯 Optimized suggestions after learning:');
    const suggestions = learner.suggest(testMetadata);
    suggestions.forEach((s, i) => {
      console.log(`   ${i + 1}. ${s.approach} (confidence: ${(s.confidence * 100).toFixed(0)}%)`);
    });
  }

  const finalStats = learner.getStats();
  console.log('\n📊 Final Learning Stats:');
  console.log(`   Patterns stored: ${finalStats.totalPatterns}`);
  console.log(`   Success rate: ${(finalStats.successfulPatterns / finalStats.totalPatterns * 100).toFixed(1)}%`);

  agentDB.close();
}

/**
 * Example: Integration with MetadataService
 */
export class EnrichmentWithLearning {
  private agentDB: AgentDBClient;
  private learner: PatternLearner;
  private metadataService: MetadataService;

  constructor() {
    this.agentDB = new AgentDBClient();
    this.learner = new PatternLearner(this.agentDB);
    this.metadataService = new MetadataService();
  }

  /**
   * Enrich with intelligent approach selection
   */
  async enrichSmart(assetId: string, fields?: string[]): Promise<EnrichmentResult> {
    // Get metadata
    const metadata = await this.metadataService.getById(assetId);
    if (!metadata) {
      throw new Error(`Asset not found: ${assetId}`);
    }

    // Get AI-powered suggestions
    const suggestions = this.learner.suggest(metadata);
    const bestApproach = suggestions[0] || {
      approach: 'default_enrichment',
      model: 'gemini-2.0'
    };

    // Perform enrichment with suggested approach
    const result = await this.metadataService.enrich({
      assetId,
      fields,
      approach: bestApproach.approach,
      model: bestApproach.model as any
    });

    // Learn from result
    await this.learner.learn(result);

    return result;
  }

  /**
   * Get learning insights
   */
  getInsights() {
    return {
      overall: this.learner.getStats(),
      byApproach: (approach: string) => this.learner.getApproachStats(approach)
    };
  }

  /**
   * Cleanup
   */
  close() {
    this.agentDB.close();
  }
}

// If running directly
if (require.main === module) {
  enrichWithLearning()
    .then(() => console.log('\n✅ Example completed successfully'))
    .catch(err => console.error('\n❌ Error:', err));
}
