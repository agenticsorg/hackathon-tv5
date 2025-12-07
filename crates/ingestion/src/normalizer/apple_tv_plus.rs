//! Apple TV+ platform normalizer
//!
//! Normalizes Apple TV+ content with proper genre mapping,
//! availability handling, and deep link generation.

use super::{
    extract_array, extract_f64, extract_i64, extract_string, AvailabilityInfo, CanonicalContent,
    ContentType, ImageSet, PlatformNormalizer, RateLimitConfig, RawContent,
};
use crate::{deep_link::DeepLinkResult, IngestionError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

/// Apple TV+ normalizer
pub struct AppleTvPlusNormalizer {
    client: Client,
    api_key: String,
    base_url: String,
}

impl AppleTvPlusNormalizer {
    /// Create a new Apple TV+ normalizer
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://streaming-availability.p.rapidapi.com".to_string(),
        }
    }

    /// Map Apple TV+ genres to canonical taxonomy
    fn map_apple_genre(&self, apple_genre: &str) -> Vec<String> {
        match apple_genre.to_lowercase().as_str() {
            // Apple TV+-specific genres
            "apple originals" | "apple original" | "apple tv+ original" => {
                vec!["Drama".to_string()]
            }
            "masterclass" | "documentary series" => vec!["Documentary".to_string()],
            "nature" | "wildlife" => vec!["Documentary".to_string()],

            // Standard genres
            "action-adventure" | "action & adventure" => vec!["Action".to_string()],
            "sci-fi" | "science fiction" => vec!["Science Fiction".to_string()],
            "thriller" | "suspense" => vec!["Thriller".to_string()],
            "comedy" | "comedies" => vec!["Comedy".to_string()],
            "drama" | "dramas" => vec!["Drama".to_string()],
            "horror" => vec!["Horror".to_string()],
            "romance" | "romantic" => vec!["Romance".to_string()],
            "documentary" | "documentaries" => vec!["Documentary".to_string()],
            "animation" | "animated" => vec!["Animation".to_string()],
            "family" | "kids" | "kids & family" => vec!["Family".to_string()],
            "fantasy" => vec!["Fantasy".to_string()],
            "mystery" => vec!["Mystery".to_string()],
            "crime" => vec!["Crime".to_string()],
            "war" => vec!["War".to_string()],
            "western" => vec!["Western".to_string()],
            "music" | "musical" => vec!["Music".to_string()],
            "history" | "historical" => vec!["History".to_string()],
            "reality" | "reality tv" => vec!["Reality".to_string()],
            "talk show" | "late night" => vec!["Talk Show".to_string()],
            "news" => vec!["News".to_string()],
            "sports" => vec!["Sports".to_string()],
            _ => vec![],
        }
    }

    /// Extract external IDs from API response
    fn extract_external_ids(&self, data: &serde_json::Value) -> HashMap<String, String> {
        let mut ids = HashMap::new();

        if let Some(imdb_id) = extract_string(data, "imdbId") {
            ids.insert("imdb".to_string(), imdb_id);
        }
        if let Some(tmdb_id) = extract_i64(data, "tmdbId") {
            ids.insert("tmdb".to_string(), tmdb_id.to_string());
        }
        if let Some(eidr) = extract_string(data, "eidr") {
            ids.insert("eidr".to_string(), eidr);
        }

        ids
    }

    /// Determine subscription tier (Apple TV+ has single tier)
    fn get_subscription_tier(&self, _data: &serde_json::Value) -> String {
        // Apple TV+ has only one tier (no ads)
        "premium".to_string()
    }

    /// Check if content is an Apple TV+ Original
    fn is_apple_original(&self, data: &serde_json::Value) -> bool {
        if let Some(genres) = extract_array(data, "genres") {
            for genre in genres {
                if let Some(g) = genre.as_str() {
                    let lower = g.to_lowercase();
                    if lower.contains("apple original") || lower.contains("apple tv+ original") {
                        return true;
                    }
                }
            }
        }

        // Also check tags
        if let Some(tags) = extract_array(data, "tags") {
            for tag in tags {
                if let Some(t) = tag.as_str() {
                    let lower = t.to_lowercase();
                    if lower.contains("original") {
                        return true;
                    }
                }
            }
        }

        // Most Apple TV+ content is original
        true
    }
}

