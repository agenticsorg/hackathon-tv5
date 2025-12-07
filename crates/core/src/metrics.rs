//! # Prometheus Metrics Module
//!
//! Centralized metrics collection and exposure for the Media Gateway platform.
//!
//! ## Features
//!
//! - HTTP request metrics (total count, duration, status codes)
//! - Connection pool metrics (active/idle database connections)
//! - Cache performance metrics (hits/misses by cache type)
//! - Standardized Prometheus metric naming conventions
//! - Thread-safe global registry using lazy_static
//! - Actix-web middleware for automatic request instrumentation
//!
//! ## Usage
//!
//! ```rust
//! use media_gateway_core::metrics::{metrics_handler, increment_counter, observe_histogram};
//!
//! // In your Actix-web application
//! HttpServer::new(|| {
//!     App::new()
//!         .route("/metrics", web::get().to(metrics_handler))
//! })
//! ```

use once_cell::sync::Lazy;
use prometheus::{
    Counter, CounterVec, Encoder, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec, Opts,
    Registry, TextEncoder,
};
use std::collections::HashMap;

/// Global Prometheus metrics registry
///
/// This registry is initialized once at startup and used throughout
/// the application lifecycle. All metrics are registered here.
pub static METRICS_REGISTRY: Lazy<MetricsRegistry> = Lazy::new(MetricsRegistry::new);

/// Histogram buckets for request duration in seconds
///
/// Covers a range from 1ms to 5s, suitable for most HTTP request patterns:
/// - 1ms, 5ms, 10ms: Fast cached responses
/// - 25ms, 50ms, 100ms: Typical database queries
/// - 250ms, 500ms: Complex queries
/// - 1s, 2.5s, 5s: Long-running operations
const DURATION_BUCKETS: &[f64] = &[
    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0,
];

/// Central metrics registry containing all application metrics
///
/// This struct wraps the Prometheus registry and provides
/// typed access to all defined metrics.
pub struct MetricsRegistry {
    /// Prometheus registry instance
    registry: Registry,

    /// HTTP request counter
    /// Labels: method (GET/POST/etc), path (/api/content), status (200/404/etc)
    pub http_requests_total: CounterVec,

    /// HTTP request duration histogram in seconds
    /// Labels: method (GET/POST/etc), path (/api/content)
    pub http_request_duration_seconds: HistogramVec,

    /// Active HTTP/WebSocket connections gauge
    pub active_connections: Gauge,

    /// Active database connections from the pool
    pub db_connections_active: Gauge,

    /// Idle database connections in the pool
    pub db_connections_idle: Gauge,

    /// Cache hit counter
    /// Labels: cache_type (redis/memory/cdn)
    pub cache_hits_total: CounterVec,

    /// Cache miss counter
    /// Labels: cache_type (redis/memory/cdn)
    pub cache_misses_total: CounterVec,
}

impl MetricsRegistry {
    /// Create a new metrics registry with all metrics registered
    ///
    /// This function is called once at startup via lazy_static.
    /// All metrics are pre-registered to avoid runtime errors.
    pub fn new() -> Self {
        let registry = Registry::new();

        // HTTP request counter
        let http_requests_total = CounterVec::new(
            Opts::new(
                "http_requests_total",
                "Total number of HTTP requests processed",
            ),
            &["method", "path", "status"],
        )
        .expect("Failed to create http_requests_total metric");

        // HTTP request duration histogram
        let http_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request latency in seconds",
            )
            .buckets(DURATION_BUCKETS.to_vec()),
            &["method", "path"],
        )
        .expect("Failed to create http_request_duration_seconds metric");

        // Active connections gauge
        let active_connections = Gauge::new(
            "active_connections",
            "Number of active HTTP/WebSocket connections",
        )
        .expect("Failed to create active_connections metric");

        // Database connection pool gauges
        let db_connections_active = Gauge::new(
            "db_connections_active",
            "Number of active database connections in the pool",
        )
        .expect("Failed to create db_connections_active metric");

        let db_connections_idle = Gauge::new(
            "db_connections_idle",
            "Number of idle database connections in the pool",
        )
        .expect("Failed to create db_connections_idle metric");

        // Cache performance counters
        let cache_hits_total = CounterVec::new(
            Opts::new("cache_hits_total", "Total number of cache hits"),
            &["cache_type"],
        )
        .expect("Failed to create cache_hits_total metric");

        let cache_misses_total = CounterVec::new(
            Opts::new("cache_misses_total", "Total number of cache misses"),
            &["cache_type"],
        )
        .expect("Failed to create cache_misses_total metric");

        // Register all metrics with the registry
        registry
            .register(Box::new(http_requests_total.clone()))
            .expect("Failed to register http_requests_total");
        registry
            .register(Box::new(http_request_duration_seconds.clone()))
            .expect("Failed to register http_request_duration_seconds");
        registry
            .register(Box::new(active_connections.clone()))
            .expect("Failed to register active_connections");
        registry
            .register(Box::new(db_connections_active.clone()))
            .expect("Failed to register db_connections_active");
        registry
            .register(Box::new(db_connections_idle.clone()))
            .expect("Failed to register db_connections_idle");
        registry
            .register(Box::new(cache_hits_total.clone()))
            .expect("Failed to register cache_hits_total");
        registry
            .register(Box::new(cache_misses_total.clone()))
            .expect("Failed to register cache_misses_total");

        Self {
            registry,
            http_requests_total,
            http_request_duration_seconds,
            active_connections,
            db_connections_active,
            db_connections_idle,
            cache_hits_total,
            cache_misses_total,
        }
    }

    /// Get the underlying Prometheus registry
    ///
    /// Useful for registering custom metrics or integrating
    /// with third-party Prometheus exporters.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Gather all metrics and encode them in Prometheus text format
    ///
    /// Returns a String containing the metrics suitable for
    /// serving at the /metrics endpoint.
    pub fn gather(&self) -> Result<String, Box<dyn std::error::Error>> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Record an HTTP request with method, path, and status code
