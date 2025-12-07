//! ObsidianRS Core - Core types and traits for the Obsidian clone
//!
//! This crate provides the fundamental data structures used throughout
//! the ObsidianRS application, including Note, Vault, Workspace, and
//! Metadata types that are compatible with the original Obsidian format.

pub mod error;
pub mod metadata;
pub mod note;
pub mod position;
pub mod vault;
pub mod workspace;

pub use error::{Error, Result};
pub use metadata::*;
pub use note::*;
pub use position::*;
pub use vault::*;
pub use workspace::*;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::metadata::{CachedMetadata, EmbedCache, HeadingCache, LinkCache, TagCache};
    pub use crate::note::{FileStat, Frontmatter, Note};
    pub use crate::position::{Location, Position};
    pub use crate::vault::{TAbstractFile, TFile, TFolder, VaultConfig};
    pub use crate::workspace::{
        SplitDirection, ViewState, WorkspaceConfig, WorkspaceLeaf, WorkspaceSplit,
    };
}
