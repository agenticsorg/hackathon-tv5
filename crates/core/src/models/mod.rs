//! Models module
//!
//! Contains domain models for content, users, and search functionality.

pub mod content;
pub mod search;
pub mod user;

// Re-export commonly used types
pub use content::CanonicalContent;
pub use search::{SearchQuery, SearchResult};
pub use user::UserProfile;
