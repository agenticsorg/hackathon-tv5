pub mod aggregator;

pub use aggregator::{
    AggregatedHealth, DependencyHealth, HealthAggregator, HealthStatus, ServiceHealth,
};

use crate::circuit_breaker::CircuitBreakerManager;
use crate::proxy::ServiceProxy;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: HashMap<String, ServiceHealthCheck>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceHealthCheck {
    pub status: String,
    pub circuit_breaker: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub services: HashMap<String, bool>,
}

pub struct HealthChecker {
    proxy: Arc<ServiceProxy>,
    circuit_breaker: Arc<CircuitBreakerManager>,
    start_time: std::time::Instant,
}

impl HealthChecker {
    pub fn new(proxy: Arc<ServiceProxy>, circuit_breaker: Arc<CircuitBreakerManager>) -> Self {
        Self {
            proxy,
            circuit_breaker,
            start_time: std::time::Instant::now(),
        }
    }

    pub async fn health_check(&self) -> HealthResponse {
        let mut checks = HashMap::new();

        checks.insert(
            "discovery".to_string(),
            self.check_service("discovery").await,
        );
        checks.insert("sona".to_string(), self.check_service("sona").await);
        checks.insert("sync".to_string(), self.check_service("sync").await);
        checks.insert("auth".to_string(), self.check_service("auth").await);
        checks.insert("playback".to_string(), self.check_service("playback").await);

        let critical_services = ["discovery", "auth"];
        let status = if critical_services.iter().any(|s| {
            checks
                .get(*s)
                .map(|h| h.status == "healthy")
                .unwrap_or(false)
        }) {
            "healthy"
        } else {
            "unhealthy"
        };

        HealthResponse {
            status: status.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            checks,
        }
    }

    pub async fn readiness_check(&self) -> ReadinessResponse {
        let mut services = HashMap::new();

        services.insert(
            "discovery".to_string(),
            self.proxy
                .get_service_health("discovery")
                .await
                .unwrap_or(false),
        );
        services.insert(
            "sona".to_string(),
            self.proxy.get_service_health("sona").await.unwrap_or(false),
        );
        services.insert(
            "sync".to_string(),
            self.proxy.get_service_health("sync").await.unwrap_or(false),
        );
        services.insert(
            "auth".to_string(),
            self.proxy.get_service_health("auth").await.unwrap_or(false),
        );
        services.insert(
            "playback".to_string(),
            self.proxy
                .get_service_health("playback")
                .await
                .unwrap_or(false),
        );

        let ready = services.values().all(|&v| v);

        ReadinessResponse { ready, services }
    }

    async fn check_service(&self, service: &str) -> ServiceHealthCheck {
        let health = self
            .proxy
            .get_service_health(service)
            .await
            .unwrap_or(false);
        let circuit_state = self.circuit_breaker.get_state(service).await;

        ServiceHealthCheck {
            status: if health { "healthy" } else { "unhealthy" }.to_string(),
            circuit_breaker: circuit_state,
        }
    }
}

pub async fn health(checker: web::Data<HealthChecker>) -> impl Responder {
    let health = checker.health_check().await;
    let status_code = if health.status == "healthy" {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    HttpResponse::build(status_code).json(health)
}

pub async fn readiness(checker: web::Data<HealthChecker>) -> impl Responder {
    let readiness = checker.readiness_check().await;
    let status_code = if readiness.ready {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    HttpResponse::build(status_code).json(readiness)
}

pub async fn liveness() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "alive"
    }))
}

pub async fn aggregate(aggregator: web::Data<HealthAggregator>) -> impl Responder {
    match aggregator.check_health().await {
        Ok(health) => {
            let status_code = match health.status {
                HealthStatus::Healthy => actix_web::http::StatusCode::OK,
                HealthStatus::Degraded => actix_web::http::StatusCode::OK,
                HealthStatus::Unhealthy => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
            };
            HttpResponse::build(status_code).json(health)
        }
        Err(e) => {
            tracing::error!("Health aggregation failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "unhealthy",
                "error": e.to_string()
            }))
        }
    }
}
