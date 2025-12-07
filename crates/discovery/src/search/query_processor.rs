//! Query spell correction and expansion
//!
//! Provides query preprocessing with spell correction using Levenshtein distance,
//! synonym expansion, and query rewriting for improved search results.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, instrument};

/// Maximum edit distance for spell correction
const MAX_EDIT_DISTANCE: usize = 2;

/// Query processor for spell correction and expansion
pub struct QueryProcessor {
    /// Dictionary of known terms (titles, genres, etc.)
    dictionary: HashSet<String>,
    /// Synonym mappings
    synonyms: HashMap<String, Vec<String>>,
    /// Common typo corrections
    typo_corrections: HashMap<String, String>,
}

/// Processed query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedQuery {
    /// Original query
    pub original: String,
    /// Corrected query (if corrections were made)
    pub corrected: String,
    /// Whether corrections were applied
    pub was_corrected: bool,
    /// Expanded terms (synonyms)
    pub expanded_terms: Vec<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

impl QueryProcessor {
    /// Create a new query processor with default dictionaries
    pub fn new() -> Self {
        let mut processor = Self {
            dictionary: HashSet::new(),
            synonyms: HashMap::new(),
            typo_corrections: HashMap::new(),
        };
        processor.init_default_dictionaries();
        processor
    }

    /// Create with custom dictionary
    pub fn with_dictionary(dictionary: HashSet<String>) -> Self {
        let mut processor = Self {
            dictionary,
            synonyms: HashMap::new(),
            typo_corrections: HashMap::new(),
        };
        processor.init_synonyms();
        processor.init_typo_corrections();
        processor
    }

    /// Initialize default dictionaries
    fn init_default_dictionaries(&mut self) {
        let genres = [
            "action",
            "adventure",
            "animation",
            "comedy",
            "crime",
            "documentary",
            "drama",
            "family",
            "fantasy",
            "history",
            "horror",
            "music",
            "musical",
            "mystery",
            "romance",
            "science fiction",
            "sci-fi",
            "thriller",
            "war",
            "western",
            "biography",
            "sport",
            "superhero",
        ];
        for genre in genres {
            self.dictionary.insert(genre.to_string());
        }

        let keywords = [
            "movie",
            "movies",
            "film",
            "films",
            "series",
            "show",
            "shows",
            "tv",
            "television",
            "documentary",
            "documentaries",
            "anime",
            "cartoon",
            "cartoons",
            "miniseries",
            "limited series",
        ];
        for keyword in keywords {
            self.dictionary.insert(keyword.to_string());
        }

        let descriptors = [
            "best", "top", "new", "latest", "classic", "popular", "trending", "award", "winning",
            "oscar", "emmy", "good", "great", "funny", "scary", "romantic", "exciting", "intense",
            "dark", "light",
        ];
        for desc in descriptors {
            self.dictionary.insert(desc.to_string());
        }

        self.init_synonyms();
        self.init_typo_corrections();
    }

    fn init_synonyms(&mut self) {
        self.synonyms.insert(
            "sci-fi".to_string(),
            vec![
                "science fiction".to_string(),
                "scifi".to_string(),
                "sf".to_string(),
            ],
        );
        self.synonyms.insert(
            "science fiction".to_string(),
            vec!["sci-fi".to_string(), "scifi".to_string()],
        );
        self.synonyms.insert(
            "romcom".to_string(),
            vec!["romantic comedy".to_string(), "romance comedy".to_string()],
        );
        self.synonyms
            .insert("action".to_string(), vec!["action-adventure".to_string()]);
        self.synonyms.insert(
            "scary".to_string(),
            vec!["horror".to_string(), "thriller".to_string()],
        );
        self.synonyms.insert(
            "funny".to_string(),
            vec!["comedy".to_string(), "comedies".to_string()],
        );
        self.synonyms.insert(
            "anime".to_string(),
            vec![
                "animation".to_string(),
                "animated".to_string(),
                "japanese animation".to_string(),
            ],
        );
    }

