/// End-to-end tests for the validate workflow
mod common;
mod helpers;

use common::*;
use helpers::*;
use std::fs;

#[test]
fn test_validate_minimal_valid_manifest() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    let output = run_cli_success(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&output, "Success");
    assert_output_contains(&output, "llms.txt is valid");
}

#[test]
fn test_validate_complete_manifest() {
    setup_test_env();
    let temp_dir = create_test_site(&create_complete_llms_txt());

    let output = run_cli_success(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&output, "llms.txt is valid");
}

#[test]
fn test_validate_missing_llms_txt() {
    setup_test_env();
    let temp_dir = create_temp_dir();

    let (_stdout, stderr) = run_cli_failure(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&stderr, "llms.txt not found");
}

#[test]
fn test_validate_invalid_manifest_missing_version() {
    setup_test_env();
    let temp_dir = create_test_site(&create_invalid_llms_txt_missing_version());

    let (_stdout, stderr) = run_cli_failure(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&stderr, "validation errors");
}

#[test]
fn test_validate_invalid_profile() {
    setup_test_env();
    let temp_dir = create_test_site(&create_invalid_llms_txt_wrong_profile());

    let (_stdout, stderr) = run_cli_failure(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&stderr, "profile");
}

#[test]
fn test_validate_with_llms_json() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    // Create llms.json
    let json_content = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": { "allowed": false },
            "inference": { "allowed": true },
            "attribution": { "required": true }
        }
    });

    fs::write(
        temp_dir.path().join("llms.json"),
        serde_json::to_string_pretty(&json_content).unwrap(),
    )
    .unwrap();

    let output = run_cli_success(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&output, "llms.json found");
    assert_output_contains(&output, "valid JSON");
}

#[test]
fn test_validate_with_well_known_files() {
    setup_test_env();
    let temp_dir = create_complete_test_site();

    // Create .well-known files
    let well_known = temp_dir.path().join(".well-known");
    fs::create_dir_all(&well_known).unwrap();

    fs::write(
        well_known.join("arw-manifest.json"),
        r#"{"version": "1.0"}"#,
    )
    .unwrap();

    fs::write(
        well_known.join("arw-policies.json"),
        r#"{"training": {"allowed": false}}"#,
    )
    .unwrap();

    let output = run_cli_success(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&output, "arw-manifest.json found");
    assert_output_contains(&output, "arw-policies.json found");
}

#[test]
fn test_validate_strict_mode_requires_robots_txt() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    let (_stdout, stderr) = run_cli_failure(
        &[
            "validate",
            "--path",
            temp_dir.path().to_str().unwrap(),
            "--strict",
        ],
        None,
    );

    assert_output_contains(&stderr, "robots.txt");
}

#[test]
fn test_validate_with_robots_txt() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    fs::write(temp_dir.path().join("robots.txt"), create_robots_txt()).unwrap();

    let output = run_cli_success(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&output, "robots.txt found");
    assert_output_contains(&output, "ARW discovery hints");
}

#[test]
fn test_validate_different_arw_profiles() {
    setup_test_env();

    // Test ARW-1 (Basic)
    let arw1_content = r#"version: "1.0"
profile: ARW-1
site:
  name: "Test"
  homepage: "https://example.com"
policies:
  training:
    allowed: false
"#;
    let temp_dir1 = create_test_site(arw1_content);
    let output1 = run_cli_success(
        &["validate", "--path", temp_dir1.path().to_str().unwrap()],
        None,
    );
    assert_output_contains(&output1, "Success");

    // Test ARW-2 (Content)
    let arw2_content = r#"version: "1.0"
profile: ARW-2
site:
  name: "Test"
  homepage: "https://example.com"
content:
  - url: "/"
    machine_view: "/index.llm.md"
    purpose: "homepage"
policies:
  training:
    allowed: false
"#;
    let temp_dir2 = create_test_site(arw2_content);
    let output2 = run_cli_success(
        &["validate", "--path", temp_dir2.path().to_str().unwrap()],
        None,
    );
    assert_output_contains(&output2, "Success");
}

#[test]
fn test_validate_missing_required_content_fields() {
    setup_test_env();
    let content = r#"version: "1.0"
profile: ARW-2
site:
  name: "Test"
  homepage: "https://example.com"
content:
  - url: "/"
    # Missing machine_view
policies:
  training:
    allowed: false
"#;
    let temp_dir = create_test_site(content);

    let (_stdout, stderr) = run_cli_failure(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&stderr, "machine_view");
}

#[test]
fn test_validate_invalid_url_format() {
    setup_test_env();
    let content = r#"version: "1.0"
profile: ARW-1
site:
  name: "Test"
  homepage: "not-a-valid-url"
policies:
  training:
    allowed: false
"#;
    let temp_dir = create_test_site(content);

    let (_stdout, stderr) = run_cli_failure(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&stderr, "homepage");
}

#[test]
fn test_validate_invalid_email_format() {
    setup_test_env();
    let content = r#"version: "1.0"
profile: ARW-1
site:
  name: "Test"
  homepage: "https://example.com"
  contact: "not-an-email"
policies:
  training:
    allowed: false
"#;
    let temp_dir = create_test_site(content);

    let (_stdout, stderr) = run_cli_failure(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&stderr, "contact");
}

#[test]
fn test_validate_verbose_output() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    let output = run_cli_success(
        &[
            "--verbose",
            "validate",
            "--path",
            temp_dir.path().to_str().unwrap(),
        ],
        None,
    );

    assert_output_contains(&output, "Validating");
}

#[test]
fn test_validate_quiet_mode() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    let output = run_cli_success(
        &[
            "--quiet",
            "validate",
            "--path",
            temp_dir.path().to_str().unwrap(),
        ],
        None,
    );

    // Quiet mode should have minimal output
    assert!(!output.contains("ARW CLI"));
}
