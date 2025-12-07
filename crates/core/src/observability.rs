//! Observability module for structured logging and distributed tracing
//!
//! Provides initialization and utilities for structured logging using tracing-subscriber,
//! with support for JSON and pretty-printed formats, correlation IDs, and request tracing.

use serde::{Deserialize, Serialize};
use std::env;
use std::io;
use tracing::{span, Span};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Log output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// JSON format for production (machine-readable)
    Json,
    /// Pretty-printed format for development (human-readable)
    Pretty,
}

impl Default for LogFormat {
    fn default() -> Self {
        // Use JSON in production, Pretty in development
        if cfg!(debug_assertions) {
            Self::Pretty
        } else {
            Self::Json
        }
    }
}

impl std::fmt::Display for LogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogFormat::Json => write!(f, "json"),
            LogFormat::Pretty => write!(f, "pretty"),
        }
    }
}

impl std::str::FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(LogFormat::Json),
            "pretty" => Ok(LogFormat::Pretty),
            _ => Err(format!(
                "Invalid log format: '{}'. Valid values are 'json' or 'pretty'",
                s
            )),
        }
    }
}

/// Configuration for logging initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Output format (json or pretty)
    pub format: LogFormat,

    /// Log level filter (e.g., "info", "debug", "warn")
    /// Can include module-level filters like "media_gateway=debug,sqlx=warn"
    pub level: String,

    /// Service name to include in log output
    pub service_name: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::default(),
            level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            service_name: "media-gateway".to_string(),
        }
    }
}

impl LogConfig {
    /// Create a new LogConfig with custom values
    pub fn new(format: LogFormat, level: String, service_name: String) -> Self {
        Self {
            format,
            level,
            service_name,
        }
    }

    /// Create a production configuration (JSON format, info level)
    pub fn production(service_name: String) -> Self {
        Self {
            format: LogFormat::Json,
            level: "info".to_string(),
            service_name,
        }
    }

    /// Create a development configuration (Pretty format, debug level)
    pub fn development(service_name: String) -> Self {
        Self {
            format: LogFormat::Pretty,
            level: "debug".to_string(),
            service_name,
        }
    }
}

/// Error type for observability initialization
#[derive(Debug, thiserror::Error)]
pub enum ObservabilityError {
    /// Error setting up the tracing subscriber
    #[error("Failed to initialize tracing subscriber: {0}")]
    InitializationError(String),

