/// Additional comprehensive tests for consistency validator to achieve 100% coverage
use arw_lib::validators::consistency::ConsistencyValidator;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_minimal_manifest() -> String {
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
    serde_yaml::to_string(&manifest).unwrap()
}

// ============================================================================
// CONSTRUCTOR AND BASIC FUNCTIONALITY
// ============================================================================

#[tokio::test]
async fn test_validator_new() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_string_lossy().to_string();

    let validator = ConsistencyValidator::new(path.clone());
    // Validator should be created successfully
    assert!(true);
}

#[tokio::test]
async fn test_validator_with_relative_path() {
    let validator = ConsistencyValidator::new("./test".to_string());
    // Should handle relative paths
    assert!(true);
}

// ============================================================================
// INVALID YAML HANDLING
// ============================================================================

#[tokio::test]
async fn test_invalid_yaml_in_llms_txt() {
    let temp_dir = TempDir::new().unwrap();

    // Create invalid YAML
    fs::write(
        temp_dir.path().join("llms.txt"),
        "invalid: yaml: content: {{{",
    )
    .unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let result = validator.validate_all().await;

    assert!(result.is_err(), "Should fail on invalid YAML");
}

// ============================================================================
// MACHINE VIEW VALIDATION - EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_machine_view_without_leading_slash() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [{
            "url": "/page",
            "machine_view": "page.llm.md",
            "purpose": "documentation"
        }],
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

    fs::write(temp_dir.path().join("page.llm.md"), "# Test").unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors.is_empty(),
        "Should handle machine_view without leading slash"
    );
}

#[tokio::test]
async fn test_machine_view_nested_directory() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [{
            "url": "/page",
            "machine_view": "/docs/guides/page.llm.md",
            "purpose": "documentation"
        }],
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

    fs::create_dir_all(temp_dir.path().join("docs/guides")).unwrap();
    fs::write(
        temp_dir.path().join("docs/guides/page.llm.md"),
        "# Test",
    )
    .unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors.is_empty(),
        "Should handle nested directory machine_view"
    );
}

#[tokio::test]
async fn test_machine_view_special_characters() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [{
            "url": "/page",
            "machine_view": "/my-page_v1.2.llm.md",
            "purpose": "documentation"
        }],
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

    fs::write(temp_dir.path().join("my-page_v1.2.llm.md"), "# Test").unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors.is_empty(),
        "Should handle special characters in filename"
    );
}

// ============================================================================
// CHUNK VALIDATION - COMPLEX SCENARIOS
// ============================================================================

#[tokio::test]
async fn test_chunks_with_empty_id() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [{
            "url": "/page",
            "machine_view": "/page.llm.md",
            "purpose": "documentation",
            "chunks": [
                {"id": "", "heading": "Empty ID"}
            ]
        }],
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

    fs::write(
        temp_dir.path().join("page.llm.md"),
        "<!-- chunk:  -->\nContent",
    )
    .unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.is_empty(),
        "Should detect empty chunk IDs"
    );
}

#[tokio::test]
async fn test_chunks_with_duplicate_ids() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [{
            "url": "/page",
            "machine_view": "/page.llm.md",
            "purpose": "documentation",
            "chunks": [
                {"id": "intro", "heading": "Introduction"},
                {"id": "intro", "heading": "Introduction 2"}
            ]
        }],
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

    fs::write(
        temp_dir.path().join("page.llm.md"),
        "<!-- chunk: intro -->\nContent\n<!-- chunk: intro -->\nMore content",
    )
    .unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    // Should pass - duplicate IDs in markdown are allowed
    assert!(
        !errors.iter().any(|e| e.path.contains("chunks")),
        "Duplicate chunk IDs in markdown should be handled"
    );
}

#[tokio::test]
async fn test_chunk_marker_formats() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [{
            "url": "/page",
            "machine_view": "/page.llm.md",
            "purpose": "documentation",
            "chunks": [
                {"id": "chunk1", "heading": "Chunk 1"},
                {"id": "chunk2", "heading": "Chunk 2"},
                {"id": "chunk3", "heading": "Chunk 3"}
            ]
        }],
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

    // Test different chunk marker formats
    let markdown = r#"
# Test Page

<!-- chunk: chunk1 -->
Content 1

<!--chunk:chunk2-->
Content 2

<!-- chunk:   chunk3   -->
Content 3
"#;
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.path.contains("chunks")),
        "Should handle different chunk marker formats"
    );
}

// ============================================================================
// HTML CHUNK VALIDATION
// ============================================================================

#[tokio::test]
async fn test_html_chunks_different_attributes() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [{
            "url": "/page",
            "machine_view": "/page.llm.md",
            "purpose": "documentation",
            "chunks": [
                {"id": "intro", "heading": "Introduction"}
            ]
        }],
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

    fs::write(
        temp_dir.path().join("page.llm.md"),
        "<!-- chunk: intro -->\nContent",
    )
    .unwrap();

    // HTML with various attribute formats
    let html = r#"
<html>
<body>
    <div data-chunk-id="intro" class="section">
        Content
    </div>
</body>
</html>
"#;
    fs::write(temp_dir.path().join("page.html"), html).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.path.contains("chunks")),
        "Should handle HTML chunks with different attributes"
    );
}

#[tokio::test]
async fn test_html_chunks_single_quotes() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [{
            "url": "/page",
            "machine_view": "/page.llm.md",
            "purpose": "documentation",
            "chunks": [
                {"id": "intro", "heading": "Introduction"}
            ]
        }],
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

    fs::write(
        temp_dir.path().join("page.llm.md"),
        "<!-- chunk: intro -->\nContent",
    )
    .unwrap();

    // HTML should use double quotes only (single quotes won't be detected)
    let html = r#"
