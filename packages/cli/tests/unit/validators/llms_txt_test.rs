use std::path::Path;

#[cfg(test)]
mod llms_txt_validator_tests {
    use super::*;

    #[test]
    fn test_validate_minimal_valid_manifest() {
        let fixture_path = Path::new("tests/fixtures/valid/minimal.llms.txt");
        assert!(fixture_path.exists(), "Fixture file should exist");

        let result = arw_lib::validators::llms_txt::validate(fixture_path);
        assert!(result.is_ok(), "Should successfully validate file");

        let errors = result.unwrap();
        assert_eq!(
            errors.len(),
            0,
            "Minimal valid manifest should have no errors"
        );
    }

    #[test]
    fn test_validate_complete_valid_manifest() {
        let fixture_path = Path::new("tests/fixtures/valid/complete.llms.txt");
        assert!(fixture_path.exists(), "Fixture file should exist");

        let result = arw_lib::validators::llms_txt::validate(fixture_path);
        assert!(result.is_ok(), "Should successfully validate file");

        let errors = result.unwrap();
        assert_eq!(
            errors.len(),
            0,
            "Complete valid manifest should have no errors"
        );
    }

    #[test]
    fn test_validate_missing_version() {
        let fixture_path = Path::new("tests/fixtures/invalid/missing-version.llms.txt");
        assert!(fixture_path.exists(), "Fixture file should exist");

        let result = arw_lib::validators::llms_txt::validate(fixture_path);
        assert!(result.is_ok(), "Should successfully parse file");

        let errors = result.unwrap();
        assert!(
            !errors.is_empty(),
            "Should have validation errors for missing version"
        );
        assert!(
            errors.iter().any(|e| e.path == "version"),
            "Should have error for version field"
        );
    }

    #[test]
    fn test_validate_invalid_profile() {
        let fixture_path = Path::new("tests/fixtures/invalid/invalid-profile.llms.txt");
        assert!(fixture_path.exists(), "Fixture file should exist");

        let result = arw_lib::validators::llms_txt::validate(fixture_path);
        assert!(result.is_ok(), "Should successfully parse file");

        let errors = result.unwrap();
        assert!(
            !errors.is_empty(),
            "Should have validation errors for invalid profile"
        );
        assert!(
            errors.iter().any(|e| e.path == "profile"),
            "Should have error for profile field"
        );
    }

    #[test]
    fn test_validate_missing_site() {
        let fixture_path = Path::new("tests/fixtures/invalid/missing-site.llms.txt");
        assert!(fixture_path.exists(), "Fixture file should exist");

        let result = arw_lib::validators::llms_txt::validate(fixture_path);
        assert!(result.is_ok(), "Should successfully parse file");

        let errors = result.unwrap();
        assert!(
            !errors.is_empty(),
            "Should have validation errors for missing site"
        );
        assert!(
            errors.iter().any(|e| e.path.starts_with("site")),
            "Should have error for site field"
        );
    }

    #[test]
    fn test_validate_url_format() {
        use serde_json::json;

        let manifest = json!({
            "version": "1.0",
            "profile": "ARW-1",
            "site": {
                "name": "Test Site",
                "homepage": "not-a-url",  // Invalid URL
                "contact": "ai@example.com"
            },
            "policies": {
                "training": {
                    "allowed": false
                },
                "inference": {
                    "allowed": true
                },
                "attribution": {
                    "required": true
                }
            }
        });

        let result = arw_lib::validators::llms_txt::validate_manifest(&manifest);
        assert!(result.is_ok());

        let errors = result.unwrap();
        assert!(
            errors.iter().any(|e| e.path == "site.homepage"),
            "Should have error for invalid homepage URL"
        );
    }

    #[test]
    fn test_validate_email_format() {
        use serde_json::json;

        let manifest = json!({
            "version": "1.0",
            "profile": "ARW-1",
            "site": {
                "name": "Test Site",
                "homepage": "https://example.com",
                "contact": "not-an-email"  // Invalid email
            },
            "policies": {
                "training": {
                    "allowed": false
                },
                "inference": {
                    "allowed": true
                },
                "attribution": {
                    "required": true
                }
            }
        });

        let result = arw_lib::validators::llms_txt::validate_manifest(&manifest);
        assert!(result.is_ok());

        let errors = result.unwrap();
        assert!(
            errors.iter().any(|e| e.path == "site.contact"),
            "Should have error for invalid contact email"
        );
    }

