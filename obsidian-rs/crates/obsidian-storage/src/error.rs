//! Storage error types

use std::path::PathBuf;
use thiserror::Error;

/// Storage error types
#[derive(Error, Debug)]
pub enum StorageError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// Directory not found
    #[error("Directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    /// File already exists
    #[error("File already exists: {0}")]
    FileExists(PathBuf),

    /// Invalid path
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Vault not initialized
    #[error("Vault not initialized: {0}")]
    VaultNotInitialized(PathBuf),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Watcher error
    #[error("Watcher error: {0}")]
    Watcher(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    /// Path outside vault
    #[error("Path outside vault: {0}")]
    PathOutsideVault(PathBuf),

    /// Invalid file type
    #[error("Invalid file type: expected {expected}, got {actual}")]
    InvalidFileType { expected: String, actual: String },

    /// Lock error
    #[error("Lock error: {0}")]
    Lock(String),

    /// Operation cancelled
    #[error("Operation cancelled")]
    Cancelled,
}

impl From<redb::Error> for StorageError {
    fn from(err: redb::Error) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<redb::DatabaseError> for StorageError {
    fn from(err: redb::DatabaseError) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<redb::TableError> for StorageError {
    fn from(err: redb::TableError) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<redb::TransactionError> for StorageError {
    fn from(err: redb::TransactionError) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<redb::CommitError> for StorageError {
    fn from(err: redb::CommitError) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<redb::StorageError> for StorageError {
    fn from(err: redb::StorageError) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<redb::CompactionError> for StorageError {
    fn from(err: redb::CompactionError) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<notify::Error> for StorageError {
    fn from(err: notify::Error) -> Self {
        StorageError::Watcher(err.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::Serialization(err.to_string())
    }
}

impl From<bincode::Error> for StorageError {
    fn from(err: bincode::Error) -> Self {
        StorageError::Serialization(err.to_string())
    }
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;
