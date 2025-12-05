/**
 * TypeScript types for Agent-Ready Web (ARW)
 *
 * These types are generated from the LinkML schema at schemas/arw_model.yaml
 * DO NOT EDIT MANUALLY - regenerate with:
 *   make generate-types
 */

// ============================================================================
// Enums
// ============================================================================

/**
 * ARW conformance profile level
 */
export enum ARWProfile {
  /** ARW-1: Basic discovery */
  ARW1 = 'ARW-1',
  /** ARW-2: Semantic content */
  ARW2 = 'ARW-2',
  /** ARW-3: Declarative actions */
  ARW3 = 'ARW-3',
  /** ARW-4: Protocol support */
  ARW4 = 'ARW-4',
}

/**
 * Content priority levels
 */
export enum Priority {
  High = 'high',
  Medium = 'medium',
  Low = 'low',
}

/**
 * Policy permission values
 */
export enum PolicyPermission {
  Allow = 'allow',
  Deny = 'deny',
  Conditional = 'conditional',
}

/**
 * OAuth grant types
 */
export enum GrantType {
  AuthorizationCode = 'authorization_code',
  ClientCredentials = 'client_credentials',
}

/**
 * Content types
 */
export enum ContentType {
  LlmMd = 'llm.md',
  Html = 'html',
  Json = 'json',
}

/**
 * Content format enumeration for machine-readable views
 */
export enum ContentFormat {
  /** Markdown format */
  Markdown = 'markdown',
  /** TOON (Type-safe Object Notation) format */
  Toon = 'toon',
  /** HTML format */
  Html = 'html',
  /** JSON format */
  Json = 'json',
}

// ============================================================================
// Core Types
// ============================================================================

/**
 * Site metadata
 */
export interface Site {
  /** Site name */
  name: string;
  /** Homepage URL */
  homepage: string;
  /** Contact email */
  contact?: string;
  /** Site description */
  description?: string;
  /** Site logo URL */
  logo?: string;
}

/**
 * Content item in the ARW manifest
 */
export interface ContentItem {
  /** Canonical URL of the content */
  url: string;
  /** URL to machine-readable view (.llm.md) */
  machine_view?: string;
  /** Format of the machine-readable view */
  machine_view_format?: ContentFormat;
  /** Content title */
  title?: string;
  /** Content description */
  description?: string;
  /** Content priority */
  priority?: Priority;
  /** Tags/categories */
  tags?: string[];
  /** Last modified timestamp (ISO 8601) */
  last_modified?: string;
  /** Change frequency */
  changefreq?: string;
}

/**
 * Content chunk with semantic boundaries
 */
export interface Chunk {
  /** Unique chunk identifier */
  id: string;
  /** Chunk title/heading */
  title?: string;
  /** Chunk content */
  content?: string;
  /** Chunk type/category */
  type?: string;
  /** Parent chunk ID (for hierarchical chunks) */
  parent?: string;
}

/**
 * HTTP header specification
 */
export interface Header {
  /** Header name */
  name: string;
  /** Header value */
  value: string;
  /** Header description */
  description?: string;
}

/**
 * Policy declarations
 */
export interface Policies {
  /** Training policy */
  training?: PolicyDeclaration;
  /** Inference policy */
  inference?: PolicyDeclaration;
  /** Attribution requirements */
  attribution?: AttributionPolicy;
  /** Rate limits */
  rate_limits?: RateLimits;
  /** Custom policy extensions */
  [key: string]: unknown;
}

/**
 * Individual policy declaration
 */
export interface PolicyDeclaration {
  /** Whether the usage is allowed */
  allowed: PolicyPermission;
  /** Policy description/conditions */
  description?: string;
  /** Policy URL for more details */
  url?: string;
}

/**
 * Attribution policy
 */
export interface AttributionPolicy {
  /** Whether attribution is required */
  required: boolean;
  /** Attribution template */
  template?: string;
  /** Attribution URL */
  url?: string;
}

