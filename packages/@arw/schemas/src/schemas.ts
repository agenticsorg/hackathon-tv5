/**
 * JSON Schema exports for ARW validation
 *
 * These schemas are loaded from the schemas/ directory in the repository.
 */

import arwModelSchema from '../../../schemas/arw_model.json';
import arwManifestSchema from '../../../schemas/arw-manifest.schema.json';
import arwPoliciesSchema from '../../../schemas/arw-policies.schema.json';
import arwContentIndexSchema from '../../../schemas/arw-content-index.schema.json';

/**
 * LinkML-generated JSON schema for llms.txt validation
 */
export const ARWModelSchema = arwModelSchema;

/**
 * JSON schema for .well-known/arw-manifest.json
 */
export const ARWManifestSchema = arwManifestSchema;

/**
 * JSON schema for .well-known/arw-policies.json
 */
export const ARWPoliciesSchema = arwPoliciesSchema;

/**
 * JSON schema for .well-known/arw-content-index.json
 */
export const ARWContentIndexSchema = arwContentIndexSchema;

/**
 * All available schemas
 */
export const schemas = {
  model: ARWModelSchema,
  manifest: ARWManifestSchema,
  policies: ARWPoliciesSchema,
  contentIndex: ARWContentIndexSchema,
} as const;

/**
 * Schema types
 */
export type SchemaType = keyof typeof schemas;
