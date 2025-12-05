/// Integration tests for TOON CLI workflow
/// Tests the complete TOON generation and validation pipeline
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// SINGLE FILE GENERATION TESTS
// ============================================================================

#[test]
fn test_generate_toon_single_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create test HTML file
    let html_content = r#"
<!DOCTYPE html>
<html>
<head><title>TOON Test</title></head>
<body>
    <h1>TOON Format Test</h1>
    <p>Testing single file TOON generation.</p>
</body>
</html>
"#;
    let html_path = temp_dir.path().join("test.html");
    fs::write(&html_path, html_content).unwrap();

    // Generate TOON format using CLI
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(&html_path)
        .arg("-F")
        .arg("toon")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify .llm.toon file was created
    let toon_path = temp_dir.path().join("test.llm.toon");
    assert!(toon_path.exists(), "TOON file should be created");

    // Verify content
    let toon_content = fs::read_to_string(&toon_path).unwrap();
    assert!(
        toon_content.contains("# TOON Format Test"),
        "TOON should contain heading"
    );
    assert!(
        toon_content.contains("Testing single file"),
        "TOON should contain content"
    );
}

#[test]
fn test_generate_toon_with_format_flag() {
    let temp_dir = TempDir::new().unwrap();

    let html_content = "<html><body><h1>Test</h1></body></html>";
    let html_path = temp_dir.path().join("input.html");
    fs::write(&html_path, html_content).unwrap();

    // Test with --format flag
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(&html_path)
        .arg("--format")
        .arg("toon")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    assert!(temp_dir.path().join("input.llm.toon").exists());
}

#[test]
fn test_generate_toon_format_alias() {
    let temp_dir = TempDir::new().unwrap();

    let html_content = "<html><body><h1>Test</h1></body></html>";
    let html_path = temp_dir.path().join("input.html");
    fs::write(&html_path, html_content).unwrap();

    // Test with short -F flag
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(&html_path)
        .arg("-F")
        .arg("toon")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    assert!(temp_dir.path().join("input.llm.toon").exists());
}

// ============================================================================
// DIRECTORY GENERATION TESTS
// ============================================================================

#[test]
fn test_generate_toon_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple HTML files
    fs::create_dir(temp_dir.path().join("pages")).unwrap();

    fs::write(
        temp_dir.path().join("index.html"),
        "<html><body><h1>Home</h1></body></html>",
    ).unwrap();

    fs::write(
        temp_dir.path().join("about.html"),
        "<html><body><h1>About</h1></body></html>",
    ).unwrap();

    fs::write(
        temp_dir.path().join("pages/blog.html"),
        "<html><body><h1>Blog</h1></body></html>",
    ).unwrap();

    // Generate TOON files recursively
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path())
        .arg("-r")
        .arg("-F")
        .arg("toon")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify all TOON files were created
    assert!(temp_dir.path().join("index.llm.toon").exists());
    assert!(temp_dir.path().join("about.llm.toon").exists());
    assert!(temp_dir.path().join("pages/blog.llm.toon").exists());
}

#[test]
fn test_generate_toon_recursive_flag() {
    let temp_dir = TempDir::new().unwrap();

    // Create nested structure
    fs::create_dir_all(temp_dir.path().join("a/b/c")).unwrap();

    fs::write(
        temp_dir.path().join("a/test1.html"),
        "<html><body><h1>Level 1</h1></body></html>",
    ).unwrap();

    fs::write(
        temp_dir.path().join("a/b/test2.html"),
        "<html><body><h1>Level 2</h1></body></html>",
    ).unwrap();

    fs::write(
        temp_dir.path().join("a/b/c/test3.html"),
        "<html><body><h1>Level 3</h1></body></html>",
    ).unwrap();

    // Generate with --recursive
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("a"))
        .arg("--recursive")
        .arg("--format")
        .arg("toon")
        .arg("--output")
        .arg(temp_dir.path().join("a"))
        .assert()
        .success();

    // Verify all levels
    assert!(temp_dir.path().join("a/test1.llm.toon").exists());
    assert!(temp_dir.path().join("a/b/test2.llm.toon").exists());
    assert!(temp_dir.path().join("a/b/c/test3.llm.toon").exists());
}

