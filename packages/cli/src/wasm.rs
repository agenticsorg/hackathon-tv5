// WASM-specific exports and utilities
// This module provides JavaScript-accessible functions when compiled to WASM

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use serde_wasm_bindgen::{from_value, to_value};

use serde::Serialize;
use serde_json::Value;

// Re-export for WASM use
use crate::{ArwConfig, ValidationErrorData, ValidationResult};

/// Initialize panic hook for better error messages in WASM
#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn wasm_init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// WASM-exported function to validate an ARW manifest (YAML format)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn validate_manifest_wasm(manifest_content: String) -> Result<JsValue, JsValue> {
    // Parse YAML to JSON
    let manifest: Value = serde_yaml::from_str(&manifest_content)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse YAML: {}", e)))?;

    // Validate using the standard validator
    let errors = crate::validators::llms_txt::validate_manifest(&manifest)
        .map_err(|e| JsValue::from_str(&format!("Validation error: {}", e)))?;

    let result = ValidationResult {
        valid: errors.is_empty(),
        errors: errors
            .into_iter()
            .map(|e| ValidationErrorData {
                path: e.path,
                message: e.message,
            })
            .collect(),
    };

    to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// WASM-exported function to validate an ARW manifest (JSON format)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn validate_manifest_json_wasm(manifest_json: String) -> Result<JsValue, JsValue> {
    // Parse JSON
    let manifest: Value = serde_json::from_str(&manifest_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {}", e)))?;

    // Validate using the standard validator
    let errors = crate::validators::llms_txt::validate_manifest(&manifest)
        .map_err(|e| JsValue::from_str(&format!("Validation error: {}", e)))?;

    let result = ValidationResult {
        valid: errors.is_empty(),
        errors: errors
            .into_iter()
            .map(|e| ValidationErrorData {
                path: e.path,
                message: e.message,
            })
            .collect(),
    };

    to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// WASM-exported function to generate an llms.txt file (alias: generate_manifest_wasm)
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = generate_manifest_wasm)]
pub fn generate_llms_txt_wasm(config: JsValue) -> Result<String, JsValue> {
    let config: ArwConfig = from_value(config)
        .map_err(|e| JsValue::from_str(&format!("Invalid config: {}", e)))?;

    // Generate llms.txt content using the shared function from lib
    let content = crate::generate_llms_txt_content(&config)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(content)
}

/// WASM-exported function to check compatibility with a specific ARW profile
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn check_compatibility_wasm(manifest_content: String, profile: String) -> Result<JsValue, JsValue> {
    // Parse YAML
    let manifest: Value = serde_yaml::from_str(&manifest_content)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse YAML: {}", e)))?;

    // Check if manifest declares the requested profile
    let manifest_profile = manifest
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("ARW-1");

    let compatible = manifest_profile == profile || profile == "ARW-1";

    #[derive(Serialize)]
    struct CompatibilityResult {
        compatible: bool,
        manifest_profile: String,
        requested_profile: String,
        message: String,
    }

    let result = CompatibilityResult {
        compatible,
        manifest_profile: manifest_profile.to_string(),
        requested_profile: profile.clone(),
        message: if compatible {
            format!("Manifest is compatible with {}", profile)
        } else {
            format!(
                "Manifest declares {} but {} was requested",
                manifest_profile, profile
            )
        },
    };

    to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// WASM-exported function to get ARW version information
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_version_info() -> JsValue {
    #[derive(Serialize)]
    struct VersionInfo {
        cli_version: String,
        spec_version: String,
        supported_profiles: Vec<String>,
    }

    let info = VersionInfo {
        cli_version: env!("CARGO_PKG_VERSION").to_string(),
        spec_version: "0.2.0".to_string(),
        supported_profiles: vec![
            "ARW-1".to_string(),
            "ARW-2".to_string(),
            "ARW-3".to_string(),
        ],
    };

    to_value(&info).unwrap_or(JsValue::NULL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_llms_txt_content() {
        let config = ArwConfig {
            site_name: "Test Site".to_string(),
            homepage: "https://example.com".to_string(),
            contact: "ai@example.com".to_string(),
            profile: "ARW-1".to_string(),
            description: Some("A test site".to_string()),
        };

        let result = crate::generate_llms_txt_content(&config);
        assert!(result.is_ok());

        let content = result.unwrap();
        assert!(content.contains("version: 1.0"));
        assert!(content.contains("profile: ARW-1"));
        assert!(content.contains("name: 'Test Site'"));
        assert!(content.contains("description: 'A test site'"));
    }
}
