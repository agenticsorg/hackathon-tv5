# ARW CLI Test

Run tests for the ARW CLI package.

## Usage

```
/arw-test [type]
```

## Types

- `all` - Run all tests (default)
- `ts` - TypeScript tests only
- `rust` - Rust tests only
- `lint` - Linting only
- `coverage` - Tests with coverage report

## Instructions

Run tests based on the requested type:

**Default (all):**
```bash
# TypeScript
npm run lint && npm run typecheck && npm test

# Rust
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

**ts:** TypeScript only
```bash
npm run lint
npm run typecheck
npm test
```

**rust:** Rust only
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

**lint:** Linting only
```bash
npm run lint
cargo fmt --check
cargo clippy -- -D warnings
```

**coverage:** With coverage
```bash
npm run test:coverage
cargo test
```

After testing, report:
1. Test results (pass/fail counts)
2. Any failing tests with details
3. Coverage percentage (if applicable)
4. Linting warnings or errors
