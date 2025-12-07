//! Markdown rendering commands

use obsidian_markdown::{MarkdownParser, MarkdownRenderer, RenderOptions};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::debug;

use crate::state::AppState;

/// Render markdown to HTML
#[tauri::command]
pub async fn render_markdown(
    content: String,
    base_path: Option<String>,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    debug!("Rendering markdown");

    let renderer = MarkdownRenderer::new();

    let options = RenderOptions {
        base_path: base_path.unwrap_or_default(),
        resolve_links: true,
        highlights: true,
        strip_comments: true,
        link_resolver: None,
    };

    Ok(renderer.render_with_options(&content, &options))
}

/// Parsed frontmatter response
#[derive(Debug, Serialize)]
pub struct FrontmatterResponse {
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Parse frontmatter from markdown
#[tauri::command]
pub async fn parse_frontmatter(content: String) -> Result<Option<FrontmatterResponse>, String> {
    debug!("Parsing frontmatter");

    let parser = MarkdownParser::new();
    let parsed = parser.parse(&content);

    match parsed.frontmatter {
        Some(fm) => Ok(Some(FrontmatterResponse {
            title: fm.title,
            tags: fm.tags,
            aliases: fm.aliases,
            created: fm.created.map(|dt| dt.to_rfc3339()),
            modified: fm.modified.map(|dt| dt.to_rfc3339()),
            extra: fm.extra,
        })),
        None => Ok(None),
    }
}