    fn init_typo_corrections(&mut self) {
        let corrections = [
            ("moive", "movie"),
            ("moives", "movies"),
            ("movei", "movie"),
            ("movis", "movies"),
            ("flim", "film"),
            ("flims", "films"),
            ("scifi", "sci-fi"),
            ("comdy", "comedy"),
            ("commedy", "comedy"),
            ("horrer", "horror"),
            ("horro", "horror"),
            ("thriler", "thriller"),
            ("thiller", "thriller"),
            ("acton", "action"),
            ("acion", "action"),
            ("documentry", "documentary"),
            ("documentery", "documentary"),
            ("animaton", "animation"),
            ("romace", "romance"),
            ("roamnce", "romance"),
            ("mystrey", "mystery"),
            ("myster", "mystery"),
            ("fantsy", "fantasy"),
            ("fantacy", "fantasy"),
            ("adveture", "adventure"),
            ("adventrue", "adventure"),
        ];
        for (typo, correct) in corrections {
            self.typo_corrections
                .insert(typo.to_string(), correct.to_string());
        }
    }

    pub fn add_to_dictionary(&mut self, terms: impl IntoIterator<Item = String>) {
        for term in terms {
            self.dictionary.insert(term.to_lowercase());
        }
    }

    #[instrument(skip(self), fields(query = %query))]
    pub fn process(&self, query: &str) -> ProcessedQuery {
        let original = query.to_string();
        let query_lower = query.to_lowercase();
        let words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut corrected_words = Vec::new();
        let mut was_corrected = false;
        let mut total_confidence = 0.0;

        for word in &words {
            let (corrected, confidence) = self.correct_word(word);
            if corrected != *word {
                was_corrected = true;
            }
            corrected_words.push(corrected);
            total_confidence += confidence;
        }

        let corrected = corrected_words.join(" ");
        let confidence = if words.is_empty() {
            1.0
        } else {
            total_confidence / words.len() as f32
        };
        let expanded_terms = self.expand_synonyms(&corrected);

        debug!(original = %original, corrected = %corrected, was_corrected = %was_corrected, "Query processed");

        ProcessedQuery {
            original,
            corrected,
            was_corrected,
            expanded_terms,
            confidence,
        }
    }

    fn correct_word(&self, word: &str) -> (String, f32) {
        if let Some(correction) = self.typo_corrections.get(word) {
            return (correction.clone(), 0.95);
        }
        if self.dictionary.contains(word) {
            return (word.to_string(), 1.0);
        }

        let mut best_match = word.to_string();
        let mut best_distance = MAX_EDIT_DISTANCE + 1;
        let mut best_confidence = 0.5;

        for dict_word in &self.dictionary {
            let distance = levenshtein_distance(word, dict_word);
            if distance < best_distance && distance <= MAX_EDIT_DISTANCE {
                best_distance = distance;
                best_match = dict_word.clone();
                best_confidence = 1.0 - (distance as f32 * 0.15);
            }
        }
        (best_match, best_confidence)
    }

    fn expand_synonyms(&self, query: &str) -> Vec<String> {
        let mut expanded = Vec::new();
        let query_lower = query.to_lowercase();
        for (term, synonyms) in &self.synonyms {
            if query_lower.contains(term) {
                for synonym in synonyms {
                    let expanded_query = query_lower.replace(term, synonym);
                    if !expanded.contains(&expanded_query) && expanded_query != query_lower {
                        expanded.push(expanded_query);
                    }
                }
            }
        }
        expanded
    }
}

impl Default for QueryProcessor {
    fn default() -> Self {
        Self::new()
    }
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let (len1, len2) = (s1_chars.len(), s2_chars.len());
    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix: Vec<Vec<usize>> = vec![vec![0; len2 + 1]; len1 + 1];
    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }
    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", "abd"), 1);
        assert_eq!(levenshtein_distance("movie", "moive"), 2);
    }

    #[test]
    fn test_typo_correction() {
        let processor = QueryProcessor::new();
        let result = processor.process("moive");
        assert!(result.was_corrected);
        assert_eq!(result.corrected, "movie");

        let result = processor.process("acton thriler");
        assert!(result.was_corrected);
        assert_eq!(result.corrected, "action thriller");
    }

    #[test]
    fn test_synonym_expansion() {
        let processor = QueryProcessor::new();
        let result = processor.process("sci-fi movies");
        assert!(!result.expanded_terms.is_empty());
        assert!(result
            .expanded_terms
            .iter()
            .any(|t| t.contains("science fiction")));
    }

    #[test]
    fn test_performance() {
        let processor = QueryProcessor::new();
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = processor.process("sci-fi action movis 2023");
        }
        assert!(start.elapsed().as_millis() < 50, "Processing too slow");
    }
}
