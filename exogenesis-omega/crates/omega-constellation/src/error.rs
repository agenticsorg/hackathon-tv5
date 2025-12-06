//! Error types for Omega Constellation

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ConstellationError>;

#[derive(Error, Debug)]
pub enum ConstellationError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Shard full: shard {shard_id} has reached max capacity of {max_devices} devices")]
    ShardFull { shard_id: u32, max_devices: usize },

    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    #[error("Federation error: {0}")]
    Federation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("gRPC status error: {0}")]
    GrpcStatus(#[from] tonic::Status),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<ConstellationError> for tonic::Status {
    fn from(err: ConstellationError) -> Self {
        match err {
            ConstellationError::DeviceNotFound(msg) => {
                tonic::Status::not_found(msg)
            }
            ConstellationError::ShardFull { .. } => {
                tonic::Status::resource_exhausted(err.to_string())
            }
            ConstellationError::InvalidPattern(msg) => {
                tonic::Status::invalid_argument(msg)
            }
            _ => tonic::Status::internal(err.to_string()),
        }
    }
}
