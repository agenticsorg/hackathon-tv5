// WASM bindings tests
// These tests verify that WASM exports are correctly defined and callable

#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use wasm_bindgen_test::*;
use arw_lib::wasm::*;
use wasm_bindgen::JsValue;
use serde_json::json;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_wasm_init() {
    // Test that initialization works without panic
    wasm_init();
}

#[wasm_bindgen_test]
fn test_get_version_info() {
    let version_info = get_version_info();
    assert!(!version_info.is_null());
    assert!(!version_info.is_undefined());
}

#[wasm_bindgen_test]
async fn test_validate_manifest_minimal() {
    let manifest = r#"
version: 1.0
profile: ARW-1
site:
  name: Test Site
  homepage: https://example.com
  contact: ai@example.com
policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#;

    let result = validate_manifest_wasm(manifest.to_string()).await;
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
async fn test_validate_manifest_invalid() {
    let manifest = r#"
version: 1.0
profile: INVALID_PROFILE
"#;

    let result = validate_manifest_wasm(manifest.to_string()).await;
    // Should succeed (return a result object) but contain validation errors
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_generate_manifest() {
    let config = json!({
        "site_name": "Test Site",
        "homepage": "https://example.com",
        "contact": "ai@example.com",
        "profile": "ARW-1",
        "description": "A test site"
    });

    let config_js = serde_wasm_bindgen::to_value(&config).unwrap();
    let result = generate_manifest_wasm(config_js);

    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(content.contains("version: 1.0"));
    assert!(content.contains("profile: ARW-1"));
}

#[wasm_bindgen_test]
fn test_check_compatibility() {
    let manifest = r#"
version: 1.0
profile: ARW-1
site:
  name: Test
  homepage: https://example.com
  contact: ai@example.com
"#;

    let result = check_compatibility_wasm(manifest.to_string(), "ARW-1".to_string());
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_error_handling_invalid_yaml() {
    let invalid_yaml = "this is not valid: yaml: content: [";
    let result = validate_manifest_wasm(invalid_yaml.to_string());

    // Should return an error
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_error_handling_invalid_config() {
    let invalid_config = JsValue::from_str("not a valid config");
    let result = generate_manifest_wasm(invalid_config);

    // Should return an error
    assert!(result.is_err());
}
