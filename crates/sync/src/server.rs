/// Actix-web HTTP server for sync service
///
/// Port: 8083
/// Endpoints:
/// - GET /health - Health check
/// - WebSocket /ws - Real-time sync connection
/// - POST /api/v1/sync/watchlist - Sync watchlist
/// - POST /api/v1/sync/progress - Sync watch progress
/// - GET /api/v1/devices - List user devices
/// - POST /api/v1/devices/handoff - Device handoff
use crate::crdt::{HybridLogicalClock, PlaybackState};
use crate::device::{DeviceHandoff, DeviceRegistry};
use crate::sync::{ProgressSync, WatchlistSync};
use crate::websocket::SyncWebSocket;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Server state shared across handlers
pub struct ServerState {
    /// User ID (in production, extract from JWT)
    pub user_id: String,

    /// Device ID (in production, extract from request)
    pub device_id: String,

    /// Watchlist sync manager
    pub watchlist_sync: Arc<WatchlistSync>,

    /// Progress sync manager
    pub progress_sync: Arc<ProgressSync>,

    /// Device registry
    pub device_registry: Arc<DeviceRegistry>,

    /// HLC for timestamp generation
    pub hlc: Arc<HybridLogicalClock>,
}

impl ServerState {
    pub fn new(user_id: String, device_id: String) -> Self {
        Self {
            user_id: user_id.clone(),
            device_id: device_id.clone(),
            watchlist_sync: Arc::new(WatchlistSync::new(user_id.clone(), device_id.clone())),
            progress_sync: Arc::new(ProgressSync::new(user_id.clone(), device_id.clone())),
            device_registry: Arc::new(DeviceRegistry::new(user_id.clone())),
            hlc: Arc::new(HybridLogicalClock::new()),
        }
    }
}

/// Health check endpoint
#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "media-gateway-sync",
        "version": "0.1.0"
    }))
}

/// WebSocket connection endpoint
#[get("/ws")]
async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<ServerState>,
) -> Result<HttpResponse> {
    let ws_session = SyncWebSocket::new(state.user_id.clone(), state.device_id.clone());
    ws::start(ws_session, &req, stream)
}

/// Sync watchlist endpoint
#[post("/api/v1/sync/watchlist")]
async fn sync_watchlist(
    req: web::Json<WatchlistSyncRequest>,
    state: web::Data<ServerState>,
) -> impl Responder {
    let response = match req.operation.as_str() {
        "add" => {
            let update = state
                .watchlist_sync
                .add_to_watchlist(req.content_id.clone());
            WatchlistSyncResponse {
                success: true,
                operation: "add".to_string(),
                content_id: update.content_id,
                timestamp: update.timestamp,
            }
        }
        "remove" => {
            let updates = state.watchlist_sync.remove_from_watchlist(&req.content_id);
            if updates.is_empty() {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Content not found in watchlist"
                }));
            }
            WatchlistSyncResponse {
                success: true,
                operation: "remove".to_string(),
                content_id: req.content_id.clone(),
                timestamp: updates[0].timestamp,
            }
        }
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid operation, must be 'add' or 'remove'"
            }));
        }
    };

    HttpResponse::Ok().json(response)
}

/// Sync watch progress endpoint
#[post("/api/v1/sync/progress")]
async fn sync_progress(
    req: web::Json<ProgressSyncRequest>,
    state: web::Data<ServerState>,
) -> impl Responder {
    let playback_state = match req.state.as_str() {
        "playing" => PlaybackState::Playing,
        "paused" => PlaybackState::Paused,
        "stopped" => PlaybackState::Stopped,
        _ => PlaybackState::Paused,
    };

    let update = state.progress_sync.update_progress(
        req.content_id.clone(),
        req.position_seconds,
        req.duration_seconds,
        playback_state,
    );

    let completion_percent = update.completion_percent();
    let response = ProgressSyncResponse {
        success: true,
        content_id: update.content_id,
        position_seconds: update.position_seconds,
        completion_percent,
        timestamp: update.timestamp,
    };

    HttpResponse::Ok().json(response)
}

