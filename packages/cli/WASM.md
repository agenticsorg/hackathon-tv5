# ARW CLI - WebAssembly Guide

This guide explains how to build, test, and use the ARW CLI as a WebAssembly module.

## Overview

The ARW CLI can be compiled to WebAssembly (WASM), enabling:
- **Browser integration**: Run validation and generation client-side
- **Node.js usage**: Use without native binary compilation
- **NPM distribution**: Easy installation via package managers
- **Cross-platform**: Works on any platform that supports WASM

## Building for WASM

### Prerequisites

Install the required tools:

```bash
# Install wasm-pack
cargo install wasm-pack

# Install binaryen (for optimization)
# macOS
brew install binaryen

# Ubuntu/Debian
sudo apt-get install binaryen

# Or download from: https://github.com/WebAssembly/binaryen
```

### Build Commands

```bash
# Build for all targets (Node.js, browser, bundlers)
./scripts/build-wasm.sh

# Build for specific target
wasm-pack build --target nodejs --features wasm
wasm-pack build --target web --features wasm
wasm-pack build --target bundler --features wasm

# Optimize WASM binaries
./scripts/optimize-wasm.sh
```

### Build Outputs

After building, you'll find:

- `wasm-pkg/nodejs/` - Node.js target
- `wasm-pkg/web/` - Browser target (ES modules)
- `wasm-pkg/bundler/` - Webpack/Rollup target

Each directory contains:
- `arw_lib.js` - JavaScript bindings
- `arw_lib_bg.wasm` - WebAssembly binary
- `arw_lib.d.ts` - TypeScript definitions
- `package.json` - NPM package metadata

## Usage Examples

### Node.js

```javascript
import {
  validate_manifest_wasm,
  generate_manifest_wasm,
  check_compatibility_wasm,
  get_version_info
} from 'arw-cli-wasm';

// Validate a manifest
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
console.log(result.valid); // true/false
console.log(result.errors); // Array of validation errors

// Generate a manifest
const config = {
  site_name: 'My Site',
  homepage: 'https://example.com',
  contact: 'ai@example.com',
  profile: 'ARW-1',
  description: 'An AI-ready website'
};

const generatedManifest = generate_manifest_wasm(config);
console.log(generatedManifest);

// Check compatibility
const compat = check_compatibility_wasm(manifest, 'ARW-1');
console.log(compat.compatible); // true/false
console.log(compat.message);

// Get version information
const version = get_version_info();
console.log(version.cli_version);
console.log(version.spec_version);
console.log(version.supported_profiles);
```

### Browser (ES Modules)

```html
<!DOCTYPE html>
<html>
<head>
    <title>ARW CLI WASM Demo</title>
</head>
<body>
    <h1>ARW Manifest Validator</h1>

    <textarea id="manifest" rows="20" cols="80">
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
    </textarea>

    <button onclick="validateManifest()">Validate</button>

    <div id="result"></div>

    <script type="module">
        import init, {
            validate_manifest_wasm
        } from './wasm-pkg/web/arw_lib.js';

        // Initialize WASM module
        await init();

        window.validateManifest = async function() {
            const manifest = document.getElementById('manifest').value;
            const result = await validate_manifest_wasm(manifest);

            const resultDiv = document.getElementById('result');
            if (result.valid) {
                resultDiv.innerHTML = '<p style="color: green;">✓ Valid manifest!</p>';
            } else {
                resultDiv.innerHTML = '<p style="color: red;">✗ Invalid manifest:</p><ul>' +
                    result.errors.map(e => `<li>${e.path}: ${e.message}</li>`).join('') +
                    '</ul>';
            }
        };
    </script>
</body>
</html>
```

### Webpack/Vite/Rollup

```javascript
// Install: npm install arw-cli-wasm
import { validate_manifest_wasm, generate_manifest_wasm } from 'arw-cli-wasm';

async function validateAndGenerate() {
  // Generate a manifest
  const config = {
    site_name: 'My App',
    homepage: 'https://myapp.com',
    contact: 'ai@myapp.com',
    profile: 'ARW-2'
  };

  const manifest = generate_manifest_wasm(config);

  // Validate it
  const result = await validate_manifest_wasm(manifest);

  if (!result.valid) {
    console.error('Validation errors:', result.errors);
  }

  return { manifest, result };
}
```

### React Component

```jsx
import { useState, useEffect } from 'react';
import { validate_manifest_wasm } from 'arw-cli-wasm';

function ManifestValidator() {
  const [manifest, setManifest] = useState('');
  const [result, setResult] = useState(null);

  const handleValidate = async () => {
    try {
      const validationResult = await validate_manifest_wasm(manifest);
      setResult(validationResult);
    } catch (error) {
      console.error('Validation failed:', error);
    }
  };

  return (
    <div>
      <textarea
        value={manifest}
        onChange={(e) => setManifest(e.target.value)}
        rows={20}
        cols={80}
      />
      <button onClick={handleValidate}>Validate</button>

      {result && (
        <div>
          {result.valid ? (
            <p style={{ color: 'green' }}>✓ Valid manifest!</p>
          ) : (
            <div>
              <p style={{ color: 'red' }}>✗ Invalid manifest:</p>
              <ul>
                {result.errors.map((error, i) => (
                  <li key={i}>{error.path}: {error.message}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
```

## API Reference

### `validate_manifest_wasm(manifest: string): Promise<ValidationResult>`

Validates an ARW manifest in YAML format.

**Parameters:**
- `manifest` - YAML string containing the manifest

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

### `validate_manifest_json_wasm(manifest: string): Promise<ValidationResult>`

Validates an ARW manifest in JSON format.

**Parameters:**
- `manifest` - JSON string containing the manifest

