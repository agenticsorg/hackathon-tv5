/**
 * Validation utilities for ARW schemas using AJV
 */

import Ajv, { type ErrorObject, type ValidateFunction } from 'ajv';
import addFormats from 'ajv-formats';
import {
  ARWModelSchema,
  ARWManifestSchema,
  ARWPoliciesSchema,
  ARWContentIndexSchema,
} from './schemas.js';
import type {
  ARWManifest,
  WellKnownManifest,
  WellKnownPolicies,
  WellKnownContentIndex,
  ToonMachineView,
  ToonBlock,
  ToonInline,
  ToonChunk,
  ContentItem,
  Chunk,
} from './types.js';

// ============================================================================
// Validation Error Type
// ============================================================================

export interface ValidationError {
  /** Path to the invalid field */
  path: string;
  /** Error message */
  message: string;
  /** Additional error data */
  data?: unknown;
}

export interface ValidationResult<T = unknown> {
  /** Whether validation succeeded */
  valid: boolean;
  /** Validation errors (if any) */
  errors?: ValidationError[];
  /** Validated data (if valid) */
  data?: T;
}

// ============================================================================
// AJV Setup
// ============================================================================

/**
 * Create AJV instance with ARW schemas
 */
function createValidator(): Ajv {
  const ajv = new Ajv({
    allErrors: true,
    verbose: true,
    strict: false,
  });

  addFormats(ajv);

  // Add all schemas
  ajv.addSchema(ARWModelSchema, 'arw-model');
  ajv.addSchema(ARWManifestSchema, 'arw-manifest');
  ajv.addSchema(ARWPoliciesSchema, 'arw-policies');
  ajv.addSchema(ARWContentIndexSchema, 'arw-content-index');

  return ajv;
}

const ajv = createValidator();

// ============================================================================
// Validation Utilities
// ============================================================================

/**
 * Format AJV errors into ValidationError array
 */
function formatErrors(errors: ErrorObject[] | null | undefined): ValidationError[] {
  if (!errors) return [];

  return errors.map((err) => ({
    path: err.instancePath || 'root',
    message: err.message || 'Validation error',
    data: err.params,
  }));
}

/**
 * Generic validation function
 */
function validate<T>(validator: ValidateFunction, data: unknown): ValidationResult<T> {
  // Add explicit null/undefined guards for better error messages
  if (data === null || data === undefined) {
    return {
      valid: false,
      errors: [
        {
          path: 'root',
          message: `Data is ${data === null ? 'null' : 'undefined'}. Expected an object.`,
        },
      ],
    };
  }

  const valid = validator(data);

  if (!valid) {
    return {
      valid: false,
      errors: formatErrors(validator.errors),
    };
  }

  return {
    valid: true,
    data: data as T,
  };
}

// ============================================================================
// ARW Manifest Validation (llms.txt)
// ============================================================================

const validateManifestSchema = ajv.getSchema('arw-model') as ValidateFunction;

/**
 * Validate an ARW manifest (llms.txt)
 *
 * @param data - The parsed manifest data (from YAML)
 * @returns Validation result with errors or validated data
 *
 * @example
 * ```typescript
 * import { validateManifest } from '@arw/schemas/validation';
 *
 * const result = validateManifest(manifestData);
 * if (result.valid) {
 *   console.log('Valid manifest:', result.data);
 * } else {
 *   console.error('Validation errors:', result.errors);
 * }
 * ```
 */
export function validateManifest(data: unknown): ValidationResult<ARWManifest> {
  return validate<ARWManifest>(validateManifestSchema, data);
}

// ============================================================================
// .well-known/arw-manifest.json Validation
// ============================================================================

const validateWellKnownManifestSchema = ajv.getSchema('arw-manifest') as ValidateFunction;

