//! Error types for ObsidianRS

use thiserror::Error;

/// Result type alias using ObsidianRS Error
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for ObsidianRS
#[derive(Error, Debug)]
pub enum Error {
    /// Note was not found at the specified path
    #[error("Note not found: {0}")]
    NoteNotFound(String),

    /// Note already exists at the specified path
    #[error("Note already exists: {0}")]
    NoteAlreadyExists(String),

    /// Vault was not found at the specified path
    #[error("Vault not found: {0}")]
    VaultNotFound(String),

    /// Invalid vault structure
    #[error("Invalid vault: {0}")]
    InvalidVault(String),

    /// Invalid markdown content
    #[error("Invalid markdown: {0}")]
    InvalidMarkdown(String),

    /// Invalid frontmatter YAML
    #[error("Invalid frontmatter: {0}")]
    InvalidFrontmatter(String),

    /// Invalid wikilink format
    #[error("Invalid wikilink: {0}")]
    InvalidWikilink(String),

    /// File system error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML serialization/deserialization error
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Search index error
    #[error("Search error: {0}")]
    Search(String),

    /// Graph operation error
    #[error("Graph error: {0}")]
    Graph(String),

    /// Plugin error
    #[error("Plugin error: {0}")]
    Plugin(String),

    /// Configuration error
    #[error("Config error: {0}")]
    Config(String),

    /// Workspace error
    #[error("Workspace error: {0}")]
    Workspace(String),

    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Create a new internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Create a search error
    pub fn search(msg: impl Into<String>) -> Self {
        Self::Search(msg.into())
    }

    /// Create a graph error
    pub fn graph(msg: impl Into<String>) -> Self {
        Self::Graph(msg.into())
    }

    /// Create a plugin error
    pub fn plugin(msg: impl Into<String>) -> Self {
        Self::Plugin(msg.into())
    }

    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
}
