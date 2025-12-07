//! Markdown renderer for converting to HTML

use crate::parser::MarkdownParser;
use pulldown_cmark::{html, Event, Options, Parser, Tag as CmarkTag};
use regex::Regex;
use std::sync::LazyLock;

/// Regex for highlights: ==text==
static HIGHLIGHT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"==([^=]+)==").expect("Invalid highlight regex"));

/// Regex for comments: %%text%%
static COMMENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"%%[^%]*%%").expect("Invalid comment regex"));

/// Options for rendering
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Base path for resolving links
    pub base_path: String,
    /// Whether to resolve wikilinks
    pub resolve_links: bool,
    /// Whether to include highlights
    pub highlights: bool,
    /// Whether to strip comments
    pub strip_comments: bool,
    /// Link resolver function
    pub link_resolver: Option<LinkResolver>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            base_path: String::new(),
            resolve_links: true,
            highlights: true,
            strip_comments: true,
            link_resolver: None,
        }
    }
}

/// Function type for resolving links
pub type LinkResolver = Box<dyn Fn(&str) -> Option<String> + Send + Sync>;

/// Markdown renderer
#[derive(Default)]
pub struct MarkdownRenderer {
    parser: MarkdownParser,
}

impl MarkdownRenderer {
    /// Create a new renderer
    pub fn new() -> Self {
        Self {
            parser: MarkdownParser::new(),
        }
    }

    /// Render markdown to HTML
    pub fn render(&self, content: &str) -> String {
        self.render_with_options(content, &RenderOptions::default())
    }

    /// Render markdown to HTML with options
    pub fn render_with_options(&self, content: &str, options: &RenderOptions) -> String {
        // Parse the markdown first
        let parsed = self.parser.parse(content);

        // Pre-process custom syntax
        let processed = self.preprocess(&parsed.content, options);

        // Convert to HTML using pulldown-cmark
        let parser_options = Options::all();
        let parser = Parser::new_ext(&processed, parser_options);

        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // Post-process HTML
        self.postprocess(&html_output)
    }

    /// Render to plain text (strip all formatting)
    pub fn render_plain(&self, content: &str) -> String {
        let parsed = self.parser.parse(content);
        let mut plain = String::new();

        // Strip comments
        let content = COMMENT_REGEX.replace_all(&parsed.content, "");

        // Use pulldown-cmark to extract text
        let parser = Parser::new(&content);

        for event in parser {
            if let Event::Text(text) = event {
                plain.push_str(&text);
            } else if let Event::SoftBreak | Event::HardBreak = event {
                plain.push(' ');
            }
        }

        plain.trim().to_string()
    }

    /// Pre-process custom Obsidian syntax
    fn preprocess(&self, content: &str, options: &RenderOptions) -> String {
        let mut result = content.to_string();

        // Strip comments if enabled
        if options.strip_comments {
            result = COMMENT_REGEX.replace_all(&result, "").to_string();
        }

        // Convert wikilinks to HTML links
        if options.resolve_links {
            result = self.convert_wikilinks(&result, options);
        }

        // Convert embeds to HTML
        result = self.convert_embeds(&result, options);

        // Convert tags to clickable links
        result = self.convert_tags(&result, options);

        // Convert highlights
        if options.highlights {
            result = HIGHLIGHT_REGEX
                .replace_all(&result, "<mark>$1</mark>")
                .to_string();
        }

        result
    }

    /// Convert wikilinks to HTML
    fn convert_wikilinks(&self, content: &str, options: &RenderOptions) -> String {
        let wikilink_regex = Regex::new(r"\[\[([^\[\]]+)\]\]").unwrap();

        wikilink_regex
            .replace_all(content, |caps: &regex::Captures| {
                let inner = &caps[1];

                // Parse link parts
                let (target, display) = if let Some(pos) = inner.find('|') {
                    let (t, d) = inner.split_at(pos);
                    (t.to_string(), d[1..].to_string())
                } else {
                    (inner.to_string(), inner.to_string())
                };

                // Resolve link
                let href = if let Some(ref resolver) = options.link_resolver {
                    resolver(&target).unwrap_or_else(|| format!("#{}", urlencoding::encode(&target)))
                } else {
                    format!(
                        "{}/{}",
                        options.base_path,
                        urlencoding::encode(&target)
                    )
                };

                // Check if link resolves
                let class = if options.link_resolver.is_some() {
                    if options.link_resolver.as_ref().unwrap()(&target).is_some() {
                        "internal-link"
                    } else {
                        "internal-link is-unresolved"
                    }
                } else {
                    "internal-link"
                };

                format!(
                    r#"<a href="{}" class="{}" data-href="{}">{}</a>"#,
                    href, class, target, display
                )
            })
            .to_string()
    }

