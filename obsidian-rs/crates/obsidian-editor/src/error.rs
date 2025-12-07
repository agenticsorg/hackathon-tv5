//! Editor error types

use thiserror::Error;

/// Editor error types
#[derive(Error, Debug)]
pub enum EditorError {
    /// Invalid position
    #[error("Invalid position: line {line}, column {column}")]
    InvalidPosition { line: usize, column: usize },

    /// Invalid range
    #[error("Invalid range: {0}..{1}")]
    InvalidRange(usize, usize),

    /// Buffer error
    #[error("Buffer error: {0}")]
    Buffer(String),

    /// Nothing to undo
    #[error("Nothing to undo")]
    NothingToUndo,

    /// Nothing to redo
    #[error("Nothing to redo")]
    NothingToRedo,

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for editor operations
pub type EditorResult<T> = Result<T, EditorError>;
