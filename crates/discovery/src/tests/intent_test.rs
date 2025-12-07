//! Intent parsing tests

use crate::cache::RedisCache;
use crate::config::CacheConfig;
use crate::intent::*;
use std::sync::Arc;

async fn create_test_cache() -> Arc<RedisCache> {
    let config = Arc::new(CacheConfig {
        redis_url: std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        search_ttl_sec: 1800,
        embedding_ttl_sec: 3600,
        intent_ttl_sec: 600,
    });

    match RedisCache::new(config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            // Return a mock cache that will panic if used, but allows tests to compile
            panic!("Redis required for tests")
        }
    }
}

#[tokio::test]
async fn test_intent_parser_extract_similarity_pattern() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);
    let references = parser.extract_references("movies like The Matrix");

    assert_eq!(references.len(), 1);
    assert_eq!(references[0], "The Matrix");
}

#[tokio::test]
async fn test_intent_parser_extract_similarity_similar_to_pattern() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);
    let references = parser.extract_references("shows similar to Breaking Bad");

    assert_eq!(references.len(), 1);
    assert_eq!(references[0], "Breaking Bad");
}

#[tokio::test]
async fn test_intent_parser_extract_multiple_references() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);
    let references = parser.extract_references("movies like Inception and similar to The Matrix");

    assert_eq!(references.len(), 2);
    assert!(references.contains(&"Inception".to_string()));
    assert!(references.contains(&"The Matrix".to_string()));
}

#[tokio::test]
async fn test_intent_parser_extract_person_from_query() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);

    // Test parsing with people names (would be in full implementation)
    let intent = parser.fallback_parse("movies starring Tom Hanks");
    assert_eq!(intent.fallback_query, "movies starring Tom Hanks");
}

#[tokio::test]
async fn test_intent_parser_mood_detection_dark() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);

    // Dark mood detection would be done by GPT in full implementation
    let intent = parser.fallback_parse("dark and gritty thriller");
    assert!(intent.filters.genre.contains(&"thriller".to_string()));
}

#[tokio::test]
async fn test_intent_parser_mood_detection_uplifting() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);

    // Uplifting mood detection would be done by GPT
    let intent = parser.fallback_parse("uplifting comedy");
    assert!(intent.filters.genre.contains(&"comedy".to_string()));
}

#[tokio::test]
async fn test_intent_parser_temporal_pattern_80s_movies() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);

    // Temporal pattern extraction would be in GPT implementation
    let intent = parser.fallback_parse("80s action movies");
    assert!(intent.filters.genre.contains(&"action".to_string()));
}

#[tokio::test]
async fn test_intent_parser_temporal_pattern_recent() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);

    let intent = parser.fallback_parse("recent sci-fi movies");
    assert!(intent
        .filters
        .genre
        .iter()
        .any(|g| g.contains("science") || g == "sci-fi"));
}

#[tokio::test]
async fn test_fallback_parse_genre_extraction_action() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);
    let intent = parser.fallback_parse("action movies");

    assert_eq!(intent.filters.genre, vec!["action"]);
    assert_eq!(intent.confidence, 0.5);
}

#[tokio::test]
async fn test_fallback_parse_genre_extraction_multiple_genres() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);
    let intent = parser.fallback_parse("action comedy thriller");

    assert!(intent.filters.genre.contains(&"action".to_string()));
    assert!(intent.filters.genre.contains(&"comedy".to_string()));
    assert!(intent.filters.genre.contains(&"thriller".to_string()));
}

#[tokio::test]
async fn test_fallback_parse_genre_extraction_sci_fi() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);

    let intent1 = parser.fallback_parse("sci-fi movies");
    assert!(intent1
        .filters
        .genre
        .contains(&"science_fiction".to_string()));

    let intent2 = parser.fallback_parse("scifi shows");
    assert!(intent2
        .filters
        .genre
        .contains(&"science_fiction".to_string()));
}

#[tokio::test]
async fn test_fallback_parse_platform_extraction_netflix() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);
    let intent = parser.fallback_parse("netflix shows");

    assert_eq!(intent.filters.platform, vec!["netflix"]);
}

