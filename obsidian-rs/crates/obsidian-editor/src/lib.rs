//! Text editor core for Obsidian-rs
//!
//! This crate provides:
//! - Rope-based text buffer
//! - Undo/redo history
//! - Cursor and selection management
//! - Text operations

pub mod buffer;
pub mod cursor;
pub mod error;
pub mod history;
pub mod selection;

pub use buffer::TextBuffer;
pub use cursor::{Cursor, CursorPosition};
pub use error::{EditorError, EditorResult};
pub use history::{EditHistory, EditOperation};
pub use selection::{Selection, SelectionRange};

/// Prelude for common imports
pub mod prelude {
    pub use crate::buffer::TextBuffer;
    pub use crate::cursor::{Cursor, CursorPosition};
    pub use crate::error::{EditorError, EditorResult};
    pub use crate::history::{EditHistory, EditOperation};
    pub use crate::selection::{Selection, SelectionRange};
}
