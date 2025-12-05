# ARW CLI WASM Build Guide

This guide explains how to build the ARW CLI for WebAssembly (WASM) and distribute it via npm.

## Prerequisites

### Required Tools

1. **Rust** (stable toolchain)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update stable
   ```

2. **wasm-pack** (WASM build tool)
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

3. **Node.js** (v16+)
   ```bash
   # Install from https://nodejs.org/
   node --version  # Should be v16 or higher
   ```

4. **wasm32 target** (for Rust)
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

## Build Process

### 1. Build Native Binary

For optimal CLI performance, build the native Rust binary:

```bash
cd tools/npx-arw
cargo build --release
```

The binary will be located at `target/release/arw` (or `arw.exe` on Windows).

### 2. Build WASM Module

To build the WASM module for Node.js:

```bash
cd tools/npx-arw
wasm-pack build --target nodejs --out-dir npm/pkg
```

For browser usage:

```bash
wasm-pack build --target web --out-dir npm/pkg-web
```

For bundlers (webpack, rollup, etc.):

```bash
wasm-pack build --target bundler --out-dir npm/pkg-bundler
```

### 3. Build Everything

To build both native and WASM:

```bash
cd tools/npx-arw/npm
npm run build:all
```

## WASM Features

The WASM build includes the following exported functions:

### `validate_manifest_wasm(manifestContent: string)`

Validates an ARW manifest against the JSON schema.

**Parameters:**
- `manifestContent`: YAML string of the llms.txt manifest

**Returns:**
```typescript
{
  valid: boolean,
  errors: Array<{
    path: string,
    message: string
  }>
}
```

**Example:**
```javascript
import { validate_manifest_wasm } from './pkg/arw_lib.js';

const manifest = `
version: 1.0
profile: ARW-1
site:
  name: My Site
  homepage: https://example.com
  contact: ai@example.com
policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
`;

const result = await validate_manifest_wasm(manifest);
console.log(result.valid); // true or false
console.log(result.errors); // Array of validation errors
```

### `generate_llms_txt_wasm(config: Object)`

Generates an llms.txt manifest from configuration.

**Parameters:**
```typescript
{
  site_name: string,
  homepage: string,
  contact: string,
  profile: 'ARW-1' | 'ARW-2' | 'ARW-3' | 'ARW-4',
  description?: string
}
```

**Returns:** YAML string

**Example:**
```javascript
import { generate_llms_txt_wasm } from './pkg/arw_lib.js';

const config = {
  site_name: 'My Blog',
  homepage: 'https://myblog.com',
  contact: 'ai@myblog.com',
  profile: 'ARW-1',
  description: 'A tech blog'
};

const manifest = generate_llms_txt_wasm(config);
console.log(manifest); // YAML string
```

## NPM Package Usage

### Installation

```bash
npm install @agent-ready-web/cli
```

### CLI Usage

```bash
# Via npx
npx @agent-ready-web/cli --help

# After global install
npm install -g @agent-ready-web/cli
arw --help
```

### JavaScript API

```javascript
const { validateManifest, generateManifest } = require('@agent-ready-web/cli');

// Validate a manifest
const result = await validateManifest(manifestYaml);
if (!result.valid) {
  console.error('Validation errors:', result.errors);
}

// Generate a manifest
const config = {
  site_name: 'My Site',
  homepage: 'https://example.com',
  contact: 'ai@example.com',
  profile: 'ARW-1'
};

const manifest = await generateManifest(config);
```

## Architecture

### Dual-Mode Compilation

The CLI is designed to work in two modes:

1. **Native Binary** (for CLI usage)
   - Compiled with `cargo build --release`
   - Provides full file system access
   - Optimal performance
   - Cross-platform (Linux, macOS, Windows)

2. **WASM Module** (for JavaScript/npm usage)
   - Compiled with `wasm-pack build`
   - Runs in Node.js or browser
   - No file system dependencies (uses virtual FS)
   - Portable and embeddable

### Conditional Compilation

The codebase uses Rust's conditional compilation to handle platform differences:

```rust
// Native file system operations
#[cfg(not(target_arch = "wasm32"))]
use std::fs;

