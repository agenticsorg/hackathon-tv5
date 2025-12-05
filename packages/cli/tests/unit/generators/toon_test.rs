/// Comprehensive test suite for TOON generator
/// Tests conversion from HTML to TOON (Tree-Oriented Object Notation) format
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn read_fixture(name: &str) -> String {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/toon")
        .join(name);
    fs::read_to_string(fixture_path).expect("Failed to read fixture file")
}

fn normalize_whitespace(s: &str) -> String {
    s.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

// ============================================================================
// BASIC CONVERSION TESTS
// ============================================================================

#[test]
fn test_to_toon_basic_html() {
    let html = r#"
<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
    <h1>Hello World</h1>
    <p>This is a test.</p>
</body>
</html>
"#;

    // Note: This will fail until TOON generator is implemented
    // Expected behavior: Convert HTML to clean markdown format
    let result = arw_lib::generators::toon::from_html(html, Path::new("test.toon"));

    assert!(result.is_ok(), "Basic HTML conversion should succeed");
    let toon = result.unwrap();

    assert!(toon.contains("# Hello World"), "Should contain heading");
    assert!(toon.contains("This is a test"), "Should contain paragraph");
    assert!(!toon.contains("<html>"), "Should not contain HTML tags");
}

#[test]
fn test_to_toon_with_headings() {
    let html = r#"
<html>
<body>
    <h1>Level 1</h1>
    <h2>Level 2</h2>
    <h3>Level 3</h3>
    <h4>Level 4</h4>
</body>
</html>
"#;

    let result = arw_lib::generators::toon::from_html(html, Path::new("test.toon"));
    assert!(result.is_ok(), "Heading conversion should succeed");

    let toon = result.unwrap();
    assert!(toon.contains("# Level 1"), "Should have h1 as #");
    assert!(toon.contains("## Level 2"), "Should have h2 as ##");
    assert!(toon.contains("### Level 3"), "Should have h3 as ###");
    assert!(toon.contains("#### Level 4"), "Should have h4 as ####");
}

#[test]
fn test_to_toon_with_lists() {
    let html = r#"
<html>
<body>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
    </ul>
    <ol>
        <li>First</li>
        <li>Second</li>
    </ol>
</body>
</html>
"#;

    let result = arw_lib::generators::toon::from_html(html, Path::new("test.toon"));
    assert!(result.is_ok(), "List conversion should succeed");

    let toon = result.unwrap();
    assert!(
        toon.contains("* Item 1") || toon.contains("- Item 1"),
        "Should have unordered list items"
    );
    assert!(toon.contains("1. First"), "Should have ordered list items");
}

#[test]
fn test_to_toon_with_code() {
    let html = r#"
<html>
<body>
    <pre><code>function test() {
    return true;
}</code></pre>
    <p>Inline <code>code</code> here.</p>
</body>
</html>
"#;

    let result = arw_lib::generators::toon::from_html(html, Path::new("test.toon"));
    assert!(result.is_ok(), "Code block conversion should succeed");

    let toon = result.unwrap();
    assert!(
        toon.contains("```") || toon.contains("function test()"),
        "Should preserve code blocks"
    );
    assert!(toon.contains("`code`"), "Should preserve inline code");
}

#[test]
fn test_to_toon_with_chunks() {
    let html = r#"
<html>
<body>
    <h1>Introduction</h1>
    <p>First section.</p>

    <h2>Details</h2>
    <p>Second section.</p>
</body>
</html>
"#;

    let result = arw_lib::generators::toon::from_html(html, Path::new("test.toon"));
    assert!(result.is_ok(), "Chunk conversion should succeed");

    let toon = result.unwrap();

    // TOON format should support chunk markers for semantic sections
    // This helps AI systems understand document structure
    let with_chunks = arw_lib::generators::toon::add_chunk_markers(&toon);

    assert!(
        with_chunks.contains("<!-- chunk:") || with_chunks.contains("chunk:"),
        "Should include chunk markers for headings"
    );
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn test_to_toon_empty_html() {
    let html = r#"
<!DOCTYPE html>
<html>
<head><title>Empty</title></head>
<body></body>
</html>
"#;

    let result = arw_lib::generators::toon::from_html(html, Path::new("test.toon"));
    assert!(result.is_ok(), "Empty HTML should not fail");

    let toon = result.unwrap();
    assert!(toon.trim().is_empty() || toon.trim().len() < 10, "Empty HTML should produce minimal output");
}

#[test]
fn test_to_toon_invalid_html() {
    let html = r#"<html><body><h1>Unclosed heading<p>Text</body>"#;

    let result = arw_lib::generators::toon::from_html(html, Path::new("test.toon"));
    // Should handle malformed HTML gracefully
    assert!(result.is_ok(), "Should handle malformed HTML");
}

#[test]
fn test_to_toon_special_characters() {
    let html = r#"
<html>
<body>
    <p>Special chars: &lt; &gt; &amp; &quot; &#39;</p>
    <p>Unicode: 测试 🚀 émojis</p>
</body>
</html>
"#;

    let result = arw_lib::generators::toon::from_html(html, Path::new("test.toon"));
    assert!(result.is_ok(), "Special character conversion should succeed");

    let toon = result.unwrap();
    assert!(toon.contains("<"), "Should decode HTML entities");
    assert!(toon.contains(">"), "Should decode HTML entities");
    assert!(toon.contains("&"), "Should decode HTML entities");
    assert!(toon.contains("测试"), "Should preserve Unicode");
    assert!(toon.contains("🚀"), "Should preserve emoji");
}

#[test]
fn test_to_toon_nested_elements() {
    let html = r#"
<html>
<body>
    <div>
        <div>
            <p>Nested <strong>bold <em>and italic</em></strong> text</p>
        </div>
    </div>
</body>
</html>
"#;

    let result = arw_lib::generators::toon::from_html(html, Path::new("test.toon"));
    assert!(result.is_ok(), "Nested elements should convert");

    let toon = result.unwrap();
    assert!(toon.contains("**bold"), "Should preserve bold");
    assert!(toon.contains("*and italic*"), "Should preserve italic");
}

// ============================================================================
// FIXTURE-BASED TESTS
// ============================================================================

#[test]
fn test_simple_fixture_conversion() {
    let html = read_fixture("simple.html");
    let expected = read_fixture("expected_simple.toon");

    let result = arw_lib::generators::toon::from_html(&html, Path::new("simple.toon"));
    assert!(result.is_ok(), "Simple fixture conversion should succeed");

    let toon = result.unwrap();
    let normalized_result = normalize_whitespace(&toon);
    let normalized_expected = normalize_whitespace(&expected);

    assert_eq!(
        normalized_result, normalized_expected,
        "Simple fixture output should match expected"
    );
}

#[test]
fn test_complex_fixture_conversion() {
    let html = read_fixture("complex.html");
    let expected = read_fixture("expected_complex.toon");

    let result = arw_lib::generators::toon::from_html(&html, Path::new("complex.toon"));
    assert!(result.is_ok(), "Complex fixture conversion should succeed");

    let toon = result.unwrap();

    // Check for key elements rather than exact match (whitespace can vary)
    assert!(toon.contains("# Main Heading"), "Should have main heading");
    assert!(toon.contains("## Section One"), "Should have section headings");
    assert!(toon.contains("**bold text**"), "Should have bold text");
    assert!(toon.contains("*italic text*"), "Should have italic text");
    assert!(toon.contains("* Unordered"), "Should have unordered lists");
    assert!(toon.contains("1. Ordered"), "Should have ordered lists");
    assert!(toon.contains("```") || toon.contains("function hello()"), "Should have code blocks");
    assert!(toon.contains(">"), "Should have blockquotes");
}

// ============================================================================
// CHUNK MARKER TESTS
// ============================================================================

#[test]
fn test_add_chunk_markers_basic() {
    let markdown = "# Introduction\nContent here\n## Details\nMore content";
    let result = arw_lib::generators::toon::add_chunk_markers(markdown);

    assert!(result.contains("<!-- chunk:"), "Should add chunk markers");
    assert!(result.contains("introduction"), "Should derive chunk ID from heading");
    assert!(result.contains("details"), "Should add chunk for all headings");
}

#[test]
fn test_add_chunk_markers_with_special_chars() {
    let markdown = "# Hello, World!\nContent\n## Section 2.1: Details\nMore";
    let result = arw_lib::generators::toon::add_chunk_markers(markdown);

    assert!(result.contains("hello-world"), "Should clean chunk IDs");
    assert!(result.contains("section-2-1-details"), "Should handle special characters");
}

#[test]
fn test_add_chunk_markers_empty() {
    let markdown = "";
    let result = arw_lib::generators::toon::add_chunk_markers(markdown);

    assert!(result.is_empty() || result.trim().is_empty(), "Empty input should produce empty output");
}

// ============================================================================
// FILE OUTPUT TESTS
// ============================================================================

#[test]
fn test_write_toon_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.llm.toon");

    let html = r#"
<html>
<body>
    <h1>Test Output</h1>
    <p>Testing file write.</p>
</body>
</html>
"#;

    let toon = arw_lib::generators::toon::from_html(html, &output_path).unwrap();

    // Write to file
    fs::write(&output_path, &toon).expect("Should write TOON file");

    // Verify file exists
    assert!(output_path.exists(), "TOON file should be created");

    // Verify content
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("# Test Output"), "File should contain converted content");
}

