//! Cached metadata types for notes

use crate::position::Position;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cached link information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkCache {
    /// Target note name or path
    pub link: String,

    /// Original markdown text (e.g., "[[note|display]]")
    pub original: String,

    /// Display text if different from link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_text: Option<String>,

    /// Target heading within the note
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,

    /// Target block ID within the note
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,

    /// Position in the source file
    pub position: Position,
}

impl LinkCache {
    /// Create a new link cache entry
    pub fn new(link: impl Into<String>, original: impl Into<String>, position: Position) -> Self {
        Self {
            link: link.into(),
            original: original.into(),
            display_text: None,
            heading: None,
            block_id: None,
            position,
        }
    }

    /// Set the display text
    pub fn with_display(mut self, display: impl Into<String>) -> Self {
        self.display_text = Some(display.into());
        self
    }

    /// Set the target heading
    pub fn with_heading(mut self, heading: impl Into<String>) -> Self {
        self.heading = Some(heading.into());
        self
    }

    /// Set the target block ID
    pub fn with_block_id(mut self, block_id: impl Into<String>) -> Self {
        self.block_id = Some(block_id.into());
        self
    }

    /// Get the full link path including subpath
    pub fn full_path(&self) -> String {
        let mut path = self.link.clone();
        if let Some(heading) = &self.heading {
            path.push('#');
            path.push_str(heading);
        }
        if let Some(block_id) = &self.block_id {
            path.push_str("#^");
            path.push_str(block_id);
        }
        path
    }

    /// Get the display text or the link target
    pub fn display(&self) -> &str {
        self.display_text.as_deref().unwrap_or(&self.link)
    }
}

/// Cached embed information (transclusion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedCache {
    /// Target path or name
    pub link: String,

    /// Original markdown text
    pub original: String,

    /// Display text for the embed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_text: Option<String>,

    /// Position in the source file
    pub position: Position,
}

impl EmbedCache {
    /// Create a new embed cache entry
    pub fn new(link: impl Into<String>, original: impl Into<String>, position: Position) -> Self {
        Self {
            link: link.into(),
            original: original.into(),
            display_text: None,
            position,
        }
    }

    /// Check if this embed is an image
    pub fn is_image(&self) -> bool {
        let lower = self.link.to_lowercase();
        lower.ends_with(".png")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".gif")
            || lower.ends_with(".webp")
            || lower.ends_with(".svg")
            || lower.ends_with(".bmp")
    }

    /// Check if this embed is a PDF
    pub fn is_pdf(&self) -> bool {
        self.link.to_lowercase().ends_with(".pdf")
    }

    /// Check if this embed is a note
    pub fn is_note(&self) -> bool {
        !self.is_image() && !self.is_pdf()
    }
}

/// Cached tag information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCache {
    /// Tag name (without #)
    pub tag: String,

    /// Position in the source file
    pub position: Position,
}

impl TagCache {
    /// Create a new tag cache entry
    pub fn new(tag: impl Into<String>, position: Position) -> Self {
        Self {
            tag: tag.into(),
            position,
        }
    }

    /// Get tag segments for nested tags
    pub fn segments(&self) -> Vec<&str> {
        self.tag.split('/').collect()
    }

    /// Check if this is a nested tag
    pub fn is_nested(&self) -> bool {
        self.tag.contains('/')
    }

    /// Get the parent tag (if nested)
    pub fn parent(&self) -> Option<&str> {
        if self.is_nested() {
            self.tag.rsplit_once('/').map(|(parent, _)| parent)
        } else {
            None
        }
    }
}

/// Cached heading information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingCache {
    /// Heading text
    pub heading: String,

    /// Heading level (1-6)
    pub level: u8,

    /// Position in the source file
    pub position: Position,
}

impl HeadingCache {
    /// Create a new heading cache entry
    pub fn new(heading: impl Into<String>, level: u8, position: Position) -> Self {
        Self {
            heading: heading.into(),
            level: level.clamp(1, 6),
            position,
        }
    }

    /// Generate a slug for linking
    pub fn slug(&self) -> String {
        self.heading
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }
}

/// Cached block reference information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockCache {
    /// Block ID
    pub id: String,

    /// Position in the source file
    pub position: Position,
}

impl BlockCache {
    /// Create a new block cache entry
    pub fn new(id: impl Into<String>, position: Position) -> Self {
        Self {
            id: id.into(),
            position,
        }
    }
}

/// Cached section information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionCache {
    /// Section type (paragraph, heading, code, etc.)
    pub section_type: String,

    /// Position in the source file
    pub position: Position,
}

/// Cached list item information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItemCache {
    /// Position in the source file
    pub position: Position,

    /// Parent list item position (for nested lists)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Position>,

    /// Task status (for task lists)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskStatus>,
}

