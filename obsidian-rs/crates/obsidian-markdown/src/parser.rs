//! Main markdown parser combining all components

use crate::embed::{Embed, EmbedParser};
use crate::frontmatter::{FrontmatterParser, FrontmatterResult};
use crate::tag::{Tag, TagParser};
use crate::wikilink::{WikiLink, WikiLinkParser};
use obsidian_core::metadata::{BlockCache, CachedMetadata, HeadingCache};
use obsidian_core::note::Frontmatter;
use obsidian_core::position::{Location, Position};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag as CmarkTag};
use regex::Regex;
use std::sync::LazyLock;

/// Regex for block IDs: ^block-id
static BLOCK_ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s\^([a-zA-Z0-9-]+)\s*$").expect("Invalid block ID regex"));

/// Result of parsing markdown
#[derive(Debug, Clone)]
pub struct ParsedMarkdown {
    /// Original raw content
    pub raw: String,
    /// Content without frontmatter
    pub content: String,
    /// Parsed frontmatter
    pub frontmatter: Option<Frontmatter>,
    /// Frontmatter position
    pub frontmatter_position: Option<Position>,
    /// All wikilinks
    pub links: Vec<WikiLink>,
    /// All embeds
    pub embeds: Vec<Embed>,
    /// All tags
    pub tags: Vec<Tag>,
    /// All headings
    pub headings: Vec<HeadingCache>,
    /// All block IDs
    pub blocks: Vec<BlockCache>,
}

impl ParsedMarkdown {
    /// Convert to CachedMetadata
    pub fn to_cached_metadata(&self) -> CachedMetadata {
        let mut metadata = CachedMetadata::default();

        metadata.links = self.links.iter().map(|l| l.to_link_cache()).collect();
        metadata.embeds = self.embeds.iter().map(|e| e.to_embed_cache()).collect();
        metadata.tags = self.tags.iter().map(|t| t.to_tag_cache()).collect();
        metadata.headings = self.headings.clone();
        metadata.blocks = self
            .blocks
            .iter()
            .map(|b| (b.id.clone(), b.clone()))
            .collect();
        metadata.frontmatter = self.frontmatter.clone();
        metadata.frontmatter_position = self.frontmatter_position;

        metadata
    }

    /// Get all unique link targets
    pub fn link_targets(&self) -> Vec<&str> {
        let mut targets: Vec<&str> = self.links.iter().map(|l| l.target.as_str()).collect();
        targets.sort();
        targets.dedup();
        targets
    }

    /// Get all unique tag names
    pub fn tag_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.tags.iter().map(|t| t.name.as_str()).collect();
        names.sort();
        names.dedup();
        names
    }

    /// Get the table of contents
    pub fn toc(&self) -> Vec<(&str, u8)> {
        self.headings
            .iter()
            .map(|h| (h.heading.as_str(), h.level))
            .collect()
    }
}

