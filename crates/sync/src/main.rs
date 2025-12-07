/// Media Gateway Sync Service - Main Entry Point
///
/// Starts the sync server on port 8083
use media_gateway_sync::{init_tracing, start_server};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    init_tracing();

    // Server configuration
    let host = std::env::var("SYNC_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("SYNC_PORT")
        .unwrap_or_else(|_| "8083".to_string())
        .parse()
        .expect("SYNC_PORT must be a valid port number");

    tracing::info!(
        "🚀 Media Gateway Sync Service starting on {}:{}",
        host,
        port
    );

    // Start server
    start_server(&host, port).await
}
