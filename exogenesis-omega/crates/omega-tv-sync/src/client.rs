//! Sync client for TV-to-Constellation communication
//!
//! Provides a high-level interface for delta synchronization.

use crate::{protocol::ConstellationProtocol, Error, GlobalPatterns, PatternData, Result, SyncDelta};
use tracing::{debug, info, warn};

/// Sync client for TV devices
///
/// Manages delta preparation, version tracking, and communication with the
/// Constellation server.
///
/// # Example
///
/// ```no_run
/// use omega_tv_sync::{SyncClient, PatternData};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let mut client = SyncClient::new(
///     "https://constellation.omega.io",
///     "tv-device-123"
/// )?;
///
/// // Prepare patterns from local AgentDB skills
/// let skills = vec![/* ... */];
/// let delta = client.prepare_delta(&skills)?;
///
/// // Sync with constellation
/// let global = client.sync(delta).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct SyncClient {
    /// Base URL of constellation server
    constellation_url: String,

    /// Unique device identifier
    device_id: String,

    /// HTTP protocol handler
    protocol: ConstellationProtocol,

    /// Last known sync version (monotonic counter)
    last_sync_version: u64,
}

impl SyncClient {
    /// Create a new sync client
    ///
    /// # Arguments
    /// * `url` - Constellation server URL (e.g., "https://constellation.omega.io")
    /// * `device_id` - Unique device identifier (e.g., "tv-device-123")
    ///
    /// # Returns
    /// A new sync client ready for use
    ///
    /// # Errors
    /// - `Error::InvalidConfig` if URL or device_id is invalid
    /// - `Error::Http` if HTTP client creation fails
    pub fn new(url: &str, device_id: &str) -> Result<Self> {
        if url.is_empty() {
            return Err(Error::InvalidConfig("url cannot be empty".to_string()));
        }

        if device_id.is_empty() {
            return Err(Error::InvalidConfig(
                "device_id cannot be empty".to_string(),
            ));
        }

        let protocol = ConstellationProtocol::new(url)?;

        Ok(Self {
            constellation_url: url.to_string(),
            device_id: device_id.to_string(),
            protocol,
            last_sync_version: 0,
        })
    }

    /// Check if constellation is reachable
    ///
    /// # Returns
    /// `Ok(())` if server is healthy, error otherwise
    pub async fn health_check(&self) -> Result<()> {
        self.protocol.health_check().await
    }

    /// Perform synchronization with constellation
    ///
    /// # Arguments
    /// * `delta` - Sync delta containing local patterns
    ///
    /// # Returns
    /// Global patterns received from constellation
    ///
    /// # Side Effects
    /// Updates `last_sync_version` on successful sync
    pub async fn sync(&mut self, delta: SyncDelta) -> Result<GlobalPatterns> {
        // Validate delta before sending
        delta.validate()?;

        debug!(
            "Syncing {} patterns (version {})",
            delta.patterns.len(),
            delta.version
        );

        // Perform sync
        let (global, result) = self.protocol.sync(delta).await?;

        // Update version tracker
        self.last_sync_version = global.version;

        info!(
            "Sync complete: pushed {}, received {} patterns, {} trends (version {})",
            result.patterns_pushed,
            result.patterns_received,
            result.trends_received,
            global.version
        );

        debug!(
            "Bandwidth: {} bytes sent, {} bytes received",
            result.bytes_sent, result.bytes_received
        );

        Ok(global)
    }

    /// Prepare a sync delta from local skills/patterns
    ///
    /// This method would typically be called with skills from omega-agentdb.
    /// For now, it accepts a generic slice of patterns and filters for quality.
    ///
    /// # Arguments
    /// * `patterns` - Local patterns to consider for sync
    ///
    /// # Returns
    /// A sync delta ready for transmission
    ///
    /// # Quality Filtering
    /// Only patterns with:
    /// - `success_rate >= 0.7`
    /// - `sample_count >= 10`
    /// are included in the delta.
    pub fn prepare_delta(&self, patterns: &[PatternData]) -> Result<SyncDelta> {
        debug!("Preparing delta from {} patterns", patterns.len());

        // Filter for high-quality patterns
        let filtered: Vec<PatternData> = patterns
            .iter()
            .filter(|p| p.meets_sync_threshold())
            .cloned()
            .collect();

        if filtered.is_empty() {
            warn!("No patterns meet sync threshold after filtering");
        } else {
            info!(
                "Filtered {} → {} patterns for sync",
                patterns.len(),
                filtered.len()
            );
        }

        // Increment version
        let version = self.last_sync_version + 1;

        let delta = SyncDelta::new(self.device_id.clone(), filtered, version);

        // Validate before returning
        delta.validate()?;

        Ok(delta)
    }

    /// Get the last known sync version
    pub fn last_version(&self) -> u64 {
        self.last_sync_version
    }

    /// Get device ID
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// Get constellation URL
    pub fn constellation_url(&self) -> &str {
        &self.constellation_url
    }

    /// Check if sync is needed by comparing local and remote versions
    ///
    /// # Returns
    /// `true` if remote version is newer than local
    pub async fn needs_sync(&self) -> Result<bool> {
        let remote_version = self.protocol.get_global_version().await?;
        Ok(remote_version > self.last_sync_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = SyncClient::new("https://constellation.omega.io", "tv-123").unwrap();
        assert_eq!(client.device_id(), "tv-123");
        assert_eq!(client.last_version(), 0);
    }

    #[test]
    fn test_client_creation_validation() {
        let err = SyncClient::new("", "tv-123").unwrap_err();
        assert!(matches!(err, Error::InvalidConfig(_)));

        let err = SyncClient::new("https://example.com", "").unwrap_err();
        assert!(matches!(err, Error::InvalidConfig(_)));
    }

    #[test]
    fn test_prepare_delta_filtering() {
        let client = SyncClient::new("https://constellation.omega.io", "tv-123").unwrap();

        let patterns = vec![
            PatternData::new(
                "high-quality".to_string(),
                vec![0.1; 384],
                0.9,
                100,
                "action".to_string(),
            ),
            PatternData::new(
                "low-quality".to_string(),
                vec![0.2; 384],
                0.5, // too low
                100,
                "comedy".to_string(),
            ),
            PatternData::new(
                "low-samples".to_string(),
                vec![0.3; 384],
                0.85,
                5, // too few samples
                "drama".to_string(),
            ),
        ];

        let delta = client.prepare_delta(&patterns).unwrap();

        // Only first pattern should pass filter
        assert_eq!(delta.patterns.len(), 1);
        assert_eq!(delta.patterns[0].id, "high-quality");
        assert_eq!(delta.version, 1); // version incremented
    }

    #[test]
    fn test_version_tracking() {
        let mut client = SyncClient::new("https://constellation.omega.io", "tv-123").unwrap();
        assert_eq!(client.last_version(), 0);

        // Simulate version updates
        client.last_sync_version = 5;
        assert_eq!(client.last_version(), 5);

        let delta = client.prepare_delta(&[]).unwrap();
        assert_eq!(delta.version, 6); // incremented from last_version
    }
}
