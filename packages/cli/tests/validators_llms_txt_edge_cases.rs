/// Edge cases and additional coverage tests for llms_txt validator
/// Integration test file for uncovered code paths
use arw_cli::validators::llms_txt::{validate, validate_manifest, ValidationError};
use serde_json::json;

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

#[test]
fn test_validation_error_clone_and_display() {
    let error = ValidationError {
        path: "test.path".to_string(),
        message: "Test message".to_string(),
    };

    let cloned = error.clone();
    assert_eq!(error.path, cloned.path);
    assert_eq!(error.message, cloned.message);

    let display_str = format!("{}", error);
    assert!(display_str.contains("test.path"));
    assert!(display_str.contains("Test message"));
}
