//! YouTube platform normalizer using YouTube Data API v3

use super::{
    extract_array, extract_i64, extract_string, AvailabilityInfo, CanonicalContent, ContentType,
    ImageSet, PlatformNormalizer, RateLimitConfig, RawContent,
};
use crate::{deep_link::DeepLinkResult, IngestionError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Utc};
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

/// YouTube normalizer using YouTube Data API v3
///
/// Uses OAuth 2.0 authentication and manages quota limits (10,000 units/day).
/// Supports multi-key rotation for higher throughput.
pub struct YouTubeNormalizer {
    client: Client,
    api_keys: Vec<String>,
    current_key_index: std::sync::atomic::AtomicUsize,
    base_url: String,
}

impl YouTubeNormalizer {
    /// Create a new YouTube normalizer with API key rotation
    ///
    /// # Arguments
    /// * `api_keys` - Vector of YouTube Data API v3 keys for rotation
    pub fn new(api_keys: Vec<String>) -> Self {
        Self {
            client: Client::new(),
            api_keys,
            current_key_index: std::sync::atomic::AtomicUsize::new(0),
            base_url: "https://www.googleapis.com/youtube/v3".to_string(),
        }
    }

    /// Get next API key in rotation
    fn get_next_api_key(&self) -> String {
        let index = self
            .current_key_index
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.api_keys[index % self.api_keys.len()].clone()
    }

    /// Map YouTube categories to canonical genres
    fn map_youtube_category(&self, category_id: &str) -> Vec<String> {
        match category_id {
            "1" => vec!["Film".to_string()],
            "2" => vec!["Automotive".to_string()],
            "10" => vec!["Music".to_string()],
            "15" => vec!["Pets".to_string()],
            "17" => vec!["Sports".to_string()],
            "19" => vec!["Travel".to_string()],
            "20" => vec!["Gaming".to_string()],
            "22" => vec!["Lifestyle".to_string()],
            "23" => vec!["Comedy".to_string()],
            "24" => vec!["Entertainment".to_string()],
            "25" => vec!["News".to_string()],
            "26" => vec!["How-to".to_string()],
            "27" => vec!["Education".to_string()],
            "28" => vec!["Science".to_string()],
            "29" => vec!["Activism".to_string()],
            _ => vec!["General".to_string()],
        }
    }

    /// Extract video duration in minutes from ISO 8601 format
    fn parse_duration(&self, iso_duration: &str) -> Option<i32> {
        // Parse ISO 8601 duration format: PT#H#M#S
        let duration_str = iso_duration.strip_prefix("PT")?;

        let mut hours = 0;
        let mut minutes = 0;
        let mut seconds = 0;

        let parts: Vec<&str> = duration_str
            .split(|c| c == 'H' || c == 'M' || c == 'S')
            .collect();
        let mut part_index = 0;

        if duration_str.contains('H') {
            hours = parts[part_index].parse().ok()?;
            part_index += 1;
        }
        if duration_str.contains('M') {
            minutes = parts[part_index].parse().ok()?;
            part_index += 1;
        }
        if duration_str.contains('S') {
            seconds = parts[part_index].parse().ok()?;
        }

        Some((hours * 60) + minutes + if seconds > 30 { 1 } else { 0 })
    }
}

