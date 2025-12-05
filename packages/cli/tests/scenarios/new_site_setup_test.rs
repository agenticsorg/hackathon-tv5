/// Real-world scenario: Setting up ARW on a brand new site
use std::fs;

mod common;
mod helpers;

use common::*;
use helpers::*;

#[test]
fn test_complete_new_site_workflow() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let site_path = temp_dir.path();

    // Step 1: Init ARW structure
    let output = run_cli_success(
        &["init", "--path", site_path.to_str().unwrap(), "--yes"],
        None,
    );
    assert_output_contains(&output, "Success");
    assert!(site_path.join("llms.txt").exists());

    // Step 2: Create some HTML pages
    fs::write(
        site_path.join("index.html"),
        create_test_html_page(),
    )
    .unwrap();

    fs::write(
        site_path.join("about.html"),
        "<html><body><h1>About Us</h1></body></html>",
    )
    .unwrap();

    // Step 3: Generate machine views
    let output = run_cli_success(
        &[
            "generate",
            site_path.to_str().unwrap(),
            "--recursive",
            "--output",
            site_path.to_str().unwrap(),
        ],
        None,
    );
    assert_output_contains(&output, "Success");
    assert!(site_path.join("index.llm.md").exists());
    assert!(site_path.join("about.llm.md").exists());

    // Step 4: Update llms.txt with content references
    let manifest = format!(
        r#"version: "1.0"
profile: ARW-2

site:
  name: "New Test Site"
  homepage: "https://newsite.example.com"
  description: "A brand new ARW-enabled site"
  contact: "admin@newsite.example.com"

content:
  - url: "/"
    machine_view: "/index.llm.md"
    purpose: "homepage"
    priority: "high"

  - url: "/about"
    machine_view: "/about.llm.md"
    purpose: "about"
    priority: "medium"

policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#
    );

    fs::write(site_path.join("llms.txt"), manifest).unwrap();

    // Step 5: Build all ARW files
    let output = run_cli_success(
        &["build", "--source", site_path.to_str().unwrap()],
        None,
    );
    assert_output_contains(&output, "Build complete");

    // Step 6: Validate everything
    let output = run_cli_success(
        &["validate", "--path", site_path.to_str().unwrap(), "--strict"],
        None,
    );
    assert_output_contains(&output, "Success");

    // Verify complete structure
    assert_directory_contains(
        site_path,
        &[
            "llms.txt",
            "llms.json",
            "sitemap.xml",
            "index.html",
            "about.html",
            "index.llm.md",
            "about.llm.md",
        ],
    );

    assert_directory_contains(
        &site_path.join(".well-known"),
        &[
            "arw-manifest.json",
            "arw-policies.json",
            "arw-content-index.json",
        ],
    );
}

#[test]
fn test_new_site_with_actions() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let site_path = temp_dir.path();

    // Create manifest with actions (ARW-3)
    let manifest = r#"version: "1.0"
profile: ARW-3

site:
  name: "Interactive Site"
  homepage: "https://interactive.example.com"

content:
  - url: "/"
    machine_view: "/index.llm.md"
    purpose: "homepage"

actions:
  - id: "search"
    name: "Search Site"
    description: "Full-text search"
    endpoint: "/api/search"
    method: "POST"
    auth: "none"
    parameters:
      - name: "query"
        type: "string"
        required: true

  - id: "subscribe"
    name: "Subscribe"
    endpoint: "/api/subscribe"
    method: "POST"
    auth: "api_key"
    parameters:
      - name: "email"
        type: "string"
        required: true

policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#;

    fs::write(site_path.join("llms.txt"), manifest).unwrap();

    // Build
    let output = run_cli_success(
        &["build", "--source", site_path.to_str().unwrap()],
        None,
    );
    assert_output_contains(&output, "Build complete");

    // Validate
    let output = run_cli_success(
        &["validate", "--path", site_path.to_str().unwrap()],
        None,
    );
    assert_output_contains(&output, "Success");

    // Verify actions are in content index
    let content_index = site_path.join(".well-known/arw-content-index.json");
    let content = fs::read_to_string(&content_index).unwrap();
    assert!(content.contains("search"));
    assert!(content.contains("subscribe"));
}

#[test]
fn test_new_site_minimal_setup() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let site_path = temp_dir.path();

    // Create absolute minimum ARW-1 manifest
    fs::write(site_path.join("llms.txt"), create_minimal_llms_txt()).unwrap();

    // Build
    let output = run_cli_success(
        &["build", "--source", site_path.to_str().unwrap()],
        None,
    );
    assert_output_contains(&output, "Build complete");

    // Validate
    let output = run_cli_success(
        &["validate", "--path", site_path.to_str().unwrap()],
        None,
    );
    assert_output_contains(&output, "Success");

    // Even minimal setup should create all discovery files
    assert!(site_path.join("llms.json").exists());
    assert!(site_path.join(".well-known/arw-manifest.json").exists());
}
