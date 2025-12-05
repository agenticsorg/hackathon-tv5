# @agent-ready-web/cli

> CLI tool for implementing Agent-Ready Web (ARW) on any website

[![npm version](https://img.shields.io/npm/v/@agent-ready-web/cli.svg)](https://www.npmjs.com/package/@agent-ready-web/cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Quick Start

No installation required! Use with `npx`:

```bash
npx @agent-ready-web/cli validate ./llms.txt
```

Or install globally:

```bash
npm install -g @agent-ready-web/cli
arw --help
```

## Features

- 🚀 **Zero Config** - Works out of the box with sensible defaults
- 📦 **Multi-Platform** - Native binaries for Linux, macOS, and Windows
- ⚡ **Fast** - Written in Rust for maximum performance
- 🔄 **WASM Fallback** - Automatically falls back to WebAssembly if native binary unavailable
- 🛡️ **Type Safe** - Full TypeScript definitions included
- 📝 **Validated** - Schema validation for ARW manifests

## Installation

### NPX (Recommended)

No installation needed:

```bash
npx @agent-ready-web/cli <command>
```

### Global Installation

```bash
npm install -g @agent-ready-web/cli
```

### Local Installation

```bash
npm install --save-dev @agent-ready-web/cli
```

## Platform Support

Native binaries are automatically downloaded for:

- **Linux**: x64, ARM64
- **macOS**: Intel (x64), Apple Silicon (ARM64)
- **Windows**: x64

If a native binary is not available, the CLI automatically falls back to WebAssembly.

## Usage

### Command Line

```bash
# Validate an ARW manifest
arw validate ./llms.txt

# Generate a new manifest interactively
arw generate --interactive

# Initialize ARW in your project
arw init --profile ARW-2

# Build machine-readable views
arw build --output ./public

# Scan directory for content
arw scan ./content

# Start development server
arw serve --port 3000

# Watch for changes
arw watch
```

### JavaScript/TypeScript

```javascript
const arw = require('@agent-ready-web/cli');

// Validate a manifest
const result = await arw.validateManifest(manifestContent);
if (result.valid) {
  console.log('✓ Valid manifest');
} else {
  console.error('Validation errors:', result.errors);
}

// Generate a manifest
const manifest = await arw.generateManifest({
  site_name: 'My Site',
  homepage: 'https://example.com',
  contact: 'ai@example.com',
  profile: 'ARW-1'
});
console.log(manifest);
```

## Commands

### `validate <file>`

Validate an ARW manifest file against the schema.

```bash
arw validate ./llms.txt
arw validate ./llms.txt --strict
arw validate ./llms.txt --format json
```

**Options:**
- `--strict` - Enable strict validation mode
- `--format <type>` - Output format (text, json)

### `generate [options]`

Generate a new ARW manifest.

```bash
arw generate --interactive
arw generate --profile ARW-2
arw generate --output ./llms.txt
```

**Options:**
- `--interactive` - Interactive mode with prompts
- `--profile <name>` - ARW profile (ARW-1, ARW-2, ARW-3, ARW-4)
- `--output <path>` - Output file path

### `init [options]`

Initialize ARW in your project.

```bash
arw init
arw init --profile ARW-2
arw init --framework next
```

**Options:**
- `--profile <name>` - ARW profile to use
- `--framework <name>` - Web framework (next, nuxt, remix, etc.)

### `build [options]`

Build machine-readable views and indexes.

```bash
arw build
arw build --output ./public
arw build --watch
```

**Options:**
- `--output <path>` - Output directory
- `--watch` - Watch for changes
- `--clean` - Clean output directory first

### `scan [dir]`

Scan directory for content to include in ARW.

```bash
arw scan ./content
arw scan ./docs --recursive
```

**Options:**
- `--recursive` - Scan subdirectories
- `--include <pattern>` - Include pattern
- `--exclude <pattern>` - Exclude pattern

### `serve [options]`

Start development server with ARW preview.

```bash
arw serve
arw serve --port 3000
arw serve --open
```

**Options:**
- `--port <number>` - Server port (default: 3000)
- `--open` - Open browser automatically
- `--cors` - Enable CORS

### `watch [options]`

Watch for changes and rebuild automatically.

```bash
arw watch
arw watch --output ./public
```

**Options:**
- `--output <path>` - Output directory
- `--debounce <ms>` - Debounce delay (default: 300ms)

## Configuration

Create an `arw.config.json` file in your project root:

```json
{
  "version": "0.1",
  "profile": "ARW-2",
  "output": "./public",
  "include": ["content/**/*.md"],
  "exclude": ["node_modules", ".git"],
  "build": {
    "sitemap": true,
    "robotsTxt": true,
    "machineViews": true
  }
}
```

## API Reference

### `validateManifest(content: string): Promise<ValidationResult>`

Validate an ARW manifest.

**Parameters:**
- `content` - YAML content of the manifest

**Returns:**
```typescript
{
  valid: boolean;
  errors?: Array<{
    path: string;
    message: string;
  }>;
}
```

### `generateManifest(config: ManifestConfig): Promise<string>`

Generate an ARW manifest.

**Parameters:**
```typescript
{
  site_name: string;
  homepage: string;
  contact: string;
  profile: 'ARW-1' | 'ARW-2' | 'ARW-3' | 'ARW-4';
  description?: string;
}
```

**Returns:** YAML manifest content as string

## Architecture

The CLI uses a hybrid approach:

1. **Native Binary** (Primary) - Rust-compiled binaries for maximum performance
2. **WASM Fallback** (Secondary) - WebAssembly module when native binary unavailable

### Binary Distribution

Binaries are automatically downloaded on first run from GitHub releases:

```
https://github.com/agent-ready-web/agent-ready-web/releases/download/v{version}/
├── arw-darwin-x64.tar.gz
├── arw-darwin-arm64.tar.gz
├── arw-linux-x64.tar.gz
├── arw-linux-arm64.tar.gz
└── arw-windows-x64.zip
```

### Directory Structure

```
@agent-ready-web/cli/
├── bin/
│   └── arw              # NPX entry point
├── lib/
│   ├── platforms.js     # Platform detection
│   ├── binary.js        # Binary execution
│   └── install.js       # Post-install downloader
├── binaries/            # Downloaded binaries (gitignored)
├── pkg/                 # WASM fallback
└── index.js             # JavaScript API
```

## Troubleshooting

### Binary Download Failed

If the binary download fails during installation:

```bash
# Try manual installation
npm install @agent-ready-web/cli --no-optional

# Or use WASM fallback
npm run build:wasm
```

### Platform Not Supported

If your platform isn't supported:

```bash
# Build from source
git clone https://github.com/agent-ready-web/agent-ready-web
cd agent-ready-web/packages/cli
cargo build --release

# Or use WASM
npm run build:wasm
```

### Permission Denied

If you get permission errors:

```bash
# Fix binary permissions
chmod +x node_modules/@agent-ready-web/cli/binaries/arw
```

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/agent-ready-web/agent-ready-web
cd agent-ready-web/packages/cli

# Build native binary
cargo build --release

# Build WASM module
npm run build:wasm

# Run tests
npm test
```

### Testing

```bash
# Rust tests
cargo test

# WASM tests
npm run test:wasm

# NPM package tests
npm run test:npm

# Integration tests
npm run test:package
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for details.

## License

MIT © [Nolan Dubeau](https://github.com/nolandubeau)

## Links

- [Documentation](https://github.com/agent-ready-web/agent-ready-web)
- [Issue Tracker](https://github.com/agent-ready-web/agent-ready-web/issues)
- [Changelog](../../CHANGELOG.md)
- [ARW Specification](../../spec/ARW-0.1-draft.md)

## Related

- [@agent-ready-web/schemas](../schemas) - ARW schema definitions
- [@agent-ready-web/validator](../validator) - JavaScript validator
