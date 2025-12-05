# ARW CLI Testing Strategy

## Comprehensive Test Coverage for Production-Ready Reliability

**Version:** 1.0
**Date:** January 2025
**Related:** CLI-EXPANSION-PLAN.md

---

## Table of Contents

1. [Overview](#1-overview)
2. [Testing Philosophy](#2-testing-philosophy)
3. [Test Structure](#3-test-structure)
4. [Unit Testing](#4-unit-testing)
5. [Integration Testing](#5-integration-testing)
6. [End-to-End Testing](#6-end-to-end-testing)
7. [Performance Testing](#7-performance-testing)
8. [Test Fixtures](#8-test-fixtures)
9. [CI/CD Integration](#9-cicd-integration)
10. [Coverage Goals](#10-coverage-goals)

---

## 1. Overview

This document defines the comprehensive testing strategy for the ARW CLI, ensuring production-ready reliability through multiple layers of testing. The goal is to achieve 85%+ overall code coverage while maintaining fast test execution and developer-friendly debugging.

### Testing Layers

```
┌─────────────────────────────────────────────────┐
│           E2E Tests (5%)                        │
│  Full workflows, real-world scenarios           │
├─────────────────────────────────────────────────┤
│         Integration Tests (25%)                  │
│  Command execution, file I/O, workflows         │
├─────────────────────────────────────────────────┤
│           Unit Tests (70%)                       │
│  Functions, modules, core logic                 │
└─────────────────────────────────────────────────┘
```

---

## 2. Testing Philosophy

### 2.1 Core Principles

1. **Fast Feedback** - Unit tests run in <5 seconds, full suite in <2 minutes
2. **Deterministic** - Tests produce same results on every run
3. **Isolated** - Tests don't depend on external services or state
4. **Readable** - Test names clearly describe what they verify
5. **Maintainable** - DRY principles, shared fixtures, helper functions

### 2.2 Test Pyramid Strategy

```
     /\
    /E2\     E2E:         5% - Full user workflows
   /----\    Integration: 25% - Command integration
  /------\   Unit:        70% - Core logic
 /--------\
```

**Rationale:**

- Unit tests provide fast feedback on core logic
- Integration tests verify commands work together
- E2E tests ensure real-world scenarios work end-to-end
- Balance between coverage and execution speed

### 2.3 What to Test

**Always Test:**

- ✅ Core business logic (generators, parsers, validators)
- ✅ Command execution and argument parsing
- ✅ File I/O operations
- ✅ Error handling and edge cases
- ✅ Configuration loading and validation

**Sometimes Test:**

- ⚠️ Third-party library integration (mock when possible)
- ⚠️ Network operations (use wiremock)
- ⚠️ File system operations (use tempdir)

**Rarely Test:**

- ❌ Third-party library internals
- ❌ Standard library functions
- ❌ Generated code (unless complex logic)

---

## 3. Test Structure

### 3.1 Directory Layout

```
cli/
├── src/
│   ├── commands/
│   ├── generators/
│   ├── parsers/
│   ├── validators/
│   └── utils/
│
└── tests/
    ├── unit/
    │   ├── commands/
    │   │   ├── mod.rs
    │   │   ├── init_test.rs
    │   │   ├── generate_test.rs
    │   │   └── validate_test.rs
    │   ├── generators/
    │   │   ├── llms_txt_test.rs
    │   │   ├── machine_view_test.rs
    │   │   ├── sitemap_test.rs
    │   │   └── robots_test.rs
    │   ├── parsers/
    │   │   ├── html_test.rs
    │   │   ├── markdown_test.rs
    │   │   └── frontmatter_test.rs
    │   ├── validators/
    │   │   ├── llms_txt_test.rs
    │   │   ├── sitemap_test.rs
    │   │   └── policy_test.rs
    │   └── utils/
    │       ├── chunking_test.rs
    │       ├── config_test.rs
    │       └── crawler_test.rs
    │
    ├── integration/
    │   ├── commands/
    │   │   ├── init_integration_test.rs
    │   │   ├── generate_integration_test.rs
    │   │   ├── validate_integration_test.rs
    │   │   └── build_integration_test.rs
    │   ├── workflows/
    │   │   ├── full_setup_test.rs
    │   │   ├── migration_test.rs
    │   │   └── cicd_test.rs
    │   └── standards/
    │       ├── robots_txt_test.rs
    │       └── sitemap_xml_test.rs
    │
    ├── e2e/
    │   ├── scenarios/
    │   │   ├── blog_setup_test.rs
    │   │   ├── ecommerce_setup_test.rs
    │   │   ├── docs_setup_test.rs
    │   │   └── saas_setup_test.rs
    │   └── real_world/
    │       ├── large_site_test.rs
    │       └── migration_test.rs
    │
    ├── fixtures/
    │   ├── sample_sites/
    │   │   ├── blog/
    │   │   ├── ecommerce/
    │   │   └── docs/
    │   ├── html/
    │   ├── markdown/
    │   └── config/
    │
    ├── helpers/
    │   ├── mod.rs
    │   ├── assertions.rs
    │   ├── fixtures.rs
    │   └── mocks.rs
    │
    └── common/
        └── mod.rs
```

### 3.2 Test File Naming Convention

- **Unit tests**: `{module}_test.rs` (e.g., `llms_txt_test.rs`)
- **Integration tests**: `{feature}_integration_test.rs`
- **E2E tests**: `{scenario}_test.rs`
- **Test modules**: Always named `tests` (Rust convention)

---

## 4. Unit Testing

### 4.1 Unit Test Structure

```rust
// tests/unit/generators/llms_txt_test.rs

use arw_cli::generators::llms_txt;
use arw_cli::utils::config::{ArwConfig, SiteConfig, PolicyConfig};

#[cfg(test)]
mod llms_txt_tests {
    use super::*;

    // Test data setup
    fn create_test_config() -> ArwConfig {
        ArwConfig {
            site: SiteConfig {
                title: "Test Site".to_string(),
                description: "Test description".to_string(),
                homepage: "https://example.com".to_string(),
                contact: Some("test@example.com".to_string()),
                languages: vec!["en".to_string()],
            },
            policies: PolicyConfig::default(),
            generation: GenerationConfig::default(),
        }
    }

    // Happy path test
    #[test]
    fn test_generate_minimal_llms_txt() {
        let config = create_test_config();
        let content = llms_txt::format_llms_txt(&config);

        assert!(content.contains("version: 0.1"));
        assert!(content.contains("title: \"Test Site\""));
        assert!(content.contains("homepage: \"https://example.com\""));
    }

    // Edge case test
    #[test]
    fn test_generate_with_empty_contact() {
        let mut config = create_test_config();
        config.site.contact = None;

        let content = llms_txt::format_llms_txt(&config);

        assert!(!content.contains("contact:"));
    }

    // Error case test
    #[test]
    fn test_invalid_yaml_characters() {
        let mut config = create_test_config();
        config.site.title = "Title with \"quotes\"".to_string();

        let content = llms_txt::format_llms_txt(&config);

        // Should escape quotes
        assert!(content.contains("title: \"Title with \\\"quotes\\\"\""));
    }

    // Parametrized test
    #[test]
    fn test_policy_variations() {
        let test_cases = vec![
            (true, true, true),
            (false, true, true),
            (true, false, false),
            (false, false, true),
        ];

        for (training, inference, attribution) in test_cases {
            let mut config = create_test_config();
            config.policies.allow_training = training;
            config.policies.allow_inference = inference;
            config.policies.require_attribution = attribution;

            let content = llms_txt::format_llms_txt(&config);

            assert_eq!(content.contains("allow_training: true"), training);
            assert_eq!(content.contains("allow_inference: true"), inference);
            assert_eq!(content.contains("require_attribution: true"), attribution);
        }
    }
}
```

### 4.2 Unit Test Categories

#### 4.2.1 Generator Tests

Test file generation logic:

```rust
// tests/unit/generators/machine_view_test.rs

#[test]
fn test_html_to_markdown_conversion() {
    let html = r#"
        <html>
            <body>
                <h1>Title</h1>
                <p>Paragraph with <strong>bold</strong> text.</p>
            </body>
        </html>
    "#;

    let markdown = machine_view::from_html(html, "test.html").unwrap();

    assert!(markdown.contains("# Title"));
    assert!(markdown.contains("Paragraph with **bold** text"));
}

#[test]
fn test_chunk_marker_insertion() {
    let markdown = "# Heading\n\nContent here\n\n## Subheading\n\nMore content";

    let with_chunks = machine_view::add_chunk_markers(&markdown);

    assert!(with_chunks.contains("<!-- chunk: heading -->"));
    assert!(with_chunks.contains("<!-- chunk: subheading -->"));
}

#[test]
fn test_semantic_chunking() {
    let content = "Long content here...";

    let chunks = chunking::semantic_chunk(&content, 500);

    assert!(chunks.len() > 1);
    for chunk in &chunks {
        assert!(chunk.tokens >= 200 && chunk.tokens <= 800);
    }
}
```

#### 4.2.2 Parser Tests

Test content parsing:

```rust
// tests/unit/parsers/html_test.rs

#[test]
fn test_extract_title() {
    let html = r#"<html><head><title>Page Title</title></head></html>"#;
    let title = html_parser::extract_title(html);
    assert_eq!(title, Some("Page Title".to_string()));
}

#[test]
fn test_extract_chunks_with_data_attributes() {
    let html = r#"
        <section data-chunk-id="intro">Introduction</section>
        <section data-chunk-id="content">Main content</section>
    "#;

    let chunks = html_parser::extract_chunks(html).unwrap();

    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].id, "intro");
    assert_eq!(chunks[1].id, "content");
}

#[test]
fn test_extract_schema_org_actions() {
    let html = r#"
        <script type="application/ld+json">
        {
            "@type": "Product",
            "potentialAction": {
                "@type": "BuyAction",
                "target": "/api/add-to-cart"
            }
        }
        </script>
    "#;

    let actions = html_parser::extract_schema_actions(html).unwrap();

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].action_type, "BuyAction");
}
```

#### 4.2.3 Validator Tests

Test validation logic:

```rust
// tests/unit/validators/llms_txt_test.rs

#[test]
fn test_valid_llms_txt() {
    let content = r#"
        version: 0.1

        site:
          title: "Test"
          homepage: "https://example.com"

        policies:
          allow_training: false
          allow_inference: true
    "#;

    let result = validator::validate_llms_txt(content);
    assert!(result.is_ok());
}

#[test]
fn test_missing_required_fields() {
    let content = r#"
        version: 0.1

        site:
          title: "Test"
          # Missing homepage

        policies:
          allow_training: false
    "#;

    let result = validator::validate_llms_txt(content);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(errors.contains("Missing required field: homepage"));
}

#[test]
fn test_invalid_version() {
    let content = r#"
        version: 999.0

        site:
          title: "Test"
          homepage: "https://example.com"
    "#;

    let result = validator::validate_llms_txt(content);
    assert!(result.is_err());
}
```

### 4.3 Test Utilities

```rust
// tests/helpers/assertions.rs

/// Assert that a file exists
pub fn assert_file_exists<P: AsRef<Path>>(path: P) {
    assert!(
        path.as_ref().exists(),
        "File does not exist: {}",
        path.as_ref().display()
    );
}

/// Assert that file contains text
pub fn assert_file_contains<P: AsRef<Path>>(path: P, text: &str) {
    let content = std::fs::read_to_string(path.as_ref()).unwrap();
    assert!(
        content.contains(text),
        "File does not contain expected text: '{}'",
        text
    );
}

/// Assert that YAML parses correctly
pub fn assert_valid_yaml(content: &str) {
    serde_yaml::from_str::<serde_yaml::Value>(content)
        .expect("Invalid YAML");
}

/// Assert that JSON parses correctly
pub fn assert_valid_json(content: &str) {
    serde_json::from_str::<serde_json::Value>(content)
        .expect("Invalid JSON");
}
```

---

## 5. Integration Testing

### 5.1 Command Integration Tests

Test full command execution:

```rust
// tests/integration/commands/init_integration_test.rs

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_init_creates_all_files() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("arw").unwrap();
    cmd.arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ Success"));

    // Verify files created
    assert!(temp_dir.path().join(".arw/config.yaml").exists());
    assert!(temp_dir.path().join("llms.txt").exists());
    assert!(temp_dir.path().join("policy.json").exists());
    assert!(temp_dir.path().join("sitemap.xml").exists());
    assert!(temp_dir.path().join("robots.txt").exists());
}

#[tokio::test]
async fn test_init_with_template() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("init")
        .arg("--template")
        .arg("ecommerce")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success();

    // Verify ecommerce template content
    let llms_txt = std::fs::read_to_string(
        temp_dir.path().join("llms.txt")
    ).unwrap();

    assert!(llms_txt.contains("actions:"));
    assert!(llms_txt.contains("add_to_cart"));
    assert!(llms_txt.contains("create_order"));
}

#[tokio::test]
async fn test_init_refuses_to_overwrite() {
    let temp_dir = TempDir::new().unwrap();

    // First init
    Command::cargo_bin("arw")
        .unwrap()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success();

    // Second init should fail
    Command::cargo_bin("arw")
        .unwrap()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}
```

### 5.2 Workflow Integration Tests

Test command workflows:

```rust
// tests/integration/workflows/full_setup_test.rs

#[tokio::test]
async fn test_complete_setup_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Initialize
    Command::cargo_bin("arw")
        .unwrap()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success();

    // Step 2: Add sample HTML
    copy_fixtures("sample_pages", temp_dir.path());

    // Step 3: Generate machine views
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("pages"))
        .arg("--recursive")
        .assert()
        .success();

    // Step 4: Validate
    Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Step 5: Build
    Command::cargo_bin("arw")
        .unwrap()
        .arg("build")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify build artifacts
    let build_dir = temp_dir.path().join(".arw-dist");
    assert!(build_dir.join("llms.txt").exists());
    assert!(build_dir.join("sitemap.xml").exists());
}
```

### 5.3 Migration Integration Tests

```rust
// tests/integration/workflows/migration_test.rs

#[tokio::test]
async fn test_migrate_from_llms_txt() {
    let temp_dir = TempDir::new().unwrap();

    // Create old-style llms.txt
    let old_llms_txt = r#"
        # My Website

        > Website description

        ## Documentation
        [Getting Started](/docs/start): Introduction
        [API Reference](/docs/api): API documentation
    "#;

    std::fs::write(
        temp_dir.path().join("llms.txt"),
        old_llms_txt
    ).unwrap();

    // Migrate
    Command::cargo_bin("arw")
        .unwrap()
        .arg("migrate")
        .arg("llmstxt")
        .arg("--source")
        .arg(temp_dir.path().join("llms.txt"))
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify new format
    let new_llms_txt = std::fs::read_to_string(
        temp_dir.path().join("llms.txt")
    ).unwrap();

    assert!(new_llms_txt.contains("version: 0.1"));
    assert!(new_llms_txt.contains("content:"));
    assert!(new_llms_txt.contains("machine_view:"));
}
```

---

## 6. End-to-End Testing

### 6.1 Real-World Scenario Tests

```rust
// tests/e2e/scenarios/ecommerce_setup_test.rs

#[tokio::test]
#[ignore] // Run only on CI or with --ignored flag
async fn test_full_ecommerce_implementation() {
    let temp_dir = TempDir::new().unwrap();

    // 1. Initialize with ecommerce template
    Command::cargo_bin("arw")
        .unwrap()
        .arg("init")
        .arg("--template")
        .arg("ecommerce")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success();

    // 2. Copy realistic e-commerce fixtures
    copy_fixtures("ecommerce/full_site", temp_dir.path());

    // 3. Generate machine views with semantic chunking
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("products"))
        .arg("--recursive")
        .arg("--chunking")
        .arg("semantic")
        .arg("--quality-score")
        .assert()
        .success();

    // 4. Add actions
    Command::cargo_bin("arw")
        .unwrap()
        .arg("actions")
        .arg("add")
        .arg("--id")
        .arg("add_to_cart")
        .arg("--endpoint")
        .arg("/api/cart/add")
        .arg("--method")
        .arg("POST")
        .arg("--auth")
        .arg("oauth2")
        .assert()
        .success();

    // 5. Validate comprehensive
    let output = Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--level")
        .arg("comprehensive")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Score:"));
    assert!(stdout.contains("PASS"));

    // 6. Build with optimization
    Command::cargo_bin("arw")
        .unwrap()
        .arg("build")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--optimize")
        .arg("--minify")
        .assert()
        .success();

    // 7. Verify build quality
    let build_dir = temp_dir.path().join(".arw-dist");
    let report_path = build_dir.join("build-report.json");
    assert!(report_path.exists());

    let report: BuildReport = serde_json::from_str(
        &std::fs::read_to_string(report_path).unwrap()
    ).unwrap();

    // Verify optimizations
    assert!(report.optimization.token_reduction >= 0.15);
    assert!(report.optimization.file_size_reduction >= 0.10);
    assert!(report.quality.arw_compliance >= 95.0);

    // 8. Verify all generated files
    assert!(build_dir.join("llms.txt").exists());
    assert!(build_dir.join("sitemap.xml").exists());
    assert!(build_dir.join("sitemap.llm.json").exists());
    assert!(build_dir.join("robots.txt").exists());
    assert!(build_dir.join("policy.json").exists());

    // 9. Verify robots.txt allows ARW files
    let robots = std::fs::read_to_string(
        build_dir.join("robots.txt")
    ).unwrap();
    assert!(robots.contains("Allow: /llms.txt"));
    assert!(robots.contains("Allow: /*.llm.md"));
}
```

### 6.2 Large Site Tests

```rust
// tests/e2e/real_world/large_site_test.rs

#[tokio::test]
#[ignore] // Long-running test
async fn test_1000_page_site_generation() {
    let temp_dir = TempDir::new().unwrap();

    // Generate 1000 sample pages
    generate_sample_pages(temp_dir.path(), 1000);

    // Initialize
    Command::cargo_bin("arw")
        .unwrap()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success();

    // Generate machine views with parallel processing
    let start = std::time::Instant::now();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("pages"))
        .arg("--recursive")
        .arg("--parallel")
        .arg("8")
        .assert()
        .success();

    let duration = start.elapsed();

    // Should complete in reasonable time (< 2 minutes for 1000 pages)
    assert!(duration.as_secs() < 120);

    // Verify all machine views created
    let machine_views = walkdir::WalkDir::new(temp_dir.path())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        .count();

    assert_eq!(machine_views, 1000);

    // Validate (should handle large sites)
    Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Build (with optimization)
    Command::cargo_bin("arw")
        .unwrap()
        .arg("build")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--optimize")
        .assert()
        .success();
}
```

---

## 7. Performance Testing

### 7.1 Benchmark Setup

```rust
// benches/generation_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use arw_cli::generators::machine_view;

fn benchmark_html_conversion(c: &mut Criterion) {
    let html_sizes = vec![
        ("small", load_fixture("small_page.html")),   // 5KB
        ("medium", load_fixture("medium_page.html")),  // 50KB
        ("large", load_fixture("large_page.html")),   // 200KB
    ];

    let mut group = c.benchmark_group("html_to_markdown");

    for (name, html) in html_sizes {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &html,
            |b, html| {
                b.iter(|| {
                    machine_view::from_html(black_box(html), black_box("test.html"))
                })
            },
        );
    }

    group.finish();
}

fn benchmark_chunking(c: &mut Criterion) {
    let content = load_fixture("large_markdown.md");

    let mut group = c.benchmark_group("chunking");

    group.bench_function("semantic", |b| {
        b.iter(|| chunking::semantic_chunk(black_box(&content), black_box(500)))
    });

    group.bench_function("heading", |b| {
        b.iter(|| chunking::heading_chunk(black_box(&content)))
    });

    group.bench_function("size", |b| {
        b.iter(|| chunking::size_chunk(black_box(&content), black_box(500)))
    });

    group.finish();
}

criterion_group!(benches, benchmark_html_conversion, benchmark_chunking);
criterion_main!(benches);
```

### 7.2 Performance Targets

| Operation                  | Target | Measurement           |
| -------------------------- | ------ | --------------------- |
| HTML → Markdown (50KB)     | <50ms  | Avg conversion time   |
| Semantic chunking (10KB)   | <20ms  | Avg chunking time     |
| Validate 100-page site     | <5s    | Total validation time |
| Generate 100 machine views | <10s   | Total generation time |
| Build 100-page site        | <15s   | Total build time      |

### 7.3 Performance Tests

```rust
// tests/performance/generation_perf_test.rs

#[test]
fn test_generation_performance() {
    let html = load_fixture("medium_page.html"); // 50KB

    let start = std::time::Instant::now();
    let _result = machine_view::from_html(&html, "test.html").unwrap();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 50,
        "Generation took {}ms, expected <50ms",
        duration.as_millis()
    );
}

#[test]
fn test_parallel_generation_scaling() {
    let temp_dir = TempDir::new().unwrap();
    generate_sample_pages(temp_dir.path(), 100);

    // Single-threaded
    let start = std::time::Instant::now();
    generate_all(temp_dir.path(), 1);
    let single_duration = start.elapsed();

    // Multi-threaded (8 cores)
    cleanup(temp_dir.path());
    generate_sample_pages(temp_dir.path(), 100);

    let start = std::time::Instant::now();
    generate_all(temp_dir.path(), 8);
    let multi_duration = start.elapsed();

    // Should be at least 3x faster with 8 cores
    let speedup = single_duration.as_secs_f64() / multi_duration.as_secs_f64();
    assert!(speedup >= 3.0, "Speedup was only {:.2}x", speedup);
}
```

---

## 8. Test Fixtures

### 8.1 Fixture Organization

```
tests/fixtures/
├── sample_sites/
│   ├── blog/
│   │   ├── index.html
│   │   ├── posts/
│   │   │   ├── post1.html
│   │   │   └── post2.html
│   │   └── llms.txt (old format)
│   │
│   ├── ecommerce/
│   │   ├── products/
│   │   │   ├── keyboard.html
│   │   │   └── mouse.html
│   │   ├── cart.html
│   │   └── checkout.html
│   │
│   └── docs/
│       ├── getting-started.html
│       ├── api/
│       │   ├── authentication.html
│       │   └── endpoints.html
│       └── guides/
│           └── advanced.html
│
├── html/
│   ├── minimal.html
│   ├── with_schema_org.html
│   ├── with_chunks.html
│   └── complex_layout.html
│
├── markdown/
│   ├── simple.md
│   ├── with_frontmatter.md
│   └── large_document.md
│
└── config/
    ├── minimal_config.yaml
    ├── full_config.yaml
    ├── ecommerce_template.yaml
    └── blog_template.yaml
```

### 8.2 Fixture Helper Functions

```rust
// tests/helpers/fixtures.rs

use std::path::{Path, PathBuf};

/// Get fixture path
pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Load fixture as string
pub fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name))
        .expect(&format!("Failed to load fixture: {}", name))
}

/// Copy fixture directory to temp location
pub fn copy_fixtures<P: AsRef<Path>>(fixture: &str, dest: P) {
    let src = fixture_path(fixture);
    copy_dir_all(&src, dest.as_ref()).unwrap();
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Generate sample pages for testing
pub fn generate_sample_pages<P: AsRef<Path>>(dir: P, count: usize) {
    let pages_dir = dir.as_ref().join("pages");
    std::fs::create_dir_all(&pages_dir).unwrap();

    for i in 0..count {
        let content = format!(
            r#"<html>
                <head><title>Page {}</title></head>
                <body>
                    <h1>Page {}</h1>
                    <p>This is test page number {}.</p>
                </body>
            </html>"#,
            i, i, i
        );

        std::fs::write(
            pages_dir.join(format!("page_{}.html", i)),
            content
        ).unwrap();
    }
}
```

---

## 9. CI/CD Integration

### 9.1 GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
          profile: minimal
          override: true
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Run unit tests
        run: cd cli && cargo test --lib

      - name: Run integration tests
        run: cd cli && cargo test --test '*'

      - name: Run doc tests
        run: cd cli && cargo test --doc

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cd cli && cargo tarpaulin --out Xml --output-dir coverage

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./cli/coverage/cobertura.xml
          fail_ci_if_error: true

  lint:
    name: Linting
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
          components: rustfmt, clippy

      - name: Check formatting
        run: cd cli && cargo fmt -- --check

      - name: Run clippy
        run: cd cli && cargo clippy -- -D warnings

  benchmark:
    name: Performance Benchmarks
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Run benchmarks
        run: cd cli && cargo bench --no-fail-fast

      - name: Store benchmark results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: cli/target/criterion/output.txt
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
```

### 9.2 Test Scripts

```bash
#!/bin/bash
# scripts/test.sh - Run all tests

set -e

echo "Running unit tests..."
cargo test --lib

echo "Running integration tests..."
cargo test --test '*'

echo "Running doc tests..."
cargo test --doc

echo "Running benchmarks..."
cargo bench --no-run

echo "Checking test coverage..."
cargo tarpaulin --out Html --output-dir coverage

echo "✓ All tests passed!"
```

```bash
#!/bin/bash
# scripts/test-watch.sh - Watch mode for development

cargo watch -x 'test --lib' -x 'test --test init_test'
```

---

## 10. Coverage Goals

### 10.1 Coverage Targets

| Component   | Target | Current | Status   |
| ----------- | ------ | ------- | -------- |
| **Overall** | 85%+   | 0%      | 🔴 To Do |
| Generators  | 90%+   | 0%      | 🔴 To Do |
| Parsers     | 85%+   | 0%      | 🔴 To Do |
| Validators  | 90%+   | 0%      | 🔴 To Do |
| Commands    | 85%+   | 0%      | 🔴 To Do |
| Utils       | 80%+   | 0%      | 🔴 To Do |

### 10.2 Coverage Report

```bash
# Generate coverage report
cd cli
cargo tarpaulin --out Html --output-dir coverage

# Open report
open coverage/index.html
```

### 10.3 Continuous Monitoring

- **Codecov Integration** - Automatic coverage reporting on PRs
- **Coverage Badges** - Display coverage in README
- **Trend Analysis** - Track coverage over time
- **Regression Prevention** - Block PRs that decrease coverage

---

## Conclusion

This comprehensive testing strategy ensures the ARW CLI achieves production-ready reliability through:

1. **Multiple Test Layers** - Unit, integration, and E2E tests
2. **High Coverage** - 85%+ overall, 90%+ for critical components
3. **Fast Feedback** - Unit tests in <5s, full suite in <2min
4. **CI/CD Integration** - Automated testing on all platforms
5. **Performance Monitoring** - Benchmarks and performance tests
6. **Quality Assurance** - Linting, formatting, and coverage tracking

By following this strategy, we ensure users can rely on the ARW CLI for their production implementations.

---

**Next Steps:**

1. Review and approve testing strategy
2. Begin implementing test infrastructure (Phase 1, Week 1)
3. Write unit tests for existing code
4. Setup CI/CD for automated testing
5. Achieve 85%+ coverage by end of Phase 2

**Related Documents:**

- CLI-EXPANSION-PLAN.md
- CLI-PUBLISHING-WORKFLOW.md
- CLI-STANDARDS-INTEGRATION.md
