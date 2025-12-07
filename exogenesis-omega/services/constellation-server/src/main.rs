//! Constellation Server - Main server binary for Exogenesis Omega
//!
//! This server handles:
//! - gRPC sync service for TV devices (port 50051)
//! - REST management API (port 8080)
//! - Prometheus metrics (port 9090)

mod config;
mod handlers;
mod health;
mod shutdown;

use config::ServerConfig;
use handlers::create_extended_router;
use health::{HealthChecker, HealthStatus};
use metrics_exporter_prometheus::PrometheusBuilder;
use omega_constellation::{
    create_rest_router, register_metrics, InMemoryStorage, RestState, ShardConfig, ShardManager,
    SyncService,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info,constellation_server=debug,omega_constellation=debug")
    });

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .json()
        .init();

    info!("Starting Exogenesis Omega Constellation Server");

    // Load configuration
    let config = ServerConfig::from_env()?;
    info!(
        "Configuration loaded: shard_id={}, region={}",
        config.shard_id, config.region
    );

    // Register Prometheus metrics
    register_metrics();

    // Start Prometheus metrics exporter
    let metrics_addr: std::net::SocketAddr = config.metrics_addr.parse()
        .map_err(|e| anyhow::anyhow!("Invalid metrics address: {}", e))?;
    info!("Starting Prometheus metrics server on {}", metrics_addr);
    PrometheusBuilder::new()
        .with_http_listener(metrics_addr)
        .install()?;

    metrics::counter!("constellation_server_starts_total",
        "shard_id" => config.shard_id.to_string(),
        "region" => config.region.clone()
    )
    .increment(1);

    // Initialize storage
    // TODO: Replace with RuVectorStorage for production
    let storage = Arc::new(InMemoryStorage::new());
    info!("Using InMemoryStorage (replace with RuVectorStorage for production)");

    // Initialize shard manager
    let shard_config = ShardConfig {
        shard_id: config.shard_id,
        region: config.region.clone(),
        max_devices: config.max_devices,
        quality_threshold: config.quality_threshold,
    };
    let shard = Arc::new(ShardManager::new(shard_config, storage).await?);
    info!("ShardManager initialized for shard {}", config.shard_id);

    // Initialize health checker (for database if configured)
    let health_checker = if let Some(postgres_url) = &config.postgres_url {
        info!("Connecting to PostgreSQL: {}", mask_url(postgres_url));
        let pool = sqlx::PgPool::connect(postgres_url).await?;

        // Ensure RuVector extension is loaded
        sqlx::query("CREATE EXTENSION IF NOT EXISTS ruvector")
            .execute(&pool)
            .await?;
        info!("RuVector extension verified");

        let checker = Arc::new(HealthChecker::new(pool.clone()));

        // Initial health check
        let health = checker.check().await;
        if health.status == HealthStatus::Unhealthy {
            error!("Initial health check failed: {:?}", health);
            anyhow::bail!("Server is unhealthy, aborting startup");
        }
        info!("Initial health check passed");

        Some(checker)
    } else {
        info!("No PostgreSQL connection configured");
        None
    };

    // Build REST API using omega-constellation's router
    let constellation_router = create_rest_router(shard.clone());

    // Extend with custom handlers (health checks, info, etc.)
    let rest_app = create_extended_router(
        constellation_router,
        shard.clone(),
        health_checker,
        config.clone(),
    );

    // Start REST server
    let rest_addr: std::net::SocketAddr = config.rest_addr.parse()
        .map_err(|e| anyhow::anyhow!("Invalid REST address: {}", e))?;
    info!("Starting REST API server on {}", rest_addr);
    let rest_listener = TcpListener::bind(rest_addr).await?;

    // Initialize gRPC sync service
    let _sync_service = SyncService::new(shard.clone());
    info!("gRPC sync service initialized");
    // TODO: Start gRPC server when protobuf definitions are ready
    info!("Note: gRPC server will be started when protobuf definitions are implemented");

    // Run REST server with graceful shutdown
    info!("Constellation server is running!");
    info!("  - REST API: http://{}", rest_addr);
    info!("  - Metrics: http://{}", metrics_addr);
    info!("  - Shard: {} ({})", config.shard_id, config.region);

    axum::serve(rest_listener, rest_app)
        .with_graceful_shutdown(shutdown::shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// Mask sensitive parts of database URL for logging
fn mask_url(url: &str) -> String {
    if let Some(at_pos) = url.rfind('@') {
        if let Some(proto_end) = url.find("://") {
            let proto = &url[..proto_end + 3];
            let host = &url[at_pos + 1..];
            return format!("{}***@{}", proto, host);
        }
    }
    "***".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_url() {
        let url = "postgresql://user:password@localhost:5432/db";
        let masked = mask_url(url);
        assert!(!masked.contains("password"));
        assert!(masked.contains("localhost"));
    }
}
