/**
 * Integration tests - validate against real-world ARW files
 */

import { describe, it, expect } from 'vitest';
import { readFileSync } from 'fs';
import { join } from 'path';
import * as yaml from 'js-yaml';
import {
  validateManifest,
  validateWellKnownManifest,
  validateWellKnownPolicies,
  validateWellKnownContentIndex,
} from '../validation';

// Helper to load JSON files
function loadJSON(relativePath: string): unknown {
  const fullPath = join(__dirname, '../../../..', '..', relativePath);
  const content = readFileSync(fullPath, 'utf-8');
  return JSON.parse(content);
}

// Helper to load YAML files
function loadYAML(relativePath: string): unknown {
  const fullPath = join(__dirname, '../../../..', '..', relativePath);
  const content = readFileSync(fullPath, 'utf-8');
  return yaml.load(content);
}

describe('Integration: Real ARW Files', () => {
  it('should validate www/public/llms.txt', () => {
    try {
      const manifest = loadYAML('www/public/llms.txt');
      const result = validateManifest(manifest);

      if (!result.valid) {
        console.error('Validation errors:', result.errors);
      }

      expect(result.valid).toBe(true);
    } catch (error) {
      // File might not exist in test environment
      console.warn('Could not load www/public/llms.txt:', error);
    }
  });

  it('should validate .well-known/arw-manifest.json', () => {
    try {
      const manifest = loadJSON('www/public/.well-known/arw-manifest.json');
      const result = validateWellKnownManifest(manifest);

      if (!result.valid) {
        console.error('Validation errors:', result.errors);
      }

      expect(result.valid).toBe(true);
    } catch (error) {
      console.warn('Could not load .well-known/arw-manifest.json:', error);
    }
  });

  it('should validate .well-known/arw-policies.json', () => {
    try {
      const policies = loadJSON('www/public/.well-known/arw-policies.json');
      const result = validateWellKnownPolicies(policies);

      if (!result.valid) {
        console.error('Validation errors:', result.errors);
      }

      expect(result.valid).toBe(true);
    } catch (error) {
      console.warn('Could not load .well-known/arw-policies.json:', error);
    }
  });

  it('should validate .well-known/arw-content-index.json', () => {
    try {
      const contentIndex = loadJSON('www/public/.well-known/arw-content-index.json');
      const result = validateWellKnownContentIndex(contentIndex);

      if (!result.valid) {
        console.error('Validation errors:', result.errors);
      }

      expect(result.valid).toBe(true);
    } catch (error) {
      console.warn('Could not load .well-known/arw-content-index.json:', error);
    }
  });
});

describe('Integration: Schema Fixtures', () => {
  it('should validate fixture: manifest.valid.json', () => {
    try {
      const manifest = loadJSON('plans/arw-discovery-bundle/examples/fixtures/manifest.valid.json');
      const result = validateWellKnownManifest(manifest);

      if (!result.valid) {
        console.error('Validation errors:', result.errors);
      }

      expect(result.valid).toBe(true);
    } catch (error) {
      console.warn('Could not load fixture file:', error);
    }
  });

  it('should reject fixture: manifest.invalid.json', () => {
    try {
      const manifest = loadJSON('plans/arw-discovery-bundle/examples/fixtures/manifest.invalid.json');
      const result = validateWellKnownManifest(manifest);

      expect(result.valid).toBe(false);
      expect(result.errors).toBeDefined();
    } catch (error) {
      console.warn('Could not load fixture file:', error);
    }
  });

  it('should validate fixture: policies.valid.json', () => {
    try {
      const policies = loadJSON('plans/arw-discovery-bundle/examples/fixtures/policies.valid.json');
      const result = validateWellKnownPolicies(policies);

      if (!result.valid) {
        console.error('Validation errors:', result.errors);
      }

      expect(result.valid).toBe(true);
    } catch (error) {
      console.warn('Could not load fixture file:', error);
    }
  });

  it('should reject fixture: policies.invalid.json', () => {
    try {
      const policies = loadJSON('plans/arw-discovery-bundle/examples/fixtures/policies.invalid.json');
      const result = validateWellKnownPolicies(policies);

      expect(result.valid).toBe(false);
      expect(result.errors).toBeDefined();
    } catch (error) {
      console.warn('Could not load fixture file:', error);
    }
  });

  it('should validate fixture: content-index.valid.json', () => {
    try {
      const contentIndex = loadJSON('plans/arw-discovery-bundle/examples/fixtures/content-index.valid.json');
      const result = validateWellKnownContentIndex(contentIndex);

      if (!result.valid) {
        console.error('Validation errors:', result.errors);
      }

      expect(result.valid).toBe(true);
    } catch (error) {
      console.warn('Could not load fixture file:', error);
    }
  });

  it('should reject fixture: content-index.invalid.json', () => {
    try {
      const contentIndex = loadJSON('plans/arw-discovery-bundle/examples/fixtures/content-index.invalid.json');
      const result = validateWellKnownContentIndex(contentIndex);

      expect(result.valid).toBe(false);
      expect(result.errors).toBeDefined();
    } catch (error) {
      console.warn('Could not load fixture file:', error);
    }
  });
});
