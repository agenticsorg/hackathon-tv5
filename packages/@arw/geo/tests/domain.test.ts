/**
 * Tests for Domain-Specific Optimization (ARW-GEO-7)
 */

import { DomainOptimization } from '../src/domain';

describe('DomainOptimization', () => {
  let domain: DomainOptimization;

  beforeEach(() => {
    domain = new DomainOptimization();
  });

  describe('classify', () => {
    it('should classify e-commerce sites', () => {
      const result = domain.classify({
        urls: ['/products/item-1', '/cart', '/checkout'],
        content: 'Price: $29.99, SKU: ABC123, Add to Cart'
      });

      expect(result.primary).toBe('ecommerce');
      expect(result.confidence).toBeGreaterThan(0.5);
    });

    it('should classify SaaS/documentation sites', () => {
      const result = domain.classify({
        urls: ['/docs/api', '/docs/integration'],
        content: 'API authentication endpoint documentation integration webhook'
      });

      expect(result.primary).toBe('saas');
      expect(result.confidence).toBeGreaterThan(0.5);
    });

    it('should classify media publishing sites', () => {
      const result = domain.classify({
        urls: ['/articles/tech-news', '/news/breaking'],
        content: 'Author: John Doe, Published: 2024-01-01, Editorial breaking updated'
      });

      expect(result.primary).toBe('media_publishing');
      expect(result.confidence).toBeGreaterThan(0.5);
    });

    it('should identify secondary domains', () => {
      const result = domain.classify({
        urls: ['/docs', '/products'],
        content: 'API documentation price SKU cart'
      });

      expect(result.secondary).toBeDefined();
      expect(result.secondary!.length).toBeGreaterThan(0);
    });
  });

  describe('getProfile', () => {
    it('should return profile for known domain', () => {
      const profile = domain.getProfile('ecommerce');

      expect(profile).toBeDefined();
      expect(profile!.domain).toBe('ecommerce');
      expect(profile!.recommended_schemas).toContain('Product');
      expect(profile!.geo_priorities).toContain('statistics');
    });

    it('should return undefined for unknown domain', () => {
      const profile = domain.getProfile('unknown' as any);
      expect(profile).toBeUndefined();
    });
  });

  describe('getGEOPriorities', () => {
    it('should return priorities for e-commerce', () => {
      const priorities = domain.getGEOPriorities('ecommerce');

      expect(Array.isArray(priorities)).toBe(true);
      expect(priorities.length).toBeGreaterThan(0);
    });

    it('should return different priorities for different domains', () => {
      const ecommercePriorities = domain.getGEOPriorities('ecommerce');
      const saasPriorities = domain.getGEOPriorities('saas');

      expect(ecommercePriorities).not.toEqual(saasPriorities);
    });
  });
});
