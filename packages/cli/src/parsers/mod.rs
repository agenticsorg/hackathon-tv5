//! Multi-language content parsers for ARW machine view generation
//!
//! This module provides an extensible parser system supporting multiple
//! file formats: HTML, Markdown, MDX, TypeScript, JSX, and more.

pub mod html;
pub mod markdown;
pub mod frontmatter;
pub mod typescript;
pub mod registry;

use anyhow::Result;
use std::path::Path;

/// Extracted content from a source file
#[derive(Debug, Clone)]
pub struct ExtractedContent {
    /// Document title (if available)
    pub title: Option<String>,
    /// Document description/summary
    pub description: Option<String>,
    /// Main text content as markdown
    pub content: String,
    /// Structured sections/headings
    pub sections: Vec<Section>,
    /// Code blocks found in the document
    pub code_blocks: Vec<CodeBlock>,
    /// Metadata extracted from the file
    pub metadata: ContentMetadata,
}

/// A section/heading in the document
#[derive(Debug, Clone)]
pub struct Section {
    pub level: u8,
    pub title: String,
    pub content: String,
}

/// A code block found in the document
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub content: String,
}

/// Metadata about the extracted content
#[derive(Debug, Clone, Default)]
pub struct ContentMetadata {
    /// Original file extension
    pub extension: String,
    /// Parser used to extract content
    pub parser: String,
    /// Exports found (for JS/TS files)
    pub exports: Vec<String>,
    /// Component name (for React components)
    pub component_name: Option<String>,
    /// Whether the file is a React component
    pub is_react_component: bool,
    /// Frontmatter data (for MDX/MD files)
    pub frontmatter: Option<String>,
}

/// Trait for language-specific content parsers
pub trait ContentParser: Send + Sync {
    /// Returns the name of this parser
    fn name(&self) -> &'static str;

    /// Returns the file extensions this parser supports
    fn extensions(&self) -> &[&'static str];

    /// Parse content from a string
    fn parse(&self, content: &str, source_path: &Path) -> Result<ExtractedContent>;

    /// Check if this parser can handle the given file
    fn can_parse(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            self.extensions().iter().any(|e| e.eq_ignore_ascii_case(ext))
        } else {
            false
        }
    }
}

impl ExtractedContent {
    /// Create a new empty ExtractedContent
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            content: String::new(),
            sections: Vec::new(),
            code_blocks: Vec::new(),
            metadata: ContentMetadata::default(),
        }
    }

    /// Convert to markdown format for machine view
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();

        // Add title
        if let Some(ref title) = self.title {
            output.push_str(&format!("# {}\n\n", title));
        }

        // Add description
        if let Some(ref desc) = self.description {
            output.push_str(&format!("{}\n\n", desc));
        }

        // Add main content
        if !self.content.is_empty() {
            output.push_str(&self.content);
            if !self.content.ends_with('\n') {
                output.push('\n');
            }
        }

        output
    }
}

impl Default for ExtractedContent {
    fn default() -> Self {
        Self::new()
    }
}
