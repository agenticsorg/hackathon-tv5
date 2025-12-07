//! Plugin error types

use thiserror::Error;

/// Plugin error types
#[derive(Error, Debug)]
pub enum PluginError {
    /// Plugin not found
    #[error("Plugin not found: {0}")]
    NotFound(String),

    /// Plugin already loaded
    #[error("Plugin already loaded: {0}")]
    AlreadyLoaded(String),

    /// Invalid manifest
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    /// Load error
    #[error("Failed to load plugin: {0}")]
    LoadError(String),

    /// Runtime error
    #[error("Plugin runtime error: {0}")]
    RuntimeError(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Version mismatch
    #[error("Version mismatch: required {required}, found {found}")]
    VersionMismatch { required: String, found: String },

    /// Dependency error
    #[error("Dependency error: {0}")]
    DependencyError(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;
