//! Health check endpoints for Ingestion Service
//!
//! Provides production-ready health monitoring for the ingestion pipeline
//! including PostgreSQL, Redis, and Qdrant health checks.

use actix_web::{web, HttpResponse, Responder};
use media_gateway_core::health::{AggregatedHealth, HealthChecker, SimpleHealth};
use std::sync::Arc;

/// Application state with health checker
pub struct HealthState {
    pub checker: Arc<HealthChecker>,
}

/// Simple health endpoint - GET /health
///
/// Returns minimal health status for load balancer checks.
/// - 200 OK if healthy or degraded (still accepting requests)
/// - 503 Service Unavailable if unhealthy (critical components down)
pub async fn health(state: web::Data<HealthState>) -> impl Responder {
    let simple_health: SimpleHealth = state.checker.check_simple().await;
    let full_health = state.checker.check_all().await;

    let status_code = if full_health.is_ready() {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    HttpResponse::build(status_code).json(simple_health)
}

/// Detailed readiness endpoint - GET /health/ready
///
/// Returns comprehensive component-level health status including:
/// - PostgreSQL (critical) - required for metadata storage
/// - Redis (non-critical) - used for caching and job queues
/// - Qdrant (critical) - required for vector storage
///
/// Returns 503 if any critical component is unhealthy.
pub async fn ready(state: web::Data<HealthState>) -> impl Responder {
    let health: AggregatedHealth = state.checker.check_ready().await;

    let status_code = if health.is_ready() {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    HttpResponse::build(status_code).json(health)
}

/// Liveness probe endpoint - GET /liveness
///
/// Minimal check that the service process is running.
/// Used by Kubernetes to determine if container should be restarted.
pub async fn liveness() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "service": "ingestion",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use media_gateway_core::health::HealthStatus;

    #[test]
    fn test_liveness_response() {
        // Liveness should always return alive (no dependencies checked)
        // This is tested in integration tests with actual HTTP calls
    }
}