#[async_trait]
impl PlatformNormalizer for YouTubeNormalizer {
    fn platform_id(&self) -> &'static str {
        "youtube"
    }

    async fn fetch_catalog_delta(
        &self,
        since: DateTime<Utc>,
        _region: &str,
    ) -> Result<Vec<RawContent>> {
        let api_key = self.get_next_api_key();

        // Search for videos published after 'since' timestamp
        let url = format!(
            "{}/search?part=snippet&type=video&publishedAfter={}&maxResults=50&key={}",
            self.base_url,
            since.to_rfc3339(),
            api_key
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(IngestionError::HttpError(
                response.error_for_status().unwrap_err(),
            ));
        }

        let data: serde_json::Value = response.json().await?;
        let items = data
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                IngestionError::NormalizationFailed("No items array in response".to_string())
            })?;

        // Fetch video details for each item (includes duration, stats, etc.)
        let video_ids: Vec<String> = items
            .iter()
            .filter_map(|item| {
                item.get("id")
                    .and_then(|id| id.get("videoId"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        if video_ids.is_empty() {
            return Ok(vec![]);
        }

        let video_ids_str = video_ids.join(",");
        let details_url = format!(
            "{}/videos?part=snippet,contentDetails,statistics&id={}&key={}",
            self.base_url, video_ids_str, api_key
        );

        let details_response = self.client.get(&details_url).send().await?;
        let details_data: serde_json::Value = details_response.json().await?;

        let raw_items = details_data
            .get("items")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|item| {
                let id = extract_string(item, "id")?;
                Some(RawContent {
                    id: id.clone(),
                    platform: "youtube".to_string(),
                    data: item.clone(),
                    fetched_at: Utc::now(),
                })
            })
            .collect();

        Ok(raw_items)
    }

    fn normalize(&self, raw: RawContent) -> Result<CanonicalContent> {
        let data = &raw.data;

        let snippet = data
            .get("snippet")
            .ok_or_else(|| IngestionError::NormalizationFailed("Missing snippet".to_string()))?;

        let title = extract_string(snippet, "title")
            .ok_or_else(|| IngestionError::NormalizationFailed("Missing title".to_string()))?;

        // Determine content type based on duration
        let content_details = data.get("contentDetails");
        let duration_iso = content_details
            .and_then(|cd| extract_string(cd, "duration"))
            .unwrap_or_else(|| "PT0S".to_string());

        let runtime_minutes = self.parse_duration(&duration_iso);

        let content_type = if let Some(runtime) = runtime_minutes {
            if runtime < 10 {
                ContentType::Short
            } else if runtime > 60 {
                ContentType::Movie
            } else {
                ContentType::Episode
            }
        } else {
            ContentType::Short
        };

        // Extract genres from category
        let category_id = extract_string(snippet, "categoryId").unwrap_or_else(|| "24".to_string());
        let genres = self.map_youtube_category(&category_id);

        // Extract thumbnails
        let thumbnails = snippet.get("thumbnails");
        let images = ImageSet {
            poster_small: thumbnails
                .and_then(|t| t.get("default"))
                .and_then(|d| extract_string(d, "url")),
            poster_medium: thumbnails
                .and_then(|t| t.get("medium"))
                .and_then(|m| extract_string(m, "url")),
            poster_large: thumbnails
                .and_then(|t| t.get("high"))
                .and_then(|h| extract_string(h, "url")),
            backdrop: thumbnails
                .and_then(|t| t.get("maxres"))
                .and_then(|m| extract_string(m, "url")),
        };

        // YouTube content is globally available and free
        let availability = AvailabilityInfo {
            regions: vec!["global".to_string()],
            subscription_required: false,
            purchase_price: None,
            rental_price: None,
            currency: None,
            available_from: snippet
                .get("publishedAt")
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            available_until: None, // YouTube videos don't typically expire
        };

        // Extract external IDs (YouTube video ID maps to itself)
        let mut external_ids = HashMap::new();
        external_ids.insert("youtube".to_string(), raw.id.clone());

        Ok(CanonicalContent {
            platform_content_id: raw.id,
            platform_id: "youtube".to_string(),
            entity_id: None,
            title,
            overview: extract_string(snippet, "description"),
            content_type,
            release_year: snippet
                .get("publishedAt")
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.year()),
            runtime_minutes,
            genres,
            external_ids,
            availability,
            images,
            rating: None, // YouTube doesn't have content ratings in the same way
            user_rating: data
                .get("statistics")
                .and_then(|s| extract_i64(s, "likeCount"))
                .and_then(|likes| {
                    data.get("statistics")
                        .and_then(|s| extract_i64(s, "viewCount"))
                        .map(|views| {
                            if views > 0 {
                                ((likes as f64 / views as f64) * 10.0) as f32
                            } else {
                                0.0
                            }
                        })
                }),
            embedding: None,
            updated_at: Utc::now(),
        })
    }

    fn generate_deep_link(&self, content_id: &str) -> DeepLinkResult {
        DeepLinkResult {
            mobile_url: Some(format!("vnd.youtube://watch?v={}", content_id)),
            web_url: format!("https://www.youtube.com/watch?v={}", content_id),
            tv_url: Some(format!("vnd.youtube://watch?v={}", content_id)),
        }
    }

    fn rate_limit_config(&self) -> RateLimitConfig {
        // YouTube Data API has a quota of 10,000 units per day
        // A search request costs 100 units, so max 100 searches per day per key
        // With 5 keys, we can do 500 searches per day
        RateLimitConfig {
            max_requests: 100,
            window: Duration::from_secs(86400), // 24 hours
            api_keys: self.api_keys.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_youtube_category_mapping() {
        let normalizer = YouTubeNormalizer::new(vec!["test_key".to_string()]);

        assert_eq!(normalizer.map_youtube_category("1"), vec!["Film"]);
        assert_eq!(normalizer.map_youtube_category("10"), vec!["Music"]);
        assert_eq!(normalizer.map_youtube_category("23"), vec!["Comedy"]);
    }

    #[test]
    fn test_duration_parsing() {
        let normalizer = YouTubeNormalizer::new(vec!["test_key".to_string()]);

        assert_eq!(normalizer.parse_duration("PT1H30M15S"), Some(90)); // 1h 30m
        assert_eq!(normalizer.parse_duration("PT5M"), Some(5)); // 5 minutes
        assert_eq!(normalizer.parse_duration("PT45S"), Some(1)); // 45 seconds rounds to 1 minute
        assert_eq!(normalizer.parse_duration("PT2H"), Some(120)); // 2 hours
    }

    #[test]
    fn test_deep_link_generation() {
        let normalizer = YouTubeNormalizer::new(vec!["test_key".to_string()]);
        let deep_link = normalizer.generate_deep_link("dQw4w9WgXcQ");

        assert_eq!(
            deep_link.mobile_url,
            Some("vnd.youtube://watch?v=dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            deep_link.web_url,
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
        );
        assert_eq!(
            deep_link.tv_url,
            Some("vnd.youtube://watch?v=dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn test_api_key_rotation() {
        let keys = vec!["key1".to_string(), "key2".to_string(), "key3".to_string()];
        let normalizer = YouTubeNormalizer::new(keys);

        assert_eq!(normalizer.get_next_api_key(), "key1");
        assert_eq!(normalizer.get_next_api_key(), "key2");
        assert_eq!(normalizer.get_next_api_key(), "key3");
        assert_eq!(normalizer.get_next_api_key(), "key1"); // Should wrap around
    }
}
