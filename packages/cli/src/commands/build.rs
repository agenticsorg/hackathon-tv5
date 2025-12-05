use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::cli;
use crate::generators;
use crate::generators::well_known::arw_content_index::ContentItem;
use crate::generators::well_known::arw_manifest::SiteInfo;

#[derive(Debug, Deserialize, Serialize)]
struct Manifest {
    version: Option<String>,
    profile: Option<String>,
    site: Option<Site>,
    content: Option<Vec<Content>>,
    policies: Option<Policies>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Site {
    name: String,
    description: String,
    homepage: String,
    contact: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Content {
    url: String,
    machine_view: String,
    purpose: String,
    priority: Option<String>,
    chunks: Option<Vec<Chunk>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Chunk {
    id: String,
    heading: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Policies {
    training: Option<TrainingPolicy>,
    inference: Option<InferencePolicy>,
    attribution: Option<AttributionPolicy>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TrainingPolicy {
    allowed: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct InferencePolicy {
    allowed: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct AttributionPolicy {
    required: bool,
}

pub async fn run(source: String, base_url: Option<String>) -> Result<()> {
    let source_path = Path::new(&source);

    cli::info(&format!("Building ARW files from: {}", source_path.display()));
    println!();

    // Load llms.txt
    let manifest_path = source_path.join("llms.txt");
    if !manifest_path.exists() {
        return Err(anyhow::anyhow!(
            "llms.txt not found at {:?}. Run 'arw init' first.",
            manifest_path
        ));
    }

    cli::step(1, 6, "Reading llms.txt");
    let manifest_content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read llms.txt at {:?}", manifest_path))?;

    let manifest: Manifest = serde_yaml::from_str(&manifest_content)
        .context("Failed to parse llms.txt")?;

    cli::success("llms.txt loaded");
    println!();

    // Generate llms.json (JSON mirror of llms.txt)
    cli::step(2, 6, "Generating llms.json");
    let json_content = serde_json::to_string_pretty(&manifest)
        .context("Failed to serialize llms.json")?;
    let json_path = source_path.join("llms.json");
    fs::write(&json_path, json_content)
        .with_context(|| format!("Failed to write llms.json to {:?}", json_path))?;
    cli::success("llms.json created (JSON mirror)");
    println!();

    // Extract data
    let site = manifest.site.ok_or_else(|| anyhow::anyhow!("Missing 'site' section in llms.txt"))?;
    let profile = manifest.profile.unwrap_or_else(|| "ARW-1".to_string());
    let base = base_url.unwrap_or_else(|| site.homepage.clone());

    // Generate .well-known/arw-manifest.json
    cli::step(3, 6, "Generating .well-known/arw-manifest.json");
    let site_info = SiteInfo {
        name: site.name.clone(),
        description: site.description.clone(),
        homepage: site.homepage.clone(),
        contact: site.contact.clone(),
    };
    generators::well_known::arw_manifest::generate(source_path, &site_info, &profile)?;
    cli::success(".well-known/arw-manifest.json created");
    println!();

    // Generate .well-known/arw-policies.json
    cli::step(4, 6, "Generating .well-known/arw-policies.json");
    let policies = manifest.policies.unwrap_or_else(|| Policies {
        training: Some(TrainingPolicy { allowed: false }),
        inference: Some(InferencePolicy { allowed: true }),
        attribution: Some(AttributionPolicy { required: true }),
    });

    generators::well_known::arw_policies::generate(
        source_path,
        policies.training.as_ref().map(|p| p.allowed).unwrap_or(false),
        policies.inference.as_ref().map(|p| p.allowed).unwrap_or(true),
        policies.attribution.as_ref().map(|p| p.required).unwrap_or(true),
    )?;
    cli::success(".well-known/arw-policies.json created");
    println!();

    // Generate .well-known/arw-content-index.json
    cli::step(5, 6, "Generating .well-known/arw-content-index.json");
    let content_items: Vec<ContentItem> = manifest
        .content
        .unwrap_or_default()
        .into_iter()
        .map(|c| ContentItem {
            url: c.url,
            machine_view: c.machine_view,
            purpose: c.purpose,
            priority: c.priority.unwrap_or_else(|| "medium".to_string()),
            chunks: c.chunks.map(|chunks| {
                chunks.into_iter().map(|ch| {
                    crate::generators::well_known::arw_content_index::ChunkInfo {
                        id: ch.id,
                        heading: ch.heading,
                        description: ch.description,
                    }
                }).collect()
            }),
        })
        .collect();

    generators::well_known::arw_content_index::generate(source_path, content_items)?;
    cli::success(".well-known/arw-content-index.json created");
    println!();

    // Generate sitemap.xml
    cli::step(6, 6, "Generating sitemap.xml");
    let sitemap_output = source_path.join("sitemap.xml");
    crate::commands::sitemap::generate_from_manifest(
        source_path,
        &sitemap_output,
        &base,
    )?;
    cli::success("sitemap.xml created");
    println!();

    println!("✨ {}", "Build complete!".bold());
    println!();
    println!("Generated files:");
    println!("  • llms.json (JSON mirror of llms.txt)");
    println!("  • .well-known/arw-manifest.json (discovery router)");
    println!("  • .well-known/arw-policies.json");
    println!("  • .well-known/arw-content-index.json");
    println!("  • sitemap.xml");
    println!();
    println!("💡 Tip: Run 'arw validate --strict' to verify everything");

    Ok(())
}

use colored::Colorize;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_llms_txt(dir: &TempDir) -> String {
        let llms_content = r#"version: 1.0
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

  - url: /about
    machine_view: /about.llm.md
    purpose: about
    priority: medium

policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#;
        let llms_path = dir.path().join("llms.txt");
        fs::write(&llms_path, llms_content).unwrap();
        dir.path().to_str().unwrap().to_string()
    }

    #[tokio::test]
    async fn test_build_generates_llms_json() {
        let temp_dir = TempDir::new().unwrap();
        let source = create_test_llms_txt(&temp_dir);

        // Run build
        let result = run(source.clone(), None).await;
        assert!(result.is_ok(), "Build should succeed");

        // Check llms.json was created
        let json_path = temp_dir.path().join("llms.json");
        assert!(json_path.exists(), "llms.json should be created");

        // Verify it's valid JSON
        let json_content = fs::read_to_string(&json_path).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_content).unwrap();

        // Verify structure
        assert_eq!(json_value["version"], "1.0");
        assert_eq!(json_value["profile"], "ARW-1");
        assert_eq!(json_value["site"]["name"], "Test Site");
        assert_eq!(json_value["site"]["contact"], "test@test.com");
    }

    #[tokio::test]
    async fn test_build_llms_json_mirrors_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let source = create_test_llms_txt(&temp_dir);

        run(source.clone(), None).await.unwrap();

        // Read both files
        let yaml_content = fs::read_to_string(temp_dir.path().join("llms.txt")).unwrap();
        let json_content = fs::read_to_string(temp_dir.path().join("llms.json")).unwrap();

        // Parse both
        let yaml_parsed: Manifest = serde_yaml::from_str(&yaml_content).unwrap();
        let json_parsed: Manifest = serde_json::from_str(&json_content).unwrap();

        // Verify structural equality
        assert_eq!(yaml_parsed.version, json_parsed.version);
        assert_eq!(yaml_parsed.profile, json_parsed.profile);

        let yaml_site = yaml_parsed.site.unwrap();
        let json_site = json_parsed.site.unwrap();
        assert_eq!(yaml_site.name, json_site.name);
        assert_eq!(yaml_site.homepage, json_site.homepage);
        assert_eq!(yaml_site.contact, json_site.contact);
    }

    #[tokio::test]
    async fn test_build_generates_well_known_files() {
        let temp_dir = TempDir::new().unwrap();
        let source = create_test_llms_txt(&temp_dir);

        run(source.clone(), None).await.unwrap();

        let well_known = temp_dir.path().join(".well-known");

        // Check all .well-known files exist
        assert!(well_known.join("arw-manifest.json").exists());
        assert!(well_known.join("arw-policies.json").exists());
        assert!(well_known.join("arw-content-index.json").exists());

        // Verify they're valid JSON
        let manifest = fs::read_to_string(well_known.join("arw-manifest.json")).unwrap();
        serde_json::from_str::<serde_json::Value>(&manifest).unwrap();

        let policies = fs::read_to_string(well_known.join("arw-policies.json")).unwrap();
        serde_json::from_str::<serde_json::Value>(&policies).unwrap();

        let content_index = fs::read_to_string(well_known.join("arw-content-index.json")).unwrap();
        serde_json::from_str::<serde_json::Value>(&content_index).unwrap();
    }

    #[tokio::test]
    async fn test_build_generates_sitemap() {
        let temp_dir = TempDir::new().unwrap();
        let source = create_test_llms_txt(&temp_dir);

        run(source.clone(), None).await.unwrap();

        let sitemap_path = temp_dir.path().join("sitemap.xml");
        assert!(sitemap_path.exists(), "sitemap.xml should be created");

        let sitemap_content = fs::read_to_string(&sitemap_path).unwrap();
        assert!(sitemap_content.contains("<?xml"));
        assert!(sitemap_content.contains("<urlset"));
    }

    #[tokio::test]
    async fn test_build_fails_without_llms_txt() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().to_str().unwrap().to_string();

        let result = run(source, None).await;
        assert!(result.is_err(), "Should fail without llms.txt");

        let error = result.unwrap_err();
        assert!(error.to_string().contains("llms.txt not found"));
    }

    #[tokio::test]
    async fn test_build_with_custom_base_url() {
        let temp_dir = TempDir::new().unwrap();
        let source = create_test_llms_txt(&temp_dir);
        let custom_url = "https://custom.example.com".to_string();

        run(source.clone(), Some(custom_url.clone())).await.unwrap();

        // Check sitemap uses custom URL
        let sitemap_content = fs::read_to_string(temp_dir.path().join("sitemap.xml")).unwrap();
        assert!(sitemap_content.contains(&custom_url));
    }

    #[tokio::test]
    async fn test_build_preserves_content_priorities() {
        let temp_dir = TempDir::new().unwrap();
        let source = create_test_llms_txt(&temp_dir);

        run(source.clone(), None).await.unwrap();

        let json_content = fs::read_to_string(temp_dir.path().join("llms.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&json_content).unwrap();

        // Verify priorities are preserved
        assert_eq!(json["content"][0]["priority"], "high");
        assert_eq!(json["content"][1]["priority"], "medium");
    }

    #[tokio::test]
    async fn test_build_preserves_policies() {
        let temp_dir = TempDir::new().unwrap();
        let source = create_test_llms_txt(&temp_dir);

        run(source.clone(), None).await.unwrap();

        let json_content = fs::read_to_string(temp_dir.path().join("llms.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&json_content).unwrap();

        // Verify policies are preserved
        assert_eq!(json["policies"]["training"]["allowed"], false);
        assert_eq!(json["policies"]["inference"]["allowed"], true);
        assert_eq!(json["policies"]["attribution"]["required"], true);
    }
}
