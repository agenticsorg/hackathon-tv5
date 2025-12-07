//! Hulu platform normalizer
//!
//! Normalizes Hulu content with proper genre mapping,
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

/// Hulu normalizer
pub struct HuluNormalizer {
    client: Client,
    api_key: String,
    base_url: String,
}

impl HuluNormalizer {
    /// Create a new Hulu normalizer
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://streaming-availability.p.rapidapi.com".to_string(),
        }
    }

    /// Map Hulu genres to canonical taxonomy
    fn map_hulu_genre(&self, hulu_genre: &str) -> Vec<String> {
        match hulu_genre.to_lowercase().as_str() {
            // Hulu-specific genres
            "hulu originals" | "hulu original" => vec!["Drama".to_string()],
            "fx originals" | "fx original" => vec!["Drama".to_string()],
            "live tv" | "live television" => vec!["Reality".to_string()],
            "anime" => vec!["Animation".to_string()],

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
        // Check for ad-supported tier
        if let Some(streaming_info) = data.get("streamingInfo") {
            if let Some(hulu) = streaming_info.get("hulu") {
                if let Some(us) = hulu.get("us") {
                    if let Some(ads) = us.get("ads") {
                        if ads.as_bool().unwrap_or(false) {
                            return "ad-supported".to_string();
                        }
                    }
                    // Check for live TV add-on
                    if let Some(live_tv) = us.get("liveTV") {
                        if live_tv.as_bool().unwrap_or(false) {
                            return "live-tv".to_string();
                        }
                    }
                }
            }
        }
        "ad-free".to_string()
    }

    /// Check if content is a Hulu Original
    fn is_hulu_original(&self, data: &serde_json::Value) -> bool {
        if let Some(genres) = extract_array(data, "genres") {
            for genre in genres {
                if let Some(g) = genre.as_str() {
                    let lower = g.to_lowercase();
                    if lower.contains("hulu original") || lower.contains("fx original") {
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
impl PlatformNormalizer for HuluNormalizer {
    fn platform_id(&self) -> &'static str {
        "hulu"
    }

    async fn fetch_catalog_delta(
        &self,
        since: DateTime<Utc>,
        region: &str,
    ) -> Result<Vec<RawContent>> {
        let url = format!(
            "{}/changes?country={}&since={}&service=hulu&show_type=all",
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
                IngestionError::NormalizationFailed("No changes array in Hulu response".to_string())
            })?;

        let raw_items = changes
            .iter()
            .filter_map(|item| {
                let id = extract_string(item, "id")?;
                Some(RawContent {
                    id: id.clone(),
                    platform: "hulu".to_string(),
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
                    genres.extend(self.map_hulu_genre(genre_str));
                }
            }
        }
        // Deduplicate genres
        genres.sort();
        genres.dedup();

        // Add Hulu Original badge if applicable
        let is_original = self.is_hulu_original(data);

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
            .and_then(|si| si.get("hulu"))
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
        // Add Hulu Original flag as metadata
        if is_original {
            external_ids.insert("hulu_original".to_string(), "true".to_string());
        }
        external_ids.insert("subscription_tier".to_string(), subscription_tier);

        Ok(CanonicalContent {
            platform_content_id: raw.id,
            platform_id: "hulu".to_string(),
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
            mobile_url: Some(format!("hulu://watch/{}", content_id)),
            web_url: format!("https://www.hulu.com/watch/{}", content_id),
            tv_url: Some(format!("hulu://watch/{}", content_id)),
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
    fn test_hulu_genre_mapping() {
        let normalizer = HuluNormalizer::new("test_key".to_string());

        assert_eq!(normalizer.map_hulu_genre("hulu originals"), vec!["Drama"]);
        assert_eq!(normalizer.map_hulu_genre("fx originals"), vec!["Drama"]);
        assert_eq!(normalizer.map_hulu_genre("anime"), vec!["Animation"]);
        assert_eq!(normalizer.map_hulu_genre("live tv"), vec!["Reality"]);
        assert_eq!(normalizer.map_hulu_genre("comedy"), vec!["Comedy"]);
    }

    #[test]
    fn test_deep_link_generation() {
        let normalizer = HuluNormalizer::new("test_key".to_string());
        let deep_link = normalizer.generate_deep_link("xyz789");

        assert!(deep_link.mobile_url.unwrap().contains("hulu://"));
        assert!(deep_link.web_url.contains("hulu.com"));
        assert!(deep_link.tv_url.unwrap().contains("hulu://"));
    }

    #[test]
    fn test_subscription_tier_detection() {
        let normalizer = HuluNormalizer::new("test_key".to_string());

        let data = serde_json::json!({
            "streamingInfo": {
                "hulu": {
                    "us": {
                        "ads": true
                    }
                }
            }
        });
        assert_eq!(normalizer.get_subscription_tier(&data), "ad-supported");

        let data_live = serde_json::json!({
            "streamingInfo": {
                "hulu": {
                    "us": {
                        "ads": false,
                        "liveTV": true
                    }
                }
            }
        });
        assert_eq!(normalizer.get_subscription_tier(&data_live), "live-tv");

        let data_ad_free = serde_json::json!({
            "streamingInfo": {
                "hulu": {
                    "us": {
                        "ads": false
                    }
                }
            }
        });
        assert_eq!(normalizer.get_subscription_tier(&data_ad_free), "ad-free");
    }

    #[test]
    fn test_hulu_original_detection() {
        let normalizer = HuluNormalizer::new("test_key".to_string());

        let original_data = serde_json::json!({
            "genres": ["Hulu Original", "Drama"]
        });
        assert!(normalizer.is_hulu_original(&original_data));

        let fx_original_data = serde_json::json!({
            "genres": ["FX Original", "Comedy"]
        });
        assert!(normalizer.is_hulu_original(&fx_original_data));

        let non_original_data = serde_json::json!({
            "genres": ["Drama", "Thriller"]
        });
        assert!(!normalizer.is_hulu_original(&non_original_data));
    }

    #[test]
    fn test_normalize_movie() {
        let normalizer = HuluNormalizer::new("test_key".to_string());
        let raw = RawContent {
            id: "hulu123".to_string(),
            platform: "hulu".to_string(),
            data: serde_json::json!({
                "title": "Test Movie",
                "overview": "A test movie",
                "showType": "movie",
                "year": 2024,
                "runtime": 120,
                "genres": ["Action", "Comedy"],
                "imdbId": "tt1234567",
                "rating": "PG-13",
                "imdbRating": 7.5
            }),
            fetched_at: Utc::now(),
        };

        let result = normalizer.normalize(raw);
        assert!(result.is_ok());
        let canonical = result.unwrap();
        assert_eq!(canonical.title, "Test Movie");
        assert_eq!(canonical.platform_id, "hulu");
        assert_eq!(canonical.content_type, ContentType::Movie);
    }

    #[test]
    fn test_external_ids_extraction() {
        let normalizer = HuluNormalizer::new("test_key".to_string());
        let data = serde_json::json!({
            "imdbId": "tt1234567",
            "tmdbId": 12345,
            "eidr": "10.5240/AAAA-BBBB-CCCC"
        });

        let ids = normalizer.extract_external_ids(&data);
        assert_eq!(ids.get("imdb"), Some(&"tt1234567".to_string()));
        assert_eq!(ids.get("tmdb"), Some(&"12345".to_string()));
        assert_eq!(ids.get("eidr"), Some(&"10.5240/AAAA-BBBB-CCCC".to_string()));
    }
}
