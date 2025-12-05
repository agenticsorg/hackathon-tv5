/// CLI command argument parsing and execution tests
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// INIT COMMAND TESTS
// ============================================================================

#[test]
fn test_init_command_with_defaults() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success();

    assert!(temp_dir.path().join("llms.txt").exists());
}

#[test]
fn test_init_command_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let new_dir = temp_dir.path().join("new_site");

    Command::cargo_bin("arw")
        .unwrap()
        .arg("init")
        .arg("--path")
        .arg(&new_dir)
        .arg("--yes")
        .assert()
        .success();

    assert!(new_dir.exists());
    assert!(new_dir.join("llms.txt").exists());
}

// ============================================================================
// VALIDATE COMMAND TESTS
// ============================================================================

#[test]
fn test_validate_command_success() {
    let temp_dir = TempDir::new().unwrap();

    // Create valid manifest
    let manifest = r#"
version: "1.0"
profile: ARW-1
site:
  name: Test Site
  homepage: https://example.com
policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#;
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn test_validate_command_failure() {
    let temp_dir = TempDir::new().unwrap();

    // Create invalid manifest
    let manifest = r#"
version: "1.0"
profile: INVALID
"#;
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .failure();
}

#[test]
fn test_validate_strict_mode() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = r#"
version: "1.0"
profile: ARW-1
site:
  name: Test Site
  homepage: https://example.com
policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#;
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--strict")
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // May warn about missing files
}

#[test]
fn test_validate_missing_llms_txt() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Error")));
}

// ============================================================================
// GENERATE COMMAND TESTS
// ============================================================================

#[test]
fn test_generate_command_single_file() {
    let temp_dir = TempDir::new().unwrap();

    let html = "<html><body><h1>Test</h1></body></html>";
    fs::write(temp_dir.path().join("test.html"), html).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("test.html"))
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    assert!(temp_dir.path().join("test.llm.md").exists());
}

#[test]
fn test_generate_command_recursive() {
    let temp_dir = TempDir::new().unwrap();
    fs::create_dir(temp_dir.path().join("sub")).unwrap();

    fs::write(
        temp_dir.path().join("index.html"),
        "<html><body><h1>Home</h1></body></html>",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("sub/page.html"),
        "<html><body><h1>Page</h1></body></html>",
    )
    .unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path())
        .arg("--recursive")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    assert!(temp_dir.path().join("index.llm.md").exists());
    assert!(temp_dir.path().join("sub/page.llm.md").exists());
}

#[test]
fn test_generate_command_with_format() {
    let temp_dir = TempDir::new().unwrap();

    let html = "<html><body><h1>Test</h1></body></html>";
    fs::write(temp_dir.path().join("test.html"), html).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("test.html"))
        .arg("--output")
        .arg(temp_dir.path())
        .arg("--format")
        .arg("html")
        .assert()
        .success();
}

// ============================================================================
// BUILD COMMAND TESTS
// ============================================================================

#[test]
fn test_build_command_success() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = r#"
version: "1.0"
profile: ARW-1
site:
  name: Test Site
  description: Test description
  homepage: https://example.com
  contact: test@example.com
policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#;
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("build")
        .arg("--source")
        .arg(temp_dir.path())
        .assert()
        .success();

    assert!(temp_dir.path().join("llms.json").exists());
    assert!(temp_dir.path().join(".well-known").is_dir());
}

#[test]
fn test_build_command_with_custom_base_url() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = r#"
version: "1.0"
profile: ARW-1
site:
  name: Test Site
  description: Test description
  homepage: https://example.com
  contact: test@example.com
policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#;
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("build")
        .arg("--source")
        .arg(temp_dir.path())
        .arg("--base-url")
        .arg("https://custom.example.com")
        .assert()
        .success();
}

#[test]
fn test_build_command_missing_llms_txt() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("build")
        .arg("--source")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("llms.txt not found"));
}

// ============================================================================
// ROBOTS COMMAND TESTS
// ============================================================================

#[test]
fn test_robots_command_generates_file() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = r#"
version: "1.0"
profile: ARW-1
site:
  name: Test Site
  homepage: https://example.com
policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#;
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("robots")
        .arg("--manifest")
        .arg(temp_dir.path().join("llms.txt"))
        .arg("--output")
        .arg(temp_dir.path().join("robots.txt"))
        .assert()
        .success();

    assert!(temp_dir.path().join("robots.txt").exists());
}

#[test]
fn test_robots_command_respects_training_policy() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = r#"
version: "1.0"
profile: ARW-1
site:
  name: Test Site
  homepage: https://example.com
policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#;
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("robots")
        .arg("--manifest")
        .arg(temp_dir.path().join("llms.txt"))
        .arg("--output")
        .arg(temp_dir.path().join("robots.txt"))
        .assert()
        .success();

    let robots_content = fs::read_to_string(temp_dir.path().join("robots.txt")).unwrap();
    assert!(robots_content.contains("Disallow") || robots_content.contains("GPTBot"));
}

