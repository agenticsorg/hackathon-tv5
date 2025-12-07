//! Autocomplete and query suggestions
//!
//! Provides fast prefix-based autocomplete suggestions using a Trie data structure
//! with Redis caching for improved performance.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, instrument};

use crate::cache::RedisCache;

/// Suggestion type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SuggestionType {
    Title,
    Person,
    Genre,
    Platform,
    Keyword,
}

/// A single autocomplete suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub text: String,
    #[serde(rename = "type")]
    pub suggestion_type: SuggestionType,
    pub popularity: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Autocomplete response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteResponse {
    pub query: String,
    pub suggestions: Vec<Suggestion>,
    pub cached: bool,
}

#[derive(Debug, Default)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
    suggestions: Vec<Suggestion>,
    is_end: bool,
}

/// Autocomplete service with Trie-based prefix matching
pub struct AutocompleteService {
    trie: Arc<RwLock<TrieNode>>,
    cache: Option<Arc<RedisCache>>,
    cache_ttl: u64,
}

impl AutocompleteService {
    pub fn new(cache: Option<Arc<RedisCache>>) -> Self {
        Self {
            trie: Arc::new(RwLock::new(TrieNode::default())),
            cache,
            cache_ttl: 3600,
        }
    }

    pub fn with_cache_ttl(cache: Option<Arc<RedisCache>>, ttl_seconds: u64) -> Self {
        Self {
            trie: Arc::new(RwLock::new(TrieNode::default())),
            cache,
            cache_ttl: ttl_seconds,
        }
    }

    pub async fn init_defaults(&self) {
        let genres = [
            ("action", 0.95),
            ("adventure", 0.90),
            ("animation", 0.85),
            ("comedy", 0.95),
            ("crime", 0.80),
            ("documentary", 0.85),
            ("drama", 0.95),
            ("family", 0.80),
            ("fantasy", 0.85),
            ("history", 0.70),
            ("horror", 0.90),
            ("music", 0.75),
            ("mystery", 0.85),
            ("romance", 0.90),
            ("science fiction", 0.90),
            ("thriller", 0.90),
            ("war", 0.70),
            ("western", 0.65),
        ];
        for (genre, popularity) in genres {
            self.add_suggestion(Suggestion {
                text: genre.to_string(),
                suggestion_type: SuggestionType::Genre,
                popularity,
                metadata: None,
            })
            .await;
        }

        let platforms = [
            ("netflix", 0.98),
            ("disney+", 0.95),
            ("hbo max", 0.92),
            ("prime video", 0.93),
            ("hulu", 0.88),
            ("apple tv+", 0.85),
            ("paramount+", 0.80),
            ("peacock", 0.75),
        ];
        for (platform, popularity) in platforms {
            self.add_suggestion(Suggestion {
                text: platform.to_string(),
                suggestion_type: SuggestionType::Platform,
                popularity,
                metadata: None,
            })
            .await;
        }
    }

