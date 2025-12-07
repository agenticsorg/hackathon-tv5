//! Fuzzy string matching for file names

use crate::{MatchType, SearchHit};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;

/// Fuzzy match result
#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    /// Matched string
    pub text: String,
    /// Match score (0.0 - 1.0)
    pub score: f32,
    /// Matched positions
    pub positions: Vec<usize>,
}

/// Perform fuzzy matching between pattern and text
pub fn fuzzy_match(pattern: &str, text: &str) -> Option<FuzzyMatch> {
    let pattern_lower = pattern.to_lowercase();
    let text_lower = text.to_lowercase();

    let pattern_chars: Vec<char> = pattern_lower.chars().collect();
    let text_chars: Vec<char> = text_lower.chars().collect();

    if pattern_chars.is_empty() {
        return Some(FuzzyMatch {
            text: text.to_string(),
            score: 1.0,
            positions: vec![],
        });
    }

    let mut pattern_idx = 0;
    let mut positions = Vec::new();
    let mut score = 0.0;
    let mut consecutive_bonus = 0.0;
    let mut prev_matched = false;

    for (text_idx, text_char) in text_chars.iter().enumerate() {
        if pattern_idx < pattern_chars.len() && *text_char == pattern_chars[pattern_idx] {
            positions.push(text_idx);

            // Base score for match
            let mut char_score = 1.0;

            // Bonus for consecutive matches
            if prev_matched {
                consecutive_bonus += 0.5;
                char_score += consecutive_bonus;
            } else {
                consecutive_bonus = 0.0;
            }

            // Bonus for match at start
            if text_idx == 0 {
                char_score += 2.0;
            }

            // Bonus for match after separator
            if text_idx > 0 {
                let prev_char = text_chars[text_idx - 1];
                if prev_char == ' ' || prev_char == '-' || prev_char == '_' || prev_char == '/' {
                    char_score += 1.5;
                }
            }

            // Bonus for case match
            if pattern.chars().nth(pattern_idx) == text.chars().nth(text_idx) {
                char_score += 0.5;
            }

            score += char_score;
            pattern_idx += 1;
            prev_matched = true;
        } else {
            prev_matched = false;
            consecutive_bonus = 0.0;
        }
    }

    // All pattern chars must be matched
    if pattern_idx < pattern_chars.len() {
        return None;
    }

    // Normalize score
    let max_possible_score = pattern_chars.len() as f32 * 5.0;
    let normalized_score = (score / max_possible_score).min(1.0);

    // Penalty for length difference
    let length_penalty = 1.0 - (text_chars.len() as f32 - pattern_chars.len() as f32).abs()
        / (text_chars.len() as f32).max(1.0)
        * 0.3;

    let final_score = normalized_score * length_penalty;

    Some(FuzzyMatch {
        text: text.to_string(),
        score: final_score,
        positions,
    })
}

/// Fuzzy matcher for file names
pub struct FuzzyMatcher {
    /// File name -> path mapping
    files: RwLock<HashMap<String, String>>,
}

impl FuzzyMatcher {
    /// Create a new fuzzy matcher
    pub fn new() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
        }
    }

    /// Add a file to the matcher
    pub fn add_file(&self, path: &str, name: &str) {
        let mut files = self.files.write();
        files.insert(name.to_string(), path.to_string());
    }

    /// Remove a file from the matcher
    pub fn remove_file(&self, path: &str) {
        let mut files = self.files.write();
        files.retain(|_, p| p != path);
    }

    /// Clear all files
    pub fn clear(&self) {
        let mut files = self.files.write();
        files.clear();
    }

    /// Get number of files
    pub fn len(&self) -> usize {
        self.files.read().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.files.read().is_empty()
    }

    /// Search for files matching the pattern
    pub fn search(&self, pattern: &str, limit: usize) -> Vec<SearchHit> {
        let files = self.files.read();

        let mut matches: Vec<(String, String, f32)> = files
            .iter()
            .filter_map(|(name, path)| {
                fuzzy_match(pattern, name).map(|m| (name.clone(), path.clone(), m.score))
            })
            .collect();

        // Sort by score descending
        matches.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Take top results
        matches.truncate(limit);

        // Convert to SearchHits
        matches
            .into_iter()
            .map(|(name, path, score)| {
                SearchHit::new(PathBuf::from(&path), score, name).with_match_type(MatchType::Fuzzy)
            })
            .collect()
    }

    /// Find exact matches
    pub fn find_exact(&self, name: &str) -> Option<String> {
        let files = self.files.read();
        files.get(name).cloned()
    }
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match_exact() {
        let result = fuzzy_match("hello", "hello").unwrap();
        assert!(result.score > 0.9);
        assert_eq!(result.positions.len(), 5);
    }

    #[test]
    fn test_fuzzy_match_prefix() {
        let result = fuzzy_match("hel", "hello").unwrap();
        assert!(result.score > 0.5);
        assert_eq!(result.positions, vec![0, 1, 2]);
    }

    #[test]
    fn test_fuzzy_match_subsequence() {
        let result = fuzzy_match("hw", "hello world").unwrap();
        assert!(result.score > 0.0);
        assert_eq!(result.positions.len(), 2);
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        let result = fuzzy_match("HELLO", "hello").unwrap();
        assert!(result.score > 0.5);
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        let result = fuzzy_match("xyz", "hello");
        assert!(result.is_none());
    }

    #[test]
    fn test_fuzzy_matcher_search() {
        let matcher = FuzzyMatcher::new();

        matcher.add_file("notes/hello.md", "hello");
        matcher.add_file("notes/world.md", "world");
        matcher.add_file("notes/hello-world.md", "hello-world");

        let results = matcher.search("hel", 10);
        assert_eq!(results.len(), 2); // hello and hello-world

        let results = matcher.search("world", 10);
        assert_eq!(results.len(), 2); // world and hello-world
    }

    #[test]
    fn test_fuzzy_matcher_remove() {
        let matcher = FuzzyMatcher::new();

        matcher.add_file("notes/test.md", "test");
        assert_eq!(matcher.len(), 1);

        matcher.remove_file("notes/test.md");
        assert_eq!(matcher.len(), 0);
    }

    #[test]
    fn test_word_boundary_bonus() {
        // Match at word boundary should score higher
        let result1 = fuzzy_match("w", "hello-world").unwrap();
        let result2 = fuzzy_match("e", "hello-world").unwrap();

        // 'w' matches at word boundary (after '-'), 'e' matches in the middle
        assert!(result1.score > result2.score);
    }
}
