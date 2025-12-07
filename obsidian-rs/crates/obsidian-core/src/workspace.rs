//! Workspace types for managing the UI layout

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Workspace layout state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Main content area
    pub main: WorkspaceSplit,

    /// Left sidebar
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<WorkspaceSplit>,

    /// Right sidebar
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<WorkspaceSplit>,

    /// Left ribbon collapsed state
    #[serde(default)]
    pub left_ribbon_hidden: bool,

    /// Right ribbon collapsed state
    #[serde(default)]
    pub right_ribbon_hidden: bool,

    /// Active leaf ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<String>,

    /// Last open files
    #[serde(default)]
    pub last_open_files: Vec<String>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            main: WorkspaceSplit::Leaf(WorkspaceLeaf::empty()),
            left: Some(WorkspaceSplit::Leaf(WorkspaceLeaf::file_explorer())),
            right: Some(WorkspaceSplit::Leaf(WorkspaceLeaf::backlinks())),
            left_ribbon_hidden: false,
            right_ribbon_hidden: false,
            active: None,
            last_open_files: Vec::new(),
        }
    }
}

/// A split in the workspace (container for leaves or nested splits)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WorkspaceSplit {
    /// A single view
    Leaf(WorkspaceLeaf),
    /// A split container
    Split {
        /// Split direction
        direction: SplitDirection,
        /// Child splits/leaves
        children: Vec<WorkspaceSplit>,
    },
    /// Tab group container
    Tabs {
        /// Tab children
        children: Vec<WorkspaceSplit>,
        /// Active tab index
        #[serde(default)]
        current_tab: usize,
    },
}

impl WorkspaceSplit {
    /// Create an empty leaf
    pub fn empty_leaf() -> Self {
        Self::Leaf(WorkspaceLeaf::empty())
    }

    /// Create a horizontal split
    pub fn horizontal(children: Vec<WorkspaceSplit>) -> Self {
        Self::Split {
            direction: SplitDirection::Horizontal,
            children,
        }
    }

    /// Create a vertical split
    pub fn vertical(children: Vec<WorkspaceSplit>) -> Self {
        Self::Split {
            direction: SplitDirection::Vertical,
            children,
        }
    }

    /// Create a tab group
    pub fn tabs(children: Vec<WorkspaceSplit>) -> Self {
        Self::Tabs {
            children,
            current_tab: 0,
        }
    }

    /// Check if this is a leaf
    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf(_))
    }

    /// Get as leaf if it is one
    pub fn as_leaf(&self) -> Option<&WorkspaceLeaf> {
        match self {
            Self::Leaf(leaf) => Some(leaf),
            _ => None,
        }
    }

    /// Get mutable reference to leaf
    pub fn as_leaf_mut(&mut self) -> Option<&mut WorkspaceLeaf> {
        match self {
            Self::Leaf(leaf) => Some(leaf),
            _ => None,
        }
    }

    /// Get all leaves recursively
    pub fn all_leaves(&self) -> Vec<&WorkspaceLeaf> {
        match self {
            Self::Leaf(leaf) => vec![leaf],
            Self::Split { children, .. } | Self::Tabs { children, .. } => {
                children.iter().flat_map(|c| c.all_leaves()).collect()
            }
        }
    }

    /// Find a leaf by ID
    pub fn find_leaf(&self, id: &str) -> Option<&WorkspaceLeaf> {
        self.all_leaves().into_iter().find(|l| l.id == id)
    }
}

/// Split direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SplitDirection {
    /// Side by side
    Horizontal,
    /// Stacked
    Vertical,
}

/// A leaf (view) in the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLeaf {
    /// Unique identifier
    pub id: String,

    /// View type (markdown, graph, file-explorer, etc.)
    #[serde(rename = "type")]
    pub view_type: String,

    /// View state
    pub state: ViewState,

    /// Whether this leaf is pinned
    #[serde(default)]
    pub pinned: bool,

    /// Group ID for linked panes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

