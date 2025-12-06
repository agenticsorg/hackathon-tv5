//! REST management API for constellation operations

use crate::{DeviceId, GlobalPatterns, PatternDelta, ShardManager, ShardStats};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

/// REST API state
#[derive(Clone)]
pub struct RestState {
    pub shard: Arc<ShardManager>,
}

/// Create REST API router
pub fn create_rest_router(shard: Arc<ShardManager>) -> Router {
    let state = RestState { shard };

    Router::new()
        .route("/health", get(health_handler))
        .route("/api/v1/sync", post(sync_handler))
        .route("/api/v1/stats", get(stats_handler))
        .route("/api/v1/shards", get(shards_handler))
        .route("/api/v1/content", post(content_handler))
        .with_state(state)
}

/// Health check handler
async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    })
}

/// Handle device sync request
async fn sync_handler(
    State(state): State<RestState>,
    Json(payload): Json<SyncRequest>,
) -> impl IntoResponse {
    info!("Handling sync request from device {}", payload.device_id);

    match state
        .shard
        .handle_sync(payload.device_id, payload.delta)
        .await
    {
        Ok(global) => (StatusCode::OK, Json(SyncResponse { global })).into_response(),
        Err(e) => {
            tracing::error!("Sync failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Sync failed: {}", e),
                }),
            )
                .into_response()
        }
    }
}

/// Get shard statistics
async fn stats_handler(State(state): State<RestState>) -> impl IntoResponse {
    let stats = state.shard.get_stats().await;
    Json(stats)
}

/// List all shards (for now, just this one)
async fn shards_handler(State(state): State<RestState>) -> impl IntoResponse {
    let stats = state.shard.get_stats().await;

    let shards = vec![ShardInfo {
        id: state.shard.shard_id(),
        region: "default".to_string(), // TODO: Get from config
        devices: stats.total_devices,
        status: if stats.active_devices > 0 {
            "healthy"
        } else {
            "idle"
        }
        .to_string(),
    }];

    Json(shards)
}

/// Add new content embedding
async fn content_handler(
    State(state): State<RestState>,
    Json(payload): Json<NewContentRequest>,
) -> impl IntoResponse {
    info!("Adding new content: {}", payload.content_id);

    // TODO: Store content embedding in storage
    // For now, just acknowledge

    (
        StatusCode::CREATED,
        Json(ContentResponse {
            content_id: payload.content_id,
            status: "accepted".to_string(),
        }),
    )
}

// Request/Response types

#[derive(Debug, Deserialize)]
struct SyncRequest {
    device_id: DeviceId,
    delta: PatternDelta,
}

#[derive(Debug, Serialize)]
struct SyncResponse {
    global: GlobalPatterns,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    timestamp: i64,
}

#[derive(Debug, Serialize)]
struct ShardInfo {
    id: u32,
    region: String,
    devices: usize,
    status: String,
}

#[derive(Debug, Deserialize)]
struct NewContentRequest {
    content_id: String,
    title: String,
    embedding: Vec<f32>,
    #[serde(default)]
    metadata: Option<ContentMetadata>,
}

#[derive(Debug, Deserialize)]
struct ContentMetadata {
    genre: Option<String>,
    year: Option<i32>,
    duration_min: Option<i32>,
}

#[derive(Debug, Serialize)]
struct ContentResponse {
    content_id: String,
    status: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{storage::InMemoryStorage, ShardConfig};
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let storage = Arc::new(InMemoryStorage::new());
        let shard = Arc::new(
            ShardManager::new(ShardConfig::default(), storage)
                .await
                .unwrap(),
        );
        let app = create_rest_router(shard);

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_stats_endpoint() {
        let storage = Arc::new(InMemoryStorage::new());
        let shard = Arc::new(
            ShardManager::new(ShardConfig::default(), storage)
                .await
                .unwrap(),
        );
        let app = create_rest_router(shard);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
