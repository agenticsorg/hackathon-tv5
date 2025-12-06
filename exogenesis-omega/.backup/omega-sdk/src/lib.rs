//! # Omega SDK
//!
//! The official SDK for integrating Exogenesis Omega distributed TV recommendation system.
//!
//! ## Overview
//!
//! Omega SDK provides a simple, high-performance interface for TV manufacturers to integrate
//! intelligent, privacy-preserving recommendations. All inference happens locally on the TV,
//! with periodic pattern synchronization to the Omega Constellation.
//!
//! ## Key Features
//!
//! - **Sub-15ms Recommendations**: Local ONNX inference with RuVector search
//! - **Privacy-First**: Viewing data never leaves the TV
//! - **Minimal Bandwidth**: ~1KB push, ~5KB pull per sync (every 5-15 minutes)
//! - **Memory Efficient**: ~200MB total footprint
//! - **C FFI**: Native integration for C/C++ TV platforms
//!
//! ## Quick Start (Rust)
//!
//! ```no_run
//! use omega_sdk::OmegaBuilder;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the Omega Brain
//!     let mut brain = OmegaBuilder::new()
//!         .model_path("/data/omega/model.onnx")
//!         .storage_path("/data/omega/brain.db")
//!         .constellation_url("https://constellation.omega.tv")
//!         .build()
//!         .await?;
//!
//!     // Get recommendations
//!     let context = r#"{"time":"evening","mood":"relaxed"}"#;
//!     let recommendations = brain.recommend(context).await?;
//!
//!     // Record viewing event
//!     let event = r#"{"content_id":"movie123","watch_pct":0.95}"#;
//!     brain.observe(event).await?;
//!
//!     // Sync with constellation (call every 5-15 minutes)
//!     brain.sync().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Quick Start (C)
//!
//! ```c
//! #include "omega_sdk.h"
//!
//! int main() {
//!     // Initialize
//!     int result = omega_init(
//!         "/data/omega/model.onnx",
//!         "/data/omega/brain.db"
//!     );
//!     if (result != 0) {
//!         fprintf(stderr, "Init failed: %s\n", omega_get_last_error());
//!         return 1;
//!     }
//!
//!     // Get recommendations
//!     char recommendations[4096];
//!     result = omega_recommend(
//!         "{\"time\":\"evening\",\"mood\":\"relaxed\"}",
//!         recommendations,
//!         sizeof(recommendations)
//!     );
//!
//!     // Record viewing
//!     omega_observe("{\"content_id\":\"movie123\",\"watch_pct\":0.95}");
//!
//!     // Sync (call periodically)
//!     omega_sync();
//!
//!     // Cleanup
//!     omega_shutdown();
//!     return 0;
//! }
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │         OMEGA BRAIN (~200MB)        │
//! ├─────────────────────────────────────┤
//! │ RuVector (50MB)                     │
//! │  - HNSW index with SIMD             │
//! │  - <1ms vector search               │
//! ├─────────────────────────────────────┤
//! │ ONNX Runtime (100MB)                │
//! │  - MiniLM 4-bit quantized           │
//! │  - <10ms inference                  │
//! ├─────────────────────────────────────┤
//! │ AgentDB Memory (50MB)               │
//! │  - Pattern learning                 │
//! │  - Episode storage                  │
//! ├─────────────────────────────────────┤
//! │ Sync Agent (5MB)                    │
//! │  - Delta-only protocol              │
//! │  - QUIC transport                   │
//! └─────────────────────────────────────┘
//! ```
//!
//! ## Performance Targets
//!
//! - **Recommendation latency**: <15ms (p99)
//! - **Vector search**: <1ms
//! - **ONNX inference**: <10ms
//! - **Memory footprint**: <200MB
//! - **Sync bandwidth**: ~1KB push, ~5KB pull
//!
//! ## Safety and Error Handling
//!
//! All FFI functions are designed to be safe and provide detailed error messages.
//! Check return codes and use `omega_get_last_error()` for diagnostics.

pub mod error;
pub mod builder;
pub mod ffi;

// Re-export main types for convenience
pub use error::{OmegaError, OmegaErrorCode, Result};
pub use builder::{OmegaBuilder, OmegaBrain, BrainConfig, Recommendation, SyncResult};

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Recommended sync interval (5 minutes)
pub const RECOMMENDED_SYNC_INTERVAL_SECS: u64 = 300;

/// Maximum sync interval (15 minutes)
pub const MAX_SYNC_INTERVAL_SECS: u64 = 900;

/// Default vector dimensions (MiniLM model)
pub const DEFAULT_DIMENSIONS: usize = 384;

/// Default maximum patterns to store
pub const DEFAULT_MAX_PATTERNS: usize = 10_000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(RECOMMENDED_SYNC_INTERVAL_SECS, 300);
        assert_eq!(MAX_SYNC_INTERVAL_SECS, 900);
        assert_eq!(DEFAULT_DIMENSIONS, 384);
        assert_eq!(DEFAULT_MAX_PATTERNS, 10_000);
    }

    #[test]
    fn test_version_exists() {
        assert!(!VERSION.is_empty());
    }
}
