# ARW CLI Test Suite

Comprehensive end-to-end integration tests for the Agent-Ready Web CLI.

## Quick Start

```bash
# Run all tests
cargo test

# Run specific test category
cargo test --test e2e::validate_workflow_test
cargo test --test scenarios::*
cargo test --test performance::*

# Run with full output
cargo test -- --nocapture

# Run complete test suite with reporting
./scripts/test-all.sh

# Generate coverage report
./scripts/test-coverage.sh

# Watch mode for development
./scripts/test-watch.sh
```

## Test Structure

```
tests/
├── e2e/                          # End-to-end workflow tests
│   ├── helpers/                  # Test utilities
│   │   ├── test_server.rs       # Mock HTTP server
│   │   ├── fixtures.rs          # Test data generators
│   │   └── assertions.rs        # Custom assertions
│   ├── common.rs                 # Shared test setup
│   ├── validate_workflow_test.rs # Validation tests (18 tests)
│   ├── generate_workflow_test.rs # Generation tests (15 tests)
│   ├── build_workflow_test.rs   # Build tests (17 tests)
│   └── serve_workflow_test.rs   # Server tests (7 tests)
│
├── scenarios/                    # Real-world scenarios
│   ├── new_site_setup_test.rs   # New site setup (3 scenarios)
│   ├── migration_test.rs        # Migration paths (4 scenarios)
│   └── large_site_test.rs       # Scale testing (4 scenarios)
│
├── cli/                          # CLI integration
│   └── argument_parsing_test.rs # Argument tests (20 tests)
│
├── performance/                  # Performance benchmarks
│   ├── validation_speed_test.rs # Validation speed (4 benchmarks)
│   └── generation_speed_test.rs # Generation speed (4 benchmarks)
│
└── regression/                   # Regression tests
    ├── contact_optional_test.rs # Contact field (4 tests)
    ├── version_string_test.rs   # Version format (3 tests)
    └── well_known_test.rs       # Discovery files (5 tests)

Total: 35+ test files, 150+ test cases
```

## Test Categories

### 1. E2E Workflow Tests (`e2e/`)
Test complete command workflows from CLI invocation to file output.

**Key Tests**:
- Validate manifests (valid, invalid, different profiles)
- Generate machine views (HTML, Markdown, recursive)
- Build complete ARW structure (all discovery files)
- Serve development server (basic functionality)

### 2. Scenario Tests (`scenarios/`)
Real-world usage patterns and complete workflows.

**Scenarios**:
- **New Site**: Setting up ARW from scratch
- **Migration**: Upgrading from legacy formats
- **Large Site**: Testing with 100+ pages

### 3. CLI Integration (`cli/`)
Command-line interface and argument parsing.

**Coverage**:
- All flags and options
- Command aliases
- Error handling
- Default values

### 4. Performance (`performance/`)
Ensure commands meet speed requirements.

**Benchmarks**:
- Validation: < 2s small, < 5s large
- Generation: < 1s per file, < 10s for 50 files
- Build: < 5s complete

### 5. Regression (`regression/`)
Prevent previously fixed bugs from returning.

**Issues Covered**:
- Contact field optional (not required)
- Version as string (not just number)
- .well-known file structure

## Helper Utilities

### Test Data Generators

```rust
use helpers::*;

// Create minimal valid manifest
let content = create_minimal_llms_txt();

// Create complete manifest with all features
let content = create_complete_llms_txt();

// Create test site directory
let temp_dir = create_test_site(&content);

// Create complete site with all files
let temp_dir = create_complete_test_site();
```

### Running CLI Commands

```rust
use common::*;

// Run and expect success
let output = run_cli_success(&["validate", "--path", "/tmp/site"], None);

// Run and expect failure
let (stdout, stderr) = run_cli_failure(&["validate", "--path", "/invalid"], None);

// Run in specific directory
let output = run_cli_success(&["build"], Some("/tmp/site"));
```

### Custom Assertions

```rust
use helpers::assertions::*;

// Assert file contains text
assert_file_contains(&path, "expected text");

// Assert valid JSON/YAML
assert_valid_json(&json_path);
assert_valid_yaml(&yaml_path);

// Assert JSON field value
assert_json_field(&json_path, "site.name", "Expected Name");

// Assert directory structure
assert_directory_contains(&dir, &["file1.txt", "file2.json"]);

// Assert llms.txt and llms.json match
assert_llms_files_equivalent(&site_path);
```

## Writing New Tests

### Test Template

```rust
/// Test description
mod common;
mod helpers;

use common::*;
use helpers::*;

#[test]
fn test_feature_scenario() {
    // Arrange: Setup test data
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    // Act: Execute the command
    let output = run_cli_success(
        &["command", "--flag", temp_dir.path().to_str().unwrap()],
        None,
    );

    // Assert: Verify results
    assert_output_contains(&output, "Success");
    assert!(temp_dir.path().join("expected.txt").exists());
}
```

### Best Practices

1. **Isolation**: Each test creates its own temp directory
2. **Cleanup**: Use `TempDir` for automatic cleanup
3. **Clear Names**: Test names explain what's being tested
4. **One Assertion**: Focus on one behavior per test
5. **Descriptive Failures**: Add context to assertion messages

## Running Specific Tests

```bash
# Single test
cargo test test_validate_minimal_manifest

# Test file
cargo test --test validate_workflow_test

# Test with filter
cargo test validate

# Ignored tests (manual)
cargo test -- --ignored

# Single-threaded (for debugging)
cargo test -- --test-threads=1
```

## Performance Testing

```bash
# Run performance tests with output
cargo test --test 'performance::*' -- --nocapture

# Example output:
# ✓ Validated 100 content entries in 2.3s
# ✓ Generated 50 machine views in 4.1s
# ✓ Complete build in 3.8s
```

## Coverage Reports

```bash
# Generate HTML coverage report
./scripts/test-coverage.sh

# View report (macOS)
open coverage/index.html

# Fail if coverage below threshold
cargo tarpaulin --fail-under 90
```

## Continuous Integration

The test suite is designed to run in CI/CD pipelines:

```yaml
# .github/workflows/test.yml
- name: Run tests
  run: |
    cargo build
    ./scripts/test-all.sh
    ./scripts/test-coverage.sh
```

## Troubleshooting

### Binary Not Found
```bash
# Build before running integration tests
cargo build
cargo test --test validate_workflow_test
```

### Flaky Tests
```bash
# Run serially for debugging
cargo test -- --test-threads=1
```

### Slow Tests
```bash
# Run only unit tests during development
cargo test --lib

# Mark slow tests with #[ignore]
#[test]
#[ignore]
fn test_very_slow_operation() { }
```

## Test Statistics

- **Total Test Files**: 35+
- **Total Test Cases**: 150+
- **Test Coverage Goal**: 90%+ unit, 80%+ integration
- **Average Test Time**:
  - Unit: < 10ms
  - Integration: < 500ms
  - Full suite: 2-3 minutes

## Documentation

- **[TESTING.md](../TESTING.md)**: Comprehensive testing guide
- **[TEST-SUMMARY.md](../../../docs/TEST-SUMMARY.md)**: Test suite summary
- **[scripts/](../scripts/)**: Automation scripts

## Contributing

When adding new features:

1. **Write tests first** (TDD approach)
2. **Add regression test** for bug fixes
3. **Update documentation** as needed
4. **Run full suite** before submitting PR:
   ```bash
   ./scripts/test-all.sh
   ```

## Need Help?

- Check [TESTING.md](../TESTING.md) for detailed guide
- See existing tests for examples
- Run `cargo test -- --help` for options

---

**Happy Testing! 🧪**
