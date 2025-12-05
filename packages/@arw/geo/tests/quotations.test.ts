/**
 * Tests for Quotation System (ARW-GEO-3)
 */

import { QuotationSystem } from '../src/quotations';
import { Quotation, Speaker } from '../src/types';

describe('QuotationSystem', () => {
  let system: QuotationSystem;

  beforeEach(() => {
    system = new QuotationSystem();
  });

  describe('extractFromMarkdown', () => {
    it('should extract quotations from markdown', () => {
      const content = `
<!-- quote:expert-1 -->
> "This is a test quote"
\`\`\`quotation
{
  "speaker": {
    "name": "Dr. Test",
    "title": "Expert",
    "affiliation": "Test Org"
  },
  "date": "2024-01-01",
  "source": "Test Interview",
  "type": "expert_opinion"
}
\`\`\`
      `;

      const quotes = system.extractFromMarkdown(content);
      expect(quotes).toHaveLength(1);
      expect(quotes[0].id).toBe('quote:expert-1');
      expect(quotes[0].text).toBe('This is a test quote');
    });
  });

  describe('calculateSpeakerAuthority', () => {
    it('should score PhDs highly', () => {
      const speaker: Speaker = {
        name: 'Dr. Test',
        title: 'Professor',
        affiliation: 'University',
        credentials: ['PhD Computer Science'],
        verified: true
      };

      const score = system.calculateSpeakerAuthority(speaker);
      expect(score).toBeGreaterThan(0.8);
    });

    it('should score C-level executives well', () => {
      const speaker: Speaker = {
        name: 'Test CEO',
        title: 'CEO',
        affiliation: 'Tech Company',
        credentials: ['20 years experience']
      };

      const score = system.calculateSpeakerAuthority(speaker);
      expect(score).toBeGreaterThan(0.6);
    });

    it('should cap score at 1.0', () => {
      const speaker: Speaker = {
        name: 'Test',
        title: 'Chief Director',
        affiliation: 'Org',
        credentials: ['PhD', 'Professor', '30 years'],
        verified: true
      };

      const score = system.calculateSpeakerAuthority(speaker);
      expect(score).toBeLessThanOrEqual(1.0);
    });
  });

  describe('validate', () => {
    it('should validate complete quotation', () => {
      const quote: Quotation = {
        id: 'quote:1',
        text: 'Test quote',
        speaker: {
          name: 'Test',
          title: 'Expert',
          affiliation: 'Org'
        },
        date: '2024-01-01',
        source: 'Interview',
        type: 'expert_opinion'
      };

      const result = system.validate(quote);
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should detect missing speaker fields', () => {
      const quote: Quotation = {
        id: 'quote:1',
        text: 'Test',
        speaker: {} as Speaker,
        date: '2024-01-01',
        source: 'Test',
        type: 'expert_opinion'
      };

      const result = system.validate(quote);
      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.includes('speaker'))).toBe(true);
    });
  });

  describe('extractFromText', () => {
    it('should extract direct quotes from text', () => {
      const text = 'The expert said "This is a significant finding that will change the industry."';
      const quotes = system.extractFromText(text);

      expect(quotes.length).toBeGreaterThan(0);
      expect(quotes[0].quote).toContain('significant finding');
    });

    it('should extract context around quotes', () => {
      const text = 'According to Dr. Smith, "Innovation is key." This was during the conference.';
      const quotes = system.extractFromText(text);

      expect(quotes[0].context).toContain('Dr. Smith');
    });

    it('should handle multiple quotes', () => {
      const text = '"First quote" and later "Second quote"';
      const quotes = system.extractFromText(text);

      expect(quotes).toHaveLength(2);
    });
  });

  describe('toSchemaOrg', () => {
    it('should convert quotation to Schema.org format', () => {
      const quote: Quotation = {
        id: 'quote:1',
        text: 'Test quote',
        speaker: {
          name: 'Dr. Test',
          title: 'Expert',
          affiliation: 'University',
          credentials: ['PhD']
        },
        date: '2024-01-01',
        source: 'Test Conference',
        type: 'expert_opinion'
      };

      const schema = system.toSchemaOrg(quote) as any;
      expect(schema).toHaveProperty('@type', 'Quotation');
      expect(schema.text).toBe('Test quote');
      expect(schema.spokenByCharacter.name).toBe('Dr. Test');
    });
  });
});
