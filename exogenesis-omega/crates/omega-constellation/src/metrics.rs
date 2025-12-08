//! Prometheus metrics for constellation monitoring

use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

/// Global flag to track if metrics have been registered
pub static METRICS_REGISTERED: AtomicBool = AtomicBool::new(false);

/// Register all Prometheus metrics
pub fn register_metrics() {
    if METRICS_REGISTERED.swap(true, Ordering::SeqCst) {
        return; // Already registered
    }

    info!("Registering Prometheus metrics");

    // These will be automatically registered when first used
    // We just document them here for reference
}

/// Start Prometheus metrics exporter on the given address
pub async fn start_metrics_server(addr: SocketAddr) -> std::io::Result<()> {
    info!("Starting Prometheus metrics server on {}", addr);

    PrometheusBuilder::new()
        .with_http_listener(addr)
        .install()
        .expect("Failed to install Prometheus exporter");

    // Register metrics
    register_metrics();

    // Metrics are now available at http://<addr>/metrics
    Ok(())
}

/// Record a sync request
pub fn record_sync_request(shard_id: u32, latency_ms: f64, success: bool) {
    counter!("constellation_sync_requests_total",
        "shard_id" => shard_id.to_string(),
        "status" => if success { "success" } else { "failure" }
    )
    .increment(1);

    if success {
        histogram!("constellation_sync_latency_seconds",
            "shard_id" => shard_id.to_string()
        )
        .record(latency_ms / 1000.0);
    }
}

/// Record patterns stored
pub fn record_patterns_stored(shard_id: u32, count: usize) {
    counter!("constellation_patterns_stored_total",
        "shard_id" => shard_id.to_string()
    )
    .increment(count as u64);
}

/// Update active devices gauge
pub fn update_active_devices(shard_id: u32, count: usize) {
    gauge!("constellation_active_devices",
        "shard_id" => shard_id.to_string()
    )
    .set(count as f64);
}

/// Update total devices gauge
pub fn update_total_devices(shard_id: u32, count: usize) {
    gauge!("constellation_total_devices",
        "shard_id" => shard_id.to_string()
    )
    .set(count as f64);
}

/// Record federation round
pub fn record_federation_round(patterns_aggregated: usize, trends_detected: usize, duration_ms: f64) {
    counter!("constellation_federation_rounds_total").increment(1);

    gauge!("constellation_federation_patterns_aggregated").set(patterns_aggregated as f64);

    gauge!("constellation_federation_trends_detected").set(trends_detected as f64);

    histogram!("constellation_federation_duration_seconds").record(duration_ms / 1000.0);
}

/// Record shard health
pub fn update_shard_health(shard_id: u32, health: f64) {
    gauge!("constellation_shard_health",
        "shard_id" => shard_id.to_string()
    )
    .set(health);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registration() {
        register_metrics();
        assert!(METRICS_REGISTERED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_record_sync_request() {
        record_sync_request(1, 150.0, true);
        record_sync_request(1, 200.0, false);
    }

    #[test]
    fn test_record_patterns() {
        record_patterns_stored(1, 100);
    }

    #[test]
    fn test_update_devices() {
        update_active_devices(1, 1000);
        update_total_devices(1, 5000);
    }
}
