/// Comprehensive test suite for llms_txt validator
/// Tests all validation rules, edge cases, and error conditions
use arw_lib::validators::llms_txt::{validate, validate_manifest, ValidationError};
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_minimal_valid_manifest() -> serde_json::Value {
    json!({
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
    })
}

fn write_manifest_to_file(temp_dir: &Path, filename: &str, manifest: &serde_json::Value) -> String {
    let yaml_content = serde_yaml::to_string(manifest).unwrap();
    let manifest_path = temp_dir.join(filename);
    fs::write(&manifest_path, yaml_content).unwrap();
    manifest_path.to_string_lossy().to_string()
}

// ============================================================================
// VALID MANIFEST TESTS
// ============================================================================

#[test]
fn test_minimal_valid_manifest() {
    let manifest = create_minimal_valid_manifest();
    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.is_empty(),
        "Minimal valid manifest should pass validation. Errors: {:?}",
        errors
    );
}

#[test]
fn test_complete_valid_manifest_arw1() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "description": "A comprehensive test site",
            "homepage": "https://example.com",
            "contact": "test@example.com"
        },
        "content": [
            {
                "url": "/",
                "machine_view": "/index.llm.md",
                "purpose": "homepage",
                "priority": "high"
            }
        ],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.is_empty(),
        "Complete valid ARW-1 manifest should pass. Errors: {:?}",
        errors
    );
}

#[test]
fn test_valid_manifest_arw3_with_actions() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-3",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [
            {
                "url": "/products/item",
                "machine_view": "/products/item.llm.md",
                "purpose": "product_information",
                "priority": "high"
            }
        ],
        "actions": [
            {
                "id": "add_to_cart",
                "name": "Add to Cart",
                "endpoint": "/api/cart/add",
                "method": "POST",
                "auth": "oauth2"
            }
        ],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.is_empty(),
        "Valid ARW-3 manifest with actions should pass. Errors: {:?}",
        errors
    );
}

#[test]
fn test_valid_manifest_with_chunks() {
    let manifest = json!({
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
                "purpose": "documentation",
                "chunks": [
                    {
                        "id": "intro",
                        "heading": "Introduction",
                        "description": "Introduction section"
                    },
                    {
                        "id": "main",
                        "heading": "Main Content"
                    }
                ]
            }
        ],
        "policies": {
            "training": {"allowed": true},
            "inference": {"allowed": true},
            "attribution": {"required": false}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.is_empty(),
        "Valid manifest with chunks should pass. Errors: {:?}",
        errors
    );
}

// ============================================================================
// MISSING REQUIRED FIELDS TESTS
// ============================================================================

#[test]
fn test_missing_version() {
    let mut manifest = create_minimal_valid_manifest();
    manifest.as_object_mut().unwrap().remove("version");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(!errors.is_empty(), "Should have errors for missing version");
    assert!(
        errors.iter().any(|e| e.path == "version"),
        "Should have error for version field"
    );
}

#[test]
fn test_empty_version() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["version"] = json!("");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "version" && e.message.contains("non-empty")),
        "Should reject empty version"
    );
}

#[test]
fn test_missing_profile() {
    let mut manifest = create_minimal_valid_manifest();
    manifest.as_object_mut().unwrap().remove("profile");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "profile"),
        "Should have error for missing profile"
    );
}

#[test]
fn test_invalid_profile_value() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["profile"] = json!("INVALID-PROFILE");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "profile" && e.message.contains("ARW-1, ARW-2, ARW-3, ARW-4")),
        "Should reject invalid profile value"
    );
}

#[test]
fn test_missing_site_section() {
    let mut manifest = create_minimal_valid_manifest();
    manifest.as_object_mut().unwrap().remove("site");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "site"),
        "Should have error for missing site section"
    );
}

#[test]
fn test_missing_site_name() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["site"].as_object_mut().unwrap().remove("name");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "site.name"),
        "Should have error for missing site.name"
    );
}

#[test]
fn test_empty_site_name() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["site"]["name"] = json!("");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "site.name" && e.message.contains("non-empty")),
        "Should reject empty site.name"
    );
}

#[test]
fn test_missing_site_homepage() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["site"].as_object_mut().unwrap().remove("homepage");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "site.homepage"),
        "Should have error for missing site.homepage"
    );
}

#[test]
fn test_missing_policies() {
    let mut manifest = create_minimal_valid_manifest();
    manifest.as_object_mut().unwrap().remove("policies");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "policies"),
        "Should have error for missing policies"
    );
}

#[test]
fn test_missing_training_policy() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["policies"].as_object_mut().unwrap().remove("training");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "policies.training"),
        "Should have error for missing policies.training"
    );
}

#[test]
fn test_missing_inference_policy() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["policies"].as_object_mut().unwrap().remove("inference");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "policies.inference"),
        "Should have error for missing policies.inference"
    );
}

#[test]
fn test_missing_attribution_policy() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["policies"].as_object_mut().unwrap().remove("attribution");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "policies.attribution"),
        "Should have error for missing policies.attribution"
    );
}