/**
 * Validate a .well-known/arw-manifest.json file
 *
 * @param data - The parsed JSON data
 * @returns Validation result with errors or validated data
 *
 * @example
 * ```typescript
 * import { validateWellKnownManifest } from '@arw/schemas/validation';
 *
 * const result = validateWellKnownManifest(jsonData);
 * if (result.valid) {
 *   console.log('Valid well-known manifest:', result.data);
 * } else {
 *   console.error('Validation errors:', result.errors);
 * }
 * ```
 */
export function validateWellKnownManifest(data: unknown): ValidationResult<WellKnownManifest> {
  return validate<WellKnownManifest>(validateWellKnownManifestSchema, data);
}

// ============================================================================
// .well-known/arw-policies.json Validation
// ============================================================================

const validateWellKnownPoliciesSchema = ajv.getSchema('arw-policies') as ValidateFunction;

/**
 * Validate a .well-known/arw-policies.json file
 *
 * @param data - The parsed JSON data
 * @returns Validation result with errors or validated data
 *
 * @example
 * ```typescript
 * import { validateWellKnownPolicies } from '@arw/schemas/validation';
 *
 * const result = validateWellKnownPolicies(jsonData);
 * if (result.valid) {
 *   console.log('Valid policies:', result.data);
 * } else {
 *   console.error('Validation errors:', result.errors);
 * }
 * ```
 */
export function validateWellKnownPolicies(data: unknown): ValidationResult<WellKnownPolicies> {
  return validate<WellKnownPolicies>(validateWellKnownPoliciesSchema, data);
}

// ============================================================================
// .well-known/arw-content-index.json Validation
// ============================================================================

const validateWellKnownContentIndexSchema = ajv.getSchema('arw-content-index') as ValidateFunction;

/**
 * Validate a .well-known/arw-content-index.json file
 *
 * @param data - The parsed JSON data
 * @returns Validation result with errors or validated data
 *
 * @example
 * ```typescript
 * import { validateWellKnownContentIndex } from '@arw/schemas/validation';
 *
 * const result = validateWellKnownContentIndex(jsonData);
 * if (result.valid) {
 *   console.log('Valid content index:', result.data);
 * } else {
 *   console.error('Validation errors:', result.errors);
 * }
 * ```
 */
export function validateWellKnownContentIndex(
  data: unknown
): ValidationResult<WellKnownContentIndex> {
  return validate<WellKnownContentIndex>(validateWellKnownContentIndexSchema, data);
}

// ============================================================================
// Convenience: Validate by Type
// ============================================================================

/**
 * ARW file types
 */
export type ARWFileType =
  | 'manifest'
  | 'well-known-manifest'
  | 'policies'
  | 'content-index'
  | 'toon-manifest';

// ============================================================================
// TOON (Type-safe Object Notation) Validation
// ============================================================================

/**
 * Validate a TOON block element
 */
function validateToonBlock(block: unknown, path = 'block'): ValidationError[] {
  if (!block || typeof block !== 'object') {
    return [{ path, message: 'Block must be an object' }];
  }

  const b = block as Record<string, unknown>;
  const errors: ValidationError[] = [];

  if (!b.type || typeof b.type !== 'string') {
    errors.push({ path: `${path}.type`, message: 'Block type is required and must be a string' });
    return errors;
  }

  switch (b.type) {
    case 'heading':
      if (typeof b.level !== 'number' || b.level < 1 || b.level > 6) {
        errors.push({ path: `${path}.level`, message: 'Heading level must be 1-6' });
      }
      if (typeof b.text !== 'string') {
        errors.push({ path: `${path}.text`, message: 'Heading text must be a string' });
      }
      break;
    case 'paragraph':
      if (!Array.isArray(b.content)) {
        errors.push({ path: `${path}.content`, message: 'Paragraph content must be an array' });
      }
      break;
    case 'list':
      if (typeof b.ordered !== 'boolean') {
        errors.push({ path: `${path}.ordered`, message: 'List ordered must be a boolean' });
      }
      if (!Array.isArray(b.items)) {
        errors.push({ path: `${path}.items`, message: 'List items must be an array' });
      }
      break;
    case 'code':
      if (typeof b.content !== 'string') {
        errors.push({ path: `${path}.content`, message: 'Code content must be a string' });
      }
      break;
    case 'blockquote':
      if (!Array.isArray(b.content)) {
        errors.push({ path: `${path}.content`, message: 'Blockquote content must be an array' });
      } else {
        b.content.forEach((nestedBlock, i) => {
          errors.push(...validateToonBlock(nestedBlock, `${path}.content[${i}]`));
        });
      }
      break;
    case 'table':
      if (!Array.isArray((b as { rows?: unknown }).rows)) {
        errors.push({ path: `${path}.rows`, message: 'Table rows must be an array' });
      }
      break;
    default:
      errors.push({ path: `${path}.type`, message: `Unknown block type: ${b.type}` });
  }

  return errors;
}

