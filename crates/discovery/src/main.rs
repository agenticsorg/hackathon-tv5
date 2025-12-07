//! Discovery Service - Natural Language Search and Content Discovery
//!
//! Port: 8081
//! SLA: 99.9% availability
//! Latency target: <500ms p95

use actix_web::{web, App, HttpResponse, HttpServer};
use media_gateway_discovery::{catalog, config, server};
use qdrant_client::Qdrant;
use std::sync::Arc;
use tracing::info;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    info!("Starting Discovery Service on port 8081");

    // Load configuration
    let config = Arc::new(config::DiscoveryConfig::load()?);
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);

    info!("Discovery Service listening on {}", bind_addr);

    // Initialize service components
    let search_service = media_gateway_discovery::init_service(config.clone()).await?;

    // Initialize Qdrant client for catalog service
    let qdrant_client = Arc::new(
        Qdrant::from_url(&config.vector.qdrant_url)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create Qdrant client: {}", e))?,
    );

    // Initialize database pool for catalog service
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .acquire_timeout(std::time::Duration::from_secs(
            config.database.connect_timeout_sec,
        ))
        .connect(&config.database.url)
        .await?;

    // Initialize catalog service
    let mut catalog_service = catalog::CatalogService::new(
        db_pool,
        qdrant_client,
        config.vector.collection_name.clone(),
        config.embedding.api_key.clone(),
        config.embedding.api_url.clone(),
    );

    // Add Kafka support if configured
    if let Ok(kafka_brokers) = std::env::var("KAFKA_BROKERS") {
        info!("Enabling Kafka event publishing to {}", kafka_brokers);
        catalog_service = catalog_service.with_kafka(&kafka_brokers)?;
    }

    let catalog_service = Arc::new(catalog_service);

    // Get JWT secret from environment
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!("JWT_SECRET not set, using default (INSECURE for production)");
        "default-jwt-secret-change-in-production".to_string()
    });

    // Create application state
    let app_state = web::Data::new(server::AppState {
        config: config.clone(),
        search_service,
        ranking_store: None,
    });

    // Create catalog state
    let catalog_state = web::Data::new(catalog::CatalogState {
        catalog_service,
        jwt_secret,
    });

    // Start HTTP server with routes
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .app_data(catalog_state.clone())
            .route("/health", web::get().to(health_check))
            .route("/ready", web::get().to(readiness_check))
            .configure(server::configure_routes)
            .wrap(actix_web::middleware::Logger::default())
    })
    .workers(config.server.workers.unwrap_or_else(num_cpus::get))
    .bind(&bind_addr)?
    .run()
    .await?;

    Ok(())
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "discovery-service",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn readiness_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ready"
    }))
}
