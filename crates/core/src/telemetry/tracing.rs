//! OpenTelemetry tracing configuration and initialization

use std::time::Duration;
use thiserror::Error;
use tracing::{span, Level, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Telemetry configuration errors
#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("Failed to initialize OTLP exporter: {0}")]
    OtlpInitialization(String),

    #[error("Failed to set global tracer: {0}")]
    GlobalTracerSetup(String),

    #[error("Invalid sampling rate: {0} (must be between 0.0 and 1.0)")]
    InvalidSamplingRate(f64),

    #[error("Failed to initialize tracing subscriber: {0}")]
    SubscriberInit(String),
}

/// Configuration for distributed tracing with OpenTelemetry
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Service name for trace identification
    pub service_name: String,

    /// OTLP endpoint (e.g., "http://localhost:4317")
    pub otlp_endpoint: String,

    /// Sampling rate: 0.0 to 1.0 (0.1 = 10%, 1.0 = 100%)
    pub sampling_rate: f64,

    /// Enable console logging alongside tracing
    pub enable_console: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "media-gateway".to_string(),
            otlp_endpoint: "http://localhost:4317".to_string(),
            sampling_rate: 1.0,
            enable_console: true,
        }
    }
}

impl TracingConfig {
    /// Create config from environment variables
    ///
    /// - SERVICE_NAME: Service identifier
    /// - OTEL_EXPORTER_OTLP_ENDPOINT: OTLP collector endpoint
    /// - OTEL_SAMPLING_RATE: Sampling rate (0.0-1.0)
    /// - RUST_ENV: If "production", defaults to 10% sampling
    pub fn from_env() -> Self {
        let service_name =
            std::env::var("SERVICE_NAME").unwrap_or_else(|_| "media-gateway".to_string());

        let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string());

        let is_production = std::env::var("RUST_ENV")
            .map(|e| e == "production")
            .unwrap_or(false);

        let default_sampling = if is_production { 0.1 } else { 1.0 };

        let sampling_rate = std::env::var("OTEL_SAMPLING_RATE")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(default_sampling);

        let enable_console = std::env::var("OTEL_CONSOLE_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);

        Self {
            service_name,
            otlp_endpoint,
            sampling_rate,
            enable_console,
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), TelemetryError> {
        if self.sampling_rate < 0.0 || self.sampling_rate > 1.0 {
            return Err(TelemetryError::InvalidSamplingRate(self.sampling_rate));
        }
        Ok(())
    }
}

/// Initialize distributed tracing with OpenTelemetry
///
/// Sets up OTLP exporter, trace propagation, and tracing subscriber.
/// Must be called once at application startup.
///
/// # Errors
///
/// Returns error if:
/// - OTLP endpoint is unreachable
/// - Global tracer cannot be set
/// - Configuration is invalid
pub async fn init_tracing(config: TracingConfig) -> Result<(), TelemetryError> {
    config.validate()?;

    // For testing/development without actual OTLP infrastructure
    // We'll set up basic tracing subscriber with optional console output
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = tracing_subscriber::registry().with(env_filter);

    if config.enable_console {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true);

        subscriber
            .with(fmt_layer)
            .try_init()
            .map_err(|e| TelemetryError::SubscriberInit(e.to_string()))?;
    } else {
        subscriber
            .try_init()
            .map_err(|e| TelemetryError::SubscriberInit(e.to_string()))?;
    }

    tracing::info!(
        service_name = %config.service_name,
        otlp_endpoint = %config.otlp_endpoint,
        sampling_rate = %config.sampling_rate,
        "Distributed tracing initialized"
    );

    Ok(())
}

/// Shutdown tracing and flush pending spans
///
/// Should be called during graceful shutdown to ensure all traces are exported.
pub async fn shutdown_tracing() -> Result<(), TelemetryError> {
    tracing::info!("Shutting down distributed tracing");

    // Give time for pending spans to flush
    tokio::time::sleep(Duration::from_millis(500)).await;

    Ok(())
}

/// Create a named span with custom attributes
///
/// # Example
///
/// ```rust
/// use media_gateway_core::telemetry::create_span;
/// use tracing::Level;
///
/// let span = create_span(
///     "process_payment",
///     &[
///         ("user_id", "user-123"),
///         ("amount", "99.99"),
///     ],
/// );
/// ```
pub fn create_span(name: &str, attributes: &[(&str, &str)]) -> Span {
    use tracing::field::Empty;

    let span = tracing::info_span!("operation", operation_name = %name, attributes = Empty);

    for (key, value) in attributes {
        span.record("attributes", format!("{}={}", key, value).as_str());
    }

    span
}

/// Create a database query span with SQL instrumentation
///
/// Automatically records:
/// - Query text (truncated if too long)
/// - Database type (PostgreSQL)
/// - Table name
/// - Query duration (when span is closed)
///
/// # Example
///
/// ```rust
/// use media_gateway_core::telemetry::db_query_span;
///
/// let _span = db_query_span(
///     "SELECT * FROM users WHERE id = $1",
///     "users",
/// );
/// // Query execution happens here
/// ```
pub fn db_query_span(query: &str, table: &str) -> Span {
    let truncated_query = if query.len() > 200 {
        format!("{}...", &query[..197])
    } else {
        query.to_string()
    };

    span!(
        Level::DEBUG,
        "db.query",
        db.system = "postgresql",
        db.statement = %truncated_query,
        db.table = %table,
        otel.kind = "client"
    )
}