#[async_trait]
impl PlatformNormalizer for AppleTvPlusNormalizer {
    fn platform_id(&self) -> &'static str {
        "apple_tv_plus"
    }

    async fn fetch_catalog_delta(
        &self,
        since: DateTime<Utc>,
        region: &str,
    ) -> Result<Vec<RawContent>> {
        let url = format!(
            "{}/changes?country={}&since={}&service=apple&show_type=all",
            self.base_url,
            region,
            since.format("%Y-%m-%d")
        );

        let response = self
            .client
            .get(&url)
            .header("X-RapidAPI-Key", &self.api_key)
            .header("X-RapidAPI-Host", "streaming-availability.p.rapidapi.com")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(IngestionError::HttpError(
                response.error_for_status().unwrap_err(),
            ));
        }

        let data: serde_json::Value = response.json().await?;
        let changes = data
            .get("changes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                IngestionError::NormalizationFailed(
                    "No changes array in Apple TV+ response".to_string(),
                )
            })?;

        let raw_items = changes
            .iter()
            .filter_map(|item| {
                let id = extract_string(item, "id")?;
                Some(RawContent {
                    id: id.clone(),
                    platform: "apple_tv_plus".to_string(),
                    data: item.clone(),
                    fetched_at: Utc::now(),
                })
            })
            .collect();

        Ok(raw_items)
    }

    fn normalize(&self, raw: RawContent) -> Result<CanonicalContent> {
        let data = &raw.data;

        let title = extract_string(data, "title")
            .ok_or_else(|| IngestionError::NormalizationFailed("Missing title".to_string()))?;

        let content_type = match extract_string(data, "showType").as_deref() {
            Some("movie") => ContentType::Movie,
            Some("series") => ContentType::Series,
            _ => ContentType::Movie,
        };

        // Extract and map genres
        let mut genres = Vec::new();
        if let Some(genre_array) = extract_array(data, "genres") {
            for g in genre_array {
                if let Some(genre_str) = g.as_str() {
                    genres.extend(self.map_apple_genre(genre_str));
                }
            }
        }
        // Deduplicate genres
        genres.sort();
        genres.dedup();

        // Add Apple Original badge if applicable
        let is_original = self.is_apple_original(data);

        // Extract images
        let images = ImageSet {
            poster_small: extract_string(data, "posterURLs.184"),
            poster_medium: extract_string(data, "posterURLs.342"),
            poster_large: extract_string(data, "posterURLs.780"),
            backdrop: extract_string(data, "backdropURLs.1280"),
        };

        // Extract availability with tier information
        let subscription_tier = self.get_subscription_tier(data);
        let region = raw
            .data
            .get("country")
            .and_then(|c| c.as_str())
            .unwrap_or("us");

        let availability = if let Some(streaming_info) = data
            .get("streamingInfo")
            .and_then(|si| si.get("apple"))
            .and_then(|n| n.get(region))
        {
            AvailabilityInfo {
                regions: vec![region.to_string()],
                subscription_required: true,
                purchase_price: None,
                rental_price: None,
                currency: None,
                available_from: streaming_info
                    .get("addedOn")
                    .and_then(|v| v.as_i64())
                    .and_then(|ts| DateTime::from_timestamp(ts, 0)),
                available_until: streaming_info
                    .get("leaving")
                    .and_then(|v| v.as_i64())
                    .and_then(|ts| DateTime::from_timestamp(ts, 0)),
            }
        } else {
            AvailabilityInfo {
                regions: vec![],
                subscription_required: true,
                purchase_price: None,
                rental_price: None,
                currency: None,
                available_from: None,
                available_until: None,
            }
        };

        let mut external_ids = self.extract_external_ids(data);
        // Add Apple Original flag as metadata
        if is_original {
            external_ids.insert("apple_original".to_string(), "true".to_string());
        }
        external_ids.insert("subscription_tier".to_string(), subscription_tier);

        Ok(CanonicalContent {
            platform_content_id: raw.id,
            platform_id: "apple_tv_plus".to_string(),
            entity_id: None,
            title,
            overview: extract_string(data, "overview"),
            content_type,
            release_year: extract_i64(data, "year").map(|y| y as i32),
            runtime_minutes: extract_i64(data, "runtime").map(|r| r as i32),
            genres,
            external_ids,
            availability,
            images,
            rating: extract_string(data, "rating"),
            user_rating: extract_f64(data, "imdbRating").map(|r| r as f32),
            embedding: None,
            updated_at: Utc::now(),
        })
    }

    fn generate_deep_link(&self, content_id: &str) -> DeepLinkResult {
        DeepLinkResult {
            mobile_url: Some(format!("videos://watch/{}", content_id)),
            web_url: format!("https://tv.apple.com/us/video/{}", content_id),
            tv_url: Some(format!("com.apple.tv://watch/{}", content_id)),
        }
    }

    fn rate_limit_config(&self) -> RateLimitConfig {
        RateLimitConfig {
            max_requests: 100,
            window: Duration::from_secs(60),
            api_keys: vec![self.api_key.clone()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apple_genre_mapping() {
        let normalizer = AppleTvPlusNormalizer::new("test_key".to_string());

        assert_eq!(normalizer.map_apple_genre("apple originals"), vec!["Drama"]);
        assert_eq!(
            normalizer.map_apple_genre("apple tv+ original"),
            vec!["Drama"]
        );
        assert_eq!(
            normalizer.map_apple_genre("documentary series"),
            vec!["Documentary"]
        );
        assert_eq!(normalizer.map_apple_genre("nature"), vec!["Documentary"]);
        assert_eq!(normalizer.map_apple_genre("comedy"), vec!["Comedy"]);
    }

    #[test]
    fn test_deep_link_generation() {
        let normalizer = AppleTvPlusNormalizer::new("test_key".to_string());
        let deep_link = normalizer.generate_deep_link("content123");

        assert!(deep_link.mobile_url.unwrap().contains("videos://"));
        assert!(deep_link.web_url.contains("tv.apple.com"));
        assert!(deep_link.tv_url.unwrap().contains("com.apple.tv://"));
    }

    #[test]
    fn test_subscription_tier_detection() {
        let normalizer = AppleTvPlusNormalizer::new("test_key".to_string());

        let data = serde_json::json!({
            "streamingInfo": {
                "apple": {
                    "us": {}
                }
            }
        });
        assert_eq!(normalizer.get_subscription_tier(&data), "premium");
    }

    #[test]
    fn test_apple_original_detection() {
        let normalizer = AppleTvPlusNormalizer::new("test_key".to_string());

        let original_data = serde_json::json!({
            "genres": ["Apple Original", "Drama"]
        });
        assert!(normalizer.is_apple_original(&original_data));

        // Most Apple TV+ content is original, defaults to true
        let generic_data = serde_json::json!({
            "genres": ["Drama", "Thriller"]
        });
        assert!(normalizer.is_apple_original(&generic_data));
    }

    #[test]
    fn test_normalize_series() {
        let normalizer = AppleTvPlusNormalizer::new("test_key".to_string());
        let raw = RawContent {
            id: "apple123".to_string(),
            platform: "apple_tv_plus".to_string(),
            data: serde_json::json!({
                "title": "Test Series",
                "overview": "A test series",
                "showType": "series",
                "year": 2024,
                "genres": ["Sci-Fi", "Drama"],
                "imdbId": "tt7654321",
                "rating": "TV-MA",
                "imdbRating": 8.5
            }),
            fetched_at: Utc::now(),
        };

        let result = normalizer.normalize(raw);
        assert!(result.is_ok());
        let canonical = result.unwrap();
        assert_eq!(canonical.title, "Test Series");
        assert_eq!(canonical.platform_id, "apple_tv_plus");
        assert_eq!(canonical.content_type, ContentType::Series);
    }

    #[test]
    fn test_external_ids_extraction() {
        let normalizer = AppleTvPlusNormalizer::new("test_key".to_string());
        let data = serde_json::json!({
            "imdbId": "tt9876543",
            "tmdbId": 54321
        });

        let ids = normalizer.extract_external_ids(&data);
        assert_eq!(ids.get("imdb"), Some(&"tt9876543".to_string()));
        assert_eq!(ids.get("tmdb"), Some(&"54321".to_string()));
    }

    #[test]
    fn test_platform_id() {
        let normalizer = AppleTvPlusNormalizer::new("test_key".to_string());
        assert_eq!(normalizer.platform_id(), "apple_tv_plus");
    }
}
