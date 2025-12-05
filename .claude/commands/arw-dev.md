# ARW CLI Development

Start local development workflow for the ARW CLI package.

## Usage

```
/arw-dev [target] [action]
```

## Targets

- `napi` - Node.js native addon (napi-rs) - default
- `cli` - Standalone executable binary
- `wasm` - WebAssembly build

## Actions

- `start` - Build and run locally (default)
- `watch` - Start watch mode
- `link` - Link/install package globally for testing
- `unlink` - Unlink package
- `test` - Run tests for the target

## Instructions

Working directory: `packages/cli`

### napi (default) - Node.js Native Addon

**start:** Build napi-rs native addon
```bash
cd packages/cli
npm install
npm run build
node -e "const cli = require('./index.js'); console.log(cli.getVersionInfo());"
```

**watch:** Watch mode not available for napi-rs, use debug builds
```bash
npm run build:debug
```

**link:** Link for npm testing
```bash
npm link
echo "Package linked. Test in other projects with: npm link @agent-ready-web/cli"
```

**test:** Run Node.js tests
```bash
npm test
```

### cli - Standalone Executable

**start:** Build standalone CLI binary
```bash
cd packages/cli
cargo build --release --features native
./target/release/arw --version
./target/release/arw --help
```

**watch:** Watch mode for CLI development
```bash
cargo install cargo-watch  # if not installed
cargo watch -x "run --features native -- --help"
```

**link:** Install globally
```bash
cargo install --path . --features native
arw --version
```

**test:** Run Rust tests
```bash
cargo test
cargo clippy -- -D warnings
```

### wasm - WebAssembly

**start:** Build WASM package
```bash
cd packages/cli
npm run build:wasm
```

**test:** Run WASM tests
```bash
npm run test:wasm
```

**Available WASM targets:**
```bash
npm run build:wasm          # Node.js
npm run build:wasm:web      # Browser
npm run build:wasm:bundler  # Bundler (webpack, etc.)
```

## Build Target Reference

| Target | Command | Output | Exports |
|--------|---------|--------|---------|
| napi | `npm run build` | `arw-cli.*.node` | `validateManifest`, `generateManifest`, `getVersionInfo` |
| cli | `cargo build --release --features native` | `target/release/arw` | CLI commands: init, validate, generate, etc. |
| wasm | `npm run build:wasm` | `wasm-pkg/` | WASM module |

After action, report:
1. Build status
2. Output location
3. How to test locally
4. Any errors or warnings
