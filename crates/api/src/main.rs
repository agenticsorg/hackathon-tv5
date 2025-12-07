//! API Gateway - HTTP Gateway and Request Router
//!
//! Port: 8080
//! SLA: 99.9% availability
//! Latency target: <100ms p95

use actix_web::{web, App, HttpResponse, HttpServer};
use media_gateway_api::routes;
use media_gateway_core::{metrics_handler, MetricsMiddleware};
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    // Validate JWT_SECRET is set before starting server
    std::env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable must be set - cannot start with default secret");

    info!("Starting API Gateway on port 8080");

    HttpServer::new(|| {
        App::new()
            .wrap(MetricsMiddleware)
            .route("/health", web::get().to(health_check))
            .route("/metrics", web::get().to(metrics_handler))
            .route("/api/v1/status", web::get().to(api_status))
            .configure(routes::configure)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "api-gateway",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn api_status() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "api_version": "v1",
        "platform": "Media Gateway",
        "status": "operational"
    }))
}
