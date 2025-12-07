//! ObsidianRS Markdown - Markdown parser with Obsidian syntax extensions
//!
//! This crate provides parsing and rendering for Obsidian-flavored markdown,
//! including wikilinks, embeds, tags, and other Obsidian-specific syntax.

pub mod embed;
pub mod frontmatter;
pub mod parser;
pub mod renderer;
pub mod tag;
pub mod wikilink;

pub use embed::{Embed, EmbedParser, EmbedType};
pub use frontmatter::FrontmatterParser;
pub use parser::{MarkdownParser, ParsedMarkdown};
pub use renderer::MarkdownRenderer;
pub use tag::{Tag, TagParser};
pub use wikilink::{WikiLink, WikiLinkParser};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::embed::{Embed, EmbedParser, EmbedType};
    pub use crate::frontmatter::FrontmatterParser;
    pub use crate::parser::{MarkdownParser, ParsedMarkdown};
    pub use crate::renderer::MarkdownRenderer;
    pub use crate::tag::{Tag, TagParser};
    pub use crate::wikilink::{WikiLink, WikiLinkParser};
}
