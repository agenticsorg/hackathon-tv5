# ARW LinkML Schemas

This directory contains the formal LinkML schema for validating Agent-Ready Web (ARW) manifests and discovery files.

**Version**: 0.2.0
**Status**: Production Ready
**Backward Compatible**: Yes (with v0.1)

## Files

- **arw_model.yaml** - LinkML schema (authoritative source) - **v0.2**
- **examples/** - Example manifests in YAML and JSON formats
  - `arw-manifest-v0.2-yaml.yaml` - Full YAML example
  - `arw-manifest-v0.2-json.json` - Full JSON example

## What's New in v0.2

### Major Features

1. **Dual Format Support**: YAML and JSON serialization
2. **Discovery Mechanisms**: robots.txt, .well-known, HTTP headers, meta tags
3. **MIME Type Configuration**: Preferred and fallback MIME types
4. **HTTP Headers**: AI-Version, AI-Profile, AI-Format headers
5. **Platform Compatibility**: Track tested frameworks and platforms
6. **Agent Compatibility**: Track AI agent testing results

### New Schema Classes

- `ManifestMetadata` - Format and discovery metadata
- `MimeTypeConfig` - MIME type preferences
- `HTTPHeaders` - Required HTTP headers
- `PlatformCompatibility` - Platform testing information
- `AgentCompatibility` - AI agent testing information
- `TestedPlatform` - Individual platform test results
- `TestedAgent` - Individual agent test results

### New Enumerations

- `SerializationFormat` - yaml, json
- `DiscoveryMethod` - robots_txt, well_known, http_headers, meta_tags
- `MimeType` - text_plain, application_json, text_markdown

## Schema Purposes

### arw_model.json

Validates the YAML structure of `/llms.txt` files:

```yaml
version: 1.0
profile: ARW-2

site:
  name: 'Example'
  homepage: 'https://example.com'
  contact: 'ai@example.com'

content:
  - url: /
    machine_view: /index.llm.md
    priority: high

policies:
  training:
    allowed: true
  inference:
    allowed: true
  attribution:
    required: true
```

### arw-manifest.schema.json

Validates `/.well-known/arw-manifest.json`:

```json
{
  "arw_version": "1.0",
  "site": {
    "name": "Example",
    "homepage": "https://example.com"
  },
  "links": {
    "guide": "/llms.txt",
    "policies": "/.well-known/arw-policies.json",
    "content_index": "/.well-known/arw-content-index.json"
  }
}
```

### arw-policies.schema.json

Validates `/.well-known/arw-policies.json`:

```json
{
  "inference": "allow",
  "training": "conditional",
  "attribution": {
    "required": true,
    "template": "Source: {site_name}"
  },
  "rate_limits": {
    "anonymous_rps": 2,
    "oauth_rps": 10
  }
}
```

### arw-content-index.schema.json

Validates `/.well-known/arw-content-index.json`:

```json
{
  "version": "1.0",
  "items": [
    {
      "id": "home",
      "type": "llm.md",
      "url": "https://example.com/index.llm.md",
      "hash": "sha256-...",
      "last_modified": "2025-11-01T00:00:00Z",
      "tags": ["marketing"]
    }
  ]
}
```

## Using the Schemas

### Validation

Use the validators in `tools/validators/`:

```bash
# Python
python tools/validators/validate-arw.py www/public/llms.txt --schema schemas/arw_model.json

# Node.js
node tools/validators/validate-arw.mjs www/public/llms.txt schemas/arw_model.json
```

### CI Integration

The schemas are automatically validated in CI via `.github/workflows/validate-arw.yml`.

### Editor Integration

#### VS Code

Add to `.vscode/settings.json`:

```json
{
  "yaml.schemas": {
    "./schemas/arw_model.json": "llms.txt"
  },
  "json.schemas": [
    {
      "fileMatch": ["**/.well-known/arw-manifest.json"],
      "url": "./schemas/arw-manifest.schema.json"
    },
    {
      "fileMatch": ["**/.well-known/arw-policies.json"],
      "url": "./schemas/arw-policies.schema.json"
    },
    {
      "fileMatch": ["**/.well-known/arw-content-index.json"],
      "url": "./schemas/arw-content-index.schema.json"
    }
  ]
}
```

#### JetBrains IDEs

1. Open Settings → Languages & Frameworks → Schemas and DTDs → JSON Schema Mappings
2. Add new mapping for each schema
3. Set file patterns and schema URLs

## Migration from v0.1

See **[Schema Migration Guide](../../docs/schema-migration-v0.2.md)** for detailed upgrade instructions.

**Quick Summary**:
- v0.2 is fully backward compatible with v0.1
- `metadata` section is optional but recommended
- YAML format continues to work unchanged
- JSON format is now officially supported

## Schema Development

### Regenerating JSON Schema from LinkML

If you modify `arw_model.yaml`:

```bash
# Install LinkML
pip install linkml

# Generate JSON Schema
gen-json-schema packages/schemas/arw_model.yaml > packages/schemas/generated/arw_model.json

# Generate Python dataclasses
gen-python packages/schemas/arw_model.yaml > packages/schemas/generated/arw_model.py

# Generate TypeScript types
gen-typescript packages/schemas/arw_model.yaml > packages/schemas/generated/arw_model.ts
```

### Validation Examples

**Validate YAML manifest**:
```bash
linkml-validate -s packages/schemas/arw_model.yaml llms.txt
```

**Validate JSON manifest**:
```bash
linkml-validate -s packages/schemas/arw_model.yaml .well-known/arw-manifest.json
```

### Schema Versioning

Schemas follow the ARW specification version:

- **v0.2** - Current release (dual format, discovery, compatibility tracking)
- **v0.1** - Legacy (YAML only, fully supported)
- Future versions will maintain backward compatibility where possible

## Conformance Levels

The schemas validate requirements for all ARW conformance levels:

- **ARW-1 Discovery**: Basic fields (version, profile, site, content)
- **ARW-2 Semantic**: Adds chunks, headers, policies
- **ARW-3 Actions**: Adds actions, OAuth configuration
- **ARW-4 Protocol**: Adds protocol endpoints

## Related Documentation

- [ARW Specification](../spec/ARW-v1.0.md)
- [Discovery Guide](../docs/discovery/ARW-Discovery-Guide.md)
- [Validator Tools](../tools/validators/README.md)

## Schema Reference

For detailed field definitions, see the [ARW Specification Section 3](../spec/ARW-v1.0.md#3-discovery--llmstxt).
