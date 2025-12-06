//! Server configuration from environment variables

use serde::{Deserialize, Serialize};
use std::env;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Shard ID (0-99)
    pub shard_id: u32,

    /// Region identifier (e.g., "us-east-1")
    pub region: String,

    /// gRPC server address
    pub grpc_addr: String,

    /// REST API server address
    pub rest_addr: String,

    /// Prometheus metrics server address
    pub metrics_addr: String,

    /// PostgreSQL connection URL (optional for in-memory mode)
    pub postgres_url: Option<String>,

    /// Maximum devices per shard
    pub max_devices: usize,

    /// Quality threshold for pattern storage (0.0-1.0)
    pub quality_threshold: f32,
}

impl ServerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            shard_id: env::var("SHARD_ID")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid SHARD_ID: {}", e))?,

            region: env::var("REGION")
                .unwrap_or_else(|_| "default".to_string()),

            grpc_addr: env::var("GRPC_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:50051".to_string()),

            rest_addr: env::var("REST_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),

            metrics_addr: env::var("METRICS_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:9090".to_string()),

            postgres_url: env::var("POSTGRES_URL").ok(),

            max_devices: env::var("MAX_DEVICES")
                .unwrap_or_else(|_| "4000000".to_string())
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid MAX_DEVICES: {}", e))?,

            quality_threshold: env::var("QUALITY_THRESHOLD")
                .unwrap_or_else(|_| "0.7".to_string())
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid QUALITY_THRESHOLD: {}", e))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = ServerConfig::from_env().unwrap();

        assert_eq!(config.shard_id, 0);
        assert_eq!(config.grpc_addr, "0.0.0.0:50051");
        assert_eq!(config.rest_addr, "0.0.0.0:8080");
        assert_eq!(config.metrics_addr, "0.0.0.0:9090");
        assert_eq!(config.max_devices, 4_000_000);
        assert!((config.quality_threshold - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_with_postgres_url() {
        env::set_var("POSTGRES_URL", "postgresql://localhost/test");

        let config = ServerConfig::from_env().unwrap();
        assert!(config.postgres_url.is_some());

        env::remove_var("POSTGRES_URL");
    }
}
