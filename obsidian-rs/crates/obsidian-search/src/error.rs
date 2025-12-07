//! Search error types

use thiserror::Error;

/// Search error types
#[derive(Error, Debug)]
pub enum SearchError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Index error
    #[error("Index error: {0}")]
    Index(String),

    /// Query parse error
    #[error("Query parse error: {0}")]
    QueryParse(String),

    /// Not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Vector error
    #[error("Vector error: {0}")]
    Vector(String),

    /// Embedding error
    #[error("Embedding error: {0}")]
    Embedding(String),
}

impl From<tantivy::TantivyError> for SearchError {
    fn from(err: tantivy::TantivyError) -> Self {
        SearchError::Index(err.to_string())
    }
}

impl From<tantivy::query::QueryParserError> for SearchError {
    fn from(err: tantivy::query::QueryParserError) -> Self {
        SearchError::QueryParse(err.to_string())
    }
}

/// Result type for search operations
pub type SearchResult<T> = Result<T, SearchError>;
