//! File system storage and watching for Obsidian-rs
//!
//! This crate provides:
//! - Vault management and file operations
//! - Real-time file watching
//! - Metadata caching with persistent database
//! - Note CRUD operations

pub mod cache;
pub mod config;
pub mod database;
pub mod error;
pub mod vault;
pub mod watcher;

pub use cache::{MetadataCache, NoteCache};
pub use config::VaultConfig;
pub use database::Database;
pub use error::{StorageError, StorageResult};
pub use vault::Vault;
pub use watcher::{FileEvent, FileEventKind, VaultWatcher};

/// Prelude for common imports
pub mod prelude {
    pub use crate::cache::{MetadataCache, NoteCache};
    pub use crate::config::VaultConfig;
    pub use crate::database::Database;
    pub use crate::error::{StorageError, StorageResult};
    pub use crate::vault::Vault;
    pub use crate::watcher::{FileEvent, FileEventKind, VaultWatcher};
}
