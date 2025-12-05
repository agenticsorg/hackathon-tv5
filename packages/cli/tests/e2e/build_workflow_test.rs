/// End-to-end tests for the build workflow
mod common;
mod helpers;

use common::*;
use helpers::*;
use std::fs;

#[test]
fn test_build_creates_all_files() {
    setup_test_env();
    let temp_dir = create_test_site(&create_complete_llms_txt());

    let output = run_cli_success(
        &["build", "--source", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&output, "Build complete");

    // Verify all files were created
    assert_directory_contains(
        temp_dir.path(),
        &["llms.txt", "llms.json", "sitemap.xml"],
    );

    let well_known = temp_dir.path().join(".well-known");
    assert_directory_contains(
        &well_known,
        &[
            "arw-manifest.json",
            "arw-policies.json",
            "arw-content-index.json",
        ],
    );
}

#[test]
fn test_build_generates_valid_llms_json() {
    setup_test_env();
    let temp_dir = create_test_site(&create_complete_llms_txt());

    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    let json_path = temp_dir.path().join("llms.json");
    assert_valid_json(&json_path);
    assert_llms_files_equivalent(temp_dir.path());
}

#[test]
fn test_build_generates_well_known_manifest() {
    setup_test_env();
    let temp_dir = create_test_site(&create_complete_llms_txt());

    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    let manifest_path = temp_dir.path().join(".well-known/arw-manifest.json");
    assert_valid_json(&manifest_path);
    assert_json_field(&manifest_path, "site.name", "Complete Test Site");
}

#[test]
fn test_build_generates_well_known_policies() {
    setup_test_env();
    let temp_dir = create_test_site(&create_complete_llms_txt());

    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    let policies_path = temp_dir.path().join(".well-known/arw-policies.json");
    assert_valid_json(&policies_path);

    let content = fs::read_to_string(&policies_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(json["training"]["allowed"], false);
    assert_eq!(json["inference"]["allowed"], true);
    assert_eq!(json["attribution"]["required"], true);
}

#[test]
fn test_build_generates_content_index() {
    setup_test_env();
    let temp_dir = create_test_site(&create_complete_llms_txt());

    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    let content_index_path = temp_dir.path().join(".well-known/arw-content-index.json");
    assert_valid_json(&content_index_path);

    let content = fs::read_to_string(&content_index_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(json["content"].is_array());
    assert!(json["content"].as_array().unwrap().len() > 0);
}

#[test]
fn test_build_generates_sitemap() {
    setup_test_env();
    let temp_dir = create_test_site(&create_complete_llms_txt());

    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    let sitemap_path = temp_dir.path().join("sitemap.xml");
    assert!(sitemap_path.exists());

    let content = fs::read_to_string(&sitemap_path).unwrap();
    assert!(content.contains("<?xml"));
    assert!(content.contains("<urlset"));
    assert!(content.contains("<url>"));
}

#[test]
fn test_build_with_custom_base_url() {
    setup_test_env();
    let temp_dir = create_test_site(&create_complete_llms_txt());

    run_cli_success(
        &[
            "build",
            "--source",
            temp_dir.path().to_str().unwrap(),
            "--base-url",
            "https://custom.example.com",
        ],
        None,
    );

    let sitemap_path = temp_dir.path().join("sitemap.xml");
    let content = fs::read_to_string(&sitemap_path).unwrap();
    assert!(content.contains("https://custom.example.com"));
}

#[test]
fn test_build_fails_without_llms_txt() {
    setup_test_env();
    let temp_dir = create_temp_dir();

    let (_stdout, stderr) = run_cli_failure(
        &["build", "--source", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&stderr, "llms.txt not found");
}

#[test]
fn test_build_preserves_existing_machine_views() {
    setup_test_env();
    let temp_dir = create_complete_test_site();

    // Create existing machine view
    let existing_content = "# Existing Content\n\nThis should be preserved.";
    fs::write(
        temp_dir.path().join("index.llm.md"),
        existing_content,
    )
    .unwrap();

    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    // Verify existing file was not overwritten
    let content = fs::read_to_string(temp_dir.path().join("index.llm.md")).unwrap();
    assert!(content.contains("Existing Content"));
}

#[test]
fn test_build_creates_well_known_directory() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    // Ensure .well-known doesn't exist
    let well_known = temp_dir.path().join(".well-known");
    if well_known.exists() {
        fs::remove_dir_all(&well_known).unwrap();
    }

    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    assert!(well_known.exists());
    assert!(well_known.is_dir());
}

#[test]
fn test_build_incremental_update() {
    setup_test_env();
    let temp_dir = create_test_site(&create_complete_llms_txt());

    // First build
    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    let json_path = temp_dir.path().join("llms.json");
    let first_mtime = fs::metadata(&json_path).unwrap().modified().unwrap();

    // Wait a bit
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Second build (should recreate files)
    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    let second_mtime = fs::metadata(&json_path).unwrap().modified().unwrap();
    assert!(second_mtime > first_mtime);
}

#[test]
fn test_build_with_invalid_manifest() {
    setup_test_env();
    let temp_dir = create_test_site(&create_invalid_llms_txt_missing_version());

    let (_stdout, stderr) = run_cli_failure(
        &["build", "--source", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&stderr, "Failed to parse");
}

#[test]
fn test_build_output_shows_all_generated_files() {
    setup_test_env();
    let temp_dir = create_test_site(&create_complete_llms_txt());

    let output = run_cli_success(
        &["build", "--source", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&output, "llms.json");
    assert_output_contains(&output, "arw-manifest.json");
    assert_output_contains(&output, "arw-policies.json");
    assert_output_contains(&output, "arw-content-index.json");
    assert_output_contains(&output, "sitemap.xml");
}

#[test]
fn test_build_preserves_content_chunks() {
    setup_test_env();
    let temp_dir = create_test_site(&create_complete_llms_txt());

    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);

    let content_index_path = temp_dir.path().join(".well-known/arw-content-index.json");
    let content = fs::read_to_string(&content_index_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Verify chunks are present
    let first_content = &json["content"][0];
    assert!(first_content["chunks"].is_array());
    let chunks = first_content["chunks"].as_array().unwrap();
    assert!(chunks.len() > 0);
    assert!(chunks[0]["id"].is_string());
    assert!(chunks[0]["heading"].is_string());
}

#[test]
fn test_build_handles_minimal_manifest() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    let output = run_cli_success(
        &["build", "--source", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&output, "Build complete");

    // Should still create all required files
    assert!(temp_dir.path().join("llms.json").exists());
    assert!(temp_dir.path().join(".well-known/arw-manifest.json").exists());
}
