//! Paramount+ platform normalizer
//!
//! Normalizes Paramount+ content with proper genre mapping,
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

/// Paramount+ normalizer
pub struct ParamountPlusNormalizer {
    client: Client,
    api_key: String,
    base_url: String,
}

impl ParamountPlusNormalizer {
    /// Create a new Paramount+ normalizer
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://streaming-availability.p.rapidapi.com".to_string(),
        }
    }

    /// Map Paramount+ genres to canonical taxonomy
    fn map_paramount_genre(&self, paramount_genre: &str) -> Vec<String> {
        match paramount_genre.to_lowercase().as_str() {
            // Paramount+-specific genres
            "paramount+ original" | "paramount originals" => vec!["Drama".to_string()],
            "cbs originals" | "cbs original" => vec!["Drama".to_string()],
            "mtv originals" | "mtv original" => vec!["Reality".to_string()],
            "nickelodeon" | "nick" => vec!["Family".to_string(), "Animation".to_string()],
            "showtime" | "showtime originals" => vec!["Drama".to_string()],
            "star trek" => vec!["Science Fiction".to_string()],

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
        // Check for Showtime bundle
        if let Some(streaming_info) = data.get("streamingInfo") {
            if let Some(paramount) = streaming_info.get("paramount") {
                if let Some(us) = paramount.get("us") {
                    if let Some(showtime) = us.get("showtime") {
                        if showtime.as_bool().unwrap_or(false) {
                            return "showtime".to_string();
                        }
                    }
                    // Check for ad-supported tier
                    if let Some(ads) = us.get("ads") {
                        if ads.as_bool().unwrap_or(false) {
                            return "essential".to_string();
                        }
                    }
                }
            }
        }
        "premium".to_string()
    }

    /// Check if content is a Paramount+ Original
    fn is_paramount_original(&self, data: &serde_json::Value) -> bool {
        if let Some(genres) = extract_array(data, "genres") {
            for genre in genres {
                if let Some(g) = genre.as_str() {
                    let lower = g.to_lowercase();
                    if lower.contains("paramount+ original")
                        || lower.contains("paramount originals")
                        || lower.contains("cbs original")
                        || lower.contains("showtime original")
                    {
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
impl PlatformNormalizer for ParamountPlusNormalizer {
    fn platform_id(&self) -> &'static str {
        "paramount_plus"
    }

    async fn fetch_catalog_delta(
        &self,
        since: DateTime<Utc>,
        region: &str,
    ) -> Result<Vec<RawContent>> {
        let url = format!(
            "{}/changes?country={}&since={}&service=paramount&show_type=all",
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
                    "No changes array in Paramount+ response".to_string(),
                )
            })?;

        let raw_items = changes
            .iter()
            .filter_map(|item| {
                let id = extract_string(item, "id")?;
                Some(RawContent {
                    id: id.clone(),
                    platform: "paramount_plus".to_string(),
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
                    genres.extend(self.map_paramount_genre(genre_str));
                }
            }
        }
        // Deduplicate genres
        genres.sort();
        genres.dedup();

        // Add Paramount+ Original badge if applicable
        let is_original = self.is_paramount_original(data);

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
            .and_then(|si| si.get("paramount"))
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
        // Add Paramount+ Original flag as metadata
        if is_original {
            external_ids.insert("paramount_original".to_string(), "true".to_string());
        }
        external_ids.insert("subscription_tier".to_string(), subscription_tier);

        Ok(CanonicalContent {
            platform_content_id: raw.id,
            platform_id: "paramount_plus".to_string(),
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
            mobile_url: Some(format!("paramountplus://content/{}", content_id)),
            web_url: format!("https://www.paramountplus.com/movies/{}", content_id),
            tv_url: Some(format!("paramountplus://content/{}", content_id)),
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
    fn test_paramount_genre_mapping() {
        let normalizer = ParamountPlusNormalizer::new("test_key".to_string());

        assert_eq!(
            normalizer.map_paramount_genre("paramount+ original"),
            vec!["Drama"]
        );
        assert_eq!(
            normalizer.map_paramount_genre("cbs originals"),
            vec!["Drama"]
        );
        assert_eq!(
            normalizer.map_paramount_genre("mtv originals"),
            vec!["Reality"]
        );
        assert_eq!(
            normalizer.map_paramount_genre("nickelodeon"),
            vec!["Family", "Animation"]
        );
        assert_eq!(
            normalizer.map_paramount_genre("star trek"),
            vec!["Science Fiction"]
        );
        assert_eq!(normalizer.map_paramount_genre("comedy"), vec!["Comedy"]);
    }

    #[test]
    fn test_deep_link_generation() {
        let normalizer = ParamountPlusNormalizer::new("test_key".to_string());
        let deep_link = normalizer.generate_deep_link("content456");

        assert!(deep_link.mobile_url.unwrap().contains("paramountplus://"));
        assert!(deep_link.web_url.contains("paramountplus.com"));
        assert!(deep_link.tv_url.unwrap().contains("paramountplus://"));
    }

    #[test]
    fn test_subscription_tier_detection() {
        let normalizer = ParamountPlusNormalizer::new("test_key".to_string());

        let data_showtime = serde_json::json!({
            "streamingInfo": {
                "paramount": {
                    "us": {
                        "showtime": true
                    }
                }
            }
        });
        assert_eq!(normalizer.get_subscription_tier(&data_showtime), "showtime");

        let data_essential = serde_json::json!({
            "streamingInfo": {
                "paramount": {
                    "us": {
                        "ads": true
                    }
                }
            }
        });
        assert_eq!(
            normalizer.get_subscription_tier(&data_essential),
            "essential"
        );

        let data_premium = serde_json::json!({
            "streamingInfo": {
                "paramount": {
                    "us": {
                        "ads": false
                    }
                }
            }
        });
        assert_eq!(normalizer.get_subscription_tier(&data_premium), "premium");
    }

    #[test]
    fn test_paramount_original_detection() {
        let normalizer = ParamountPlusNormalizer::new("test_key".to_string());

        let original_data = serde_json::json!({
            "genres": ["Paramount+ Original", "Drama"]
        });
        assert!(normalizer.is_paramount_original(&original_data));

        let cbs_data = serde_json::json!({
            "genres": ["CBS Original", "Comedy"]
        });
        assert!(normalizer.is_paramount_original(&cbs_data));

        let non_original_data = serde_json::json!({
            "genres": ["Drama", "Thriller"]
        });
        assert!(!normalizer.is_paramount_original(&non_original_data));
    }

    #[test]
    fn test_normalize_movie() {
        let normalizer = ParamountPlusNormalizer::new("test_key".to_string());
        let raw = RawContent {
            id: "paramount123".to_string(),
            platform: "paramount_plus".to_string(),
            data: serde_json::json!({
                "title": "Test Movie",
                "overview": "A test movie",
                "showType": "movie",
                "year": 2024,
                "runtime": 110,
                "genres": ["Action", "Sci-Fi"],
                "imdbId": "tt2468902",
                "rating": "PG-13",
                "imdbRating": 7.8
            }),
            fetched_at: Utc::now(),
        };

        let result = normalizer.normalize(raw);
        assert!(result.is_ok());
        let canonical = result.unwrap();
        assert_eq!(canonical.title, "Test Movie");
        assert_eq!(canonical.platform_id, "paramount_plus");
        assert_eq!(canonical.content_type, ContentType::Movie);
    }

    #[test]
    fn test_external_ids_extraction() {
        let normalizer = ParamountPlusNormalizer::new("test_key".to_string());
        let data = serde_json::json!({
            "imdbId": "tt8888888",
            "tmdbId": 88888,
            "eidr": "10.5240/XXXX-YYYY-ZZZZ"
        });

        let ids = normalizer.extract_external_ids(&data);
        assert_eq!(ids.get("imdb"), Some(&"tt8888888".to_string()));
        assert_eq!(ids.get("tmdb"), Some(&"88888".to_string()));
        assert_eq!(ids.get("eidr"), Some(&"10.5240/XXXX-YYYY-ZZZZ".to_string()));
    }

    #[test]
    fn test_platform_id() {
        let normalizer = ParamountPlusNormalizer::new("test_key".to_string());
        assert_eq!(normalizer.platform_id(), "paramount_plus");
    }
}
