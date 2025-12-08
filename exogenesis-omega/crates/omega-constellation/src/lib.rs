//! Omega Constellation - Server-side coordination layer
//!
//! The Constellation handles pattern aggregation and distribution across
//! 400M TVs with 10M concurrent users. It does NOT perform inference -
//! all inference happens locally on each TV.

pub mod api;
pub mod federation;
pub mod metrics;
pub mod shard;
pub mod storage;

// Re-exports
pub use api::grpc::SyncService;
pub use api::rest::{create_rest_router, RestState};
pub use federation::{FederationCoordinator, FederationConfig, FederationResult};
pub use metrics::{register_metrics, METRICS_REGISTERED};
pub use shard::{ShardConfig, ShardManager};
pub use storage::{InMemoryStorage, PatternStorage, PostgresStorage};

// Common types
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Device identifier
pub type DeviceId = Uuid;

/// Pattern identifier
pub type PatternId = Uuid;

/// Viewing pattern with embedding and quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewingPattern {
    pub id: PatternId,
    pub embedding: Vec<f32>,
    pub success_rate: f32,
    pub sample_count: u32,
    pub context: PatternContext,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Context information for a pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternContext {
    pub time_of_day: String,
    pub day_of_week: String,
    pub content_type: String,
    pub genre_hints: Vec<String>,
}

/// Delta update for sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDelta {
    pub patterns_added: Vec<ViewingPattern>,
    pub patterns_updated: Vec<PatternUpdate>,
    pub patterns_removed: Vec<PatternId>,
    pub local_version: u64,
}

/// Pattern update with new metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternUpdate {
    pub id: PatternId,
    pub new_success_rate: f32,
    pub additional_samples: u32,
}

/// Global patterns sent to TV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPatterns {
    pub similar: Vec<ViewingPattern>,
    pub trending: Vec<TrendSignal>,
    pub global_version: u64,
}

/// Trend signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendSignal {
    pub content_id: String,
    pub trending_score: f32,
    pub region: String,
}

/// Device state tracking
#[derive(Debug, Clone)]
pub struct DeviceState {
    pub device_id: DeviceId,
    pub last_sync: i64,
    pub local_version: u64,
    pub pattern_count: usize,
    pub region: String,
}

/// Shard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardStats {
    pub shard_id: u32,
    pub total_devices: usize,
    pub active_devices: usize,
    pub patterns_stored: usize,
    pub sync_requests_per_min: f64,
    pub avg_sync_latency_ms: f64,
}

/// Error types
#[derive(Debug, thiserror::Error)]
pub enum ConstellationError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(DeviceId),

    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    #[error("Shard overload: {0}")]
    ShardOverload(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ConstellationError>;
