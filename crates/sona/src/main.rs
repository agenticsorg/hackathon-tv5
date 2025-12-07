//! SONA Engine - AI-Powered Personalization and Recommendations
//!
//! Port: 8082
//! SLA: 99.9% availability
//! Latency target: <5ms personalization, <200ms recommendations

use actix_web::{web, App, HttpResponse, HttpServer};
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    info!("Starting SONA Engine on port 8082");

    HttpServer::new(|| App::new().route("/health", web::get().to(health_check)))
        .bind(("0.0.0.0", 8082))?
        .run()
        .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "sona-engine",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
