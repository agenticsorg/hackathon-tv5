use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::cli;

#[derive(Debug, Clone)]
pub enum SitemapFormat {
    Json,
    Xml,
}

impl SitemapFormat {
    pub fn from_output(output: &str) -> Self {
        if output.ends_with(".xml") {
            SitemapFormat::Xml
        } else {
            SitemapFormat::Json
        }
    }
}

// Manifest structures for parsing llms.txt
#[derive(Debug, Deserialize, Serialize)]
struct Manifest {
    content: Option<Vec<ContentItem>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ContentItem {
    url: String,
    #[serde(default)]
    priority: Option<String>,
}

pub async fn run(
    source: String,
    output: String,
    _depth: usize,
    base_url: Option<String>,
) -> Result<()> {
    let base = base_url.unwrap_or_else(|| "https://example.com".to_string());
    let format = SitemapFormat::from_output(&output);

    match format {
        SitemapFormat::Json => {
            cli::info("Generating sitemap.llm.json");
            generate_json_sitemap(&source, &output, &base)?;
        }
        SitemapFormat::Xml => {
            cli::info("Generating sitemap.xml");
            generate_xml_sitemap(&source, &output, &base)?;
        }
    }

    cli::success(&format!("Sitemap created: {}", output));
    Ok(())
}

fn generate_json_sitemap(source: &str, output: &str, base_url: &str) -> Result<()> {
    let sitemap = crate::generators::sitemap::generate_sitemap(
        Path::new(source),
        base_url,
        vec![],
    )?;

    let content = serde_json::to_string_pretty(&sitemap)?;
    fs::write(output, content)?;

    Ok(())
}

/// Helper function to generate sitemap from llms.txt manifest
pub fn generate_from_manifest<P: AsRef<Path>>(
    source_path: P,
    output_path: P,
    base_url: &str,
) -> Result<()> {
    generate_xml_sitemap(
        source_path.as_ref().to_str().unwrap(),
        output_path.as_ref().to_str().unwrap(),
        base_url,
    )
}

fn generate_xml_sitemap(source: &str, output: &str, base_url: &str) -> Result<()> {
    let source_path = Path::new(source);

    // Try to load llms.txt manifest from source directory
    let manifest_path = source_path.join("llms.txt");
    let manifest = if manifest_path.exists() {
        cli::info("Found llms.txt - using priorities from manifest");
        let manifest_content = fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read manifest at {:?}", manifest_path))?;
        serde_yaml::from_str::<Manifest>(&manifest_content).ok()
    } else {
        cli::info("No llms.txt found - using default priorities");
        None
    };

    let mut pages = Vec::new();

    // If we have a manifest, use its content items
    if let Some(ref m) = manifest {
        if let Some(ref content) = m.content {
            for item in content {
                // Normalize URL to path
                let url_path = item.url.trim_start_matches('/');

                // Try to find the corresponding file to get lastmod
                let possible_paths = vec![
                    source_path.join(format!("{}.html", url_path)),
                    source_path.join(format!("{}/index.html", url_path)),
                    source_path.join(url_path),
                ];

                let modified = possible_paths.iter()
                    .find(|p| p.exists())
                    .and_then(|p| fs::metadata(p).ok())
                    .and_then(|m| m.modified().ok())
                    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|duration| duration.as_secs())
                    .unwrap_or(0);

                pages.push(crate::generators::sitemap::SitemapEntry {
                    loc: format!("{}{}", base_url.trim_end_matches('/'), &item.url),
                    lastmod: format_timestamp(modified),
                    changefreq: "weekly".to_string(),
                    priority: map_priority(item.priority.as_deref()),
                });
            }
        }
    }

    // If no manifest or no content in manifest, fall back to file scanning
    if pages.is_empty() {
        cli::info("Scanning directory for files...");
        use walkdir::WalkDir;

        for entry in WalkDir::new(source_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "html" || ext == "md" {
                        if let Ok(relative) = path.strip_prefix(source_path) {
                            let url_path = relative.to_string_lossy().to_string();
                            let url_path = url_path.replace('\\', "/");

                            let metadata = fs::metadata(path)?;
                            let modified = metadata
                                .modified()
                                .ok()
                                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|duration| duration.as_secs())
                                .unwrap_or(0);

                            pages.push(crate::generators::sitemap::SitemapEntry {
                                loc: format!("{}/{}", base_url.trim_end_matches('/'), url_path),
                                lastmod: format_timestamp(modified),
                                changefreq: "weekly".to_string(),
                                priority: 0.5, // Default priority when no manifest
                            });
                        }
                    }
                }
            }
        }
    }

    // Generate XML content
    let xml = crate::generators::sitemap::generate_sitemap_xml(pages)?;
    fs::write(output, xml)?;

    Ok(())
}

fn format_timestamp(seconds: u64) -> String {
    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(seconds as i64, 0)
        .unwrap_or_else(|| Utc::now());
    dt.format("%Y-%m-%d").to_string()
}

/// Map ARW priority strings to sitemap.xml numeric priorities
fn map_priority(priority: Option<&str>) -> f32 {
    match priority {
        Some("high") => 1.0,
        Some("medium") => 0.8,
        Some("low") => 0.5,
        _ => 0.5, // Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sitemap_format_detection() {
        assert!(matches!(
            SitemapFormat::from_output("sitemap.xml"),
            SitemapFormat::Xml
        ));
        assert!(matches!(
            SitemapFormat::from_output("sitemap.llm.json"),
            SitemapFormat::Json
        ));
    }

    #[test]
    fn test_map_priority() {
        assert_eq!(map_priority(Some("high")), 1.0);
        assert_eq!(map_priority(Some("medium")), 0.8);
        assert_eq!(map_priority(Some("low")), 0.5);
        assert_eq!(map_priority(None), 0.5);
        assert_eq!(map_priority(Some("unknown")), 0.5);
    }
}
