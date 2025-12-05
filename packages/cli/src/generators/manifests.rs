use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Generate sitemap.xml from llms.txt
pub fn generate_sitemap(site_path: &Path, base_url: &str) -> Result<()> {
    let llms_txt_path = site_path.join("llms.txt");

    if !llms_txt_path.exists() {
        anyhow::bail!("llms.txt not found. Run 'arw init' first.");
    }

    let llms_content = fs::read_to_string(&llms_txt_path)
        .with_context(|| format!("Failed to read llms.txt from {:?}", llms_txt_path))?;

    // Parse llms.txt to extract URLs
    let urls = parse_urls_from_llms_txt(&llms_content)?;

    // Generate sitemap XML
    let sitemap = create_sitemap_xml(&urls, base_url);

    // Write sitemap.xml
    let sitemap_path = site_path.join("sitemap.xml");
    fs::write(&sitemap_path, sitemap)
        .with_context(|| format!("Failed to write sitemap.xml to {:?}", sitemap_path))?;

    Ok(())
}

/// Generate .well-known/arw-manifest.json from llms.txt (ARW-1 standard)
pub fn generate_arw_manifest(site_path: &Path, base_url: &str) -> Result<()> {
    let llms_txt_path = site_path.join("llms.txt");

    if !llms_txt_path.exists() {
        anyhow::bail!("llms.txt not found. Run 'arw init' first.");
    }

    let llms_content = fs::read_to_string(&llms_txt_path)
        .with_context(|| format!("Failed to read llms.txt from {:?}", llms_txt_path))?;

    // Parse llms.txt to extract full manifest
    let manifest = parse_arw_manifest(&llms_content, base_url)?;

    // Create .well-known directory if it doesn't exist
    let well_known_dir = site_path.join(".well-known");
    fs::create_dir_all(&well_known_dir)
        .with_context(|| format!("Failed to create .well-known directory at {:?}", well_known_dir))?;

    // Write .well-known/arw-manifest.json
    let json = serde_json::to_string_pretty(&manifest)
        .with_context(|| "Failed to serialize arw-manifest.json")?;

    let manifest_path = well_known_dir.join("arw-manifest.json");
    fs::write(&manifest_path, json)
        .with_context(|| format!("Failed to write arw-manifest.json to {:?}", manifest_path))?;

    Ok(())
}

/// Parse URLs from llms.txt content section
fn parse_urls_from_llms_txt(content: &str) -> Result<Vec<String>> {
    let mut urls = Vec::new();
    let mut in_content_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("content:") {
            in_content_section = true;
            continue;
        }

        if in_content_section {
            // Stop at next top-level section
            if !trimmed.is_empty() && !trimmed.starts_with('-') && !trimmed.starts_with(' ') && trimmed.contains(':') {
                break;
            }

            // Extract URL from "  - url: /path"
            if trimmed.starts_with("url:") {
                if let Some(url) = trimmed.strip_prefix("url:") {
                    urls.push(url.trim().to_string());
                }
            }
        }
    }

    Ok(urls)
}

/// Create sitemap XML from URLs
fn create_sitemap_xml(urls: &[String], base_url: &str) -> String {
    let base_url = base_url.trim_end_matches('/');
    let mut sitemap = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    sitemap.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    for url in urls {
        let full_url = if url == "/" {
            base_url.to_string()
        } else {
            format!("{}{}", base_url, url)
        };

        sitemap.push_str("  <url>\n");
        sitemap.push_str(&format!("    <loc>{}</loc>\n", escape_xml(&full_url)));
        sitemap.push_str("    <changefreq>weekly</changefreq>\n");
        sitemap.push_str("    <priority>0.8</priority>\n");
        sitemap.push_str("  </url>\n");
    }

    sitemap.push_str("</urlset>\n");
    sitemap
}

/// ARW Manifest structure (ARW-1 standard)
#[derive(Debug, Serialize, Deserialize)]
struct ARWManifest {
    version: String,
    profile: String,
    site: SiteInfo,
    content: Vec<ContentEntry>,
    policies: Policies,
}

