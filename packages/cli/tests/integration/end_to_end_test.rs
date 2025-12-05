/// End-to-end integration tests
/// Tests complete workflows from initialization to validation
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// INIT → VALIDATE WORKFLOW
// ============================================================================

#[test]
fn test_init_then_validate_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize ARW structure
    let mut cmd = Command::cargo_bin("arw").unwrap();
    cmd.arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success();

    // Verify llms.txt was created
    assert!(temp_dir.path().join("llms.txt").exists());

    // Validate the created structure
    let mut cmd = Command::cargo_bin("arw").unwrap();
    cmd.arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();
}

// ============================================================================
// INIT → BUILD → VALIDATE WORKFLOW
// ============================================================================

#[test]
fn test_init_build_validate_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize
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

    // Verify build artifacts
    assert!(temp_dir.path().join("llms.json").exists());
    assert!(temp_dir.path().join(".well-known/arw-manifest.json").exists());
    assert!(temp_dir.path().join(".well-known/arw-policies.json").exists());

    // Validate with strict mode
    Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--strict")
        .assert()
        .success();
}

// ============================================================================
// GENERATE → VALIDATE WORKFLOW
// ============================================================================

#[test]
fn test_generate_machine_view_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Create sample HTML file
    let html_content = r#"
<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
    <h1>Welcome</h1>
    <p>This is test content.</p>
</body>
</html>
"#;
    fs::write(temp_dir.path().join("index.html"), html_content).unwrap();

    // Generate machine view
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("index.html"))
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify machine view was created
    let md_file = temp_dir.path().join("index.llm.md");
    assert!(md_file.exists(), "Machine view should be created");

    let md_content = fs::read_to_string(&md_file).unwrap();
    assert!(md_content.contains("Welcome"), "Should contain content");
}

// ============================================================================
// ROBOTS GENERATION WORKFLOW
// ============================================================================

#[test]
fn test_robots_generation_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Create minimal llms.txt
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

    // Generate robots.txt
    Command::cargo_bin("arw")
        .unwrap()
        .arg("robots")
        .arg("--manifest")
        .arg(temp_dir.path().join("llms.txt"))
        .arg("--output")
        .arg(temp_dir.path().join("robots.txt"))
        .assert()
        .success();

    // Verify robots.txt exists and has correct content
    let robots_path = temp_dir.path().join("robots.txt");
    assert!(robots_path.exists());

    let robots_content = fs::read_to_string(&robots_path).unwrap();
    assert!(robots_content.contains("User-agent:"));
    assert!(robots_content.contains("llms.txt"));
}

// ============================================================================
// FULL SITE SETUP WORKFLOW
// ============================================================================

#[test]
fn test_complete_site_setup_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // 1. Initialize
    Command::cargo_bin("arw")
        .unwrap()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success();

    // 2. Create HTML files
    fs::write(
        temp_dir.path().join("index.html"),
        "<html><body><h1>Home</h1></body></html>",
    )
    .unwrap();

    // 3. Generate machine views
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("index.html"))
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    // 4. Build all ARW files
    Command::cargo_bin("arw")
        .unwrap()
        .arg("build")
        .arg("--source")
        .arg(temp_dir.path())
        .assert()
        .success();

    // 5. Generate robots.txt
    Command::cargo_bin("arw")
        .unwrap()
        .arg("robots")
        .arg("--manifest")
        .arg(temp_dir.path().join("llms.txt"))
        .arg("--output")
        .arg(temp_dir.path().join("robots.txt"))
        .assert()
        .success();

    // 6. Final validation
    Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--strict")
        .assert()
        .success();

    // Verify all expected files exist
    assert!(temp_dir.path().join("llms.txt").exists());
    assert!(temp_dir.path().join("llms.json").exists());
    assert!(temp_dir.path().join("robots.txt").exists());
    assert!(temp_dir.path().join("sitemap.xml").exists());
    assert!(temp_dir.path().join("index.llm.md").exists());
    assert!(temp_dir.path().join(".well-known").is_dir());
}

// ============================================================================
// ERROR RECOVERY WORKFLOW
// ============================================================================

#[test]
fn test_validation_failure_then_fix_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Create invalid llms.txt (missing required fields)
    let invalid_manifest = r#"
