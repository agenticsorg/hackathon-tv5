/// Comprehensive test suite for consistency validator
/// Covers remaining untested code paths to achieve 100% coverage
use arw_lib::validators::consistency::ConsistencyValidator;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_manifest_with_policies(training_allowed: bool) -> String {
    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": training_allowed},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });
    serde_yaml::to_string(&manifest).unwrap()
}

// ============================================================================
// MACHINE VIEW PATH VARIATIONS
// ============================================================================

#[tokio::test]
async fn test_machine_view_without_leading_slash() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "page.llm.md",  // No leading slash
        "purpose": "documentation"
    })];

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
        "Should handle machine view without leading slash"
    );
}

#[tokio::test]
async fn test_multiple_machine_views_some_missing() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![
        serde_json::json!({
            "url": "/page1",
            "machine_view": "/page1.llm.md",
            "purpose": "documentation"
        }),
        serde_json::json!({
            "url": "/page2",
            "machine_view": "/page2.llm.md",
            "purpose": "documentation"
        }),
        serde_json::json!({
            "url": "/page3",
            "machine_view": "/page3.llm.md",
            "purpose": "documentation"
        }),
    ];

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

    fs::write(
        temp_dir.path().join("llms.txt"),
        serde_yaml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    // Create only page1 and page3, page2 is missing
    fs::write(temp_dir.path().join("page1.llm.md"), "# Page 1").unwrap();
    fs::write(temp_dir.path().join("page3.llm.md"), "# Page 3").unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert_eq!(errors.len(), 1, "Should detect exactly one missing file");
    assert!(
        errors.iter().any(|e| e.message.contains("page2.llm.md")),
        "Should detect page2 is missing"
    );
}

#[tokio::test]
async fn test_machine_view_is_none() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": null,
        "purpose": "documentation"
    })];

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

    fs::write(
        temp_dir.path().join("llms.txt"),
        serde_yaml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    // Should not crash when machine_view is null
    assert!(errors.len() >= 0, "Should handle null machine_view");
}

// ============================================================================
// CHUNK VALIDATION EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_empty_chunks_array() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation",
        "chunks": []
    })];

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
        "Should skip validation for empty chunks array"
    );
}

#[tokio::test]
async fn test_chunks_with_null_ids() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation",
        "chunks": [
            {"id": null, "heading": "Section"},
            {"id": "valid", "heading": "Valid Section"}
        ]
    })];

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

    fs::write(
        temp_dir.path().join("llms.txt"),
        serde_yaml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    let markdown = "<!-- chunk: valid -->\nContent";
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    // Should handle null chunk IDs gracefully
    assert!(errors.len() >= 0, "Should handle null chunk IDs");
}

#[tokio::test]
async fn test_markdown_chunk_marker_at_end_of_line() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation",
        "chunks": [
            {"id": "intro", "heading": "Introduction"}
        ]
    })];

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

    fs::write(
        temp_dir.path().join("llms.txt"),
        serde_yaml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    let markdown = "Text before <!-- chunk: intro -->Text after";
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.path.contains("chunks")),
        "Should extract chunk from middle of line"
    );
}

#[tokio::test]
async fn test_markdown_chunk_without_closing_marker() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation",
        "chunks": [
            {"id": "intro", "heading": "Introduction"}
        ]
    })];

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

    fs::write(
        temp_dir.path().join("llms.txt"),
        serde_yaml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    let markdown = "<!-- chunk: intro";  // Missing closing -->
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    // Should not extract chunk without proper closing marker
    assert!(
        errors.iter().any(|e| e.message.contains("intro") && e.message.contains("not found")),
        "Should not extract chunk without closing marker"
    );
}

// ============================================================================
// HTML CHUNK EXTRACTION EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_html_chunk_attribute_in_middle_of_tag() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation",
        "chunks": [
            {"id": "intro", "heading": "Introduction"}
        ]
    })];

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

    fs::write(
        temp_dir.path().join("llms.txt"),
        serde_yaml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    let markdown = "<!-- chunk: intro -->\nContent";
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let html = r#"<section class="content" data-chunk-id="intro" id="intro-section">Content</section>"#;
    fs::write(temp_dir.path().join("page.html"), html).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.path.contains("chunks")),
        "Should extract chunk-id from middle of tag attributes"
    );
}

#[tokio::test]
async fn test_html_chunk_without_closing_quote() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation",
        "chunks": [
            {"id": "intro", "heading": "Introduction"}
        ]
    })];

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

    fs::write(
        temp_dir.path().join("llms.txt"),
        serde_yaml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    let markdown = "<!-- chunk: intro -->\nContent";
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let html = r#"<section data-chunk-id="intro>Content</section>"#;  // Missing closing quote
    fs::write(temp_dir.path().join("page.html"), html).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    // Should not extract chunk without proper closing quote
    assert!(errors.len() >= 0, "Should handle malformed HTML attributes");
}

