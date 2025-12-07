use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::cache::RedisCache;

/// Natural Language Intent Parser
/// Extracts search intent from user queries
pub struct IntentParser {
    /// GPT-4o-mini API client
    client: reqwest::Client,

    /// API configuration
    api_url: String,
    api_key: String,

    /// Redis cache for intent results
    cache: Arc<RedisCache>,
}

/// Parsed search intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedIntent {
    /// Mood/emotional tone keywords
    pub mood: Vec<String>,

    /// Theme keywords
    pub themes: Vec<String>,

    /// Referenced titles (for "like X" queries)
    pub references: Vec<String>,

    /// Extracted filters
    pub filters: IntentFilters,

    /// Fallback query string
    pub fallback_query: String,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,
}

/// Intent filters extracted from query
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntentFilters {
    pub genre: Vec<String>,
    pub platform: Vec<String>,
    pub year_range: Option<(i32, i32)>,
}

/// Intent type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentType {
    /// Direct search query
    Search,

    /// Recommendation request
    Recommendation,

    /// Trivia/information query
    Trivia,
}

impl IntentParser {
    /// Create new intent parser
    pub fn new(api_url: String, api_key: String, cache: Arc<RedisCache>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_url,
            api_key,
            cache,
        }
    }

    /// Parse natural language query into structured intent
    pub async fn parse(&self, query: &str) -> anyhow::Result<ParsedIntent> {
        // Normalize query for consistent cache keys
        let normalized_query = query.trim().to_lowercase();

        // Check cache first
        match self
            .cache
            .get_intent::<String, ParsedIntent>(&normalized_query)
            .await
        {
            Ok(Some(cached_intent)) => {
                tracing::debug!(query = %query, "Intent cache hit");
                return Ok(cached_intent);
            }
            Ok(None) => {
                tracing::debug!(query = %query, "Intent cache miss");
            }
            Err(e) => {
                tracing::warn!(error = %e, "Cache lookup failed, continuing with GPT parsing");
            }
        }

        // Try GPT parsing
        match self.parse_with_gpt(query).await {
            Ok(intent) => {
                // Cache successful parse
                if let Err(e) = self.cache.cache_intent(&normalized_query, &intent).await {
                    tracing::warn!(error = %e, "Failed to cache intent");
                }
                Ok(intent)
            }
            Err(e) => {
                tracing::warn!("GPT parsing failed, using fallback: {}", e);
                let fallback_intent = self.fallback_parse(query);

                // Cache fallback result with lower confidence
                if let Err(cache_err) = self
                    .cache
                    .cache_intent(&normalized_query, &fallback_intent)
                    .await
                {
                    tracing::warn!(error = %cache_err, "Failed to cache fallback intent");
                }

                Ok(fallback_intent)
            }
        }
    }

    /// Parse using GPT-4o-mini
    async fn parse_with_gpt(&self, query: &str) -> anyhow::Result<ParsedIntent> {
        let prompt = self.build_prompt(query);

        let request = serde_json::json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "system",
                    "content": INTENT_PARSER_SYSTEM_PROMPT
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "response_format": { "type": "json_object" }
        });

        let response = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        let response_body: serde_json::Value = response.json().await?;

        // Extract content from GPT response
        let content = response_body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid GPT response"))?;

        let intent: ParsedIntent = serde_json::from_str(content)?;

        // Validate
        if intent.confidence < 0.0 || intent.confidence > 1.0 {
            anyhow::bail!("Invalid confidence score");
        }

        Ok(intent)
    }

    /// Fallback parsing using simple pattern matching
    fn fallback_parse(&self, query: &str) -> ParsedIntent {
        let query_lower = query.to_lowercase();
        let tokens: Vec<&str> = query_lower.split_whitespace().collect();

        // Extract genres
        let genres = self.extract_genres(&tokens);

        // Extract platforms
        let platforms = self.extract_platforms(&tokens);

        // Extract references (simple "like X" pattern)
        let references = self.extract_references(query);

        ParsedIntent {
            mood: Vec::new(),
            themes: Vec::new(),
            references,
            filters: IntentFilters {
                genre: genres,
                platform: platforms,
                year_range: None,
            },
            fallback_query: query.to_string(),
            confidence: 0.5,
        }
    }

    /// Build GPT prompt
    fn build_prompt(&self, query: &str) -> String {
        format!(
            r#"Analyze this media search query and extract structured information:

Query: "{}"

Extract:
1. Mood/Vibes: emotional tone (e.g., "dark", "uplifting", "tense")
2. Themes: main subjects (e.g., "heist", "romance", "sci-fi")
3. References: "similar to X" or "like Y" mentions
4. Filters: platform, genre, year constraints
5. Confidence: 0.0-1.0 score for extraction quality

Return JSON:
{{
  "mood": ["mood1", "mood2"],
  "themes": ["theme1", "theme2"],
  "references": ["title1", "title2"],
  "filters": {{
    "genre": ["genre1"],
    "platform": ["platform1"],
    "year_range": {{"min": 2020, "max": 2024}}
  }},
  "fallback_query": "simplified query string",
  "confidence": 0.85
}}"#,
            query
        )
    }

    /// Extract genres from tokens
    fn extract_genres(&self, tokens: &[&str]) -> Vec<String> {
        let genre_keywords: HashMap<&str, &str> = [
            ("action", "action"),
            ("comedy", "comedy"),
            ("drama", "drama"),
            ("horror", "horror"),
            ("thriller", "thriller"),
            ("romance", "romance"),
            ("sci-fi", "science_fiction"),
            ("scifi", "science_fiction"),
            ("fantasy", "fantasy"),
            ("documentary", "documentary"),
        ]
        .iter()
        .cloned()
        .collect();

        tokens
            .iter()
            .filter_map(|&token| genre_keywords.get(token).map(|&g| g.to_string()))
            .collect()
    }

    /// Extract platforms from tokens
    fn extract_platforms(&self, tokens: &[&str]) -> Vec<String> {
        let platform_keywords: HashMap<&str, &str> = [
            ("netflix", "netflix"),
            ("prime", "prime_video"),
            ("hulu", "hulu"),
            ("disney", "disney_plus"),
            ("hbo", "hbo_max"),
        ]
        .iter()
        .cloned()
        .collect();

        tokens
            .iter()
            .filter_map(|&token| platform_keywords.get(token).map(|&p| p.to_string()))
            .collect()
    }

    /// Extract title references
    fn extract_references(&self, query: &str) -> Vec<String> {
        let mut references = Vec::new();

        // Pattern: "like The Matrix"
        if let Some(caps) = regex::Regex::new(r"like\s+([A-Z][a-zA-Z0-9\s]+)")
            .ok()
            .and_then(|re| re.captures(query))
        {
            if let Some(title) = caps.get(1) {
                references.push(title.as_str().trim().to_string());
            }
        }

        // Pattern: "similar to Inception"
        if let Some(caps) = regex::Regex::new(r"similar to\s+([A-Z][a-zA-Z0-9\s]+)")
            .ok()
            .and_then(|re| re.captures(query))
        {
            if let Some(title) = caps.get(1) {
                references.push(title.as_str().trim().to_string());
            }
        }

        references
    }
}

