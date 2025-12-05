# @arw/schemas

## 1.0.0

### Initial Release

**Features:**

- Complete TypeScript types for ARW v1.0 specification
- JSON schema exports for all ARW file formats:
  - `arw_model.json` - llms.txt validation (from LinkML)
  - `arw-manifest.schema.json` - .well-known/arw-manifest.json
  - `arw-policies.schema.json` - .well-known/arw-policies.json
  - `arw-content-index.schema.json` - .well-known/arw-content-index.json
- Validation utilities using AJV:
  - `validateManifest()` - Validate llms.txt manifests
  - `validateWellKnownManifest()` - Validate .well-known/arw-manifest.json
  - `validateWellKnownPolicies()` - Validate .well-known/arw-policies.json
  - `validateWellKnownContentIndex()` - Validate .well-known/arw-content-index.json
  - `validateByType()` - Generic validation by ARW file type
- Support for all ARW conformance levels (ARW-1 through ARW-4)
- Tree-shakeable ESM and CommonJS builds
- Complete TypeScript declarations
- Comprehensive documentation and examples

**Types:**

Core types:

- `ARWManifest` - Complete llms.txt structure
- `Site` - Site metadata
- `ContentItem` - Content with machine views
- `Chunk` - Semantic content chunks
- `Header` - HTTP headers
- `Policies` - Policy declarations
- `Action` - Declarative actions
- `OAuthConfig` - OAuth configuration
- `Protocol` - Protocol endpoints

.well-known types:

- `WellKnownManifest`
- `WellKnownPolicies`
- `WellKnownContentIndex`
- `ContentIndexItem`

Enums:

- `ARWProfile` - Conformance levels
- `Priority` - Content priority
- `PolicyPermission` - Policy values
- `GrantType` - OAuth grant types
- `ContentType` - Content types

**Package Exports:**

- `@arw/schemas` - Main entry point (types + schemas + validation)
- `@arw/schemas/schemas` - JSON schemas only
- `@arw/schemas/validation` - Validation utilities only

**Development:**

- Built with tsup for optimal bundle size
- TypeScript 5.6+ support
- Dual ESM/CJS output
- Source maps included
