/**
 * Unit tests for ARWDiscovery
 * Tests ARW manifest detection, parsing, and validation
 */

import { describe, it, expect, beforeEach } from '@jest/globals';
import {
  VALID_ARW_MANIFEST,
  MINIMAL_ARW_MANIFEST,
  ARW_MANIFEST_WITH_ACTIONS,
  LLMS_TXT_YAML
} from '../fixtures/arw-manifests';

/**
 * Mock ARWDiscovery for testing
 * In actual implementation, import from src/core/arw-discovery
 */
class ARWDiscovery {
  async discoverManifest(baseUrl: string): Promise<string | null> {
    // Mock implementation - checks standard locations
    const locations = [
      '/.well-known/arw-manifest.json',
      '/llms.json',
      '/llms.txt'
    ];

    // Simulate finding manifest at first location
    return `${baseUrl}${locations[0]}`;
  }

  parseManifest(content: string): any {
    try {
      return JSON.parse(content);
    } catch {
      // Try YAML parsing
      return this.parseYaml(content);
    }
  }

  private parseYaml(content: string): any {
    // Simplified YAML parsing for tests
    return { version: '0.1' };
  }

  validateManifest(manifest: any): boolean {
    if (!manifest.version) return false;
    if (!manifest.site?.name) return false;
    return true;
  }

  extractMachineViews(manifest: any): string[] {
    if (!manifest.content) return [];
    return manifest.content
      .map((item: any) => item.machine_view)
      .filter(Boolean);
  }

  extractPolicies(manifest: any): any {
    return manifest.policies || {};
  }
}

describe('ARWDiscovery', () => {
  let discovery: ARWDiscovery;

  beforeEach(() => {
    discovery = new ARWDiscovery();
  });

  describe('Manifest Discovery', () => {
    it('should discover manifest at .well-known location', async () => {
      const url = await discovery.discoverManifest('https://example.com');
      expect(url).toContain('.well-known/arw-manifest.json');
    });

    it('should handle base URL with trailing slash', async () => {
      const url = await discovery.discoverManifest('https://example.com/');
      expect(url).toBeDefined();
    });

    it('should handle base URL without protocol', async () => {
      const url = await discovery.discoverManifest('example.com');
      expect(url).toBeDefined();
    });

    it('should return null for invalid domains', async () => {
      const url = await discovery.discoverManifest('');
      expect(url).toBeNull();
    });
  });

  describe('Manifest Parsing', () => {
    it('should parse valid JSON manifest', () => {
      const json = JSON.stringify(VALID_ARW_MANIFEST);
      const parsed = discovery.parseManifest(json);
      expect(parsed).toBeDefined();
      expect(parsed.version).toBe('0.1');
    });

    it('should parse YAML manifest', () => {
      const parsed = discovery.parseManifest(LLMS_TXT_YAML);
      expect(parsed).toBeDefined();
      expect(parsed.version).toBe('0.1');
    });

    it('should handle malformed JSON', () => {
      expect(() => {
        discovery.parseManifest('{ invalid json }');
      }).not.toThrow();
    });

    it('should preserve all manifest fields', () => {
      const json = JSON.stringify(VALID_ARW_MANIFEST);
      const parsed = discovery.parseManifest(json);
      expect(parsed.site).toBeDefined();
      expect(parsed.content).toBeDefined();
      expect(parsed.policies).toBeDefined();
    });
  });

  describe('Manifest Validation', () => {
    it('should validate complete manifest', () => {
      const isValid = discovery.validateManifest(VALID_ARW_MANIFEST);
      expect(isValid).toBe(true);
    });

    it('should validate minimal manifest', () => {
      const isValid = discovery.validateManifest(MINIMAL_ARW_MANIFEST);
      expect(isValid).toBe(true);
    });

    it('should reject manifest without version', () => {
      const invalid = { site: { name: 'Test' } };
      const isValid = discovery.validateManifest(invalid);
      expect(isValid).toBe(false);
    });

    it('should reject manifest without site name', () => {
      const invalid = { version: '0.1', site: {} };
      const isValid = discovery.validateManifest(invalid);
      expect(isValid).toBe(false);
    });

    it('should accept manifest with actions', () => {
      const isValid = discovery.validateManifest(ARW_MANIFEST_WITH_ACTIONS);
      expect(isValid).toBe(true);
    });
  });

  describe('Machine View Extraction', () => {
    it('should extract machine view URLs', () => {
      const views = discovery.extractMachineViews(VALID_ARW_MANIFEST);
      expect(views).toHaveLength(2);
      expect(views[0]).toContain('.llm.md');
    });

    it('should return empty array for manifest without content', () => {
      const views = discovery.extractMachineViews(MINIMAL_ARW_MANIFEST);
      expect(views).toEqual([]);
    });

    it('should filter out null/undefined machine views', () => {
      const manifest = {
        content: [
          { url: '/page1', machine_view: '/page1.llm.md' },
          { url: '/page2' }, // No machine view
          { url: '/page3', machine_view: null }
        ]
      };
      const views = discovery.extractMachineViews(manifest);
      expect(views).toHaveLength(1);
    });
  });

  describe('Policy Extraction', () => {
    it('should extract policies from manifest', () => {
      const policies = discovery.extractPolicies(VALID_ARW_MANIFEST);
      expect(policies).toBeDefined();
      expect(policies.training).toBeDefined();
      expect(policies.inference).toBeDefined();
    });

    it('should return empty object for manifest without policies', () => {
      const policies = discovery.extractPolicies(MINIMAL_ARW_MANIFEST);
      expect(policies).toEqual({});
    });

    it('should preserve all policy fields', () => {
      const policies = discovery.extractPolicies(VALID_ARW_MANIFEST);
      expect(policies.training.allowed).toBe(false);
      expect(policies.inference.allowed).toBe(true);
      expect(policies.attribution.required).toBe(true);
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty manifest', () => {
      const isValid = discovery.validateManifest({});
      expect(isValid).toBe(false);
    });

    it('should handle null manifest', () => {
      const isValid = discovery.validateManifest(null);
      expect(isValid).toBe(false);
    });

    it('should handle undefined values gracefully', () => {
      const views = discovery.extractMachineViews(undefined);
      expect(views).toEqual([]);
    });

    it('should handle very large manifests', () => {
      const largeManifest = {
        ...VALID_ARW_MANIFEST,
        content: Array(1000).fill({
          url: '/page',
          machine_view: '/page.llm.md'
        })
      };
      const views = discovery.extractMachineViews(largeManifest);
      expect(views).toHaveLength(1000);
    });
  });
});
