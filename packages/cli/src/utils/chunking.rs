use scraper::{Html, Selector};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Chunk {
    pub id: String,
    pub heading: Option<String>,
    pub content: String,
    pub level: usize,
}

/// Extract chunks from HTML content based on headings
#[allow(dead_code)]
pub fn extract_chunks(html: &str) -> Vec<Chunk> {
    let document = Html::parse_document(html);
    let mut chunks = Vec::new();

    // Selectors for different heading levels
    let selectors: Vec<_> = (1..=6)
        .map(|level| Selector::parse(&format!("h{}", level)).unwrap())
        .collect();

    // Extract chunks based on heading hierarchy
    for (level, selector) in selectors.iter().enumerate() {
        for element in document.select(selector) {
            let heading = element.text().collect::<String>().trim().to_string();
            let id = slugify(&heading);

            // Find content between this heading and the next one
            let content = extract_section_content(&document, &id, level + 1);

            chunks.push(Chunk {
                id,
                heading: Some(heading),
                content,
                level: level + 1,
            });
        }
    }

    // If no headings found, create a single chunk with all content
    if chunks.is_empty() {
        chunks.push(Chunk {
            id: "content".to_string(),
            heading: None,
            content: document.root_element().text().collect::<String>(),
            level: 0,
        });
    }

    chunks
}

