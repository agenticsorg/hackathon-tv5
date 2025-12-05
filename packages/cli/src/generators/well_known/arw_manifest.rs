use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArwManifest {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String,
    pub profile: String,
    pub site: SiteInfo,
    pub discovery: DiscoveryLinks,
    pub capabilities: Capabilities,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SiteInfo {
    pub name: String,
    pub description: String,
    pub homepage: String,
    pub contact: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryLinks {
    pub llms_txt: String,
    pub content_index: String,
    pub policies: String,
    pub sitemap: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Capabilities {
    pub machine_views: bool,
    pub chunking: bool,
    pub actions: bool,
    pub oauth: bool,
    pub protocols: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub last_updated: String,
    pub generator: String,
    pub spec_version: String,
}

/// Generate .well-known/arw-manifest.json from llms.txt
pub fn generate<P: AsRef<Path>>(
    site_path: P,
    site_info: &SiteInfo,
    profile: &str,
) -> Result<()> {
    let manifest = ArwManifest {
        schema: "https://arw.dev/schemas/arw-manifest.schema.json".to_string(),
        version: "1.0".to_string(),
        profile: profile.to_string(),
        site: SiteInfo {
            name: site_info.name.clone(),
            description: site_info.description.clone(),
            homepage: site_info.homepage.clone(),
            contact: site_info.contact.clone(),
        },
        discovery: DiscoveryLinks {
            llms_txt: "/llms.txt".to_string(),
            content_index: "/.well-known/arw-content-index.json".to_string(),
            policies: "/.well-known/arw-policies.json".to_string(),
            sitemap: "/sitemap.xml".to_string(),
        },
        capabilities: Capabilities {
            machine_views: true,
            chunking: true,
            actions: false,
            oauth: false,
            protocols: vec![],
        },
        metadata: Metadata {
            last_updated: chrono::Utc::now().to_rfc3339(),
            generator: "arw-cli".to_string(),
            spec_version: "1.0".to_string(),
        },
    };

    let well_known_dir = site_path.as_ref().join(".well-known");
    fs::create_dir_all(&well_known_dir)
        .context("Failed to create .well-known directory")?;

    let output_path = well_known_dir.join("arw-manifest.json");
    let content = serde_json::to_string_pretty(&manifest)?;

    fs::write(&output_path, content)
        .with_context(|| format!("Failed to write arw-manifest.json to {:?}", output_path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_manifest() {
        let site = SiteInfo {
            name: "Test Site".to_string(),
            description: "A test site".to_string(),
            homepage: "https://example.com".to_string(),
            contact: "ai@example.com".to_string(),
        };

        let manifest = ArwManifest {
            schema: "https://arw.dev/schemas/arw-manifest.schema.json".to_string(),
            version: "1.0".to_string(),
            profile: "ARW-1".to_string(),
            site,
            discovery: DiscoveryLinks {
                llms_txt: "/llms.txt".to_string(),
                content_index: "/.well-known/arw-content-index.json".to_string(),
                policies: "/.well-known/arw-policies.json".to_string(),
                sitemap: "/sitemap.xml".to_string(),
            },
            capabilities: Capabilities {
                machine_views: true,
                chunking: true,
                actions: false,
                oauth: false,
                protocols: vec![],
            },
            metadata: Metadata {
                last_updated: "2025-01-08T00:00:00Z".to_string(),
                generator: "arw-cli".to_string(),
                spec_version: "1.0".to_string(),
            },
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        assert!(json.contains("arw-manifest.schema.json"));
        assert!(json.contains("llms_txt"));
        assert!(json.contains("/llms.txt"));
    }
}
