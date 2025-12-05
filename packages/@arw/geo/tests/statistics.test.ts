/**
 * Tests for Statistics Enhancement (ARW-GEO-2)
 */

import { StatisticsEnhancement } from '../src/statistics';
import { Statistic } from '../src/types';

describe('StatisticsEnhancement', () => {
  let enhancement: StatisticsEnhancement;

  beforeEach(() => {
    enhancement = new StatisticsEnhancement();
  });

  describe('extractFromMarkdown', () => {
    it('should extract statistics from markdown', () => {
      const content = `
\`\`\`statistics
{
  "id": "stat:1",
  "type": "market_metric",
  "value": 1000,
  "unit": "USD",
  "date": "2024-01-01",
  "source": "Test Source"
}
\`\`\`
      `;

      const stats = enhancement.extractFromMarkdown(content);
      expect(stats).toHaveLength(1);
      expect(stats[0].id).toBe('stat:1');
      expect(stats[0].value).toBe(1000);
    });

    it('should handle multiple statistics blocks', () => {
      const content = `
\`\`\`statistics
{"id": "stat:1", "type": "market_metric", "value": 100, "unit": "USD", "date": "2024-01-01", "source": "Test"}
\`\`\`

\`\`\`statistics
{"id": "stat:2", "type": "performance_metric", "value": 95, "unit": "percent", "date": "2024-02-01", "source": "Test"}
\`\`\`
      `;

      const stats = enhancement.extractFromMarkdown(content);
      expect(stats).toHaveLength(2);
    });
  });

  describe('calculateFreshnessScore', () => {
    it('should give high score for recent statistics', () => {
      const recent: Partial<Statistic> = {
        date: new Date().toISOString()
      };

      const score = enhancement.calculateFreshnessScore(recent);
      expect(score).toBeGreaterThan(0.9);
    });

    it('should give lower score for old statistics', () => {
      const old: Partial<Statistic> = {
        date: '2020-01-01'
      };

      const score = enhancement.calculateFreshnessScore(old);
      expect(score).toBeLessThan(0.5);
    });

    it('should handle missing date', () => {
      const noDate: Partial<Statistic> = {};
      const score = enhancement.calculateFreshnessScore(noDate);
      expect(score).toBe(0.5);
    });
  });

  describe('validate', () => {
    it('should validate complete statistic', () => {
      const stat: Statistic = {
        id: 'stat:1',
        type: 'market_metric',
        value: 1000,
        unit: 'USD',
        date: '2024-01-01',
        source: 'Test Source'
      };

      const result = enhancement.validate(stat);
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should detect missing fields', () => {
      const incomplete = {} as Statistic;
      const result = enhancement.validate(incomplete);
      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it('should validate confidence range', () => {
      const stat: Statistic = {
        id: 'stat:1',
        type: 'market_metric',
        value: 1000,
        unit: 'USD',
        date: '2024-01-01',
        source: 'Test',
        confidence: 1.5
      };

      const result = enhancement.validate(stat);
      expect(result.valid).toBe(false);
    });
  });

  describe('extractFromText', () => {
    it('should extract numerical statistics from text', () => {
      const text = 'The market grew by 25% reaching $1.5 billion in revenue.';
      const stats = enhancement.extractFromText(text);

      expect(stats.length).toBeGreaterThan(0);
      const percentStat = stats.find(s => s.unit.includes('percent'));
      expect(percentStat).toBeDefined();
      expect(percentStat?.value).toBe(25);
    });

    it('should handle large numbers with commas', () => {
      const text = 'The population reached 1,234,567 people.';
      const stats = enhancement.extractFromText(text);

      expect(stats.length).toBeGreaterThan(0);
    });

    it('should extract context around numbers', () => {
      const text = 'Revenue increased significantly by 42% over the previous year.';
      const stats = enhancement.extractFromText(text);

      expect(stats.length).toBeGreaterThan(0);
      expect(stats[0].context).toContain('Revenue');
    });
  });

  describe('formatForMachineView', () => {
    it('should format statistics for machine view', () => {
      const stats: Statistic[] = [{
        id: 'stat:1',
        type: 'market_metric',
        value: 1000,
        unit: 'USD',
        date: '2024-01-01',
        source: 'Test'
      }];

      const output = enhancement.formatForMachineView(stats);
      expect(output).toContain('<!-- stat:stat:1 -->');
      expect(output).toContain('```statistics');
    });
  });

  describe('toSchemaOrg', () => {
    it('should convert statistic to Schema.org Dataset', () => {
      const stat: Statistic = {
        id: 'stat:1',
        type: 'market_metric',
        value: 1000,
        unit: 'USD',
        date: '2024-01-01',
        source: 'Test Source'
      };

      const schema = enhancement.toSchemaOrg(stat);
      expect(schema).toHaveProperty('@type', 'Dataset');
      expect(schema).toHaveProperty('name', 'stat:1');
    });
  });
});
