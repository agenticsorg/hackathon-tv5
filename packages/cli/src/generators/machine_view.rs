use anyhow::Result;
use std::path::Path;
use scraper::{Html, Selector};
use chrono::Utc;

/// Generate a machine view from HTML content (Markdown format)
pub fn from_html(html: &str, _output_path: &Path) -> Result<String> {
    // Simple conversion using html2md
    Ok(html2md::parse_html(html))
}

/// Generate TOON format MachineView from HTML content
pub fn to_toon(html: &str, source_path: &Path) -> Result<String> {
    let document = Html::parse_document(html);

    // Extract title
    let title = extract_html_title(&document)
        .unwrap_or_else(|| "Untitled".to_string());

    // Build TOON structure
    let mut toon = String::new();
    toon.push_str("MachineView {\n");
    toon.push_str("  version: \"1.0\"\n");
    toon.push_str(&format!("  title: \"{}\"\n", escape_string(&title)));
    toon.push_str("  content: [\n");

    // Extract content blocks
    let content_blocks = extract_content_blocks(&document);
    for (i, block) in content_blocks.iter().enumerate() {
        toon.push_str("    ");
        toon.push_str(block);
        if i < content_blocks.len() - 1 {
            toon.push('\n');
        }
    }
    toon.push_str("\n  ]\n");

    // Extract chunks
    toon.push_str("  chunks: [\n");
    let chunks = extract_chunks(&document);
    for (i, chunk) in chunks.iter().enumerate() {
        toon.push_str(chunk);
        if i < chunks.len() - 1 {
            toon.push('\n');
        }
    }
    toon.push_str("\n  ]\n");

    // Add metadata
    toon.push_str("  metadata: {\n");
    toon.push_str(&format!("    source: \"{}\"\n", escape_string(&source_path.display().to_string())));
    toon.push_str(&format!("    generated_at: \"{}\"\n", Utc::now().to_rfc3339()));
    toon.push_str("    format: \"arw-machine-view\"\n");
    toon.push_str("  }\n");
    toon.push_str("}\n");

    Ok(toon)
}

/// Extract title from HTML document
fn extract_html_title(document: &Html) -> Option<String> {
    let selector = Selector::parse("title").ok()?;
    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
}

/// Extract content blocks from HTML
fn extract_content_blocks(document: &Html) -> Vec<String> {
    let mut blocks = Vec::new();

    // Extract headings
    for level in 1..=6 {
        if let Ok(selector) = Selector::parse(&format!("h{}", level)) {
            for element in document.select(&selector) {
                let text = element.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    blocks.push(format!(
                        "Heading {{ level: {}, text: \"{}\" }}",
                        level,
                        escape_string(&text)
                    ));
                }
            }
        }
    }

    // Extract paragraphs
    if let Ok(selector) = Selector::parse("p") {
        for element in document.select(&selector) {
            let text = element.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                blocks.push(format!(
                    "Paragraph {{ content: [Text(\"{}\")] }}",
                    escape_string(&text)
                ));
            }
        }
    }

    // Extract lists
    if let Ok(ul_selector) = Selector::parse("ul") {
        for ul_element in document.select(&ul_selector) {
            let mut items = Vec::new();
            if let Ok(li_selector) = Selector::parse("li") {
                for li in ul_element.select(&li_selector) {
                    let text = li.text().collect::<String>().trim().to_string();
                    if !text.is_empty() {
                        items.push(format!("\"{}\"", escape_string(&text)));
                    }
                }
            }
            if !items.is_empty() {
                blocks.push(format!("List {{ items: [{}] }}", items.join(", ")));
            }
        }
    }

    // Extract code blocks
    if let Ok(selector) = Selector::parse("pre code, pre, code") {
        for element in document.select(&selector) {
            let text = element.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                blocks.push(format!(
                    "Code {{ content: \"{}\" }}",
                    escape_string(&text)
                ));
            }
        }
    }

    blocks
}

/// Extract chunks from HTML document
fn extract_chunks(document: &Html) -> Vec<String> {
    let mut chunks = Vec::new();

    // Create chunks based on h1 and h2 headings
    for level in 1..=2 {
        if let Ok(selector) = Selector::parse(&format!("h{}", level)) {
            for element in document.select(&selector) {
                let title = element.text().collect::<String>().trim().to_string();
                if title.is_empty() {
                    continue;
                }

                let chunk_id = generate_chunk_id(&title);

                let mut chunk = String::new();
                chunk.push_str("    Chunk {\n");
                chunk.push_str(&format!("      id: \"{}\"\n", chunk_id));
                chunk.push_str(&format!("      title: \"{}\"\n", escape_string(&title)));
                chunk.push_str("      blocks: [\n");
                chunk.push_str(&format!("        Heading {{ level: {}, text: \"{}\" }}\n", level, escape_string(&title)));
                chunk.push_str("      ]\n");
                chunk.push_str("    }");

                chunks.push(chunk);
            }
        }
    }

    chunks
}

