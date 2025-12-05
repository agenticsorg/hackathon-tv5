use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::validators::llms_txt::ValidationError;

/// Validates cross-file consistency in ARW implementation
pub struct ConsistencyValidator {
    base_path: String,
}

impl ConsistencyValidator {
    pub fn new(base_path: String) -> Self {
        Self { base_path }
    }

    /// Run all consistency checks
    pub async fn validate_all(&self) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Load and parse llms.txt
        let llms_txt_path = Path::new(&self.base_path).join("llms.txt");
        if !llms_txt_path.exists() {
            errors.push(ValidationError {
                path: "llms.txt".to_string(),
                message: "llms.txt not found".to_string(),
            });
            return Ok(errors);
        }

        let manifest_content = fs::read_to_string(&llms_txt_path)
            .context("Failed to read llms.txt")?;

        let manifest: serde_json::Value = serde_yaml::from_str(&manifest_content)
            .context("Failed to parse llms.txt")?;

        // Run consistency checks
        errors.extend(self.validate_machine_view_files(&manifest)?);
        errors.extend(self.validate_chunk_consistency(&manifest).await?);
        errors.extend(self.validate_robots_consistency(&manifest)?);

        Ok(errors)
    }

    /// Validate that all machine_view files exist
    pub fn validate_machine_view_files(
        &self,
        manifest: &serde_json::Value,
    ) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Some(content) = manifest.get("content").and_then(|c| c.as_array()) {
            for (idx, item) in content.iter().enumerate() {
                if let Some(machine_view) = item.get("machine_view").and_then(|m| m.as_str()) {
                    // Check if file exists
                    let file_path = Path::new(&self.base_path).join(machine_view.trim_start_matches('/'));

                    if !file_path.exists() {
                        errors.push(ValidationError {
                            path: format!("content[{}].machine_view", idx),
                            message: format!(
                                "Machine view file not found: {}",
                                machine_view
                            ),
                        });
                    } else {
                        // Check file is readable
                        if fs::read_to_string(&file_path).is_err() {
                            errors.push(ValidationError {
                                path: format!("content[{}].machine_view", idx),
                                message: format!(
                                    "Machine view file not readable: {}",
                                    machine_view
                                ),
                            });
                        }
                    }
                }
            }
        }

        Ok(errors)
    }

    /// Validate chunk consistency between HTML, manifest, and .llm.md
    pub async fn validate_chunk_consistency(
        &self,
        manifest: &serde_json::Value,
    ) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Some(content) = manifest.get("content").and_then(|c| c.as_array()) {
            for (idx, item) in content.iter().enumerate() {
                // Get declared chunks from manifest
                let declared_chunks: HashSet<String> = item
                    .get("chunks")
                    .and_then(|c| c.as_array())
                    .map(|chunks| {
                        chunks
                            .iter()
                            .filter_map(|chunk| {
                                chunk.get("id").and_then(|id| id.as_str()).map(String::from)
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                // Skip if no chunks declared
                if declared_chunks.is_empty() {
                    continue;
                }

                // Get machine view path
                if let Some(machine_view) = item.get("machine_view").and_then(|m| m.as_str()) {
                    let md_path = Path::new(&self.base_path)
                        .join(machine_view.trim_start_matches('/'));

                    if md_path.exists() {
                        // Extract chunks from .llm.md
                        let md_chunks = self.extract_markdown_chunks(&md_path)?;

                        // Check for missing chunks in markdown
                        for chunk_id in &declared_chunks {
                            if !md_chunks.contains(chunk_id) {
                                errors.push(ValidationError {
                                    path: format!("content[{}].chunks", idx),
                                    message: format!(
                                        "Chunk '{}' declared in manifest but not found in {}",
                                        chunk_id, machine_view
                                    ),
                                });
                            }
                        }

                        // Check for undeclared chunks in markdown
                        for chunk_id in &md_chunks {
                            if !declared_chunks.contains(chunk_id) {
                                errors.push(ValidationError {
                                    path: format!("content[{}].chunks", idx),
                                    message: format!(
                                        "Chunk '{}' found in {} but not declared in manifest",
                                        chunk_id, machine_view
                                    ),
                                });
                            }
                        }
                    }
                }

                // Check HTML source if URL is local
                if let Some(url) = item.get("url").and_then(|u| u.as_str()) {
                    if url.starts_with('/') {
                        let html_path = Path::new(&self.base_path)
                            .join(url.trim_start_matches('/'))
                            .with_extension("html");

                        if html_path.exists() {
                            let html_chunks = self.extract_html_chunks(&html_path)?;

                            // Check for missing chunks in HTML
                            for chunk_id in &declared_chunks {
                                if !html_chunks.contains(chunk_id) {
                                    errors.push(ValidationError {
                                        path: format!("content[{}].chunks", idx),
                                        message: format!(
                                            "Chunk '{}' declared in manifest but not found in HTML {}",
                                            chunk_id, url
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(errors)
    }

    /// Validate robots.txt matches policy
    pub fn validate_robots_consistency(
        &self,
        manifest: &serde_json::Value,
    ) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        let robots_path = Path::new(&self.base_path).join("robots.txt");
        if !robots_path.exists() {
            // robots.txt is optional, so just return
            return Ok(errors);
        }

        let robots_content = fs::read_to_string(&robots_path)
            .context("Failed to read robots.txt")?;

        // Check policy consistency
        if let Some(policies) = manifest.get("policies") {
            // Check training policy
            if let Some(training) = policies.get("training") {
                if let Some(allowed) = training.get("allowed").and_then(|a| a.as_bool()) {
                    if !allowed {
                        // Should disallow training bots
                        if !robots_content.contains("User-agent: GPTBot")
                            || !robots_content.contains("Disallow: /")
                        {
                            errors.push(ValidationError {
                                path: "robots.txt".to_string(),
                                message: "robots.txt does not block training agents as specified in policy".to_string(),
                            });
                        }
                    }
                }
            }

            // Check for ARW hints
            if !robots_content.contains("llms.txt")
                && !robots_content.contains("Agent-Ready Web")
            {
                errors.push(ValidationError {
                    path: "robots.txt".to_string(),
                    message: "robots.txt missing ARW discovery hints".to_string(),
                });
            }
        }

        Ok(errors)
    }

    /// Extract chunk IDs from markdown file
    fn extract_markdown_chunks(&self, path: &Path) -> Result<HashSet<String>> {
        let content = fs::read_to_string(path)?;
        let mut chunks = HashSet::new();

        // Look for <!-- chunk: id --> markers
        for line in content.lines() {
            if line.contains("<!-- chunk:") || line.contains("<!--chunk:") {
                if let Some(start) = line.find("chunk:") {
                    let after_marker = &line[start + 6..];
                    if let Some(end) = after_marker.find("-->") {
                        let chunk_id = after_marker[..end].trim();
                        chunks.insert(chunk_id.to_string());
                    }
                }
            }
        }

        Ok(chunks)
    }

    /// Extract chunk IDs from HTML file
    fn extract_html_chunks(&self, path: &Path) -> Result<HashSet<String>> {
        let content = fs::read_to_string(path)?;
        let mut chunks = HashSet::new();

        // Look for data-chunk-id attributes
        for line in content.lines() {
            if line.contains("data-chunk-id") {
                // Simple regex-free extraction
                if let Some(start) = line.find("data-chunk-id=\"") {
                    let after_attr = &line[start + 15..];
                    if let Some(end) = after_attr.find('"') {
                        let chunk_id = &after_attr[..end];
                        chunks.insert(chunk_id.to_string());
                    }
                }
            }
        }

        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extract_markdown_chunks() {
        let temp_dir = TempDir::new().unwrap();
        let md_path = temp_dir.path().join("test.llm.md");

        fs::write(
            &md_path,
            r#"
# Page Title

<!-- chunk: intro -->
Introduction text

<!-- chunk: main-content -->
Main content here
"#,
        )
        .unwrap();

        let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
        let chunks = validator.extract_markdown_chunks(&md_path).unwrap();

        assert_eq!(chunks.len(), 2);
        assert!(chunks.contains("intro"));
        assert!(chunks.contains("main-content"));
    }

    #[test]
    fn test_extract_html_chunks() {
        let temp_dir = TempDir::new().unwrap();
        let html_path = temp_dir.path().join("test.html");

        fs::write(
            &html_path,
            r#"
<html>
<body>
    <section data-chunk-id="intro">
        <h1>Introduction</h1>
    </section>
    <section data-chunk-id="main-content">
        <p>Main content</p>
    </section>
</body>
</html>
"#,
        )
        .unwrap();

        let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
        let chunks = validator.extract_html_chunks(&html_path).unwrap();

        assert_eq!(chunks.len(), 2);
        assert!(chunks.contains("intro"));
        assert!(chunks.contains("main-content"));
    }
}