/// Task list item status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Incomplete task `[ ]`
    Incomplete,
    /// Completed task `[x]` or `[X]`
    Complete,
    /// Cancelled task `[-]`
    Cancelled,
    /// In progress `[/]`
    InProgress,
    /// Custom status
    Custom(char),
}

impl TaskStatus {
    /// Parse task status from a character
    pub fn from_char(c: char) -> Self {
        match c {
            ' ' => Self::Incomplete,
            'x' | 'X' => Self::Complete,
            '-' => Self::Cancelled,
            '/' => Self::InProgress,
            other => Self::Custom(other),
        }
    }

    /// Convert to character
    pub fn to_char(self) -> char {
        match self {
            Self::Incomplete => ' ',
            Self::Complete => 'x',
            Self::Cancelled => '-',
            Self::InProgress => '/',
            Self::Custom(c) => c,
        }
    }

    /// Check if the task is done (complete or cancelled)
    pub fn is_done(self) -> bool {
        matches!(self, Self::Complete | Self::Cancelled)
    }
}

/// Complete cached metadata for a note
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CachedMetadata {
    /// Internal links [[...]]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<LinkCache>,

    /// Embedded content ![[...]]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub embeds: Vec<EmbedCache>,

    /// Tags #tag
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<TagCache>,

    /// Headings
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headings: Vec<HeadingCache>,

    /// Block references ^block-id
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub blocks: HashMap<String, BlockCache>,

    /// Sections (paragraphs, code blocks, etc.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<SectionCache>,

    /// List items
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub list_items: Vec<ListItemCache>,

    /// Frontmatter properties
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frontmatter: Option<crate::note::Frontmatter>,

    /// Position of the frontmatter block
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frontmatter_position: Option<Position>,
}

impl CachedMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if metadata is empty
    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
            && self.embeds.is_empty()
            && self.tags.is_empty()
            && self.headings.is_empty()
            && self.blocks.is_empty()
            && self.frontmatter.is_none()
    }

    /// Get all unique link targets
    pub fn link_targets(&self) -> Vec<&str> {
        let mut targets: Vec<&str> = self.links.iter().map(|l| l.link.as_str()).collect();
        targets.sort();
        targets.dedup();
        targets
    }

    /// Get all unique tag names
    pub fn tag_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.tags.iter().map(|t| t.tag.as_str()).collect();
        names.sort();
        names.dedup();
        names
    }

    /// Get the table of contents from headings
    pub fn toc(&self) -> Vec<(&HeadingCache, usize)> {
        self.headings.iter().map(|h| (h, h.level as usize)).collect()
    }

    /// Find a heading by text
    pub fn find_heading(&self, text: &str) -> Option<&HeadingCache> {
        let lower = text.to_lowercase();
        self.headings
            .iter()
            .find(|h| h.heading.to_lowercase() == lower)
    }

    /// Find a block by ID
    pub fn find_block(&self, id: &str) -> Option<&BlockCache> {
        self.blocks.get(id)
    }

    /// Get task items
    pub fn tasks(&self) -> Vec<&ListItemCache> {
        self.list_items.iter().filter(|l| l.task.is_some()).collect()
    }

    /// Get incomplete tasks
    pub fn incomplete_tasks(&self) -> Vec<&ListItemCache> {
        self.list_items
            .iter()
            .filter(|l| matches!(l.task, Some(status) if !status.is_done()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Location;

    fn test_position() -> Position {
        Position::new(Location::new(0, 0, 0), Location::new(0, 10, 10))
    }

    #[test]
    fn test_link_cache() {
        let link = LinkCache::new("note", "[[note|Display]]", test_position())
            .with_display("Display")
            .with_heading("section");

        assert_eq!(link.link, "note");
        assert_eq!(link.display(), "Display");
        assert_eq!(link.full_path(), "note#section");
    }

    #[test]
    fn test_embed_cache() {
        let embed = EmbedCache::new("image.png", "![[image.png]]", test_position());
        assert!(embed.is_image());
        assert!(!embed.is_pdf());
        assert!(!embed.is_note());
    }

    #[test]
    fn test_tag_cache() {
        let tag = TagCache::new("nested/tag/here", test_position());
        assert!(tag.is_nested());
        assert_eq!(tag.segments(), vec!["nested", "tag", "here"]);
        assert_eq!(tag.parent(), Some("nested/tag"));
    }

    #[test]
    fn test_heading_cache() {
        let heading = HeadingCache::new("Hello World!", 2, test_position());
        assert_eq!(heading.slug(), "hello-world");
    }

    #[test]
    fn test_task_status() {
        assert_eq!(TaskStatus::from_char(' '), TaskStatus::Incomplete);
        assert_eq!(TaskStatus::from_char('x'), TaskStatus::Complete);
        assert!(TaskStatus::Complete.is_done());
        assert!(!TaskStatus::Incomplete.is_done());
    }
}