/// List user devices endpoint
#[get("/api/v1/devices")]
async fn list_devices(state: web::Data<ServerState>) -> impl Responder {
    let devices = state.device_registry.get_all_devices();

    let response = DevicesListResponse {
        devices: devices
            .iter()
            .map(|d| DeviceResponse {
                device_id: d.device_id.clone(),
                device_type: format!("{:?}", d.device_type),
                platform: format!("{:?}", d.platform),
                is_online: d.is_online,
                last_seen: d.last_seen.to_rfc3339(),
                device_name: d.device_name.clone(),
            })
            .collect(),
        total: devices.len(),
    };

    HttpResponse::Ok().json(response)
}

/// Device handoff endpoint
#[post("/api/v1/devices/handoff")]
async fn device_handoff(
    req: web::Json<DeviceHandoffRequest>,
    state: web::Data<ServerState>,
) -> impl Responder {
    // Check if target device exists
    let target_device = state.device_registry.get_device(&req.target_device_id);
    if target_device.is_none() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Target device not found"
        }));
    }

    let target = target_device.unwrap();
    if !target.is_online {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Target device is offline"
        }));
    }

    // Get current progress for content
    let position = state
        .progress_sync
        .get_resume_position(&req.content_id)
        .unwrap_or(0);

    let handoff = DeviceHandoff {
        source_device_id: state.device_id.clone(),
        target_device_id: req.target_device_id.clone(),
        content_id: req.content_id.clone(),
        position_seconds: Some(position),
        timestamp: state.hlc.now(),
    };

    let response = DeviceHandoffResponse {
        success: true,
        target_device_id: handoff.target_device_id,
        content_id: handoff.content_id,
        position_seconds: handoff.position_seconds,
    };

    HttpResponse::Ok().json(response)
}

/// Request/Response types

#[derive(Debug, Serialize, Deserialize)]
pub struct WatchlistSyncRequest {
    pub operation: String,
    pub content_id: String,
}

#[derive(Debug, Serialize)]
pub struct WatchlistSyncResponse {
    pub success: bool,
    pub operation: String,
    pub content_id: String,
    pub timestamp: crate::crdt::HLCTimestamp,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressSyncRequest {
    pub content_id: String,
    pub position_seconds: u32,
    pub duration_seconds: u32,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct ProgressSyncResponse {
    pub success: bool,
    pub content_id: String,
    pub position_seconds: u32,
    pub completion_percent: f32,
    pub timestamp: crate::crdt::HLCTimestamp,
}

#[derive(Debug, Serialize)]
pub struct DevicesListResponse {
    pub devices: Vec<DeviceResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub device_id: String,
    pub device_type: String,
    pub platform: String,
    pub is_online: bool,
    pub last_seen: String,
    pub device_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceHandoffRequest {
    pub target_device_id: String,
    pub content_id: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceHandoffResponse {
    pub success: bool,
    pub target_device_id: String,
    pub content_id: String,
    pub position_seconds: Option<u32>,
}

/// Start the sync server
pub async fn start_server(host: &str, port: u16) -> std::io::Result<()> {
    tracing::info!("Starting Media Gateway Sync Service on {}:{}", host, port);

    // Initialize server state (in production, this would be per-user)
    let state = web::Data::new(ServerState::new(
        "demo-user".to_string(),
        "demo-device".to_string(),
    ));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(health_check)
            .service(websocket)
            .service(sync_watchlist)
            .service(sync_progress)
            .service(list_devices)
            .service(device_handoff)
    })
    .bind((host, port))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(App::new().service(health_check)).await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_sync_watchlist() {
        let state = web::Data::new(ServerState::new(
            "test-user".to_string(),
            "test-device".to_string(),
        ));

        let app =
            test::init_service(App::new().app_data(state.clone()).service(sync_watchlist)).await;

        let req = test::TestRequest::post()
            .uri("/api/v1/sync/watchlist")
            .set_json(WatchlistSyncRequest {
                operation: "add".to_string(),
                content_id: "content-1".to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_sync_progress() {
        let state = web::Data::new(ServerState::new(
            "test-user".to_string(),
            "test-device".to_string(),
        ));

        let app =
            test::init_service(App::new().app_data(state.clone()).service(sync_progress)).await;

        let req = test::TestRequest::post()
            .uri("/api/v1/sync/progress")
            .set_json(ProgressSyncRequest {
                content_id: "content-1".to_string(),
                position_seconds: 100,
                duration_seconds: 1000,
                state: "playing".to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
