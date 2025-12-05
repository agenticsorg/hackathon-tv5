# WASM Quick Start Guide

## Prerequisites

```bash
# Install wasm-pack (required)
cargo install wasm-pack

# Install binaryen for optimization (optional but recommended)
# macOS
brew install binaryen

# Ubuntu/Debian
sudo apt-get install binaryen
```

## Build WASM

```bash
cd packages/cli

# Build for all targets (Node.js, browser, bundlers)
./scripts/build-wasm.sh

# Or build individual targets
wasm-pack build --target nodejs --features wasm
wasm-pack build --target web --features wasm
wasm-pack build --target bundler --features wasm
```

## Test WASM

```bash
# Run Rust WASM tests
wasm-pack test --headless --firefox --features wasm
wasm-pack test --node --features wasm

# Run Node.js tests (after building)
./scripts/test-wasm.sh

# Or run specific Node.js tests
cd wasm-tests
npm install
npm test                  # All tests
npm run test:validate     # Validation only
npm run test:generate     # Generation only
npm run test:perf         # Performance only
```

## Optimize WASM

```bash
# Advanced optimization with size reporting
./scripts/optimize-wasm.sh
```

## Usage Example

### Node.js

```javascript
import {
  validate_manifest_wasm,
  generate_manifest_wasm
} from '../wasm-pkg/nodejs/arw_lib.js';

// Validate
const result = await validate_manifest_wasm(yamlString);
console.log(result.valid, result.errors);

// Generate
const config = {
  site_name: 'My Site',
  homepage: 'https://example.com',
  contact: 'ai@example.com',
  profile: 'ARW-1'
};
const manifest = generate_manifest_wasm(config);
```

### Browser

```html
<script type="module">
  import init, { validate_manifest_wasm }
    from './wasm-pkg/web/arw_lib.js';

  await init();
  const result = await validate_manifest_wasm(yamlString);
</script>
```

## File Locations

- **Build scripts**: `./scripts/`
- **WASM source**: `./src/wasm.rs`
- **Rust tests**: `./tests/wasm/`
- **Node.js tests**: `./wasm-tests/`
- **Build output**: `./wasm-pkg/`
- **Documentation**: `./WASM.md`

## Troubleshooting

### Build fails with "wasm-pack not found"
```bash
cargo install wasm-pack
```

### Tests fail with "module not found"
```bash
# Build WASM first
./scripts/build-wasm.sh

# Then install dependencies
cd wasm-tests && npm install
```

### Large WASM binary size
```bash
# Run optimization
./scripts/optimize-wasm.sh
```

## Next Steps

See `WASM.md` for:
- Complete API reference
- Performance optimization
- Browser integration
- React/Vue examples
- Distribution strategies
