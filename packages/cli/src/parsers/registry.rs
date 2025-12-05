//! Parser registry for managing and selecting content parsers by file type

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use super::{ContentParser, ExtractedContent};
use super::typescript::TypeScriptParser;

/// HTML parser implementation
pub struct HtmlParser;

impl ContentParser for HtmlParser {
    fn name(&self) -> &'static str {
        "html"
    }

    fn extensions(&self) -> &[&'static str] {
        &["html", "htm"]
    }

    fn parse(&self, content: &str, source_path: &Path) -> Result<ExtractedContent> {
        let mut extracted = ExtractedContent::new();

        let ext = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("html");

        extracted.metadata.extension = ext.to_string();
        extracted.metadata.parser = self.name().to_string();

        // Extract title
        extracted.title = super::html::extract_title(content);

        // Convert HTML to markdown
        extracted.content = html2md::parse_html(content);

        Ok(extracted)
    }
}

/// Markdown/MDX parser implementation
pub struct MarkdownParser;

impl ContentParser for MarkdownParser {
    fn name(&self) -> &'static str {
        "markdown"
    }

    fn extensions(&self) -> &[&'static str] {
        &["md", "mdx", "markdown"]
    }

    fn parse(&self, content: &str, source_path: &Path) -> Result<ExtractedContent> {
        let mut extracted = ExtractedContent::new();

        let ext = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("md");

        extracted.metadata.extension = ext.to_string();
        extracted.metadata.parser = self.name().to_string();

        // Parse frontmatter if present
        let (frontmatter, body) = parse_frontmatter(content);

        if let Some(fm) = frontmatter {
            extracted.metadata.frontmatter = Some(fm.clone());

            // Try to extract title from frontmatter
            if let Some(title) = extract_frontmatter_field(&fm, "title") {
                extracted.title = Some(title);
            }

            // Try to extract description from frontmatter
            if let Some(desc) = extract_frontmatter_field(&fm, "description") {
                extracted.description = Some(desc);
            }
        }

        // If no title from frontmatter, try first heading
        if extracted.title.is_none() {
            extracted.title = extract_first_heading(body);
        }

        // For MDX, strip JSX components but keep the content
        let clean_content = if ext == "mdx" {
            strip_jsx_from_mdx(body)
        } else {
            body.to_string()
        };

        extracted.content = clean_content;

        // Extract sections
        extracted.sections = extract_markdown_sections(body);

        Ok(extracted)
    }
}

/// Parse YAML frontmatter from markdown content
fn parse_frontmatter(content: &str) -> (Option<String>, &str) {
    if !content.starts_with("---") {
        return (None, content);
    }

    // Find the closing ---
    if let Some(end) = content[3..].find("\n---") {
        let frontmatter = &content[3..end + 3];
        let body = &content[end + 7..]; // Skip past closing --- and newline
        (Some(frontmatter.trim().to_string()), body.trim_start())
    } else {
        (None, content)
    }
}