    pub async fn add_suggestion(&self, suggestion: Suggestion) {
        let mut trie = self.trie.write().await;
        let text_lower = suggestion.text.to_lowercase();
        let mut node = &mut *trie;
        for ch in text_lower.chars() {
            node = node.children.entry(ch).or_default();
        }
        node.is_end = true;

        if let Some(existing) = node
            .suggestions
            .iter_mut()
            .find(|s| s.text.to_lowercase() == text_lower)
        {
            if suggestion.popularity > existing.popularity {
                *existing = suggestion;
            }
        } else {
            node.suggestions.push(suggestion);
            node.suggestions.sort_by(|a, b| {
                b.popularity
                    .partial_cmp(&a.popularity)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }

    pub async fn add_suggestions(&self, suggestions: Vec<Suggestion>) {
        for suggestion in suggestions {
            self.add_suggestion(suggestion).await;
        }
    }

    #[instrument(skip(self), fields(prefix = %prefix, limit = %limit))]
    pub async fn suggest(&self, prefix: &str, limit: usize) -> Result<AutocompleteResponse> {
        let prefix_lower = prefix.to_lowercase().trim().to_string();
        if prefix_lower.is_empty() {
            return Ok(AutocompleteResponse {
                query: prefix.to_string(),
                suggestions: vec![],
                cached: false,
            });
        }

        if let Some(ref cache) = self.cache {
            let cache_key = format!("autocomplete:{}", prefix_lower);
            if let Ok(Some(cached)) = cache.get::<Vec<Suggestion>>(&cache_key).await {
                debug!(prefix = %prefix_lower, "Cache hit for autocomplete");
                return Ok(AutocompleteResponse {
                    query: prefix.to_string(),
                    suggestions: cached.into_iter().take(limit).collect(),
                    cached: true,
                });
            }
        }

        let suggestions = self.search_trie(&prefix_lower, limit).await;

        if let Some(ref cache) = self.cache {
            let cache_key = format!("autocomplete:{}", prefix_lower);
            let _ = cache.set(&cache_key, &suggestions, self.cache_ttl).await;
        }

        Ok(AutocompleteResponse {
            query: prefix.to_string(),
            suggestions,
            cached: false,
        })
    }

    async fn search_trie(&self, prefix: &str, limit: usize) -> Vec<Suggestion> {
        let trie = self.trie.read().await;
        let mut node = &*trie;
        for ch in prefix.chars() {
            match node.children.get(&ch) {
                Some(child) => node = child,
                None => return vec![],
            }
        }

        let mut results = Vec::new();
        self.collect_suggestions(node, &mut results, limit * 2);
        results.sort_by(|a, b| {
            b.popularity
                .partial_cmp(&a.popularity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        results
    }

    fn collect_suggestions(&self, node: &TrieNode, results: &mut Vec<Suggestion>, limit: usize) {
        if results.len() >= limit {
            return;
        }
        for suggestion in &node.suggestions {
            if results.len() >= limit {
                break;
            }
            if !results.iter().any(|s| s.text == suggestion.text) {
                results.push(suggestion.clone());
            }
        }
        for child in node.children.values() {
            if results.len() >= limit {
                break;
            }
            self.collect_suggestions(child, results, limit);
        }
    }

    pub async fn clear(&self) {
        let mut trie = self.trie.write().await;
        *trie = TrieNode::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_suggest() {
        let service = AutocompleteService::new(None);
        service
            .add_suggestion(Suggestion {
                text: "The Dark Knight".to_string(),
                suggestion_type: SuggestionType::Title,
                popularity: 0.95,
                metadata: None,
            })
            .await;
        service
            .add_suggestion(Suggestion {
                text: "Dark Shadows".to_string(),
                suggestion_type: SuggestionType::Title,
                popularity: 0.70,
                metadata: None,
            })
            .await;

        let response = service.suggest("dark", 10).await.unwrap();
        assert_eq!(response.suggestions.len(), 2);
        assert_eq!(response.suggestions[0].text, "The Dark Knight");
    }

    #[tokio::test]
    async fn test_case_insensitive() {
        let service = AutocompleteService::new(None);
        service
            .add_suggestion(Suggestion {
                text: "Netflix".to_string(),
                suggestion_type: SuggestionType::Platform,
                popularity: 0.95,
                metadata: None,
            })
            .await;

        let response = service.suggest("NET", 10).await.unwrap();
        assert_eq!(response.suggestions.len(), 1);
    }

    #[tokio::test]
    async fn test_empty_prefix() {
        let service = AutocompleteService::new(None);
        let response = service.suggest("", 10).await.unwrap();
        assert!(response.suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_performance() {
        let service = AutocompleteService::new(None);
        for i in 0..10000 {
            service
                .add_suggestion(Suggestion {
                    text: format!("movie title number {}", i),
                    suggestion_type: SuggestionType::Title,
                    popularity: (i as f32 / 10000.0),
                    metadata: None,
                })
                .await;
        }

        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = service.suggest("movie", 10).await.unwrap();
        }
        assert!(start.elapsed().as_millis() < 100, "Autocomplete too slow");
    }
}