/**
 * Validate a TOON Machine View document
 *
 * @param data - The TOON data to validate
 * @returns Validation result with errors or validated data
 *
 * @example
 * ```typescript
 * import { validateToonMachineView } from '@arw/schemas/validation';
 *
 * const result = validateToonMachineView(toonData);
 * if (result.valid) {
 *   console.log('Valid TOON document:', result.data);
 * } else {
 *   console.error('Validation errors:', result.errors);
 * }
 * ```
 */
export function validateToonMachineView(data: unknown): ValidationResult<ToonMachineView> {
  if (data === null || data === undefined) {
    return {
      valid: false,
      errors: [
        {
          path: 'root',
          message: `Data is ${data === null ? 'null' : 'undefined'}. Expected a TOON object.`,
        },
      ],
    };
  }

  if (typeof data !== 'object') {
    return {
      valid: false,
      errors: [{ path: 'root', message: 'TOON data must be an object' }],
    };
  }

  const toon = data as Record<string, unknown>;
  const errors: ValidationError[] = [];

  // Validate required fields
  if (toon.type !== 'MachineView') {
    errors.push({ path: 'type', message: 'Type must be "MachineView"' });
  }

  if (!toon.version || typeof toon.version !== 'string') {
    errors.push({ path: 'version', message: 'Version is required and must be a string' });
  }

  if (!toon.title || typeof toon.title !== 'string') {
    errors.push({ path: 'title', message: 'Title is required and must be a string' });
  }

  if (!Array.isArray(toon.content)) {
    errors.push({ path: 'content', message: 'Content must be an array of blocks' });
  } else {
    toon.content.forEach((block, i) => {
      errors.push(...validateToonBlock(block, `content[${i}]`));
    });
  }

  if (!toon.metadata || typeof toon.metadata !== 'object') {
    errors.push({ path: 'metadata', message: 'Metadata is required and must be an object' });
  }

  // Validate optional chunks
  if (toon.chunks !== undefined) {
    if (!Array.isArray(toon.chunks)) {
      errors.push({ path: 'chunks', message: 'Chunks must be an array' });
    } else {
      toon.chunks.forEach((chunk: unknown, i) => {
        if (!chunk || typeof chunk !== 'object') {
          errors.push({ path: `chunks[${i}]`, message: 'Chunk must be an object' });
          return;
        }
        const c = chunk as Record<string, unknown>;
        if (!c.id || typeof c.id !== 'string') {
          errors.push({ path: `chunks[${i}].id`, message: 'Chunk id is required' });
        }
        if (!Array.isArray(c.blocks)) {
          errors.push({ path: `chunks[${i}].blocks`, message: 'Chunk blocks must be an array' });
        }
      });
    }
  }

  if (errors.length > 0) {
    return { valid: false, errors };
  }

  return { valid: true, data: toon as ToonMachineView };
}

/**
 * Convert a TOON Machine View to an ARW Manifest
 *
 * This helper function transforms TOON-formatted content into the standard
 * ARW manifest format, enabling interoperability between TOON and ARW.
 *
 * @param toon - The TOON Machine View to convert
 * @returns An ARW Manifest
 *
 * @example
 * ```typescript
 * import { toonToManifest } from '@arw/schemas/validation';
 *
 * const manifest = toonToManifest(toonData);
 * console.log('ARW Manifest:', manifest);
 * ```
 */