// ============================================================================
// FIELD FORMAT VALIDATION TESTS
// ============================================================================

#[test]
fn test_invalid_homepage_url_no_protocol() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["site"]["homepage"] = json!("example.com");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "site.homepage" && e.message.contains("http")),
        "Should reject URL without protocol"
    );
}

#[test]
fn test_invalid_homepage_url_ftp() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["site"]["homepage"] = json!("ftp://example.com");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "site.homepage" && e.message.contains("http")),
        "Should reject non-HTTP(S) URLs"
    );
}

#[test]
fn test_valid_homepage_http() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["site"]["homepage"] = json!("http://example.com");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "site.homepage"),
        "Should accept HTTP URL"
    );
}

#[test]
fn test_valid_homepage_https() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["site"]["homepage"] = json!("https://example.com");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "site.homepage"),
        "Should accept HTTPS URL"
    );
}

#[test]
fn test_invalid_email_format() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["site"]["contact"] = json!("not-an-email");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "site.contact" && e.message.contains("email")),
        "Should reject invalid email format"
    );
}

#[test]
fn test_valid_email_format() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["site"]["contact"] = json!("test@example.com");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "site.contact"),
        "Should accept valid email"
    );
}

#[test]
fn test_valid_email_with_subdomain() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["site"]["contact"] = json!("admin@mail.example.co.uk");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "site.contact"),
        "Should accept email with subdomain"
    );
}

// ============================================================================
// CONTENT VALIDATION TESTS
// ============================================================================

#[test]
fn test_content_missing_url() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["content"] = json!([
        {
            "machine_view": "/page.llm.md",
            "purpose": "page"
        }
    ]);

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path.contains("content[0].url")),
        "Should require url field in content"
    );
}

#[test]
fn test_content_missing_machine_view() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["content"] = json!([
        {
            "url": "/page",
            "purpose": "page"
        }
    ]);

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path.contains("content[0].machine_view")),
        "Should require machine_view field in content"
    );
}

#[test]
fn test_content_invalid_priority() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["content"] = json!([
        {
            "url": "/page",
            "machine_view": "/page.llm.md",
            "purpose": "page",
            "priority": "super-high"
        }
    ]);

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path.contains("priority") && e.message.contains("high, medium, low")),
        "Should reject invalid priority value"
    );
}

#[test]
fn test_content_valid_priorities() {
    for priority in &["high", "medium", "low"] {
        let mut manifest = create_minimal_valid_manifest();
        manifest["content"] = json!([
            {
                "url": "/page",
                "machine_view": "/page.llm.md",
                "purpose": "page",
                "priority": priority
            }
        ]);

        let errors = validate_manifest(&manifest).unwrap();
        assert!(
            !errors.iter().any(|e| e.path.contains("priority")),
            "Should accept priority: {}",
            priority
        );
    }
}

#[test]
fn test_chunk_missing_id() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["content"] = json!([
        {
            "url": "/page",
            "machine_view": "/page.llm.md",
            "purpose": "page",
            "chunks": [
                {
                    "heading": "Section 1"
                }
            ]
        }
    ]);

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path.contains("chunks[0].id")),
        "Should require id field in chunks"
    );
}

// ============================================================================
// ACTIONS VALIDATION TESTS (ARW-3)
// ============================================================================

#[test]
fn test_action_missing_required_fields() {
    let required_fields = vec!["id", "name", "endpoint", "method", "auth"];

    for field in required_fields {
        let mut manifest = create_minimal_valid_manifest();
        manifest["profile"] = json!("ARW-3");

        let mut action = json!({
            "id": "test_action",
            "name": "Test Action",
            "endpoint": "/api/test",
            "method": "POST",
            "auth": "none"
        });

        action.as_object_mut().unwrap().remove(field);
        manifest["actions"] = json!([action]);

        let errors = validate_manifest(&manifest).unwrap();
        assert!(
            errors.iter().any(|e| e.path.contains(&format!("actions[0].{}", field))),
            "Should require {} field in actions",
            field
        );
    }
}

#[test]
fn test_action_invalid_method() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["profile"] = json!("ARW-3");
    manifest["actions"] = json!([
        {
            "id": "test_action",
            "name": "Test Action",
            "endpoint": "/api/test",
            "method": "INVALID",
            "auth": "none"
        }
    ]);

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path.contains("method") && e.message.contains("GET, POST, PUT, PATCH, DELETE")),
        "Should reject invalid HTTP method"
    );
}

#[test]
fn test_action_valid_methods() {
    for method in &["GET", "POST", "PUT", "PATCH", "DELETE"] {
        let mut manifest = create_minimal_valid_manifest();
        manifest["profile"] = json!("ARW-3");
        manifest["actions"] = json!([
            {
                "id": "test_action",
                "name": "Test Action",
                "endpoint": "/api/test",
                "method": method,
                "auth": "none"
            }
        ]);

        let errors = validate_manifest(&manifest).unwrap();
        assert!(
            !errors.iter().any(|e| e.path.contains("method")),
            "Should accept method: {}",
            method
        );
    }
}

