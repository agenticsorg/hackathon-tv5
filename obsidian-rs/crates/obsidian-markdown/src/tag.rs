//! Tag parser for #tag syntax

use obsidian_core::position::{Location, Position};
use regex::Regex;
use std::sync::LazyLock;

/// Regex for matching tags: #tag or #nested/tag
static TAG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|[^\w#])#([a-zA-Z][a-zA-Z0-9_/-]*)").expect("Invalid tag regex"));

/// A parsed tag
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    /// Tag name (without #)
    pub name: String,
    /// Original text including #
    pub original: String,
    /// Position in source
    pub position: Position,
}

impl Tag {
    /// Create a new tag
    pub fn new(name: impl Into<String>, position: Position) -> Self {
        let name = name.into();
        let original = format!("#{}", name);
        Self {
            name,
            original,
            position,
        }
    }

    /// Get tag segments for nested tags
    pub fn segments(&self) -> Vec<&str> {
        self.name.split('/').collect()
    }

    /// Check if this is a nested tag
    pub fn is_nested(&self) -> bool {
        self.name.contains('/')
    }

    /// Get the parent tag name (if nested)
    pub fn parent(&self) -> Option<&str> {
        if self.is_nested() {
            self.name.rsplit_once('/').map(|(parent, _)| parent)
        } else {
            None
        }
    }

    /// Get all ancestor tags (for nested tags)
    pub fn ancestors(&self) -> Vec<String> {
        let segments = self.segments();
        let mut ancestors = Vec::new();
        let mut current = String::new();

        for (i, segment) in segments.iter().enumerate() {
            if i > 0 {
                current.push('/');
            }
            current.push_str(segment);
            if i < segments.len() - 1 {
                ancestors.push(current.clone());
            }
        }

        ancestors
    }

    /// Get the leaf tag name (last segment)
    pub fn leaf(&self) -> &str {
        self.name.rsplit('/').next().unwrap_or(&self.name)
    }

    /// Convert to a TagCache
    pub fn to_tag_cache(&self) -> obsidian_core::metadata::TagCache {
        obsidian_core::metadata::TagCache::new(&self.name, self.position)
    }
}

/// Parser for tags
#[derive(Debug, Default)]
pub struct TagParser;

impl TagParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self
    }

    /// Parse all tags from content
    pub fn parse(&self, content: &str) -> Vec<Tag> {
        let mut tags = Vec::new();

        for cap in TAG_REGEX.captures_iter(content) {
            let tag_name = cap.get(1).unwrap();

            // Get the actual position of the # character
            let full_match = cap.get(0).unwrap();
            let hash_offset = full_match.as_str().find('#').unwrap();
            let start_offset = full_match.start() + hash_offset;
            let end_offset = tag_name.end();

            let (start_line, start_col) = offset_to_line_col(content, start_offset);
            let (end_line, end_col) = offset_to_line_col(content, end_offset);

            let position = Position::new(
                Location::new(start_line, start_col, start_offset),
                Location::new(end_line, end_col, end_offset),
            );

            let tag = Tag::new(tag_name.as_str(), position);
            tags.push(tag);
        }

        tags
    }

    /// Parse tags from frontmatter tags field
    pub fn parse_frontmatter_tags(&self, tags: &[String]) -> Vec<Tag> {
        tags.iter()
            .map(|name| {
                let name = name.strip_prefix('#').unwrap_or(name);
                Tag::new(name, Position::default())
            })
            .collect()
    }

    /// Validate a tag name
    pub fn is_valid_tag(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let first_char = name.chars().next().unwrap();
        if !first_char.is_alphabetic() {
            return false;
        }

        name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '/')
    }
}

/// Normalize a tag name for comparison
pub fn normalize_tag(name: &str) -> String {
    name.trim()
        .trim_start_matches('#')
        .to_lowercase()
}

/// Get all unique tags with their counts
pub fn tag_counts(tags: &[Tag]) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    for tag in tags {
        *counts.entry(tag.name.clone()).or_insert(0) += 1;
    }
    counts
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tag() {
        let parser = TagParser::new();
        let tags = parser.parse("This is a #tag example.");

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "tag");
        assert!(!tags[0].is_nested());
    }

    #[test]
    fn test_nested_tag() {
        let parser = TagParser::new();
        let tags = parser.parse("Using #project/work/important here.");

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "project/work/important");
        assert!(tags[0].is_nested());
        assert_eq!(tags[0].segments(), vec!["project", "work", "important"]);
        assert_eq!(tags[0].parent(), Some("project/work"));
        assert_eq!(tags[0].leaf(), "important");
    }

    #[test]
    fn test_tag_ancestors() {
        let tag = Tag::new("a/b/c", Position::default());
        let ancestors = tag.ancestors();
        assert_eq!(ancestors, vec!["a", "a/b"]);
    }

    #[test]
    fn test_multiple_tags() {
        let parser = TagParser::new();
        let content = "Multiple #tags #here and #nested/tag too.";
        let tags = parser.parse(content);

        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0].name, "tags");
        assert_eq!(tags[1].name, "here");
        assert_eq!(tags[2].name, "nested/tag");
    }

    #[test]
    fn test_tag_at_start() {
        let parser = TagParser::new();
        let tags = parser.parse("#starttag is at the beginning");

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "starttag");
    }

    #[test]
    fn test_tag_validation() {
        assert!(TagParser::is_valid_tag("valid"));
        assert!(TagParser::is_valid_tag("Valid123"));
        assert!(TagParser::is_valid_tag("tag-with-dash"));
        assert!(TagParser::is_valid_tag("tag_with_underscore"));
        assert!(TagParser::is_valid_tag("nested/tag"));

        assert!(!TagParser::is_valid_tag(""));
        assert!(!TagParser::is_valid_tag("123invalid"));
        assert!(!TagParser::is_valid_tag("-invalid"));
    }

    #[test]
    fn test_normalize_tag() {
        assert_eq!(normalize_tag("  #Tag  "), "tag");
        assert_eq!(normalize_tag("TAG"), "tag");
        assert_eq!(normalize_tag("#nested/TAG"), "nested/tag");
    }

    #[test]
    fn test_tag_counts() {
        let tags = vec![
            Tag::new("a", Position::default()),
            Tag::new("b", Position::default()),
            Tag::new("a", Position::default()),
            Tag::new("c", Position::default()),
            Tag::new("a", Position::default()),
        ];

        let counts = tag_counts(&tags);
        assert_eq!(counts.get("a"), Some(&3));
        assert_eq!(counts.get("b"), Some(&1));
        assert_eq!(counts.get("c"), Some(&1));
    }

    #[test]
    fn test_no_false_positives() {
        let parser = TagParser::new();

        // Should not match these
        let content = "email@example.com #123 ##double C#sharp";
        let tags = parser.parse(content);

        // Only valid tags should be found
        assert!(tags.iter().all(|t| TagParser::is_valid_tag(&t.name)));
    }
}
