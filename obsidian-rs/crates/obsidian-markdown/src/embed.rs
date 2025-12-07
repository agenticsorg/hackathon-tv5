//! Embed parser for ![[embed]] syntax

use obsidian_core::position::{Location, Position};
use regex::Regex;
use std::sync::LazyLock;

/// Regex for matching embeds: ![[target]]
static EMBED_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!\[\[([^\[\]]+)\]\]").expect("Invalid embed regex"));

/// Type of embedded content
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedType {
    /// Embedded note
    Note,
    /// Image file
    Image,
    /// PDF document
    Pdf,
    /// Audio file
    Audio,
    /// Video file
    Video,
    /// Unknown file type
    Unknown,
}

impl EmbedType {
    /// Determine embed type from filename
    pub fn from_filename(name: &str) -> Self {
        let lower = name.to_lowercase();

        if lower.ends_with(".png")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".gif")
            || lower.ends_with(".webp")
            || lower.ends_with(".svg")
            || lower.ends_with(".bmp")
        {
            Self::Image
        } else if lower.ends_with(".pdf") {
            Self::Pdf
        } else if lower.ends_with(".mp3")
            || lower.ends_with(".wav")
            || lower.ends_with(".ogg")
            || lower.ends_with(".flac")
            || lower.ends_with(".m4a")
        {
            Self::Audio
        } else if lower.ends_with(".mp4")
            || lower.ends_with(".webm")
            || lower.ends_with(".mov")
            || lower.ends_with(".avi")
        {
            Self::Video
        } else if lower.ends_with(".md") || !lower.contains('.') {
            Self::Note
        } else {
            Self::Unknown
        }
    }
}

/// A parsed embed
#[derive(Debug, Clone, PartialEq)]
pub struct Embed {
    /// Target file path or name
    pub target: String,
    /// Display text or alt text
    pub display: Option<String>,
    /// Target heading (for note embeds)
    pub heading: Option<String>,
    /// Target block ID (for note embeds)
    pub block_id: Option<String>,
    /// Embed dimensions (for images)
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// Original text
    pub original: String,
    /// Embed type
    pub embed_type: EmbedType,
    /// Position in source
    pub position: Position,
}

impl Embed {
    /// Create a new embed
    pub fn new(target: impl Into<String>, position: Position) -> Self {
        let target = target.into();
        let embed_type = EmbedType::from_filename(&target);
        let original = format!("![[{}]]", target);

        Self {
            target,
            display: None,
            heading: None,
            block_id: None,
            width: None,
            height: None,
            original,
            embed_type,
            position,
        }
    }

    /// Check if this is an image embed
    pub fn is_image(&self) -> bool {
        self.embed_type == EmbedType::Image
    }

    /// Check if this is a note embed
    pub fn is_note(&self) -> bool {
        self.embed_type == EmbedType::Note
    }

    /// Check if this is a PDF embed
    pub fn is_pdf(&self) -> bool {
        self.embed_type == EmbedType::Pdf
    }

    /// Check if this is a media embed (audio or video)
    pub fn is_media(&self) -> bool {
        matches!(self.embed_type, EmbedType::Audio | EmbedType::Video)
    }

    /// Get the full embed path including subpath
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

    /// Convert to an EmbedCache
    pub fn to_embed_cache(&self) -> obsidian_core::metadata::EmbedCache {
        obsidian_core::metadata::EmbedCache::new(&self.target, &self.original, self.position)
    }
}

/// Parser for embeds
#[derive(Debug, Default)]
pub struct EmbedParser;

