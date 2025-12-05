/**
 * Tests for ARW Discovery
 */

import { describe, it, expect } from '@jest/globals';
import { ARWHelper } from '../../src/arw/discovery';

describe('ARWHelper', () => {
  describe('generateLlmsTxtUrls()', () => {
    it('should generate potential llms.txt URLs', () => {
      const url = 'https://docs.example.com/guide';
      const urls = ARWHelper.generateLlmsTxtUrls(url);

      expect(urls).toContain('https://docs.example.com/.well-known/llms.txt');
      expect(urls).toContain('https://docs.example.com/llms.txt');
      expect(urls.length).toBeGreaterThan(0);
    });
  });

  describe('extractDomain()', () => {
    it('should extract domain from URL', () => {
      expect(ARWHelper.extractDomain('https://docs.example.com/path')).toBe(
        'docs.example.com'
      );
      expect(ARWHelper.extractDomain('http://api.example.com')).toBe(
        'api.example.com'
      );
    });
  });

  describe('likelyHasSupport()', () => {
    it('should detect likely ARW support', () => {
      expect(ARWHelper.likelyHasSupport('https://docs.example.com')).toBe(true);
      expect(ARWHelper.likelyHasSupport('https://api.example.com')).toBe(true);
      expect(ARWHelper.likelyHasSupport('https://developer.example.com')).toBe(
        true
      );
      expect(ARWHelper.likelyHasSupport('https://example.com')).toBe(false);
    });
  });

  describe('normalizeLlmsTxtUrl()', () => {
    it('should normalize llms.txt URL', () => {
      expect(
        ARWHelper.normalizeLlmsTxtUrl('https://example.com')
      ).toBe('https://example.com/.well-known/llms.txt');

      expect(
        ARWHelper.normalizeLlmsTxtUrl('https://example.com/', 'llms.txt')
      ).toBe('https://example.com/llms.txt');

      expect(
        ARWHelper.normalizeLlmsTxtUrl(
          'https://example.com',
          'https://cdn.example.com/llms.txt'
        )
      ).toBe('https://cdn.example.com/llms.txt');
    });
  });
});
