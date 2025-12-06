//! # omega-protocol
//!
//! Shared protocol types for Exogenesis Omega distributed TV recommendation system.

pub mod compression;
pub mod events;
pub mod messages;
pub mod patterns;
pub mod recommendations;
pub mod sync;
pub mod types;

// Re-export commonly used types
pub use compression::{compress_delta, compress_patterns, decompress_delta, decompress_patterns};
pub use events::*;
pub use messages::*;
pub use patterns::*;
pub use recommendations::*;
pub use sync::*;
pub use types::*;

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Embedding dimensions (MiniLM)
pub const EMBEDDING_DIMENSIONS: usize = 384;

/// Success threshold for viewing (70%)
pub const SUCCESS_THRESHOLD: f32 = 0.7;

/// Minimum quality for sync
pub const MIN_QUALITY_THRESHOLD: f32 = 0.7;