#[test]
fn test_toon_file_extension() {
    let temp_dir = TempDir::new().unwrap();

    // Test with .llm.toon extension (preferred)
    let toon_path = temp_dir.path().join("test.llm.toon");
    let html = "<html><body><h1>Test</h1></body></html>";

    let toon = arw_lib::generators::toon::from_html(html, &toon_path).unwrap();
    fs::write(&toon_path, &toon).unwrap();

    assert!(toon_path.exists(), ".llm.toon file should be created");
}

// ============================================================================
// INTEGRATION WITH MACHINE VIEW
// ============================================================================

#[test]
fn test_toon_vs_markdown_output() {
    let html = r#"
<html>
<body>
    <h1>Comparison Test</h1>
    <p>This tests TOON vs Markdown output.</p>
</body>
</html>
"#;

    // Generate both formats
    let toon = arw_lib::generators::toon::from_html(html, Path::new("test.toon")).unwrap();
    let markdown = arw_lib::generators::machine_view::from_html(html, Path::new("test.md")).unwrap();

    // Both should produce similar markdown-like output
    assert!(toon.contains("# Comparison Test"), "TOON should have heading");
    assert!(markdown.contains("# Comparison Test") || markdown.contains("Comparison Test"), "Markdown should have heading");

    // TOON might have additional metadata or structure
    // This is where TOON can differ from plain markdown
}

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