impl EmbedParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self
    }

    /// Parse all embeds from content
    pub fn parse(&self, content: &str) -> Vec<Embed> {
        let mut embeds = Vec::new();

        for cap in EMBED_REGEX.captures_iter(content) {
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

            let embed = self.parse_inner(inner, full_match.as_str(), position);
            embeds.push(embed);
        }

        embeds
    }

    /// Parse the inner content of an embed
    fn parse_inner(&self, inner: &str, original: &str, position: Position) -> Embed {
        // Check for display/alt text: ![[target|alt]]
        let (target_part, display) = if let Some(pos) = inner.find('|') {
            let (target, display) = inner.split_at(pos);
            (target.to_string(), Some(display[1..].to_string()))
        } else {
            (inner.to_string(), None)
        };

        // Parse dimensions from display text for images (e.g., "200x300" or "200")
        let (display, width, height) = if let Some(ref disp) = display {
            if let Some((w, h)) = parse_dimensions(disp) {
                (None, Some(w), h)
            } else {
                (display, None, None)
            }
        } else {
            (None, None, None)
        };

        // Check for heading/block reference
        let (target, heading, block_id) = if let Some(hash_pos) = target_part.find('#') {
            let (note, subpath) = target_part.split_at(hash_pos);
            let subpath = &subpath[1..];

            if subpath.starts_with('^') {
                (note.to_string(), None, Some(subpath[1..].to_string()))
            } else {
                (note.to_string(), Some(subpath.to_string()), None)
            }
        } else {
            (target_part, None, None)
        };

        let embed_type = EmbedType::from_filename(&target);

        Embed {
            target,
            display,
            heading,
            block_id,
            width,
            height,
            original: original.to_string(),
            embed_type,
            position,
        }
    }

    /// Parse a single embed string (without the ![[]])
    pub fn parse_single(&self, inner: &str) -> Embed {
        self.parse_inner(inner, &format!("![[{}]]", inner), Position::default())
    }
}

/// Parse dimension string like "200x300" or "200"
fn parse_dimensions(s: &str) -> Option<(u32, Option<u32>)> {
    let s = s.trim();

    if let Some(x_pos) = s.find('x') {
        let width = s[..x_pos].parse().ok()?;
        let height = s[x_pos + 1..].parse().ok();
        Some((width, height))
    } else {
        let width = s.parse().ok()?;
        Some((width, None))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_type_detection() {
        assert_eq!(EmbedType::from_filename("image.png"), EmbedType::Image);
        assert_eq!(EmbedType::from_filename("doc.pdf"), EmbedType::Pdf);
        assert_eq!(EmbedType::from_filename("song.mp3"), EmbedType::Audio);
        assert_eq!(EmbedType::from_filename("video.mp4"), EmbedType::Video);
        assert_eq!(EmbedType::from_filename("note.md"), EmbedType::Note);
        assert_eq!(EmbedType::from_filename("Note Name"), EmbedType::Note);
    }

    #[test]
    fn test_simple_embed() {
        let parser = EmbedParser::new();
        let embeds = parser.parse("Here is ![[image.png]] embedded.");

        assert_eq!(embeds.len(), 1);
        assert_eq!(embeds[0].target, "image.png");
        assert!(embeds[0].is_image());
    }

    #[test]
    fn test_note_embed() {
        let parser = EmbedParser::new();
        let embeds = parser.parse("![[My Note]] is embedded here.");

        assert_eq!(embeds.len(), 1);
        assert_eq!(embeds[0].target, "My Note");
        assert!(embeds[0].is_note());
    }

    #[test]
    fn test_embed_with_heading() {
        let parser = EmbedParser::new();
        let embeds = parser.parse("![[Note#Section]]");

        assert_eq!(embeds.len(), 1);
        assert_eq!(embeds[0].target, "Note");
        assert_eq!(embeds[0].heading, Some("Section".to_string()));
    }

    #[test]
    fn test_embed_with_block() {
        let parser = EmbedParser::new();
        let embeds = parser.parse("![[Note#^abc123]]");

        assert_eq!(embeds.len(), 1);
        assert_eq!(embeds[0].target, "Note");
        assert_eq!(embeds[0].block_id, Some("abc123".to_string()));
    }

    #[test]
    fn test_image_with_dimensions() {
        let parser = EmbedParser::new();
        let embeds = parser.parse("![[image.png|200x300]]");

        assert_eq!(embeds.len(), 1);
        assert_eq!(embeds[0].target, "image.png");
        assert_eq!(embeds[0].width, Some(200));
        assert_eq!(embeds[0].height, Some(300));
    }

    #[test]
    fn test_image_with_width_only() {
        let parser = EmbedParser::new();
        let embeds = parser.parse("![[image.png|400]]");

        assert_eq!(embeds.len(), 1);
        assert_eq!(embeds[0].width, Some(400));
        assert_eq!(embeds[0].height, None);
    }

    #[test]
    fn test_multiple_embeds() {
        let parser = EmbedParser::new();
        let content = "![[image1.png]] and ![[note]] and ![[doc.pdf]]";
        let embeds = parser.parse(content);

        assert_eq!(embeds.len(), 3);
        assert!(embeds[0].is_image());
        assert!(embeds[1].is_note());
        assert!(embeds[2].is_pdf());
    }
}
