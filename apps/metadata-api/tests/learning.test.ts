/**
 * AgentDB Learning Integration Tests
 */

import { describe, it, expect, beforeAll, afterAll } from '@jest/globals';
import { AgentDBClient, PatternLearner } from '../src/learning';
import { EnrichmentResult } from '../src/types';
import * as fs from 'fs';
import * as path from 'path';

describe('AgentDB Learning Integration', () => {
  const testDbPath = path.join(__dirname, 'test-learning.db');
  let client: AgentDBClient;
  let learner: PatternLearner;

  beforeAll(() => {
    // Clean up any existing test database
    if (fs.existsSync(testDbPath)) {
      fs.unlinkSync(testDbPath);
    }

    // Initialize client and learner
    client = new AgentDBClient({
      dbPath: testDbPath,
      embeddingDimension: 384,
      vectorSearchEnabled: true
    });

    learner = new PatternLearner(client, {
      minPatterns: 2,
      minQuality: 0.6,
      similarityThreshold: 0.5,
      topK: 3
    });
  });

  afterAll(() => {
    // Clean up
    client.close();
    if (fs.existsSync(testDbPath)) {
      fs.unlinkSync(testDbPath);
    }
  });

  describe('AgentDBClient', () => {
    it('should initialize database schema', () => {
      const stats = client.getStats();
      expect(stats).toBeDefined();
      expect(stats.totalPatterns).toBe(0);
    });

    it('should store enrichment pattern', () => {
      const patternId = client.storeEnrichmentPattern(
        { title: 'Test Movie', type: 'movie', genres: ['action'] },
        { synopsis: 'An action-packed thriller', keywords: ['action', 'thriller'] },
        true,
        {
          assetId: 'test-001',
          approach: 'genre_specialist',
          model: 'gemini-2.0',
          quality: 0.85,
          latencyMs: 1500,
          tokensUsed: 300,
          fieldsEnriched: ['synopsis', 'keywords']
        }
      );

      expect(patternId).toBeGreaterThan(0);
    });

    it('should retrieve similar patterns', () => {
      // Store multiple patterns
      for (let i = 0; i < 5; i++) {
        client.storeEnrichmentPattern(
          { title: `Movie ${i}`, type: 'movie', genres: ['action', 'thriller'] },
          { synopsis: `Synopsis ${i}`, keywords: ['action', 'thriller'] },
          true,
          {
            assetId: `test-00${i}`,
            approach: 'genre_specialist',
            model: 'gemini-2.0',
            quality: 0.8 + (i * 0.02),
            latencyMs: 1400 + (i * 100),
            tokensUsed: 280 + (i * 20),
            fieldsEnriched: ['synopsis', 'keywords']
          }
        );
      }

      const similar = client.retrieveSimilarPatterns(
        { title: 'New Action Movie', type: 'movie', genres: ['action'] },
        5,
        0.7
      );

      expect(similar).toBeDefined();
      expect(similar.length).toBeGreaterThan(0);
      expect(similar[0].similarity).toBeGreaterThanOrEqual(0);
      expect(similar[0].similarity).toBeLessThanOrEqual(1);
    });

    it('should get approach statistics', () => {
      const stats = client.getApproachStats('genre_specialist');

      expect(stats).toBeDefined();
      expect(stats.totalAttempts).toBeGreaterThan(0);
      expect(stats.successRate).toBeGreaterThanOrEqual(0);
      expect(stats.successRate).toBeLessThanOrEqual(1);
      expect(stats.avgQuality).toBeGreaterThan(0);
    });

    it('should get overall statistics', () => {
      const stats = client.getStats();

      expect(stats.totalPatterns).toBeGreaterThan(0);
      expect(stats.successfulPatterns).toBeGreaterThan(0);
      expect(stats.uniqueApproaches).toBeGreaterThanOrEqual(1);
      expect(stats.avgQuality).toBeGreaterThan(0);
    });
  });

  describe('PatternLearner', () => {
    it('should learn from enrichment result', async () => {
      const mockResult: EnrichmentResult = {
        assetId: 'test-learning-001',
        title: 'Learning Test Movie',
        approach: 'mood_analyzer',
        model: 'claude-3.5',
        quality: 0.88,
        latencyMs: 1800,
        tokensUsed: 350,
        partialMetadata: {
          title: 'Learning Test Movie',
          type: 'movie',
          genres: ['sci-fi', 'thriller']
        },
        enrichedMetadata: {
          synopsis: 'A futuristic thriller about AI',
          keywords: ['ai', 'future', 'technology'],
          moodTags: ['intense', 'thought-provoking']
        },
        fieldsEnriched: ['synopsis', 'keywords', 'moodTags'],
        timestamp: new Date()
      };

      await learner.learn(mockResult);

      const stats = learner.getStats();
      expect(stats.totalPatterns).toBeGreaterThan(0);
    });

    it('should provide enrichment suggestions', () => {
      const metadata = {
        title: 'Future Wars',
        type: 'movie' as const,
        genres: ['sci-fi', 'action'],
        keywords: ['future', 'war']
      };

      const suggestions = learner.suggest(metadata);

      expect(suggestions).toBeDefined();
      expect(Array.isArray(suggestions)).toBe(true);
      expect(suggestions.length).toBeGreaterThan(0);

      const firstSuggestion = suggestions[0];
      expect(firstSuggestion.approach).toBeDefined();
      expect(firstSuggestion.model).toBeDefined();
      expect(firstSuggestion.confidence).toBeGreaterThanOrEqual(0);
      expect(firstSuggestion.confidence).toBeLessThanOrEqual(1);
      expect(firstSuggestion.expectedQuality).toBeGreaterThanOrEqual(0);
      expect(firstSuggestion.reason).toBeDefined();
    });

    it('should provide default suggestions with insufficient data', () => {
      // Clear patterns
      client.clearPatterns();

      const metadata = {
        title: 'Unknown Genre',
        type: 'documentary' as const,
        genres: ['experimental'],
        keywords: []
      };

      const suggestions = learner.suggest(metadata);

      expect(suggestions).toBeDefined();
      expect(suggestions.length).toBeGreaterThan(0);
      expect(suggestions[0].approach).toBe('default_enrichment');
      expect(suggestions[0].similarPatterns).toBe(0);
    });

    it('should track approach statistics', () => {
      const stats = learner.getApproachStats('mood_analyzer');

      expect(stats).toBeDefined();
      if (stats.totalAttempts > 0) {
        expect(stats.successRate).toBeGreaterThanOrEqual(0);
        expect(stats.avgQuality).toBeGreaterThan(0);
      }
    });

    it('should get overall learning statistics', () => {
      const stats = learner.getStats();

      expect(stats).toBeDefined();
      expect(stats.totalPatterns).toBeGreaterThanOrEqual(0);
      expect(stats.uniqueApproaches).toBeGreaterThanOrEqual(0);
    });
  });

  describe('Integration Workflow', () => {
    it('should complete learn-suggest-learn cycle', async () => {
      // Clear for clean test
      client.clearPatterns();

      // Learn from several enrichments
      const mockResults: EnrichmentResult[] = [
        {
          assetId: 'workflow-001',
          title: 'Action Movie 1',
          approach: 'genre_specialist',
          model: 'gemini-2.0',
          quality: 0.90,
          latencyMs: 1500,
          tokensUsed: 300,
          partialMetadata: { type: 'movie', genres: ['action'] },
          enrichedMetadata: { synopsis: 'Action packed', keywords: ['action'] },
          fieldsEnriched: ['synopsis', 'keywords'],
          timestamp: new Date()
        },
        {
          assetId: 'workflow-002',
          title: 'Action Movie 2',
          approach: 'genre_specialist',
          model: 'gemini-2.0',
          quality: 0.88,
          latencyMs: 1600,
          tokensUsed: 320,
          partialMetadata: { type: 'movie', genres: ['action', 'thriller'] },
          enrichedMetadata: { synopsis: 'Thrilling action', keywords: ['action', 'thriller'] },
          fieldsEnriched: ['synopsis', 'keywords'],
          timestamp: new Date()
        },
        {
          assetId: 'workflow-003',
          title: 'Action Movie 3',
          approach: 'genre_specialist',
          model: 'gemini-2.0',
          quality: 0.92,
          latencyMs: 1450,
          tokensUsed: 290,
          partialMetadata: { type: 'movie', genres: ['action'] },
          enrichedMetadata: { synopsis: 'Epic action', keywords: ['action', 'epic'] },
          fieldsEnriched: ['synopsis', 'keywords'],
          timestamp: new Date()
        }
      ];

      // Learn from all results
      for (const result of mockResults) {
        await learner.learn(result);
      }

      // Get suggestions for similar content
      const suggestions = learner.suggest({
        type: 'movie',
        genres: ['action'],
        keywords: []
      });

      // Should have meaningful suggestions now
      expect(suggestions.length).toBeGreaterThan(0);
      const topSuggestion = suggestions[0];
      expect(topSuggestion.approach).toBe('genre_specialist');
      expect(topSuggestion.confidence).toBeGreaterThan(0.5);
      expect(topSuggestion.similarPatterns).toBeGreaterThanOrEqual(3);

      // Verify statistics
      const approachStats = learner.getApproachStats('genre_specialist');
      expect(approachStats.totalAttempts).toBe(3);
      expect(approachStats.successRate).toBe(1.0); // All successful
      expect(approachStats.avgQuality).toBeGreaterThan(0.85);
    });
  });
});
