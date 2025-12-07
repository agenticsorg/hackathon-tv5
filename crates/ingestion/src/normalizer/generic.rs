//! Generic platform normalizer fallback using Streaming Availability API

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

/// Generic normalizer for platforms without specific implementations
///
/// Uses Streaming Availability API as a fallback for any supported platform.
pub struct GenericNormalizer {
    client: Client,
    api_key: String,
    base_url: String,
    platform_id: String,
    service_id: String, // Service ID for Streaming Availability API
}

impl GenericNormalizer {
    /// Create a new generic normalizer for a specific platform
    ///
    /// # Arguments
    /// * `api_key` - Streaming Availability API key
    /// * `platform_id` - Platform identifier (e.g., "hulu", "hbo_max")
    /// * `service_id` - Service ID used by Streaming Availability API
    pub fn new(api_key: String, platform_id: String, service_id: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://streaming-availability.p.rapidapi.com".to_string(),
            platform_id,
            service_id,
        }
    }

    /// Map generic genres to canonical taxonomy
    fn map_generic_genre(&self, genre: &str) -> Vec<String> {
        match genre.to_lowercase().as_str() {
            "action" => vec!["Action".to_string()],
            "adventure" => vec!["Adventure".to_string()],
            "animation" => vec!["Animation".to_string()],
            "comedy" => vec!["Comedy".to_string()],
            "crime" => vec!["Crime".to_string()],
            "documentary" => vec!["Documentary".to_string()],
            "drama" => vec!["Drama".to_string()],
            "family" => vec!["Family".to_string()],
            "fantasy" => vec!["Fantasy".to_string()],
            "history" => vec!["History".to_string()],
            "horror" => vec!["Horror".to_string()],
            "music" => vec!["Music".to_string()],
            "mystery" => vec!["Mystery".to_string()],
            "romance" => vec!["Romance".to_string()],
            "science fiction" | "sci-fi" => vec!["Science Fiction".to_string()],
            "thriller" => vec!["Thriller".to_string()],
            "war" => vec!["War".to_string()],
            "western" => vec!["Western".to_string()],
            _ => vec![],
        }
    }

    /// Extract external IDs
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
impl PlatformNormalizer for GenericNormalizer {
    fn platform_id(&self) -> &'static str {
        // Need to use Box::leak to convert String to &'static str
        // In production, consider using a better approach
        Box::leak(self.platform_id.clone().into_boxed_str())
    }

    async fn fetch_catalog_delta(
        &self,
        since: DateTime<Utc>,
        region: &str,
    ) -> Result<Vec<RawContent>> {
        let url = format!(
            "{}/changes?country={}&since={}&service={}&show_type=all",
            self.base_url,
            region,
            since.format("%Y-%m-%d"),
            self.service_id
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
                    platform: self.platform_id.clone(),
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
        let genres = if let Some(genre_array) = extract_array(data, "genres") {
            genre_array
                .iter()
                .filter_map(|g| g.as_str())
                .flat_map(|g| self.map_generic_genre(g))
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
            .and_then(|si| si.get(&self.service_id))
            .and_then(|s| s.as_array())
            .and_then(|arr| arr.first())
        {
            let stream_type = extract_string(streaming_info, "type");
            let is_subscription = stream_type.as_deref() == Some("subscription");

            AvailabilityInfo {
                regions: vec![raw
                    .data
                    .get("country")
                    .and_then(|c| c.as_str())
                    .unwrap_or("us")
                    .to_string()],
                subscription_required: is_subscription,
                purchase_price: if !is_subscription {
                    extract_f64(streaming_info, "price").map(|p| p as f32)
                } else {
                    None
                },
                rental_price: extract_f64(streaming_info, "rent").map(|r| r as f32),
                currency: extract_string(streaming_info, "currency"),
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
                subscription_required: false,
                purchase_price: None,
                rental_price: None,
                currency: None,
                available_from: None,
                available_until: None,
            }
        };

        Ok(CanonicalContent {
            platform_content_id: raw.id,
            platform_id: self.platform_id.clone(),
            entity_id: None,
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
            embedding: None,
            updated_at: Utc::now(),
        })
    }

    fn generate_deep_link(&self, content_id: &str) -> DeepLinkResult {
        // Generic deep link - web only
        DeepLinkResult {
            mobile_url: None,
            web_url: format!("https://{}.com/watch/{}", self.platform_id, content_id),
            tv_url: None,
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
    fn test_generic_genre_mapping() {
        let normalizer = GenericNormalizer::new(
            "test_key".to_string(),
            "hulu".to_string(),
            "hulu".to_string(),
        );

        assert_eq!(normalizer.map_generic_genre("action"), vec!["Action"]);
        assert_eq!(
            normalizer.map_generic_genre("sci-fi"),
            vec!["Science Fiction"]
        );
        assert_eq!(
            normalizer.map_generic_genre("documentary"),
            vec!["Documentary"]
        );
    }
}