    /// Convert embeds to HTML
    fn convert_embeds(&self, content: &str, options: &RenderOptions) -> String {
        let embed_regex = Regex::new(r"!\[\[([^\[\]]+)\]\]").unwrap();

        embed_regex
            .replace_all(content, |caps: &regex::Captures| {
                let inner = &caps[1];

                // Parse embed parts
                let (target, alt) = if let Some(pos) = inner.find('|') {
                    let (t, a) = inner.split_at(pos);
                    (t.to_string(), a[1..].to_string())
                } else {
                    (inner.to_string(), inner.to_string())
                };

                let lower = target.to_lowercase();

                // Determine embed type and generate HTML
                if lower.ends_with(".png")
                    || lower.ends_with(".jpg")
                    || lower.ends_with(".jpeg")
                    || lower.ends_with(".gif")
                    || lower.ends_with(".webp")
                    || lower.ends_with(".svg")
                {
                    // Parse dimensions from alt text
                    let (alt_text, width, height) = parse_image_dimensions(&alt);

                    let src = format!(
                        "{}/{}",
                        options.base_path,
                        urlencoding::encode(&target)
                    );

                    let mut attrs = format!(r#"src="{}" alt="{}""#, src, alt_text);
                    if let Some(w) = width {
                        attrs.push_str(&format!(r#" width="{}""#, w));
                    }
                    if let Some(h) = height {
                        attrs.push_str(&format!(r#" height="{}""#, h));
                    }

                    format!(r#"<img {} class="internal-embed"/>"#, attrs)
                } else if lower.ends_with(".pdf") {
                    let src = format!(
                        "{}/{}",
                        options.base_path,
                        urlencoding::encode(&target)
                    );
                    format!(
                        r#"<embed src="{}" type="application/pdf" class="internal-embed pdf-embed"/>"#,
                        src
                    )
                } else if lower.ends_with(".mp3")
                    || lower.ends_with(".wav")
                    || lower.ends_with(".ogg")
                {
                    let src = format!(
                        "{}/{}",
                        options.base_path,
                        urlencoding::encode(&target)
                    );
                    format!(
                        r#"<audio controls src="{}" class="internal-embed"></audio>"#,
                        src
                    )
                } else if lower.ends_with(".mp4")
                    || lower.ends_with(".webm")
                    || lower.ends_with(".mov")
                {
                    let src = format!(
                        "{}/{}",
                        options.base_path,
                        urlencoding::encode(&target)
                    );
                    format!(
                        r#"<video controls src="{}" class="internal-embed"></video>"#,
                        src
                    )
                } else {
                    // Note embed - create a placeholder
                    format!(
                        r#"<div class="internal-embed note-embed" data-src="{}">{}</div>"#,
                        target, alt
                    )
                }
            })
            .to_string()
    }

    /// Convert tags to clickable links
    fn convert_tags(&self, content: &str, options: &RenderOptions) -> String {
        let tag_regex = Regex::new(r"(?:^|[^\w#])#([a-zA-Z][a-zA-Z0-9_/-]*)").unwrap();

        tag_regex
            .replace_all(content, |caps: &regex::Captures| {
                let full_match = caps.get(0).unwrap().as_str();
                let tag = &caps[1];
                let prefix = if full_match.starts_with('#') {
                    ""
                } else {
                    &full_match[..1]
                };

                format!(
                    r#"{}<a href="{}/tag/{}" class="tag">#{}</a>"#,
                    prefix, options.base_path, tag, tag
                )
            })
            .to_string()
    }

    /// Post-process generated HTML
    fn postprocess(&self, html: &str) -> String {
        // Add classes to certain elements, fix any issues
        html.replace("<pre><code", r#"<pre class="code-block"><code"#)
            .replace("<blockquote>", r#"<blockquote class="callout">"#)
    }
}

/// Parse image dimensions from alt text (e.g., "200x300" or "200")
fn parse_image_dimensions(alt: &str) -> (String, Option<u32>, Option<u32>) {
    let dimension_regex = Regex::new(r"^(\d+)(?:x(\d+))?$").unwrap();

    if let Some(caps) = dimension_regex.captures(alt) {
        let width = caps.get(1).and_then(|m| m.as_str().parse().ok());
        let height = caps.get(2).and_then(|m| m.as_str().parse().ok());
        (String::new(), width, height)
    } else {
        (alt.to_string(), None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_render() {
        let renderer = MarkdownRenderer::new();
        let html = renderer.render("# Hello World\n\nSome **bold** text.");

        assert!(html.contains("<h1>Hello World</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_wikilink_render() {
        let renderer = MarkdownRenderer::new();
        let html = renderer.render("Check out [[My Note]] for more.");

        assert!(html.contains("internal-link"));
        assert!(html.contains("My Note"));
    }

    #[test]
    fn test_wikilink_with_display() {
        let renderer = MarkdownRenderer::new();
        let html = renderer.render("See [[Note|click here]].");

        assert!(html.contains("click here"));
        assert!(html.contains(r#"data-href="Note""#));
    }

    #[test]
    fn test_image_embed() {
        let renderer = MarkdownRenderer::new();
        let html = renderer.render("![[image.png]]");

        assert!(html.contains("<img"));
        assert!(html.contains("internal-embed"));
    }

    #[test]
    fn test_image_with_dimensions() {
        let renderer = MarkdownRenderer::new();
        let html = renderer.render("![[image.png|200x300]]");

        assert!(html.contains(r#"width="200""#));
        assert!(html.contains(r#"height="300""#));
    }

    #[test]
    fn test_tag_render() {
        let renderer = MarkdownRenderer::new();
        let html = renderer.render("This has a #tag.");

        assert!(html.contains(r#"class="tag""#));
        assert!(html.contains("#tag"));
    }

    #[test]
    fn test_highlight_render() {
        let renderer = MarkdownRenderer::new();
        let html = renderer.render("This is ==highlighted== text.");

        assert!(html.contains("<mark>highlighted</mark>"));
    }

    #[test]
    fn test_comment_strip() {
        let renderer = MarkdownRenderer::new();
        let html = renderer.render("Visible %%hidden%% text");

        assert!(!html.contains("hidden"));
        assert!(html.contains("Visible"));
        assert!(html.contains("text"));
    }

    #[test]
    fn test_plain_text() {
        let renderer = MarkdownRenderer::new();
        let plain = renderer.render_plain("# Hello\n\n**Bold** and *italic*.");

        assert_eq!(plain, "Hello Bold and italic.");
    }
}
