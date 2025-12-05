/// Additional comprehensive tests for llms_txt validator to achieve 100% coverage
use arw_lib::validators::llms_txt::{validate_manifest, ValidationError};
use serde_json::json;

// ============================================================================
// VALIDATION ERROR DISPLAY TESTS
// ============================================================================

#[test]
fn test_validation_error_display() {
    let error = ValidationError {
        path: "test.field".to_string(),
        message: "test error message".to_string(),
    };

    let display = format!("{}", error);
    assert_eq!(display, "test.field: test error message");
}

#[test]
fn test_validation_error_clone() {
    let error1 = ValidationError {
        path: "path".to_string(),
        message: "message".to_string(),
    };

    let error2 = error1.clone();
    assert_eq!(error1.path, error2.path);
    assert_eq!(error1.message, error2.message);
}

// ============================================================================
// VERSION VALIDATION TESTS
// ============================================================================

#[test]
fn test_version_as_number() {
    let manifest = json!({
        "version": 1.0,
        "profile": "ARW-1",
        "site": {
            "name": "Test",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "version"),
        "Should accept numeric version"
    );
}

#[test]
fn test_version_as_integer() {
    let manifest = json!({
        "version": 1,
        "profile": "ARW-1",
        "site": {
            "name": "Test",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "version"),
        "Should accept integer version"
    );
}

#[test]
fn test_version_empty_string() {
    let manifest = json!({
        "version": "",
        "profile": "ARW-1",
        "site": {
            "name": "Test",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        errors.iter().any(|e| e.path == "version" && e.message.contains("non-empty")),
        "Should reject empty version string"
    );
}

// ============================================================================
// PROFILE VALIDATION TESTS
// ============================================================================

#[test]
fn test_all_valid_profiles() {
    for profile in &["ARW-1", "ARW-2", "ARW-3", "ARW-4"] {
        let manifest = json!({
            "version": "1.0",
            "profile": profile,
            "site": {
                "name": "Test",
                "homepage": "https://example.com"
            },
            "policies": {
                "training": {"allowed": false},
                "inference": {"allowed": true},
                "attribution": {"required": true}
            }
        });

        let result = validate_manifest(&manifest);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(
            !errors.iter().any(|e| e.path == "profile"),
            "Profile {} should be valid",
            profile
        );
    }
}

#[test]
fn test_profile_case_sensitive() {
    let manifest = json!({
        "version": "1.0",
        "profile": "arw-1",
        "site": {
            "name": "Test",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        errors.iter().any(|e| e.path == "profile" && e.message.contains("arw-1")),
        "Should reject lowercase profile"
    );
}

// ============================================================================
// SITE VALIDATION TESTS
// ============================================================================

#[test]
fn test_site_missing_name() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        errors.iter().any(|e| e.path == "site.name"),
        "Should detect missing site.name"
    );
}

#[test]
fn test_site_empty_name() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        errors.iter().any(|e| e.path == "site.name"),
        "Should detect empty site.name"
    );
}

#[test]
fn test_site_missing_homepage() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        errors.iter().any(|e| e.path == "site.homepage"),
        "Should detect missing site.homepage"
    );
}

#[test]
fn test_site_http_homepage_valid() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "http://example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "site.homepage" && e.message.contains("valid URL")),
        "Should accept http:// URLs"
    );
}

#[test]
fn test_site_contact_valid_email() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com",
            "contact": "test@example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "site.contact"),
        "Should accept valid email"
    );
}

#[test]
fn test_site_contact_complex_email() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com",
            "contact": "user+tag@subdomain.example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        !errors.iter().any(|e| e.path == "site.contact"),
        "Should accept complex valid email"
    );
}

// ============================================================================
// POLICIES VALIDATION TESTS
// ============================================================================

#[test]
fn test_policies_training_missing_allowed() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        errors.iter().any(|e| e.path == "policies.training.allowed"),
        "Should detect missing training.allowed"
    );
}

#[test]
fn test_policies_inference_missing_allowed() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        errors.iter().any(|e| e.path == "policies.inference.allowed"),
        "Should detect missing inference.allowed"
    );
}

#[test]
fn test_policies_attribution_missing_required() {
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
            "attribution": {}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        errors.iter().any(|e| e.path == "policies.attribution.required"),
        "Should detect missing attribution.required"
    );
}

#[test]
fn test_policies_missing_training() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "policies": {
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        errors.iter().any(|e| e.path == "policies.training"),
        "Should detect missing training policy"
    );
}

// ============================================================================
// CONTENT VALIDATION TESTS
// ============================================================================

#[test]
fn test_content_all_priority_values() {
    for priority in &["high", "medium", "low"] {
        let manifest = json!({
            "version": "1.0",
            "profile": "ARW-2",
            "site": {
                "name": "Test Site",
                "homepage": "https://example.com"
            },
            "content": [{
                "url": "/page",
                "machine_view": "/page.llm.md",
                "priority": priority
            }],
            "policies": {
                "training": {"allowed": false},
                "inference": {"allowed": true},
                "attribution": {"required": true}
            }
        });

        let result = validate_manifest(&manifest);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(
            !errors.iter().any(|e| e.path.contains("priority")),
            "Priority {} should be valid",
            priority
        );
    }
}

