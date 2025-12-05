/// Regression test: Version should be accepted as string
use std::fs;

mod common;
mod helpers;

use common::*;
use helpers::*;

#[test]
fn test_version_as_string() {
    setup_test_env();
    let temp_dir = create_temp_dir();

    // Version as string (correct format)
    let manifest = r#"version: "1.0"
profile: ARW-1

site:
  name: "Test"
  homepage: "https://example.com"

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
fn test_version_without_quotes() {
    setup_test_env();
    let temp_dir = create_temp_dir();

    // Version without quotes (YAML will parse as number)
    let manifest = r#"version: 1.0
profile: ARW-1

site:
  name: "Test"
  homepage: "https://example.com"

policies:
  training:
    allowed: false
"#;

    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    // Should still work (parser should handle both)
    let output = run_cli_success(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&output, "Success");
}

#[test]
fn test_llms_json_preserves_version_format() {
    setup_test_env();
    let temp_dir = create_temp_dir();

    let manifest = r#"version: "1.0"
profile: ARW-1
site:
  name: "Test"
  homepage: "https://example.com"
policies:
  training:
    allowed: false
"#;

    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    // Check llms.json
    let json_content = fs::read_to_string(temp_dir.path().join("llms.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_content).unwrap();

    // Version should be a string
    assert!(json["version"].is_string());
    assert_eq!(json["version"], "1.0");
}
