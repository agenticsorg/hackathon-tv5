/**
 * Tests for Citation Framework (ARW-GEO-1)
 */

import { CitationFramework } from '../src/citations';
import { Citation } from '../src/types';

describe('CitationFramework', () => {
  let framework: CitationFramework;

  beforeEach(() => {
    framework = new CitationFramework();
  });

  describe('extractFromMarkdown', () => {
    it('should extract citations from markdown content', () => {
      const content = `
Some content with citation^[cite:1]

[cite:1]: {
  "source": "Test Source",
  "type": "academic",
  "url": "https://example.com",
  "date": "2024-01-01"
}
      `;

      const citations = framework.extractFromMarkdown(content);
      expect(citations).toHaveLength(1);
      expect(citations[0].id).toBe('cite:1');
      expect(citations[0].source).toBe('Test Source');
    });

    it('should handle multiple citations', () => {
      const content = `
Text^[cite:1] more text^[cite:2]

[cite:1]: {"source": "Source 1", "type": "academic", "url": "https://example.com/1", "date": "2024-01-01"}
[cite:2]: {"source": "Source 2", "type": "news", "url": "https://example.com/2", "date": "2024-02-01"}
      `;

      const citations = framework.extractFromMarkdown(content);
      expect(citations).toHaveLength(2);
    });

    it('should handle malformed citations gracefully', () => {
      const content = `
Text^[cite:1]
[cite:1]: {invalid json}
      `;

      const citations = framework.extractFromMarkdown(content);
      expect(citations).toHaveLength(0);
    });
  });

  describe('calculateAuthorityScore', () => {
    it('should score academic citations higher', () => {
      const academic: Partial<Citation> = {
        type: 'academic',
        date: '2024-01-01',
        doi: '10.1234/test'
      };

      const score = framework.calculateAuthorityScore(academic);
      expect(score).toBeGreaterThan(0.8);
    });

    it('should give recency bonus for recent citations', () => {
      const recent: Partial<Citation> = {
        type: 'academic',
        date: new Date().toISOString()
      };

      const old: Partial<Citation> = {
        type: 'academic',
        date: '2015-01-01'
      };

      const recentScore = framework.calculateAuthorityScore(recent);
      const oldScore = framework.calculateAuthorityScore(old);
      expect(recentScore).toBeGreaterThan(oldScore);
    });

    it('should cap score at 1.0', () => {
      const maxCitation: Partial<Citation> = {
        type: 'academic',
        date: new Date().toISOString(),
        doi: '10.1234/test'
      };

      const score = framework.calculateAuthorityScore(maxCitation);
      expect(score).toBeLessThanOrEqual(1.0);
    });
  });

  describe('validate', () => {
    it('should validate complete citation', () => {
      const citation: Citation = {
        id: 'cite:1',
        source: 'Test Source',
        type: 'academic',
        url: 'https://example.com',
        date: '2024-01-01'
      };

      const result = framework.validate(citation);
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should detect missing required fields', () => {
      const incomplete = {} as Citation;
      const result = framework.validate(incomplete);
      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it('should validate date format', () => {
      const invalidDate: Citation = {
        id: 'cite:1',
        source: 'Test',
        type: 'academic',
        url: 'https://example.com',
        date: 'invalid-date'
      };

      const result = framework.validate(invalidDate);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain(expect.stringContaining('date format'));
    });

    it('should validate authority score range', () => {
      const invalidScore: Citation = {
        id: 'cite:1',
        source: 'Test',
        type: 'academic',
        url: 'https://example.com',
        date: '2024-01-01',
        authority_score: 1.5
      };

      const result = framework.validate(invalidScore);
      expect(result.valid).toBe(false);
    });
  });

  describe('formatForMachineView', () => {
    it('should format citations for machine view', () => {
      const citations: Citation[] = [{
        id: 'cite:1',
        source: 'Test Source',
        type: 'academic',
        url: 'https://example.com',
        date: '2024-01-01'
      }];

      const output = framework.formatForMachineView(citations);
      expect(output).toContain('<!-- citations -->');
      expect(output).toContain('[cite:1]:');
      expect(output).toContain('Test Source');
    });
  });

  describe('toSchemaOrg', () => {
    it('should convert citation to Schema.org format', () => {
      const citation: Citation = {
        id: 'cite:1',
        source: 'Test Source',
        type: 'academic',
        url: 'https://example.com',
        date: '2024-01-01',
        author: 'John Doe',
        publisher: 'Test Publisher'
      };

      const schema = framework.toSchemaOrg(citation);
      expect(schema).toHaveProperty('@type', 'CreativeWork');
      expect(schema).toHaveProperty('name', 'Test Source');
      expect(schema).toHaveProperty('url', 'https://example.com');
    });

    it('should handle complex author object', () => {
      const citation: Citation = {
        id: 'cite:1',
        source: 'Test Source',
        type: 'academic',
        url: 'https://example.com',
        date: '2024-01-01',
        author: {
          name: 'Dr. Jane Smith',
          affiliation: 'University',
          credentials: ['PhD']
        }
      };

      const schema = framework.toSchemaOrg(citation) as any;
      expect(schema.author).toHaveProperty('@type', 'Person');
      expect(schema.author.name).toBe('Dr. Jane Smith');
    });
  });
});
