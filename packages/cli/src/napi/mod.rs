// NAPI-RS bindings for Node.js native integration
// This module provides high-performance native Node.js bindings for ARW functionality

#![deny(clippy::all)]

#[cfg(feature = "napi")]
use napi_derive::napi;

// Sub-modules for NAPI exports
pub mod validators;
pub mod generators;

/// Initialize NAPI module
/// This is called when the module is loaded by Node.js
#[cfg(feature = "napi")]
#[napi::module_init]
fn init() {
    // Module initialization logic if needed
    // This runs once when the native module is first loaded
}

/// Get version information about ARW CLI and supported specs
#[cfg(feature = "napi")]
#[napi(object)]
pub struct VersionInfo {
    /// CLI version from Cargo.toml
    pub cli_version: String,
    /// ARW specification version
    pub spec_version: String,
    /// List of supported ARW profiles
    pub supported_profiles: Vec<String>,
}

#[cfg(feature = "napi")]
#[napi]
pub fn get_version_info() -> VersionInfo {
    VersionInfo {
        cli_version: env!("CARGO_PKG_VERSION").to_string(),
        spec_version: "0.2.0".to_string(),
        supported_profiles: vec![
            "ARW-1".to_string(),
            "ARW-2".to_string(),
            "ARW-3".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_napi_module_compiles() {
        // Basic compilation test
        assert!(true);
    }
}