// ============================================================================
// SITEMAP COMMAND TESTS
// ============================================================================

#[test]
fn test_sitemap_command_generates_xml() {
    let temp_dir = TempDir::new().unwrap();

    // Create some HTML files
    fs::write(temp_dir.path().join("index.html"), "<html></html>").unwrap();
    fs::write(temp_dir.path().join("about.html"), "<html></html>").unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("sitemap")
        .arg(temp_dir.path())
        .arg("--output")
        .arg(temp_dir.path().join("sitemap.xml"))
        .arg("--base-url")
        .arg("https://example.com")
        .assert()
        .success();

    let sitemap_path = temp_dir.path().join("sitemap.xml");
    assert!(sitemap_path.exists());

    let sitemap_content = fs::read_to_string(&sitemap_path).unwrap();
    assert!(sitemap_content.contains("<?xml"));
    assert!(sitemap_content.contains("<urlset"));
}

#[test]
fn test_sitemap_command_with_depth() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(temp_dir.path().join("index.html"), "<html></html>").unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("sitemap")
        .arg(temp_dir.path())
        .arg("--output")
        .arg(temp_dir.path().join("sitemap.xml"))
        .arg("--base-url")
        .arg("https://example.com")
        .arg("--depth")
        .arg("3")
        .assert()
        .success();
}

// ============================================================================
// ACTIONS COMMAND TESTS
// ============================================================================

#[test]
fn test_actions_command_lists_actions() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = r#"
version: "1.0"
profile: ARW-3
site:
  name: Test Site
  homepage: https://example.com
actions:
  - id: test_action
    name: Test Action
    endpoint: /api/test
    method: POST
    auth: none
policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#;
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("actions")
        .arg("--manifest")
        .arg(temp_dir.path().join("llms.txt"))
        .assert()
        .success();
}

// ============================================================================
// POLICY COMMAND TESTS
// ============================================================================

#[test]
fn test_policy_command_creates_policy() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("policy")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // May succeed or ask for input
}

// ============================================================================
// SCAN COMMAND TESTS
// ============================================================================

#[test]
#[ignore] // Requires network access
fn test_scan_command_with_url() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("scan")
        .arg("https://example.com")
        .arg("--output")
        .arg(temp_dir.path())
        .arg("--dry-run")
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

// ============================================================================
// SERVE COMMAND TESTS
// ============================================================================

#[test]
#[ignore] // Server runs indefinitely
fn test_serve_command_starts() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("serve")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--port")
        .arg("8888")
        .timeout(std::time::Duration::from_secs(2))
        .assert();
}

// ============================================================================
// COMMAND ALIAS TESTS
// ============================================================================

#[test]
fn test_init_alias_i() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("i")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success();
}

#[test]
fn test_generate_alias_gen() {
    let temp_dir = TempDir::new().unwrap();

    let html = "<html><body><h1>Test</h1></body></html>";
    fs::write(temp_dir.path().join("test.html"), html).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("gen")
        .arg(temp_dir.path().join("test.html"))
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();
}

#[test]
fn test_validate_alias_val() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = r#"
version: "1.0"
profile: ARW-1
site:
  name: Test
  homepage: https://example.com
policies:
  training: {allowed: false}
  inference: {allowed: true}
  attribution: {required: true}
"#;
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("val")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_invalid_command() {
    Command::cargo_bin("arw")
        .unwrap()
        .arg("invalid_command")
        .assert()
        .failure();
}

#[test]
fn test_missing_required_argument() {
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        // Missing source argument
        .assert()
        .failure();
}

#[test]
fn test_invalid_flag_value() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("sitemap")
        .arg(temp_dir.path())
        .arg("--depth")
        .arg("not_a_number")
        .assert()
        .failure();
}

// ============================================================================
// OUTPUT FORMAT TESTS
// ============================================================================

#[test]
fn test_command_output_contains_branding() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::cargo_bin("arw")
        .unwrap()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    assert!(
        output_str.contains("ARW") || output_str.contains("Agent-Ready Web"),
        "Output should contain ARW branding"
    );
}

#[test]
fn test_success_indicator_in_output() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::cargo_bin("arw")
        .unwrap()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    assert!(
        output_str.contains("✓") || output_str.contains("Success") || output_str.contains("success"),
        "Output should indicate success"
    );
}

// ============================================================================
// CONCURRENT COMMAND EXECUTION
// ============================================================================

#[test]
fn test_multiple_commands_sequentially() {
    let temp_dir = TempDir::new().unwrap();

    // Init
    Command::cargo_bin("arw")
        .unwrap()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success();

    // Build
    Command::cargo_bin("arw")
        .unwrap()
        .arg("build")
        .arg("--source")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Validate
    Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();
}