impl WorkspaceLeaf {
    /// Create a new leaf
    pub fn new(id: impl Into<String>, view_type: impl Into<String>, state: ViewState) -> Self {
        Self {
            id: id.into(),
            view_type: view_type.into(),
            state,
            pinned: false,
            group: None,
        }
    }

    /// Create an empty leaf
    pub fn empty() -> Self {
        Self::new(uuid::Uuid::new_v4().to_string(), "empty", ViewState::empty())
    }

    /// Create a markdown view leaf
    pub fn markdown(file: impl Into<String>) -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "markdown",
            ViewState::markdown(file),
        )
    }

    /// Create a file explorer leaf
    pub fn file_explorer() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "file-explorer",
            ViewState::empty(),
        )
    }

    /// Create a search leaf
    pub fn search() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "search",
            ViewState::empty(),
        )
    }

    /// Create a graph view leaf
    pub fn graph() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "graph",
            ViewState::empty(),
        )
    }

    /// Create a backlinks leaf
    pub fn backlinks() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "backlink",
            ViewState::empty(),
        )
    }

    /// Create an outline leaf
    pub fn outline() -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            "outline",
            ViewState::empty(),
        )
    }

    /// Create a tag pane leaf
    pub fn tags() -> Self {
        Self::new(uuid::Uuid::new_v4().to_string(), "tag", ViewState::empty())
    }

    /// Check if this is a markdown view
    pub fn is_markdown(&self) -> bool {
        self.view_type == "markdown"
    }

    /// Get the file path if this is a markdown view
    pub fn file(&self) -> Option<&str> {
        self.state.file.as_deref()
    }
}

/// View state for a leaf
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewState {
    /// File path for file-based views
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,

    /// View mode (source, preview, live)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,

    /// Whether source mode is active (for backwards compat)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<bool>,

    /// Scroll position
    #[serde(default)]
    pub scroll: i32,

    /// Cursor position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<CursorState>,

    /// Additional state data
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl ViewState {
    /// Create empty view state
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create markdown view state
    pub fn markdown(file: impl Into<String>) -> Self {
        Self {
            file: Some(file.into()),
            mode: Some("source".into()),
            ..Default::default()
        }
    }

    /// Create preview mode state
    pub fn preview(file: impl Into<String>) -> Self {
        Self {
            file: Some(file.into()),
            mode: Some("preview".into()),
            ..Default::default()
        }
    }

    /// Check if in source mode
    pub fn is_source(&self) -> bool {
        self.mode.as_deref() == Some("source") || self.source == Some(true)
    }

    /// Check if in preview mode
    pub fn is_preview(&self) -> bool {
        self.mode.as_deref() == Some("preview")
    }
}

/// Cursor state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CursorState {
    /// Line number
    pub line: usize,
    /// Column/character position
    pub ch: usize,
}

impl CursorState {
    /// Create new cursor state
    pub fn new(line: usize, ch: usize) -> Self {
        Self { line, ch }
    }
}

/// Named workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedWorkspace {
    /// Workspace name
    pub name: String,
    /// Workspace configuration
    pub config: WorkspaceConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_config() {
        let config = WorkspaceConfig::default();
        assert!(!config.left_ribbon_hidden);
        assert!(config.left.is_some());
        assert!(config.right.is_some());
    }

    #[test]
    fn test_workspace_split() {
        let split = WorkspaceSplit::horizontal(vec![
            WorkspaceSplit::Leaf(WorkspaceLeaf::markdown("note1.md")),
            WorkspaceSplit::Leaf(WorkspaceLeaf::markdown("note2.md")),
        ]);

        let leaves = split.all_leaves();
        assert_eq!(leaves.len(), 2);
    }

    #[test]
    fn test_workspace_leaf() {
        let leaf = WorkspaceLeaf::markdown("test.md");
        assert!(leaf.is_markdown());
        assert_eq!(leaf.file(), Some("test.md"));
    }

    #[test]
    fn test_view_state() {
        let state = ViewState::markdown("test.md");
        assert!(state.is_source());
        assert!(!state.is_preview());

        let preview = ViewState::preview("test.md");
        assert!(preview.is_preview());
    }
}