version: "1.0"
profile: INVALID
"#;
    fs::write(temp_dir.path().join("llms.txt"), invalid_manifest).unwrap();

    // Validation should fail
    Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .failure();

    // Fix the manifest
    let valid_manifest = r#"
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
    fs::write(temp_dir.path().join("llms.txt"), valid_manifest).unwrap();

    // Validation should now succeed
    Command::cargo_bin("arw")
        .unwrap()
        .arg("validate")
        .arg("--path")
        .arg(temp_dir.path())
        .assert()
        .success();
}

// ============================================================================
// WATCH MODE SIMULATION (if available)
// ============================================================================

#[test]
#[ignore] // Ignore by default as watch mode runs indefinitely
fn test_watch_mode_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize
    Command::cargo_bin("arw")
        .unwrap()
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success();

    // Watch command (would run indefinitely, so we just test it starts)
    // This is a smoke test to ensure the command doesn't crash immediately
    let mut cmd = Command::cargo_bin("arw").unwrap();
    cmd.arg("watch")
        .arg("--path")
        .arg(temp_dir.path())
        .timeout(std::time::Duration::from_secs(2));

    // We expect a timeout, which means watch started successfully
    let result = cmd.assert();
    // Either succeeds (unlikely in 2 seconds) or times out (expected)
}

// ============================================================================
// RECURSIVE GENERATION WORKFLOW
// ============================================================================

#[test]
fn test_recursive_generation_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Create directory structure
    fs::create_dir(temp_dir.path().join("pages")).unwrap();
    fs::create_dir(temp_dir.path().join("pages/blog")).unwrap();

    // Create HTML files in different directories
    fs::write(
        temp_dir.path().join("index.html"),
        "<html><body><h1>Home</h1></body></html>",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("pages/about.html"),
        "<html><body><h1>About</h1></body></html>",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("pages/blog/post1.html"),
        "<html><body><h1>Post 1</h1></body></html>",
    )
    .unwrap();

    // Generate machine views recursively
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path())
        .arg("--recursive")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Verify all machine views were created
    assert!(temp_dir.path().join("index.llm.md").exists());
    assert!(temp_dir.path().join("pages/about.llm.md").exists());
    assert!(temp_dir.path().join("pages/blog/post1.llm.md").exists());
}

// ============================================================================
// VERSION AND HELP COMMANDS
// ============================================================================

#[test]
fn test_version_command() {
    Command::cargo_bin("arw")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("arw"));
}

#[test]
fn test_help_command() {
    Command::cargo_bin("arw")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Agent-Ready Web"));
}

#[test]
fn test_command_aliases() {
    let temp_dir = TempDir::new().unwrap();

    // Test init alias
    Command::cargo_bin("arw")
        .unwrap()
        .arg("i")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success();

    assert!(temp_dir.path().join("llms.txt").exists());
}

// ============================================================================
// QUIET AND VERBOSE MODES
// ============================================================================

#[test]
fn test_quiet_mode() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::cargo_bin("arw")
        .unwrap()
        .arg("--quiet")
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
    // Quiet mode should suppress most output
    assert!(
        output_str.len() < 100,
        "Quiet mode should have minimal output"
    );
}

#[test]
fn test_verbose_mode() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("arw")
        .unwrap()
        .arg("--verbose")
        .arg("init")
        .arg("--path")
        .arg(temp_dir.path())
        .arg("--yes")
        .assert()
        .success();
    // Just verify it doesn't crash in verbose mode
}

// ============================================================================
// FORCE FLAG TESTS
// ============================================================================

#[test]
fn test_force_overwrite_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Create HTML and machine view
    fs::write(
        temp_dir.path().join("index.html"),
        "<html><body><h1>Version 1</h1></body></html>",
    )
    .unwrap();

    // Generate first time
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("index.html"))
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    let md_path = temp_dir.path().join("index.llm.md");
    let original_content = fs::read_to_string(&md_path).unwrap();

    // Update HTML
    fs::write(
        temp_dir.path().join("index.html"),
        "<html><body><h1>Version 2</h1></body></html>",
    )
    .unwrap();

    // Generate again with force flag
    Command::cargo_bin("arw")
        .unwrap()
        .arg("generate")
        .arg(temp_dir.path().join("index.html"))
        .arg("--output")
        .arg(temp_dir.path())
        .arg("--force")
        .assert()
        .success();

    let new_content = fs::read_to_string(&md_path).unwrap();
    assert_ne!(
        original_content, new_content,
        "Content should be updated with force flag"
    );
    assert!(new_content.contains("Version 2"));
}
