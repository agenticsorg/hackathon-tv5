//! Omega TV Sync - Delta synchronization protocol for TV-to-Constellation communication
//!
//! This crate implements efficient delta synchronization between TV devices and the
//! Constellation server using omega-core types. It achieves:
//!
//! - ~1KB compressed delta uploads (TV patterns to Constellation)
//! - ~5KB compressed global pattern downloads (Constellation to TV)
//! - zstd compression for bandwidth efficiency
//! - HTTP/REST protocol for constellation API
//!
//! # Architecture
//!
//! ```text
//! TV Brain (AgentDB) → SyncDelta (~1KB compressed) → Constellation
//!                    ← GlobalPatterns (~5KB)      ←
//! ```
//!
//! # Example
//!
//! ```no_run
//! use omega_tv_sync::{SyncClient, SyncDelta, PatternData};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut client = SyncClient::new(
//!         "https://constellation.omega.io",
//!         "tv-device-123"
//!     )?;
//!
//!     // Prepare delta from local patterns
//!     let patterns = vec![
//!         PatternData {
//!             id: "pattern-1".to_string(),
//!             embedding: vec![0.1; 384],
//!             success_rate: 0.85,
//!             sample_count: 100,
//!             genre: "action".to_string(),
//!         }
//!     ];
//!
//!     let delta = SyncDelta {
//!         device_id: "tv-device-123".to_string(),
//!         patterns,
//!         version: 1,
//!         timestamp: chrono::Utc::now(),
//!     };
//!
//!     // Sync with constellation
//!     let global = client.sync(delta).await?;
//!     println!("Received {} global patterns", global.patterns.len());
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod compression;
pub mod protocol;
pub mod types;

// Re-export main types
pub use client::SyncClient;
pub use compression::{compress, decompress};
pub use protocol::{ConstellationProtocol, SyncResult};
pub use types::{GlobalPatterns, PatternData, SyncDelta, TrendSignal};

/// Result type for omega-tv-sync operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for omega-tv-sync
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Compression error: {0}")]
    Compression(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Version conflict: local={0}, remote={1}")]
    VersionConflict(u64, u64),

    #[error("Compression ratio too high: {0} bytes (limit: {1})")]
    CompressionLimit(usize, usize),

    #[error("Invalid pattern data: {0}")]
    InvalidPattern(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_types() {
        let err = Error::InvalidConfig("missing url".to_string());
        assert_eq!(err.to_string(), "Invalid configuration: missing url");

        let err = Error::VersionConflict(5, 10);
        assert_eq!(err.to_string(), "Version conflict: local=5, remote=10");
    }
}