**Returns:** Same as `validate_manifest_wasm`

### `generate_manifest_wasm(config: ArwConfig): string`

Generates an ARW manifest (llms.txt) from a configuration object.

**Parameters:**
```typescript
{
  site_name: string,
  homepage: string,
  contact: string,
  profile: string,
  description?: string
}
```

**Returns:** YAML string containing the generated manifest

### `check_compatibility_wasm(manifest: string, profile: string): CompatibilityResult`

Checks if a manifest is compatible with a specific ARW profile.

**Parameters:**
- `manifest` - YAML string containing the manifest
- `profile` - Profile to check (e.g., "ARW-1", "ARW-2", "ARW-3")

**Returns:**
```typescript
{
  compatible: boolean,
  manifest_profile: string,
  requested_profile: string,
  message: string
}
```

### `get_version_info(): VersionInfo`

Gets version information about the ARW CLI and specification.

**Returns:**
```typescript
{
  cli_version: string,
  spec_version: string,
  supported_profiles: string[]
}
```

## Performance Characteristics

Based on performance tests:

| Operation | Average Time | Throughput |
|-----------|-------------|------------|
| Validation | ~5ms | ~200 ops/sec |
| Generation | ~0.5ms | ~2000 ops/sec |
| Version info | ~0.05ms | ~20000 ops/sec |
| Large manifest | ~30ms | ~33 ops/sec |

**Binary Sizes** (after optimization):

- Node.js: ~500KB (uncompressed), ~120KB (gzipped)
- Browser: ~480KB (uncompressed), ~115KB (gzipped)
- Bundler: ~490KB (uncompressed), ~118KB (gzipped)

## Testing

### Run WASM Tests

```bash
# Build WASM first
./scripts/build-wasm.sh

# Run Node.js tests
./scripts/test-wasm.sh

# Or run specific test suites
cd wasm-tests
npm test                  # All tests
npm run test:validate     # Validation tests only
npm run test:generate     # Generation tests only
npm run test:perf         # Performance tests only
```

### Run Rust WASM Tests

```bash
# Install wasm-pack if needed
cargo install wasm-pack

# Run WASM tests in browser
wasm-pack test --headless --firefox --features wasm

# Run WASM tests in Node.js
wasm-pack test --node --features wasm
```

## Optimization Tips

### 1. Enable Aggressive Optimization

The build scripts use these optimization flags:
- `-Oz` - Optimize for size
- `--lto=fat` - Link-time optimization
- `wasm-opt -Oz` - Post-processing optimization

### 2. Lazy Loading

For browser usage, load WASM lazily:

```javascript
let wasmModule = null;

async function getWasm() {
  if (!wasmModule) {
    const module = await import('./wasm-pkg/web/arw_lib.js');
    await module.default(); // Initialize
    wasmModule = module;
  }
  return wasmModule;
}

// Use it
const wasm = await getWasm();
const result = await wasm.validate_manifest_wasm(manifest);
```

### 3. Cache WASM Binary

Cache the WASM binary in service workers:

```javascript
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open('arw-wasm-v1').then((cache) => {
      return cache.addAll([
        '/wasm-pkg/web/arw_lib_bg.wasm',
        '/wasm-pkg/web/arw_lib.js'
      ]);
    })
  );
});
```

## Troubleshooting

### "RuntimeError: memory access out of bounds"

This usually means the WASM module needs more memory. Increase the initial memory:

```rust
#[wasm_bindgen(start)]
pub fn init() {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();
}
```

### "Module parse failed: Unexpected character"

Your bundler doesn't recognize WASM. Add WASM support:

**Webpack:**
```javascript
module.exports = {
  experiments: {
    asyncWebAssembly: true
  }
};
```

**Vite:**
```javascript
// vite.config.js
export default {
  optimizeDeps: {
    exclude: ['arw-cli-wasm']
  }
};
```

### "Cannot find module 'arw-cli-wasm'"

Make sure to build the WASM module first:

```bash
./scripts/build-wasm.sh
```

And install it as a local dependency:

```bash
cd wasm-tests
npm install ../wasm-pkg/nodejs
```

### Performance Issues

1. **Use streaming compilation**:
   ```javascript
   const module = await WebAssembly.compileStreaming(
     fetch('/wasm-pkg/web/arw_lib_bg.wasm')
   );
   ```

2. **Batch operations**: Validate multiple manifests in parallel:
   ```javascript
   const results = await Promise.all(
     manifests.map(m => validate_manifest_wasm(m))
   );
   ```

3. **Use Web Workers**: Run WASM in a worker for non-blocking validation:
   ```javascript
   // worker.js
   import init, { validate_manifest_wasm } from './arw_lib.js';
   await init();

   self.onmessage = async (e) => {
     const result = await validate_manifest_wasm(e.data);
     self.postMessage(result);
   };
   ```

## Distribution

### NPM Package

To publish as an NPM package:

```bash
# Build for all targets
./scripts/build-wasm.sh

# Create package.json
cd wasm-pkg/nodejs
npm publish
```

### CDN Distribution

For CDN usage (e.g., unpkg, jsdelivr):

```html
<script type="module">
  import init, { validate_manifest_wasm }
    from 'https://unpkg.com/arw-cli-wasm@latest/arw_lib.js';

  await init();
  // Use the module
</script>
```

## Further Reading

- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [WebAssembly MDN](https://developer.mozilla.org/en-US/docs/WebAssembly)
- [ARW Specification](../../spec/ARW-0.2-DRAFT.md)

## Support

For issues or questions:
- GitHub Issues: https://github.com/agent-ready-web/agent-ready-web/issues
- Documentation: https://agent-ready-web.dev
