// WASM generation tests
// Test manifest generation through WASM interface

#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use wasm_bindgen_test::*;
use arw_lib::wasm::*;
use serde_json::json;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_generate_minimal_manifest() {
    let config = json!({
        "site_name": "Test Site",
        "homepage": "https://example.com",
        "contact": "ai@example.com",
        "profile": "ARW-1"
    });

    let config_js = serde_wasm_bindgen::to_value(&config).unwrap();
    let result = generate_manifest_wasm(config_js);

    assert!(result.is_ok());
    let content = result.unwrap();

    assert!(content.contains("version: 1.0"));
    assert!(content.contains("profile: ARW-1"));
    assert!(content.contains("name: 'Test Site'"));
    assert!(content.contains("homepage: 'https://example.com'"));
    assert!(content.contains("contact: 'ai@example.com'"));
}

#[wasm_bindgen_test]
fn test_generate_with_description() {
    let config = json!({
        "site_name": "Test Site",
        "homepage": "https://example.com",
        "contact": "ai@example.com",
        "profile": "ARW-2",
        "description": "A comprehensive test site for WASM testing"
    });

    let config_js = serde_wasm_bindgen::to_value(&config).unwrap();
    let result = generate_manifest_wasm(config_js);

    assert!(result.is_ok());
    let content = result.unwrap();

    assert!(content.contains("profile: ARW-2"));
    assert!(content.contains("description: 'A comprehensive test site for WASM testing'"));
}

#[wasm_bindgen_test]
fn test_generate_then_validate() {
    let config = json!({
        "site_name": "Round Trip Test",
        "homepage": "https://roundtrip.com",
        "contact": "test@roundtrip.com",
        "profile": "ARW-1"
    });

    // Generate manifest
    let config_js = serde_wasm_bindgen::to_value(&config).unwrap();
    let gen_result = generate_manifest_wasm(config_js);
    assert!(gen_result.is_ok());

    let manifest = gen_result.unwrap();

    // Validate generated manifest
    let val_result = validate_manifest_wasm(manifest);
    assert!(val_result.is_ok());
}

#[wasm_bindgen_test]
fn test_generate_with_missing_fields() {
    let config = json!({
        "site_name": "Incomplete Site",
        "homepage": "https://example.com"
        // Missing contact and profile
    });

    let config_js = serde_wasm_bindgen::to_value(&config).unwrap();
    let result = generate_manifest_wasm(config_js);

    // Should fail because required fields are missing
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_generate_all_profiles() {
    let profiles = vec!["ARW-1", "ARW-2", "ARW-3"];

    for profile in profiles {
        let config = json!({
            "site_name": format!("Test Site {}", profile),
            "homepage": "https://example.com",
            "contact": "ai@example.com",
            "profile": profile
        });

        let config_js = serde_wasm_bindgen::to_value(&config).unwrap();
        let result = generate_manifest_wasm(config_js);

        assert!(result.is_ok(), "Failed to generate for profile {}", profile);
        let content = result.unwrap();
        assert!(content.contains(&format!("profile: {}", profile)));
    }
}

#[wasm_bindgen_test]
fn test_generate_with_special_characters() {
    let config = json!({
        "site_name": "Test & Site 'with' \"quotes\"",
        "homepage": "https://example.com/path?query=value",
        "contact": "ai+test@example.com",
        "profile": "ARW-1",
        "description": "Testing special chars: & < > ' \""
    });

    let config_js = serde_wasm_bindgen::to_value(&config).unwrap();
    let result = generate_manifest_wasm(config_js);

    assert!(result.is_ok());
    let content = result.unwrap();

    // Verify content is valid YAML
    assert!(content.contains("name: 'Test & Site"));
}

#[wasm_bindgen_test]
fn test_generate_consistency() {
    let config = json!({
        "site_name": "Consistency Test",
        "homepage": "https://example.com",
        "contact": "ai@example.com",
        "profile": "ARW-1"
    });

    let config_js1 = serde_wasm_bindgen::to_value(&config).unwrap();
    let config_js2 = serde_wasm_bindgen::to_value(&config).unwrap();

    let result1 = generate_manifest_wasm(config_js1);
    let result2 = generate_manifest_wasm(config_js2);

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // Same input should produce same output
    assert_eq!(result1.unwrap(), result2.unwrap());
}
