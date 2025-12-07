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
    pub checks: HashMap<String, ServiceHealth>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceHealth {
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
    pub fn new(
        proxy: Arc<ServiceProxy>,
        circuit_breaker: Arc<CircuitBreakerManager>,
    ) -> Self {
        Self {
            proxy,
            circuit_breaker,
            start_time: std::time::Instant::now(),
        }
    }

    pub async fn health_check(&self) -> HealthResponse {
        let mut checks = HashMap::new();

        // Check discovery service
        checks.insert(
            "discovery".to_string(),
            self.check_service("discovery").await,
        );

        // Check SONA service
        checks.insert("sona".to_string(), self.check_service("sona").await);

        // Check sync service
        checks.insert("sync".to_string(), self.check_service("sync").await);

        // Check auth service
        checks.insert("auth".to_string(), self.check_service("auth").await);

        // Check playback service
        checks.insert("playback".to_string(), self.check_service("playback").await);

        // Overall status is healthy if at least one critical service is up
        let critical_services = ["discovery", "auth"];
        let status = if critical_services
            .iter()
            .any(|s| checks.get(*s).map(|h| h.status == "healthy").unwrap_or(false))
        {
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

        // Check all services
        services.insert(
            "discovery".to_string(),
            self.proxy.get_service_health("discovery").await.unwrap_or(false),
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
            self.proxy.get_service_health("playback").await.unwrap_or(false),
        );

        // Ready if all services are up
        let ready = services.values().all(|&v| v);

        ReadinessResponse { ready, services }
    }

    async fn check_service(&self, service: &str) -> ServiceHealth {
        let health = self.proxy.get_service_health(service).await.unwrap_or(false);
        let circuit_state = self.circuit_breaker.get_state(service).await;

        ServiceHealth {
            status: if health { "healthy" } else { "unhealthy" }.to_string(),
            circuit_breaker: circuit_state,
        }
    }
}

// Handler functions
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