/// Convert heading text to a slug (chunk ID)
#[allow(dead_code)]
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Extract content for a section (simplified version)
#[allow(dead_code)]
fn extract_section_content(document: &Html, _id: &str, _level: usize) -> String {
    // Simplified: just return body text
    // In a full implementation, this would extract content between headings
    document
        .select(&Selector::parse("body").unwrap())
        .next()
        .map(|body| body.text().collect::<String>())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Getting Started"), "getting-started");
        assert_eq!(slugify("API Reference"), "api-reference");
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
    }

    #[test]
    fn test_extract_chunks_basic() {
        let html = r#"
            <html>
                <body>
                    <h1>Title</h1>
                    <p>Content here</p>
                </body>
            </html>
        "#;

        let chunks = extract_chunks(html);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_extract_chunks_multiple_headings() {
        let html = r#"
            <html>
                <body>
                    <h1>First Title</h1>
                    <p>First content</p>
                    <h2>Second Title</h2>
                    <p>Second content</p>
                    <h3>Third Title</h3>
                    <p>Third content</p>
                </body>
            </html>
        "#;

        let chunks = extract_chunks(html);
        assert!(chunks.len() >= 3);

        // Check that headings were extracted
        let headings: Vec<_> = chunks.iter()
            .filter_map(|c| c.heading.as_ref())
            .collect();
        assert!(headings.iter().any(|h| h.contains("First Title")));
        assert!(headings.iter().any(|h| h.contains("Second Title")));
        assert!(headings.iter().any(|h| h.contains("Third Title")));
    }

    #[test]
    fn test_extract_chunks_no_headings() {
        let html = r#"
            <html>
                <body>
                    <p>Just some content without any headings</p>
                    <p>More content here</p>
                </body>
            </html>
        "#;

        let chunks = extract_chunks(html);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].heading, None);
        assert_eq!(chunks[0].id, "content");
        assert_eq!(chunks[0].level, 0);
    }

    #[test]
    fn test_extract_chunks_all_heading_levels() {
        let html = r#"
            <html>
                <body>
                    <h1>Level 1</h1>
                    <h2>Level 2</h2>
                    <h3>Level 3</h3>
                    <h4>Level 4</h4>
                    <h5>Level 5</h5>
                    <h6>Level 6</h6>
                </body>
            </html>
        "#;

        let chunks = extract_chunks(html);
        assert_eq!(chunks.len(), 6);

        // Verify levels
        let levels: Vec<_> = chunks.iter().map(|c| c.level).collect();
        assert!(levels.contains(&1));
        assert!(levels.contains(&2));
        assert!(levels.contains(&3));
        assert!(levels.contains(&4));
        assert!(levels.contains(&5));
        assert!(levels.contains(&6));
    }

    #[test]
    fn test_slugify_special_characters() {
        assert_eq!(slugify("Hello World!"), "hello-world");
        assert_eq!(slugify("Test@123"), "test-123");
        assert_eq!(slugify("Foo & Bar"), "foo-bar");
    }

    #[test]
    fn test_slugify_multiple_spaces() {
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
    }

    #[test]
    fn test_slugify_numbers() {
        assert_eq!(slugify("Section 1.2.3"), "section-1-2-3");
        assert_eq!(slugify("Version 2.0"), "version-2-0");
    }

    #[test]
    fn test_slugify_empty_string() {
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn test_slugify_all_special_chars() {
        assert_eq!(slugify("!!!"), "");
        assert_eq!(slugify("@@@"), "");
    }

    #[test]
    fn test_slugify_mixed_case() {
        assert_eq!(slugify("MixedCase"), "mixedcase");
        assert_eq!(slugify("CamelCaseString"), "camelcasestring");
    }

    #[test]
    fn test_chunk_structure() {
        let chunk = Chunk {
            id: "test-id".to_string(),
            heading: Some("Test Heading".to_string()),
            content: "Test content".to_string(),
            level: 1,
        };

        assert_eq!(chunk.id, "test-id");
        assert_eq!(chunk.heading, Some("Test Heading".to_string()));
        assert_eq!(chunk.content, "Test content");
        assert_eq!(chunk.level, 1);
    }

    #[test]
    fn test_chunk_clone() {
        let chunk = Chunk {
            id: "test".to_string(),
            heading: Some("Heading".to_string()),
            content: "Content".to_string(),
            level: 2,
        };

        let cloned = chunk.clone();
        assert_eq!(chunk.id, cloned.id);
        assert_eq!(chunk.heading, cloned.heading);
        assert_eq!(chunk.level, cloned.level);
    }

    #[test]
    fn test_chunk_debug_format() {
        let chunk = Chunk {
            id: "test".to_string(),
            heading: None,
            content: "content".to_string(),
            level: 0,
        };

        let debug_str = format!("{:?}", chunk);
        assert!(debug_str.contains("Chunk"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_extract_chunks_empty_html() {
        let html = r#"<html><body></body></html>"#;
        let chunks = extract_chunks(html);

        // Should return one chunk with empty or minimal content
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_extract_chunks_nested_headings() {
        let html = r#"
            <html>
                <body>
                    <h1>Main Title</h1>
                    <p>Main content</p>
                    <h2>Subsection</h2>
                    <p>Sub content</p>
                    <h2>Another Subsection</h2>
                    <p>More content</p>
                </body>
            </html>
        "#;

        let chunks = extract_chunks(html);
        assert!(chunks.len() >= 3);
    }

    #[test]
    fn test_extract_chunks_heading_with_whitespace() {
        let html = r#"
            <html>
                <body>
                    <h1>  Heading With Spaces  </h1>
                    <p>Content</p>
                </body>
            </html>
        "#;

        let chunks = extract_chunks(html);
        let first_chunk = &chunks[0];

        // Heading should be trimmed
        assert_eq!(first_chunk.heading, Some("Heading With Spaces".to_string()));
    }

    #[test]
    fn test_slugify_unicode() {
        // Unicode characters remain in lowercase
        assert_eq!(slugify("Café"), "café");
        assert_eq!(slugify("日本語"), "日本語");
    }

    #[test]
    fn test_slugify_leading_trailing_dashes() {
        assert_eq!(slugify("---test---"), "test");
        assert_eq!(slugify("--multiple--dashes--"), "multiple-dashes");
    }

    #[test]
    fn test_extract_section_content_basic() {
        let html = r#"
            <html>
                <body>
                    <p>Body content here</p>
                </body>
            </html>
        "#;

        let document = Html::parse_document(html);
        let content = extract_section_content(&document, "test", 1);

        assert!(content.contains("Body content"));
    }

    #[test]
    fn test_extract_chunks_id_generation() {
        let html = r#"
            <html>
                <body>
                    <h1>Getting Started</h1>
                    <h2>API Reference</h2>
                </body>
            </html>
        "#;

        let chunks = extract_chunks(html);

        // Check that IDs are properly slugified
        assert!(chunks.iter().any(|c| c.id == "getting-started"));
        assert!(chunks.iter().any(|c| c.id == "api-reference"));
    }
}
