/// Real-world scenario: Large site with 100+ pages
use std::fs;

mod common;
mod helpers;

use common::*;
use helpers::*;

#[test]
fn test_large_site_generation() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let site_path = temp_dir.path();

    // Create 100 HTML pages
    for i in 0..100 {
        let page_name = format!("page{}.html", i);
        let page_content = format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Page {}</title></head>
<body>
    <h1>Page {}</h1>
    <p>Content for page {}.</p>
</body>
</html>"#,
            i, i, i
        );
        fs::write(site_path.join(&page_name), page_content).unwrap();
    }

    // Generate machine views for all pages
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

    // Verify all .llm.md files were created
    for i in 0..100 {
        let llm_md = site_path.join(format!("page{}.llm.md", i));
        assert!(llm_md.exists(), "Missing page{}.llm.md", i);
    }
}

#[test]
fn test_large_site_manifest_generation() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let site_path = temp_dir.path();

    // Create manifest with many content entries
    let mut content_entries = Vec::new();
    for i in 0..100 {
        content_entries.push(format!(
            r#"  - url: "/page{}"
    machine_view: "/page{}.llm.md"
    purpose: "content"
    priority: "medium""#,
            i, i
        ));
    }

    let manifest = format!(
        r#"version: "1.0"
profile: ARW-2

site:
  name: "Large Site"
  homepage: "https://large.example.com"

content:
{}

policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#,
        content_entries.join("\n")
    );

    fs::write(site_path.join("llms.txt"), manifest).unwrap();

    // Build
    let output = run_cli_success(
        &["build", "--source", site_path.to_str().unwrap()],
        None,
    );
    assert_output_contains(&output, "Build complete");

    // Verify content index has all entries
    let content_index = site_path.join(".well-known/arw-content-index.json");
    let content = fs::read_to_string(&content_index).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(json["content"].as_array().unwrap().len(), 100);
}

#[test]
fn test_large_site_validation_performance() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let site_path = temp_dir.path();

    // Create manifest with many entries
    let mut content_entries = Vec::new();
    for i in 0..50 {
        content_entries.push(format!(
            r#"  - url: "/page{}"
    machine_view: "/page{}.llm.md"
    purpose: "content""#,
            i, i
        ));
    }

    let manifest = format!(
        r#"version: "1.0"
profile: ARW-2
site:
  name: "Large"
  homepage: "https://large.example.com"
content:
{}
policies:
  training:
    allowed: false
"#,
        content_entries.join("\n")
    );

    fs::write(site_path.join("llms.txt"), manifest).unwrap();

    // Time validation
    let start = std::time::Instant::now();
    run_cli_success(&["validate", "--path", site_path.to_str().unwrap()], None);
    let duration = start.elapsed();

    // Validation should complete in reasonable time even for large manifests
    assert!(
        duration.as_secs() < 10,
        "Validation took too long: {:?}",
        duration
    );
}

#[test]
fn test_large_site_sitemap_generation() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let site_path = temp_dir.path();

    // Create manifest with many pages
    let mut content_entries = Vec::new();
    for i in 0..200 {
        content_entries.push(format!(
            r#"  - url: "/page{}"
    machine_view: "/page{}.llm.md"
    purpose: "content""#,
            i, i
        ));
    }

    let manifest = format!(
        r#"version: "1.0"
profile: ARW-2
site:
  name: "Large"
  homepage: "https://large.example.com"
content:
{}
policies:
  training:
    allowed: false
"#,
        content_entries.join("\n")
    );

    fs::write(site_path.join("llms.txt"), manifest).unwrap();

    // Build
    run_cli_success(&["build", "--source", site_path.to_str().unwrap()], None);

    // Verify sitemap includes all pages
    let sitemap = site_path.join("sitemap.xml");
    let content = fs::read_to_string(&sitemap).unwrap();

    // Count URL entries (should have 200)
    let url_count = content.matches("<url>").count();
    assert!(url_count >= 200, "Sitemap should contain all 200 URLs");
}