#[test]
fn test_action_invalid_auth() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["profile"] = json!("ARW-3");
    manifest["actions"] = json!([
        {
            "id": "test_action",
            "name": "Test Action",
            "endpoint": "/api/test",
            "method": "POST",
            "auth": "basic_auth"
        }
    ]);

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path.contains("auth") && e.message.contains("oauth2, api_key, none")),
        "Should reject invalid auth type"
    );
}

#[test]
fn test_action_valid_auth_types() {
    for auth in &["oauth2", "api_key", "none"] {
        let mut manifest = create_minimal_valid_manifest();
        manifest["profile"] = json!("ARW-3");
        manifest["actions"] = json!([
            {
                "id": "test_action",
                "name": "Test Action",
                "endpoint": "/api/test",
                "method": "POST",
                "auth": auth
            }
        ]);

        let errors = validate_manifest(&manifest).unwrap();
        assert!(
            !errors.iter().any(|e| e.path.contains("auth")),
            "Should accept auth: {}",
            auth
        );
    }
}

// ============================================================================
// FILE VALIDATION TESTS
// ============================================================================

#[test]
fn test_validate_file_success() {
    let temp_dir = TempDir::new().unwrap();
    let manifest = create_minimal_valid_manifest();
    let manifest_path = write_manifest_to_file(temp_dir.path(), "llms.txt", &manifest);

    let result = validate(Path::new(&manifest_path));
    assert!(result.is_ok(), "Should successfully validate file");

    let errors = result.unwrap();
    assert!(errors.is_empty(), "Should have no validation errors");
}

#[test]
fn test_validate_nonexistent_file() {
    let result = validate(Path::new("/nonexistent/path/llms.txt"));
    assert!(result.is_err(), "Should fail for nonexistent file");
}

#[test]
fn test_validate_invalid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("llms.txt");
    fs::write(&manifest_path, "invalid: yaml: content: [").unwrap();

    let result = validate(&manifest_path);
    assert!(result.is_err(), "Should fail for invalid YAML");
}

// ============================================================================
// EDGE CASES AND SPECIAL CHARACTERS
// ============================================================================

#[test]
fn test_special_characters_in_site_name() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["site"]["name"] = json!("Test Site™ with \"quotes\" and 'apostrophes'");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "site.name"),
        "Should handle special characters in site name"
    );
}

#[test]
fn test_unicode_in_description() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["site"]["description"] = json!("Test site with emoji 🚀 and Chinese 测试");

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.is_empty(),
        "Should handle Unicode characters"
    );
}

#[test]
fn test_numeric_version() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["version"] = json!(1.0);

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "version"),
        "Should accept numeric version"
    );
}

#[test]
fn test_very_long_url() {
    let long_path = format!("/{}", "a".repeat(2000));
    let mut manifest = create_minimal_valid_manifest();
    manifest["content"] = json!([
        {
            "url": long_path,
            "machine_view": "/page.llm.md",
            "purpose": "test"
        }
    ]);

    let errors = validate_manifest(&manifest).unwrap();
    // Should not crash, URL validation is format-based not length-based
    assert!(
        !errors.iter().any(|e| e.path.contains("content[0].url")),
        "Should handle very long URLs"
    );
}

#[test]
fn test_empty_content_array() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["content"] = json!([]);

    let errors = validate_manifest(&manifest).unwrap();
    // Empty content array is valid - it's optional
    assert!(
        !errors.iter().any(|e| e.path.contains("content")),
        "Should accept empty content array"
    );
}

#[test]
fn test_empty_actions_array() {
    let mut manifest = create_minimal_valid_manifest();
    manifest["profile"] = json!("ARW-3");
    manifest["actions"] = json!([]);

    let errors = validate_manifest(&manifest).unwrap();
    // Empty actions array is valid
    assert!(
        !errors.iter().any(|e| e.path.contains("actions")),
        "Should accept empty actions array"
    );
}

// ============================================================================
// MULTIPLE PROFILES TESTS
// ============================================================================

#[test]
fn test_all_valid_profiles() {
    for profile in &["ARW-1", "ARW-2", "ARW-3", "ARW-4"] {
        let mut manifest = create_minimal_valid_manifest();
        manifest["profile"] = json!(profile);

        let errors = validate_manifest(&manifest).unwrap();
        assert!(
            !errors.iter().any(|e| e.path == "profile"),
            "Should accept profile: {}",
            profile
        );
    }
}

// ============================================================================
// VALIDATION ERROR MESSAGE TESTS
// ============================================================================

#[test]
fn test_validation_error_display() {
    let error = ValidationError {
        path: "site.homepage".to_string(),
        message: "Invalid URL format".to_string(),
    };

    let error_string = format!("{}", error);
    assert!(error_string.contains("site.homepage"));
    assert!(error_string.contains("Invalid URL format"));
}

#[test]
fn test_multiple_validation_errors() {
    let manifest = json!({
        "version": "",
        "profile": "INVALID"
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(errors.len() >= 3, "Should have multiple validation errors");
}