    #[test]
    fn test_validate_content_required_fields() {
        use serde_json::json;

        let manifest = json!({
            "version": "1.0",
            "profile": "ARW-2",
            "site": {
                "name": "Test Site",
                "homepage": "https://example.com",
                "contact": "ai@example.com"
            },
            "content": [
                {
                    "url": "/page1"
                    // Missing machine_view
                }
            ],
            "policies": {
                "training": {
                    "allowed": false
                },
                "inference": {
                    "allowed": true
                },
                "attribution": {
                    "required": true
                }
            }
        });

        let result = arw_lib::validators::llms_txt::validate_manifest(&manifest);
        assert!(result.is_ok());

        let errors = result.unwrap();
        assert!(
            errors.iter().any(|e| e.path.contains("machine_view")),
            "Should have error for missing machine_view"
        );
    }

    #[test]
    fn test_validate_action_required_fields() {
        use serde_json::json;

        let manifest = json!({
            "version": "1.0",
            "profile": "ARW-3",
            "site": {
                "name": "Test Site",
                "homepage": "https://example.com",
                "contact": "ai@example.com"
            },
            "actions": [
                {
                    "id": "test_action",
                    "name": "Test Action"
                    // Missing endpoint, method, auth
                }
            ],
            "policies": {
                "training": {
                    "allowed": false
                },
                "inference": {
                    "allowed": true
                },
                "attribution": {
                    "required": true
                }
            }
        });

        let result = arw_lib::validators::llms_txt::validate_manifest(&manifest);
        assert!(result.is_ok());

        let errors = result.unwrap();
        assert!(
            errors.iter().any(|e| e.path.contains("endpoint")),
            "Should have error for missing endpoint"
        );
        assert!(
            errors.iter().any(|e| e.path.contains("method")),
            "Should have error for missing method"
        );
        assert!(
            errors.iter().any(|e| e.path.contains("auth")),
            "Should have error for missing auth"
        );
    }

    #[test]
    fn test_validate_enum_values() {
        use serde_json::json;

        let manifest = json!({
            "version": "1.0",
            "profile": "ARW-3",
            "site": {
                "name": "Test Site",
                "homepage": "https://example.com",
                "contact": "ai@example.com"
            },
            "content": [
                {
                    "url": "/page1",
                    "machine_view": "/page1.llm.md",
                    "priority": "super-high"  // Invalid priority
                }
            ],
            "actions": [
                {
                    "id": "test",
                    "name": "Test",
                    "endpoint": "/api/test",
                    "method": "INVALID",  // Invalid method
                    "auth": "magic"  // Invalid auth
                }
            ],
            "policies": {
                "training": {
                    "allowed": false
                },
                "inference": {
                    "allowed": true
                },
                "attribution": {
                    "required": true
                }
            }
        });

        let result = arw_lib::validators::llms_txt::validate_manifest(&manifest);
        assert!(result.is_ok());

        let errors = result.unwrap();
        assert!(
            errors.iter().any(|e| e.path.contains("priority")),
            "Should have error for invalid priority"
        );
        assert!(
            errors.iter().any(|e| e.path.contains("method")),
            "Should have error for invalid method"
        );
        assert!(
            errors.iter().any(|e| e.path.contains("auth")),
            "Should have error for invalid auth"
        );
    }

    #[test]
    fn test_validate_chunk_structure() {
        use serde_json::json;

        let manifest = json!({
            "version": "1.0",
            "profile": "ARW-2",
            "site": {
                "name": "Test Site",
                "homepage": "https://example.com",
                "contact": "ai@example.com"
            },
            "content": [
                {
                    "url": "/page1",
                    "machine_view": "/page1.llm.md",
                    "chunks": [
                        {
                            "heading": "Section 1"
                            // Missing id
                        }
                    ]
                }
            ],
            "policies": {
                "training": {
                    "allowed": false
                },
                "inference": {
                    "allowed": true
                },
                "attribution": {
                    "required": true
                }
            }
        });

        let result = arw_lib::validators::llms_txt::validate_manifest(&manifest);
        assert!(result.is_ok());

        let errors = result.unwrap();
        assert!(
            errors.iter().any(|e| e.path.contains("chunks") && e.path.contains("id")),
            "Should have error for missing chunk id"
        );
    }
}
