# ARW CLI Testing Guide

Comprehensive testing documentation for the Agent-Ready Web CLI.

## Table of Contents

1. [Overview](#overview)
2. [Test Organization](#test-organization)
3. [Running Tests](#running-tests)
4. [Test Categories](#test-categories)
5. [Writing Tests](#writing-tests)
6. [Continuous Integration](#continuous-integration)
7. [Coverage Goals](#coverage-goals)
8. [Troubleshooting](#troubleshooting)

## Overview

The ARW CLI test suite provides comprehensive coverage across multiple dimensions:

- **Unit Tests**: Test individual functions and modules
- **Integration Tests**: Test command workflows end-to-end
- **E2E Tests**: Test complete user scenarios
- **Performance Tests**: Benchmark critical operations
- **Regression Tests**: Prevent known bugs from reoccurring

### Test Statistics

- **Total Test Files**: 25+
- **Test Coverage Goal**: 90%+ for unit tests, 80%+ for integration
- **Performance Benchmarks**: All commands < 5s for typical workloads

## Test Organization

```
tests/
├── unit/                      # Unit tests (co-located with source)
│   ├── commands/             # Command-specific tests
│   ├── validators/           # Validation logic tests
│   └── generators/           # Generator tests
│
├── e2e/                      # End-to-end workflow tests
│   ├── helpers/              # Test utilities
│   │   ├── test_server.rs   # Mock HTTP server
│   │   ├── fixtures.rs      # Test data generators
│   │   └── assertions.rs    # Custom assertions
│   ├── common.rs             # Shared test setup
│   ├── validate_workflow_test.rs
│   ├── generate_workflow_test.rs
│   ├── build_workflow_test.rs
│   └── serve_workflow_test.rs
│
├── scenarios/                # Real-world scenario tests
│   ├── new_site_setup_test.rs
│   ├── migration_test.rs
│   └── large_site_test.rs
│
├── cli/                      # CLI-specific tests
│   └── argument_parsing_test.rs
│
├── performance/              # Performance benchmarks
│   ├── validation_speed_test.rs
│   └── generation_speed_test.rs
│
├── regression/               # Regression tests
│   ├── contact_optional_test.rs
│   ├── version_string_test.rs
│   └── well_known_test.rs
│
└── fixtures/                 # Static test fixtures
    ├── valid/                # Valid test manifests
    └── invalid/              # Invalid test manifests
```

## Running Tests

### All Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run with specific test threads
cargo test -- --test-threads=1
```

### Unit Tests Only

```bash
# Run only unit tests
cargo test --lib

# Run specific module
cargo test --lib validators::llms_txt
```

### Integration Tests Only

```bash
# Run all integration tests
cargo test --test '*'

# Run specific test file
cargo test --test validate_workflow_test

# Run E2E tests only
cargo test --test 'e2e::*'
```

### Performance Tests

```bash
# Run performance benchmarks
cargo test --test 'performance::*' -- --nocapture

# Run specific benchmark
cargo test --test validation_speed_test
```

### Regression Tests

```bash
# Run all regression tests
cargo test --test 'regression::*'
```

### Scenario Tests

```bash
# Run real-world scenario tests
cargo test --test 'scenarios::*'
```

### Test with Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View coverage
open coverage/index.html
```

## Test Categories

### 1. Unit Tests

**Purpose**: Test individual functions and modules in isolation.

**Location**: `tests/unit/` and inline `#[cfg(test)]` modules

**Examples**:
- Validator logic (email format, URL validation)
- Parser functions (YAML/JSON parsing)
- Generator logic (HTML to Markdown conversion)

**Characteristics**:
- Fast (< 100ms per test)
- Isolated (no file I/O, no network)
- Deterministic (same input = same output)

### 2. Integration Tests (E2E Workflows)

**Purpose**: Test complete command workflows from CLI invocation to file output.

**Location**: `tests/e2e/`

**Examples**:
- `validate_workflow_test.rs`: Testing full validation workflow
- `generate_workflow_test.rs`: Testing HTML → .llm.md generation
- `build_workflow_test.rs`: Testing complete build process

**Characteristics**:
- Moderate speed (< 5s per test)
- File system interaction
- Tests actual CLI binary

### 3. Scenario Tests

**Purpose**: Test real-world usage patterns and workflows.

**Location**: `tests/scenarios/`

**Examples**:
- Setting up ARW on a new site
- Migrating from legacy llms.txt
- Handling large sites (100+ pages)

**Characteristics**:
- Comprehensive (test multiple commands)
- End-to-end (full user workflows)
- Realistic test data

### 4. Performance Tests

**Purpose**: Ensure commands meet performance requirements.

**Location**: `tests/performance/`

**Performance Goals**:
- Validation: < 2s for small, < 5s for large (100 entries)
- Generation: < 1s per file, < 10s for 50 files
- Build: < 5s for complete build

**Examples**:
```rust
#[test]
fn test_validate_large_manifest_performance() {
    let start = Instant::now();
    validate_100_entry_manifest();
    let duration = start.elapsed();

    assert!(duration.as_secs() < 5);
}
```

### 5. Regression Tests

**Purpose**: Ensure previously fixed bugs don't resurface.

**Location**: `tests/regression/`

**Examples**:
- Contact field is optional (not required)
- Version accepted as string (not just number)
- .well-known files properly structured

**Naming Convention**: Each regression test should reference the issue/PR that fixed it.

## Writing Tests

### Test Structure

Follow the **Arrange-Act-Assert** pattern:

```rust
#[test]
fn test_validate_minimal_manifest() {
    // Arrange: Setup test data
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    // Act: Execute the command
    let output = run_cli_success(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    // Assert: Verify results
    assert_output_contains(&output, "Success");
    assert!(temp_dir.path().join("llms.txt").exists());
}
```

### Using Test Helpers

The test suite provides several helpers:

#### Creating Test Data

```rust
// Create minimal valid manifest
let content = create_minimal_llms_txt();

// Create complete manifest with all features
let content = create_complete_llms_txt();

// Create test site directory
let temp_dir = create_test_site(&content);

// Create complete test site with files
let temp_dir = create_complete_test_site();
```

#### Running CLI Commands

```rust
// Run and expect success
let output = run_cli_success(&["validate", "--path", "/tmp/site"], None);

// Run and expect failure
let (stdout, stderr) = run_cli_failure(&["validate", "--path", "/invalid"], None);

// Run with custom working directory
let output = run_cli_success(&["build"], Some("/tmp/site"));
```

#### Assertions

```rust
// Assert file contains text
assert_file_contains(&path, "expected text");

// Assert valid JSON
assert_valid_json(&json_path);

// Assert valid YAML
assert_valid_yaml(&yaml_path);

// Assert JSON field value
assert_json_field(&json_path, "site.name", "Expected Name");

// Assert directory contains files
assert_directory_contains(&dir, &["file1.txt", "file2.json"]);

// Assert llms.txt and llms.json are equivalent
assert_llms_files_equivalent(&site_path);
```

### Test Naming Conventions

- **Unit tests**: `test_<function>_<scenario>`
  - Example: `test_validate_manifest_missing_version`

- **Integration tests**: `test_<command>_<workflow>`
  - Example: `test_validate_complete_workflow`

- **Scenario tests**: `test_<scenario>_<aspect>`
  - Example: `test_new_site_with_actions`

- **Performance tests**: `test_<operation>_performance`
  - Example: `test_validate_large_manifest_performance`

### Best Practices

1. **Isolation**: Each test should be independent
   ```rust
   // Good: Creates its own temp directory
   let temp_dir = create_temp_dir();

   // Bad: Uses shared directory
   let path = "/tmp/shared";
   ```

2. **Cleanup**: Use `TempDir` for automatic cleanup
   ```rust
   let temp_dir = TempDir::new().unwrap();
   // Automatically cleaned up when temp_dir goes out of scope
   ```

3. **Clear Intent**: Test names should explain what's being tested
   ```rust
   // Good
   #[test]
   fn test_validate_rejects_invalid_email()

   // Bad
   #[test]
   fn test1()
   ```

4. **One Assertion Per Test**: Focus on testing one behavior
   ```rust
   // Good: Tests one thing
   #[test]
   fn test_contact_field_is_optional() {
       let manifest = create_manifest_without_contact();
       assert!(validate(manifest).is_ok());
   }

   // Bad: Tests multiple things
   #[test]
   fn test_manifest_fields() {
       assert!(contact_optional());
       assert!(version_required());
       assert!(profile_valid());
   }
   ```

5. **Descriptive Failure Messages**: Help debugging
   ```rust
   assert!(
       duration.as_secs() < 5,
       "Validation took too long: {:?}",
       duration
   );
   ```

## Continuous Integration

### GitHub Actions Workflow

The CI pipeline runs on every push and pull request:

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run tests
        run: cargo test --all

      - name: Run integration tests
        run: cargo test --test '*'

      - name: Generate coverage
        run: cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

### Pre-commit Hooks

Run tests before committing:

```bash
# .git/hooks/pre-commit
#!/bin/sh
cargo test --lib
if [ $? -ne 0 ]; then
    echo "Unit tests failed. Commit aborted."
    exit 1
fi
```

## Coverage Goals

### Target Coverage

- **Unit Tests**: 90%+ coverage
- **Integration Tests**: 80%+ coverage of commands
- **E2E Tests**: 100% coverage of user-facing commands
- **Regression Tests**: 100% of known bugs

### Measuring Coverage

```bash
# Generate HTML coverage report
cargo tarpaulin --out Html

# Generate coverage for specific module
cargo tarpaulin --lib --packages arw-cli

# Fail if coverage below threshold
cargo tarpaulin --fail-under 90
```

### Current Coverage Status

Run `cargo tarpaulin` to see current coverage:

```
|| Tested/Total Lines:
|| src/commands/validate.rs: 95%
|| src/commands/build.rs: 92%
|| src/validators/llms_txt.rs: 97%
|| Total: 93%
```

## Troubleshooting

### Common Issues

#### 1. Tests Failing Due to File Permissions

**Problem**: Tests fail with "Permission denied"

**Solution**: Ensure temp directories are writable
```rust
let temp_dir = TempDir::new().unwrap();
let path = temp_dir.path();
// Use temp_dir, not hardcoded paths
```

#### 2. Flaky Tests (Intermittent Failures)

**Problem**: Tests pass sometimes, fail other times

**Causes**:
- Race conditions in parallel tests
- Time-dependent assertions
- File system timing issues

**Solutions**:
```rust
// Run tests serially
cargo test -- --test-threads=1

// Add delays for file system operations
thread::sleep(Duration::from_millis(100));
```

#### 3. Slow Test Execution

**Problem**: Test suite takes too long

**Solutions**:
- Run only changed tests during development
- Use `cargo test --lib` for faster feedback
- Mark slow tests with `#[ignore]`

```rust
#[test]
#[ignore]  // Run only with: cargo test -- --ignored
fn test_very_slow_operation() {
    // ...
}
```

#### 4. Binary Not Found

**Problem**: `Failed to find arw binary`

**Solution**: Ensure binary is built
```bash
# Build before running integration tests
cargo build
cargo test --test validate_workflow_test
```

### Debugging Tests

```rust
// Print debug output
#[test]
fn test_with_debug() {
    let result = validate_manifest(&manifest);
    dbg!(&result);  // Print debug info
    assert!(result.is_ok());
}

// Use println in tests
cargo test -- --nocapture

// Run single test with full output
cargo test test_specific_case -- --nocapture --test-threads=1
```

## Contributing Tests

When adding new features or fixing bugs:

1. **Write tests first (TDD)**
   - Write failing test
   - Implement feature
   - Verify test passes

2. **Add regression test for bugs**
   - Every bug fix should have a regression test
   - Reference the issue number in the test

3. **Update this documentation**
   - Document new test helpers
   - Update coverage statistics

4. **Run full test suite before submitting PR**
   ```bash
   cargo test --all
   cargo test --test '*'
   cargo clippy
   cargo fmt
   ```

## Additional Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [ARW Specification](../../spec/ARW-0.1-draft.md)

---

**Last Updated**: 2024-11-12
**Test Suite Version**: 1.0.0
