//! HTTP protocol implementation for Constellation API
//!
//! Defines the REST endpoints and protocol for TV-to-Constellation synchronization.

use crate::{compression, Error, GlobalPatterns, Result, SyncDelta};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info};

/// HTTP timeout for sync operations
const SYNC_TIMEOUT: Duration = Duration::from_secs(30);

/// Constellation API endpoints
const ENDPOINT_SYNC: &str = "/api/v1/sync";
const ENDPOINT_HEALTH: &str = "/api/v1/health";

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Number of patterns pushed to constellation
    pub patterns_pushed: usize,

    /// Number of patterns received from constellation
    pub patterns_received: usize,

    /// Number of trend signals received
    pub trends_received: usize,

    /// Global version after sync
    pub global_version: u64,

    /// Compressed payload size (bytes) sent
    pub bytes_sent: usize,

    /// Compressed payload size (bytes) received
    pub bytes_received: usize,
}

/// Constellation protocol handler
///
/// Manages HTTP communication with the Constellation server using
/// compressed delta synchronization.
pub struct ConstellationProtocol {
    /// HTTP client
    client: Client,

    /// Base URL of constellation server
    constellation_url: String,
}

impl std::fmt::Debug for ConstellationProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConstellationProtocol")
            .field("constellation_url", &self.constellation_url)
            .finish_non_exhaustive()
    }
}

impl ConstellationProtocol {
    /// Create a new constellation protocol handler
    ///
    /// # Arguments
    /// * `constellation_url` - Base URL of the constellation server (e.g., "https://constellation.omega.io")
    ///
    /// # Returns
    /// A new protocol handler ready for sync operations
    pub fn new(constellation_url: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(SYNC_TIMEOUT)
            .build()
            .map_err(|e| Error::Http(e))?;

        Ok(Self {
            client,
            constellation_url: constellation_url.trim_end_matches('/').to_string(),
        })
    }

    /// Check if constellation server is healthy
    ///
    /// # Returns
    /// `Ok(())` if server is healthy, error otherwise
    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}{}", self.constellation_url, ENDPOINT_HEALTH);

        debug!("Health check: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            error!("Health check failed: {}", response.status());
            return Err(Error::Protocol(format!(
                "Health check failed with status: {}",
                response.status()
            )));
        }

        info!("Constellation health check: OK");
        Ok(())
    }

    /// Perform synchronization with constellation
    ///
    /// # Arguments
    /// * `delta` - Local patterns to push
    ///
    /// # Returns
    /// Global patterns and trends received from constellation
    ///
    /// # Protocol
    /// 1. Compress delta using zstd (~1KB)
    /// 2. POST to /api/v1/sync with compressed payload
    /// 3. Receive compressed global patterns (~5KB)
    /// 4. Decompress and return
    pub async fn sync(&self, delta: SyncDelta) -> Result<(GlobalPatterns, SyncResult)> {
        let url = format!("{}{}", self.constellation_url, ENDPOINT_SYNC);

        debug!("Syncing with constellation: {}", url);
        debug!("Delta: {} patterns, version {}", delta.patterns.len(), delta.version);

        // Compress delta
        let compressed_delta = compression::compress(&delta)?;
        let bytes_sent = compressed_delta.len();

        info!(
            "Sending {} patterns ({} bytes compressed)",
            delta.patterns.len(),
            bytes_sent
        );

        // Send compressed delta
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .header("X-Device-ID", &delta.device_id)
            .header("X-Sync-Version", delta.version.to_string())
            .body(compressed_delta)
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Sync failed: {}", response.status());
            return Err(Error::Protocol(format!(
                "Sync failed with status: {}",
                response.status()
            )));
        }

        // Read compressed response
        let compressed_response = response.bytes().await?;
        let bytes_received = compressed_response.len();

        debug!("Received {} bytes compressed", bytes_received);

        // Decompress global patterns
        let global = compression::decompress(&compressed_response)?;

        info!(
            "Received {} patterns, {} trends (version {})",
            global.patterns.len(),
            global.trends.len(),
            global.version
        );

        let sync_result = SyncResult {
            patterns_pushed: delta.patterns.len(),
            patterns_received: global.patterns.len(),
            trends_received: global.trends.len(),
            global_version: global.version,
            bytes_sent,
            bytes_received,
        };

        Ok((global, sync_result))
    }

    /// Get current global version without syncing
    ///
    /// Useful for checking if sync is needed
    pub async fn get_global_version(&self) -> Result<u64> {
        let url = format!("{}{}/version", self.constellation_url, ENDPOINT_SYNC);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(Error::Protocol(format!(
                "Version check failed with status: {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct VersionResponse {
            version: u64,
        }

        let version_response: VersionResponse = response.json().await?;
        Ok(version_response.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_creation() {
        let proto = ConstellationProtocol::new("https://constellation.omega.io").unwrap();
        assert_eq!(proto.constellation_url, "https://constellation.omega.io");

        // Test URL normalization (trailing slash removal)
        let proto2 = ConstellationProtocol::new("https://constellation.omega.io/").unwrap();
        assert_eq!(proto2.constellation_url, "https://constellation.omega.io");
    }

    #[test]
    fn test_sync_result_serialization() {
        let result = SyncResult {
            patterns_pushed: 10,
            patterns_received: 50,
            trends_received: 5,
            global_version: 100,
            bytes_sent: 850,
            bytes_received: 4200,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SyncResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.patterns_pushed, 10);
        assert_eq!(deserialized.patterns_received, 50);
        assert_eq!(deserialized.global_version, 100);
    }

    // Integration tests with mock server would go here
    // For now, we'll skip actual HTTP tests as they require a running server
}