/// Main markdown parser
#[derive(Debug)]
pub struct MarkdownParser {
    frontmatter_parser: FrontmatterParser,
    wikilink_parser: WikiLinkParser,
    embed_parser: EmbedParser,
    tag_parser: TagParser,
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            frontmatter_parser: FrontmatterParser::new(),
            wikilink_parser: WikiLinkParser::new(),
            embed_parser: EmbedParser::new(),
            tag_parser: TagParser::new(),
        }
    }

    /// Parse markdown content
    pub fn parse(&self, content: &str) -> ParsedMarkdown {
        // Parse frontmatter first
        let FrontmatterResult {
            frontmatter,
            position: frontmatter_position,
            content: body,
            content_start,
        } = self.frontmatter_parser.parse(content);

        // Parse Obsidian-specific syntax
        let links = self.parse_wikilinks(&body, content_start);
        let embeds = self.parse_embeds(&body, content_start);
        let mut tags = self.parse_tags(&body, content_start);

        // Add frontmatter tags
        if let Some(ref fm) = frontmatter {
            let fm_tags = self.tag_parser.parse_frontmatter_tags(&fm.tags);
            tags.extend(fm_tags);
        }

        // Parse headings and blocks using pulldown-cmark
        let (headings, blocks) = self.parse_structure(&body, content_start);

        ParsedMarkdown {
            raw: content.to_string(),
            content: body,
            frontmatter,
            frontmatter_position,
            links,
            embeds,
            tags,
            headings,
            blocks,
        }
    }

    /// Parse wikilinks with offset adjustment
    fn parse_wikilinks(&self, content: &str, offset: usize) -> Vec<WikiLink> {
        let mut links = self.wikilink_parser.parse(content);

        // Adjust positions by offset
        for link in &mut links {
            link.position.start.offset += offset;
            link.position.end.offset += offset;
        }

        links
    }

    /// Parse embeds with offset adjustment
    fn parse_embeds(&self, content: &str, offset: usize) -> Vec<Embed> {
        let mut embeds = self.embed_parser.parse(content);

        // Adjust positions by offset
        for embed in &mut embeds {
            embed.position.start.offset += offset;
            embed.position.end.offset += offset;
        }

        embeds
    }

    /// Parse tags with offset adjustment
    fn parse_tags(&self, content: &str, offset: usize) -> Vec<Tag> {
        let mut tags = self.tag_parser.parse(content);

        // Adjust positions by offset
        for tag in &mut tags {
            tag.position.start.offset += offset;
            tag.position.end.offset += offset;
        }

        tags
    }

    /// Parse structure (headings and blocks) using pulldown-cmark
    fn parse_structure(&self, content: &str, offset: usize) -> (Vec<HeadingCache>, Vec<BlockCache>) {
        let mut headings = Vec::new();
        let mut blocks = Vec::new();

        // Parse with pulldown-cmark
        let options = Options::all();
        let parser = Parser::new_ext(content, options);

        let mut current_heading: Option<(HeadingLevel, usize)> = None;
        let mut heading_text = String::new();
        let mut current_offset = 0;

        for (event, range) in parser.into_offset_iter() {
            match event {
                Event::Start(CmarkTag::Heading(level, ..)) => {
                    current_heading = Some((level, range.start));
                    heading_text.clear();
                }
                Event::End(CmarkTag::Heading(..)) => {
                    if let Some((level, start)) = current_heading.take() {
                        let heading_level = match level {
                            HeadingLevel::H1 => 1,
                            HeadingLevel::H2 => 2,
                            HeadingLevel::H3 => 3,
                            HeadingLevel::H4 => 4,
                            HeadingLevel::H5 => 5,
                            HeadingLevel::H6 => 6,
                        };

                        let (start_line, start_col) = offset_to_line_col(content, start);
                        let (end_line, end_col) = offset_to_line_col(content, range.end);

                        let position = Position::new(
                            Location::new(start_line, start_col, start + offset),
                            Location::new(end_line, end_col, range.end + offset),
                        );

                        headings.push(HeadingCache::new(heading_text.trim(), heading_level, position));
                    }
                }
                Event::Text(text) => {
                    if current_heading.is_some() {
                        heading_text.push_str(&text);
                    }

                    // Check for block IDs
                    if let Some(caps) = BLOCK_ID_REGEX.captures(&text) {
                        if let Some(id_match) = caps.get(1) {
                            let id = id_match.as_str();
                            let (line, col) = offset_to_line_col(content, range.start);
                            let position = Position::new(
                                Location::new(line, col, range.start + offset),
                                Location::new(line, col + text.len(), range.end + offset),
                            );
                            blocks.push(BlockCache::new(id, position));
                        }
                    }
                }
                _ => {}
            }
            current_offset = range.end;
        }

        // Also search for block IDs in the raw content (for edge cases)
        for caps in BLOCK_ID_REGEX.captures_iter(content) {
            if let Some(id_match) = caps.get(1) {
                let full_match = caps.get(0).unwrap();
                let id = id_match.as_str();

                // Check if we already found this block
                if !blocks.iter().any(|b| b.id == id) {
                    let (line, col) = offset_to_line_col(content, full_match.start());
                    let position = Position::new(
                        Location::new(line, col, full_match.start() + offset),
                        Location::new(line, col + full_match.len(), full_match.end() + offset),
                    );
                    blocks.push(BlockCache::new(id, position));
                }
            }
        }

        (headings, blocks)
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
    fn test_full_parse() {
        let parser = MarkdownParser::new();
        let content = r#"---
title: Test Note
tags:
  - test
---
# Heading 1

Some text with [[link]] and #tag.

## Heading 2

![[embedded.png]]

Paragraph with ^block-id
"#;

        let result = parser.parse(content);

        // Check frontmatter
        assert!(result.frontmatter.is_some());
        let fm = result.frontmatter.unwrap();
        assert_eq!(fm.title, Some("Test Note".to_string()));
        assert_eq!(fm.tags, vec!["test"]);

        // Check links
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.links[0].target, "link");

        // Check embeds
        assert_eq!(result.embeds.len(), 1);
        assert_eq!(result.embeds[0].target, "embedded.png");

        // Check tags (frontmatter + inline)
        assert_eq!(result.tags.len(), 2);

        // Check headings
        assert_eq!(result.headings.len(), 2);
        assert_eq!(result.headings[0].heading, "Heading 1");
        assert_eq!(result.headings[0].level, 1);
        assert_eq!(result.headings[1].heading, "Heading 2");
        assert_eq!(result.headings[1].level, 2);

        // Check blocks
        assert_eq!(result.blocks.len(), 1);
        assert_eq!(result.blocks[0].id, "block-id");
    }

    #[test]
    fn test_parse_without_frontmatter() {
        let parser = MarkdownParser::new();
        let content = "# Simple Note\n\n[[link]] #tag";

        let result = parser.parse(content);

        assert!(result.frontmatter.is_none());
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.tags.len(), 1);
        assert_eq!(result.headings.len(), 1);
    }

    #[test]
    fn test_cached_metadata_conversion() {
        let parser = MarkdownParser::new();
        let content = "# Test\n\n[[link]] #tag ![[image.png]]";

        let result = parser.parse(content);
        let metadata = result.to_cached_metadata();

        assert_eq!(metadata.links.len(), 1);
        assert_eq!(metadata.tags.len(), 1);
        assert_eq!(metadata.embeds.len(), 1);
        assert_eq!(metadata.headings.len(), 1);
    }

    #[test]
    fn test_link_targets() {
        let parser = MarkdownParser::new();
        let content = "[[a]] [[b]] [[a]] [[c]]";

        let result = parser.parse(content);
        let targets = result.link_targets();

        assert_eq!(targets, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_toc() {
        let parser = MarkdownParser::new();
        let content = "# H1\n## H2\n### H3\n## Another H2";

        let result = parser.parse(content);
        let toc = result.toc();

        assert_eq!(toc.len(), 4);
        assert_eq!(toc[0], ("H1", 1));
        assert_eq!(toc[1], ("H2", 2));
        assert_eq!(toc[2], ("H3", 3));
        assert_eq!(toc[3], ("Another H2", 2));
    }
}
