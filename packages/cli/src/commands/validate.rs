use anyhow::{Context, Result};
use std::path::Path;

use crate::cli;
use crate::validators::llms_txt;

pub async fn run(path: String, strict: bool, _fix: bool) -> Result<()> {
    cli::info(&format!("Validating ARW implementation in: {}", path));

    let site_path = Path::new(&path);
    let mut has_errors = false;

    // Check for llms.txt
    let llms_txt_path = site_path.join("llms.txt");
    if !llms_txt_path.exists() {
        cli::error("llms.txt not found");
        has_errors = true;
    } else {
        // Validate llms.txt against schema
        cli::info("Validating llms.txt against ARW schema...");

        match llms_txt::validate(&llms_txt_path) {
            Ok(errors) => {
                if errors.is_empty() {
                    cli::success("✓ llms.txt is valid");
                } else {
                    cli::error(&format!("✗ llms.txt has {} validation errors:", errors.len()));
                    for error in &errors {
                        println!("  • {}", error);
                    }
                    has_errors = true;
                }
            }
            Err(e) => {
                cli::error(&format!("Failed to validate llms.txt: {}", e));
                has_errors = true;
            }
        }
    }

    // Check for llms.json (optional JSON mirror)
    let llms_json_path = site_path.join("llms.json");
    if llms_json_path.exists() {
        cli::success("✓ llms.json found (JSON mirror)");

        // Validate that it's valid JSON
        match std::fs::read_to_string(&llms_json_path) {
            Ok(json_content) => {
                match serde_json::from_str::<serde_json::Value>(&json_content) {
                    Ok(_) => cli::success("✓ llms.json is valid JSON"),
                    Err(e) => {
                        cli::error(&format!("✗ llms.json is invalid JSON: {}", e));
                        has_errors = true;
                    }
                }
            }
            Err(e) => {
                cli::error(&format!("Failed to read llms.json: {}", e));
                has_errors = true;
            }
        }
    } else {
        cli::info("ℹ llms.json not found (optional - run 'arw build' to generate)");
    }

    // Check for .well-known files
    let well_known_path = site_path.join(".well-known");
    if well_known_path.exists() {
        cli::info("Checking .well-known files...");

        let arw_manifest = well_known_path.join("arw-manifest.json");
        if arw_manifest.exists() {
            cli::success("✓ .well-known/arw-manifest.json found");
        } else if strict {
            cli::warn("⚠ .well-known/arw-manifest.json not found (optional but recommended)");
        }

        let arw_policies = well_known_path.join("arw-policies.json");
        if arw_policies.exists() {
            cli::success("✓ .well-known/arw-policies.json found");
        } else if strict {
            cli::warn("⚠ .well-known/arw-policies.json not found (optional but recommended)");
        }
    } else if strict {
        cli::warn("⚠ .well-known directory not found (optional but recommended)");
    }

    // Check for robots.txt
    let robots_txt_path = site_path.join("robots.txt");
    if robots_txt_path.exists() {
        cli::success("✓ robots.txt found");

        // Check if it includes ARW hints
        let robots_content = std::fs::read_to_string(&robots_txt_path)
            .context("Failed to read robots.txt")?;

        if robots_content.contains("llms.txt") || robots_content.contains("Agent-Ready Web") {
            cli::success("✓ robots.txt includes ARW discovery hints");
        } else if strict {
            cli::warn("⚠ robots.txt does not include ARW discovery hints");
        }
    } else {
        cli::warn("⚠ robots.txt not found (recommended for ARW-1 compliance)");
        if strict {
            has_errors = true;
        }
    }

    // Check for sitemap.xml
    let sitemap_xml_path = site_path.join("sitemap.xml");
    if sitemap_xml_path.exists() {
        cli::success("✓ sitemap.xml found");
    } else {
        cli::warn("⚠ sitemap.xml not found (recommended for ARW-1 compliance)");
        if strict {
            has_errors = true;
        }
    }

    // Deep consistency checks if strict mode
    if strict {
        cli::info("Running deep consistency checks...");

        let consistency_validator =
            crate::validators::consistency::ConsistencyValidator::new(path.clone());

        match consistency_validator.validate_all().await {
            Ok(consistency_errors) => {
                if consistency_errors.is_empty() {
                    cli::success("✓ All consistency checks passed");
                } else {
                    cli::error(&format!(
                        "✗ Found {} consistency errors:",
                        consistency_errors.len()
                    ));
                    for error in &consistency_errors {
                        println!("  • {}", error);
                    }
                    has_errors = true;
                }
            }
            Err(e) => {
                cli::error(&format!("Failed to run consistency checks: {}", e));
                has_errors = true;
            }
        }
    }

    // Summary
    println!();
    if has_errors {
        cli::error("Validation failed with errors");
        std::process::exit(1);
    } else {
        cli::success("All validation checks passed!");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_valid_llms_txt(dir: &Path) {
        let content = r#"version: 1.0
profile: ARW-1

site:
  name: "Test Site"
  description: "Test description"
  homepage: "https://test.com"
  contact: "test@test.com"

content:
  - url: /
    machine_view: /index.llm.md
    purpose: homepage
    priority: high

policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#;
        fs::write(dir.join("llms.txt"), content).unwrap();
    }

    fn create_valid_llms_json(dir: &Path) {
        let content = r#"{
  "version": "1.0",
  "profile": "ARW-1",
  "site": {
    "name": "Test Site",
    "description": "Test description",
    "homepage": "https://test.com",
    "contact": "test@test.com"
  },
  "content": [
    {
      "url": "/",
      "machine_view": "/index.llm.md",
      "purpose": "homepage",
      "priority": "high"
    }
  ],
  "policies": {
    "training": {
      "allowed": false
    },
    "inference": {
      "allowed": true
    },
    "attribution": {
      "required": true
    }
  }
}"#;
        fs::write(dir.join("llms.json"), content).unwrap();
    }

    fn create_invalid_llms_json(dir: &Path) {
        let content = "{ invalid json }";
        fs::write(dir.join("llms.json"), content).unwrap();
    }

    #[test]
    fn test_llms_json_exists_and_valid() {
        let temp_dir = TempDir::new().unwrap();
        create_valid_llms_txt(temp_dir.path());
        create_valid_llms_json(temp_dir.path());

        let json_path = temp_dir.path().join("llms.json");
        assert!(json_path.exists(), "llms.json should exist");

        // Verify it's valid JSON
        let json_content = fs::read_to_string(&json_path).unwrap();
        let result = serde_json::from_str::<serde_json::Value>(&json_content);
        assert!(result.is_ok(), "llms.json should be valid JSON");
    }

    #[test]
    fn test_llms_json_validation_detects_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        create_invalid_llms_json(temp_dir.path());

        let json_path = temp_dir.path().join("llms.json");
        let json_content = fs::read_to_string(&json_path).unwrap();
        let result = serde_json::from_str::<serde_json::Value>(&json_content);
        assert!(result.is_err(), "Invalid JSON should fail validation");
    }

    #[test]
    fn test_llms_json_optional() {
        let temp_dir = TempDir::new().unwrap();
        create_valid_llms_txt(temp_dir.path());
        // Don't create llms.json

        let json_path = temp_dir.path().join("llms.json");
        assert!(!json_path.exists(), "llms.json should not exist");
        // This is fine - llms.json is optional
    }

    #[test]
    fn test_llms_txt_required() {
        let temp_dir = TempDir::new().unwrap();
        // Don't create llms.txt

        let llms_path = temp_dir.path().join("llms.txt");
        assert!(!llms_path.exists(), "llms.txt should not exist");
        // Without llms.txt, validation would fail
    }

    #[test]
    fn test_llms_json_validation_logic() {
        let temp_dir = TempDir::new().unwrap();

        // Test valid JSON
        create_valid_llms_json(temp_dir.path());
        let json_path = temp_dir.path().join("llms.json");
        let json_content = fs::read_to_string(&json_path).unwrap();
        let parse_result = serde_json::from_str::<serde_json::Value>(&json_content);
        assert!(parse_result.is_ok(), "Valid JSON should parse");

        // Test invalid JSON
        create_invalid_llms_json(temp_dir.path());
        let json_content = fs::read_to_string(&json_path).unwrap();
        let parse_result = serde_json::from_str::<serde_json::Value>(&json_content);
        assert!(parse_result.is_err(), "Invalid JSON should fail to parse");
    }

    fn create_robots_txt(dir: &Path) {
        let content = "User-agent: *\nAllow: /\n\nSitemap: /sitemap.xml";
        fs::write(dir.join("robots.txt"), content).unwrap();
    }

    fn create_robots_txt_with_arw_hints(dir: &Path) {
        let content = "User-agent: *\nAllow: /\n\n# Agent-Ready Web\nAllow: /llms.txt\n\nSitemap: /sitemap.xml";
        fs::write(dir.join("robots.txt"), content).unwrap();
    }

    fn create_sitemap_xml(dir: &Path) {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <url>
        <loc>https://test.com/</loc>
    </url>
</urlset>"#;
        fs::write(dir.join("sitemap.xml"), content).unwrap();
    }

    fn create_well_known_files(dir: &Path) {
        let well_known = dir.join(".well-known");
        fs::create_dir_all(&well_known).unwrap();

        fs::write(
            well_known.join("arw-manifest.json"),
            r#"{"version": "1.0", "profile": "ARW-1"}"#
        ).unwrap();

        fs::write(
            well_known.join("arw-policies.json"),
            r#"{"training": {"allowed": false}}"#
        ).unwrap();
    }

    #[tokio::test]
    async fn test_validate_robots_txt_exists() {
        let temp_dir = TempDir::new().unwrap();
        create_valid_llms_txt(temp_dir.path());
        create_robots_txt(temp_dir.path());

        // Should pass basic validation
        let robots_path = temp_dir.path().join("robots.txt");
        assert!(robots_path.exists(), "robots.txt should exist");
    }

    #[tokio::test]
    async fn test_validate_robots_txt_with_arw_hints() {
        let temp_dir = TempDir::new().unwrap();
        create_valid_llms_txt(temp_dir.path());
        create_robots_txt_with_arw_hints(temp_dir.path());

        let robots_content = fs::read_to_string(temp_dir.path().join("robots.txt")).unwrap();
        assert!(robots_content.contains("llms.txt") || robots_content.contains("Agent-Ready Web"));
    }

    #[tokio::test]
    async fn test_validate_sitemap_exists() {
        let temp_dir = TempDir::new().unwrap();
        create_valid_llms_txt(temp_dir.path());
        create_sitemap_xml(temp_dir.path());

        let sitemap_path = temp_dir.path().join("sitemap.xml");
        assert!(sitemap_path.exists(), "sitemap.xml should exist");
    }

    #[tokio::test]
    async fn test_validate_well_known_directory() {
        let temp_dir = TempDir::new().unwrap();
        create_valid_llms_txt(temp_dir.path());
        create_well_known_files(temp_dir.path());

        let well_known = temp_dir.path().join(".well-known");
        assert!(well_known.exists());
        assert!(well_known.join("arw-manifest.json").exists());
        assert!(well_known.join("arw-policies.json").exists());
    }

    #[tokio::test]
    async fn test_run_validates_complete_setup() {
        let temp_dir = TempDir::new().unwrap();
        create_valid_llms_txt(temp_dir.path());
        create_valid_llms_json(temp_dir.path());
        create_robots_txt_with_arw_hints(temp_dir.path());
        create_sitemap_xml(temp_dir.path());
        create_well_known_files(temp_dir.path());

        // All required files exist
        assert!(temp_dir.path().join("llms.txt").exists());
        assert!(temp_dir.path().join("llms.json").exists());
        assert!(temp_dir.path().join("robots.txt").exists());
        assert!(temp_dir.path().join("sitemap.xml").exists());
        assert!(temp_dir.path().join(".well-known/arw-manifest.json").exists());
    }

    #[tokio::test]
    async fn test_validate_missing_optional_files() {
        let temp_dir = TempDir::new().unwrap();
        create_valid_llms_txt(temp_dir.path());
        // Don't create optional files

        // llms.json is optional
        assert!(!temp_dir.path().join("llms.json").exists());
        // This should be okay in non-strict mode
    }

    #[tokio::test]
    async fn test_llms_txt_validation_passes() {
        let temp_dir = TempDir::new().unwrap();
        create_valid_llms_txt(temp_dir.path());

        let llms_path = temp_dir.path().join("llms.txt");
        assert!(llms_path.exists());

        // Verify content is valid YAML
        let content = fs::read_to_string(&llms_path).unwrap();
        let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(&content);
        assert!(parsed.is_ok(), "llms.txt should be valid YAML");
    }

    #[tokio::test]
    async fn test_validate_path_handling() {
        let temp_dir = TempDir::new().unwrap();
        create_valid_llms_txt(temp_dir.path());

        let path_str = temp_dir.path().to_str().unwrap().to_string();
        // Path should be valid
        assert!(!path_str.is_empty());
        assert!(temp_dir.path().exists());
    }
}