/**
 * Rate limit configuration
 */
export interface RateLimits {
  /** Requests per second for anonymous users */
  anonymous_rps?: number;
  /** Requests per second for OAuth authenticated users */
  oauth_rps?: number;
  /** Requests per second for API key authenticated users */
  apikey_rps?: number;
}

/**
 * Declarative action definition
 */
export interface Action {
  /** Action identifier */
  id: string;
  /** Action name/title */
  name: string;
  /** Action description */
  description: string;
  /** HTTP method */
  method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  /** Endpoint URL */
  endpoint: string;
  /** Whether OAuth is required */
  requires_oauth: boolean;
  /** Input schema (JSON Schema) */
  input_schema?: Record<string, unknown>;
  /** Output schema (JSON Schema) */
  output_schema?: Record<string, unknown>;
  /** Required scopes */
  scopes?: string[];
}

/**
 * OAuth configuration
 */
export interface OAuthConfig {
  /** Authorization endpoint URL */
  authorization_endpoint: string;
  /** Token endpoint URL */
  token_endpoint: string;
  /** Supported grant types */
  grant_types: GrantType[];
  /** Available scopes */
  scopes?: string[];
  /** Client registration URL */
  registration_url?: string;
}

/**
 * Protocol endpoint definition
 */
export interface Protocol {
  /** Protocol name */
  name: string;
  /** Protocol version */
  version: string;
  /** Endpoint URL */
  endpoint: string;
  /** Protocol description */
  description?: string;
  /** Supported methods */
  methods?: string[];
}

/**
 * Link to related resources
 */
export interface Link {
  /** Link relationship type */
  rel: string;
  /** Link URL */
  href: string;
  /** Link type/format */
  type?: string;
  /** Link title */
  title?: string;
}

// ============================================================================
// Main ARW Manifest
// ============================================================================

/**
 * ARW Manifest (llms.txt) structure
 *
 * This is the main configuration file for Agent-Ready Web implementations.
 */
export interface ARWManifest {
  /** ARW specification version */
  version: string;
  /** ARW conformance profile */
  profile: ARWProfile | string;
  /** Site metadata */
  site: Site;
  /** Content items */
  content?: ContentItem[];
  /** Content chunks */
  chunks?: Chunk[];
  /** HTTP headers */
  headers?: Header[];
  /** Policy declarations */
  policies?: Policies;
  /** Declarative actions */
  actions?: Action[];
  /** OAuth configuration */
  oauth?: OAuthConfig;
  /** Protocol endpoints */
  protocols?: Protocol[];
  /** Related links */
  links?: Link[];
}

// ============================================================================
// .well-known File Types
// ============================================================================

/**
 * .well-known/arw-manifest.json structure
 */
export interface WellKnownManifest {
  /** ARW version */
  arw_version: string;
  /** Site metadata */
  site: Site;
  /** Links to other ARW files */
  links: {
    /** Link to llms.txt */
    guide?: string;
    /** Link to arw-policies.json */
    policies?: string;
    /** Link to arw-content-index.json */
    content_index?: string;
    /** Link to sitemap.xml */
    sitemap?: string;
  };
}

/**
 * .well-known/arw-policies.json structure
 */
export interface WellKnownPolicies {
  /** Training policy */
  training?: PolicyPermission | PolicyDeclaration;
  /** Inference policy */
  inference?: PolicyPermission | PolicyDeclaration;
  /** Attribution requirements */
  attribution?: AttributionPolicy;
  /** Rate limits */
  rate_limits?: RateLimits;
}

/**
 * Content index item
 */
export interface ContentIndexItem {
  /** Item identifier */
  id: string;
  /** Content type */
  type: ContentType | string;
  /** Item URL */
  url: string;
  /** Content hash (e.g., sha256-...) */
  hash?: string;
  /** Last modified timestamp (ISO 8601) */
  last_modified?: string;
  /** Tags/categories */
  tags?: string[];
  /** Item title */
  title?: string;
  /** Item description */
  description?: string;
}