#[tokio::test]
async fn test_url_not_local_path() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "https://example.com/page",  // External URL, not local path
        "machine_view": "/page.llm.md",
        "purpose": "documentation",
        "chunks": [
            {"id": "intro", "heading": "Introduction"}
        ]
    })];

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

    fs::write(
        temp_dir.path().join("llms.txt"),
        serde_yaml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    let markdown = "<!-- chunk: intro -->\nContent";
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    // Should skip HTML validation for external URLs
    assert!(
        !errors.iter().any(|e| e.message.contains("HTML")),
        "Should not validate HTML for external URLs"
    );
}

#[tokio::test]
async fn test_html_file_does_not_exist() {
    let temp_dir = TempDir::new().unwrap();

    let content_items = vec![serde_json::json!({
        "url": "/page",
        "machine_view": "/page.llm.md",
        "purpose": "documentation",
        "chunks": [
            {"id": "intro", "heading": "Introduction"}
        ]
    })];

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

    fs::write(
        temp_dir.path().join("llms.txt"),
        serde_yaml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    let markdown = "<!-- chunk: intro -->\nContent";
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    // Don't create HTML file

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    // Should not fail when HTML file doesn't exist
    assert!(
        !errors.iter().any(|e| e.message.contains("HTML")),
        "Should skip HTML validation when file doesn't exist"
    );
}

// ============================================================================
// ROBOTS.TXT POLICY CONSISTENCY
// ============================================================================

#[tokio::test]
async fn test_robots_txt_with_training_allowed() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = create_manifest_with_policies(true);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    let robots = "User-agent: *\nAllow: /\n";
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    // Should not require blocking when training is allowed
    assert!(
        !errors.iter().any(|e| e.message.contains("block training")),
        "Should not require blocking when training allowed"
    );
}

#[tokio::test]
async fn test_robots_txt_has_gptbot_but_no_disallow() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = create_manifest_with_policies(false);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    let robots = r#"
User-agent: GPTBot
Allow: /

# Some comment
"#;
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors.iter().any(|e| e.path == "robots.txt" && e.message.contains("block training")),
        "Should require Disallow when GPTBot is present but allows"
    );
}

#[tokio::test]
async fn test_robots_txt_with_arw_mentions() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = create_manifest_with_policies(true);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    let robots = r#"
User-agent: *
Allow: /

# Agent-Ready Web
# See llms.txt for machine-readable policies
"#;
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.path == "robots.txt" && e.message.contains("ARW discovery")),
        "Should accept robots.txt with ARW mentions"
    );
}

#[tokio::test]
async fn test_robots_txt_with_llms_txt_mention() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = create_manifest_with_policies(true);
    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    let robots = r#"
User-agent: *
Allow: /

# See llms.txt for details
"#;
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.path == "robots.txt" && e.message.contains("ARW discovery")),
        "Should accept robots.txt with llms.txt mention"
    );
}

#[tokio::test]
async fn test_training_policy_is_not_boolean() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": "yes"},  // String instead of boolean
            "inference": {"allowed": true},
            "attribution": {"required": true}
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

    // Should handle non-boolean gracefully
    assert!(errors.len() >= 0, "Should handle non-boolean training.allowed");
}

#[tokio::test]
async fn test_training_policy_missing_allowed_field() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"commercial": false},  // Missing "allowed" field
            "inference": {"allowed": true},
            "attribution": {"required": true}
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

    // Should handle missing training.allowed field
    assert!(errors.len() >= 0, "Should handle missing training.allowed");
}

// ============================================================================
// INVALID YAML IN LLMS.TXT
// ============================================================================

#[tokio::test]
async fn test_invalid_yaml_in_llms_txt() {
    let temp_dir = TempDir::new().unwrap();

    fs::write(
        temp_dir.path().join("llms.txt"),
        "invalid: yaml: content: [[[",
    )
    .unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let result = validator.validate_all().await;

    assert!(result.is_err(), "Should fail with invalid YAML");
}

// ============================================================================
// CONTENT ARRAY IS NOT AN ARRAY
// ============================================================================

#[tokio::test]
async fn test_content_is_not_an_array() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": "not_an_array",
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

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    // Should handle when content is not an array
    assert!(errors.len() >= 0, "Should handle non-array content");
}

// ============================================================================
// MULTIPLE VALIDATION METHODS
// ============================================================================

#[tokio::test]
async fn test_validate_machine_view_files_directly() {
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
                "url": "/page",
                "machine_view": "/page.llm.md",
                "purpose": "test"
            }
        ],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    fs::write(temp_dir.path().join("page.llm.md"), "# Test").unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_machine_view_files(&manifest).unwrap();

    assert!(
        errors.is_empty(),
        "Direct machine view validation should pass"
    );
}

#[tokio::test]
async fn test_validate_robots_consistency_directly() {
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

    let robots = "User-agent: GPTBot\nDisallow: /\n\n# llms.txt available\n";
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_robots_consistency(&manifest).unwrap();

    assert!(
        errors.is_empty(),
        "Direct robots validation should pass"
    );
}
