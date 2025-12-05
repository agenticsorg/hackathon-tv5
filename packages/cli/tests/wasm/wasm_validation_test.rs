// WASM validation tests
// Test validation logic through WASM interface

#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use wasm_bindgen_test::*;
use arw_lib::wasm::*;
use serde_wasm_bindgen::from_value;
use serde_json::Value;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_validate_complete_manifest() {
    let manifest = r#"
version: 1.0
profile: ARW-2
site:
  name: Complete Test Site
  description: A fully featured test site
  homepage: https://example.com
  contact: ai@example.com
  logo: https://example.com/logo.png
  documentation: https://docs.example.com

content:
  - url: /docs
    title: Documentation
    format: markdown
    frequency: weekly
  - url: /api
    title: API Reference
    format: openapi
    frequency: monthly

policies:
  training:
    allowed: false
    restrictions: No training on private data
  inference:
    allowed: true
  attribution:
    required: true
    format: "Powered by Example.com"

actions:
  - id: search
    name: Search
    description: Search the site
    method: GET
    endpoint: https://api.example.com/search
    parameters:
      - name: q
        type: string
        required: true
"#;

    let result = validate_manifest_wasm(manifest.to_string()).await;
    assert!(result.is_ok());

    let result_value: Value = from_value(result.unwrap()).unwrap();
    assert_eq!(result_value["valid"], true);
}

#[wasm_bindgen_test]
async fn test_validate_missing_required_fields() {
    let manifest = r#"
version: 1.0
profile: ARW-1
"#;

    let result = validate_manifest_wasm(manifest.to_string()).await;
    assert!(result.is_ok());

    let result_value: Value = from_value(result.unwrap()).unwrap();
    assert_eq!(result_value["valid"], false);
    assert!(result_value["errors"].as_array().unwrap().len() > 0);
}

#[wasm_bindgen_test]
async fn test_validate_invalid_url() {
    let manifest = r#"
version: 1.0
profile: ARW-1
site:
  name: Test
  homepage: not-a-valid-url
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

    let result_value: Value = from_value(result.unwrap()).unwrap();
    assert_eq!(result_value["valid"], false);
}

#[wasm_bindgen_test]
async fn test_validate_invalid_email() {
    let manifest = r#"
version: 1.0
profile: ARW-1
site:
  name: Test
  homepage: https://example.com
  contact: not-an-email
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

    let result_value: Value = from_value(result.unwrap()).unwrap();
    assert_eq!(result_value["valid"], false);
}

#[wasm_bindgen_test]
async fn test_validate_json_format() {
    let manifest_json = r#"{
  "version": 1.0,
  "profile": "ARW-1",
  "site": {
    "name": "Test Site",
    "homepage": "https://example.com",
    "contact": "ai@example.com"
  },
  "policies": {
    "training": {"allowed": false},
    "inference": {"allowed": true},
    "attribution": {"required": true}
  }
}"#;

    let result = validate_manifest_json_wasm(manifest_json.to_string()).await;
    assert!(result.is_ok());

    let result_value: Value = from_value(result.unwrap()).unwrap();
    assert_eq!(result_value["valid"], true);
}

#[wasm_bindgen_test]
async fn test_validate_invalid_profile() {
    let manifest = r#"
version: 1.0
profile: NONEXISTENT-PROFILE
site:
  name: Test
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

    let result_value: Value = from_value(result.unwrap()).unwrap();
    assert_eq!(result_value["valid"], false);

    let errors = result_value["errors"].as_array().unwrap();
    assert!(errors.iter().any(|e| {
        e["path"].as_str().unwrap_or("") == "profile"
    }));
}

#[wasm_bindgen_test]
async fn test_validate_multiple_errors() {
    let manifest = r#"
version: 1.0
profile: INVALID
"#;

    let result = validate_manifest_wasm(manifest.to_string()).await;
    assert!(result.is_ok());

    let result_value: Value = from_value(result.unwrap()).unwrap();
    assert_eq!(result_value["valid"], false);

    let errors = result_value["errors"].as_array().unwrap();
    assert!(errors.len() > 1, "Should have multiple validation errors");
}
