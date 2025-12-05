/**
 * Tests for main GEOOptimizer class
 */

import { GEOOptimizer } from '../src';

describe('GEOOptimizer', () => {
  describe('constructor', () => {
    it('should initialize with ARW-2.1 profile', () => {
      const optimizer = new GEOOptimizer({
        profile: 'ARW-2.1',
        domain: 'ecommerce'
      });

      expect(optimizer.citations).toBeDefined();
      expect(optimizer.statistics).toBeDefined();
      expect(optimizer.quotations).toBeDefined();
      expect(optimizer.quality).toBeDefined();
    });

    it('should initialize with ARW-2.2 profile', () => {
      const optimizer = new GEOOptimizer({
        profile: 'ARW-2.2',
        domain: 'saas'
      });

      expect(optimizer.entities).toBeDefined();
      expect(optimizer.clustering).toBeDefined();
      expect(optimizer.domain).toBeDefined();
    });

    it('should not initialize LLM without config', () => {
      const optimizer = new GEOOptimizer({
        profile: 'ARW-2.1'
      });

      expect(optimizer.llm).toBeUndefined();
    });
  });

  describe('analyze', () => {
    it('should analyze content and extract features', async () => {
      const optimizer = new GEOOptimizer({
        profile: 'ARW-2.1'
      });

      const content = `
Test content with citation^[cite:1]

\`\`\`statistics
{"id": "stat:1", "type": "market_metric", "value": 100, "unit": "USD", "date": "2024-01-01", "source": "Test"}
\`\`\`

[cite:1]: {"source": "Test", "type": "academic", "url": "https://example.com", "date": "2024-01-01"}
      `;

      const result = await optimizer.analyze(content);

      expect(result).toHaveProperty('citations');
      expect(result).toHaveProperty('statistics');
      expect(result.profile).toBe('ARW-2.1');
    });

    it('should respect feature flags', async () => {
      const optimizer = new GEOOptimizer({
        profile: 'ARW-2.1',
        enabledFeatures: {
          citations: false
        }
      });

      const content = '[cite:1]: {"source": "Test", "type": "academic", "url": "https://example.com", "date": "2024-01-01"}';
      const result = await optimizer.analyze(content);

      expect(result.citations).toBeUndefined();
    });

    it('should calculate quality metrics when enabled', async () => {
      const optimizer = new GEOOptimizer({
        profile: 'ARW-2.2'
      });

      const content = 'This is a test article with multiple sentences. It discusses various topics.';
      const result = await optimizer.analyze(content, {
        calculateQuality: true
      });

      expect(result.quality).toBeDefined();
    });
  });

  describe('generateManifest', () => {
    it('should generate GEO metadata manifest', () => {
      const optimizer = new GEOOptimizer({
        profile: 'ARW-2.1'
      });

      const manifest = optimizer.generateManifest({
        contentPages: [
          {
            url: '/page1',
            machineView: '/page1.llm.md',
            citationsCount: 5,
            statisticsCount: 3
          }
        ]
      });

      expect(manifest.version).toBe('0.2');
      expect(manifest.profile).toBe('ARW-2.1');
      expect(manifest.geo_enhancements.citations_enabled).toBe(true);
      expect(manifest.content_pages).toHaveLength(1);
    });

    it('should reflect ARW-2.2 features in manifest', () => {
      const optimizer = new GEOOptimizer({
        profile: 'ARW-2.2'
      });

      const manifest = optimizer.generateManifest({
        contentPages: []
      });

      expect(manifest.geo_enhancements.entity_enrichment).toBe(true);
      expect(manifest.geo_enhancements.semantic_clustering).toBe(true);
    });
  });

  describe('getRecommendations', () => {
    it('should provide recommendations for domain', () => {
      const optimizer = new GEOOptimizer({
        profile: 'ARW-2.1',
        domain: 'ecommerce'
      });

      const recommendations = optimizer.getRecommendations();

      expect(recommendations).toHaveProperty('priorities');
      expect(recommendations).toHaveProperty('schemas');
      expect(recommendations).toHaveProperty('estimatedImpact');
      expect(Array.isArray(recommendations.priorities)).toBe(true);
    });

    it('should show higher impact for ARW-2.2', () => {
      const optimizer21 = new GEOOptimizer({ profile: 'ARW-2.1' });
      const optimizer22 = new GEOOptimizer({ profile: 'ARW-2.2' });

      const rec21 = optimizer21.getRecommendations();
      const rec22 = optimizer22.getRecommendations();

      expect(rec22.estimatedImpact).toContain('230-290%');
      expect(rec21.estimatedImpact).toContain('140-170%');
    });
  });
});
