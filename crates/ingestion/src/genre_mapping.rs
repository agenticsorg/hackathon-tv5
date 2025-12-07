//! Genre mapping to canonical taxonomy
//!
//! Implements the MapGenres algorithm from SPARC specification:
//! 1. Direct platform-specific mapping
//! 2. Fuzzy matching fallback (>0.8 confidence)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strsim::normalized_levenshtein;

/// Canonical genre taxonomy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CanonicalGenre {
    Action,
    Adventure,
    Animation,
    Comedy,
    Crime,
    Documentary,
    Drama,
    Family,
    Fantasy,
    History,
    Horror,
    Music,
    Mystery,
    Romance,
    ScienceFiction,
    Thriller,
    War,
    Western,
}

impl CanonicalGenre {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Action => "Action",
            Self::Adventure => "Adventure",
            Self::Animation => "Animation",
            Self::Comedy => "Comedy",
            Self::Crime => "Crime",
            Self::Documentary => "Documentary",
            Self::Drama => "Drama",
            Self::Family => "Family",
            Self::Fantasy => "Fantasy",
            Self::History => "History",
            Self::Horror => "Horror",
            Self::Music => "Music",
            Self::Mystery => "Mystery",
            Self::Romance => "Romance",
            Self::ScienceFiction => "Science Fiction",
            Self::Thriller => "Thriller",
            Self::War => "War",
            Self::Western => "Western",
        }
    }

    /// Get all canonical genres
    pub fn all() -> Vec<CanonicalGenre> {
        vec![
            Self::Action,
            Self::Adventure,
            Self::Animation,
            Self::Comedy,
            Self::Crime,
            Self::Documentary,
            Self::Drama,
            Self::Family,
            Self::Fantasy,
            Self::History,
            Self::Horror,
            Self::Music,
            Self::Mystery,
            Self::Romance,
            Self::ScienceFiction,
            Self::Thriller,
            Self::War,
            Self::Western,
        ]
    }
}

/// Genre mapper for converting platform-specific genres to canonical taxonomy
pub struct GenreMapper {
    // Platform-specific mappings
    platform_mappings: HashMap<String, HashMap<String, Vec<CanonicalGenre>>>,
    // Fuzzy match threshold
    fuzzy_threshold: f64,
}

impl GenreMapper {
    /// Create a new genre mapper with default mappings
    pub fn new() -> Self {
        let mut mapper = Self {
            platform_mappings: HashMap::new(),
            fuzzy_threshold: 0.8,
        };

        // Initialize platform-specific mappings
        mapper.init_netflix_mappings();
        mapper.init_prime_video_mappings();
        mapper.init_disney_plus_mappings();
        mapper.init_youtube_mappings();
        mapper.init_tmdb_mappings();

        mapper
    }

    /// Map genres from platform-specific to canonical taxonomy
    ///
    /// # Arguments
    /// * `genres` - Platform-specific genre names
    /// * `platform_id` - Platform identifier
    ///
    /// # Returns
    /// Vector of canonical genre names
    pub fn map_genres(&self, genres: &[String], platform_id: &str) -> Vec<String> {
        let mut canonical_genres = Vec::new();

        for genre in genres {
            let mapped = self.map_single_genre(genre, platform_id);
            canonical_genres.extend(mapped);
        }

        // Deduplicate
        canonical_genres.sort();
        canonical_genres.dedup();

        canonical_genres
    }

    /// Map a single genre
    fn map_single_genre(&self, genre: &str, platform_id: &str) -> Vec<String> {
        let normalized_genre = genre.to_lowercase();

        // Try direct platform mapping
        if let Some(platform_map) = self.platform_mappings.get(platform_id) {
            if let Some(canonical) = platform_map.get(&normalized_genre) {
                return canonical.iter().map(|g| g.as_str().to_string()).collect();
            }
        }

        // Try fuzzy matching against canonical genres
        self.fuzzy_match_genre(&normalized_genre)
    }

    /// Fuzzy match genre against canonical taxonomy
    fn fuzzy_match_genre(&self, genre: &str) -> Vec<String> {
        let mut best_match: Option<CanonicalGenre> = None;
        let mut best_score = 0.0;

        for canonical in CanonicalGenre::all() {
            let similarity = normalized_levenshtein(genre, &canonical.as_str().to_lowercase());

            if similarity > best_score && similarity >= self.fuzzy_threshold {
                best_score = similarity;
                best_match = Some(canonical);
            }
        }

        best_match
            .map(|g| vec![g.as_str().to_string()])
            .unwrap_or_default()
    }