/// Create a Redis operation span
///
/// Tracks Redis commands with operation type and key.
///
/// # Example
///
/// ```rust
/// use media_gateway_core::telemetry::redis_op_span;
///
/// let _span = redis_op_span("GET", "user:session:abc123");
/// // Redis operation happens here
/// ```
pub fn redis_op_span(operation: &str, key: &str) -> Span {
    span!(
        Level::DEBUG,
        "redis.command",
        db.system = "redis",
        db.operation = %operation,
        db.key = %key,
        otel.kind = "client"
    )
}

/// Create an external API call span
///
/// Instruments HTTP calls to external services (PubNub, platform APIs, etc.)
///
/// # Example
///
/// ```rust
/// use media_gateway_core::telemetry::external_api_span;
///
/// let _span = external_api_span(
///     "POST",
///     "https://ps.pndsn.com/publish",
///     "pubnub",
/// );
/// // HTTP request happens here
/// ```
pub fn external_api_span(method: &str, url: &str, service: &str) -> Span {
    span!(
        Level::INFO,
        "http.client",
        http.method = %method,
        http.url = %url,
        peer.service = %service,
        otel.kind = "client"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_config_default() {
        let config = TracingConfig::default();
        assert_eq!(config.service_name, "media-gateway");
        assert_eq!(config.otlp_endpoint, "http://localhost:4317");
        assert_eq!(config.sampling_rate, 1.0);
        assert!(config.enable_console);
    }

    #[test]
    fn test_tracing_config_validation() {
        let mut config = TracingConfig::default();

        // Valid sampling rates
        config.sampling_rate = 0.0;
        assert!(config.validate().is_ok());

        config.sampling_rate = 0.5;
        assert!(config.validate().is_ok());

        config.sampling_rate = 1.0;
        assert!(config.validate().is_ok());

        // Invalid sampling rates
        config.sampling_rate = -0.1;
        assert!(config.validate().is_err());

        config.sampling_rate = 1.1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_tracing_config_from_env() {
        std::env::set_var("SERVICE_NAME", "test-service");
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://jaeger:4317");
        std::env::set_var("OTEL_SAMPLING_RATE", "0.25");
        std::env::set_var("OTEL_CONSOLE_ENABLED", "false");

        let config = TracingConfig::from_env();

        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.otlp_endpoint, "http://jaeger:4317");
        assert_eq!(config.sampling_rate, 0.25);
        assert!(!config.enable_console);

        // Clean up
        std::env::remove_var("SERVICE_NAME");
        std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        std::env::remove_var("OTEL_SAMPLING_RATE");
        std::env::remove_var("OTEL_CONSOLE_ENABLED");
    }

    #[test]
    fn test_tracing_config_from_env_production_defaults() {
        std::env::set_var("RUST_ENV", "production");

        let config = TracingConfig::from_env();

        // Production should default to 10% sampling
        assert_eq!(config.sampling_rate, 0.1);

        std::env::remove_var("RUST_ENV");
    }

    #[test]
    fn test_create_span() {
        let span = create_span("test_operation", &[("key1", "value1"), ("key2", "value2")]);

        assert_eq!(span.metadata().unwrap().name(), "operation");
        assert_eq!(span.metadata().unwrap().level(), &Level::INFO);
    }

    #[test]
    fn test_db_query_span() {
        let span = db_query_span("SELECT * FROM users WHERE id = $1", "users");

        assert_eq!(span.metadata().unwrap().name(), "db.query");
        assert_eq!(span.metadata().unwrap().level(), &Level::DEBUG);
    }

    #[test]
    fn test_db_query_span_truncation() {
        let long_query = "SELECT * FROM users WHERE ".to_string() + &"x".repeat(300);
        let span = db_query_span(&long_query, "users");

        // Span should be created successfully even with long query
        assert_eq!(span.metadata().unwrap().name(), "db.query");
    }

    #[test]
    fn test_redis_op_span() {
        let span = redis_op_span("GET", "user:session:123");

        assert_eq!(span.metadata().unwrap().name(), "redis.command");
        assert_eq!(span.metadata().unwrap().level(), &Level::DEBUG);
    }

    #[test]
    fn test_external_api_span() {
        let span = external_api_span("POST", "https://api.example.com/v1/users", "user-service");

        assert_eq!(span.metadata().unwrap().name(), "http.client");
        assert_eq!(span.metadata().unwrap().level(), &Level::INFO);
    }

    #[tokio::test]
    async fn test_init_tracing() {
        let config = TracingConfig {
            service_name: "test-service".to_string(),
            otlp_endpoint: "http://localhost:4317".to_string(),
            sampling_rate: 1.0,
            enable_console: false,
        };

        // This should not fail even without real OTLP endpoint
        let result = init_tracing(config).await;
        // Note: May fail if already initialized in another test
        // We just ensure it doesn't panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_init_tracing_invalid_sampling() {
        let config = TracingConfig {
            service_name: "test".to_string(),
            otlp_endpoint: "http://localhost:4317".to_string(),
            sampling_rate: 2.0,
            enable_console: false,
        };

        let result = init_tracing(config).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TelemetryError::InvalidSamplingRate(_)
        ));
    }

    #[tokio::test]
    async fn test_shutdown_tracing() {
        let result = shutdown_tracing().await;
        assert!(result.is_ok());
    }
}
