//! Error types and C error codes for the SDK

use std::ffi::NulError;
use std::os::raw::c_int;
use thiserror::Error;

/// C error codes
pub const OMEGA_SUCCESS: c_int = 0;
pub const OMEGA_ERR_INIT: c_int = -1;
pub const OMEGA_ERR_RECOMMEND: c_int = -2;
pub const OMEGA_ERR_OBSERVE: c_int = -3;
pub const OMEGA_ERR_SYNC: c_int = -4;
pub const OMEGA_ERR_INVALID_ARG: c_int = -5;
pub const OMEGA_ERR_JSON_PARSE: c_int = -6;
pub const OMEGA_ERR_BUFFER_TOO_SMALL: c_int = -7;
pub const OMEGA_ERR_NOT_INITIALIZED: c_int = -8;
pub const OMEGA_ERR_ALREADY_INITIALIZED: c_int = -9;
pub const OMEGA_ERR_INTERNAL: c_int = -10;

/// SDK errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("Initialization error: {0}")]
    Init(String),

    #[error("Recommendation error: {0}")]
    Recommend(String),

    #[error("Observation error: {0}")]
    Observe(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("Invalid argument: {0}")]
    InvalidArg(String),

    #[error("JSON parse error: {0}")]
    JsonParse(String),

    #[error("Buffer too small: need {0} bytes")]
    BufferTooSmall(usize),

    #[error("SDK not initialized")]
    NotInitialized,

    #[error("SDK already initialized")]
    AlreadyInitialized,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Null error: {0}")]
    Null(#[from] NulError),
}

/// Result type for SDK operations
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Convert error to C error code
    pub fn to_c_code(&self) -> c_int {
        match self {
            Error::Init(_) => OMEGA_ERR_INIT,
            Error::Recommend(_) => OMEGA_ERR_RECOMMEND,
            Error::Observe(_) => OMEGA_ERR_OBSERVE,
            Error::Sync(_) => OMEGA_ERR_SYNC,
            Error::InvalidArg(_) => OMEGA_ERR_INVALID_ARG,
            Error::JsonParse(_) => OMEGA_ERR_JSON_PARSE,
            Error::BufferTooSmall(_) => OMEGA_ERR_BUFFER_TOO_SMALL,
            Error::NotInitialized => OMEGA_ERR_NOT_INITIALIZED,
            Error::AlreadyInitialized => OMEGA_ERR_ALREADY_INITIALIZED,
            Error::Internal(_) | Error::Null(_) => OMEGA_ERR_INTERNAL,
        }
    }

    /// Get error message
    pub fn message(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(Error::Init("test".into()).to_c_code(), OMEGA_ERR_INIT);
        assert_eq!(Error::Recommend("test".into()).to_c_code(), OMEGA_ERR_RECOMMEND);
        assert_eq!(Error::Observe("test".into()).to_c_code(), OMEGA_ERR_OBSERVE);
        assert_eq!(Error::Sync("test".into()).to_c_code(), OMEGA_ERR_SYNC);
        assert_eq!(Error::InvalidArg("test".into()).to_c_code(), OMEGA_ERR_INVALID_ARG);
    }

    #[test]
    fn test_error_messages() {
        let err = Error::Init("initialization failed".into());
        assert!(err.message().contains("initialization failed"));
    }
}
