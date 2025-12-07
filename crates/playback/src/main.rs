//! Playback Service - Device Management and Deep Linking
//!
//! Port: 8086
//! SLA: 99.5% availability

mod cleanup;
mod continue_watching;
mod events;
mod progress;
mod session;
mod watch_history;

use actix_web::{web, App, HttpResponse, HttpServer};
use continue_watching::{
    ContinueWatchingError, ContinueWatchingService, HttpContentMetadataProvider,
    ProgressUpdateRequest, SyncServiceClient,
};
use serde::Deserialize;
use session::{
    CreateSessionRequest, SessionError, SessionManager,
    UpdatePositionRequest,
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Application state
struct AppState {
    session_manager: Arc<SessionManager>,
    continue_watching: Arc<ContinueWatchingService>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    info!("Starting Playback Service on port 8086");

    let session_manager = SessionManager::from_env().expect("Failed to connect to Redis");

    // Initialize continue watching service
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/media_gateway".to_string()
    });

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let catalog_service_url = std::env::var("CATALOG_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8084".to_string());

    let sync_service_url =
        std::env::var("SYNC_SERVICE_URL").unwrap_or_else(|_| "http://localhost:8083".to_string());

    let metadata_provider: Arc<dyn continue_watching::ContentMetadataProvider> =
        Arc::new(HttpContentMetadataProvider::new(catalog_service_url));

    let sync_client = Arc::new(SyncServiceClient::new(sync_service_url));

    let continue_watching_service = Arc::new(
        ContinueWatchingService::new(pool, metadata_provider).with_sync_service(sync_client),
    );

    // Start background cleanup task
    let cleanup_service = continue_watching_service.clone();
    tokio::spawn(async move {
        cleanup::run_cleanup_task(cleanup_service).await;
    });

    let state = web::Data::new(AppState {
        session_manager: Arc::new(session_manager),
        continue_watching: continue_watching_service,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/health", web::get().to(health_check))
            .route("/ready", web::get().to(readiness_check))
            .service(
                web::scope("/api/v1")
                    .route("/sessions", web::post().to(create_session))
                    .route("/sessions/{id}", web::get().to(get_session))
                    .route("/sessions/{id}", web::delete().to(delete_session))
                    .route("/sessions/{id}/position", web::patch().to(update_position))
                    .route(
                        "/users/{user_id}/sessions",
                        web::get().to(get_user_sessions),
                    )
                    .route(
                        "/playback/continue-watching",
                        web::get().to(get_continue_watching),
                    )
                    .route("/playback/progress", web::post().to(update_progress)),
            )
    })
    .bind(("0.0.0.0", 8086))?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "playback-service",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn readiness_check(state: web::Data<AppState>) -> HttpResponse {
    let redis_healthy = state.session_manager.is_healthy().await;

    if redis_healthy {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "redis": "connected"
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "redis": "disconnected"
        }))
    }
}

async fn create_session(
    state: web::Data<AppState>,
    request: web::Json<CreateSessionRequest>,
) -> Result<HttpResponse, SessionError> {
    let response = state.session_manager.create(request.into_inner()).await?;
    Ok(HttpResponse::Created().json(response))
}

async fn get_session(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, SessionError> {
    let session_id = path.into_inner();
    let session = state
        .session_manager
        .get(session_id)
        .await?
        .ok_or(SessionError::NotFound)?;
    Ok(HttpResponse::Ok().json(session))
}

async fn delete_session(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, SessionError> {
    let session_id = path.into_inner();
    state.session_manager.delete(session_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn update_position(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    request: web::Json<UpdatePositionRequest>,
) -> Result<HttpResponse, SessionError> {
    let session_id = path.into_inner();
    let session = state
        .session_manager
        .update_position(session_id, request.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(session))
}

async fn get_user_sessions(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, SessionError> {
    let user_id = path.into_inner();
    let sessions = state.session_manager.get_user_sessions(user_id).await?;
    Ok(HttpResponse::Ok().json(sessions))
}

async fn get_continue_watching(
    state: web::Data<AppState>,
    query: web::Query<ContinueWatchingQuery>,
) -> Result<HttpResponse, ContinueWatchingError> {
    let user_id = query.user_id;
    let limit = query.limit;

    let response = state
        .continue_watching
        .get_continue_watching(user_id, limit)
        .await?;

    Ok(HttpResponse::Ok().json(response))
}

async fn update_progress(
    state: web::Data<AppState>,
    request: web::Json<ProgressUpdateRequestWithUser>,
) -> Result<HttpResponse, ContinueWatchingError> {
    let user_id = request.user_id;
    let progress_request = ProgressUpdateRequest {
        content_id: request.content_id,
        platform_id: request.platform_id.clone(),
        progress_seconds: request.progress_seconds,
        duration_seconds: request.duration_seconds,
        device_id: request.device_id,
    };

    let response = state
        .continue_watching
        .update_progress(user_id, progress_request)
        .await?;

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug, Deserialize)]
struct ContinueWatchingQuery {
    user_id: Uuid,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ProgressUpdateRequestWithUser {
    user_id: Uuid,
    content_id: Uuid,
    platform_id: String,
    progress_seconds: i32,
    duration_seconds: i32,
    device_id: Option<Uuid>,
}
