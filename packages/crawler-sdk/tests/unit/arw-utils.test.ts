/**
 * Unit tests for ARW utilities
 * Tests ARW-specific helper functions
 */

import { describe, it, expect } from '@jest/globals';

class ARWUtils {
  isArwCompliant(html: string): boolean {
    return html.includes('arw-manifest.json') || html.includes('llms.txt');
  }

  extractManifestUrl(html: string): string | null {
    const match = html.match(/href=["']([^"']*arw-manifest\.json[^"']*)["']/i);
    return match ? match[1] : null;
  }

  parseChunkId(html: string): string[] {
    const chunks: string[] = [];
    const regex = /data-chunk-id=["']([^"']+)["']/gi;
    let match;
    while ((match = regex.exec(html)) !== null) {
      chunks.push(match[1]);
    }
    return chunks;
  }
}

describe('ARWUtils', () => {
  let utils: ARWUtils;

  beforeEach(() => {
    utils = new ARWUtils();
  });

  describe('ARW Compliance Detection', () => {
    it('should detect ARW-compliant HTML', () => {
      const html = '<link rel="alternate" href="/.well-known/arw-manifest.json">';
      expect(utils.isArwCompliant(html)).toBe(true);
    });

    it('should detect llms.txt reference', () => {
      const html = '<link href="/llms.txt">';
      expect(utils.isArwCompliant(html)).toBe(true);
    });

    it('should return false for non-compliant HTML', () => {
      const html = '<html><body>No ARW</body></html>';
      expect(utils.isArwCompliant(html)).toBe(false);
    });
  });

  describe('Manifest URL Extraction', () => {
    it('should extract manifest URL', () => {
      const html = '<link href="/.well-known/arw-manifest.json">';
      const url = utils.extractManifestUrl(html);
      expect(url).toBe('/.well-known/arw-manifest.json');
    });

    it('should return null when no manifest found', () => {
      const html = '<html><body>No manifest</body></html>';
      const url = utils.extractManifestUrl(html);
      expect(url).toBeNull();
    });
  });

  describe('Chunk ID Parsing', () => {
    it('should extract chunk IDs', () => {
      const html = '<div data-chunk-id="intro"></div><div data-chunk-id="content"></div>';
      const chunks = utils.parseChunkId(html);
      expect(chunks).toEqual(['intro', 'content']);
    });

    it('should return empty array when no chunks', () => {
      const html = '<div>No chunks</div>';
      const chunks = utils.parseChunkId(html);
      expect(chunks).toEqual([]);
    });
  });
});
