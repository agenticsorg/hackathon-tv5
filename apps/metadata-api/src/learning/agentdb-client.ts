/**
 * AgentDB Client for Pattern Learning
 *
 * Integrates with ReasoningBank memory database for storing and retrieving
 * enrichment patterns using vector embeddings for semantic similarity search.
 */

import Database from 'better-sqlite3';
import path from 'path';

/**
 * Pattern record stored in the database
 */
export interface EnrichmentPattern {
  id: number;
  assetId: string;
  inputMetadata: string; // JSON serialized Partial<MediaMetadata>
  outputMetadata: string; // JSON serialized Partial<MediaMetadata>
  approach: string;
  model: string;
  quality: number;
  latencyMs: number;
  tokensUsed: number;
  fieldsEnriched: string; // JSON serialized string[]
  success: boolean;
  embedding?: string; // JSON serialized number[] (semantic representation)
  createdAt: string;
}

/**
 * Pattern retrieval result with similarity score
 */
export interface PatternMatch {
  pattern: EnrichmentPattern;
  similarity: number;
}

/**
 * AgentDB Client Configuration
 */
export interface AgentDBClientConfig {
  dbPath?: string; // Path to SQLite database (defaults to mondweep/.swarm/memory.db)
  embeddingDimension?: number; // Dimension of embedding vectors (default: 384)
  vectorSearchEnabled?: boolean; // Enable vector similarity search (default: true)
}

/**
 * AgentDBClient - Pattern storage and retrieval for metadata enrichment
 *
 * Features:
 * - SQLite-based persistent storage
 * - Vector embeddings for semantic search
 * - Pattern matching for similar content
 * - Success rate tracking
 * - Integration with ReasoningBank memory
 */
export class AgentDBClient {
  private db: Database.Database;
  private embeddingDim: number;
  private vectorSearchEnabled: boolean;

  constructor(config: AgentDBClientConfig = {}) {
    const {
      dbPath = path.join(process.cwd(), 'mondweep', '.swarm', 'memory.db'),
      embeddingDimension = 384,
      vectorSearchEnabled = true
    } = config;

    this.db = new Database(dbPath);
    this.embeddingDim = embeddingDimension;
    this.vectorSearchEnabled = vectorSearchEnabled;

    this.initialize();
  }

