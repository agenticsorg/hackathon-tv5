//! Wikilink parser for [[link]] syntax

use obsidian_core::position::{Location, Position};
use regex::Regex;
use std::sync::LazyLock;

/// Regex for matching wikilinks: [[target]] or [[target|display]]
static WIKILINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[([^\[\]]+)\]\]").expect("Invalid wikilink regex"));

/// A parsed wikilink
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiLink {
    /// Target note path or name
    pub target: String,
    /// Display text (if using pipe alias)
    pub display: Option<String>,
    /// Target heading within the note
    pub heading: Option<String>,
    /// Target block ID (^block-id)
    pub block_id: Option<String>,
    /// Original text
    pub original: String,
    /// Position in source
    pub position: Position,
}

impl WikiLink {
    /// Create a new wikilink
    pub fn new(target: impl Into<String>, position: Position) -> Self {
        let target = target.into();
        let original = format!("[[{}]]", target);
        Self {
            target,
            display: None,
            heading: None,
            block_id: None,
            original,
            position,
        }
    }

    /// Get the display text (or target if none)
    pub fn display_text(&self) -> &str {
        self.display.as_deref().unwrap_or(&self.target)
    }

    /// Get the full link path including subpath
    pub fn full_path(&self) -> String {
        let mut path = self.target.clone();
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

    /// Check if this link has a subpath (heading or block)
    pub fn has_subpath(&self) -> bool {
        self.heading.is_some() || self.block_id.is_some()
    }

    /// Convert to a LinkCache
    pub fn to_link_cache(&self) -> obsidian_core::metadata::LinkCache {
        let mut cache =
            obsidian_core::metadata::LinkCache::new(&self.target, &self.original, self.position);

        if let Some(display) = &self.display {
            cache = cache.with_display(display);
        }
        if let Some(heading) = &self.heading {
            cache = cache.with_heading(heading);
        }
        if let Some(block_id) = &self.block_id {
            cache = cache.with_block_id(block_id);
        }

        cache
    }
}

/// Parser for wikilinks
#[derive(Debug, Default)]
pub struct WikiLinkParser;

impl WikiLinkParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self
    }

    /// Parse all wikilinks from content
    pub fn parse(&self, content: &str) -> Vec<WikiLink> {
        let mut links = Vec::new();

        for cap in WIKILINK_REGEX.captures_iter(content) {
            let full_match = cap.get(0).unwrap();
            let inner = cap.get(1).unwrap().as_str();

            // Calculate position
            let start_offset = full_match.start();
            let end_offset = full_match.end();
            let (start_line, start_col) = offset_to_line_col(content, start_offset);
            let (end_line, end_col) = offset_to_line_col(content, end_offset);

            let position = Position::new(
                Location::new(start_line, start_col, start_offset),
                Location::new(end_line, end_col, end_offset),
            );

            let link = self.parse_inner(inner, full_match.as_str(), position);
            links.push(link);
        }

        links
    }

    /// Parse the inner content of a wikilink
    fn parse_inner(&self, inner: &str, original: &str, position: Position) -> WikiLink {
        // Check for display text: [[target|display]]
        let (target_part, display) = if let Some(pos) = inner.find('|') {
            let (target, display) = inner.split_at(pos);
            (target.to_string(), Some(display[1..].to_string()))
        } else {
            (inner.to_string(), None)
        };

        // Check for heading: target#heading
        let (target, heading, block_id) = if let Some(hash_pos) = target_part.find('#') {
            let (note, subpath) = target_part.split_at(hash_pos);
            let subpath = &subpath[1..]; // Remove #

            // Check for block reference: #^block-id
            if subpath.starts_with('^') {
                (
                    note.to_string(),
                    None,
                    Some(subpath[1..].to_string()),
                )
            } else {
                (note.to_string(), Some(subpath.to_string()), None)
            }
        } else {
            (target_part, None, None)
        };

        WikiLink {
            target,
            display,
            heading,
            block_id,
            original: original.to_string(),
            position,
        }
    }

    /// Parse a single wikilink string (without the brackets)
    pub fn parse_single(&self, inner: &str) -> WikiLink {
        self.parse_inner(inner, &format!("[[{}]]", inner), Position::default())
    }
}

/// Convert byte offset to line and column
fn offset_to_line_col(content: &str, offset: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;
    let mut current_offset = 0;

    for c in content.chars() {
        if current_offset >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        current_offset += c.len_utf8();
    }

    (line, col)
}

/// Convert a note name to a valid filename
pub fn to_filename(name: &str) -> String {
    let mut result = String::new();
    for c in name.chars() {
        match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => result.push('_'),
            _ => result.push(c),
        }
    }
    result
}

/// Normalize a wikilink target for matching
pub fn normalize_target(target: &str) -> String {
    target
        .trim()
        .to_lowercase()
        .replace(['/', '\\'], "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_wikilink() {
        let parser = WikiLinkParser::new();
        let links = parser.parse("Check out [[My Note]] for more info.");

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "My Note");
        assert_eq!(links[0].display, None);
        assert_eq!(links[0].display_text(), "My Note");
    }

    #[test]
    fn test_wikilink_with_display() {
        let parser = WikiLinkParser::new();
        let links = parser.parse("See [[My Note|this note]] for details.");

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "My Note");
        assert_eq!(links[0].display, Some("this note".to_string()));
        assert_eq!(links[0].display_text(), "this note");
    }

    #[test]
    fn test_wikilink_with_heading() {
        let parser = WikiLinkParser::new();
        let links = parser.parse("Refer to [[Note#Section]] for the details.");

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Note");
        assert_eq!(links[0].heading, Some("Section".to_string()));
        assert!(links[0].has_subpath());
        assert_eq!(links[0].full_path(), "Note#Section");
    }

    #[test]
    fn test_wikilink_with_block() {
        let parser = WikiLinkParser::new();
        let links = parser.parse("See [[Note#^abc123]] for the quote.");

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Note");
        assert_eq!(links[0].block_id, Some("abc123".to_string()));
        assert_eq!(links[0].full_path(), "Note#^abc123");
    }

    #[test]
    fn test_multiple_wikilinks() {
        let parser = WikiLinkParser::new();
        let content = "Links: [[Note A]], [[Note B|B]], and [[Note C#heading]].";
        let links = parser.parse(content);

        assert_eq!(links.len(), 3);
        assert_eq!(links[0].target, "Note A");
        assert_eq!(links[1].target, "Note B");
        assert_eq!(links[1].display, Some("B".to_string()));
        assert_eq!(links[2].target, "Note C");
        assert_eq!(links[2].heading, Some("heading".to_string()));
    }

    #[test]
    fn test_wikilink_position() {
        let parser = WikiLinkParser::new();
        let content = "Hello [[World]]!";
        let links = parser.parse(content);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].position.start.offset, 6);
        assert_eq!(links[0].position.end.offset, 15);
    }

    #[test]
    fn test_normalize_target() {
        assert_eq!(normalize_target("  My Note  "), "my note");
        assert_eq!(normalize_target("folder/note"), "folder/note");
        assert_eq!(normalize_target("folder\\note"), "folder/note");
    }
}
