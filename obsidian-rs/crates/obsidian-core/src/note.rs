//! Note and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// File statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStat {
    /// Creation time
    pub ctime: DateTime<Utc>,
    /// Modification time
    pub mtime: DateTime<Utc>,
    /// File size in bytes
    pub size: u64,
}

impl FileStat {
    /// Create file stats from std::fs::Metadata
    pub fn from_metadata(metadata: &std::fs::Metadata) -> Self {
        use std::time::UNIX_EPOCH;

        let ctime = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| DateTime::from_timestamp(d.as_secs() as i64, d.subsec_nanos()))
            .flatten()
            .unwrap_or_else(Utc::now);

        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| DateTime::from_timestamp(d.as_secs() as i64, d.subsec_nanos()))
            .flatten()
            .unwrap_or_else(Utc::now);

        Self {
            ctime,
            mtime,
            size: metadata.len(),
        }
    }

    /// Create new file stats with current time
    pub fn now(size: u64) -> Self {
        let now = Utc::now();
        Self {
            ctime: now,
            mtime: now,
            size,
        }
    }
}

impl Default for FileStat {
    fn default() -> Self {
        Self::now(0)
    }
}

/// YAML frontmatter from a note
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Frontmatter {
    /// Note title (overrides filename)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Tags defined in frontmatter
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Alternative names for the note
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,

    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,

    /// Modification timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<DateTime<Utc>>,

    /// CSS classes to apply to the note
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cssclasses: Vec<String>,

    /// Publish status for Obsidian Publish
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish: Option<bool>,

    /// Additional custom fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Frontmatter {
    /// Create new empty frontmatter
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if frontmatter is empty
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.tags.is_empty()
            && self.aliases.is_empty()
            && self.created.is_none()
            && self.modified.is_none()
            && self.cssclasses.is_empty()
            && self.publish.is_none()
            && self.extra.is_empty()
    }

    /// Get all tags including nested paths
    pub fn all_tags(&self) -> Vec<&str> {
        self.tags.iter().map(|s| s.as_str()).collect()
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Add an alias
    pub fn add_alias(&mut self, alias: impl Into<String>) {
        let alias = alias.into();
        if !self.aliases.contains(&alias) {
            self.aliases.push(alias);
        }
    }
}

/// A note in the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Unique identifier (relative path from vault root)
    pub id: String,

    /// Absolute file path
    pub path: PathBuf,

    /// Filename without extension
    pub basename: String,

    /// File extension (without dot)
    pub extension: String,

    /// Raw markdown content
    pub content: String,

    /// Parsed frontmatter (if present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontmatter: Option<Frontmatter>,

    /// File statistics
    pub stat: FileStat,
}

impl Note {
    /// Create a new note
    pub fn new(
        id: impl Into<String>,
        path: impl Into<PathBuf>,
        content: impl Into<String>,
    ) -> Self {
        let path = path.into();
        let id = id.into();
        let content = content.into();

        let basename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("md")
            .to_string();

        Self {
            id,
            path,
            basename,
            extension,
            stat: FileStat::now(content.len() as u64),
            content,
            frontmatter: None,
        }
    }

    /// Get the display title (frontmatter title or basename)
    pub fn title(&self) -> &str {
        self.frontmatter
            .as_ref()
            .and_then(|f| f.title.as_deref())
            .unwrap_or(&self.basename)
    }

    /// Check if this is a markdown file
    pub fn is_markdown(&self) -> bool {
        self.extension == "md" || self.extension == "markdown"
    }

    /// Check if this is a canvas file
    pub fn is_canvas(&self) -> bool {
        self.extension == "canvas"
    }

    /// Get all tags (from frontmatter and content)
    pub fn all_tags(&self) -> Vec<&str> {
        self.frontmatter
            .as_ref()
            .map(|f| f.all_tags())
            .unwrap_or_default()
    }

    /// Get all aliases
    pub fn aliases(&self) -> Vec<&str> {
        self.frontmatter
            .as_ref()
            .map(|f| f.aliases.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get the parent folder path
    pub fn parent(&self) -> Option<&std::path::Path> {
        self.path.parent()
    }

    /// Get the relative path components
    pub fn path_components(&self) -> Vec<&str> {
        self.id.split('/').collect()
    }
}

/// Builder for creating notes
#[derive(Debug, Default)]
pub struct NoteBuilder {
    id: Option<String>,
    path: Option<PathBuf>,
    content: String,
    frontmatter: Option<Frontmatter>,
}

impl NoteBuilder {
    /// Create a new note builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the note ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the note path
    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the note content
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// Set the frontmatter
    pub fn frontmatter(mut self, frontmatter: Frontmatter) -> Self {
        self.frontmatter = Some(frontmatter);
        self
    }

    /// Build the note
    pub fn build(self) -> crate::Result<Note> {
        let path = self.path.ok_or_else(|| crate::Error::internal("Note path is required"))?;
        let id = self.id.unwrap_or_else(|| path.to_string_lossy().to_string());

        let mut note = Note::new(id, path, self.content);
        note.frontmatter = self.frontmatter;
        Ok(note)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_creation() {
        let note = Note::new("test.md", "/vault/test.md", "# Test\n\nContent");

        assert_eq!(note.id, "test.md");
        assert_eq!(note.basename, "test");
        assert_eq!(note.extension, "md");
        assert!(note.is_markdown());
    }

    #[test]
    fn test_frontmatter() {
        let mut fm = Frontmatter::new();
        assert!(fm.is_empty());

        fm.title = Some("Test Note".into());
        fm.add_tag("test");
        fm.add_alias("testing");

        assert!(!fm.is_empty());
        assert_eq!(fm.all_tags(), vec!["test"]);
    }

    #[test]
    fn test_note_builder() {
        let note = NoteBuilder::new()
            .id("notes/test.md")
            .path("/vault/notes/test.md")
            .content("# Hello World")
            .build()
            .unwrap();

        assert_eq!(note.id, "notes/test.md");
        assert_eq!(note.title(), "test");
    }
}