///
/// # Arguments
///
/// * `method` - HTTP method (GET, POST, PUT, DELETE, etc.)
/// * `path` - Request path (e.g., "/api/content")
/// * `status` - HTTP status code (200, 404, 500, etc.)
///
/// # Example
///
/// ```rust
/// record_http_request("GET", "/api/content", "200");
/// ```
pub fn record_http_request(method: &str, path: &str, status: &str) {
    METRICS_REGISTRY
        .http_requests_total
        .with_label_values(&[method, path, status])
        .inc();
}

/// Observe HTTP request duration
///
/// # Arguments
///
/// * `method` - HTTP method (GET, POST, etc.)
/// * `path` - Request path
/// * `duration_seconds` - Duration in seconds (e.g., 0.123)
///
/// # Example
///
/// ```rust
/// use std::time::Instant;
/// let start = Instant::now();
/// // ... handle request ...
/// observe_http_duration("GET", "/api/content", start.elapsed().as_secs_f64());
/// ```
pub fn observe_http_duration(method: &str, path: &str, duration_seconds: f64) {
    METRICS_REGISTRY
        .http_request_duration_seconds
        .with_label_values(&[method, path])
        .observe(duration_seconds);
}

/// Increment active connections counter
pub fn increment_active_connections() {
    METRICS_REGISTRY.active_connections.inc();
}

/// Decrement active connections counter
pub fn decrement_active_connections() {
    METRICS_REGISTRY.active_connections.dec();
}

/// Update database connection pool metrics
///
/// # Arguments
///
/// * `active` - Number of active connections
/// * `idle` - Number of idle connections
///
/// # Example
///
/// ```rust
/// update_db_pool_metrics(5, 15);
/// ```
pub fn update_db_pool_metrics(active: usize, idle: usize) {
    METRICS_REGISTRY.db_connections_active.set(active as f64);
    METRICS_REGISTRY.db_connections_idle.set(idle as f64);
}

/// Record a cache hit
///
/// # Arguments
///
/// * `cache_type` - Type of cache (e.g., "redis", "memory", "cdn")
///
/// # Example
///
/// ```rust
/// record_cache_hit("redis");
/// ```
pub fn record_cache_hit(cache_type: &str) {
    METRICS_REGISTRY
        .cache_hits_total
        .with_label_values(&[cache_type])
        .inc();
}

/// Record a cache miss
///
/// # Arguments
///
/// * `cache_type` - Type of cache (e.g., "redis", "memory", "cdn")
///
/// # Example
///
/// ```rust
/// record_cache_miss("redis");
/// ```
pub fn record_cache_miss(cache_type: &str) {
    METRICS_REGISTRY
        .cache_misses_total
        .with_label_values(&[cache_type])
        .inc();
}