/// Extract a field from YAML frontmatter
fn extract_frontmatter_field(frontmatter: &str, field: &str) -> Option<String> {
    for line in frontmatter.lines() {
        let line = line.trim();
        if line.starts_with(field) {
            if let Some(value) = line.strip_prefix(field) {
                let value = value.trim_start_matches(':').trim();
                // Remove quotes if present
                let value = value.trim_matches('"').trim_matches('\'');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// Extract first heading from markdown
fn extract_first_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            return Some(line.trim_start_matches('#').trim().to_string());
        }
    }
    None
}

/// Strip JSX components from MDX content
fn strip_jsx_from_mdx(content: &str) -> String {
    let mut result = String::new();
    let mut in_jsx_block = false;
    let mut jsx_depth = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Check for JSX block start (standalone component on its own line)
        if !in_jsx_block && trimmed.starts_with('<') && !trimmed.starts_with("<!") {
            // Check if it's a self-closing or has content
            if trimmed.ends_with("/>") || trimmed.ends_with('>') {
                // Check for opening tags
                if trimmed.contains("</") || trimmed.ends_with("/>") {
                    // Self-contained JSX, skip the line
                    continue;
                } else {
                    in_jsx_block = true;
                    jsx_depth = 1;
                    continue;
                }
            }
        }

        if in_jsx_block {
            // Count tag depth
            for c in trimmed.chars() {
                if c == '<' {
                    jsx_depth += 1;
                } else if c == '>' && trimmed.contains("</") {
                    jsx_depth -= 1;
                }
            }

            if jsx_depth <= 0 {
                in_jsx_block = false;
                jsx_depth = 0;
            }
            continue;
        }

        // Regular content line
        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Extract sections from markdown content
fn extract_markdown_sections(content: &str) -> Vec<super::Section> {
    let mut sections = Vec::new();
    let mut current_section: Option<(u8, String, String)> = None;

    for line in content.lines() {
        if line.starts_with('#') {
            // Save previous section
            if let Some((level, title, content)) = current_section.take() {
                sections.push(super::Section {
                    level,
                    title,
                    content: content.trim().to_string(),
                });
            }

            // Start new section
            let level = line.chars().take_while(|&c| c == '#').count() as u8;
            let title = line.trim_start_matches('#').trim().to_string();
            current_section = Some((level, title, String::new()));
        } else if let Some((_, _, ref mut content)) = current_section {
            content.push_str(line);
            content.push('\n');
        }
    }

    // Don't forget the last section
    if let Some((level, title, content)) = current_section {
        sections.push(super::Section {
            level,
            title,
            content: content.trim().to_string(),
        });
    }

    sections
}

/// Registry of content parsers
pub struct ParserRegistry {
    parsers: Vec<Arc<dyn ContentParser>>,
    extension_map: HashMap<String, usize>,
}

impl ParserRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
            extension_map: HashMap::new(),
        }
    }

    /// Register a parser
    pub fn register<P: ContentParser + 'static>(&mut self, parser: P) {
        let index = self.parsers.len();
        let extensions = parser.extensions().to_vec();

        self.parsers.push(Arc::new(parser));

        for ext in extensions {
            self.extension_map.insert(ext.to_lowercase(), index);
        }
    }

    /// Get a parser for a file path
    pub fn get_parser(&self, path: &Path) -> Option<Arc<dyn ContentParser>> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        let index = self.extension_map.get(&ext)?;
        self.parsers.get(*index).cloned()
    }

    /// Parse content from a file
    pub fn parse(&self, content: &str, path: &Path) -> Result<ExtractedContent> {
        let parser = self.get_parser(path)
            .ok_or_else(|| anyhow!("No parser found for file: {:?}", path))?;
        parser.parse(content, path)
    }

    /// Get all supported extensions
    pub fn supported_extensions(&self) -> Vec<&str> {
        self.parsers
            .iter()
            .flat_map(|p| p.extensions().iter().copied())
            .collect()
    }

    /// Check if a file is supported
    pub fn is_supported(&self, path: &Path) -> bool {
        self.get_parser(path).is_some()
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register(HtmlParser);
        registry.register(MarkdownParser);
        registry.register(TypeScriptParser);
        registry
    }
}

/// Get the default parser registry with all built-in parsers
pub fn get_default_registry() -> ParserRegistry {
    ParserRegistry::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_registry_html() {
        let registry = get_default_registry();
        let path = PathBuf::from("test.html");
        assert!(registry.is_supported(&path));

        let content = "<html><head><title>Test</title></head><body><p>Hello</p></body></html>";
        let result = registry.parse(content, &path).unwrap();
        assert_eq!(result.title, Some("Test".to_string()));
    }

    #[test]
    fn test_registry_markdown() {
        let registry = get_default_registry();
        let path = PathBuf::from("test.md");
        assert!(registry.is_supported(&path));

        let content = "# Hello World\n\nThis is content.";
        let result = registry.parse(content, &path).unwrap();
        assert_eq!(result.title, Some("Hello World".to_string()));
    }

    #[test]
    fn test_registry_mdx_with_frontmatter() {
        let registry = get_default_registry();
        let path = PathBuf::from("test.mdx");
        assert!(registry.is_supported(&path));

        let content = r#"---
title: My Page
description: A test page
---

# Content Heading

Some content here.
"#;
        let result = registry.parse(content, &path).unwrap();
        assert_eq!(result.title, Some("My Page".to_string()));
        assert_eq!(result.description, Some("A test page".to_string()));
    }

    #[test]
    fn test_registry_typescript() {
        let registry = get_default_registry();
        let path = PathBuf::from("Component.tsx");
        assert!(registry.is_supported(&path));

        let content = r#"
            export default function MyComponent() {
                return <div>Hello</div>;
            }
        "#;
        let result = registry.parse(content, &path).unwrap();
        assert!(result.metadata.is_react_component);
    }

    #[test]
    fn test_registry_unsupported() {
        let registry = get_default_registry();
        let path = PathBuf::from("test.xyz");
        assert!(!registry.is_supported(&path));
    }

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
title: Test
author: John
---

# Heading
Content"#;
        let (fm, body) = parse_frontmatter(content);
        assert!(fm.is_some());
        assert!(fm.unwrap().contains("title: Test"));
        assert!(body.starts_with("# Heading"));
    }

    #[test]
    fn test_extract_frontmatter_field() {
        let fm = "title: My Title\ndescription: \"A description\"";
        assert_eq!(extract_frontmatter_field(fm, "title"), Some("My Title".to_string()));
        assert_eq!(extract_frontmatter_field(fm, "description"), Some("A description".to_string()));
    }
}
