use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArwContentIndex {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String,
    pub total_items: usize,
    pub items: Vec<ContentItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentItem {
    pub url: String,
    pub machine_view: String,
    pub purpose: String,
    pub priority: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunks: Option<Vec<ChunkInfo>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChunkInfo {
    pub id: String,
    pub heading: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination {
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

/// Generate .well-known/arw-content-index.json from llms.txt
pub fn generate<P: AsRef<Path>>(
    site_path: P,
    content_items: Vec<ContentItem>,
) -> Result<()> {
    let index = ArwContentIndex {
        schema: "https://arw.dev/schemas/arw-content-index.schema.json".to_string(),
        version: "1.0".to_string(),
        total_items: content_items.len(),
        items: content_items,
        pagination: None, // Single page for now
    };

    let well_known_dir = site_path.as_ref().join(".well-known");
    fs::create_dir_all(&well_known_dir)
        .context("Failed to create .well-known directory")?;

    let output_path = well_known_dir.join("arw-content-index.json");
    let content = serde_json::to_string_pretty(&index)?;

    fs::write(&output_path, content)
        .with_context(|| format!("Failed to write arw-content-index.json to {:?}", output_path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_content_index() {
        let items = vec![
            ContentItem {
                url: "/".to_string(),
                machine_view: "/index.llm.md".to_string(),
                purpose: "homepage".to_string(),
                priority: "high".to_string(),
                chunks: None,
            },
        ];

        let index = ArwContentIndex {
            schema: "https://arw.dev/schemas/arw-content-index.schema.json".to_string(),
            version: "1.0".to_string(),
            total_items: items.len(),
            items,
            pagination: None,
        };

        let json = serde_json::to_string_pretty(&index).unwrap();
        assert!(json.contains("arw-content-index.schema.json"));
        assert!(json.contains("total_items"));
    }
}