/**
 * .well-known/arw-content-index.json structure
 */
export interface WellKnownContentIndex {
  /** ARW version */
  version: string;
  /** Content items */
  items: ContentIndexItem[];
  /** Pagination: next page URL */
  next?: string;
  /** Pagination: previous page URL */
  prev?: string;
  /** Total item count */
  total?: number;
}

// ============================================================================
// TOON (Type-safe Object Notation) Types
// ============================================================================

/**
 * TOON inline element types
 */
export type ToonInline =
  | { type: 'text'; content: string }
  | { type: 'emphasis'; content: string }
  | { type: 'strong'; content: string }
  | { type: 'code'; content: string }
  | { type: 'link'; href: string; text: string };

/**
 * TOON block element types
 */
export type ToonBlock =
  | ToonHeading
  | ToonParagraph
  | ToonList
  | ToonCode
  | ToonBlockquote
  | ToonTable;

/**
 * TOON heading block
 */
export interface ToonHeading {
  /** Block type identifier */
  type: 'heading';
  /** Heading level (1-6) */
  level: 1 | 2 | 3 | 4 | 5 | 6;
  /** Heading text content */
  text: string;
}

/**
 * TOON paragraph block
 */
export interface ToonParagraph {
  /** Block type identifier */
  type: 'paragraph';
  /** Paragraph content as inline elements */
  content: ToonInline[];
}

/**
 * TOON list block
 */
export interface ToonList {
  /** Block type identifier */
  type: 'list';
  /** List style (ordered or unordered) */
  ordered: boolean;
  /** List items */
  items: string[];
}

/**
 * TOON code block
 */
export interface ToonCode {
  /** Block type identifier */
  type: 'code';
  /** Code content */
  content: string;
  /** Programming language hint */
  language?: string;
}

/**
 * TOON blockquote block
 */
export interface ToonBlockquote {
  /** Block type identifier */
  type: 'blockquote';
  /** Blockquote content as blocks */
  content: ToonBlock[];
}

/**
 * TOON table row
 */
export interface ToonTableRow {
  /** Row cells */
  cells: string[];
}

/**
 * TOON table block
 */
export interface ToonTable {
  /** Block type identifier */
  type: 'table';
  /** Table header row */
  header?: ToonTableRow;
  /** Table body rows */
  rows: ToonTableRow[];
}

/**
 * TOON content chunk with semantic boundaries
 */
export interface ToonChunk {
  /** Unique chunk identifier */
  id: string;
  /** Chunk title/heading */
  title?: string;
  /** Chunk content as blocks */
  blocks: ToonBlock[];
  /** Parent chunk ID (for hierarchical chunks) */
  parent?: string;
  /** Chunk type/category */
  type?: string;
}

/**
 * TOON metadata for machine-readable views
 */
export interface ToonMetadata {
  /** Content title */
  title?: string;
  /** Content description */
  description?: string;
  /** Content author(s) */
  author?: string | string[];
  /** Creation date (ISO 8601) */
  created?: string;
  /** Last modified date (ISO 8601) */
  modified?: string;
  /** Tags/categories */
  tags?: string[];
  /** Custom metadata */
  [key: string]: unknown;
}

/**
 * TOON Machine View structure
 *
 * This is the root structure for TOON-formatted machine-readable content.
 * It provides a type-safe, structured representation of content that can be
 * easily parsed and validated by AI agents.
 */
export interface ToonMachineView {
  /** Document type identifier */
  type: 'MachineView';
  /** TOON format version */
  version: string;
  /** Document title */
  title: string;
  /** Document content as blocks */
  content: ToonBlock[];
  /** Optional content chunks for semantic boundaries */
  chunks?: ToonChunk[];
  /** Document metadata */
  metadata: ToonMetadata;
}
