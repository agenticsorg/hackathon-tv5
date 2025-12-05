//! TypeScript/JavaScript/JSX/TSX parser for extracting content from React components
//! and JavaScript modules.

use anyhow::Result;
use regex::Regex;
use std::path::Path;

use super::{ContentParser, ExtractedContent, ContentMetadata, Section, CodeBlock};

/// Parser for TypeScript, JavaScript, JSX, and TSX files
pub struct TypeScriptParser;

impl ContentParser for TypeScriptParser {
    fn name(&self) -> &'static str {
        "typescript"
    }

    fn extensions(&self) -> &[&'static str] {
        &["ts", "tsx", "js", "jsx", "mjs", "cjs"]
    }

    fn parse(&self, content: &str, source_path: &Path) -> Result<ExtractedContent> {
        let mut extracted = ExtractedContent::new();

        let ext = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        extracted.metadata.extension = ext.to_string();
        extracted.metadata.parser = self.name().to_string();

        // Extract component name from file or content
        extracted.metadata.component_name = extract_component_name(content, source_path);
        extracted.metadata.is_react_component = is_react_component(content);
        extracted.metadata.exports = extract_exports(content);

        // Set title from component name or filename
        extracted.title = extracted.metadata.component_name.clone()
            .or_else(|| source_path.file_stem().and_then(|s| s.to_str()).map(String::from));

        // Extract JSDoc comments and descriptions
        if let Some(desc) = extract_jsdoc_description(content) {
            extracted.description = Some(desc);
        }

        // Build structured markdown content from JSX
        let markdown = extract_structured_markdown(content);
        extracted.content = markdown;

        // Extract sections from comments
        extracted.sections = extract_sections_from_comments(content);

        // Add the source code as a code block for reference
        extracted.code_blocks.push(CodeBlock {
            language: Some(ext.to_string()),
            content: content.to_string(),
        });

        Ok(extracted)
    }
}

/// Simple HTML tag stripper
fn strip_html_tags(text: &str) -> String {
    if let Ok(re) = Regex::new(r"<[^>]+>") {
        re.replace_all(text, "").to_string()
    } else {
        text.to_string()
    }
}

/// Extract structured markdown from JSX content
/// This parses React/JSX code (not rendered HTML) to extract content with structure
fn extract_structured_markdown(content: &str) -> String {
    let mut markdown = String::new();
    let mut seen_text = std::collections::HashSet::new();

    // Navigation patterns to skip
    let nav_patterns = ["Home", "About", "Login", "Sign up", "Menu", "Toggle menu", "Header", "Footer"];

    // Extract h1/h2/h3 JSX elements: <h1 className="...">Text</h1>
    if let Ok(h_re) = Regex::new(r"<h([1-3])[^>]*>([^<]+)</h\1>") {
        for caps in h_re.captures_iter(content) {
            let level = &caps[1];
            let text = caps[2].trim();

            if nav_patterns.iter().any(|p| text.contains(p)) || text.len() <= 3 {
                continue;
            }

            let heading = match level {
                "1" => format!("# {}\n\n", text),
                "2" => format!("## {}\n\n", text),
                "3" => format!("### {}\n\n", text),
                _ => String::new(),
            };

            if !heading.is_empty() && seen_text.insert(text.to_string()) {
                markdown.push_str(&heading);
            }
        }
    }

    // Extract <a href="url">Text</a> links from JSX
    if let Ok(link_re) = Regex::new(r#"<a\s+href=["']([^"']+)["'][^>]*>(?:\s*<[^>]+>)*\s*([^<]+?)(?:</[^>]+>)*\s*</a>"#) {
        let mut links = Vec::new();

        for caps in link_re.captures_iter(content) {
            let url = caps[1].trim();
            let text = caps[2].trim();

            if nav_patterns.iter().any(|p| text.contains(p)) || text.len() <= 3 {
                continue;
            }

            if url.starts_with('#') || url.is_empty() || !seen_text.insert(text.to_string()) {
                continue;
            }

            // Ensure URL is absolute
            let full_url = if url.starts_with("http") {
                url.to_string()
            } else if url.starts_with('/') {
                format!("https://arw.dev{}", url)
            } else {
                url.to_string()
            };

            links.push(format!("[{}]({})", text, full_url));
        }

        if !links.is_empty() {
            markdown.push_str("## Links\n\n");
            for link in links {
                markdown.push_str(&format!("- {}\n", link));
            }
            markdown.push_str("\n");
        }
    }

    // Extract <p> paragraphs
    if let Ok(p_re) = Regex::new(r"<p[^>]*>([^<]+)</p>") {
        for caps in p_re.captures_iter(content) {
            let text = caps[1].trim();

            if text.len() > 20 && !nav_patterns.iter().any(|p| text.contains(p)) && seen_text.insert(text.to_string()) {
                markdown.push_str(&format!("{}\n\n", text));
            }
        }
    }

    // If we got nothing, fall back to simple text extraction
    if markdown.is_empty() {
        return extract_jsx_text(content);
    }

    markdown
}

