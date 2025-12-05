/**
 * Tests for ARW schema validation
 */

import { describe, it, expect } from 'vitest';
import {
  validateManifest,
  validateWellKnownManifest,
  validateWellKnownPolicies,
  validateWellKnownContentIndex,
  validateByType,
} from '../validation';
import type { ARWManifest, WellKnownManifest, WellKnownPolicies, WellKnownContentIndex } from '../types';
import { PolicyPermission } from '../types';

describe('validateManifest', () => {
  it('should validate a valid manifest', () => {
    const validManifest: ARWManifest = {
      version: '1.0',
      profile: 'ARW-1',
      site: {
        name: 'Test Site',
        description: 'A test site',
        homepage: 'https://example.com',
      },
    };

    const result = validateManifest(validManifest);
    expect(result.valid).toBe(true);
    expect(result.data).toEqual(validManifest);
    expect(result.errors).toBeUndefined();
  });

  it('should reject null data with clear error message', () => {
    const result = validateManifest(null);
    expect(result.valid).toBe(false);
    expect(result.errors).toBeDefined();
    expect(result.errors?.[0]?.message).toContain('null');
  });

  it('should reject undefined data with clear error message', () => {
    const result = validateManifest(undefined);
    expect(result.valid).toBe(false);
    expect(result.errors).toBeDefined();
    expect(result.errors?.[0]?.message).toContain('undefined');
  });

  it('should reject manifest missing required fields', () => {
    const invalidManifest = {
      version: '1.0',
      // Missing profile and site
    };

    const result = validateManifest(invalidManifest);
    expect(result.valid).toBe(false);
    expect(result.errors).toBeDefined();
    expect(result.errors!.length).toBeGreaterThan(0);
  });

  it('should reject manifest with invalid version', () => {
    const invalidManifest = {
      version: 123, // Should be string
      profile: 'ARW-1',
      site: {
        name: 'Test',
        description: 'Test',
        homepage: 'https://example.com',
      },
    };

    const result = validateManifest(invalidManifest);
    expect(result.valid).toBe(false);
    expect(result.errors).toBeDefined();
  });
});

describe('validateWellKnownManifest', () => {
  it('should validate a valid well-known manifest', () => {
    const validManifest: WellKnownManifest = {
      arw_version: '1.0',
      site: {
        name: 'Test Site',
        description: 'A test site',
        homepage: 'https://example.com',
      },
      links: {
        guide: 'https://example.com/llms.txt',
        policies: 'https://example.com/.well-known/arw-policies.json',
      },
    };

    const result = validateWellKnownManifest(validManifest);
    expect(result.valid).toBe(true);
    expect(result.data).toEqual(validManifest);
  });

  it('should reject null data', () => {
    const result = validateWellKnownManifest(null);
    expect(result.valid).toBe(false);
    expect(result.errors).toBeDefined();
  });
});

describe('validateWellKnownPolicies', () => {
  it('should validate valid policies', () => {
    const validPolicies: WellKnownPolicies = {
      training: {
        allowed: PolicyPermission.Allow,
        description: 'Training allowed with attribution',
      },
      inference: {
        allowed: PolicyPermission.Allow,
        description: 'Inference allowed with some restrictions',
      },
    };

    const result = validateWellKnownPolicies(validPolicies);
    expect(result.valid).toBe(true);
    expect(result.data).toEqual(validPolicies);
  });

  it('should reject null data', () => {
    const result = validateWellKnownPolicies(null);
    expect(result.valid).toBe(false);
  });
});

describe('validateWellKnownContentIndex', () => {
  it('should validate valid content index', () => {
    const validIndex: WellKnownContentIndex = {
      version: '1.0',
      items: [],
      total: 0,
    };

    const result = validateWellKnownContentIndex(validIndex);
    expect(result.valid).toBe(true);
    expect(result.data).toEqual(validIndex);
  });

  it('should reject null data', () => {
    const result = validateWellKnownContentIndex(null);
    expect(result.valid).toBe(false);
  });

  it('should validate content index with entries', () => {
    const validIndex: WellKnownContentIndex = {
      version: '1.0',
      items: [
        {
          id: 'page1',
          type: 'llm.md',
          url: 'https://example.com/page1',
          title: 'Page 1',
        },
      ],
      next: 'https://example.com/.well-known/arw-content-index.json?page=2',
      total: 10,
    };

    const result = validateWellKnownContentIndex(validIndex);
    expect(result.valid).toBe(true);
  });
});

describe('validateByType', () => {
  it('should route to correct validator for manifest type', () => {
    const data: ARWManifest = {
      version: '1.0',
      profile: 'ARW-1',
      site: {
        name: 'Test',
        description: 'Test',
        homepage: 'https://example.com',
      },
    };

    const result = validateByType('manifest', data);
    expect(result.valid).toBe(true);
  });

  it('should route to correct validator for well-known-manifest type', () => {
    const data: WellKnownManifest = {
      arw_version: '1.0',
      site: {
        name: 'Test',
        description: 'Test',
        homepage: 'https://example.com',
      },
      links: {
        guide: 'https://example.com/llms.txt',
      },
    };

    const result = validateByType('well-known-manifest', data);
    expect(result.valid).toBe(true);
  });

  it('should route to correct validator for policies type', () => {
    const data: WellKnownPolicies = {
      training: PolicyPermission.Allow,
    };

    const result = validateByType('policies', data);
    expect(result.valid).toBe(true);
  });

  it('should route to correct validator for content-index type', () => {
    const data: WellKnownContentIndex = {
      version: '1.0',
      items: [],
      total: 0,
    };

    const result = validateByType('content-index', data);
    expect(result.valid).toBe(true);
  });
});

describe('Edge Cases', () => {
  it('should handle empty objects', () => {
    const result = validateManifest({});
    expect(result.valid).toBe(false);
    expect(result.errors).toBeDefined();
  });

  it('should handle arrays as input', () => {
    const result = validateManifest([]);
    expect(result.valid).toBe(false);
  });

  it('should handle primitive values', () => {
    const result = validateManifest('string');
    expect(result.valid).toBe(false);
  });
});