    /// Error parsing environment filter
    #[error("Invalid log level filter: {0}")]
    InvalidFilter(#[from] tracing_subscriber::filter::ParseError),
}

/// Initialize structured logging with tracing-subscriber
///
/// Sets up a layered tracing subscriber with configurable format (JSON or Pretty),
/// log levels, and service metadata. Supports correlation IDs and span tracking
/// for distributed tracing.
///
/// # Arguments
///
/// * `config` - Logging configuration
///
/// # Returns
///
/// * `Ok(())` - Logging initialized successfully
/// * `Err(ObservabilityError)` - Failed to initialize logging
///
/// # Examples
///
/// ```no_run
/// use media_gateway_core::observability::{init_logging, LogConfig, LogFormat};
///
/// let config = LogConfig::new(
///     LogFormat::Json,
///     "info,media_gateway=debug".to_string(),
///     "media-gateway-api".to_string(),
/// );
///
/// init_logging(&config).expect("Failed to initialize logging");
/// ```
pub fn init_logging(config: &LogConfig) -> Result<(), ObservabilityError> {
    // Parse the log level filter from RUST_LOG env var or config
    let env_filter = EnvFilter::try_new(&config.level)
        .or_else(|_| EnvFilter::try_from_default_env())
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Build the subscriber based on the format
    match config.format {
        LogFormat::Json => {
            // JSON format for production
            let fmt_layer = fmt::layer()
                .json()
                .with_target(true)
                .with_level(true)
                .with_current_span(true)
                .with_span_list(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(true)
                .with_line_number(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_writer(io::stdout)
                .with_filter(env_filter);

            tracing_subscriber::registry()
                .with(fmt_layer)
                .try_init()
                .map_err(|e| {
                    ObservabilityError::InitializationError(format!(
                        "Failed to set global subscriber: {}",
                        e
                    ))
                })?;
        }
        LogFormat::Pretty => {
            // Pretty format for development
            let fmt_layer = fmt::layer()
                .pretty()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(true)
                .with_line_number(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_writer(io::stdout)
                .with_filter(env_filter);

            tracing_subscriber::registry()
                .with(fmt_layer)
                .try_init()
                .map_err(|e| {
                    ObservabilityError::InitializationError(format!(
                        "Failed to set global subscriber: {}",
                        e
                    ))
                })?;
        }
    }

    tracing::info!(
        service_name = %config.service_name,
        log_format = %config.format,
        log_level = %config.level,
        "Observability initialized"
    );

    Ok(())
}

/// Execute a function within a correlation ID context
///
/// Creates a tracing span with the given correlation ID and executes the provided
/// function within that span. This allows all log messages and spans created within
/// the function to be correlated.
///
/// # Arguments
///
/// * `id` - Correlation ID (e.g., request ID, trace ID)
/// * `f` - Function to execute within the correlation context
///
/// # Returns
///
/// Returns the result of the function execution
///
/// # Examples
///
/// ```
/// use media_gateway_core::observability::with_correlation_id;
///
/// let result = with_correlation_id("req-12345", || {
///     // All logs here will include correlation_id = "req-12345"
///     tracing::info!("Processing request");
///     42
/// });
///
/// assert_eq!(result, 42);
/// ```
pub fn with_correlation_id<T, F>(id: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let span = span!(
        tracing::Level::INFO,
        "request",
        correlation_id = %id,
        otel.kind = "server"
    );
    let _enter = span.enter();
    f()
}

/// Get the current correlation ID from the active tracing span
///
/// Attempts to extract the correlation_id field from the current span context.
/// Returns None if no span is active or if the correlation_id field is not set.
///
/// # Returns
///
/// * `Some(String)` - The correlation ID if found
/// * `None` - No correlation ID in the current context
///
/// # Examples
///
/// ```
/// use media_gateway_core::observability::{with_correlation_id, current_correlation_id};
///
/// with_correlation_id("req-67890", || {
///     let id = current_correlation_id();
///     // Note: This may return None due to limitations in extracting
///     // field values from the current span. Use the span context directly
///     // for production use cases.
/// });
/// ```
pub fn current_correlation_id() -> Option<String> {
    // Note: Extracting field values from the current span is not directly supported
    // by tracing. This function provides a best-effort implementation.
    // For production use, consider using a thread-local or context manager.

    // Try to get the current span
    let current_span = Span::current();

    // Check if we're in a span
    if current_span.is_none() {
        return None;
    }

    // For now, we return None as tracing doesn't provide direct field access
    // In production, you would typically:
    // 1. Use a thread-local storage
    // 2. Use tokio::task_local for async contexts
    // 3. Use OpenTelemetry context propagation
    None
}

/// Create a new span for request tracing
///
/// Helper function to create a properly configured span for request tracing
/// with common fields like correlation_id, method, path, etc.
///
/// # Arguments
///
/// * `correlation_id` - Request correlation ID
/// * `method` - HTTP method (e.g., "GET", "POST")
/// * `path` - Request path
///
/// # Returns
///
/// A configured tracing span
///
/// # Examples
///
/// ```
/// use media_gateway_core::observability::request_span;
///
/// let span = request_span("req-12345", "GET", "/api/content");
/// let _enter = span.enter();
///
/// tracing::info!("Processing request");
/// ```
pub fn request_span(correlation_id: &str, method: &str, path: &str) -> Span {
    span!(
        tracing::Level::INFO,
        "http.request",
        correlation_id = %correlation_id,
        http.method = %method,
        http.path = %path,
        otel.kind = "server",
        otel.status_code = tracing::field::Empty,
    )
}

/// Create a new span for database operations
///
/// Helper function to create a span for database query tracing with relevant metadata.
///
/// # Arguments
///
/// * `operation` - Database operation (e.g., "SELECT", "INSERT", "UPDATE")
/// * `table` - Database table name
///
/// # Returns
///
/// A configured tracing span
///
/// # Examples
///
/// ```
/// use media_gateway_core::observability::db_span;
///
/// let span = db_span("SELECT", "users");
/// let _enter = span.enter();
///
/// tracing::debug!("Executing query");
/// ```
pub fn db_span(operation: &str, table: &str) -> Span {
    span!(
        tracing::Level::DEBUG,
        "db.query",
        db.operation = %operation,
        db.table = %table,
        otel.kind = "client",
    )
}

/// Create a new span for external API calls
///
/// Helper function to create a span for external API call tracing.
///
/// # Arguments
///
/// * `service` - External service name (e.g., "tmdb", "imdb")
/// * `endpoint` - API endpoint being called
///
/// # Returns
///
/// A configured tracing span
///
/// # Examples
///
/// ```
/// use media_gateway_core::observability::api_span;
///
/// let span = api_span("tmdb", "/movie/550");
/// let _enter = span.enter();
///
/// tracing::info!("Calling external API");
/// ```
pub fn api_span(service: &str, endpoint: &str) -> Span {
    span!(
        tracing::Level::INFO,
        "api.call",
        api.service = %service,
        api.endpoint = %endpoint,
        otel.kind = "client",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_default() {
        let format = LogFormat::default();
        if cfg!(debug_assertions) {
            assert_eq!(format, LogFormat::Pretty);
        } else {
            assert_eq!(format, LogFormat::Json);
        }
    }

    #[test]
    fn test_log_format_display() {
        assert_eq!(LogFormat::Json.to_string(), "json");
        assert_eq!(LogFormat::Pretty.to_string(), "pretty");
    }

    #[test]
    fn test_log_format_from_str() {
        assert_eq!("json".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert_eq!("JSON".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert_eq!("pretty".parse::<LogFormat>().unwrap(), LogFormat::Pretty);
        assert_eq!("PRETTY".parse::<LogFormat>().unwrap(), LogFormat::Pretty);

        assert!("invalid".parse::<LogFormat>().is_err());
    }

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.service_name, "media-gateway");
    }

    #[test]
    fn test_log_config_production() {
        let config = LogConfig::production("test-service".to_string());
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.level, "info");
        assert_eq!(config.service_name, "test-service");
    }

    #[test]
    fn test_log_config_development() {
        let config = LogConfig::development("dev-service".to_string());
        assert_eq!(config.format, LogFormat::Pretty);
        assert_eq!(config.level, "debug");
        assert_eq!(config.service_name, "dev-service");
    }

    #[test]
    fn test_with_correlation_id() {
        let result = with_correlation_id("test-id-123", || {
            // Verify function executes correctly
            42
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_request_span_creation() {
        let span = request_span("req-123", "GET", "/api/test");
        assert!(!span.is_disabled());
    }

    #[test]
    fn test_db_span_creation() {
        let span = db_span("SELECT", "users");
        assert!(!span.is_disabled());
    }

    #[test]
    fn test_api_span_creation() {
        let span = api_span("tmdb", "/movie/550");
        assert!(!span.is_disabled());
    }

    #[test]
    fn test_current_correlation_id_no_span() {
        // Should return None when not in a span
        let id = current_correlation_id();
        assert_eq!(id, None);
    }

    #[test]
    fn test_init_logging_with_invalid_level() {
        // Test with completely invalid filter syntax
        let config = LogConfig::new(
            LogFormat::Pretty,
            "invalid[filter".to_string(),
            "test-service".to_string(),
        );

        // This should fail with InvalidFilter error
        let result = init_logging(&config);
        assert!(result.is_err());
    }
}
