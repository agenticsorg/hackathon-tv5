/// Comprehensive test suite for consistency validator
/// Tests cross-file consistency checking between manifest, HTML, and markdown
use arw_lib::validators::consistency::ConsistencyValidator;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_test_manifest(content_items: Vec<serde_json::Value>) -> String {
    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": content_items,
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });
    serde_yaml::to_string(&manifest).unwrap()
}

fn create_markdown_with_chunks(chunks: Vec<&str>) -> String {
    let mut content = String::from("# Test Page\n\n");
    for chunk_id in chunks {
        content.push_str(&format!("<!-- chunk: {} -->\n", chunk_id));
        content.push_str(&format!("Content for chunk {}\n\n", chunk_id));
    }
    content
}

fn create_html_with_chunks(chunks: Vec<&str>) -> String {
    let mut content = String::from("<html><body>\n");
    for chunk_id in chunks {
        content.push_str(&format!(
            "<section data-chunk-id=\"{}\">\n  <p>Content</p>\n</section>\n",
            chunk_id
        ));
    }
    content.push_str("</body></html>");
    content
}

// ============================================================================
// MACHINE VIEW FILE EXISTENCE TESTS
// ============================================================================

#[tokio::test]
async fn test_valid_machine_view_files_exist() {
    let temp_dir = TempDir::new().unwrap();

    // Create manifest
    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation"
    })];
    let manifest = create_test_manifest(content_items);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    // Create machine view file
    fs::write(
        temp_dir.path().join("page.llm.md"),
        "# Test Page\nContent here",
    )
    .unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors.is_empty(),
        "Should pass when machine view files exist. Errors: {:?}",
        errors
    );
}

#[tokio::test]
async fn test_missing_machine_view_file() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/missing.llm.md",
        "purpose": "documentation"
    })];
    let manifest = create_test_manifest(content_items);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors
            .iter()
            .any(|e| e.path.contains("machine_view") && e.message.contains("not found")),
        "Should detect missing machine view file"
    );
}

#[tokio::test]
async fn test_unreadable_machine_view_file() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation"
    })];
    let manifest = create_test_manifest(content_items);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    // Create file but make it a directory (unreadable as text)
    let md_path = temp_dir.path().join("page.llm.md");
    fs::create_dir(&md_path).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("not readable")),
        "Should detect unreadable machine view file"
    );
}

#[tokio::test]
async fn test_machine_view_with_leading_slash() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/subdir/page.llm.md",
        "purpose": "documentation"
    })];
    let manifest = create_test_manifest(content_items);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    // Create subdirectory and file
    fs::create_dir(temp_dir.path().join("subdir")).unwrap();
    fs::write(
        temp_dir.path().join("subdir/page.llm.md"),
        "# Test",
    )
    .unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors.is_empty(),
        "Should handle machine views with leading slash in subdirectories"
    );
}

// ============================================================================
// CHUNK CONSISTENCY TESTS
// ============================================================================

#[tokio::test]
async fn test_chunks_match_between_manifest_and_markdown() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation",
        "chunks": [
            {"id": "intro", "heading": "Introduction"},
            {"id": "main", "heading": "Main Content"}
        ]
    })];
    let manifest = create_test_manifest(content_items);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    let markdown = create_markdown_with_chunks(vec!["intro", "main"]);
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.path.contains("chunks")),
        "Should pass when chunks match. Errors: {:?}",
        errors
    );
}

#[tokio::test]
async fn test_chunk_in_manifest_not_in_markdown() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation",
        "chunks": [
            {"id": "intro", "heading": "Introduction"},
            {"id": "missing", "heading": "Missing Section"}
        ]
    })];
    let manifest = create_test_manifest(content_items);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    let markdown = create_markdown_with_chunks(vec!["intro"]);
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors.iter().any(|e| e.message.contains("missing") && e.message.contains("not found in")),
        "Should detect chunk declared in manifest but not in markdown"
    );
}

#[tokio::test]
async fn test_chunk_in_markdown_not_in_manifest() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation",
        "chunks": [
            {"id": "intro", "heading": "Introduction"}
        ]
    })];
    let manifest = create_test_manifest(content_items);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    let markdown = create_markdown_with_chunks(vec!["intro", "undeclared"]);
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors.iter().any(|e| e.message.contains("undeclared") && e.message.contains("not declared")),
        "Should detect chunk in markdown but not declared in manifest"
    );
}

#[tokio::test]
async fn test_no_chunks_declared_skips_validation() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation"
    })];
    let manifest = create_test_manifest(content_items);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    // Markdown has chunks but none declared in manifest
    let markdown = create_markdown_with_chunks(vec!["intro", "main"]);
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    // Should not fail - validation is skipped when no chunks declared
    assert!(
        !errors.iter().any(|e| e.path.contains("chunks")),
        "Should skip chunk validation when no chunks declared"
    );
}

#[tokio::test]
async fn test_chunk_markers_with_different_whitespace() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation",
        "chunks": [
            {"id": "intro", "heading": "Introduction"}
        ]
    })];
    let manifest = create_test_manifest(content_items);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    // Test various whitespace variations
    let markdown = r#"
# Test Page

<!--chunk:intro-->
Content here

<!-- chunk: intro -->
More content
"#;
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.path.contains("chunks")),
        "Should handle chunk markers with various whitespace"
    );
}

// ============================================================================
// HTML CHUNK CONSISTENCY TESTS
// ============================================================================