export function toonToManifest(toon: ToonMachineView): ARWManifest {
  const contentItems: ContentItem[] = [];
  const chunks: Chunk[] = [];

  // Convert TOON chunks to ARW chunks
  if (toon.chunks) {
    toon.chunks.forEach((toonChunk) => {
      chunks.push({
        id: toonChunk.id,
        title: toonChunk.title,
        type: toonChunk.type,
        parent: toonChunk.parent,
        // Content will be generated from blocks (simplified for now)
        content: toonChunk.blocks
          .map((block) => {
            if (block.type === 'heading') {
              return `${'#'.repeat(block.level)} ${block.text}`;
            }
            if (block.type === 'code') {
              return `\`\`\`${block.language || ''}\n${block.content}\n\`\`\``;
            }
            return '';
          })
          .join('\n\n'),
      });
    });
  }

  // Create a content item for the TOON document itself
  contentItems.push({
    url: toon.metadata.title || toon.title,
    title: toon.title,
    description: toon.metadata.description,
    tags: toon.metadata.tags,
    last_modified: toon.metadata.modified,
    machine_view_format: 'toon' as any, // ContentFormat.Toon
  });

  return {
    version: toon.version,
    profile: 'ARW-2', // TOON supports semantic content (ARW-2)
    site: {
      name: toon.title,
      homepage: toon.metadata.title || '',
      description: toon.metadata.description,
    },
    content: contentItems,
    chunks: chunks.length > 0 ? chunks : undefined,
  };
}

/**
 * Validate a TOON manifest and convert to ARW format
 *
 * This function validates TOON-formatted data and converts it to an ARW manifest.
 *
 * @param data - The TOON data (as a string or object)
 * @returns Validation result with ARW manifest or errors
 *
 * @example
 * ```typescript
 * import { validateToonManifest } from '@arw/schemas/validation';
 *
 * const result = validateToonManifest(toonJsonString);
 * if (result.valid) {
 *   console.log('Valid ARW manifest from TOON:', result.data);
 * } else {
 *   console.error('Validation errors:', result.errors);
 * }
 * ```
 */
export function validateToonManifest(data: string | unknown): ValidationResult<ARWManifest> {
  let parsed: unknown;

  if (typeof data === 'string') {
    try {
      parsed = JSON.parse(data);
    } catch (error) {
      return {
        valid: false,
        errors: [
          {
            path: 'root',
            message: `Failed to parse TOON JSON: ${error instanceof Error ? error.message : 'Unknown error'}`,
          },
        ],
      };
    }
  } else {
    parsed = data;
  }

  const toonResult = validateToonMachineView(parsed);

  if (!toonResult.valid) {
    return { valid: false, errors: toonResult.errors };
  }

  const manifest = toonToManifest(toonResult.data!);
  return validateManifest(manifest);
}

/**
 * Validate ARW data by file type
 *
 * @param type - The ARW file type
 * @param data - The data to validate
 * @returns Validation result
 *
 * @example
 * ```typescript
 * import { validateByType } from '@arw/schemas/validation';
 *
 * const result = validateByType('manifest', yamlData);
 * if (result.valid) {
 *   console.log('Valid!');
 * }
 * ```
 */
export function validateByType(type: ARWFileType, data: unknown): ValidationResult {
  switch (type) {
    case 'manifest':
      return validateManifest(data);
    case 'well-known-manifest':
      return validateWellKnownManifest(data);
    case 'policies':
      return validateWellKnownPolicies(data);
    case 'content-index':
      return validateWellKnownContentIndex(data);
    case 'toon-manifest':
      return validateToonManifest(data);
    default: {
      // Exhaustiveness check: TypeScript will error if a new type is added but not handled
      const _exhaustiveCheck: never = type;
      return {
        valid: false,
        errors: [{ path: 'root', message: `Unknown ARW file type: ${_exhaustiveCheck}` }],
      };
    }
  }
}