  /**
   * Initialize database schema for enrichment patterns
   */
  private initialize(): void {
    // Create enrichment_patterns table if it doesn't exist
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS enrichment_patterns (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        asset_id TEXT NOT NULL,
        input_metadata TEXT NOT NULL,
        output_metadata TEXT NOT NULL,
        approach TEXT NOT NULL,
        model TEXT NOT NULL,
        quality REAL NOT NULL,
        latency_ms INTEGER NOT NULL,
        tokens_used INTEGER NOT NULL,
        fields_enriched TEXT NOT NULL,
        success INTEGER NOT NULL,
        embedding TEXT,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
      );

      CREATE INDEX IF NOT EXISTS idx_enrichment_patterns_asset
        ON enrichment_patterns(asset_id);

      CREATE INDEX IF NOT EXISTS idx_enrichment_patterns_approach
        ON enrichment_patterns(approach);

      CREATE INDEX IF NOT EXISTS idx_enrichment_patterns_quality
        ON enrichment_patterns(quality DESC);

      CREATE INDEX IF NOT EXISTS idx_enrichment_patterns_success
        ON enrichment_patterns(success);

      CREATE INDEX IF NOT EXISTS idx_enrichment_patterns_created
        ON enrichment_patterns(created_at DESC);
    `);
  }

  /**
   * Store enrichment pattern for learning
   *
   * @param input - Original partial metadata
   * @param output - Enriched metadata fields
   * @param success - Whether enrichment was successful
   * @param metadata - Additional enrichment metadata
   */
  public storeEnrichmentPattern(
    input: any,
    output: any,
    success: boolean,
    metadata: {
      assetId: string;
      approach: string;
      model: string;
      quality: number;
      latencyMs: number;
      tokensUsed: number;
      fieldsEnriched: string[];
      embedding?: number[];
    }
  ): number {
    const stmt = this.db.prepare(`
      INSERT INTO enrichment_patterns (
        asset_id, input_metadata, output_metadata, approach, model,
        quality, latency_ms, tokens_used, fields_enriched, success, embedding
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `);

    const result = stmt.run(
      metadata.assetId,
      JSON.stringify(input),
      JSON.stringify(output),
      metadata.approach,
      metadata.model,
      metadata.quality,
      metadata.latencyMs,
      metadata.tokensUsed,
      JSON.stringify(metadata.fieldsEnriched),
      success ? 1 : 0,
      metadata.embedding ? JSON.stringify(metadata.embedding) : null
    );

    return result.lastInsertRowid as number;
  }

  /**
   * Retrieve similar patterns using vector similarity or metadata matching
   *
   * @param input - Input metadata to match against
   * @param limit - Maximum number of patterns to return
   * @param minQuality - Minimum quality threshold (0-1)
   * @returns Array of pattern matches with similarity scores
   */
  public retrieveSimilarPatterns(
    input: any,
    limit: number = 10,
    minQuality: number = 0.7
  ): PatternMatch[] {
    // For now, use metadata-based matching (genre, type, keywords)
    // Vector similarity can be added when embeddings are integrated
    const patterns = this.db.prepare(`
      SELECT * FROM enrichment_patterns
      WHERE success = 1 AND quality >= ?
      ORDER BY quality DESC, created_at DESC
      LIMIT ?
    `).all(minQuality, limit) as any[];

    return patterns.map(p => ({
      pattern: this.deserializePattern(p),
      similarity: this.calculateSimilarity(input, JSON.parse(p.input_metadata))
    })).sort((a, b) => b.similarity - a.similarity);
  }

  /**
   * Get patterns by approach for analysis
   *
   * @param approach - Enrichment approach name
   * @param limit - Maximum number of patterns
   * @returns Array of enrichment patterns
   */
  public getPatternsByApproach(approach: string, limit: number = 50): EnrichmentPattern[] {
    const patterns = this.db.prepare(`
      SELECT * FROM enrichment_patterns
      WHERE approach = ?
      ORDER BY created_at DESC
      LIMIT ?
    `).all(approach, limit) as any[];

    return patterns.map(p => this.deserializePattern(p));
  }

  /**
   * Get success rate statistics for an approach
   *
   * @param approach - Enrichment approach name
   * @returns Success rate and statistics
   */
  public getApproachStats(approach: string): {
    totalAttempts: number;
    successCount: number;
    successRate: number;
    avgQuality: number;
    avgLatency: number;
    avgTokens: number;
  } {
    const stats = this.db.prepare(`
      SELECT
        COUNT(*) as total_attempts,
        SUM(success) as success_count,
        AVG(quality) as avg_quality,
        AVG(latency_ms) as avg_latency,
        AVG(tokens_used) as avg_tokens
      FROM enrichment_patterns
      WHERE approach = ?
    `).get(approach) as any;

    const totalAttempts = stats.total_attempts || 0;
    const successCount = stats.success_count || 0;

    return {
      totalAttempts,
      successCount,
      successRate: totalAttempts > 0 ? successCount / totalAttempts : 0,
      avgQuality: stats.avg_quality || 0,
      avgLatency: stats.avg_latency || 0,
      avgTokens: stats.avg_tokens || 0
    };
  }

  /**
   * Get overall learning statistics
   *
   * @returns Database statistics
   */
  public getStats(): {
    totalPatterns: number;
    successfulPatterns: number;
    uniqueApproaches: number;
    avgQuality: number;
  } {
    const stats = this.db.prepare(`
      SELECT
        COUNT(*) as total_patterns,
        SUM(success) as successful_patterns,
        COUNT(DISTINCT approach) as unique_approaches,
        AVG(quality) as avg_quality
      FROM enrichment_patterns
    `).get() as any;

    return {
      totalPatterns: stats.total_patterns || 0,
      successfulPatterns: stats.successful_patterns || 0,
      uniqueApproaches: stats.unique_approaches || 0,
      avgQuality: stats.avg_quality || 0
    };
  }

  /**
   * Clear all patterns (use with caution)
   */
  public clearPatterns(): void {
    this.db.exec('DELETE FROM enrichment_patterns');
  }

  /**
   * Close database connection
   */
  public close(): void {
    this.db.close();
  }

  /**
   * Deserialize database row to EnrichmentPattern
   */
  private deserializePattern(row: any): EnrichmentPattern {
    return {
      id: row.id,
      assetId: row.asset_id,
      inputMetadata: row.input_metadata,
      outputMetadata: row.output_metadata,
      approach: row.approach,
      model: row.model,
      quality: row.quality,
      latencyMs: row.latency_ms,
      tokensUsed: row.tokens_used,
      fieldsEnriched: row.fields_enriched,
      success: row.success === 1,
      embedding: row.embedding,
      createdAt: row.created_at
    };
  }

  /**
   * Calculate similarity between two metadata objects
   * Uses simple field overlap for now (can be enhanced with embeddings)
   *
   * @param a - First metadata object
   * @param b - Second metadata object
   * @returns Similarity score (0-1)
   */
  private calculateSimilarity(a: any, b: any): number {
    let score = 0;
    let totalFields = 0;

    // Type match
    if (a.type && b.type) {
      totalFields++;
      if (a.type === b.type) score += 0.3;
    }

    // Genre overlap
    if (a.genres && b.genres) {
      totalFields++;
      const aGenres = new Set(a.genres);
      const bGenres = new Set(b.genres);
      const intersection = new Set([...aGenres].filter(x => bGenres.has(x)));
      const union = new Set([...aGenres, ...bGenres]);
      if (union.size > 0) {
        score += 0.4 * (intersection.size / union.size);
      }
    }

    // Keyword overlap
    if (a.keywords && b.keywords) {
      totalFields++;
      const aKeywords = new Set(a.keywords);
      const bKeywords = new Set(b.keywords);
      const intersection = new Set([...aKeywords].filter(x => bKeywords.has(x)));
      const union = new Set([...aKeywords, ...bKeywords]);
      if (union.size > 0) {
        score += 0.3 * (intersection.size / union.size);
      }
    }

    return totalFields > 0 ? score : 0;
  }

  /**
   * Calculate cosine similarity between two embedding vectors
   *
   * @param a - First embedding vector
   * @param b - Second embedding vector
   * @returns Cosine similarity score (0-1)
   */
  private cosineSimilarity(a: number[], b: number[]): number {
    if (a.length !== b.length) {
      throw new Error('Embedding dimensions must match');
    }

    let dotProduct = 0;
    let normA = 0;
    let normB = 0;

    for (let i = 0; i < a.length; i++) {
      dotProduct += a[i] * b[i];
      normA += a[i] * a[i];
      normB += b[i] * b[i];
    }

    const denominator = Math.sqrt(normA) * Math.sqrt(normB);
    return denominator === 0 ? 0 : dotProduct / denominator;
  }
}

export default AgentDBClient;
