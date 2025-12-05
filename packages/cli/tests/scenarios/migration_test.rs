/// Real-world scenario: Migrating from plain llms.txt to full ARW
use std::fs;

mod common;
mod helpers;

use common::*;
use helpers::*;

#[test]
fn test_migration_from_llms_txt_only() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let site_path = temp_dir.path();

    // Start with just llms.txt (legacy)
    let legacy_manifest = r#"version: "1.0"
profile: ARW-1

site:
  name: "Legacy Site"
  homepage: "https://legacy.example.com"

policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#;

    fs::write(site_path.join("llms.txt"), legacy_manifest).unwrap();

    // Validate current state
    let output = run_cli_success(
        &["validate", "--path", site_path.to_str().unwrap()],
        None,
    );
    assert_output_contains(&output, "Success");

    // Run build to generate modern ARW structure
    let output = run_cli_success(
        &["build", "--source", site_path.to_str().unwrap()],
        None,
    );
    assert_output_contains(&output, "Build complete");

    // Verify migration created all new files
    assert!(site_path.join("llms.json").exists());
    assert!(site_path.join(".well-known/arw-manifest.json").exists());
    assert!(site_path.join(".well-known/arw-policies.json").exists());

    // Original llms.txt should still exist
    assert!(site_path.join("llms.txt").exists());

    // Validate complete setup
    let output = run_cli_success(
        &["validate", "--path", site_path.to_str().unwrap()],
        None,
    );
    assert_output_contains(&output, "Success");
}

#[test]
fn test_migration_arw1_to_arw2() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let site_path = temp_dir.path();

    // Start with ARW-1
    fs::write(site_path.join("llms.txt"), create_minimal_llms_txt()).unwrap();

    // Build initial state
    run_cli_success(&["build", "--source", site_path.to_str().unwrap()], None);

    // Upgrade to ARW-2 by adding content
    let upgraded_manifest = r#"version: "1.0"
profile: ARW-2

site:
  name: "Upgraded Site"
  homepage: "https://upgraded.example.com"

content:
  - url: "/"
    machine_view: "/index.llm.md"
    purpose: "homepage"
    priority: "high"

  - url: "/docs"
    machine_view: "/docs.llm.md"
    purpose: "documentation"
    priority: "high"

policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#;

    fs::write(site_path.join("llms.txt"), upgraded_manifest).unwrap();

    // Create machine views
    fs::write(site_path.join("index.llm.md"), "# Homepage").unwrap();
    fs::write(site_path.join("docs.llm.md"), "# Documentation").unwrap();

    // Rebuild
    let output = run_cli_success(
        &["build", "--source", site_path.to_str().unwrap()],
        None,
    );
    assert_output_contains(&output, "Build complete");

    // Validate upgraded structure
    let output = run_cli_success(
        &["validate", "--path", site_path.to_str().unwrap()],
        None,
    );
    assert_output_contains(&output, "Success");

    // Verify content index includes new entries
    let content_index = site_path.join(".well-known/arw-content-index.json");
    let content = fs::read_to_string(&content_index).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(json["content"].as_array().unwrap().len(), 2);
}

#[test]
fn test_migration_preserves_custom_configs() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let site_path = temp_dir.path();

    // Create manifest with custom policies
    let custom_manifest = r#"version: "1.0"
profile: ARW-1

site:
  name: "Custom Site"
  homepage: "https://custom.example.com"

policies:
  training:
    allowed: true
    conditions: "Only for research purposes"
  inference:
    allowed: true
    rate_limits: "1000 requests per day"
  attribution:
    required: true
    format: "Custom Site Name - URL - Date Accessed"
"#;

    fs::write(site_path.join("llms.txt"), custom_manifest).unwrap();

    // Build
    run_cli_success(&["build", "--source", site_path.to_str().unwrap()], None);

    // Verify custom policies are preserved
    let policies_path = site_path.join(".well-known/arw-policies.json");
    let content = fs::read_to_string(&policies_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(json["training"]["allowed"], true);
    assert!(json["training"]["conditions"].as_str().unwrap().contains("research"));
    assert!(json["inference"]["rate_limits"].as_str().is_some());
}

#[test]
fn test_migration_handles_existing_well_known() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let site_path = temp_dir.path();

    // Create existing .well-known with other files
    let well_known = site_path.join(".well-known");
    fs::create_dir_all(&well_known).unwrap();
    fs::write(well_known.join("security.txt"), "Contact: security@example.com").unwrap();

    // Add ARW manifest
    fs::write(site_path.join("llms.txt"), create_minimal_llms_txt()).unwrap();

    // Build
    run_cli_success(&["build", "--source", site_path.to_str().unwrap()], None);

    // Verify existing file was preserved
    assert!(well_known.join("security.txt").exists());

    // Verify ARW files were added
    assert!(well_known.join("arw-manifest.json").exists());
}