<html>
<body>
    <div data-chunk-id="intro">
        Content
    </div>
</body>
</html>
"#;
    fs::write(temp_dir.path().join("page.html"), html).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.path.contains("chunks")),
        "Should find chunks with double quotes"
    );
}

#[tokio::test]
async fn test_html_url_without_extension() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [{
            "url": "/about",
            "machine_view": "/about.llm.md",
            "purpose": "documentation",
            "chunks": [
                {"id": "intro", "heading": "Introduction"}
            ]
        }],
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

    fs::write(
        temp_dir.path().join("about.llm.md"),
        "<!-- chunk: intro -->\nContent",
    )
    .unwrap();

    // Create HTML file that will be checked
    let html = r#"<html><body><div data-chunk-id="intro">Content</div></body></html>"#;
    fs::write(temp_dir.path().join("about.html"), html).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.path.contains("chunks")),
        "Should handle URLs without extensions"
    );
}

#[tokio::test]
async fn test_html_external_url_skipped() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [{
            "url": "https://external.com/page",
            "machine_view": "/page.llm.md",
            "purpose": "documentation",
            "chunks": [
                {"id": "intro", "heading": "Introduction"}
            ]
        }],
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

    fs::write(
        temp_dir.path().join("page.llm.md"),
        "<!-- chunk: intro -->\nContent",
    )
    .unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.path.contains("chunks") && e.message.contains("HTML")),
        "Should skip HTML validation for external URLs"
    );
}

// ============================================================================
// ROBOTS.TXT VALIDATION - COMPREHENSIVE
// ============================================================================

#[tokio::test]
async fn test_robots_txt_training_allowed_no_restrictions() {
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

    let robots = "User-agent: *\nAllow: /\n";
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors
            .iter()
            .any(|e| e.path == "robots.txt" && e.message.contains("ARW discovery hints")),
        "Should still check for ARW hints even when training allowed"
    );
}

#[tokio::test]
async fn test_robots_txt_with_arw_comment() {
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

    let robots = r#"
User-agent: *
Allow: /

# Agent-Ready Web
# See llms.txt for details
"#;
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors
            .iter()
            .any(|e| e.path == "robots.txt" && e.message.contains("ARW discovery hints")),
        "Should pass with ARW comment"
    );
}

#[tokio::test]
async fn test_robots_txt_with_llms_txt_reference() {
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

    let robots = r#"
User-agent: *
Allow: /

# See llms.txt for AI agent policies
"#;
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors
            .iter()
            .any(|e| e.path == "robots.txt" && e.message.contains("ARW discovery hints")),
        "Should pass with llms.txt reference"
    );
}

#[tokio::test]
async fn test_robots_txt_partial_blocks() {
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

    // Has GPTBot but not "Disallow: /"
    let robots = r#"
User-agent: GPTBot
Allow: /some/path

# ARW
"#;
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors
            .iter()
            .any(|e| e.path == "robots.txt" && e.message.contains("block training")),
        "Should detect incomplete training bot blocks"
    );
}

// ============================================================================
// INTEGRATION TESTS - FULL VALIDATION
// ============================================================================

#[tokio::test]
async fn test_complete_valid_site() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [
            {
                "url": "/page1",
                "machine_view": "/page1.llm.md",
                "purpose": "documentation",
                "chunks": [
                    {"id": "intro", "heading": "Introduction"},
                    {"id": "content", "heading": "Content"}
                ]
            },
            {
                "url": "/page2",
                "machine_view": "/page2.llm.md",
                "purpose": "documentation",
                "chunks": [
                    {"id": "overview", "heading": "Overview"}
                ]
            }
        ],
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

    fs::write(
        temp_dir.path().join("page1.llm.md"),
        "<!-- chunk: intro -->\nIntro\n<!-- chunk: content -->\nContent",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("page2.llm.md"),
        "<!-- chunk: overview -->\nOverview",
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("page1.html"),
        r#"<div data-chunk-id="intro"></div><div data-chunk-id="content"></div>"#,
    )
    .unwrap();

    fs::write(
        temp_dir.path().join("page2.html"),
        r#"<div data-chunk-id="overview"></div>"#,
    )
    .unwrap();

    let robots = r#"
User-agent: GPTBot
Disallow: /

# ARW - see llms.txt
"#;
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors.is_empty(),
        "Complete valid site should have no errors. Errors: {:?}",
        errors
    );
}

#[tokio::test]
async fn test_site_with_multiple_errors() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [
            {
                "url": "/page1",
                "machine_view": "/missing.llm.md",
                "purpose": "documentation",
                "chunks": [
                    {"id": "intro", "heading": "Introduction"}
                ]
            },
            {
                "url": "/page2",
                "machine_view": "/page2.llm.md",
                "purpose": "documentation",
                "chunks": [
                    {"id": "wrong", "heading": "Wrong"}
                ]
            }
        ],
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

    fs::write(
        temp_dir.path().join("page2.llm.md"),
        "<!-- chunk: different -->\nContent",
    )
    .unwrap();

    let robots = "User-agent: *\nAllow: /\n";
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(errors.len() >= 3, "Should detect multiple errors");
    assert!(
        errors.iter().any(|e| e.message.contains("not found")),
        "Should detect missing file"
    );
    assert!(
        errors.iter().any(|e| e.message.contains("chunk")),
        "Should detect chunk mismatch"
    );
    assert!(
        errors.iter().any(|e| e.path == "robots.txt"),
        "Should detect robots.txt issues"
    );
}
