/// Regression test: Contact field should be optional
use std::fs;

mod common;
mod helpers;

use common::*;
use helpers::*;

#[test]
fn test_contact_field_is_optional() {
    setup_test_env();
    let temp_dir = create_temp_dir();

    // Create manifest WITHOUT contact field
    let manifest = r#"version: "1.0"
profile: ARW-1

site:
  name: "Test Site"
  homepage: "https://example.com"
  # No contact field

policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#;

    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    // Should validate successfully
    let output = run_cli_success(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&output, "Success");
}

#[test]
fn test_contact_field_when_present_is_validated() {
    setup_test_env();
    let temp_dir = create_temp_dir();

    // Create manifest with INVALID contact
    let manifest = r#"version: "1.0"
profile: ARW-1

site:
  name: "Test Site"
  homepage: "https://example.com"
  contact: "not-an-email"

policies:
  training:
    allowed: false
"#;

    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    // Should fail validation due to invalid email
    let (_stdout, stderr) = run_cli_failure(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&stderr, "contact");
}

#[test]
fn test_contact_field_with_valid_email() {
    setup_test_env();
    let temp_dir = create_temp_dir();

    let manifest = r#"version: "1.0"
profile: ARW-1

site:
  name: "Test Site"
  homepage: "https://example.com"
  contact: "valid@example.com"

policies:
  training:
    allowed: false
"#;

    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    let output = run_cli_success(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&output, "Success");
}

#[test]
fn test_build_works_without_contact() {
    setup_test_env();
    let temp_dir = create_temp_dir();

    let manifest = r#"version: "1.0"
profile: ARW-1

site:
  name: "Test Site"
  homepage: "https://example.com"

policies:
  training:
    allowed: false
"#;

    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    let output = run_cli_success(
        &["build", "--source", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&output, "Build complete");

    // Verify files were created
    assert!(temp_dir.path().join("llms.json").exists());
}
