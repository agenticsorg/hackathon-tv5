//! Full-text and semantic search for Obsidian-rs
//!
//! This crate provides:
//! - Full-text search using Tantivy
//! - Semantic/vector search using embeddings
//! - Fuzzy file name matching
//! - Combined search results

pub mod error;
pub mod fulltext;
pub mod fuzzy;
pub mod semantic;

pub use error::{SearchError, SearchResult};
pub use fulltext::{FullTextIndex, FullTextSearcher};
pub use fuzzy::{fuzzy_match, FuzzyMatcher};
pub use semantic::{SemanticIndex, VectorStore};

use obsidian_core::note::Note;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    /// File path
    pub path: PathBuf,
    /// Match score (higher is better)
    pub score: f32,
    /// Title of the note
    pub title: String,
    /// Matched content snippet
    pub snippet: Option<String>,
    /// Line number of match
    pub line: Option<usize>,
    /// Type of match
    pub match_type: MatchType,
}

impl SearchHit {
    /// Create a new search hit
    pub fn new(path: impl Into<PathBuf>, score: f32, title: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            score,
            title: title.into(),
            snippet: None,
            line: None,
            match_type: MatchType::Content,
        }
    }

    /// Set the snippet
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    /// Set the line number
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Set the match type
    pub fn with_match_type(mut self, match_type: MatchType) -> Self {
        self.match_type = match_type;
        self
    }
}

/// Type of match
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchType {
    /// Match in title/filename
    Title,
    /// Match in content
    Content,
    /// Match in heading
    Heading,
    /// Match in tag
    Tag,
    /// Match by semantic similarity
    Semantic,
    /// Fuzzy match
    Fuzzy,
}

/// Search options
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Minimum score threshold
    pub min_score: Option<f32>,
    /// Search in titles
    pub search_titles: bool,
    /// Search in content
    pub search_content: bool,
    /// Search in tags
    pub search_tags: bool,
    /// Use fuzzy matching
    pub fuzzy: bool,
    /// Use semantic search
    pub semantic: bool,
    /// Paths to include (empty = all)
    pub include_paths: Vec<PathBuf>,
    /// Paths to exclude
    pub exclude_paths: Vec<PathBuf>,
}

impl SearchOptions {
    /// Create options for a quick search
    pub fn quick() -> Self {
        Self {
            limit: Some(20),
            search_titles: true,
            search_content: true,
            search_tags: false,
            fuzzy: false,
            semantic: false,
            ..Default::default()
        }
    }

    /// Create options for a full search
    pub fn full() -> Self {
        Self {
            limit: Some(100),
            search_titles: true,
            search_content: true,
            search_tags: true,
            fuzzy: true,
            semantic: false,
            ..Default::default()
        }
    }

    /// Create options for semantic search
    pub fn semantic() -> Self {
        Self {
            limit: Some(20),
            search_titles: false,
            search_content: false,
            search_tags: false,
            fuzzy: false,
            semantic: true,
            ..Default::default()
        }
    }
}

/// Combined search engine
pub struct SearchEngine {
    /// Full-text index
    fulltext: Option<FullTextIndex>,
    /// Semantic index
    semantic: Option<SemanticIndex>,
    /// Fuzzy matcher
    fuzzy: FuzzyMatcher,
}

impl SearchEngine {
    /// Create a new search engine
    pub fn new() -> Self {
        Self {
            fulltext: None,
            semantic: None,
            fuzzy: FuzzyMatcher::new(),
        }
    }

    /// Initialize full-text index
    pub fn with_fulltext(mut self, index_path: impl Into<PathBuf>) -> SearchResult<Self> {
        self.fulltext = Some(FullTextIndex::open(index_path)?);
        Ok(self)
    }

    /// Initialize semantic index
    pub fn with_semantic(mut self, store: VectorStore) -> Self {
        self.semantic = Some(SemanticIndex::new(store));
        self
    }

    /// Index a note
    pub fn index_note(&self, note: &Note) -> SearchResult<()> {
        if let Some(ref fulltext) = self.fulltext {
            fulltext.index_note(note)?;
        }

        if let Some(ref semantic) = self.semantic {
            semantic.index_note(note)?;
        }

        self.fuzzy.add_file(&note.id, &note.basename);

        Ok(())
    }

    /// Remove a note from the index
    pub fn remove_note(&self, path: &str) -> SearchResult<()> {
        if let Some(ref fulltext) = self.fulltext {
            fulltext.remove_note(path)?;
        }

        if let Some(ref semantic) = self.semantic {
            semantic.remove_note(path)?;
        }

        self.fuzzy.remove_file(path);

        Ok(())
    }

    /// Search for notes
    pub fn search(&self, query: &str, options: &SearchOptions) -> SearchResult<Vec<SearchHit>> {
        let mut results = Vec::new();

        // Full-text search
        if (options.search_titles || options.search_content || options.search_tags)
            && !options.semantic
        {
            if let Some(ref fulltext) = self.fulltext {
                let searcher = fulltext.searcher()?;
                let hits = searcher.search(query, options)?;
                results.extend(hits);
            }
        }

        // Fuzzy search for file names
        if options.fuzzy && options.search_titles {
            let fuzzy_hits = self.fuzzy.search(query, options.limit.unwrap_or(20));
            results.extend(fuzzy_hits);
        }

        // Semantic search
        if options.semantic {
            if let Some(ref semantic) = self.semantic {
                let hits = semantic.search(query, options.limit.unwrap_or(20))?;
                results.extend(hits);
            }
        }

        // Sort by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Apply limit
        if let Some(limit) = options.limit {
            results.truncate(limit);
        }

        // Apply min score filter
        if let Some(min_score) = options.min_score {
            results.retain(|h| h.score >= min_score);
        }

        Ok(results)
    }

    /// Commit changes to the indices
    pub fn commit(&self) -> SearchResult<()> {
        if let Some(ref fulltext) = self.fulltext {
            fulltext.commit()?;
        }
        Ok(())
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Prelude for common imports
pub mod prelude {
    pub use crate::error::{SearchError, SearchResult};
    pub use crate::fulltext::{FullTextIndex, FullTextSearcher};
    pub use crate::fuzzy::{fuzzy_match, FuzzyMatcher};
    pub use crate::semantic::{SemanticIndex, VectorStore};
    pub use crate::{MatchType, SearchEngine, SearchHit, SearchOptions};
}