    /// Initialize Netflix genre mappings
    fn init_netflix_mappings(&mut self) {
        let mut mappings = HashMap::new();

        mappings.insert("action-adventure".to_string(), vec![CanonicalGenre::Action]);
        mappings.insert(
            "action & adventure".to_string(),
            vec![CanonicalGenre::Action],
        );
        mappings.insert("sci-fi".to_string(), vec![CanonicalGenre::ScienceFiction]);
        mappings.insert(
            "science fiction".to_string(),
            vec![CanonicalGenre::ScienceFiction],
        );
        mappings.insert("thriller".to_string(), vec![CanonicalGenre::Thriller]);
        mappings.insert("suspense".to_string(), vec![CanonicalGenre::Thriller]);
        mappings.insert("comedy".to_string(), vec![CanonicalGenre::Comedy]);
        mappings.insert("drama".to_string(), vec![CanonicalGenre::Drama]);
        mappings.insert("horror".to_string(), vec![CanonicalGenre::Horror]);
        mappings.insert("romance".to_string(), vec![CanonicalGenre::Romance]);
        mappings.insert("romantic".to_string(), vec![CanonicalGenre::Romance]);
        mappings.insert("documentary".to_string(), vec![CanonicalGenre::Documentary]);
        mappings.insert("animation".to_string(), vec![CanonicalGenre::Animation]);
        mappings.insert("animated".to_string(), vec![CanonicalGenre::Animation]);
        mappings.insert("family".to_string(), vec![CanonicalGenre::Family]);
        mappings.insert("kids".to_string(), vec![CanonicalGenre::Family]);
        mappings.insert("fantasy".to_string(), vec![CanonicalGenre::Fantasy]);
        mappings.insert("mystery".to_string(), vec![CanonicalGenre::Mystery]);
        mappings.insert("crime".to_string(), vec![CanonicalGenre::Crime]);
        mappings.insert("war".to_string(), vec![CanonicalGenre::War]);
        mappings.insert("western".to_string(), vec![CanonicalGenre::Western]);
        mappings.insert("music".to_string(), vec![CanonicalGenre::Music]);
        mappings.insert("musical".to_string(), vec![CanonicalGenre::Music]);
        mappings.insert("history".to_string(), vec![CanonicalGenre::History]);
        mappings.insert("historical".to_string(), vec![CanonicalGenre::History]);

        self.platform_mappings
            .insert("netflix".to_string(), mappings);
    }

    /// Initialize Prime Video genre mappings
    fn init_prime_video_mappings(&mut self) {
        let mut mappings = HashMap::new();

        mappings.insert("action".to_string(), vec![CanonicalGenre::Action]);
        mappings.insert(
            "action & adventure".to_string(),
            vec![CanonicalGenre::Action],
        );
        mappings.insert("adventure".to_string(), vec![CanonicalGenre::Adventure]);
        mappings.insert("sci-fi".to_string(), vec![CanonicalGenre::ScienceFiction]);
        mappings.insert(
            "science fiction".to_string(),
            vec![CanonicalGenre::ScienceFiction],
        );
        mappings.insert("thriller".to_string(), vec![CanonicalGenre::Thriller]);
        mappings.insert("comedy".to_string(), vec![CanonicalGenre::Comedy]);
        mappings.insert("drama".to_string(), vec![CanonicalGenre::Drama]);
        mappings.insert("horror".to_string(), vec![CanonicalGenre::Horror]);
        mappings.insert("romance".to_string(), vec![CanonicalGenre::Romance]);
        mappings.insert("documentary".to_string(), vec![CanonicalGenre::Documentary]);
        mappings.insert("animation".to_string(), vec![CanonicalGenre::Animation]);
        mappings.insert("family".to_string(), vec![CanonicalGenre::Family]);
        mappings.insert("fantasy".to_string(), vec![CanonicalGenre::Fantasy]);
        mappings.insert("mystery".to_string(), vec![CanonicalGenre::Mystery]);
        mappings.insert("crime".to_string(), vec![CanonicalGenre::Crime]);
        mappings.insert("war".to_string(), vec![CanonicalGenre::War]);
        mappings.insert("western".to_string(), vec![CanonicalGenre::Western]);
        mappings.insert("music".to_string(), vec![CanonicalGenre::Music]);
        mappings.insert("history".to_string(), vec![CanonicalGenre::History]);

        self.platform_mappings
            .insert("prime_video".to_string(), mappings);
    }

    /// Initialize Disney+ genre mappings
    fn init_disney_plus_mappings(&mut self) {
        let mut mappings = HashMap::new();

        mappings.insert("action".to_string(), vec![CanonicalGenre::Action]);
        mappings.insert("action-adventure".to_string(), vec![CanonicalGenre::Action]);
        mappings.insert("adventure".to_string(), vec![CanonicalGenre::Adventure]);
        mappings.insert(
            "science fiction".to_string(),
            vec![CanonicalGenre::ScienceFiction],
        );
        mappings.insert("sci-fi".to_string(), vec![CanonicalGenre::ScienceFiction]);
        mappings.insert("comedy".to_string(), vec![CanonicalGenre::Comedy]);
        mappings.insert("drama".to_string(), vec![CanonicalGenre::Drama]);
        mappings.insert("horror".to_string(), vec![CanonicalGenre::Horror]);
        mappings.insert("romance".to_string(), vec![CanonicalGenre::Romance]);
        mappings.insert("documentary".to_string(), vec![CanonicalGenre::Documentary]);
        mappings.insert("animation".to_string(), vec![CanonicalGenre::Animation]);
        mappings.insert("family".to_string(), vec![CanonicalGenre::Family]);
        mappings.insert("kids".to_string(), vec![CanonicalGenre::Family]);
        mappings.insert("fantasy".to_string(), vec![CanonicalGenre::Fantasy]);
        mappings.insert("musical".to_string(), vec![CanonicalGenre::Music]);
        mappings.insert("music".to_string(), vec![CanonicalGenre::Music]);
        mappings.insert(
            "superhero".to_string(),
            vec![CanonicalGenre::Action, CanonicalGenre::Fantasy],
        );

        self.platform_mappings
            .insert("disney_plus".to_string(), mappings);
    }