// ============================================================================
// VALIDATION TESTS
// ============================================================================

#[test]
fn test_validate_toon_manifest() {
    let temp_dir = TempDir::new().unwrap();

    // Create llms.toon manifest
    let manifest = r#"
version: "1.0"
profile: ARW-1
format: TOON
site:
  name: Test Site
  homepage: https://example.com
content:
  - url: /index
    machine_view: /index.llm.toon
  - url: /about
    machine_view: /about.llm.toon
"#;
    fs::write(temp_dir.path().join("llms.toon"), manifest).unwrap();

    // Create referenced TOON files
    fs::write(
        temp_dir.path().join("index.llm.toon"),
        "# Home\nContent",
    ).unwrap();

    fs::write(
        temp_dir.path().join("about.llm.toon"),
        "# About\nContent",
    ).unwrap();

    // Validate
    Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();
}

#[test]
fn test_validate_toon_missing_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create manifest referencing non-existent files
    let manifest = r#"
version: "1.0"
profile: ARW-1
format: TOON
site:
  name: Test Site
  homepage: https://example.com
content:
  - url: /missing
    machine_view: /missing.llm.toon
"#;
    fs::write(temp_dir.path().join("llms.toon"), manifest).unwrap();

    // Validation should fail or warn
    Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .failure();
}

// ============================================================================
// OUTPUT FORMAT TESTS
// ============================================================================

#[test]
fn test_toon_output_format() {
    let temp_dir = TempDir::new().unwrap();

    let html = r#"
<html>
<body>
    <h1>Test</h1>
    <p>Paragraph 1</p>
    <h2>Section</h2>
    <p>Paragraph 2</p>
</body>
</html>
"#;
    fs::write(temp_dir.path().join("test.html"), html).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("test.html"))
        .arg("-F")
        .arg("toon")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    let toon_path = temp_dir.path().join("test.llm.toon");
    let content = fs::read_to_string(&toon_path).unwrap();

    // Verify TOON format structure
    assert!(content.contains("# Test"), "Should have h1");
    assert!(content.contains("## Section"), "Should have h2");
    assert!(content.contains("Paragraph 1"), "Should have content");
    assert!(content.contains("Paragraph 2"), "Should have content");

    // Verify it's clean markdown without HTML
    assert!(!content.contains("<html>"), "Should not contain HTML tags");
    assert!(!content.contains("<body>"), "Should not contain HTML tags");
}

#[test]
fn test_toon_with_chunks() {
    let temp_dir = TempDir::new().unwrap();

    let html = r#"
<html>
<body>
    <h1>Introduction</h1>
    <p>Intro content</p>
    <h2>Details</h2>
    <p>Detail content</p>
</body>
</html>
"#;
    fs::write(temp_dir.path().join("test.html"), html).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("test.html"))
        .arg("-F")
        .arg("toon")
        .arg("--chunks")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    let toon_path = temp_dir.path().join("test.llm.toon");
    let content = fs::read_to_string(&toon_path).unwrap();

    // Should include chunk markers
    assert!(
        content.contains("<!-- chunk:") || content.contains("chunk:"),
        "Should include chunk markers"
    );
}

// ============================================================================
// BACKWARD COMPATIBILITY TESTS
// ============================================================================

#[test]
fn test_markdown_still_works() {
    let temp_dir = TempDir::new().unwrap();

    let html = "<html><body><h1>Test</h1></body></html>";
    fs::write(temp_dir.path().join("test.html"), html).unwrap();

    // Default should still generate .llm.md
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("test.html"))
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    assert!(
        temp_dir.path().join("test.llm.md").exists(),
        "Default should still create .llm.md files"
    );
}