/// Extract component name from React component
fn extract_component_name(content: &str, path: &Path) -> Option<String> {
    // Try to find: export default function ComponentName
    let default_fn = Regex::new(r"export\s+default\s+function\s+(\w+)").ok()?;
    if let Some(caps) = default_fn.captures(content) {
        return Some(caps[1].to_string());
    }

    // Try to find: export default ComponentName
    let default_export = Regex::new(r"export\s+default\s+(\w+)").ok()?;
    if let Some(caps) = default_export.captures(content) {
        let name = &caps[1];
        // Skip if it's a common keyword
        if !["function", "class", "async", "const", "let", "var"].contains(&name) {
            return Some(name.to_string());
        }
    }

    // Try to find: const ComponentName = () =>
    let arrow_fn = Regex::new(r"(?:export\s+)?const\s+([A-Z]\w+)\s*=\s*(?:\([^)]*\)|[^=])\s*=>").ok()?;
    if let Some(caps) = arrow_fn.captures(content) {
        return Some(caps[1].to_string());
    }

    // Try to find: function ComponentName
    let fn_decl = Regex::new(r"(?:export\s+)?function\s+([A-Z]\w+)").ok()?;
    if let Some(caps) = fn_decl.captures(content) {
        return Some(caps[1].to_string());
    }

    // Fall back to filename (PascalCase)
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| {
            // Convert to PascalCase if it looks like a component file
            if s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                s.to_string()
            } else {
                // Convert kebab-case or snake_case to PascalCase
                s.split(|c| c == '-' || c == '_')
                    .map(|part| {
                        let mut chars = part.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    })
                    .collect()
            }
        })
}

/// Check if content is a React component
fn is_react_component(content: &str) -> bool {
    // Check for React imports
    let has_react_import = content.contains("from 'react'")
        || content.contains("from \"react\"")
        || content.contains("import React");

    // Check for JSX syntax
    let has_jsx = Regex::new(r"<[A-Z]\w*|<[a-z]+[^>]*>|</>")
        .map(|re| re.is_match(content))
        .unwrap_or(false);

    // Check for hooks
    let has_hooks = content.contains("useState")
        || content.contains("useEffect")
        || content.contains("useContext")
        || content.contains("useRef")
        || content.contains("useMemo")
        || content.contains("useCallback");

    has_react_import || has_jsx || has_hooks
}

/// Extract exports from the file
fn extract_exports(content: &str) -> Vec<String> {
    let mut exports = Vec::new();

    // Named exports: export const/function/class Name
    if let Ok(re) = Regex::new(r"export\s+(?:const|let|var|function|class|type|interface)\s+(\w+)") {
        for caps in re.captures_iter(content) {
            exports.push(caps[1].to_string());
        }
    }

    // Export { name1, name2 }
    if let Ok(re) = Regex::new(r"export\s*\{([^}]+)\}") {
        for caps in re.captures_iter(content) {
            for name in caps[1].split(',') {
                let name = name.split(" as ").next().unwrap_or(name).trim();
                if !name.is_empty() {
                    exports.push(name.to_string());
                }
            }
        }
    }

    // Default export
    if content.contains("export default") {
        exports.push("default".to_string());
    }

    exports
}

/// Extract JSDoc description from the file
fn extract_jsdoc_description(content: &str) -> Option<String> {
    // Look for top-level JSDoc comment
    let re = Regex::new(r"(?s)/\*\*\s*\n([^*]|\*(?!/))*\*/").ok()?;

    if let Some(caps) = re.captures(content) {
        let comment = caps[0].to_string();
        // Extract the description part (before @tags)
        let desc_re = Regex::new(r"(?s)/\*\*\s*\n?\s*\*?\s*(.+?)(?:\n\s*\*\s*@|\s*\*/)").ok()?;
        if let Some(desc_caps) = desc_re.captures(&comment) {
            let desc = desc_caps[1]
                .lines()
                .map(|l| l.trim().trim_start_matches('*').trim())
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            if !desc.is_empty() {
                return Some(desc);
            }
        }
    }

    None
}

