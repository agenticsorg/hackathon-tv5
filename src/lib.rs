//! Media Gateway - Root Library Module
//!
//! This is the root library module for the Media Gateway platform.
//! The actual implementation is organized into workspace crates under `crates/`.
//!
//! # Architecture
//!
//! - **Core**: Shared types, errors, and utilities (`crates/core`)
//! - **Discovery**: Search and content discovery service (`crates/discovery`)
//! - **SONA**: AI-powered personalization engine (`crates/sona`)
//! - **Sync**: Real-time cross-device synchronization (`crates/sync`)
//! - **Auth**: Authentication and authorization (`crates/auth`)
//! - **Ingestion**: Platform data ingestion pipeline (`crates/ingestion`)
//! - **Playback**: Device management and playback control (`crates/playback`)
//! - **API**: HTTP API gateway (`crates/api`)
//!
//! # Technology Stack
//!
//! - **Language**: Rust 2021 edition (1.75+)
//! - **HTTP Framework**: Actix-web 4.x
//! - **Async Runtime**: Tokio 1.x
//! - **Database**: PostgreSQL 15 (via sqlx)
//! - **Cache**: Redis 7
//! - **Vector DB**: Qdrant
//! - **Real-time**: PubNub
//!
//! # Performance Targets
//!
//! - Search latency (p95): <500ms
//! - SONA personalization: <5ms
//! - Cross-device sync: <100ms
//! - System availability: 99.9%
//!
//! # Documentation
//!
//! See `/workspaces/media-gateway/src/ARCHITECTURE_CONTEXT.md` for comprehensive
//! architecture documentation and implementation guidelines.

#![warn(
    missing_docs,
    rust_2018_idioms,
    unused_qualifications,
    clippy::all,
    clippy::pedantic
)]
#![allow(clippy::module_name_repetitions)]

/// Version of the Media Gateway platform
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// API version prefix for all HTTP endpoints
pub const API_VERSION: &str = "v1";

/// Maximum allowed request payload size (10MB)
pub const MAX_PAYLOAD_SIZE: usize = 10 * 1024 * 1024;

/// Default request timeout (30 seconds)
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_format() {
        // Version should follow semver format
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert_eq!(parts.len(), 3, "Version should be in x.y.z format");
    }

    #[test]
    fn test_api_version() {
        assert_eq!(API_VERSION, "v1");
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_PAYLOAD_SIZE, 10_485_760);
        assert_eq!(DEFAULT_TIMEOUT_SECS, 30);
    }
}