#[test]
fn test_both_formats_simultaneously() {
    let temp_dir = TempDir::new().unwrap();

    let html = "<html><body><h1>Test</h1></body></html>";
    fs::write(temp_dir.path().join("test.html"), html).unwrap();

    // Generate markdown
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("test.html"))
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Generate TOON
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("test.html"))
        .arg("-F")
        .arg("toon")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Both should exist
    assert!(temp_dir.path().join("test.llm.md").exists());
    assert!(temp_dir.path().join("test.llm.toon").exists());
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_generate_toon_nonexistent_file() {
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg("/nonexistent/file.html")
        .arg("-F")
        .arg("toon")
        .assert()
        .failure();
}

#[test]
fn test_generate_toon_invalid_format() {
    let temp_dir = TempDir::new().unwrap();

    let html = "<html><body><h1>Test</h1></body></html>";
    fs::write(temp_dir.path().join("test.html"), html).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("test.html"))
        .arg("-F")
        .arg("invalid_format")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid").or(predicate::str::contains("format")));
}

// ============================================================================
// FORCE OVERWRITE TESTS
// ============================================================================

#[test]
fn test_toon_force_overwrite() {
    let temp_dir = TempDir::new().unwrap();

    let html_v1 = "<html><body><h1>Version 1</h1></body></html>";
    fs::write(temp_dir.path().join("test.html"), html_v1).unwrap();

    // Generate first time
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("test.html"))
        .arg("-F")
        .arg("toon")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    let original = fs::read_to_string(temp_dir.path().join("test.llm.toon")).unwrap();

    // Update HTML
    let html_v2 = "<html><body><h1>Version 2</h1></body></html>";
    fs::write(temp_dir.path().join("test.html"), html_v2).unwrap();

    // Generate again with force
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("test.html"))
        .arg("-F")
        .arg("toon")
        .arg("--force")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    let updated = fs::read_to_string(temp_dir.path().join("test.llm.toon")).unwrap();

    assert_ne!(original, updated, "Content should be updated");
    assert!(updated.contains("Version 2"), "Should have new content");
}

// ============================================================================
// COMPLETE WORKFLOW TESTS
// ============================================================================

#[test]
fn test_complete_toon_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // 1. Create HTML files
    fs::write(
        temp_dir.path().join("index.html"),
        "<html><body><h1>Home</h1><p>Welcome</p></body></html>",
    ).unwrap();

    fs::write(
        temp_dir.path().join("about.html"),
        "<html><body><h1>About</h1><p>Info</p></body></html>",
    ).unwrap();

    // 2. Generate TOON files
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path())
        .arg("-r")
        .arg("-F")
        .arg("toon")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    // 3. Create TOON manifest
    let manifest = r#"
version: "1.0"
profile: ARW-1
format: TOON
site:
  name: Test Site
  homepage: https://example.com
content:
  - url: /
    machine_view: /index.llm.toon
  - url: /about
    machine_view: /about.llm.toon
"#;
    fs::write(temp_dir.path().join("llms.toon"), manifest).unwrap();

    // 4. Validate everything
    Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();

    // 5. Verify all files exist and have content
    assert!(temp_dir.path().join("index.llm.toon").exists());
    assert!(temp_dir.path().join("about.llm.toon").exists());
    assert!(temp_dir.path().join("llms.toon").exists());

    let index_content = fs::read_to_string(temp_dir.path().join("index.llm.toon")).unwrap();
    assert!(index_content.contains("# Home"));
    assert!(index_content.contains("Welcome"));
}

#[test]
fn test_toon_with_sitemap() {
    let temp_dir = TempDir::new().unwrap();

    // Generate TOON files
    fs::write(
        temp_dir.path().join("page.html"),
        "<html><body><h1>Page</h1></body></html>",
    ).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("page.html"))
        .arg("-F")
        .arg("toon")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Generate sitemap that references TOON files
    // This verifies TOON integrates with existing ARW infrastructure
    let sitemap = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <url>
        <loc>https://example.com/page</loc>
        <lastmod>2024-01-01</lastmod>
    </url>
</urlset>"#;
    fs::write(temp_dir.path().join("sitemap.xml"), sitemap).unwrap();

    assert!(temp_dir.path().join("page.llm.toon").exists());
    assert!(temp_dir.path().join("sitemap.xml").exists());
}
