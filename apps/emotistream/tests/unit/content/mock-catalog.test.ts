/**
 * MockCatalogGenerator Unit Tests
 */

import { MockCatalogGenerator } from '../../../src/content/mock-catalog';

describe('MockCatalogGenerator', () => {
  let generator: MockCatalogGenerator;

  beforeEach(() => {
    generator = new MockCatalogGenerator();
  });

  describe('generate', () => {
    it('should generate 200 diverse mock content items', () => {
      const catalog = generator.generate(200);

      expect(catalog.length).toBe(200);
    });

    it('should include all categories', () => {
      const catalog = generator.generate(200);
      const categories = new Set(catalog.map(c => c.category));

      expect(categories.has('movie')).toBe(true);
      expect(categories.has('series')).toBe(true);
      expect(categories.has('documentary')).toBe(true);
      expect(categories.has('music')).toBe(true);
      expect(categories.has('meditation')).toBe(true);
      expect(categories.has('short')).toBe(true);
    });

    it('should have valid content metadata', () => {
      const catalog = generator.generate(50);

      catalog.forEach(content => {
        expect(content.contentId).toBeDefined();
        expect(content.title).toBeDefined();
        expect(content.description).toBeDefined();
        expect(content.platform).toBe('mock');
        expect(content.genres.length).toBeGreaterThan(0);
        expect(content.tags.length).toBeGreaterThan(0);
        expect(content.duration).toBeGreaterThan(0);
      });
    });

    it('should generate diverse genres', () => {
      const catalog = generator.generate(200);
      const allGenres = catalog.flatMap(c => c.genres);
      const uniqueGenres = new Set(allGenres);

      expect(uniqueGenres.size).toBeGreaterThan(10);
    });
  });
});
