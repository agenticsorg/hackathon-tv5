// NAPI-RS validation functions
// Provides high-performance manifest validation for Node.js

#[cfg(feature = "napi")]
use napi::bindgen_prelude::*;
#[cfg(feature = "napi")]
use napi_derive::napi;

/// Validation result returned to JavaScript
#[cfg(feature = "napi")]
#[napi(object)]
pub struct ValidationResult {
    /// Whether the manifest is valid
    pub valid: bool,
    /// List of validation errors (empty if valid)
    pub errors: Vec<ValidationError>,
}

/// Individual validation error
#[cfg(feature = "napi")]
#[napi(object)]
pub struct ValidationError {
    /// JSON path to the error location (e.g., "site.name")
    pub path: String,
    /// Human-readable error message
    pub message: String,
    /// Error severity level
    pub severity: String,
}

/// Validate an ARW manifest
///
/// # Arguments
/// * `content` - YAML or JSON string containing the manifest
///
/// # Returns
/// ValidationResult with validation status and any errors
#[cfg(feature = "napi")]
#[napi]
pub fn validate_manifest(content: String) -> Result<ValidationResult> {
    // Parse YAML to JSON
    let manifest: serde_json::Value = serde_yaml::from_str(&content)
        .map_err(|e| Error::new(
            Status::InvalidArg,
            format!("Failed to parse YAML: {}", e)
        ))?;

    // Validate using existing validation logic
    let validation_errors = crate::validators::llms_txt::validate_manifest(&manifest)
        .map_err(|e| Error::new(
            Status::GenericFailure,
            format!("Validation error: {}", e)
        ))?;

    // Convert to NAPI types
    let errors: Vec<ValidationError> = validation_errors
        .into_iter()
        .map(|e| ValidationError {
            path: e.path,
            message: e.message,
            severity: "error".to_string(),
        })
        .collect();

    Ok(ValidationResult {
        valid: errors.is_empty(),
        errors,
    })
}

/// Check compatibility with a specific ARW profile
#[cfg(feature = "napi")]
#[napi(object)]
pub struct CompatibilityResult {
    /// Whether the manifest is compatible with the requested profile
    pub compatible: bool,
    /// The profile declared in the manifest
    pub manifest_profile: String,
    /// The profile that was requested for compatibility check
    pub requested_profile: String,
    /// Human-readable message about compatibility
    pub message: String,
}

#[cfg(feature = "napi")]
#[napi]
pub fn check_compatibility(content: String, profile: String) -> Result<CompatibilityResult> {
    // Parse YAML
    let manifest: serde_json::Value = serde_yaml::from_str(&content)
        .map_err(|e| Error::new(
            Status::InvalidArg,
            format!("Failed to parse YAML: {}", e)
        ))?;

    // Check if manifest declares the requested profile
    let manifest_profile = manifest
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("ARW-1");

    let compatible = manifest_profile == profile || profile == "ARW-1";

    Ok(CompatibilityResult {
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
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_validators_compile() {
        // Basic compilation test
        assert!(true);
    }
}