#[derive(Debug, Serialize, Deserialize)]
struct SiteInfo {
    name: String,
    description: String,
    homepage: String,
    contact: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContentEntry {
    url: String,
    machine_view: String,
    purpose: String,
    priority: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Policies {
    training: TrainingPolicy,
    inference: InferencePolicy,
    attribution: AttributionPolicy,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrainingPolicy {
    allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InferencePolicy {
    allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    restrictions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AttributionPolicy {
    required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
}

/// Parse ARW manifest from llms.txt
fn parse_arw_manifest(content: &str, base_url: &str) -> Result<ARWManifest> {
    // Parse YAML-like structure from llms.txt
    let mut site_name = "My Website".to_string();
    let mut site_description = "Website description".to_string();
    let mut site_homepage = base_url.to_string();
    let mut site_contact = "ai@example.com".to_string();
    let mut content_entries = Vec::new();

    // Policy defaults
    let mut training_allowed = false;
    let mut inference_allowed = true;
    let mut attribution_required = true;

    let mut current_section = "";
    let mut current_subsection = "";
    let mut current_entry: Option<ContentEntry> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Detect top-level sections
        if trimmed.starts_with("site:") {
            current_section = "site";
            current_subsection = "";
            continue;
        } else if trimmed.starts_with("content:") {
            current_section = "content";
            current_subsection = "";
            continue;
        } else if trimmed.starts_with("policies:") {
            current_section = "policies";
            current_subsection = "";
            continue;
        }

        // Parse site info
        if current_section == "site" {
            if let Some(value) = trimmed.strip_prefix("name:") {
                site_name = value.trim().trim_matches('"').to_string();
            } else if let Some(value) = trimmed.strip_prefix("description:") {
                site_description = value.trim().trim_matches('"').to_string();
            } else if let Some(value) = trimmed.strip_prefix("homepage:") {
                site_homepage = value.trim().trim_matches('"').to_string();
            } else if let Some(value) = trimmed.strip_prefix("contact:") {
                site_contact = value.trim().trim_matches('"').to_string();
            }
        }

        // Parse content entries
        if current_section == "content" {
            if trimmed.starts_with("- url:") || trimmed.starts_with("url:") {
                // Save previous entry
                if let Some(entry) = current_entry.take() {
                    content_entries.push(entry);
                }

                let url = trimmed
                    .strip_prefix("- url:")
                    .or_else(|| trimmed.strip_prefix("url:"))
                    .unwrap()
                    .trim()
                    .to_string();

                current_entry = Some(ContentEntry {
                    url,
                    machine_view: String::new(),
                    purpose: "page".to_string(),
                    priority: "medium".to_string(),
                });
            } else if let Some(ref mut entry) = current_entry {
                if let Some(value) = trimmed.strip_prefix("machine_view:") {
                    entry.machine_view = value.trim().to_string();
                } else if let Some(value) = trimmed.strip_prefix("purpose:") {
                    entry.purpose = value.trim().to_string();
                } else if let Some(value) = trimmed.strip_prefix("priority:") {
                    entry.priority = value.trim().to_string();
                }
            }
        }

        // Parse policies
        if current_section == "policies" {
            if trimmed.starts_with("training:") {
                current_subsection = "training";
            } else if trimmed.starts_with("inference:") {
                current_subsection = "inference";
            } else if trimmed.starts_with("attribution:") {
                current_subsection = "attribution";
            } else if let Some(value) = trimmed.strip_prefix("allowed:") {
                let allowed = value.trim() == "true";
                match current_subsection {
                    "training" => training_allowed = allowed,
                    "inference" => inference_allowed = allowed,
                    _ => {}
                }
            } else if let Some(value) = trimmed.strip_prefix("required:") {
                if current_subsection == "attribution" {
                    attribution_required = value.trim() == "true";
                }
            }
        }
    }

    // Save last content entry
    if let Some(entry) = current_entry {
        content_entries.push(entry);
    }

    Ok(ARWManifest {
        version: "0.1".to_string(),
        profile: "ARW-1".to_string(),
        site: SiteInfo {
            name: site_name,
            description: site_description,
            homepage: site_homepage,
            contact: site_contact,
        },
        content: content_entries,
        policies: Policies {
            training: TrainingPolicy {
                allowed: training_allowed,
                note: if !training_allowed {
                    Some("Content not licensed for model training".to_string())
                } else {
                    None
                },
            },
            inference: InferencePolicy {
                allowed: inference_allowed,
                restrictions: if attribution_required {
                    Some(vec!["attribution_required".to_string()])
                } else {
                    None
                },
            },
            attribution: AttributionPolicy {
                required: attribution_required,
                format: if attribution_required {
                    Some("link".to_string())
                } else {
                    None
                },
            },
        },
    })
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_urls_from_llms_txt() {
        let content = r#"
version: 1.0
profile: ARW-1

site:
  name: "Test Site"

content:
  - url: /
    machine_view: /index.llm.md
  - url: /about
    machine_view: /about.llm.md

policies:
  training:
    allowed: false
"#;

        let urls = parse_urls_from_llms_txt(content).unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "/");
        assert_eq!(urls[1], "/about");
    }

    #[test]
    fn test_create_sitemap_xml() {
        let urls = vec!["/".to_string(), "/about".to_string()];
        let sitemap = create_sitemap_xml(&urls, "https://example.com");

        assert!(sitemap.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(sitemap.contains("<urlset"));
        assert!(sitemap.contains("<loc>https://example.com</loc>"));
        assert!(sitemap.contains("<loc>https://example.com/about</loc>"));
    }

    #[test]
    fn test_escape_xml() {
        let escaped = escape_xml("Test & <script>'alert'</script>");
        assert_eq!(escaped, "Test &amp; &lt;script&gt;&apos;alert&apos;&lt;/script&gt;");
    }
}
