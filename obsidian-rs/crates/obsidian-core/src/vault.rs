//! Vault types and configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Abstract file type (file or folder)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TAbstractFile {
    /// A file
    File(TFile),
    /// A folder
    Folder(TFolder),
}

impl TAbstractFile {
    /// Get the path
    pub fn path(&self) -> &str {
        match self {
            Self::File(f) => &f.path,
            Self::Folder(f) => &f.path,
        }
    }

    /// Get the name
    pub fn name(&self) -> &str {
        match self {
            Self::File(f) => &f.name,
            Self::Folder(f) => &f.name,
        }
    }

    /// Check if this is a file
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    /// Check if this is a folder
    pub fn is_folder(&self) -> bool {
        matches!(self, Self::Folder(_))
    }

    /// Get as file if it is one
    pub fn as_file(&self) -> Option<&TFile> {
        match self {
            Self::File(f) => Some(f),
            _ => None,
        }
    }

    /// Get as folder if it is one
    pub fn as_folder(&self) -> Option<&TFolder> {
        match self {
            Self::Folder(f) => Some(f),
            _ => None,
        }
    }
}

/// A file in the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TFile {
    /// Relative path from vault root
    pub path: String,

    /// File name with extension
    pub name: String,

    /// File name without extension
    pub basename: String,

    /// File extension (without dot)
    pub extension: String,

    /// File statistics
    pub stat: TFileStat,
}

impl TFile {
    /// Create a new TFile
    pub fn new(path: impl Into<String>) -> Self {
        let path = path.into();
        let path_buf = PathBuf::from(&path);

        let name = path_buf
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let basename = path_buf
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let extension = path_buf
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        Self {
            path,
            name,
            basename,
            extension,
            stat: TFileStat::default(),
        }
    }

    /// Check if this is a markdown file
    pub fn is_markdown(&self) -> bool {
        self.extension == "md" || self.extension == "markdown"
    }

    /// Check if this is a canvas file
    pub fn is_canvas(&self) -> bool {
        self.extension == "canvas"
    }

    /// Get the parent folder path
    pub fn parent(&self) -> Option<String> {
        PathBuf::from(&self.path)
            .parent()
            .and_then(|p| p.to_str())
            .map(|s| s.to_string())
    }
}

/// File statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TFileStat {
    /// Creation time (Unix timestamp in ms)
    pub ctime: i64,

    /// Modification time (Unix timestamp in ms)
    pub mtime: i64,

    /// File size in bytes
    pub size: u64,
}

impl TFileStat {
    /// Create from std::fs::Metadata
    pub fn from_metadata(metadata: &std::fs::Metadata) -> Self {
        use std::time::UNIX_EPOCH;

        let ctime = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        Self {
            ctime,
            mtime,
            size: metadata.len(),
        }
    }
}

/// A folder in the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TFolder {
    /// Relative path from vault root
    pub path: String,

    /// Folder name
    pub name: String,

    /// Children (files and folders)
    #[serde(default)]
    pub children: Vec<TAbstractFile>,
}

impl TFolder {
    /// Create a new TFolder
    pub fn new(path: impl Into<String>) -> Self {
        let path = path.into();
        let name = PathBuf::from(&path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        Self {
            path,
            name,
            children: Vec::new(),
        }
    }

    /// Create the root folder
    pub fn root() -> Self {
        Self {
            path: String::new(),
            name: String::new(),
            children: Vec::new(),
        }
    }

    /// Add a child
    pub fn add_child(&mut self, child: TAbstractFile) {
        self.children.push(child);
    }

    /// Get files in this folder (not recursive)
    pub fn files(&self) -> Vec<&TFile> {
        self.children.iter().filter_map(|c| c.as_file()).collect()
    }

    /// Get subfolders in this folder
    pub fn folders(&self) -> Vec<&TFolder> {
        self.children.iter().filter_map(|c| c.as_folder()).collect()
    }

    /// Check if folder is empty
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}

/// Vault configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Always update internal links when renaming files
    #[serde(default = "default_true")]
    pub always_update_links: bool,

    /// Attachment folder path
    #[serde(default = "default_attachment_folder")]
    pub attachment_folder_path: String,

    /// Where to create new files
    #[serde(default)]
    pub new_file_location: NewFileLocation,

    /// Folder for new files (if new_file_location is "folder")
    #[serde(default)]
    pub new_file_folder_path: String,

    /// Show unsupported files in file explorer
    #[serde(default)]
    pub show_unsupported_files: bool,

    /// Use markdown links instead of wikilinks
    #[serde(default)]
    pub use_markdown_links: bool,

    /// Deleted files destination
    #[serde(default)]
    pub trash_option: TrashOption,

    /// Default view mode for new notes
    #[serde(default)]
    pub default_view_mode: ViewMode,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            always_update_links: true,
            attachment_folder_path: default_attachment_folder(),
            new_file_location: NewFileLocation::default(),
            new_file_folder_path: String::new(),
            show_unsupported_files: false,
            use_markdown_links: false,
            trash_option: TrashOption::default(),
            default_view_mode: ViewMode::default(),
        }
    }
}

/// Where to create new files
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NewFileLocation {
    /// In the vault root
    #[default]
    Root,
    /// In the same folder as current file
    Current,
    /// In a specific folder
    Folder,
}

/// What to do with deleted files
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TrashOption {
    /// Move to .trash folder in vault
    #[default]
    Local,
    /// Move to system trash
    System,
    /// Delete permanently
    None,
}

/// Default view mode for notes
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ViewMode {
    /// Raw markdown source
    #[default]
    Source,
    /// Live preview (WYSIWYG hybrid)
    Live,
    /// Read-only rendered view
    Preview,
}

fn default_true() -> bool {
    true
}

fn default_attachment_folder() -> String {
    "attachments".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tfile() {
        let file = TFile::new("notes/test.md");
        assert_eq!(file.path, "notes/test.md");
        assert_eq!(file.name, "test.md");
        assert_eq!(file.basename, "test");
        assert_eq!(file.extension, "md");
        assert!(file.is_markdown());
        assert_eq!(file.parent(), Some("notes".to_string()));
    }

    #[test]
    fn test_tfolder() {
        let mut folder = TFolder::new("notes");
        assert_eq!(folder.name, "notes");
        assert!(folder.is_empty());

        folder.add_child(TAbstractFile::File(TFile::new("notes/test.md")));
        assert!(!folder.is_empty());
        assert_eq!(folder.files().len(), 1);
    }

    #[test]
    fn test_vault_config() {
        let config = VaultConfig::default();
        assert!(config.always_update_links);
        assert_eq!(config.attachment_folder_path, "attachments");
        assert_eq!(config.new_file_location, NewFileLocation::Root);
    }
}
