/**
 * Tests for Content Quality Signals (ARW-GEO-4)
 */

import { ContentQualitySignals } from '../src/quality';

describe('ContentQualitySignals', () => {
  let quality: ContentQualitySignals;

  beforeEach(() => {
    quality = new ContentQualitySignals();
  });

  describe('calculateReadability', () => {
    it('should calculate readability metrics', () => {
      const text = 'This is a simple test. It has short sentences. They are easy to read.';
      const metrics = quality.calculateReadability(text);

      expect(metrics).toHaveProperty('flesch_kincaid_grade');
      expect(metrics).toHaveProperty('gunning_fog');
      expect(metrics).toHaveProperty('smog_index');
      expect(metrics).toHaveProperty('reading_ease');
      expect(metrics).toHaveProperty('average_sentence_length');
      expect(metrics).toHaveProperty('complex_words_percent');
    });

    it('should score simple text as more readable', () => {
      const simple = 'The cat sat on the mat. It was a nice day.';
      const complex = 'The phenomenological manifestation of contemporary socioeconomic paradigms necessitates comprehensive analytical frameworks.';

      const simpleMetrics = quality.calculateReadability(simple);
      const complexMetrics = quality.calculateReadability(complex);

      expect(simpleMetrics.flesch_kincaid_grade).toBeLessThan(complexMetrics.flesch_kincaid_grade);
      expect(simpleMetrics.reading_ease).toBeGreaterThan(complexMetrics.reading_ease);
    });
  });

  describe('calculateFluencyScore', () => {
    it('should return score between 0 and 1', () => {
      const text = 'This is a test. It should work well.';
      const score = quality.calculateFluencyScore(text);

      expect(score).toBeGreaterThanOrEqual(0);
      expect(score).toBeLessThanOrEqual(1);
    });

    it('should score moderate text around 0.5-0.8', () => {
      const text = `
        This article discusses the importance of content quality.
        Research shows that readable content performs better in search.
        We analyze various metrics to assess content quality.
      `.trim();

      const score = quality.calculateFluencyScore(text);
      expect(score).toBeGreaterThan(0.3);
      expect(score).toBeLessThan(1.0);
    });
  });

  describe('calculateEEATScore', () => {
    it('should calculate E-E-A-T score', () => {
      const score = quality.calculateEEATScore({
        citations: 15,
        quotations: 8,
        authorCredentials: ['PhD', 'Professor', '20 years'],
        firstPartyData: true,
        factChecked: true
      });

      expect(score).toBeGreaterThan(0.8);
    });

    it('should score minimal content lower', () => {
      const score = quality.calculateEEATScore({
        citations: 0,
        quotations: 0,
        authorCredentials: [],
        firstPartyData: false,
        factChecked: false
      });

      expect(score).toBeLessThan(0.3);
    });

    it('should cap score at 1.0', () => {
      const score = quality.calculateEEATScore({
        citations: 100,
        quotations: 100,
        authorCredentials: ['PhD', 'MD', 'Professor', '50 years'],
        firstPartyData: true,
        factChecked: true
      });

      expect(score).toBeLessThanOrEqual(1.0);
    });
  });

  describe('analyzeQuality', () => {
    it('should provide quality analysis and recommendations', () => {
      const text = 'This is a test. Simple sentences.';
      const analysis = quality.analyzeQuality(text);

      expect(analysis).toHaveProperty('fluency');
      expect(analysis).toHaveProperty('readability');
      expect(analysis).toHaveProperty('recommendations');
      expect(Array.isArray(analysis.recommendations)).toBe(true);
    });

    it('should recommend simplification for complex text', () => {
      const complex = 'The multifaceted dimensionality of contemporary epistemological frameworks necessitates comprehensive reevaluation.';
      const analysis = quality.analyzeQuality(complex);

      expect(analysis.recommendations.some(r =>
        r.toLowerCase().includes('simpl') || r.toLowerCase().includes('difficult')
      )).toBe(true);
    });

    it('should recommend adding depth for simple text', () => {
      const simple = 'It is good. Very good. Really good.';
      const analysis = quality.analyzeQuality(simple);

      expect(analysis.recommendations.length).toBeGreaterThan(0);
    });
  });
});
