//! Graph error types

use thiserror::Error;

/// Graph error types
#[derive(Error, Debug)]
pub enum GraphError {
    /// Node not found
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    /// Edge not found
    #[error("Edge not found: {0} -> {1}")]
    EdgeNotFound(String, String),

    /// Cycle detected
    #[error("Cycle detected in graph")]
    CycleDetected,

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<serde_json::Error> for GraphError {
    fn from(err: serde_json::Error) -> Self {
        GraphError::Serialization(err.to_string())
    }
}

/// Result type for graph operations
pub type GraphResult<T> = Result<T, GraphError>;
