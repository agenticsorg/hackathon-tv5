//! Netflix platform normalizer using Streaming Availability API

use super::{
    extract_array, extract_f64, extract_i64, extract_string, AvailabilityInfo, CanonicalContent,
    ContentType, ImageSet, PlatformNormalizer, RateLimitConfig, RawContent,
};
use crate::{deep_link::DeepLinkResult, IngestionError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

/// Netflix normalizer using Streaming Availability API
pub struct NetflixNormalizer {
    client: Client,
    api_key: String,
    base_url: String,
}

impl NetflixNormalizer {
    /// Create a new Netflix normalizer
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://streaming-availability.p.rapidapi.com".to_string(),
        }
    }

    /// Map Netflix genres to canonical taxonomy
    fn map_netflix_genre(&self, netflix_genre: &str) -> Vec<String> {
        match netflix_genre.to_lowercase().as_str() {
            "action-adventure" | "action & adventure" => vec!["Action".to_string()],
            "sci-fi" | "science fiction" => vec!["Science Fiction".to_string()],
            "thriller" | "suspense" => vec!["Thriller".to_string()],
            "comedy" => vec!["Comedy".to_string()],
            "drama" => vec!["Drama".to_string()],
            "horror" => vec!["Horror".to_string()],
            "romance" | "romantic" => vec!["Romance".to_string()],
            "documentary" => vec!["Documentary".to_string()],
            "animation" | "animated" => vec!["Animation".to_string()],
            "family" | "kids" => vec!["Family".to_string()],
            "fantasy" => vec!["Fantasy".to_string()],
            "mystery" => vec!["Mystery".to_string()],
            "crime" => vec!["Crime".to_string()],
            "war" => vec!["War".to_string()],
            "western" => vec!["Western".to_string()],
            "music" | "musical" => vec!["Music".to_string()],
            "history" | "historical" => vec!["History".to_string()],
            _ => vec![],
        }
    }

    /// Extract external IDs from Streaming Availability API response
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
}

#[async_trait]
impl PlatformNormalizer for NetflixNormalizer {
    fn platform_id(&self) -> &'static str {
        "netflix"
    }

    async fn fetch_catalog_delta(
        &self,
        since: DateTime<Utc>,
        region: &str,
    ) -> Result<Vec<RawContent>> {
        let url = format!(
            "{}/changes?country={}&since={}&service=netflix&show_type=all",
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
                IngestionError::NormalizationFailed("No changes array in response".to_string())
            })?;

        let raw_items = changes
            .iter()
            .filter_map(|item| {
                let id = extract_string(item, "id")?;
                Some(RawContent {
                    id: id.clone(),
                    platform: "netflix".to_string(),
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
            _ => ContentType::Movie, // Default to movie
        };

        // Extract genres and map to canonical taxonomy
        let genres = if let Some(genre_array) = extract_array(data, "genres") {
            genre_array
                .iter()
                .filter_map(|g| g.as_str())
                .flat_map(|g| self.map_netflix_genre(g))
                .collect()
        } else {
            vec![]
        };

        // Extract images
        let images = ImageSet {
            poster_small: extract_string(data, "posterURLs.184"),
            poster_medium: extract_string(data, "posterURLs.342"),
            poster_large: extract_string(data, "posterURLs.780"),
            backdrop: extract_string(data, "backdropURLs.1280"),
        };

        // Extract availability
        let availability = if let Some(streaming_info) = data
            .get("streamingInfo")
            .and_then(|si| si.get("netflix"))
            .and_then(|n| {
                n.get(
                    raw.data
                        .get("country")
                        .and_then(|c| c.as_str())
                        .unwrap_or("us"),
                )
            }) {
            AvailabilityInfo {
                regions: vec![raw
                    .data
                    .get("country")
                    .and_then(|c| c.as_str())
                    .unwrap_or("us")
                    .to_string()],
                subscription_required: true, // Netflix is subscription-based
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

        Ok(CanonicalContent {
            platform_content_id: raw.id,
            platform_id: "netflix".to_string(),
            entity_id: None, // Will be resolved later
            title,
            overview: extract_string(data, "overview"),
            content_type,
            release_year: extract_i64(data, "year").map(|y| y as i32),
            runtime_minutes: extract_i64(data, "runtime").map(|r| r as i32),
            genres,
            external_ids: self.extract_external_ids(data),
            availability,
            images,
            rating: extract_string(data, "rating"),
            user_rating: extract_f64(data, "imdbRating").map(|r| r as f32),
            embedding: None, // Will be generated later
            updated_at: Utc::now(),
        })
    }

    fn generate_deep_link(&self, content_id: &str) -> DeepLinkResult {
        DeepLinkResult {
            mobile_url: Some(format!("netflix://title/{}", content_id)),
            web_url: format!("https://www.netflix.com/title/{}", content_id),
            tv_url: Some(format!("netflix://title/{}", content_id)),
        }
    }

    fn rate_limit_config(&self) -> RateLimitConfig {
        RateLimitConfig {
            max_requests: 100,
            window: Duration::from_secs(60), // 100 requests per minute
            api_keys: vec![self.api_key.clone()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_netflix_genre_mapping() {
        let normalizer = NetflixNormalizer::new("test_key".to_string());

        assert_eq!(
            normalizer.map_netflix_genre("action-adventure"),
            vec!["Action"]
        );
        assert_eq!(
            normalizer.map_netflix_genre("sci-fi"),
            vec!["Science Fiction"]
        );
        assert_eq!(normalizer.map_netflix_genre("comedy"), vec!["Comedy"]);
    }

    #[test]
    fn test_deep_link_generation() {
        let normalizer = NetflixNormalizer::new("test_key".to_string());
        let deep_link = normalizer.generate_deep_link("80057281");

        assert_eq!(
            deep_link.mobile_url,
            Some("netflix://title/80057281".to_string())
        );
        assert_eq!(deep_link.web_url, "https://www.netflix.com/title/80057281");
        assert_eq!(
            deep_link.tv_url,
            Some("netflix://title/80057281".to_string())
        );
    }
}
