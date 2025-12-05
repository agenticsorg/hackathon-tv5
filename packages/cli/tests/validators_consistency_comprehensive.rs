/// Comprehensive integration tests for consistency validator
/// Covers remaining untested code paths
use arw_cli::validators::consistency::ConsistencyValidator;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// MACHINE VIEW PATH VARIATIONS
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
            "machine_view": "page.llm.md",  // No leading slash
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
        "Should handle machine view without leading slash"
    );
}

#[tokio::test]
async fn test_markdown_chunk_without_closing_marker() {
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

    let markdown = "<!-- chunk: intro";  // Missing closing -->
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        errors.iter().any(|e| e.message.contains("intro") && e.message.contains("not found")),
        "Should not extract chunk without closing marker"
    );
}

#[tokio::test]
async fn test_url_not_local_path() {
    let temp_dir = TempDir::new().unwrap();

    let manifest = serde_json::json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [{
            "url": "https://example.com/page",  // External URL
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

    let markdown = "<!-- chunk: intro -->\nContent";
    fs::write(temp_dir.path().join("page.llm.md"), markdown).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.message.contains("HTML")),
        "Should not validate HTML for external URLs"
    );
}

#[tokio::test]
async fn test_robots_txt_with_training_allowed() {
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

    assert!(
        !errors.iter().any(|e| e.message.contains("block training")),
        "Should not require blocking when training allowed"
    );
}

#[tokio::test]
async fn test_robots_txt_with_llms_txt_mention() {
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
            "attribution": {"required": true}
        }
    });

    fs::write(
        temp_dir.path().join("llms.txt"),
        serde_yaml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    let robots = "User-agent: *\nAllow: /\n\n# See llms.txt for details\n";
    fs::write(temp_dir.path().join("robots.txt"), robots).unwrap();

    let validator = ConsistencyValidator::new(temp_dir.path().to_string_lossy().to_string());
    let errors = validator.validate_all().await.unwrap();

    assert!(
        !errors.iter().any(|e| e.path == "robots.txt" && e.message.contains("ARW discovery")),
        "Should accept robots.txt with llms.txt mention"
    );
}

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
        "content": [{
            "url": "/page",
            "machine_view": "/page.llm.md",
            "purpose": "test"
        }],
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