/// Generate a chunk ID from title text
fn generate_chunk_id(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

/// Escape special characters in strings for TOON format
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Add chunk markers to markdown content
pub fn add_chunk_markers(markdown: &str) -> String {
    // TODO: Implement intelligent chunk detection
    // For now, add chunk markers at heading boundaries
    let mut output = String::new();

    for line in markdown.lines() {
        if line.starts_with('#') {
            // Extract heading text to generate chunk ID
            let heading_text = line.trim_start_matches('#').trim();
            let chunk_id = heading_text
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("-");

            if !chunk_id.is_empty() {
                output.push_str(&format!("\n<!-- chunk: {} -->\n", chunk_id));
            }
        }

        output.push_str(line);
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_add_chunk_markers() {
        let markdown = "# Introduction\nContent here\n## Details\nMore content";
        let result = add_chunk_markers(markdown);

        assert!(result.contains("<!-- chunk: introduction -->"));
        assert!(result.contains("<!-- chunk: details -->"));
    }

    #[test]
    fn test_to_toon_basic_structure() {
        let html = r#"<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
    <h1>Main Heading</h1>
    <p>This is a paragraph.</p>
</body>
</html>"#;
        let path = PathBuf::from("/test/page.html");
        let result = to_toon(html, &path).unwrap();

        assert!(result.contains("MachineView {"));
        assert!(result.contains("version: \"1.0\""));
        assert!(result.contains("title: \"Test Page\""));
        assert!(result.contains("content: ["));
        assert!(result.contains("chunks: ["));
        assert!(result.contains("metadata: {"));
        assert!(result.contains("format: \"arw-machine-view\""));
    }

    #[test]
    fn test_to_toon_extracts_headings() {
        let html = r#"<h1>Level 1</h1><h2>Level 2</h2><h3>Level 3</h3>"#;
        let path = PathBuf::from("/test.html");
        let result = to_toon(html, &path).unwrap();

        assert!(result.contains("Heading { level: 1, text: \"Level 1\" }"));
        assert!(result.contains("Heading { level: 2, text: \"Level 2\" }"));
        assert!(result.contains("Heading { level: 3, text: \"Level 3\" }"));
    }

    #[test]
    fn test_to_toon_extracts_paragraphs() {
        let html = r#"<p>First paragraph</p><p>Second paragraph</p>"#;
        let path = PathBuf::from("/test.html");
        let result = to_toon(html, &path).unwrap();

        assert!(result.contains("Paragraph { content: [Text(\"First paragraph\")] }"));
        assert!(result.contains("Paragraph { content: [Text(\"Second paragraph\")] }"));
    }

    #[test]
    fn test_to_toon_extracts_lists() {
        let html = r#"<ul><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul>"#;
        let path = PathBuf::from("/test.html");
        let result = to_toon(html, &path).unwrap();

        assert!(result.contains("List { items: ["));
        assert!(result.contains("\"Item 1\""));
        assert!(result.contains("\"Item 2\""));
        assert!(result.contains("\"Item 3\""));
    }

    #[test]
    fn test_to_toon_extracts_code_blocks() {
        let html = r#"<pre><code>console.log('hello');</code></pre>"#;
        let path = PathBuf::from("/test.html");
        let result = to_toon(html, &path).unwrap();

        assert!(result.contains("Code { content: \"console.log('hello');\" }"));
    }

    #[test]
    fn test_to_toon_creates_chunks() {
        let html = r#"<h1>Introduction</h1><h2>Details Section</h2>"#;
        let path = PathBuf::from("/test.html");
        let result = to_toon(html, &path).unwrap();

        assert!(result.contains("Chunk {"));
        assert!(result.contains("id: \"introduction\""));
        assert!(result.contains("title: \"Introduction\""));
        assert!(result.contains("id: \"details-section\""));
    }

    #[test]
    fn test_escape_string_special_chars() {
        assert_eq!(escape_string("test\"quote"), "test\\\"quote");
        assert_eq!(escape_string("test\\backslash"), "test\\\\backslash");
        assert_eq!(escape_string("test\nnewline"), "test\\nnewline");
        assert_eq!(escape_string("test\ttab"), "test\\ttab");
    }

    #[test]
    fn test_generate_chunk_id() {
        assert_eq!(generate_chunk_id("Introduction"), "introduction");
        assert_eq!(generate_chunk_id("Getting Started"), "getting-started");
        assert_eq!(generate_chunk_id("API Reference!"), "api-reference");
        assert_eq!(generate_chunk_id("Test-123"), "test-123");
    }

    #[test]
    fn test_to_toon_with_no_title() {
        let html = r#"<body><p>Content without title</p></body>"#;
        let path = PathBuf::from("/test.html");
        let result = to_toon(html, &path).unwrap();

        assert!(result.contains("title: \"Untitled\""));
    }

    #[test]
    fn test_to_toon_includes_metadata() {
        let html = r#"<p>Test</p>"#;
        let path = PathBuf::from("/absolute/path/to/file.html");
        let result = to_toon(html, &path).unwrap();

        assert!(result.contains("metadata: {"));
        assert!(result.contains("source: \"/absolute/path/to/file.html\""));
        assert!(result.contains("generated_at:"));
        assert!(result.contains("format: \"arw-machine-view\""));
    }

    #[test]
    fn test_to_toon_handles_empty_html() {
        let html = "";
        let path = PathBuf::from("/test.html");
        let result = to_toon(html, &path).unwrap();

        assert!(result.contains("MachineView {"));
        assert!(result.contains("content: ["));
        assert!(result.contains("]"));
    }

    #[test]
    fn test_to_toon_escapes_quotes_in_content() {
        let html = r#"<p>She said "hello" to me</p>"#;
        let path = PathBuf::from("/test.html");
        let result = to_toon(html, &path).unwrap();

        assert!(result.contains("She said \\\"hello\\\" to me"));
    }
}
