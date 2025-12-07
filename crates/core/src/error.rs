//! Error types for the Media Gateway platform
//!
//! Comprehensive error handling using thiserror for ergonomic error definitions.

use thiserror::Error;

/// Main error type for Media Gateway operations
///
/// Provides comprehensive error variants covering all platform operations.
#[derive(Error, Debug)]
pub enum MediaGatewayError {
    /// Validation error
    #[error("Validation error: {message}")]
    ValidationError {
        /// Error message
        message: String,
        /// Field that failed validation (if applicable)
        field: Option<String>,
    },

    /// Content not found
    #[error("Content not found: {content_id}")]
    NotFoundError {
        /// Content canonical ID
        content_id: String,
    },

    /// User not found
    #[error("User not found: {user_id}")]
    UserNotFoundError {
        /// User ID
        user_id: String,
    },

    /// Authentication error
    #[error("Authentication failed: {reason}")]
    AuthenticationError {
        /// Reason for authentication failure
        reason: String,
    },

    /// Authorization error
    #[error("Authorization failed: {reason}")]
    AuthorizationError {
        /// Reason for authorization failure
        reason: String,
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {limit} requests per {window}")]
    RateLimitError {
        /// Rate limit threshold
        limit: u32,
        /// Time window (e.g., "minute", "hour")
        window: String,
        /// Time until limit resets (seconds)
        retry_after: Option<u64>,
    },

    /// Database error
    #[error("Database error: {message}")]
    DatabaseError {
        /// Error message
        message: String,
        /// Operation that failed (e.g., "query", "insert", "update")
        operation: String,
        /// Source error (if available)
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// External API error
    #[error("External API error from {api}: {message}")]
    ExternalAPIError {
        /// API name (e.g., "TMDB", "IMDb")
        api: String,
        /// Error message
        message: String,
        /// HTTP status code (if applicable)
        status_code: Option<u16>,
        /// Source error (if available)
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Network error
    #[error("Network error: {message}")]
    NetworkError {
        /// Error message
        message: String,
        /// Source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Serialization/deserialization error
    #[error("Serialization error: {message}")]
    SerializationError {
        /// Error message
        message: String,
        /// Source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError {
        /// Error message
        message: String,
        /// Configuration key that caused the error
        key: Option<String>,
    },

    /// Search error
    #[error("Search error: {message}")]
    SearchError {
        /// Error message
        message: String,
        /// Query that caused the error
        query: Option<String>,
    },

    /// Cache error
    #[error("Cache error: {operation} failed: {message}")]
    CacheError {
        /// Operation (e.g., "get", "set", "delete")
        operation: String,
        /// Error message
        message: String,
    },

    /// Conflict error (e.g., duplicate key)
    #[error("Conflict: {message}")]
    ConflictError {
        /// Error message
        message: String,
        /// Resource that conflicts
        resource: Option<String>,
    },

    /// Service unavailable
    #[error("Service unavailable: {service}")]
    ServiceUnavailableError {
        /// Service name
        service: String,
        /// Retry after seconds (if known)
        retry_after: Option<u64>,
    },

    /// Timeout error
    #[error("Operation timed out after {duration_ms}ms: {operation}")]
    TimeoutError {
        /// Operation that timed out
        operation: String,
        /// Duration in milliseconds
        duration_ms: u64,
    },

    /// Invalid state error
    #[error("Invalid state: {message}")]
    InvalidStateError {
        /// Error message
        message: String,
        /// Expected state
        expected: Option<String>,
        /// Actual state
        actual: Option<String>,
    },

    /// Feature not implemented
    #[error("Feature not implemented: {feature}")]
    NotImplementedError {
        /// Feature name
        feature: String,
    },

    /// Generic internal error
    #[error("Internal error: {message}")]
    InternalError {
        /// Error message
        message: String,
        /// Source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl MediaGatewayError {
    /// Create a validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::ValidationError {
            message: message.into(),
            field: None,
        }
    }

    /// Create a validation error with field information
    pub fn validation_field<S: Into<String>, F: Into<String>>(message: S, field: F) -> Self {
        Self::ValidationError {
            message: message.into(),
            field: Some(field.into()),
        }
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(content_id: S) -> Self {
        Self::NotFoundError {
            content_id: content_id.into(),
        }
    }

    /// Create a user not found error
    pub fn user_not_found<S: Into<String>>(user_id: S) -> Self {
        Self::UserNotFoundError {
            user_id: user_id.into(),
        }
    }

    /// Create an authentication error
    pub fn authentication<S: Into<String>>(reason: S) -> Self {
        Self::AuthenticationError {
            reason: reason.into(),
        }
    }

    /// Create an authorization error
    pub fn authorization<S: Into<String>>(reason: S) -> Self {
        Self::AuthorizationError {
            reason: reason.into(),
        }
    }

    /// Create a rate limit error
    pub fn rate_limit(limit: u32, window: String, retry_after: Option<u64>) -> Self {
        Self::RateLimitError {
            limit,
            window,
            retry_after,
        }
    }

    /// Create a database error
    pub fn database<S: Into<String>, O: Into<String>>(message: S, operation: O) -> Self {
        Self::DatabaseError {
            message: message.into(),
            operation: operation.into(),
            source: None,
        }
    }

    /// Create an external API error
    pub fn external_api<S: Into<String>, A: Into<String>>(
        api: A,
        message: S,
        status_code: Option<u16>,
    ) -> Self {
        Self::ExternalAPIError {
            api: api.into(),
            message: message.into(),
            status_code,
            source: None,
        }
    }

    /// Create a search error
    pub fn search<S: Into<String>>(message: S) -> Self {
        Self::SearchError {
            message: message.into(),
            query: None,
        }
    }

    /// Create a search error with query information
    pub fn search_with_query<S: Into<String>, Q: Into<String>>(message: S, query: Q) -> Self {
        Self::SearchError {
            message: message.into(),
            query: Some(query.into()),
        }
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::InternalError {
            message: message.into(),
            source: None,
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimitError { .. }
                | Self::ServiceUnavailableError { .. }
                | Self::TimeoutError { .. }
                | Self::NetworkError { .. }
        )
    }

    /// Get retry delay in seconds (if applicable)
    pub fn retry_after_seconds(&self) -> Option<u64> {
        match self {
            Self::RateLimitError { retry_after, .. } => *retry_after,
            Self::ServiceUnavailableError { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    /// Check if error is client-side (4xx equivalent)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::ValidationError { .. }
                | Self::NotFoundError { .. }
                | Self::UserNotFoundError { .. }
                | Self::AuthenticationError { .. }
                | Self::AuthorizationError { .. }
                | Self::ConflictError { .. }
        )
    }

    /// Check if error is server-side (5xx equivalent)
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            Self::DatabaseError { .. }
                | Self::InternalError { .. }
                | Self::ServiceUnavailableError { .. }
        )
    }
}

/// Result type alias for validator integration
impl From<validator::ValidationErrors> for MediaGatewayError {
    fn from(errors: validator::ValidationErrors) -> Self {
        Self::ValidationError {
            message: format!("Validation failed: {}", errors),
            field: None,
        }
    }
}

/// Result type alias for serde_json errors
impl From<serde_json::Error> for MediaGatewayError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerializationError {
            message: error.to_string(),
            source: Some(Box::new(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error() {
        let err = MediaGatewayError::validation("Invalid input");
        assert!(matches!(err, MediaGatewayError::ValidationError { .. }));
        assert!(err.is_client_error());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_not_found_error() {
        let err = MediaGatewayError::not_found("12345");
        assert!(matches!(err, MediaGatewayError::NotFoundError { .. }));
        assert!(err.is_client_error());
    }

    #[test]
    fn test_rate_limit_error() {
        let err = MediaGatewayError::rate_limit(100, "minute".to_string(), Some(60));
        assert!(matches!(err, MediaGatewayError::RateLimitError { .. }));
        assert!(err.is_retryable());
        assert_eq!(err.retry_after_seconds(), Some(60));
    }

    #[test]
    fn test_database_error() {
        let err = MediaGatewayError::database("Connection failed", "query");
        assert!(matches!(err, MediaGatewayError::DatabaseError { .. }));
        assert!(err.is_server_error());
    }

    #[test]
    fn test_external_api_error() {
        let err = MediaGatewayError::external_api("TMDB", "API key invalid", Some(401));
        assert!(matches!(err, MediaGatewayError::ExternalAPIError { .. }));
    }

    #[test]
    fn test_error_display() {
        let err = MediaGatewayError::validation("Test error");
        let display = format!("{}", err);
        assert!(display.contains("Validation error"));
        assert!(display.contains("Test error"));
    }

    #[test]
    fn test_search_error_with_query() {
        let err = MediaGatewayError::search_with_query("Search failed", "test query");
        match err {
            MediaGatewayError::SearchError { query, .. } => {
                assert_eq!(query, Some("test query".to_string()));
            }
            _ => panic!("Wrong error type"),
        }
    }
}
