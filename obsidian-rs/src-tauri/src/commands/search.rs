//! Search commands

use crate::state::AppState;
use obsidian_search::{MatchType, SearchHit, SearchOptions};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::debug;

/// Search result for frontend
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub score: f32,
    pub snippet: Option<String>,
    pub match_type: String,
}

impl From<SearchHit> for SearchResult {
    fn from(hit: SearchHit) -> Self {
        Self {
            path: hit.path.to_string_lossy().to_string(),
            title: hit.title,
            score: hit.score,
            snippet: hit.snippet,
            match_type: match hit.match_type {
                MatchType::Title => "title".to_string(),
                MatchType::Content => "content".to_string(),
                MatchType::Heading => "heading".to_string(),
                MatchType::Tag => "tag".to_string(),
                MatchType::Semantic => "semantic".to_string(),
                MatchType::Fuzzy => "fuzzy".to_string(),
            },
        }
    }
}

/// Search notes by query
#[tauri::command]
pub async fn search_notes(
    query: String,
    limit: Option<usize>,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<SearchResult>, String> {
    debug!("Searching notes: {}", query);

    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let search = state.search.read();

    if let Some(ref engine) = *search {
        let options = SearchOptions {
            limit,
            search_titles: true,
            search_content: true,
            search_tags: true,
            fuzzy: true,
            ..Default::default()
        };

        let results = engine
            .search(&query, &options)
            .map_err(|e| format!("Search failed: {}", e))?;

        Ok(results.into_iter().map(SearchResult::from).collect())
    } else {
        // Fallback to vault search if search engine not initialized
        let vault = state.vault.read();
        if let Some(ref vault) = *vault {
            let files = vault.search_by_name(&query);
            Ok(files
                .into_iter()
                .take(limit.unwrap_or(20))
                .map(|path| SearchResult {
                    title: path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    path: path.to_string_lossy().to_string(),
                    score: 1.0,
                    snippet: None,
                    match_type: "fuzzy".to_string(),
                })
                .collect())
        } else {
            Err("No vault open".to_string())
        }
    }
}

/// Search content within notes
#[tauri::command]
pub async fn search_content(
    query: String,
    limit: Option<usize>,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<SearchResult>, String> {
    debug!("Searching content: {}", query);

    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let search = state.search.read();

    if let Some(ref engine) = *search {
        let options = SearchOptions {
            limit,
            search_titles: false,
            search_content: true,
            search_tags: false,
            fuzzy: false,
            ..Default::default()
        };

        let results = engine
            .search(&query, &options)
            .map_err(|e| format!("Search failed: {}", e))?;

        Ok(results.into_iter().map(SearchResult::from).collect())
    } else {
        Ok(Vec::new())
    }
}

/// Quick switch (fast file search)
#[tauri::command]
pub async fn quick_switch(
    query: String,
    limit: Option<usize>,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<SearchResult>, String> {
    debug!("Quick switch: {}", query);

    let vault = state.vault.read();
    let vault = vault
        .as_ref()
        .ok_or_else(|| "No vault open".to_string())?;

    if query.trim().is_empty() {
        // Return recent files
        let open_files = state.open_files.read();
        return Ok(open_files
            .iter()
            .take(limit.unwrap_or(10))
            .map(|path| SearchResult {
                title: path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default(),
                path: path.to_string_lossy().to_string(),
                score: 1.0,
                snippet: None,
                match_type: "recent".to_string(),
            })
            .collect());
    }

    // Search by name
    let files = vault.search_by_name(&query);

    Ok(files
        .into_iter()
        .take(limit.unwrap_or(20))
        .map(|path| SearchResult {
            title: path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default(),
            path: path.to_string_lossy().to_string(),
            score: 1.0,
            snippet: None,
            match_type: "fuzzy".to_string(),
        })
        .collect())
}