/// System prompt for GPT intent parsing
const INTENT_PARSER_SYSTEM_PROMPT: &str = r#"You are a media search intent parser.
Extract structured information from user queries about movies and TV shows.
Focus on mood, themes, references to other content, and filters.
Return valid JSON matching the specified schema.
Be conservative with confidence scores - only give high scores when intent is very clear."#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    async fn create_test_cache() -> Arc<RedisCache> {
        let config = Arc::new(crate::config::CacheConfig {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            search_ttl_sec: 1800,
            embedding_ttl_sec: 3600,
            intent_ttl_sec: 600,
        });

        match RedisCache::new(config).await {
            Ok(c) => Arc::new(c),
            Err(e) => panic!("Redis required for tests: {}", e),
        }
    }

    #[tokio::test]
    async fn test_fallback_parse_genres() {
        let cache = create_test_cache().await;
        let parser = IntentParser::new(String::new(), String::new(), cache);
        let intent = parser.fallback_parse("action comedy movies");

        assert_eq!(intent.filters.genre, vec!["action", "comedy"]);
    }

    #[tokio::test]
    async fn test_fallback_parse_platforms() {
        let cache = create_test_cache().await;
        let parser = IntentParser::new(String::new(), String::new(), cache);
        let intent = parser.fallback_parse("netflix shows");

        assert_eq!(intent.filters.platform, vec!["netflix"]);
    }

    #[tokio::test]
    async fn test_extract_references() {
        let cache = create_test_cache().await;
        let parser = IntentParser::new(String::new(), String::new(), cache);
        let references = parser.extract_references("movies like The Matrix");

        assert_eq!(references, vec!["The Matrix"]);
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let config = Arc::new(crate::config::CacheConfig {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            search_ttl_sec: 1800,
            embedding_ttl_sec: 3600,
            intent_ttl_sec: 600,
        });

        let cache = match RedisCache::new(config).await {
            Ok(c) => Arc::new(c),
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let parser = IntentParser::new(String::new(), String::new(), cache.clone());

        let query = "action movies on netflix";
        let normalized = query.trim().to_lowercase();

        // First call should be cache miss
        let intent = parser.fallback_parse(query);

        // Manually cache it
        cache.cache_intent(&normalized, &intent).await.unwrap();

        // Second lookup should be cache hit
        let cached: Option<ParsedIntent> = cache.get_intent(&normalized).await.unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().filters.genre, intent.filters.genre);

        // Cleanup
        cache.clear_intent_cache().await.unwrap();
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let config = Arc::new(crate::config::CacheConfig {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            search_ttl_sec: 1800,
            embedding_ttl_sec: 3600,
            intent_ttl_sec: 600,
        });

        let cache = match RedisCache::new(config).await {
            Ok(c) => Arc::new(c),
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let query = "unique query that should not be cached yet";
        let normalized = query.trim().to_lowercase();

        // Should be cache miss
        let cached: Option<ParsedIntent> = cache.get_intent(&normalized).await.unwrap();
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_query_normalization() {
        let config = Arc::new(crate::config::CacheConfig {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            search_ttl_sec: 1800,
            embedding_ttl_sec: 3600,
            intent_ttl_sec: 600,
        });

        let cache = match RedisCache::new(config).await {
            Ok(c) => Arc::new(c),
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let parser = IntentParser::new(String::new(), String::new(), cache.clone());

        // Different case/whitespace should normalize to same key
        let query1 = "  Action Movies  ";
        let query2 = "action movies";
        let query3 = "ACTION MOVIES";

        let intent = parser.fallback_parse(query1);
        cache
            .cache_intent(&query1.trim().to_lowercase(), &intent)
            .await
            .unwrap();

        // All variations should hit cache
        let cached2: Option<ParsedIntent> = cache
            .get_intent(&query2.trim().to_lowercase())
            .await
            .unwrap();
        let cached3: Option<ParsedIntent> = cache
            .get_intent(&query3.trim().to_lowercase())
            .await
            .unwrap();

        assert!(cached2.is_some());
        assert!(cached3.is_some());

        // Cleanup
        cache.clear_intent_cache().await.unwrap();
    }

    #[tokio::test]
    async fn test_cache_ttl() {
        let config = Arc::new(crate::config::CacheConfig {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            search_ttl_sec: 1800,
            embedding_ttl_sec: 3600,
            intent_ttl_sec: 1, // 1 second TTL for testing
        });

        let cache = match RedisCache::new(config).await {
            Ok(c) => Arc::new(c),
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let query = "ttl test query";
        let normalized = query.trim().to_lowercase();
        let intent = ParsedIntent {
            mood: vec!["test".to_string()],
            themes: vec![],
            references: vec![],
            filters: IntentFilters::default(),
            fallback_query: query.to_string(),
            confidence: 0.5,
        };

        // Cache with 1 second TTL
        cache.cache_intent(&normalized, &intent).await.unwrap();

        // Should be present immediately
        let cached: Option<ParsedIntent> = cache.get_intent(&normalized).await.unwrap();
        assert!(cached.is_some());

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Should be expired
        let cached_after: Option<ParsedIntent> = cache.get_intent(&normalized).await.unwrap();
        assert!(cached_after.is_none());

        // Cleanup
        cache.clear_intent_cache().await.unwrap();
    }
}