/// Extract JSX text content (visible text in the component)
/// Excludes common navigation/header/footer patterns
fn extract_jsx_text(content: &str) -> String {
    let mut text_parts = Vec::new();

    // Common navigation/header/footer keywords to skip
    let skip_patterns = [
        "Home", "About", "Contact", "Login", "Sign up", "Sign in",
        "Menu", "Toggle menu", "Navigation", "Footer", "Header",
        "© ", "All rights reserved", "Privacy", "Terms", "Cookie"
    ];

    // Extract text from JSX elements
    // Match text between > and < that isn't a tag
    if let Ok(re) = Regex::new(r">([^<>{]+)<") {
        for caps in re.captures_iter(content) {
            let text = caps[1].trim();

            // Skip if it's navigation/header/footer content
            let is_nav = skip_patterns.iter().any(|pattern| text.contains(pattern));

            // Skip if it's just whitespace, looks like code, or is too short
            if !text.is_empty()
                && !text.starts_with('{')
                && !text.contains("className")
                && !is_nav
                && text.len() > 5  // Increase minimum length to skip nav items
            {
                text_parts.push(text.to_string());
            }
        }
    }

    // Extract string literals that look like content (not navigation)
    if let Ok(re) = Regex::new(r#"(?:title|label|text|content|heading|description|placeholder|alt)\s*[:=]\s*["'`]([^"'`]+)["'`]"#) {
        for caps in re.captures_iter(content) {
            let text = caps[1].trim();
            let is_nav = skip_patterns.iter().any(|pattern| text.contains(pattern));
            if !is_nav && text.len() > 5 {
                text_parts.push(text.to_string());
            }
        }
    }

    // Deduplicate and join
    let mut seen = std::collections::HashSet::new();
    text_parts
        .into_iter()
        .filter(|s| seen.insert(s.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extract comments from the code
fn extract_comments(content: &str) -> Vec<String> {
    let mut comments = Vec::new();

    // Single-line comments
    if let Ok(re) = Regex::new(r"//\s*(.+)") {
        for caps in re.captures_iter(content) {
            let comment = caps[1].trim();
            // Skip common noise
            if !comment.starts_with("eslint")
                && !comment.starts_with("@ts-")
                && !comment.starts_with("prettier")
                && !comment.is_empty()
                && comment.len() > 3
            {
                comments.push(comment.to_string());
            }
        }
    }

    // Multi-line comments (non-JSDoc)
    if let Ok(re) = Regex::new(r"(?s)/\*(?!\*)(.+?)\*/") {
        for caps in re.captures_iter(content) {
            let comment = caps[1]
                .lines()
                .map(|l| l.trim().trim_start_matches('*').trim())
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            if !comment.is_empty() && comment.len() > 10 {
                comments.push(comment);
            }
        }
    }

    comments
}

/// Extract sections from JSDoc and comments
fn extract_sections_from_comments(content: &str) -> Vec<Section> {
    let mut sections = Vec::new();

    // Look for @section or @heading tags in comments
    if let Ok(re) = Regex::new(r"@(?:section|heading)\s+(.+)") {
        for caps in re.captures_iter(content) {
            sections.push(Section {
                level: 2,
                title: caps[1].trim().to_string(),
                content: String::new(),
            });
        }
    }

    sections
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_component_name_default_function() {
        let content = "export default function MyComponent() { return <div />; }";
        let path = PathBuf::from("MyComponent.tsx");
        assert_eq!(extract_component_name(content, &path), Some("MyComponent".to_string()));
    }

    #[test]
    fn test_extract_component_name_arrow_function() {
        let content = "const MyComponent = () => { return <div />; }";
        let path = PathBuf::from("MyComponent.tsx");
        assert_eq!(extract_component_name(content, &path), Some("MyComponent".to_string()));
    }

    #[test]
    fn test_is_react_component() {
        let content = r#"
            import React from 'react';
            export default function App() {
                return <div>Hello</div>;
            }
        "#;
        assert!(is_react_component(content));
    }

    #[test]
    fn test_extract_exports() {
        let content = r#"
            export const foo = 1;
            export function bar() {}
            export class Baz {}
            export default App;
        "#;
        let exports = extract_exports(content);
        assert!(exports.contains(&"foo".to_string()));
        assert!(exports.contains(&"bar".to_string()));
        assert!(exports.contains(&"Baz".to_string()));
        assert!(exports.contains(&"default".to_string()));
    }

    #[test]
    fn test_extract_jsx_text() {
        let content = r#"
            return (
                <div>
                    <h1>Welcome to My App</h1>
                    <p>This is some content</p>
                </div>
            );
        "#;
        let text = extract_jsx_text(content);
        assert!(text.contains("Welcome to My App"));
        assert!(text.contains("This is some content"));
    }

    #[test]
    fn test_parser_parse() {
        let parser = TypeScriptParser;
        let content = r#"
            /**
             * A sample React component
             */
            export default function SampleComponent() {
                return <div>Hello World</div>;
            }
        "#;
        let path = PathBuf::from("SampleComponent.tsx");
        let result = parser.parse(content, &path).unwrap();

        assert_eq!(result.title, Some("SampleComponent".to_string()));
        assert!(result.metadata.is_react_component);
    }
}
