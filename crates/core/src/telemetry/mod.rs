//! Distributed tracing and telemetry using OpenTelemetry
//!
//! This module provides comprehensive distributed tracing capabilities across
//! all Media Gateway services using OpenTelemetry and OTLP exporters.
//!
//! # Features
//!
//! - Automatic trace context propagation via `traceparent` headers
//! - Span creation for HTTP handlers, database queries, Redis operations
//! - Service-to-service trace correlation
//! - Jaeger/Zipkin exporter support
//! - Configurable sampling rates by environment
//!
//! # Example
//!
//! ```rust,no_run
//! use media_gateway_core::telemetry::{TracingConfig, init_tracing};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = TracingConfig {
//!         service_name: "api-gateway".to_string(),
//!         otlp_endpoint: "http://localhost:4317".to_string(),
//!         sampling_rate: 1.0,
//!         enable_console: true,
//!     };
//!
//!     init_tracing(config).await?;
//!     Ok(())
//! }
//! ```

pub mod middleware;
pub mod tracing;

pub use self::tracing::{
    create_span, db_query_span, external_api_span, init_tracing, redis_op_span, shutdown_tracing,
    TelemetryError, TracingConfig,
};
pub use middleware::{
    extract_trace_context, inject_trace_context, TraceContext, TracingMiddleware,
};