// WASM-compatible operations
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
```

Key files:
- `src/lib.rs` - WASM exports and library interface
- `src/main.rs` - CLI binary entrypoint
- `npm/index.js` - JavaScript wrapper
- `npm/cli.js` - CLI wrapper script

### WASM Feature Flag

The WASM dependencies are gated behind a feature flag:

```toml
[features]
wasm = ["wasm-bindgen", "wasm-bindgen-futures", "serde-wasm-bindgen", "getrandom"]
```

This allows building without WASM dependencies when not needed.

## Testing

### Unit Tests

```bash
cd tools/npx-arw
cargo test
```

### WASM Tests

```bash
cd tools/npx-arw
wasm-pack test --node
```

### Integration Tests

```bash
cd tools/npx-arw
cargo test --test integration
```

## Optimization

### Size Optimization

The release profile is configured for WASM size optimization:

```toml
[profile.release]
opt-level = 3       # Binary optimization
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit
strip = true        # Strip symbols

[profile.release.package."*"]
opt-level = "z"     # Size optimization for dependencies (WASM)
```

### Further Optimization

After building, use `wasm-opt` for additional size reduction:

```bash
# Install wasm-opt (part of binaryen)
npm install -g wasm-opt

# Optimize WASM file
wasm-opt -Oz -o output_optimized.wasm output.wasm
```

Expected sizes:
- Unoptimized: ~2-3 MB
- wasm-pack optimized: ~500KB-1MB
- wasm-opt -Oz: ~300KB-500KB

## Publishing

### Pre-publish Checklist

1. ✅ All tests passing (`cargo test`)
2. ✅ WASM tests passing (`wasm-pack test --node`)
3. ✅ Native binary builds (`cargo build --release`)
4. ✅ WASM module builds (`wasm-pack build`)
5. ✅ Package.json version updated
6. ✅ CHANGELOG updated
7. ✅ Documentation updated

### Publish to npm

```bash
cd tools/npx-arw/npm
npm run build:all
npm publish --access public
```

### Publish to crates.io

```bash
cd tools/npx-arw
cargo publish
```

## Troubleshooting

### Error: "wasm-bindgen not found"

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

### Error: "Cannot find module './pkg'"

Run `npm run build:wasm` to generate the WASM package.

### Error: "Binary not found"

Run `npm run build` to compile the native binary.

### Large WASM Size

1. Ensure release mode: `wasm-pack build --release`
2. Run wasm-opt: `wasm-opt -Oz`
3. Review dependencies for unnecessary features

### Network Errors During Build

If building in an offline environment:

```bash
# Use cargo vendor
cargo vendor

# Add to .cargo/config.toml:
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
```

## Platform Support

### Supported Platforms

#### Native Binary
- ✅ Linux (x86_64, aarch64)
- ✅ macOS (x86_64, Apple Silicon)
- ✅ Windows (x86_64)

#### WASM
- ✅ Node.js (v16+)
- ✅ Browser (all modern browsers)
- ✅ Deno (with npm compatibility)
- ✅ Bun (v1.0+)

### CI/CD Integration

Example GitHub Actions workflow:

```yaml
name: Build and Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test
      - run: wasm-pack test --node

  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
      - run: wasm-pack build --target nodejs

  publish:
    needs: [test, build]
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v')
    steps:
      - uses: actions/checkout@v3
      - run: cargo publish
      - run: cd npm && npm publish
```

## Additional Resources

- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [Rust and WebAssembly Book](https://rustwasm.github.io/book/)
- [ARW Specification](https://github.com/agent-ready-web/agent-ready-web)

## License

MIT License - See LICENSE file for details
