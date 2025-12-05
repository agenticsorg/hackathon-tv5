// HTML parsing utilities

use scraper::{Html, Selector};

#[allow(dead_code)]
pub fn extract_text(html: &str) -> String {
    let document = Html::parse_document(html);
    document.root_element().text().collect()
}

#[allow(dead_code)]
pub fn extract_title(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("title").ok()?;
    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_from_simple_html() {
        let html = r#"<html><body><p>Hello World</p></body></html>"#;
        let text = extract_text(html);
        assert!(text.contains("Hello World"), "Should extract text content");
    }

    #[test]
    fn test_extract_text_from_complex_html() {
        let html = r#"
            <html>
                <head><title>Test Page</title></head>
                <body>
                    <h1>Main Heading</h1>
                    <p>First paragraph</p>
                    <div>
                        <span>Nested text</span>
                    </div>
                    <p>Second paragraph</p>
                </body>
            </html>
        "#;
        let text = extract_text(html);
        assert!(text.contains("Test Page"), "Should include title text");
        assert!(text.contains("Main Heading"), "Should include heading text");
        assert!(text.contains("First paragraph"), "Should include paragraph text");
        assert!(text.contains("Nested text"), "Should include nested text");
        assert!(text.contains("Second paragraph"), "Should include all paragraphs");
    }

    #[test]
    fn test_extract_text_strips_tags() {
        let html = r#"<p><strong>Bold</strong> and <em>italic</em> text</p>"#;
        let text = extract_text(html);
        assert!(text.contains("Bold"), "Should extract bold text");
        assert!(text.contains("italic"), "Should extract italic text");
        assert!(!text.contains("<strong>"), "Should not include HTML tags");
        assert!(!text.contains("</strong>"), "Should not include closing tags");
    }

    #[test]
    fn test_extract_text_from_empty_html() {
        let html = "";
        let text = extract_text(html);
        assert_eq!(text, "", "Should handle empty HTML");
    }

    #[test]
    fn test_extract_text_from_html_with_scripts() {
        let html = r#"
            <html>
                <body>
                    <p>Visible text</p>
                    <script>console.log('hidden');</script>
                </body>
            </html>
        "#;
        let text = extract_text(html);
        assert!(text.contains("Visible text"), "Should extract visible text");
        // Script content is still part of text nodes, but we're testing it parses
        assert!(!text.is_empty(), "Should extract text even with scripts");
    }

    #[test]
    fn test_extract_text_with_special_entities() {
        let html = r#"<p>Less than &lt; and greater than &gt; and ampersand &amp;</p>"#;
        let text = extract_text(html);
        assert!(
            text.contains("Less than < and greater than > and ampersand &")
            || text.contains("Less than &lt;"),
            "Should handle HTML entities"
        );
    }

    #[test]
    fn test_extract_text_with_unicode() {
        let html = r#"<p>Unicode: 你好 世界 🚀 ñ é</p>"#;
        let text = extract_text(html);
        assert!(text.contains("你好"), "Should handle Chinese characters");
        assert!(text.contains("🚀"), "Should handle emoji");
        assert!(text.contains("ñ"), "Should handle accented characters");
    }

    #[test]
    fn test_extract_title_from_valid_html() {
        let html = r#"<html><head><title>Test Page Title</title></head><body></body></html>"#;
        let title = extract_title(html);
        assert_eq!(
            title,
            Some("Test Page Title".to_string()),
            "Should extract title from HTML"
        );
    }

    #[test]
    fn test_extract_title_trims_whitespace() {
        let html = r#"<html><head><title>
            Test Title
        </title></head></html>"#;
        let title = extract_title(html);
        assert_eq!(
            title,
            Some("Test Title".to_string()),
            "Should trim whitespace from title"
        );
    }

    #[test]
    fn test_extract_title_from_html_without_title() {
        let html = r#"<html><head></head><body><h1>No Title Tag</h1></body></html>"#;
        let title = extract_title(html);
        assert_eq!(title, None, "Should return None when no title tag exists");
    }

    #[test]
    fn test_extract_title_from_empty_title_tag() {
        let html = r#"<html><head><title></title></head><body></body></html>"#;
        let title = extract_title(html);
        assert_eq!(
            title,
            Some("".to_string()),
            "Should return empty string for empty title tag"
        );
    }

    #[test]
    fn test_extract_title_from_malformed_html() {
        let html = r#"<title>Malformed Title</title><p>Body content"#;
        let title = extract_title(html);
        assert_eq!(
            title,
            Some("Malformed Title".to_string()),
            "Should extract title from malformed HTML"
        );
    }

    #[test]
    fn test_extract_title_from_multiple_title_tags() {
        let html = r#"
            <html>
                <head><title>First Title</title></head>
                <body><title>Second Title</title></body>
            </html>
        "#;
        let title = extract_title(html);
        assert_eq!(
            title,
            Some("First Title".to_string()),
            "Should extract only the first title tag"
        );
    }

    #[test]
    fn test_extract_title_with_nested_tags() {
        let html = r#"<html><head><title>Title with <span>nested</span> tags</title></head></html>"#;
        let title = extract_title(html);
        assert!(title.is_some(), "Should handle title with nested tags");
        // The title should include text from nested elements
        let title_text = title.unwrap();
        assert!(
            title_text.contains("Title with") && title_text.contains("nested"),
            "Should extract all text from title including nested tags"
        );
    }

    #[test]
    fn test_extract_title_with_entities() {
        let html = r#"<title>Test &amp; Title &lt;Special&gt;</title>"#;
        let title = extract_title(html);
        assert!(title.is_some(), "Should extract title with entities");
    }

    #[test]
    fn test_extract_title_with_unicode() {
        let html = r#"<title>测试标题 🚀 Test Title</title>"#;
        let title = extract_title(html);
        assert_eq!(
            title,
            Some("测试标题 🚀 Test Title".to_string()),
            "Should handle Unicode in title"
        );
    }

    #[test]
    fn test_extract_text_from_table() {
        let html = r#"
            <table>
                <tr><th>Header 1</th><th>Header 2</th></tr>
                <tr><td>Cell 1</td><td>Cell 2</td></tr>
            </table>
        "#;
        let text = extract_text(html);
        assert!(text.contains("Header 1"), "Should extract table header text");
        assert!(text.contains("Cell 1"), "Should extract table cell text");
    }

    #[test]
    fn test_extract_text_from_list() {
        let html = r#"
            <ul>
                <li>Item 1</li>
                <li>Item 2</li>
                <li>Item 3</li>
            </ul>
        "#;
        let text = extract_text(html);
        assert!(text.contains("Item 1"), "Should extract list item 1");
        assert!(text.contains("Item 2"), "Should extract list item 2");
        assert!(text.contains("Item 3"), "Should extract list item 3");
    }

    #[test]
    fn test_extract_text_with_line_breaks() {
        let html = r#"<p>Line 1<br>Line 2<br/>Line 3</p>"#;
        let text = extract_text(html);
        assert!(text.contains("Line 1"), "Should extract text before br");
        assert!(text.contains("Line 2"), "Should extract text between br tags");
        assert!(text.contains("Line 3"), "Should extract text after br");
    }

    #[test]
    fn test_extract_title_from_fragment() {
        let html = r#"<title>Fragment Title</title>"#;
        let title = extract_title(html);
        assert_eq!(
            title,
            Some("Fragment Title".to_string()),
            "Should extract title from HTML fragment"
        );
    }
}
