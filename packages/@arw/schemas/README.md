# @arw/schemas

TypeScript types and JSON schemas for [Agent-Ready Web (ARW)](https://github.com/agent-ready/agent-ready-web).

## Features

- **TypeScript Types**: Complete type definitions for ARW manifests and discovery files
- **JSON Schemas**: Validation schemas for all ARW file formats
- **Validation Utilities**: Pre-configured validators using AJV
- **Schema-First**: All types derived from authoritative LinkML schema

## Installation

```bash
npm install @arw/schemas
# or
pnpm add @arw/schemas
# or
yarn add @arw/schemas
```

## Usage

### TypeScript Types

```typescript
import type { ARWManifest, Site, ContentItem } from '@arw/schemas';

const manifest: ARWManifest = {
  version: '1.0',
  profile: 'ARW-2',
  site: {
    name: 'CloudCart',
    homepage: 'https://cloudcart.example.com',
    contact: 'ai@cloudcart.example.com',
  },
  content: [
    {
      url: '/products/wireless-keyboard',
      machine_view: '/products/wireless-keyboard.llm.md',
      title: 'Wireless Keyboard',
      priority: 'high',
    },
  ],
};
```

### Validation

```typescript
import { validateManifest } from '@arw/schemas/validation';

const result = validateManifest(manifestData);

if (result.valid) {
  console.log('Valid manifest:', result.data);
} else {
  console.error('Validation errors:');
  result.errors?.forEach((err) => {
    console.error(`  ${err.path}: ${err.message}`);
  });
}
```

### .well-known Files

```typescript
import type { WellKnownManifest, WellKnownPolicies, WellKnownContentIndex } from '@arw/schemas';
import {
  validateWellKnownManifest,
  validateWellKnownPolicies,
  validateWellKnownContentIndex,
} from '@arw/schemas/validation';

// Validate .well-known/arw-manifest.json
const manifestResult = validateWellKnownManifest(jsonData);

// Validate .well-known/arw-policies.json
const policiesResult = validateWellKnownPolicies(policiesData);

// Validate .well-known/arw-content-index.json
const contentIndexResult = validateWellKnownContentIndex(contentIndexData);
```

### Access JSON Schemas Directly

```typescript
import {
  ARWModelSchema,
  ARWManifestSchema,
  ARWPoliciesSchema,
  ARWContentIndexSchema,
} from '@arw/schemas/schemas';

// Use with your own validator
import Ajv from 'ajv';

const ajv = new Ajv();
const validate = ajv.compile(ARWModelSchema);
const valid = validate(data);
```

### Validate by Type

```typescript
import { validateByType } from '@arw/schemas/validation';

// Validate llms.txt manifest
const manifestResult = validateByType('manifest', yamlData);

// Validate .well-known files
const wellKnownResult = validateByType('well-known-manifest', jsonData);
const policiesResult = validateByType('policies', policiesData);
const indexResult = validateByType('content-index', indexData);
```

## Available Types

### Core Types

- `ARWManifest` - Complete llms.txt manifest structure
- `Site` - Site metadata
- `ContentItem` - Content item with machine view
- `Chunk` - Content chunk with semantic boundaries
- `Header` - HTTP header specification
- `Policies` - Policy declarations
- `Action` - Declarative action definition
- `OAuthConfig` - OAuth configuration
- `Protocol` - Protocol endpoint definition

### .well-known Types

- `WellKnownManifest` - .well-known/arw-manifest.json
- `WellKnownPolicies` - .well-known/arw-policies.json
- `WellKnownContentIndex` - .well-known/arw-content-index.json
- `ContentIndexItem` - Individual content index item

### Enums

- `ARWProfile` - Conformance levels (ARW-1, ARW-2, ARW-3, ARW-4)
- `Priority` - Content priority (high, medium, low)
- `PolicyPermission` - Policy values (allow, deny, conditional)
- `GrantType` - OAuth grant types
- `ContentType` - Content types (llm.md, html, json)

## Validation Functions

- `validateManifest(data)` - Validate llms.txt manifest
- `validateWellKnownManifest(data)` - Validate .well-known/arw-manifest.json
- `validateWellKnownPolicies(data)` - Validate .well-known/arw-policies.json
- `validateWellKnownContentIndex(data)` - Validate .well-known/arw-content-index.json
- `validateByType(type, data)` - Validate by ARW file type

All validation functions return:

```typescript
interface ValidationResult<T> {
  valid: boolean;
  errors?: ValidationError[];
  data?: T;
}

interface ValidationError {
  path: string;
  message: string;
  data?: unknown;
}
```

## Schema Sources

This package uses JSON schemas from the ARW repository:

- `schemas/arw_model.json` - Generated from LinkML (authoritative source: `schemas/arw_model.yaml`)
- `schemas/arw-manifest.schema.json` - .well-known/arw-manifest.json schema
- `schemas/arw-policies.schema.json` - .well-known/arw-policies.json schema
- `schemas/arw-content-index.schema.json` - .well-known/arw-content-index.json schema

## Development

### Building

```bash
pnpm install
pnpm build
```

### Type Checking

```bash
pnpm typecheck
```

### Watch Mode

```bash
pnpm dev
```

## Conformance Levels

ARW defines four conformance levels:

- **ARW-1 (Discovery)**: Basic llms.txt with site info and content links
- **ARW-2 (Semantic)**: Adds chunking, headers, and policies
- **ARW-3 (Actions)**: Adds declarative actions with OAuth
- **ARW-4 (Protocol)**: Adds protocol endpoint support

All levels are supported by this package's types and validation.

## Related Packages

- **[@arw/validator](../arw-validator-cli/)** - CLI tool for validating ARW implementations
- **[npx-arw](../../tools/npx-arw/)** - Rust CLI for generating ARW implementations
- **[ARW Inspector](../../tools/arw-inspector/)** - Web-based ARW inspector

## Resources

- [ARW Specification](../../../spec/ARW-v1.0.md)
- [Schema Documentation](../../../schemas/README.md)
- [ARW Website](https://agent-ready-web.org)
- [GitHub Repository](https://github.com/agent-ready/agent-ready-web)

## License

MIT License - see [LICENSE](../../../LICENSE) for details.

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/agent-ready/agent-ready-web) for contribution guidelines.

## Support

- **Issues**: [GitHub Issues](https://github.com/agent-ready/agent-ready-web/issues)
- **Discussions**: [GitHub Discussions](https://github.com/agent-ready/agent-ready-web/discussions)
- **Email**: support@agent-ready-web.org