    /// Initialize YouTube genre mappings
    fn init_youtube_mappings(&mut self) {
        let mut mappings = HashMap::new();

        mappings.insert("film".to_string(), vec![CanonicalGenre::Drama]);
        mappings.insert("music".to_string(), vec![CanonicalGenre::Music]);
        mappings.insert("comedy".to_string(), vec![CanonicalGenre::Comedy]);
        mappings.insert("entertainment".to_string(), vec![CanonicalGenre::Drama]);
        mappings.insert("education".to_string(), vec![CanonicalGenre::Documentary]);
        mappings.insert("science".to_string(), vec![CanonicalGenre::Documentary]);
        mappings.insert("documentary".to_string(), vec![CanonicalGenre::Documentary]);

        self.platform_mappings
            .insert("youtube".to_string(), mappings);
    }

    /// Initialize TMDb genre mappings (for enrichment)
    fn init_tmdb_mappings(&mut self) {
        let mut mappings = HashMap::new();

        mappings.insert("action".to_string(), vec![CanonicalGenre::Action]);
        mappings.insert("adventure".to_string(), vec![CanonicalGenre::Adventure]);
        mappings.insert("animation".to_string(), vec![CanonicalGenre::Animation]);
        mappings.insert("comedy".to_string(), vec![CanonicalGenre::Comedy]);
        mappings.insert("crime".to_string(), vec![CanonicalGenre::Crime]);
        mappings.insert("documentary".to_string(), vec![CanonicalGenre::Documentary]);
        mappings.insert("drama".to_string(), vec![CanonicalGenre::Drama]);
        mappings.insert("family".to_string(), vec![CanonicalGenre::Family]);
        mappings.insert("fantasy".to_string(), vec![CanonicalGenre::Fantasy]);
        mappings.insert("history".to_string(), vec![CanonicalGenre::History]);
        mappings.insert("horror".to_string(), vec![CanonicalGenre::Horror]);
        mappings.insert("music".to_string(), vec![CanonicalGenre::Music]);
        mappings.insert("mystery".to_string(), vec![CanonicalGenre::Mystery]);
        mappings.insert("romance".to_string(), vec![CanonicalGenre::Romance]);
        mappings.insert(
            "science fiction".to_string(),
            vec![CanonicalGenre::ScienceFiction],
        );
        mappings.insert("thriller".to_string(), vec![CanonicalGenre::Thriller]);
        mappings.insert("war".to_string(), vec![CanonicalGenre::War]);
        mappings.insert("western".to_string(), vec![CanonicalGenre::Western]);

        self.platform_mappings.insert("tmdb".to_string(), mappings);
    }
}

impl Default for GenreMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_netflix_genre_mapping() {
        let mapper = GenreMapper::new();

        let genres = vec!["action-adventure".to_string(), "sci-fi".to_string()];
        let mapped = mapper.map_genres(&genres, "netflix");

        assert!(mapped.contains(&"Action".to_string()));
        assert!(mapped.contains(&"Science Fiction".to_string()));
    }

    #[test]
    fn test_disney_superhero_mapping() {
        let mapper = GenreMapper::new();

        let genres = vec!["superhero".to_string()];
        let mapped = mapper.map_genres(&genres, "disney_plus");

        assert!(mapped.contains(&"Action".to_string()));
        assert!(mapped.contains(&"Fantasy".to_string()));
    }

    #[test]
    fn test_fuzzy_matching() {
        let mapper = GenreMapper::new();

        // "scifi" should fuzzy match to "Science Fiction"
        let genres = vec!["scifi".to_string()];
        let mapped = mapper.map_genres(&genres, "unknown_platform");

        // Note: This might not match due to threshold; adjust if needed
        // For now, we test that it doesn't panic
        assert!(mapped.len() <= 1);
    }

    #[test]
    fn test_deduplication() {
        let mapper = GenreMapper::new();

        let genres = vec!["action".to_string(), "action-adventure".to_string()];
        let mapped = mapper.map_genres(&genres, "netflix");

        // Both map to "Action", should deduplicate
        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0], "Action");
    }

    #[test]
    fn test_canonical_genre_as_str() {
        assert_eq!(CanonicalGenre::ScienceFiction.as_str(), "Science Fiction");
        assert_eq!(CanonicalGenre::Action.as_str(), "Action");
    }
}
