//! Extended REST API handlers for constellation server
//!
//! This module adds server-specific handlers beyond the base omega-constellation REST API

use crate::{config::ServerConfig, health::HealthChecker};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use omega_constellation::ShardManager;
use serde::Serialize;
use std::sync::Arc;

/// Extended application state
#[derive(Clone)]
pub struct ExtendedState {
    pub shard: Arc<ShardManager>,
    pub health_checker: Option<Arc<HealthChecker>>,
    pub config: ServerConfig,
}

/// Create extended router with custom handlers
pub fn create_extended_router(
    constellation_router: Router,
    shard: Arc<ShardManager>,
    health_checker: Option<Arc<HealthChecker>>,
    config: ServerConfig,
) -> Router {
    let state = ExtendedState {
        shard,
        health_checker,
        config,
    };

    // Create separate router for extended endpoints with their own state
    let extended_router = Router::new()
        .route("/health/ready", get(readiness_handler))
        .route("/health/live", get(liveness_handler))
        .route("/health/detailed", get(detailed_health_handler))
        .route("/api/v1/info", get(info_handler))
        .with_state(state);

    // Merge routers using nest (preserves separate states)
    Router::new()
        .nest("/", constellation_router)
        .nest("/", extended_router)
}

/// Readiness probe (for Kubernetes)
async fn readiness_handler(State(state): State<ExtendedState>) -> impl IntoResponse {
    // Check if we have a health checker and if it's healthy
    if let Some(checker) = &state.health_checker {
        let is_ready = checker.is_healthy().await;
        if is_ready {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        }
    } else {
        // No database configured, just check if shard is operational
        StatusCode::OK
    }
}

/// Liveness probe (for Kubernetes)
async fn liveness_handler() -> impl IntoResponse {
    // Server is alive if it can respond
    StatusCode::OK
}

/// Detailed health check
async fn detailed_health_handler(
    State(state): State<ExtendedState>,
) -> Json<DetailedHealthResponse> {
    let mut components = Vec::new();

    // Check shard health
    let shard_stats = state.shard.get_stats().await;
    components.push(ComponentHealth {
        name: "shard".to_string(),
        status: "healthy".to_string(),
        details: serde_json::json!({
            "shard_id": shard_stats.shard_id,
            "total_devices": shard_stats.total_devices,
            "active_devices": shard_stats.active_devices,
            "patterns_stored": shard_stats.patterns_stored,
        }),
    });

    // Check database health if configured
    if let Some(checker) = &state.health_checker {
        let db_health = checker.check().await;
        components.push(ComponentHealth {
            name: "database".to_string(),
            status: format!("{:?}", db_health.status).to_lowercase(),
            details: serde_json::to_value(&db_health).unwrap_or_default(),
        });
    }

    let overall_status = if components.iter().all(|c| c.status == "healthy") {
        "healthy"
    } else if components.iter().any(|c| c.status == "unhealthy") {
        "unhealthy"
    } else {
        "degraded"
    };

    Json(DetailedHealthResponse {
        status: overall_status.to_string(),
        components,
        timestamp: chrono::Utc::now().timestamp(),
    })
}

/// Server info endpoint
async fn info_handler(State(state): State<ExtendedState>) -> Json<ServerInfo> {
    let shard_stats = state.shard.get_stats().await;

    Json(ServerInfo {
        name: "Exogenesis Omega Constellation Server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        shard_id: state.config.shard_id,
        region: state.config.region.clone(),
        max_devices: state.config.max_devices,
        quality_threshold: state.config.quality_threshold,
        current_devices: shard_stats.total_devices,
        active_devices: shard_stats.active_devices,
        patterns_stored: shard_stats.patterns_stored,
    })
}

// Response types

#[derive(Debug, Serialize)]
struct DetailedHealthResponse {
    status: String,
    components: Vec<ComponentHealth>,
    timestamp: i64,
}

#[derive(Debug, Serialize)]
struct ComponentHealth {
    name: String,
    status: String,
    details: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ServerInfo {
    name: String,
    version: String,
    shard_id: u32,
    region: String,
    max_devices: usize,
    quality_threshold: f32,
    current_devices: usize,
    active_devices: usize,
    patterns_stored: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_constellation::{InMemoryStorage, ShardConfig};

    #[tokio::test]
    async fn test_liveness_always_ok() {
        let response = liveness_handler().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_readiness_without_db() {
        let storage = Arc::new(InMemoryStorage::new());
        let shard = Arc::new(
            ShardManager::new(ShardConfig::default(), storage)
                .await
                .unwrap(),
        );
        let state = ExtendedState {
            shard,
            health_checker: None,
            config: ServerConfig::from_env().unwrap(),
        };

        let response = readiness_handler(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
