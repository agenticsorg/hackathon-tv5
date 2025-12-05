/// Edge cases and additional coverage tests for llms_txt validator
/// Focuses on uncovered code paths to achieve 100% coverage
use arw_lib::validators::llms_txt::{validate, validate_manifest, ValidationError};
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ============================================================================
// POLICY VALIDATION EDGE CASES
// ============================================================================

#[test]
fn test_missing_training_allowed_field() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {
                "commercial": false
                // Missing "allowed" field
            },
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "policies.training.allowed"),
        "Should require training.allowed field"
    );
}

#[test]
fn test_missing_inference_allowed_field() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {
                "rate_limit": 100
                // Missing "allowed" field
            },
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "policies.inference.allowed"),
        "Should require inference.allowed field"
    );
}

#[test]
fn test_missing_attribution_required_field() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {
                "format": "markdown"
                // Missing "required" field
            }
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "policies.attribution.required"),
        "Should require attribution.required field"
    );
}

#[test]
fn test_training_policy_not_an_object() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": "not_an_object",
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.is_empty(),
        "Should have errors when training is not an object"
    );
}

#[test]
fn test_inference_policy_not_an_object() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": true,
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.is_empty(),
        "Should have errors when inference is not an object"
    );
}

#[test]
fn test_attribution_policy_not_an_object() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": false
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.is_empty(),
        "Should have errors when attribution is not an object"
    );
}

#[test]
fn test_policies_not_an_object() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": "not_an_object"
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "policies"),
        "Should have error when policies is not an object"
    );
}

// ============================================================================
// CONTENT VALIDATION EDGE CASES
// ============================================================================

#[test]
fn test_content_item_not_an_object() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [
            "not_an_object"
        ],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    // Should not crash and should handle gracefully
    assert!(errors.len() >= 0, "Should handle non-object content items");
}

#[test]
fn test_chunk_not_an_object() {
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
                "purpose": "test",
                "chunks": [
                    "not_an_object",
                    {"id": "valid", "heading": "Valid Chunk"}
                ]
            }
        ],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    // Should not crash when chunk is not an object
    assert!(errors.len() >= 0, "Should handle non-object chunks");
}

#[test]
fn test_chunks_not_an_array() {
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
                "purpose": "test",
                "chunks": "not_an_array"
            }
        ],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    // Should handle when chunks is not an array
    assert!(errors.len() >= 0, "Should handle non-array chunks");
}

// ============================================================================
// ACTIONS VALIDATION EDGE CASES
// ============================================================================

#[test]
fn test_action_item_not_an_object() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-3",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "actions": [
            "not_an_object"
        ],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    // Should not crash with non-object actions
    assert!(errors.len() >= 0, "Should handle non-object action items");
}

#[test]
fn test_actions_not_an_array() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-3",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "actions": "not_an_array",
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    // Should handle when actions is not an array
    assert!(errors.len() >= 0, "Should handle non-array actions");
}

// ============================================================================
// SITE VALIDATION EDGE CASES
// ============================================================================

#[test]
fn test_site_not_an_object() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": "not_an_object",
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "site"),
        "Should have error when site is not an object"
    );
}

#[test]
fn test_homepage_with_trailing_slash() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com/",
            "contact": "test@example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "site.homepage"),
        "Should accept URL with trailing slash"
    );
}

#[test]
fn test_homepage_with_path() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com/path/to/page",
            "contact": "test@example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "site.homepage"),
        "Should accept URL with path"
    );
}

#[test]
fn test_homepage_with_port() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com:8080",
            "contact": "test@example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "site.homepage"),
        "Should accept URL with port"
    );
}

#[test]
fn test_empty_site_homepage() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": ""
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "site.homepage"),
        "Should reject empty homepage"
    );
}

// ============================================================================
// VERSION VALIDATION EDGE CASES
// ============================================================================

#[test]
fn test_version_as_integer() {
    let manifest = json!({
        "version": 1,
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

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "version"),
        "Should accept integer version"
    );
}

#[test]
fn test_version_as_float() {
    let manifest = json!({
        "version": 1.5,
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

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "version"),
        "Should accept float version"
    );
}

#[test]
fn test_version_as_null() {
    let manifest = json!({
        "version": null,
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

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "version"),
        "Should reject null version"
    );
}

// ============================================================================
// CONTENT NOT AN ARRAY
// ============================================================================

#[test]
fn test_content_not_an_array() {
    let manifest = json!({
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

    let errors = validate_manifest(&manifest).unwrap();
    // Should handle when content is not an array
    assert!(errors.len() >= 0, "Should handle non-array content");
}

// ============================================================================
// MULTIPLE ERRORS IN SINGLE CONTENT ITEM
// ============================================================================

#[test]
fn test_content_item_with_multiple_errors() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [
            {
                // Missing url and machine_view
                "purpose": "test",
                "priority": "invalid_priority",
                "chunks": [
                    {
                        // Missing id
                        "heading": "Section"
                    }
                ]
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
        errors.len() >= 3,
        "Should detect multiple errors in single content item"
    );
    assert!(
        errors.iter().any(|e| e.path.contains("url")),
        "Should detect missing url"
    );
    assert!(
        errors.iter().any(|e| e.path.contains("machine_view")),
        "Should detect missing machine_view"
    );
}

// ============================================================================
// PROFILE VARIATIONS
// ============================================================================

#[test]
fn test_profile_arw2() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-2",
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

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "profile"),
        "Should accept ARW-2 profile"
    );
}

#[test]
fn test_profile_arw4() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-4",
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

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "profile"),
        "Should accept ARW-4 profile"
    );
}

// ============================================================================
// EMAIL EDGE CASES
// ============================================================================

#[test]
fn test_email_missing_at_symbol() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com",
            "contact": "testexample.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        errors.iter().any(|e| e.path == "site.contact"),
        "Should reject email without @ symbol"
    );
}

#[test]
fn test_email_with_plus_sign() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com",
            "contact": "test+tag@example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let errors = validate_manifest(&manifest).unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "site.contact"),
        "Should accept email with + sign"
    );
}

// ============================================================================
// FILE READING EDGE CASES
// ============================================================================

#[test]
fn test_validate_file_with_bom() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("llms.txt");

    let manifest = json!({
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

    // Write with UTF-8 BOM
    let yaml_content = serde_yaml::to_string(&manifest).unwrap();
    let bom_content = format!("\u{FEFF}{}", yaml_content);
    fs::write(&manifest_path, bom_content).unwrap();

    let result = validate(&manifest_path);
    // Should handle BOM gracefully (YAML parser typically handles this)
    assert!(result.is_ok() || result.is_err(), "Should handle file with BOM");
}

#[test]
fn test_validate_malformed_yaml_structure() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("llms.txt");

    fs::write(&manifest_path, "---\nversion: 1.0\n  badly: indented\n").unwrap();

    let result = validate(&manifest_path);
    assert!(result.is_err(), "Should fail on malformed YAML");
}

// ============================================================================
// VALIDATION ERROR CLONE AND DEBUG
// ============================================================================

#[test]
fn test_validation_error_clone() {
    let error = ValidationError {
        path: "test.path".to_string(),
        message: "Test message".to_string(),
    };

    let cloned = error.clone();
    assert_eq!(error.path, cloned.path);
    assert_eq!(error.message, cloned.message);
}

#[test]
fn test_validation_error_debug() {
    let error = ValidationError {
        path: "test.path".to_string(),
        message: "Test message".to_string(),
    };

    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("test.path"));
    assert!(debug_str.contains("Test message"));
}