#[tokio::test]
async fn test_chunks_match_between_manifest_and_html() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation",
        "chunks": [
            {"id": "intro", "heading": "Introduction"},
            {"id": "main", "heading": "Main Content"}
        ]
    })];
    let manifest = create_test_manifest(content_items);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    // Create both markdown and HTML with matching chunks
    let markdown = create_markdown_with_chunks(vec!["intro", "main"]);
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let html = create_html_with_chunks(vec!["intro", "main"]);
    fs::write(temp_dir.path().join("page.html"), html).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.path.contains("chunks")),
        "Should pass when chunks match in HTML. Errors: {:?}",
        errors
    );
}

#[tokio::test]
async fn test_chunk_in_manifest_not_in_html() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation",
        "chunks": [
            {"id": "intro", "heading": "Introduction"},
            {"id": "missing", "heading": "Missing"}
        ]
    })];
    let manifest = create_test_manifest(content_items);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    let markdown = create_markdown_with_chunks(vec!["intro", "missing"]);
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let html = create_html_with_chunks(vec!["intro"]);
    fs::write(temp_dir.path().join("page.html"), html).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors.iter().any(|e| e.message.contains("missing") && e.message.contains("HTML")),
        "Should detect chunk missing in HTML"
    );
}

// ============================================================================
// ROBOTS.TXT CONSISTENCY TESTS
// ============================================================================

#[tokio::test]
async fn test_robots_txt_blocks_training_when_policy_disallows() {
    let temp_dir = TempDir::new().unwrap();

    // Create manifest with training disallowed
    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });
    fs::write(
        temp_dir.path().join("llms.txt"),
        serde_yaml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    // Create robots.txt that properly blocks training bots
    let robots = r#"
User-agent: GPTBot
Disallow: /

User-agent: CCBot
Disallow: /

# ARW Discovery
# See llms.txt for agent-specific policies
"#;
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.path == "robots.txt"),
        "Should pass when robots.txt properly blocks training. Errors: {:?}",
        errors
    );
}

#[tokio::test]
async fn test_robots_txt_missing_blocks_when_training_disallowed() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });
    fs::write(
        temp_dir.path().join("llms.txt"),
        serde_yaml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    // Create robots.txt without proper blocks
    let robots = "User-agent: *\nAllow: /\n";
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors
            .iter()
            .any(|e| e.path == "robots.txt" && e.message.contains("block training")),
        "Should detect missing training bot blocks"
    );
}

#[tokio::test]
async fn test_robots_txt_missing_arw_hints() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": true},
            "inference": {"allowed": true},
            "attribution": {"required": false}
        }
    });
    fs::write(
        temp_dir.path().join("llms.txt"),
        serde_yaml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    // Create robots.txt without ARW hints
    let robots = "User-agent: *\nAllow: /\n";
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors
            .iter()
            .any(|e| e.path == "robots.txt" && e.message.contains("ARW discovery hints")),
        "Should detect missing ARW hints in robots.txt"
    );
}

#[tokio::test]
async fn test_robots_txt_optional() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": true},
            "inference": {"allowed": true},
            "attribution": {"required": false}
        }
    });
    fs::write(
        temp_dir.path().join("llms.txt"),
        serde_yaml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    // No robots.txt file

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    // Should not fail - robots.txt is optional
    assert!(
        errors.is_empty(),
        "Should pass when robots.txt is missing (optional)"
    );
}

// ============================================================================
// MISSING LLMS.TXT TESTS
// ============================================================================

#[tokio::test]
async fn test_missing_llms_txt() {
    let temp_dir = TempDir::new().unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors
            .iter()
            .any(|e| e.path == "llms.txt" && e.message.contains("not found")),
        "Should detect missing llms.txt"
    );
}

// ============================================================================
// MULTIPLE CONTENT ITEMS TESTS
// ============================================================================

#[tokio::test]
async fn test_multiple_pages_with_chunks() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![
        serde_json::json!({
            "url": "/page1",
            "machine_view": "/page1.llm.md",
            "purpose": "documentation",
            "chunks": [
                {"id": "intro", "heading": "Introduction"}
            ]
        }),
        serde_json::json!({
            "url": "/page2",
            "machine_view": "/page2.llm.md",
            "purpose": "documentation",
            "chunks": [
                {"id": "overview", "heading": "Overview"}
            ]
        }),
    ];
    let manifest = create_test_manifest(content_items);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    fs::write(
        temp_dir.path().join("page1.llm.md"),
        create_markdown_with_chunks(vec!["intro"]),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("page2.llm.md"),
        create_markdown_with_chunks(vec!["overview"]),
    )
    .unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors.is_empty(),
        "Should validate multiple pages correctly"
    );
}

#[tokio::test]
async fn test_one_page_valid_one_invalid() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![
        serde_json::json!({
            "url": "/page1",
            "machine_view": "/page1.llm.md",
            "purpose": "documentation",
            "chunks": [
                {"id": "intro", "heading": "Introduction"}
            ]
        }),
        serde_json::json!({
            "url": "/page2",
            "machine_view": "/page2.llm.md",
            "purpose": "documentation",
            "chunks": [
                {"id": "missing", "heading": "Missing Chunk"}
            ]
        }),
    ];
    let manifest = create_test_manifest(content_items);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    fs::write(
        temp_dir.path().join("page1.llm.md"),
        create_markdown_with_chunks(vec!["intro"]),
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("page2.llm.md"),
        create_markdown_with_chunks(vec!["different"]),
    )
    .unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert_eq!(
        errors.len(),
        2,
        "Should detect errors in second page only"
    );
    assert!(
        errors.iter().all(|e| e.path.contains("content[1]")),
        "Errors should be for second page only"
    );
}