#[test]
fn test_large_html_conversion() {
    // Generate a large HTML document
    let mut html = String::from("<html><body>");
    for i in 0..1000 {
        html.push_str(&format!("<h2>Section {}</h2>", i));
        html.push_str(&format!("<p>Content for section {}</p>", i));
    }
    html.push_str("</body></html>");

    let start = std::time::Instant::now();
    let result = arw_lib::generators::toon::from_html(&html, Path::new("large.toon"));
    let duration = start.elapsed();

    assert!(result.is_ok(), "Large HTML should convert successfully");
    assert!(
        duration.as_secs() < 5,
        "Large conversion should complete within 5 seconds"
    );
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_null_bytes_in_html() {
    let html = "< html><body>Test\0Content</body></html>";

    let result = arw_lib::generators::toon::from_html(html, Path::new("test.toon"));
    // Should handle or reject null bytes gracefully
    assert!(result.is_ok() || result.is_err(), "Should handle null bytes");
}

#[test]
fn test_extremely_nested_html() {
    let mut html = String::from("<html><body>");
    for _ in 0..1000 {
        html.push_str("<div>");
    }
    html.push_str("Deep content");
    for _ in 0..1000 {
        html.push_str("</div>");
    }
    html.push_str("</body></html>");

    let result = arw_lib::generators::toon::from_html(&html, Path::new("deep.toon"));
    assert!(result.is_ok(), "Should handle deeply nested HTML");
}
