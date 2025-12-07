//! Ingestion Service - Platform Data Pipeline
//!
//! Port: 8085
//! SLA: 99.5% availability

use actix_web::{web, App, HttpResponse, HttpServer};
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    info!("Starting Ingestion Service on port 8085");

    HttpServer::new(|| App::new().route("/health", web::get().to(health_check)))
        .bind(("0.0.0.0", 8085))?
        .run()
        .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "ingestion-service",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