/// Actix-web handler for the /metrics endpoint
///
/// Returns metrics in Prometheus text exposition format.
///
/// # Example
///
/// ```rust
/// use actix_web::{web, App, HttpServer};
/// use media_gateway_core::metrics::metrics_handler;
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     HttpServer::new(|| {
///         App::new()
///             .route("/metrics", web::get().to(metrics_handler))
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
pub async fn metrics_handler() -> actix_web::HttpResponse {
    match METRICS_REGISTRY.gather() {
        Ok(metrics) => actix_web::HttpResponse::Ok()
            .content_type("text/plain; version=0.0.4")
            .body(metrics),
        Err(e) => {
            tracing::error!("Failed to gather metrics: {}", e);
            actix_web::HttpResponse::InternalServerError()
                .body(format!("Failed to gather metrics: {}", e))
        }
    }
}

/// Middleware for automatic HTTP request instrumentation
///
/// This middleware automatically records HTTP request metrics including:
/// - Request count by method, path, and status
/// - Request duration by method and path
/// - Active connection tracking
///
/// # Example
///
/// ```rust
/// use actix_web::{App, HttpServer};
/// use media_gateway_core::metrics::MetricsMiddleware;
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     HttpServer::new(|| {
///         App::new()
///             .wrap(MetricsMiddleware)
///             // ... your routes ...
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
pub struct MetricsMiddleware;

impl<S, B> actix_web::dev::Transform<S, actix_web::dev::ServiceRequest> for MetricsMiddleware
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse<B>,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = MetricsMiddlewareService<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(MetricsMiddlewareService { service }))
    }
}

pub struct MetricsMiddlewareService<S> {
    service: S,
}

impl<S, B> actix_web::dev::Service<actix_web::dev::ServiceRequest> for MetricsMiddlewareService<S>
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse<B>,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: actix_web::dev::ServiceRequest) -> Self::Future {
        let start = std::time::Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();

        increment_active_connections();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start.elapsed().as_secs_f64();
            let status = res.status().as_u16().to_string();

            record_http_request(&method, &path, &status);
            observe_http_duration(&method, &path, duration);
            decrement_active_connections();

            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registry_creation() {
        let registry = MetricsRegistry::new();
        assert!(registry.gather().is_ok());
    }

    #[test]
    fn test_record_http_request() {
        record_http_request("GET", "/api/test", "200");
        record_http_request("POST", "/api/test", "201");

        let metrics = METRICS_REGISTRY.gather().unwrap();
        assert!(metrics.contains("http_requests_total"));
    }

    #[test]
    fn test_observe_http_duration() {
        observe_http_duration("GET", "/api/test", 0.123);
        observe_http_duration("POST", "/api/test", 0.456);

        let metrics = METRICS_REGISTRY.gather().unwrap();
        assert!(metrics.contains("http_request_duration_seconds"));
    }

    #[test]
    fn test_active_connections() {
        let initial = METRICS_REGISTRY.active_connections.get();
        increment_active_connections();
        assert_eq!(METRICS_REGISTRY.active_connections.get(), initial + 1.0);
        decrement_active_connections();
        assert_eq!(METRICS_REGISTRY.active_connections.get(), initial);
    }

    #[test]
    fn test_db_pool_metrics() {
        update_db_pool_metrics(10, 20);
        assert_eq!(METRICS_REGISTRY.db_connections_active.get(), 10.0);
        assert_eq!(METRICS_REGISTRY.db_connections_idle.get(), 20.0);
    }

    #[test]
    fn test_cache_metrics() {
        record_cache_hit("redis");
        record_cache_miss("redis");
        record_cache_hit("memory");

        let metrics = METRICS_REGISTRY.gather().unwrap();
        assert!(metrics.contains("cache_hits_total"));
        assert!(metrics.contains("cache_misses_total"));
    }

    #[test]
    fn test_histogram_buckets() {
        // Verify that buckets are configured correctly
        let buckets = DURATION_BUCKETS;
        assert_eq!(buckets.len(), 11);
        assert_eq!(buckets[0], 0.001); // 1ms
        assert_eq!(buckets[10], 5.0); // 5s
    }

    #[test]
    fn test_metrics_text_format() {
        record_http_request("GET", "/test", "200");
        let metrics = METRICS_REGISTRY.gather().unwrap();

        // Verify Prometheus text format
        assert!(metrics.contains("# HELP"));
        assert!(metrics.contains("# TYPE"));
        assert!(metrics.contains("http_requests_total"));
    }

    #[tokio::test]
    async fn test_metrics_handler() {
        let response = metrics_handler().await;
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
    }
}