#[test]
fn test_content_chunk_missing_id() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [{
            "url": "/page",
            "machine_view": "/page.llm.md",
            "chunks": [{
                "heading": "Test"
            }]
        }],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        errors.iter().any(|e| e.path.contains("chunks") && e.path.contains("id")),
        "Should detect missing chunk.id"
    );
}

#[test]
fn test_content_multiple_chunks() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [{
            "url": "/page",
            "machine_view": "/page.llm.md",
            "chunks": [
                {"id": "chunk1", "heading": "First"},
                {"id": "chunk2", "heading": "Second"},
                {"id": "chunk3", "heading": "Third"}
            ]
        }],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        !errors.iter().any(|e| e.path.contains("chunks")),
        "Should validate multiple chunks"
    );
}

// ============================================================================
// ACTIONS VALIDATION TESTS
// ============================================================================

#[test]
fn test_actions_all_http_methods() {
    for method in &["GET", "POST", "PUT", "PATCH", "DELETE"] {
        let manifest = json!({
            "version": "1.0",
            "profile": "ARW-3",
            "site": {
                "name": "Test Site",
                "homepage": "https://example.com"
            },
            "actions": [{
                "id": "test",
                "name": "Test Action",
                "endpoint": "/api/test",
                "method": method,
                "auth": "none"
            }],
            "policies": {
                "training": {"allowed": false},
                "inference": {"allowed": true},
                "attribution": {"required": true}
            }
        });

        let result = validate_manifest(&manifest);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(
            !errors.iter().any(|e| e.path.contains("method")),
            "Method {} should be valid",
            method
        );
    }
}

#[test]
fn test_actions_all_auth_types() {
    for auth in &["oauth2", "api_key", "none"] {
        let manifest = json!({
            "version": "1.0",
            "profile": "ARW-3",
            "site": {
                "name": "Test Site",
                "homepage": "https://example.com"
            },
            "actions": [{
                "id": "test",
                "name": "Test Action",
                "endpoint": "/api/test",
                "method": "GET",
                "auth": auth
            }],
            "policies": {
                "training": {"allowed": false},
                "inference": {"allowed": true},
                "attribution": {"required": true}
            }
        });

        let result = validate_manifest(&manifest);
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(
            !errors.iter().any(|e| e.path.contains("auth")),
            "Auth {} should be valid",
            auth
        );
    }
}

#[test]
fn test_actions_missing_id() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-3",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "actions": [{
            "name": "Test Action",
            "endpoint": "/api/test",
            "method": "GET",
            "auth": "none"
        }],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        errors.iter().any(|e| e.path.contains("actions") && e.message.contains("id")),
        "Should detect missing action.id"
    );
}

#[test]
fn test_actions_missing_name() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-3",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "actions": [{
            "id": "test",
            "endpoint": "/api/test",
            "method": "GET",
            "auth": "none"
        }],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        errors.iter().any(|e| e.path.contains("actions") && e.message.contains("name")),
        "Should detect missing action.name"
    );
}

#[test]
fn test_actions_missing_endpoint() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-3",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "actions": [{
            "id": "test",
            "name": "Test Action",
            "method": "GET",
            "auth": "none"
        }],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        errors.iter().any(|e| e.path.contains("actions") && e.message.contains("endpoint")),
        "Should detect missing action.endpoint"
    );
}

#[test]
fn test_actions_method_case_sensitive() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-3",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "actions": [{
            "id": "test",
            "name": "Test Action",
            "endpoint": "/api/test",
            "method": "get",
            "auth": "none"
        }],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        errors.iter().any(|e| e.path.contains("method") && e.message.contains("get")),
        "Should reject lowercase method"
    );
}

// ============================================================================
// EDGE CASES AND BOUNDARY TESTS
// ============================================================================

#[test]
fn test_manifest_with_null_values() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com",
            "contact": null
        },
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok(), "Should handle null values");
}

#[test]
fn test_manifest_with_extra_fields() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-1",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com",
            "extra_field": "ignored"
        },
        "extra_top_level": "also ignored",
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok(), "Should handle extra fields");
}

#[test]
fn test_empty_content_array() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-2",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "content": [],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        !errors.iter().any(|e| e.path.contains("content")),
        "Should accept empty content array"
    );
}

#[test]
fn test_empty_actions_array() {
    let manifest = json!({
        "version": "1.0",
        "profile": "ARW-3",
        "site": {
            "name": "Test Site",
            "homepage": "https://example.com"
        },
        "actions": [],
        "policies": {
            "training": {"allowed": false},
            "inference": {"allowed": true},
            "attribution": {"required": true}
        }
    });

    let result = validate_manifest(&manifest);
    assert!(result.is_ok());
    let errors = result.unwrap();
    assert!(
        !errors.iter().any(|e| e.path.contains("actions")),
        "Should accept empty actions array"
    );
}
