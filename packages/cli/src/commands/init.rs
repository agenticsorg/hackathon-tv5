use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, Input};
use std::fs;
use std::path::Path;

use crate::cli;
use crate::generators;
use crate::generators::llms_txt::{PolicyInfo, SiteInfo};
use crate::utils::config::ArwConfig;

#[allow(unused_imports)]
use crate::utils::config::{PolicyConfig, SiteConfig};
#[allow(unused_imports)]
use dialoguer::MultiSelect;

pub async fn run(path: String, yes: bool, with_manifests: bool, base_url: Option<String>) -> Result<()> {
    let site_path = Path::new(&path);

    // Create directory if it doesn't exist
    if !site_path.exists() {
        fs::create_dir_all(site_path)
            .with_context(|| format!("Failed to create directory {:?}", site_path))?;
    }

    // Check if already initialized (config is in root, not in site_path)
    if ArwConfig::exists(".") {
        if !yes {
            let overwrite = Confirm::new()
                .with_prompt("ARW is already initialized. Overwrite?")
                .default(false)
                .interact()?;

            if !overwrite {
                cli::info("Initialization cancelled");
                return Ok(());
            }
        }
    }

    cli::info(&format!("Initializing ARW in: {}", site_path.display()));
    println!();

    // Gather site information
    let (site_info, policy_info) = if yes {
        (
            SiteInfo {
                name: "My Website".to_string(),
                description: "Website description".to_string(),
                homepage: "https://example.com".to_string(),
                contact: "ai@example.com".to_string(),
            },
            PolicyInfo {
                training_allowed: false,
                inference_allowed: true,
                attribution_required: true,
            },
        )
    } else {
        gather_site_info()?
    };

    let total_steps = if with_manifests { 5 } else { 3 };

    // Generate llms.txt (PRIMARY SOURCE OF TRUTH)
    cli::step(1, total_steps, "Generating llms.txt");
    generators::llms_txt::generate(site_path, &site_info, &policy_info)?;
    cli::success("llms.txt created (primary source of truth)");
    println!();

    // Create .arw directory in root with CLI preferences (optional)
    cli::step(2, total_steps, "Creating .arw/config.yaml (CLI preferences only)");
    let config = ArwConfig::default();
    config.save(".")?;  // Save to root, not in public/
    cli::success("CLI configuration saved to .arw/config.yaml");
    println!();

    // Create example machine view
    cli::step(3, total_steps, "Creating example machine view");
    create_example_machine_view(site_path)?;
    cli::success("index.llm.md created");
    println!();

    // Generate manifests if requested
    if with_manifests {
        let url = base_url.as_deref().unwrap_or_else(|| {
            site_info.homepage.as_str()
        });

        cli::step(4, total_steps, "Generating sitemap.xml");
        match generators::manifests::generate_sitemap(site_path, url) {
            Ok(_) => cli::success("sitemap.xml created"),
            Err(e) => cli::warn(&format!("Failed to generate sitemap.xml: {}", e)),
        }
        println!();

        cli::step(5, total_steps, "Generating .well-known/arw-manifest.json");
        match generators::manifests::generate_arw_manifest(site_path, url) {
            Ok(_) => cli::success(".well-known/arw-manifest.json created"),
            Err(e) => cli::warn(&format!("Failed to generate .well-known/arw-manifest.json: {}", e)),
        }
        println!();
    }

    // Print next steps
    print_next_steps(with_manifests);

    Ok(())
}

fn gather_site_info() -> Result<(SiteInfo, PolicyInfo)> {
    println!("Please provide some information about your site:\n");

    let name: String = Input::new()
        .with_prompt("Site name")
        .default("My Website".to_string())
        .interact_text()?;

    let description: String = Input::new()
        .with_prompt("Description")
        .default("Website description".to_string())
        .interact_text()?;

    let homepage: String = Input::new()
        .with_prompt("Homepage URL")
        .default("https://example.com".to_string())
        .interact_text()?;

    let contact: String = Input::new()
        .with_prompt("Contact email")
        .default("ai@example.com".to_string())
        .interact_text()?;

    println!("\n📋 Content Policy Configuration:\n");

    let training_allowed = Confirm::new()
        .with_prompt("Allow AI training on content?")
        .default(false)
        .interact()?;

    let inference_allowed = Confirm::new()
        .with_prompt("Allow AI inference (answering queries)?")
        .default(true)
        .interact()?;

    let attribution_required = Confirm::new()
        .with_prompt("Require attribution in AI responses?")
        .default(true)
        .interact()?;

    let site_info = SiteInfo {
        name,
        description,
        homepage,
        contact,
    };

    let policy_info = PolicyInfo {
        training_allowed,
        inference_allowed,
        attribution_required,
    };

    Ok((site_info, policy_info))
}

// Legacy function - no longer used
// Site info now gathered via gather_site_info() and stored in llms.txt
#[allow(dead_code)]
fn gather_config() -> Result<ArwConfig> {
    unimplemented!("This function is deprecated. Use gather_site_info() instead.")
}

