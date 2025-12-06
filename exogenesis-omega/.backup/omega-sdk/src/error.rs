//! Error types for Omega SDK
//!
//! Provides comprehensive error handling with C FFI compatibility

use std::fmt;

/// Main error type for Omega SDK operations
#[derive(Debug)]
pub enum OmegaError {
    /// Initialization failed
    InitError(String),

    /// Inference/recommendation failed
    InferenceError(String),

    /// Sync with constellation failed
    SyncError(String),

    /// Storage/persistence failed
    StorageError(String),

    /// Invalid input parameter
    InvalidInput(String),

    /// Internal error
    Internal(String),
}

impl fmt::Display for OmegaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OmegaError::InitError(msg) => write!(f, "Initialization error: {}", msg),
            OmegaError::InferenceError(msg) => write!(f, "Inference error: {}", msg),
            OmegaError::SyncError(msg) => write!(f, "Sync error: {}", msg),
            OmegaError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            OmegaError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            OmegaError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for OmegaError {}

/// C FFI error codes
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmegaErrorCode {
    /// Success
    Success = 0,

    /// Initialization error
    InitError = -1,

    /// Inference error
    InferenceError = -2,

    /// Sync error
    SyncError = -3,

    /// Storage error
    StorageError = -4,

    /// Invalid input
    InvalidInput = -5,

    /// Internal error
    Internal = -99,
}

impl From<&OmegaError> for OmegaErrorCode {
    fn from(error: &OmegaError) -> Self {
        match error {
            OmegaError::InitError(_) => OmegaErrorCode::InitError,
            OmegaError::InferenceError(_) => OmegaErrorCode::InferenceError,
            OmegaError::SyncError(_) => OmegaErrorCode::SyncError,
            OmegaError::StorageError(_) => OmegaErrorCode::StorageError,
            OmegaError::InvalidInput(_) => OmegaErrorCode::InvalidInput,
            OmegaError::Internal(_) => OmegaErrorCode::Internal,
        }
    }
}

impl From<OmegaError> for i32 {
    fn from(error: OmegaError) -> Self {
        OmegaErrorCode::from(&error) as i32
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, OmegaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_mapping() {
        let error = OmegaError::InitError("test".to_string());
        let code: OmegaErrorCode = (&error).into();
        assert_eq!(code, OmegaErrorCode::InitError);

        let i32_code: i32 = error.into();
        assert_eq!(i32_code, -1);
    }

    #[test]
    fn test_error_display() {
        let error = OmegaError::InferenceError("model not loaded".to_string());
        assert_eq!(
            format!("{}", error),
            "Inference error: model not loaded"
        );
    }
}