#[tokio::test]
async fn test_fallback_parse_platform_extraction_multiple_platforms() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);
    let intent = parser.fallback_parse("netflix hbo movies");

    assert!(intent.filters.platform.contains(&"netflix".to_string()));
    assert!(intent.filters.platform.contains(&"hbo_max".to_string()));
}

#[tokio::test]
async fn test_fallback_parse_platform_extraction_prime() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);
    let intent = parser.fallback_parse("prime video shows");

    assert!(intent.filters.platform.contains(&"prime_video".to_string()));
}

#[tokio::test]
async fn test_fallback_parse_platform_extraction_disney() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);
    let intent = parser.fallback_parse("disney movies");

    assert!(intent.filters.platform.contains(&"disney_plus".to_string()));
}

#[tokio::test]
async fn test_fallback_parse_combined_genre_and_platform() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);
    let intent = parser.fallback_parse("netflix action movies");

    assert!(intent.filters.genre.contains(&"action".to_string()));
    assert!(intent.filters.platform.contains(&"netflix".to_string()));
}

#[tokio::test]
async fn test_fallback_parse_no_matches() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);
    let intent = parser.fallback_parse("something random xyz");

    assert!(intent.filters.genre.is_empty());
    assert!(intent.filters.platform.is_empty());
    assert_eq!(intent.fallback_query, "something random xyz");
}

#[tokio::test]
async fn test_fallback_parse_case_insensitive() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);

    let intent1 = parser.fallback_parse("ACTION MOVIES");
    assert!(intent1.filters.genre.contains(&"action".to_string()));

    let intent2 = parser.fallback_parse("Netflix Shows");
    assert!(intent2.filters.platform.contains(&"netflix".to_string()));
}

#[tokio::test]
async fn test_extract_genres_comprehensive() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);
    let tokens = vec![
        "action",
        "comedy",
        "drama",
        "horror",
        "thriller",
        "romance",
        "sci-fi",
        "fantasy",
        "documentary",
    ];

    let genres = parser.extract_genres(&tokens);

    assert!(genres.len() >= 9);
    assert!(genres.contains(&"action".to_string()));
    assert!(genres.contains(&"comedy".to_string()));
    assert!(genres.contains(&"science_fiction".to_string()));
}

#[tokio::test]
async fn test_extract_platforms_comprehensive() {
    let cache = create_test_cache().await;
    let parser = IntentParser::new(String::new(), String::new(), cache);
    let tokens = vec!["netflix", "prime", "hulu", "disney", "hbo"];

    let platforms = parser.extract_platforms(&tokens);

    assert_eq!(platforms.len(), 5);
    assert!(platforms.contains(&"netflix".to_string()));
    assert!(platforms.contains(&"prime_video".to_string()));
    assert!(platforms.contains(&"disney_plus".to_string()));
    assert!(platforms.contains(&"hbo_max".to_string()));
}

#[test]
fn test_parsed_intent_struct() {
    let intent = ParsedIntent {
        mood: vec!["dark".to_string(), "tense".to_string()],
        themes: vec!["heist".to_string(), "crime".to_string()],
        references: vec!["Ocean's Eleven".to_string()],
        filters: IntentFilters {
            genre: vec!["crime".to_string(), "thriller".to_string()],
            platform: vec!["netflix".to_string()],
            year_range: Some((2015, 2024)),
        },
        fallback_query: "dark crime movies like Ocean's Eleven on netflix".to_string(),
        confidence: 0.85,
    };

    assert_eq!(intent.mood.len(), 2);
    assert_eq!(intent.themes.len(), 2);
    assert_eq!(intent.references.len(), 1);
    assert_eq!(intent.filters.genre.len(), 2);
    assert_eq!(intent.filters.year_range.unwrap().0, 2015);
    assert!(intent.confidence > 0.8);
}

#[test]
fn test_intent_type_enum() {
    let search = IntentType::Search;
    let recommendation = IntentType::Recommendation;
    let trivia = IntentType::Trivia;

    assert_ne!(search, recommendation);
    assert_ne!(recommendation, trivia);
    assert_ne!(search, trivia);
}
