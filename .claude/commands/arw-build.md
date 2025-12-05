# ARW CLI Build

Build the ARW CLI package for local development or release.

## Usage

```
/arw-build [target] [mode]
```

## Targets

- `all` - Build all targets (default)
- `napi` - Node.js native addon only
- `cli` - Standalone executable only
- `wasm` - WASM builds only

## Modes

- `dev` - Development build (fast, debug info)
- `release` - Production build (optimized)

## Instructions

Working directory: `packages/cli`

### Build All Targets

**Default / all:** Build napi-rs addon and standalone CLI
```bash
cd packages/cli
npm install
npm run build                              # napi-rs addon
cargo build --release --features native    # standalone CLI
```

**all release:** Full release builds
```bash
cd packages/cli
npm run build
cargo build --release --features native
npm run build:wasm
```

### Build Specific Targets

**napi:** Node.js native addon (napi-rs)
```bash
cd packages/cli
npm run build          # release
npm run build:debug    # dev/debug
```
Output: `arw-cli.<platform>.node`, `index.js`, `index.d.ts`

**cli:** Standalone executable binary
```bash
cd packages/cli
cargo build --features native                  # dev
cargo build --release --features native        # release
```
Output: `target/debug/arw` or `target/release/arw`

**wasm:** WebAssembly builds
```bash
cd packages/cli
npm run build:wasm              # Node.js target
npm run build:wasm:web          # Browser target
npm run build:wasm:bundler      # Bundler target
```
Output: `wasm-pkg/nodejs/`, `wasm-pkg/web/`, `wasm-pkg/bundler/`

### Cross-Platform CLI Builds

```bash
cargo build --release --features native --target x86_64-unknown-linux-gnu
cargo build --release --features native --target x86_64-apple-darwin
cargo build --release --features native --target aarch64-apple-darwin
cargo build --release --features native --target x86_64-pc-windows-msvc
```

## Build Output Summary

| Target | Dev Command | Release Command | Output |
|--------|-------------|-----------------|--------|
| napi | `npm run build:debug` | `npm run build` | `arw-cli.*.node` |
| cli | `cargo build --features native` | `cargo build --release --features native` | `target/*/arw` |
| wasm | - | `npm run build:wasm` | `wasm-pkg/` |

After building, report:
1. Build status (success/failure)
2. Output locations
3. File sizes for release builds
4. Any warnings or errors
