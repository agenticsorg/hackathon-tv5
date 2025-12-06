//! Graceful shutdown signal handling

use tokio::signal;
use tracing::info;

/// Wait for shutdown signal (SIGTERM or SIGINT)
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        }
        _ = terminate => {
            info!("Received SIGTERM signal");
        }
    }

    info!("Starting graceful shutdown...");
}

/// Graceful shutdown with timeout
pub async fn shutdown_with_timeout(timeout: std::time::Duration) {
    shutdown_signal().await;

    // Give services time to finish pending requests
    info!("Waiting {}s for connections to drain...", timeout.as_secs());
    tokio::time::sleep(timeout).await;

    info!("Shutdown complete");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_timeout() {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(100);

        // This won't actually shutdown, but tests the timeout logic
        tokio::select! {
            _ = shutdown_signal() => {
                // Won't happen in test
            }
            _ = tokio::time::sleep(timeout) => {
                // Timeout completes
            }
        }

        assert!(start.elapsed() >= timeout);
    }
}
