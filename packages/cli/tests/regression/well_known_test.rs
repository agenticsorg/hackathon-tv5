/// Regression test: .well-known discovery files
use std::fs;

mod common;
mod helpers;

use common::*;
use helpers::*;

#[test]
fn test_well_known_manifest_structure() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    let manifest_path = temp_dir.path().join(".well-known/arw-manifest.json");
    assert!(manifest_path.exists());

    let content = fs::read_to_string(&manifest_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Verify required fields
    assert!(json["version"].is_string());
    assert!(json["site"].is_object());
    assert!(json["site"]["name"].is_string());
    assert!(json["site"]["homepage"].is_string());
}

#[test]
fn test_well_known_policies_structure() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    let policies_path = temp_dir.path().join(".well-known/arw-policies.json");
    assert!(policies_path.exists());

    let content = fs::read_to_string(&policies_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Verify policy structure
    assert!(json["training"].is_object());
    assert!(json["training"]["allowed"].is_boolean());
    assert!(json["inference"].is_object());
    assert!(json["inference"]["allowed"].is_boolean());
}

#[test]
fn test_well_known_content_index_structure() {
    setup_test_env();
    let temp_dir = create_test_site(&create_complete_llms_txt());

    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    let content_index_path = temp_dir.path().join(".well-known/arw-content-index.json");
    assert!(content_index_path.exists());

    let content = fs::read_to_string(&content_index_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Verify content index structure
    assert!(json["content"].is_array());

    if let Some(first_item) = json["content"].as_array().and_then(|a| a.first()) {
        assert!(first_item["url"].is_string());
        assert!(first_item["machine_view"].is_string());
        assert!(first_item["purpose"].is_string());
    }
}

#[test]
fn test_well_known_files_are_valid_json() {
    setup_test_env();
    let temp_dir = create_test_site(&create_complete_llms_txt());

    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    let well_known = temp_dir.path().join(".well-known");

    // All .well-known files should be valid JSON
    for entry in fs::read_dir(&well_known).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path).unwrap();
            serde_json::from_str::<serde_json::Value>(&content)
                .unwrap_or_else(|e| panic!("Invalid JSON in {}: {}", path.display(), e));
        }
    }
}

#[test]
fn test_well_known_directory_permissions() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    let well_known = temp_dir.path().join(".well-known");

    // Directory should be readable
    assert!(well_known.exists());
    assert!(well_known.is_dir());

    // Files should be readable
    let manifest = well_known.join("arw-manifest.json");
    assert!(manifest.exists());
    let _ = fs::read_to_string(&manifest).expect("File should be readable");
}
