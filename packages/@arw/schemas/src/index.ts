/**
 * @arw/schemas
 *
 * TypeScript types and JSON schemas for Agent-Ready Web (ARW) with TOON support
 *
 * This package provides:
 * - TypeScript types generated from LinkML schemas
 * - TOON (Type-safe Object Notation) types for machine-readable content
 * - JSON schemas for validation
 * - Validation utilities using AJV
 * - TOON to ARW conversion utilities
 *
 * @example Basic ARW Manifest
 * ```typescript
 * import { ARWManifest, validateManifest } from '@arw/schemas';
 *
 * const manifest: ARWManifest = {
 *   version: '1.0',
 *   profile: 'ARW-2',
 *   site: {
 *     name: 'Example',
 *     homepage: 'https://example.com'
 *   }
 * };
 *
 * const result = validateManifest(manifest);
 * if (result.valid) {
 *   console.log('Valid manifest');
 * }
 * ```
 *
 * @example TOON Machine View
 * ```typescript
 * import { ToonMachineView, validateToonMachineView, toonToManifest } from '@arw/schemas';
 *
 * const toon: ToonMachineView = {
 *   type: 'MachineView',
 *   version: '1.0',
 *   title: 'API Documentation',
 *   content: [
 *     { type: 'heading', level: 1, text: 'Welcome' },
 *     { type: 'paragraph', content: [{ type: 'text', content: 'Hello world' }] }
 *   ],
 *   metadata: {
 *     description: 'API docs',
 *     tags: ['api', 'docs']
 *   }
 * };
 *
 * const result = validateToonMachineView(toon);
 * if (result.valid) {
 *   const manifest = toonToManifest(result.data);
 *   console.log('Converted to ARW manifest:', manifest);
 * }
 * ```
 */

// Re-export types
export * from './types.js';

// Re-export schemas
export * from './schemas.js';

// Re-export validation utilities
export * from './validation.js';
