//! Peacock platform normalizer
//!
//! Normalizes Peacock content with proper genre mapping,
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

/// Peacock normalizer
pub struct PeacockNormalizer {
    client: Client,
    api_key: String,
    base_url: String,
}

impl PeacockNormalizer {
    /// Create a new Peacock normalizer
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://streaming-availability.p.rapidapi.com".to_string(),
        }
    }

    /// Map Peacock genres to canonical taxonomy
    fn map_peacock_genre(&self, peacock_genre: &str) -> Vec<String> {
        match peacock_genre.to_lowercase().as_str() {
            // Peacock-specific genres
            "peacock originals" | "peacock original" => vec!["Drama".to_string()],
            "nbc originals" | "nbc original" => vec!["Drama".to_string()],
            "universal" | "universal pictures" => vec!["Drama".to_string()],
            "wwe" | "wrestling" => vec!["Sports".to_string()],
            "premier league" | "soccer" => vec!["Sports".to_string()],
            "true crime" => vec!["Crime".to_string(), "Documentary".to_string()],

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

    /// Determine subscription tier from availability data
    fn get_subscription_tier(&self, data: &serde_json::Value) -> String {
        // Check for different Peacock tiers
        if let Some(streaming_info) = data.get("streamingInfo") {
            if let Some(peacock) = streaming_info.get("peacock") {
                if let Some(us) = peacock.get("us") {
                    // Premium Plus (no ads)
                    if let Some(premium_plus) = us.get("premiumPlus") {
                        if premium_plus.as_bool().unwrap_or(false) {
                            return "premium-plus".to_string();
                        }
                    }
                    // Premium (limited ads)
                    if let Some(premium) = us.get("premium") {
                        if premium.as_bool().unwrap_or(false) {
                            return "premium".to_string();
                        }
                    }
                    // Free tier (with ads)
                    if let Some(free) = us.get("free") {
                        if free.as_bool().unwrap_or(false) {
                            return "free".to_string();
                        }
                    }
                }
            }
        }
        "premium".to_string()
    }

    /// Check if content is a Peacock Original
    fn is_peacock_original(&self, data: &serde_json::Value) -> bool {
        if let Some(genres) = extract_array(data, "genres") {
            for genre in genres {
                if let Some(g) = genre.as_str() {
                    let lower = g.to_lowercase();
                    if lower.contains("peacock original") || lower.contains("nbc original") {
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

        false
    }
}

#[async_trait]
impl PlatformNormalizer for PeacockNormalizer {
    fn platform_id(&self) -> &'static str {
        "peacock"
    }

    async fn fetch_catalog_delta(
        &self,
        since: DateTime<Utc>,
        region: &str,
    ) -> Result<Vec<RawContent>> {
        let url = format!(
            "{}/changes?country={}&since={}&service=peacock&show_type=all",
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
                    "No changes array in Peacock response".to_string(),
                )
            })?;

        let raw_items = changes
            .iter()
            .filter_map(|item| {
                let id = extract_string(item, "id")?;
                Some(RawContent {
                    id: id.clone(),
                    platform: "peacock".to_string(),
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
                    genres.extend(self.map_peacock_genre(genre_str));
                }
            }
        }
        // Deduplicate genres
        genres.sort();
        genres.dedup();

        // Add Peacock Original badge if applicable
        let is_original = self.is_peacock_original(data);

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
            .and_then(|si| si.get("peacock"))
            .and_then(|n| n.get(region))
        {
            let subscription_required = subscription_tier != "free";
            AvailabilityInfo {
                regions: vec![region.to_string()],
                subscription_required,
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
        // Add Peacock Original flag as metadata
        if is_original {
            external_ids.insert("peacock_original".to_string(), "true".to_string());
        }
        external_ids.insert("subscription_tier".to_string(), subscription_tier);

        Ok(CanonicalContent {
            platform_content_id: raw.id,
            platform_id: "peacock".to_string(),
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
            mobile_url: Some(format!("peacock://watch/{}", content_id)),
            web_url: format!("https://www.peacocktv.com/watch/{}", content_id),
            tv_url: Some(format!("peacock://watch/{}", content_id)),
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
    fn test_peacock_genre_mapping() {
        let normalizer = PeacockNormalizer::new("test_key".to_string());

        assert_eq!(
            normalizer.map_peacock_genre("peacock originals"),
            vec!["Drama"]
        );
        assert_eq!(normalizer.map_peacock_genre("nbc originals"), vec!["Drama"]);
        assert_eq!(normalizer.map_peacock_genre("wwe"), vec!["Sports"]);
        assert_eq!(
            normalizer.map_peacock_genre("premier league"),
            vec!["Sports"]
        );
        assert_eq!(
            normalizer.map_peacock_genre("true crime"),
            vec!["Crime", "Documentary"]
        );
        assert_eq!(normalizer.map_peacock_genre("comedy"), vec!["Comedy"]);
    }

    #[test]
    fn test_deep_link_generation() {
        let normalizer = PeacockNormalizer::new("test_key".to_string());
        let deep_link = normalizer.generate_deep_link("watch123");

        assert!(deep_link.mobile_url.unwrap().contains("peacock://"));
        assert!(deep_link.web_url.contains("peacocktv.com"));
        assert!(deep_link.tv_url.unwrap().contains("peacock://"));
    }

    #[test]
    fn test_subscription_tier_detection() {
        let normalizer = PeacockNormalizer::new("test_key".to_string());

        let data_premium_plus = serde_json::json!({
            "streamingInfo": {
                "peacock": {
                    "us": {
                        "premiumPlus": true
                    }
                }
            }
        });
        assert_eq!(
            normalizer.get_subscription_tier(&data_premium_plus),
            "premium-plus"
        );

        let data_premium = serde_json::json!({
            "streamingInfo": {
                "peacock": {
                    "us": {
                        "premium": true
                    }
                }
            }
        });
        assert_eq!(normalizer.get_subscription_tier(&data_premium), "premium");

        let data_free = serde_json::json!({
            "streamingInfo": {
                "peacock": {
                    "us": {
                        "free": true
                    }
                }
            }
        });
        assert_eq!(normalizer.get_subscription_tier(&data_free), "free");
    }

    #[test]
    fn test_peacock_original_detection() {
        let normalizer = PeacockNormalizer::new("test_key".to_string());

        let original_data = serde_json::json!({
            "genres": ["Peacock Original", "Drama"]
        });
        assert!(normalizer.is_peacock_original(&original_data));

        let nbc_data = serde_json::json!({
            "genres": ["NBC Original", "Comedy"]
        });
        assert!(normalizer.is_peacock_original(&nbc_data));

        let non_original_data = serde_json::json!({
            "genres": ["Drama", "Thriller"]
        });
        assert!(!normalizer.is_peacock_original(&non_original_data));
    }

    #[test]
    fn test_normalize_series() {
        let normalizer = PeacockNormalizer::new("test_key".to_string());
        let raw = RawContent {
            id: "peacock456".to_string(),
            platform: "peacock".to_string(),
            data: serde_json::json!({
                "title": "Test Series",
                "overview": "A test series",
                "showType": "series",
                "year": 2024,
                "genres": ["Drama", "Mystery"],
                "imdbId": "tt5555555",
                "rating": "TV-14",
                "imdbRating": 8.2
            }),
            fetched_at: Utc::now(),
        };

        let result = normalizer.normalize(raw);
        assert!(result.is_ok());
        let canonical = result.unwrap();
        assert_eq!(canonical.title, "Test Series");
        assert_eq!(canonical.platform_id, "peacock");
        assert_eq!(canonical.content_type, ContentType::Series);
    }

    #[test]
    fn test_external_ids_extraction() {
        let normalizer = PeacockNormalizer::new("test_key".to_string());
        let data = serde_json::json!({
            "imdbId": "tt9999999",
            "tmdbId": 99999
        });

        let ids = normalizer.extract_external_ids(&data);
        assert_eq!(ids.get("imdb"), Some(&"tt9999999".to_string()));
        assert_eq!(ids.get("tmdb"), Some(&"99999".to_string()));
    }

    #[test]
    fn test_platform_id() {
        let normalizer = PeacockNormalizer::new("test_key".to_string());
        assert_eq!(normalizer.platform_id(), "peacock");
    }

    #[test]
    fn test_free_tier_subscription() {
        let normalizer = PeacockNormalizer::new("test_key".to_string());
        let raw = RawContent {
            id: "peacock_free".to_string(),
            platform: "peacock".to_string(),
            data: serde_json::json!({
                "title": "Free Content",
                "showType": "movie",
                "streamingInfo": {
                    "peacock": {
                        "us": {
                            "free": true
                        }
                    }
                },
                "country": "us"
            }),
            fetched_at: Utc::now(),
        };

        let result = normalizer.normalize(raw);
        assert!(result.is_ok());
        let canonical = result.unwrap();
        assert_eq!(canonical.availability.subscription_required, false);
    }
}
