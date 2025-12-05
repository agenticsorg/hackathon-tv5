# Publishing ARW CLI to NPM for NPX Usage

This guide covers publishing the ARW CLI as a WASM-powered npm package for simple npx installation.

## Overview

The ARW CLI is published as a **WASM-only package** for universal compatibility:
- ✅ Works on all platforms (Linux, macOS, Windows, any architecture)
- ✅ No binary downloads required
- ✅ Simple npx installation: `npx @agent-ready-web/cli@alpha`
- ✅ Smaller package size (~500KB vs multi-GB with binaries)
- ✅ Programmatic JavaScript API included

## Prerequisites

Before publishing:

- [ ] [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) installed
- [ ] NPM account with access to `@agent-ready-web` organization
- [ ] Logged in to npm: `npm login`
- [ ] All tests passing: `cargo test`

## Publishing Process

### 1. Update Version

Update version in **both** files:

**Cargo.toml:**
```toml
[package]
version = "0.1.0-alpha.1"  # Update this
```

**npm/package.json:**
```json
{
  "version": "0.1.0-alpha.1"  # Match Cargo.toml
}
```

### 2. Build WASM Module

```bash
cd packages/cli

# Build WASM for Node.js
npm run build:wasm

# This creates: npm/pkg/ with WASM module
```

### 3. Test Locally

```bash
# Test the package locally
cd npm
npm link

# Test in another directory
cd /tmp
npx @agent-ready-web/cli --version
```

### 4. Publish to NPM

```bash
cd packages/cli/npm

# For alpha/beta releases
npm publish --tag alpha

# For stable releases
npm publish
```

### 5. Verify Publication

```bash
# Check package page
npm view @agent-ready-web/cli

# Test installation
npx @agent-ready-web/cli@alpha --version
npx @agent-ready-web/cli@alpha --help
```

## Usage Examples

### NPX Command Line

```bash
# Validate a manifest
npx @agent-ready-web/cli@alpha validate ./llms.txt

# Generate a manifest (coming soon - needs CLI implementation)
npx @agent-ready-web/cli@alpha generate --interactive

# Initialize ARW
npx @agent-ready-web/cli@alpha init
```

### JavaScript API

```javascript
const arw = require('@agent-ready-web/cli');

// Validate a manifest
const manifest = `
version: "1.0"
profile: ARW-1
site:
  name: "My Site"
  homepage: "https://example.com"
  contact: "ai@example.com"
policies:
  training:
    allowed: false
  inference:
    allowed: true
`;

const result = await arw.validateManifest(manifest);
if (result.valid) {
  console.log('✓ Valid manifest');
} else {
  console.error('Validation errors:', result.errors);
}

// Generate a manifest
const config = {
  site_name: 'My Site',
  homepage: 'https://example.com',
  contact: 'ai@example.com',
  profile: 'ARW-1'
};

const generated = await arw.generateManifest(config);
console.log(generated);
```

### TypeScript

```typescript
import { validateManifest, generateManifest, ManifestConfig } from '@agent-ready-web/cli';

const config: ManifestConfig = {
  site_name: 'My Site',
  homepage: 'https://example.com',
  contact: 'ai@example.com',
  profile: 'ARW-1'
};

const manifest = await generateManifest(config);
const result = await validateManifest(manifest);
```

## Package Structure

```
@agent-ready-web/cli/
├── package.json          # NPM metadata
├── index.js              # JavaScript API
├── index.d.ts            # TypeScript definitions
├── bin/
│   └── arw               # NPX executable
├── pkg/                  # WASM module (generated)
│   ├── arw_lib.js        # WASM bindings
│   ├── arw_lib_bg.wasm   # WASM binary
│   └── arw_lib.d.ts      # TypeScript defs
└── README.md             # Package documentation
```

## Version Tags

Use npm dist-tags for different release channels:

```bash
# Alpha releases (development, frequent updates)
npm publish --tag alpha
# Install: npx @agent-ready-web/cli@alpha

# Beta releases (testing, feature-complete)
npm publish --tag beta
# Install: npx @agent-ready-web/cli@beta

# Latest (stable releases)
npm publish
# Install: npx @agent-ready-web/cli
```

## Troubleshooting

### WASM Module Not Found

**Problem:** `❌ WASM module not found!`

**Solution:**
```bash
# Build WASM module
cd packages/cli
npm run build:wasm

# Verify it exists
ls -la npm/pkg/
```

### Permission Denied

**Problem:** `npm ERR! code EACCES`

**Solution:**
```bash
# Login to npm
npm login

# Verify you're logged in
npm whoami

# Check organization access
npm org ls @agent-ready-web
```

### Package Already Published

**Problem:** `cannot publish over previously published version`

**Solution:**
```bash
# Bump version in Cargo.toml and package.json
# Then publish again

# Or use a tag for pre-releases
npm publish --tag alpha
```

## Updating the Package

To release a new version:

1. Make code changes
2. Update tests
3. Bump version in both Cargo.toml and package.json
4. Build WASM: `npm run build:wasm`
5. Test locally with `npm link`
6. Publish: `npm publish --tag alpha`
7. Test installation: `npx @agent-ready-web/cli@alpha`

## Performance

WASM module characteristics:
- **Size:** ~500KB uncompressed, ~120KB gzipped
- **Validation:** ~5ms average
- **Generation:** ~0.5ms average
- **Startup:** ~10-20ms (WASM initialization)

For comparison with native binaries:
- WASM is 2-3x slower but universally compatible
- Native binaries are faster but platform-specific
- WASM works everywhere without downloads

## Migration Path

For users wanting native performance:

```bash
# Option 1: Use published releases (future)
# Download from GitHub releases
curl -L https://github.com/agent-ready-web/agent-ready-web/releases/download/v0.1.0/arw-macos-arm64.tar.gz

# Option 2: Build from source
git clone https://github.com/agent-ready-web/agent-ready-web
cd agent-ready-web/packages/cli
cargo build --release
sudo cp target/release/arw /usr/local/bin/
```

## Support

- **Issues:** https://github.com/agent-ready-web/agent-ready-web/issues
- **NPM Package:** https://www.npmjs.com/package/@agent-ready-web/cli
- **Documentation:** https://github.com/agent-ready-web/agent-ready-web

---

**Quick Reference:**

```bash
# Build WASM
npm run build:wasm

# Test locally
npm link
npx @agent-ready-web/cli@alpha

# Publish alpha
npm publish --tag alpha

# Publish stable
npm publish
```
