# SPARC Refinement - Code Quality Standards

**Document Version:** 1.0.0
**Last Updated:** 2025-12-06
**Status:** Planning Phase
**Related Documents:** SPARC_REFINEMENT_PART_1.md (TDD Strategy), SPARC_REFINEMENT_PART_2.md (Acceptance Criteria)

---

## Table of Contents

1. [Code Style Standards](#1-code-style-standards)
2. [Documentation Standards](#2-documentation-standards)
3. [Code Review Checklist](#3-code-review-checklist)
4. [Quality Gates](#4-quality-gates)
5. [Technical Debt Management](#5-technical-debt-management)
6. [Performance Standards](#6-performance-standards)
7. [Security Standards](#7-security-standards)
8. [Accessibility Standards](#8-accessibility-standards)
9. [Tool Configuration](#9-tool-configuration)
10. [Enforcement Strategy](#10-enforcement-strategy)

---

## 1. Code Style Standards

### 1.1 Rust Standards

#### Formatting Configuration (rustfmt.toml)

```toml
# /rustfmt.toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
fn_call_width = 60
struct_lit_width = 18
struct_variant_width = 35
array_width = 60
chain_width = 60
single_line_if_else_max_width = 50
wrap_comments = true
format_code_in_doc_comments = true
normalize_comments = true
normalize_doc_attributes = true
format_strings = false
format_macro_matchers = true
format_macro_bodies = true
hex_literal_case = "Preserve"
empty_item_single_line = true
struct_lit_single_line = true
fn_single_line = false
where_single_line = false
imports_indent = "Block"
imports_granularity = "Crate"
imports_layout = "Mixed"
group_imports = "StdExternalCrate"
reorder_imports = true
reorder_modules = true
reorder_impl_items = false
type_punctuation_density = "Wide"
space_before_colon = false
space_after_colon = true
spaces_around_ranges = false
binop_separator = "Front"
remove_nested_parens = true
combine_control_expr = true
overflow_delimited_expr = true
struct_field_align_threshold = 0
enum_discrim_align_threshold = 0
match_arm_blocks = true
match_arm_leading_pipes = "Never"
force_multiline_blocks = false
fn_args_layout = "Tall"
brace_style = "SameLineWhere"
control_brace_style = "AlwaysSameLine"
trailing_semicolon = true
trailing_comma = "Vertical"
match_block_trailing_comma = false
blank_lines_upper_bound = 1
blank_lines_lower_bound = 0
edition = "2021"
version = "Two"
merge_derives = true
use_try_shorthand = true
use_field_init_shorthand = true
force_explicit_abi = true
condense_wildcard_suffixes = true
color = "Auto"
required_version = "1.70.0"
unstable_features = false
disable_all_formatting = false
skip_children = false
hide_parse_errors = false
error_on_line_overflow = false
error_on_unformatted = false
report_todo = "Never"
report_fixme = "Never"
ignore = []
emit_mode = "Files"
make_backup = false
```

#### Clippy Configuration (.clippy.toml)

```toml
# /.clippy.toml
# Deny level lints
warn-on-all-wildcard-imports = true
disallowed-methods = [
    "std::env::set_var",  # Prefer config files
    "std::process::exit",  # Prefer Result returns
]

# Performance
cognitive-complexity-threshold = 15
too-many-arguments-threshold = 7
type-complexity-threshold = 250

# Style
enum-variant-name-threshold = 3
single-char-binding-names-threshold = 4
too-large-for-stack = 200

# Documentation
missing-docs-in-crate-items = true
```

#### Rust Naming Conventions

```rust
// Modules: snake_case
mod media_processor;
mod streaming_service;

// Types: PascalCase
struct MediaFile;
enum StreamingProtocol;
trait MediaProcessor;

// Functions/Methods: snake_case
fn process_media_file();
fn create_stream();

// Constants: SCREAMING_SNAKE_CASE
const MAX_FILE_SIZE: usize = 1024 * 1024 * 100; // 100MB
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// Statics: SCREAMING_SNAKE_CASE
static GLOBAL_CACHE: LazyLock<Cache> = LazyLock::new(|| Cache::new());

// Type Parameters: Single uppercase letter or PascalCase
fn process<T: MediaType>(item: T) -> Result<Output>;
fn transform<Input, Output, Error>(input: Input) -> Result<Output, Error>;

// Lifetimes: Short lowercase
fn process_stream<'a>(stream: &'a Stream) -> &'a Output;

// Features: kebab-case
#[cfg(feature = "advanced-codecs")]

// Crate names: kebab-case
// media-gateway-core
// streaming-protocol-handler
```

#### Rust File Organization

```
src/
├── lib.rs or main.rs              # Crate root (max 200 lines)
├── config.rs                       # Configuration (max 300 lines)
├── error.rs                        # Error types (max 200 lines)
├── types/                          # Type definitions
│   ├── mod.rs                      # Module exports
│   ├── media.rs                    # Media-related types
│   └── streaming.rs                # Streaming types
├── services/                       # Business logic
│   ├── mod.rs
│   ├── media_processor.rs          # Max 500 lines
│   └── stream_handler.rs           # Max 500 lines
├── handlers/                       # Request handlers
│   ├── mod.rs
│   ├── http.rs                     # Max 400 lines
│   └── websocket.rs                # Max 400 lines
├── utils/                          # Utilities
│   ├── mod.rs
│   └── validation.rs               # Max 300 lines
└── tests/                          # Integration tests
    ├── integration_test.rs
    └── common/
        └── mod.rs                  # Test utilities
```

#### Rust Code Complexity Limits

```rust
// Maximum file length: 500 lines (excluding comments/blank lines)
// Maximum function length: 50 lines
// Maximum function parameters: 7
// Maximum cognitive complexity: 15
// Maximum cyclomatic complexity: 10

// ❌ TOO COMPLEX (Cognitive Complexity: 22)
fn bad_example(data: Vec<Data>) -> Result<Output> {
    if data.is_empty() {
        return Err(Error::Empty);
    }
    let mut result = Vec::new();
    for item in data {
        if item.is_valid() {
            if item.needs_processing() {
                match item.process() {
                    Ok(processed) => {
                        if processed.is_complete() {
                            result.push(processed);
                        } else {
                            for retry in 0..3 {
                                if let Ok(r) = item.retry() {
                                    result.push(r);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => log::error!("Error: {}", e),
                }
            }
        }
    }
    Ok(Output::new(result))
}

// ✅ REFACTORED (Multiple focused functions)
fn good_example(data: Vec<Data>) -> Result<Output> {
    validate_data(&data)?;
    let processed = process_items(data)?;
    Ok(Output::new(processed))
}

fn validate_data(data: &[Data]) -> Result<()> {
    if data.is_empty() {
        return Err(Error::Empty);
    }
    Ok(())
}

fn process_items(data: Vec<Data>) -> Result<Vec<ProcessedItem>> {
    data.into_iter()
        .filter(|item| item.is_valid())
        .map(process_single_item)
        .collect()
}

fn process_single_item(item: Data) -> Result<ProcessedItem> {
    if !item.needs_processing() {
        return item.as_processed();
    }

    item.process()
        .or_else(|_| retry_processing(&item))
}

fn retry_processing(item: &Data) -> Result<ProcessedItem> {
    (0..3)
        .find_map(|_| item.retry().ok())
        .ok_or(Error::RetryFailed)
}
```

### 1.2 TypeScript Standards

#### ESLint Configuration (.eslintrc.json)

```json
{
  "root": true,
  "parser": "@typescript-eslint/parser",
  "parserOptions": {
    "ecmaVersion": 2022,
    "sourceType": "module",
    "project": "./tsconfig.json"
  },
  "plugins": [
    "@typescript-eslint",
    "import",
    "promise",
    "security",
    "jsdoc"
  ],
  "extends": [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:@typescript-eslint/recommended-requiring-type-checking",
    "plugin:import/recommended",
    "plugin:import/typescript",
    "plugin:promise/recommended",
    "plugin:security/recommended",
    "prettier"
  ],
  "rules": {
    "@typescript-eslint/explicit-function-return-type": "error",
    "@typescript-eslint/no-explicit-any": "error",
    "@typescript-eslint/no-unused-vars": ["error", {
      "argsIgnorePattern": "^_",
      "varsIgnorePattern": "^_"
    }],
    "@typescript-eslint/no-floating-promises": "error",
    "@typescript-eslint/await-thenable": "error",
    "@typescript-eslint/no-misused-promises": "error",
    "@typescript-eslint/strict-boolean-expressions": ["error", {
      "allowString": false,
      "allowNumber": false,
      "allowNullableObject": false
    }],
    "@typescript-eslint/naming-convention": [
      "error",
      {
        "selector": "default",
        "format": ["camelCase"]
      },
      {
        "selector": "variable",
        "format": ["camelCase", "UPPER_CASE"]
      },
      {
        "selector": "parameter",
        "format": ["camelCase"],
        "leadingUnderscore": "allow"
      },
      {
        "selector": "memberLike",
        "modifiers": ["private"],
        "format": ["camelCase"],
        "leadingUnderscore": "require"
      },
      {
        "selector": "typeLike",
        "format": ["PascalCase"]
      },
      {
        "selector": "enumMember",
        "format": ["PascalCase"]
      },
      {
        "selector": "interface",
        "format": ["PascalCase"],
        "prefix": ["I"]
      }
    ],
    "complexity": ["error", 15],
    "max-depth": ["error", 4],
    "max-lines": ["error", {
      "max": 500,
      "skipBlankLines": true,
      "skipComments": true
    }],
    "max-lines-per-function": ["error", {
      "max": 50,
      "skipBlankLines": true,
      "skipComments": true
    }],
    "max-params": ["error", 7],
    "max-statements": ["error", 20],
    "import/order": ["error", {
      "groups": [
        "builtin",
        "external",
        "internal",
        ["parent", "sibling"],
        "index"
      ],
      "newlines-between": "always",
      "alphabetize": {
        "order": "asc",
        "caseInsensitive": true
      }
    }],
    "import/no-cycle": "error",
    "import/no-unused-modules": "error",
    "promise/always-return": "error",
    "promise/catch-or-return": "error",
    "security/detect-object-injection": "warn",
    "security/detect-non-literal-regexp": "warn",
    "jsdoc/require-description": "error",
    "jsdoc/require-param": "error",
    "jsdoc/require-returns": "error"
  },
  "overrides": [
    {
      "files": ["*.test.ts", "*.spec.ts"],
      "rules": {
        "@typescript-eslint/no-explicit-any": "off",
        "max-lines-per-function": "off"
      }
    }
  ]
}
```

#### Prettier Configuration (.prettierrc.json)

```json
{
  "semi": true,
  "trailingComma": "all",
  "singleQuote": true,
  "printWidth": 100,
  "tabWidth": 2,
  "useTabs": false,
  "arrowParens": "always",
  "endOfLine": "lf",
  "bracketSpacing": true,
  "bracketSameLine": false,
  "quoteProps": "as-needed",
  "jsxSingleQuote": false,
  "proseWrap": "preserve",
  "htmlWhitespaceSensitivity": "css",
  "embeddedLanguageFormatting": "auto"
}
```

#### TypeScript Naming Conventions

```typescript
// Interfaces: PascalCase with 'I' prefix
interface IMediaFile {
  id: string;
  name: string;
}

// Types: PascalCase
type StreamingProtocol = 'HLS' | 'DASH' | 'RTMP';
type MediaProcessor = (file: IMediaFile) => Promise<ProcessedMedia>;

// Classes: PascalCase
class MediaService {
  private _cache: Map<string, IMediaFile>; // Private with underscore

  public async processFile(file: IMediaFile): Promise<void> {
    // Method: camelCase
  }
}

// Enums: PascalCase, Members: PascalCase
enum StreamState {
  Idle = 'IDLE',
  Streaming = 'STREAMING',
  Paused = 'PAUSED',
  Error = 'ERROR',
}

// Constants: UPPER_SNAKE_CASE
const MAX_FILE_SIZE = 100 * 1024 * 1024; // 100MB
const DEFAULT_TIMEOUT_MS = 30000;

// Variables/Functions: camelCase
const mediaFiles: IMediaFile[] = [];
function processMediaFile(file: IMediaFile): Promise<void> {
  // Implementation
}

// Generics: Single uppercase letter or PascalCase
function process<T extends IMediaFile>(item: T): Promise<ProcessedItem<T>> {
  // Implementation
}

// Modules/Files: kebab-case
// media-processor.ts
// streaming-service.ts
```

#### TypeScript File Organization

```
src/
├── index.ts                        # Entry point (max 100 lines)
├── config/                         # Configuration
│   ├── index.ts
│   ├── database.config.ts
│   └── server.config.ts
├── types/                          # Type definitions
│   ├── index.ts                    # Re-exports
│   ├── media.types.ts
│   └── streaming.types.ts
├── interfaces/                     # Interface definitions
│   ├── index.ts
│   ├── media-processor.interface.ts
│   └── stream-handler.interface.ts
├── services/                       # Business logic
│   ├── media-processor.service.ts  # Max 500 lines
│   └── streaming.service.ts        # Max 500 lines
├── controllers/                    # Request handlers
│   ├── media.controller.ts         # Max 400 lines
│   └── stream.controller.ts        # Max 400 lines
├── middleware/                     # Express middleware
│   ├── auth.middleware.ts
│   └── validation.middleware.ts
├── utils/                          # Utilities
│   ├── index.ts
│   ├── validation.util.ts
│   └── logger.util.ts
├── errors/                         # Custom errors
│   ├── index.ts
│   ├── app.error.ts
│   └── validation.error.ts
└── __tests__/                      # Tests
    ├── unit/
    ├── integration/
    └── fixtures/
```

#### TypeScript Code Complexity Limits

```typescript
// Maximum file length: 500 lines (excluding comments/blank lines)
// Maximum function length: 50 lines
// Maximum function parameters: 7
// Maximum cyclomatic complexity: 15
// Maximum nesting depth: 4

// ❌ TOO COMPLEX
async function badExample(data: IData[]): Promise<IOutput> {
  if (data.length === 0) {
    throw new Error('Empty data');
  }
  const result: IProcessedItem[] = [];
  for (const item of data) {
    if (item.isValid) {
      if (item.needsProcessing) {
        try {
          const processed = await item.process();
          if (processed.isComplete) {
            result.push(processed);
          } else {
            for (let i = 0; i < 3; i++) {
              try {
                const retried = await item.retry();
                result.push(retried);
                break;
              } catch (error) {
                console.error(`Retry ${i} failed:`, error);
              }
            }
          }
        } catch (error) {
          console.error('Processing failed:', error);
        }
      }
    }
  }
  return { items: result };
}

// ✅ REFACTORED
async function goodExample(data: IData[]): Promise<IOutput> {
  validateData(data);
  const processed = await processItems(data);
  return { items: processed };
}

function validateData(data: IData[]): void {
  if (data.length === 0) {
    throw new Error('Empty data');
  }
}

async function processItems(data: IData[]): Promise<IProcessedItem[]> {
  const validItems = data.filter((item) => item.isValid);
  const promises = validItems.map(processSingleItem);
  return Promise.all(promises);
}

async function processSingleItem(item: IData): Promise<IProcessedItem> {
  if (!item.needsProcessing) {
    return item.asProcessed();
  }

  try {
    return await processWithRetry(item);
  } catch (error) {
    throw new ProcessingError(`Failed to process item: ${item.id}`, error);
  }
}

async function processWithRetry(item: IData): Promise<IProcessedItem> {
  const maxRetries = 3;

  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      return await item.process();
    } catch (error) {
      if (attempt === maxRetries - 1) {
        throw error;
      }
      await delay(Math.pow(2, attempt) * 1000); // Exponential backoff
    }
  }

  throw new Error('Unexpected: retry loop exited without return or throw');
}
```

### 1.3 General File Organization Principles

1. **Single Responsibility**: Each file has one clear purpose
2. **Dependency Direction**: Dependencies flow inward (no circular dependencies)
3. **Colocation**: Related files are grouped together
4. **Separation of Concerns**: Business logic, presentation, data access separated
5. **Test Proximity**: Tests mirror the source structure

---

## 2. Documentation Standards

### 2.1 Rust Documentation (rustdoc)

#### Public API Documentation

```rust
//! # Media Gateway Core
//!
//! This crate provides core functionality for media processing and streaming.
//!
//! ## Features
//!
//! - **media-processing**: Advanced media file processing capabilities
//! - **streaming**: Real-time streaming protocol support
//! - **caching**: Distributed caching for media files
//!
//! ## Quick Start
//!
//! ```rust
//! use media_gateway_core::{MediaProcessor, ProcessorConfig};
//!
//! let config = ProcessorConfig::default();
//! let processor = MediaProcessor::new(config)?;
//! let result = processor.process_file("input.mp4").await?;
//! ```
//!
//! ## Architecture
//!
//! See [Architecture Decision Records](../docs/adr/) for design decisions.

/// Processes media files with advanced codec support.
///
/// The `MediaProcessor` handles transcoding, thumbnail generation, and metadata
/// extraction for various media formats.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use media_gateway_core::{MediaProcessor, ProcessorConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = ProcessorConfig::builder()
///     .max_file_size(100 * 1024 * 1024) // 100MB
///     .timeout(Duration::from_secs(300))
///     .build();
///
/// let processor = MediaProcessor::new(config)?;
/// let result = processor.process_file("video.mp4").await?;
///
/// println!("Processed: {} bytes", result.size);
/// # Ok(())
/// # }
/// ```
///
/// Advanced usage with custom codec settings:
///
/// ```rust
/// use media_gateway_core::{MediaProcessor, CodecSettings};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let codec = CodecSettings::h264()
///     .bitrate(5_000_000)
///     .preset("medium");
///
/// let result = processor.process_with_codec("input.mp4", codec).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`ProcessingError`] if:
/// - File format is unsupported
/// - File size exceeds configured maximum
/// - Processing timeout is exceeded
/// - Codec errors occur during transcoding
///
/// # Panics
///
/// This function does not panic under normal circumstances. If a panic occurs,
/// it indicates a critical bug that should be reported.
///
/// # Safety
///
/// This function is safe to call from multiple threads. Internal state is
/// protected by appropriate synchronization primitives.
///
/// # Performance
///
/// Processing time scales linearly with file size. For optimal performance:
/// - Use hardware acceleration when available
/// - Process files in parallel for batch operations
/// - Configure appropriate buffer sizes
///
/// Typical processing time: ~50ms per MB on modern hardware.
pub struct MediaProcessor {
    /// Configuration for the processor
    config: ProcessorConfig,
    /// Internal codec registry
    codecs: CodecRegistry,
}

impl MediaProcessor {
    /// Creates a new `MediaProcessor` with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings for the processor
    ///
    /// # Returns
    ///
    /// * `Ok(MediaProcessor)` - Successfully created processor
    /// * `Err(InitError)` - If initialization fails (e.g., missing codec libraries)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use media_gateway_core::{MediaProcessor, ProcessorConfig};
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ProcessorConfig::default();
    /// let processor = MediaProcessor::new(config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(config: ProcessorConfig) -> Result<Self, InitError> {
        // Implementation
    }
}
```

#### Internal Documentation

```rust
// Internal module documentation
mod internal {
    //! Internal utilities for media processing.
    //!
    //! This module contains implementation details and should not be used
    //! directly by external consumers.

    // Complex algorithm explanation
    /// Processes the media stream using a two-pass encoding strategy.
    ///
    /// Algorithm:
    /// 1. First pass: Analyze bitrate distribution
    /// 2. Calculate optimal encoding parameters
    /// 3. Second pass: Encode with optimized settings
    ///
    /// Time complexity: O(n) where n is file size
    /// Space complexity: O(1) - streaming algorithm
    fn two_pass_encode(stream: &mut Stream) -> Result<EncodedData> {
        // Step 1: First pass analysis
        let stats = analyze_stream(stream)?;

        // Step 2: Calculate parameters
        let params = calculate_encoding_params(&stats);

        // Step 3: Second pass encoding
        encode_with_params(stream, params)
    }
}
```

### 2.2 TypeScript Documentation (TSDoc)

```typescript
/**
 * Media Gateway Service
 *
 * Provides comprehensive media processing and streaming capabilities.
 *
 * @packageDocumentation
 */

/**
 * Processes media files with advanced codec support.
 *
 * The MediaProcessor handles transcoding, thumbnail generation, and metadata
 * extraction for various media formats.
 *
 * @example
 * Basic usage:
 * ```typescript
 * const config: IProcessorConfig = {
 *   maxFileSize: 100 * 1024 * 1024, // 100MB
 *   timeout: 300000, // 5 minutes
 * };
 *
 * const processor = new MediaProcessor(config);
 * const result = await processor.processFile('video.mp4');
 * console.log(`Processed: ${result.size} bytes`);
 * ```
 *
 * @example
 * Advanced usage with custom codec:
 * ```typescript
 * const codec: ICodecSettings = {
 *   type: 'h264',
 *   bitrate: 5000000,
 *   preset: 'medium',
 * };
 *
 * const result = await processor.processWithCodec('input.mp4', codec);
 * ```
 *
 * @public
 */
export class MediaProcessor {
  private readonly _config: IProcessorConfig;
  private readonly _codecs: CodecRegistry;

  /**
   * Creates a new MediaProcessor instance.
   *
   * @param config - Configuration settings for the processor
   * @throws {InitializationError} If codec libraries are missing or invalid
   *
   * @example
   * ```typescript
   * const config: IProcessorConfig = {
   *   maxFileSize: 100 * 1024 * 1024,
   *   timeout: 300000,
   * };
   * const processor = new MediaProcessor(config);
   * ```
   */
  constructor(config: IProcessorConfig) {
    // Implementation
  }

  /**
   * Processes a media file.
   *
   * @param filePath - Path to the media file to process
   * @returns Promise resolving to processing result
   * @throws {ProcessingError} If processing fails due to:
   *   - Unsupported file format
   *   - File size exceeds maximum
   *   - Timeout exceeded
   *   - Codec errors
   *
   * @remarks
   * Processing time scales linearly with file size. Typical performance:
   * ~50ms per MB on modern hardware.
   *
   * For optimal performance:
   * - Use hardware acceleration when available
   * - Process files in parallel for batch operations
   * - Configure appropriate buffer sizes
   *
   * @example
   * ```typescript
   * try {
   *   const result = await processor.processFile('video.mp4');
   *   console.log(`Success: ${result.size} bytes`);
   * } catch (error) {
   *   if (error instanceof ProcessingError) {
   *     console.error(`Failed: ${error.message}`);
   *   }
   * }
   * ```
   *
   * @see {@link IProcessingResult} for return value structure
   * @see {@link ProcessingError} for error details
   */
  public async processFile(filePath: string): Promise<IProcessingResult> {
    // Implementation
  }
}

/**
 * Processing result metadata.
 *
 * @public
 */
export interface IProcessingResult {
  /**
   * Size of processed file in bytes
   */
  size: number;

  /**
   * Duration in milliseconds
   */
  duration: number;

  /**
   * Processing timestamp (ISO 8601)
   */
  timestamp: string;

  /**
   * Codec used for processing
   */
  codec: string;

  /**
   * Optional metadata extracted from file
   */
  metadata?: IMediaMetadata;
}
```

### 2.3 README Requirements Per Service

Each service MUST include a README.md with the following sections:

```markdown
# Service Name

Brief description (1-2 sentences).

## Features

- Feature 1
- Feature 2
- Feature 3

## Prerequisites

- Node.js 18+ / Rust 1.70+
- PostgreSQL 15+
- Redis 7+
- Other dependencies

## Installation

```bash
# Installation commands
npm install
# or
cargo build
```

## Configuration

Environment variables:

```env
DATABASE_URL=postgresql://localhost/media_gateway
REDIS_URL=redis://localhost:6379
MAX_FILE_SIZE=104857600  # 100MB
```

Configuration file: `config/default.json`

## Usage

Basic usage example:

```typescript
// Code example
```

## API Reference

### Endpoints

#### `POST /api/media/process`

Processes a media file.

**Request:**
```json
{
  "file": "base64-encoded-data",
  "format": "mp4"
}
```

**Response:**
```json
{
  "id": "uuid",
  "status": "processing",
  "url": "/api/media/status/uuid"
}
```

## Development

```bash
# Run tests
npm test

# Run linting
npm run lint

# Start dev server
npm run dev
```

## Testing

```bash
# Unit tests
npm run test:unit

# Integration tests
npm run test:integration

# E2E tests
npm run test:e2e

# Coverage
npm run test:coverage
```

## Architecture

High-level architecture overview. See [ADR-001](../docs/adr/001-architecture.md) for details.

## Performance

Expected performance characteristics:
- Throughput: 100 requests/second
- Latency: p50 < 100ms, p99 < 500ms
- Memory: < 512MB under normal load

## Security

Security considerations and best practices.

## Troubleshooting

Common issues and solutions:

### Issue: Connection timeout

**Cause:** Database connection pool exhausted

**Solution:** Increase `DB_POOL_SIZE` environment variable

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md)

## License

MIT License - see [LICENSE](../LICENSE)

## Related Documentation

- [Architecture Decisions](../docs/adr/)
- [API Documentation](../docs/api/)
- [Deployment Guide](../docs/deployment/)
```

### 2.4 Architecture Decision Records (ADRs)

Template for ADRs (`docs/adr/NNN-title.md`):

```markdown
# ADR-NNN: Title

**Status:** Proposed | Accepted | Deprecated | Superseded by ADR-XXX
**Date:** YYYY-MM-DD
**Deciders:** @user1, @user2
**Technical Story:** Link to issue/epic

## Context

What is the issue we're seeing that motivates this decision?

## Decision

What is the change we're proposing and/or doing?

## Consequences

### Positive

- Benefit 1
- Benefit 2

### Negative

- Tradeoff 1
- Tradeoff 2

### Neutral

- Other consideration 1

## Alternatives Considered

### Alternative 1: Name

**Description:** What was considered

**Pros:**
- Pro 1

**Cons:**
- Con 1

**Decision:** Why rejected

## Implementation Plan

1. Step 1
2. Step 2
3. Step 3

## Metrics for Success

- Metric 1: Target value
- Metric 2: Target value

## References

- [RFC Title](link)
- [Documentation](link)
- [Research Paper](link)
```

### 2.5 Inline Comment Guidelines

```rust
// ✅ GOOD: Explain WHY, not WHAT
// Use exponential backoff to avoid overwhelming the service during recovery
let delay = Duration::from_secs(2_u64.pow(attempt));

// ❌ BAD: Restates code
// Set delay to 2 to the power of attempt
let delay = Duration::from_secs(2_u64.pow(attempt));

// ✅ GOOD: Document assumptions
// SAFETY: Buffer is guaranteed to be initialized by the constructor
unsafe { &*self.buffer.get() }

// ✅ GOOD: Explain complex algorithms
/// Implements the Levenshtein distance algorithm using dynamic programming.
/// Time: O(m*n), Space: O(min(m,n)) via space optimization.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    // Implementation
}

// ✅ GOOD: Document workarounds
// WORKAROUND: Library X has a bug with concurrent connections.
// See: https://github.com/library-x/issues/123
// Remove this once version 2.0 is released.
let pool = Pool::new().max_connections(1);

// ✅ GOOD: Highlight important constraints
// IMPORTANT: This function must complete within 100ms to meet SLA requirements
async fn critical_path() -> Result<Response> {
    // Implementation
}
```

```typescript
// ✅ GOOD: Explain business logic
// Apply discount only for premium users who have been active in last 30 days
if (user.isPremium && user.lastActiveAt > thirtyDaysAgo) {
  applyDiscount();
}

// ❌ BAD: Obvious comment
// Check if user is premium
if (user.isPremium) {
  // ...
}

// ✅ GOOD: Document magic numbers
const CACHE_TTL_MS = 5 * 60 * 1000; // 5 minutes - balances freshness vs API calls

// ✅ GOOD: Explain non-obvious type assertions
// Type assertion safe here: API contract guarantees 'items' is always present
const items = (response.data as { items: Item[] }).items;

// ✅ GOOD: Document performance considerations
// Using Set for O(1) lookup instead of Array.includes() which is O(n)
const uniqueIds = new Set(items.map((item) => item.id));
```

---

## 3. Code Review Checklist

### 3.1 Automated Checks (Pre-Review)

Before manual review, verify automated checks pass:

```bash
# Rust
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo audit
cargo outdated

# TypeScript
npm run format:check
npm run lint
npm run typecheck
npm run test
npm run audit
```

### 3.2 Functionality Review

**Correctness**
- [ ] Code implements requirements correctly
- [ ] Edge cases are handled (empty arrays, null values, boundary conditions)
- [ ] Error conditions are handled appropriately
- [ ] Business logic is correct
- [ ] State transitions are valid
- [ ] Concurrency issues are addressed (race conditions, deadlocks)

**API Design**
- [ ] Public API is intuitive and well-documented
- [ ] Function signatures are clear and type-safe
- [ ] Breaking changes are documented
- [ ] Backward compatibility is maintained (when required)

### 3.3 Testing Review

**Coverage**
- [ ] Unit tests cover happy paths
- [ ] Unit tests cover error cases
- [ ] Unit tests cover edge cases
- [ ] Integration tests verify component interactions
- [ ] E2E tests verify critical user flows
- [ ] Overall coverage >= 80%
- [ ] Critical paths have >= 95% coverage

**Quality**
- [ ] Tests are deterministic (no flaky tests)
- [ ] Tests are fast (unit tests < 100ms each)
- [ ] Tests are isolated (no shared state)
- [ ] Test names clearly describe what they test
- [ ] Tests follow AAA pattern (Arrange, Act, Assert)
- [ ] Mocks/stubs are used appropriately
- [ ] Tests use actual database for integration tests (not mocks)

### 3.4 Performance Review

**Efficiency**
- [ ] No obvious performance bottlenecks
- [ ] Algorithms have appropriate time complexity
- [ ] Database queries are optimized (proper indexes, no N+1)
- [ ] Caching is used appropriately
- [ ] Resource usage is reasonable (memory, CPU, connections)

**Scalability**
- [ ] Code scales with data volume
- [ ] No hard-coded limits that prevent scaling
- [ ] Connection pools are sized appropriately
- [ ] Async operations are used for I/O-bound work

**Benchmarks**
- [ ] Performance-critical code has benchmarks
- [ ] No performance regression vs baseline
- [ ] Meets performance requirements (see section 6)

### 3.5 Security Review

**Input Validation**
- [ ] All user input is validated
- [ ] Input size limits are enforced
- [ ] Type checking is performed
- [ ] Regex patterns are safe (no ReDoS vulnerabilities)

**Data Protection**
- [ ] No secrets in code (passwords, API keys, tokens)
- [ ] Sensitive data is not logged
- [ ] PII is handled according to policy
- [ ] Data is encrypted at rest and in transit

**Injection Prevention**
- [ ] SQL queries use parameterization (no string concatenation)
- [ ] HTML output is escaped
- [ ] Command injection is prevented
- [ ] Path traversal is prevented

**Authentication & Authorization**
- [ ] Authentication is required where needed
- [ ] Authorization checks are performed
- [ ] Session management is secure
- [ ] CSRF protection is implemented

**Dependencies**
- [ ] No known vulnerabilities in dependencies
- [ ] Dependencies are from trusted sources
- [ ] Dependency versions are pinned

### 3.6 Code Quality Review

**Readability**
- [ ] Code is self-documenting
- [ ] Variable names are descriptive
- [ ] Function names indicate purpose
- [ ] Magic numbers are replaced with named constants
- [ ] Complex logic has explanatory comments
- [ ] Code follows established patterns

**Maintainability**
- [ ] Functions are small and focused (< 50 lines)
- [ ] Files are reasonably sized (< 500 lines)
- [ ] Cyclomatic complexity is low (< 15)
- [ ] Code duplication is minimal
- [ ] Dependencies are well-managed
- [ ] Technical debt is documented

**Design**
- [ ] SOLID principles are followed
- [ ] Separation of concerns is maintained
- [ ] Abstractions are appropriate
- [ ] No premature optimization
- [ ] Design patterns are used correctly

**Error Handling**
- [ ] Errors are caught and handled appropriately
- [ ] Error messages are helpful
- [ ] Errors are logged with appropriate context
- [ ] Panic/crash conditions are avoided
- [ ] Resources are cleaned up in error cases

### 3.7 Documentation Review

**Code Documentation**
- [ ] Public APIs are documented
- [ ] Complex algorithms are explained
- [ ] Assumptions are documented
- [ ] Examples are provided
- [ ] Parameters and return values are described
- [ ] Errors/exceptions are documented

**External Documentation**
- [ ] README is updated
- [ ] API documentation is updated
- [ ] Architecture diagrams are updated
- [ ] ADRs are created for significant decisions
- [ ] Migration guides are provided (for breaking changes)

### 3.8 Logging & Monitoring Review

**Logging**
- [ ] Appropriate log levels are used
- [ ] Log messages are informative
- [ ] Sensitive data is not logged
- [ ] Structured logging is used
- [ ] Log volume is reasonable

**Observability**
- [ ] Key metrics are emitted
- [ ] Distributed tracing is implemented
- [ ] Health checks are provided
- [ ] Dashboards are updated

### 3.9 Review Checklist Template

Copy this template to PR descriptions:

```markdown
## Code Review Checklist

### Functionality
- [ ] Requirements met
- [ ] Edge cases handled
- [ ] Errors handled appropriately

### Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Coverage >= 80%
- [ ] All tests passing

### Performance
- [ ] No performance regressions
- [ ] Meets latency requirements
- [ ] Database queries optimized

### Security
- [ ] Input validated
- [ ] No secrets in code
- [ ] SQL injection prevented
- [ ] No dependency vulnerabilities

### Code Quality
- [ ] Functions < 50 lines
- [ ] Files < 500 lines
- [ ] No code duplication
- [ ] SOLID principles followed

### Documentation
- [ ] Public APIs documented
- [ ] README updated
- [ ] ADR created (if significant design)
- [ ] Comments explain WHY not WHAT

### Logging & Monitoring
- [ ] Appropriate logging added
- [ ] Metrics emitted
- [ ] No sensitive data in logs
```

---

**[Continuing with sections 4-10 in next response due to length...]**

---

## Summary

This document defines comprehensive **Code Quality Standards** for the Media Gateway platform. These standards ensure:

1. Consistent code style across Rust and TypeScript
2. Thorough documentation at all levels
3. Rigorous code review processes
4. Multi-gate quality enforcement
5. Proactive technical debt management
6. Performance optimization
7. Security best practices
8. Accessibility compliance

All standards are enforceable through automated tooling and CI/CD pipelines.

---

**Document Control:**
- **Version:** 1.0.0
- **Status:** Planning Phase - Part 1 of 2
- **Next Steps:** Complete sections 4-10 in separate document
- **Maintainer:** @architecture-team
