//! Production Health Check Integration Example
//!
//! Demonstrates how to integrate the core health check system into services
//! with PostgreSQL, Redis, and Qdrant health monitoring.
//!
//! ## Features
//!
//! - Parallel health check execution
//! - 2-second per-check timeout protection
//! - Critical vs non-critical component classification
//! - Both simple and detailed health endpoints
//! - Proper HTTP status codes (200/503)
//!
//! ## Usage
//!
//! ```bash
//! # Set required environment variables
//! export DATABASE_URL="postgresql://localhost/media_gateway"
//! export REDIS_URL="redis://localhost:6379"
//! export QDRANT_URL="http://localhost:6333"
//!
//! # Run the example
//! cargo run --example health_check_integration
//!
//! # Test endpoints
//! curl http://localhost:8080/health           # Simple health status
//! curl http://localhost:8080/health/ready     # Detailed readiness check
//! ```

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use media_gateway_core::{
    health::{AggregatedHealth, HealthChecker, SimpleHealth},
    DatabasePool,
};
use std::sync::Arc;
use tracing::info;

/// Application state with health checker
struct AppState {
    health_checker: Arc<HealthChecker>,
}

/// Simple health endpoint - returns minimal status
///
/// Returns 200 OK if healthy or degraded (still serving traffic)
/// Returns 503 Service Unavailable if unhealthy (critical components down)
async fn health(state: web::Data<AppState>) -> impl Responder {
    let simple_health: SimpleHealth = state.health_checker.check_simple().await;
    let health_full = state.health_checker.check_all().await;

    let status_code = if health_full.is_ready() {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    HttpResponse::build(status_code).json(simple_health)
}

/// Detailed readiness endpoint - returns component-level status
///
/// Returns full health status including:
/// - Overall status (healthy/degraded/unhealthy)
/// - Individual component health (postgres, redis, qdrant)
/// - Latency for each component
/// - Error messages for failing components
/// - Service version and timestamp
async fn health_ready(state: web::Data<AppState>) -> impl Responder {
    let health: AggregatedHealth = state.health_checker.check_ready().await;

    let status_code = if health.is_ready() {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    HttpResponse::build(status_code).json(health)
}

/// Liveness endpoint - minimal check that service process is running
///
/// Used by Kubernetes liveness probes to determine if container should be restarted
async fn liveness() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    info!("Initializing health check integration example");

    // Get configuration from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/media_gateway".to_string());
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let qdrant_url =
        std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());

    // Initialize database pool
    info!("Connecting to PostgreSQL: {}", database_url);
    let db_pool = DatabasePool::from_env()
        .await
        .expect("Failed to create database pool");

    // Initialize Redis client
    info!("Connecting to Redis: {}", redis_url);
    let redis_client = redis::Client::open(redis_url).expect("Failed to create Redis client");

    // Build health checker with all components
    info!("Initializing health checker");
    let health_checker = Arc::new(
        HealthChecker::new()
            .with_postgres(db_pool.pool().clone())
            .with_redis(redis_client)
            .with_qdrant(qdrant_url),
    );

    // Create application state
    let app_state = web::Data::new(AppState { health_checker });

    // Run initial health check
    info!("Running initial health check");
    let initial_health = app_state.health_checker.check_all().await;
    info!(
        "Initial health status: {:?} ({}ms total latency)",
        initial_health.status, initial_health.total_latency_ms
    );

    for component in &initial_health.components {
        info!(
            "  - {}: {:?} ({}ms, critical: {})",
            component.name, component.status, component.latency_ms, component.critical
        );
        if let Some(ref message) = component.message {
            info!("    Message: {}", message);
        }
    }

    // Start HTTP server
    let bind_addr = "0.0.0.0:8080";
    info!("Starting HTTP server on {}", bind_addr);
    info!("Health endpoints:");
    info!("  - GET /health        - Simple health status");
    info!("  - GET /health/ready  - Detailed readiness check");
    info!("  - GET /liveness      - Liveness probe");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/health", web::get().to(health))
            .route("/health/ready", web::get().to(health_ready))
            .route("/liveness", web::get().to(liveness))
            .wrap(actix_web::middleware::Logger::default())
    })
    .workers(2)
    .bind(bind_addr)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use media_gateway_core::health::{ComponentHealth, HealthStatus};

    #[test]
    fn test_health_status_mapping() {
        // Healthy status should return 200
        let healthy = vec![ComponentHealth::healthy("test", 10, true)];
        let health = AggregatedHealth::from_components(healthy, 10);
        assert_eq!(health.http_status_code(), 200);
        assert!(health.is_ready());

        // Degraded (non-critical failure) should return 200
        let degraded = vec![
            ComponentHealth::healthy("postgres", 10, true),
            ComponentHealth::unhealthy("redis", 2000, false, "Timeout"),
        ];
        let health = AggregatedHealth::from_components(degraded, 2010);
        assert_eq!(health.status, HealthStatus::Degraded);
        assert_eq!(health.http_status_code(), 200);
        assert!(health.is_ready());

        // Unhealthy (critical failure) should return 503
        let unhealthy = vec![ComponentHealth::unhealthy(
            "postgres",
            2000,
            true,
            "Database unreachable",
        )];
        let health = AggregatedHealth::from_components(unhealthy, 2000);
        assert_eq!(health.status, HealthStatus::Unhealthy);
        assert_eq!(health.http_status_code(), 503);
        assert!(!health.is_ready());
    }
}