fn create_example_machine_view(site_path: &Path) -> Result<()> {
    let content = r#"# Homepage

This is an example machine view file generated by ARW CLI.

<!-- chunk: introduction -->
## Introduction

Machine views are Markdown files optimized for AI agents to read. They provide clean, structured content without HTML complexity.

<!-- chunk: getting-started -->
## Getting Started

To create your own machine views:

1. Use `arw generate <source>` to convert HTML files
2. Edit the generated `.llm.md` files to optimize for agents
3. Add chunk comments to mark addressable sections
4. Update `llms.txt` to reference your machine views

<!-- chunk: best-practices -->
## Best Practices

- Keep content concise and well-structured
- Use semantic headings (H1, H2, H3)
- Include all essential information
- Add chunk comments for important sections
- Maintain consistency across your machine views

For more information, see: https://github.com/agent-ready-web/agent-ready-web
"#;

    let example_path = site_path.join("index.llm.md");
    fs::write(example_path, content).context("Failed to create example machine view")?;

    Ok(())
}

fn print_next_steps(manifests_generated: bool) {
    println!("{}", "🚀 Next Steps:".bold());
    println!();
    println!("  1. Review and customize llms.txt (single source of truth):");
    println!("     • Add your pages to the content section");
    println!("     • Set priorities: high, medium, or low");
    println!("     • Update policies as needed");
    println!();
    println!("  2. Generate machine views from your content:");
    println!("     arw generate <source-directory> --recursive --sync");
    println!();

    if !manifests_generated {
        println!("  3. Generate sitemap.xml and .well-known/arw-manifest.json:");
        println!("     arw init --with-manifests --base-url https://yoursite.com");
        println!("     (or run 'arw sitemap' separately)");
        println!();
        println!("  4. Generate robots.txt from llms.txt:");
        println!("     arw robots");
        println!();
        println!("  5. Validate your implementation:");
        println!("     arw validate --strict");
    } else {
        println!("  3. Generate robots.txt from llms.txt:");
        println!("     arw robots");
        println!();
        println!("  4. Validate your implementation:");
        println!("     arw validate --strict");
    }

    println!();
    println!("📖 Learn more: https://github.com/agent-ready-web/agent-ready-web");
    println!();
    println!("{}", "💡 Tip: llms.txt is your single source of truth.".yellow());
    println!("   All other files (sitemap, robots.txt, .well-known/arw-manifest.json) are generated from it.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Mutex to serialize tests that change working directory
    // This prevents tests from interfering with each other when run in parallel
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[tokio::test]
    async fn test_init_creates_public_directory() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Run init with default public directory
        let result = run("public".to_string(), true, false, None).await;
        assert!(result.is_ok(), "Init should succeed");

        // Verify public directory was created
        let public_dir = temp_dir.path().join("public");
        assert!(public_dir.exists(), "public directory should be created");
        assert!(public_dir.is_dir(), "public should be a directory");
    }

    #[tokio::test]
    async fn test_init_creates_files_in_public() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        run("public".to_string(), true, false, None).await.unwrap();

        let public_dir = temp_dir.path().join("public");

        // Check llms.txt was created in public/
        let llms_path = public_dir.join("llms.txt");
        assert!(llms_path.exists(), "llms.txt should be created in public/");

        // Check index.llm.md was created in public/
        let index_path = public_dir.join("index.llm.md");
        assert!(index_path.exists(), "index.llm.md should be created in public/");
    }

    #[tokio::test]
    async fn test_init_creates_config_in_root() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        run("public".to_string(), true, false, None).await.unwrap();

        // Check .arw/config.yaml was created in root (not in public/)
        let config_path = temp_dir.path().join(".arw").join("config.yaml");
        assert!(config_path.exists(), ".arw/config.yaml should be created in root");

        // Verify it's NOT in public/
        let wrong_config = temp_dir.path().join("public").join(".arw");
        assert!(!wrong_config.exists(), ".arw should NOT be in public/");
    }

    #[tokio::test]
    async fn test_init_llms_txt_has_correct_structure() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        run("public".to_string(), true, false, None).await.unwrap();

        let llms_content = fs::read_to_string(
            temp_dir.path().join("public").join("llms.txt")
        ).unwrap();

        // Verify essential sections exist
        assert!(llms_content.contains("version: 1.0"));
        assert!(llms_content.contains("profile: ARW-1"));
        assert!(llms_content.contains("site:"));
        assert!(llms_content.contains("content:"));
        assert!(llms_content.contains("policies:"));
    }

    #[tokio::test]
    async fn test_init_generates_valid_yaml() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        run("public".to_string(), true, false, None).await.unwrap();

        let llms_content = fs::read_to_string(
            temp_dir.path().join("public").join("llms.txt")
        ).unwrap();

        // Verify it's valid YAML
        let parsed: serde_yaml::Value = serde_yaml::from_str(&llms_content).unwrap();
        // Version can be a number or string in YAML
        assert!(parsed["version"].as_str().is_some() || parsed["version"].as_f64().is_some());
        assert!(parsed["site"].as_mapping().is_some());
    }

    #[tokio::test]
    async fn test_init_custom_directory_path() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Use custom directory instead of "public"
        run("custom-dir".to_string(), true, false, None).await.unwrap();

        let custom_dir = temp_dir.path().join("custom-dir");
        assert!(custom_dir.exists(), "custom directory should be created");

        // Files should be in custom directory
        assert!(custom_dir.join("llms.txt").exists());
        assert!(custom_dir.join("index.llm.md").exists());

        // Config still in root
        assert!(temp_dir.path().join(".arw").join("config.yaml").exists());
    }
}
