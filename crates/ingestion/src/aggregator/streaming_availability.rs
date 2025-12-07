//! Streaming Availability API client
//!
//! Rate limit: 100 requests per minute

use super::{AggregatorContent, AggregatorResponse};
use crate::{IngestionError, Result};
use chrono::{DateTime, Utc};
use moka::future::Cache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// Streaming Availability API client
pub struct StreamingAvailabilityClient {
    client: Client,
    api_key: String,
    base_url: String,
    cache: Cache<String, AggregatorResponse>,
}

impl StreamingAvailabilityClient {
    /// Create a new Streaming Availability API client
    ///
    /// # Arguments
    /// * `api_key` - RapidAPI key for Streaming Availability
    pub fn new(api_key: String) -> Self {
        // Initialize cache with 1 hour TTL
        let cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(3600))
            .build();

        Self {
            client: Client::new(),
            api_key,
            base_url: "https://streaming-availability.p.rapidapi.com".to_string(),
            cache,
        }
    }

    /// Search for content by title
    ///
    /// # Arguments
    /// * `title` - Content title to search for
    /// * `country` - ISO 3166-1 alpha-2 country code
    ///
    /// # Returns
    /// Vector of matching content items
    pub async fn search(&self, title: &str, country: &str) -> Result<Vec<AggregatorContent>> {
        let cache_key = format!("search:{}:{}", title, country);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            return self.parse_search_response(&cached.data);
        }

        let url = format!(
            "{}/search/title?title={}&country={}",
            self.base_url,
            urlencoding::encode(title),
            country
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

        let data: Value = response.json().await?;
        let response_data = AggregatorResponse {
            data: data.clone(),
            fetched_at: Utc::now(),
            source: "streaming_availability".to_string(),
        };

        // Cache the response
        self.cache.insert(cache_key, response_data.clone()).await;

        self.parse_search_response(&data)
    }

    /// Get content details by ID
    ///
    /// # Arguments
    /// * `content_id` - Content ID from Streaming Availability
    ///
    /// # Returns
    /// Content details
    pub async fn get_details(&self, content_id: &str) -> Result<AggregatorContent> {
        let cache_key = format!("details:{}", content_id);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            return self.parse_content_details(&cached.data);
        }

        let url = format!("{}/get?tmdb_id={}", self.base_url, content_id);

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

        let data: Value = response.json().await?;
        let response_data = AggregatorResponse {
            data: data.clone(),
            fetched_at: Utc::now(),
            source: "streaming_availability".to_string(),
        };

        // Cache the response
        self.cache.insert(cache_key, response_data).await;

        self.parse_content_details(&data)
    }

    /// Get changes since a specific date
    ///
    /// # Arguments
    /// * `since` - Get changes since this date
    /// * `country` - ISO 3166-1 alpha-2 country code
    /// * `service` - Streaming service (e.g., "netflix", "prime")
    ///
    /// # Returns
    /// Vector of changed content items
    pub async fn get_changes(
        &self,
        since: DateTime<Utc>,
        country: &str,
        service: &str,
    ) -> Result<Vec<AggregatorContent>> {
        let url = format!(
            "{}/changes?country={}&since={}&service={}&show_type=all",
            self.base_url,
            country,
            since.format("%Y-%m-%d"),
            service
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

        let data: Value = response.json().await?;
        self.parse_changes_response(&data)
    }

    /// Parse search response
    fn parse_search_response(&self, data: &Value) -> Result<Vec<AggregatorContent>> {
        let results = data
            .get("result")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                IngestionError::NormalizationFailed("No results array in response".to_string())
            })?;

        Ok(results
            .iter()
            .filter_map(|item| self.parse_content_item(item).ok())
            .collect::<Vec<_>>())
    }

    /// Parse changes response
    fn parse_changes_response(&self, data: &Value) -> Result<Vec<AggregatorContent>> {
        let changes = data
            .get("changes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                IngestionError::NormalizationFailed("No changes array in response".to_string())
            })?;

        Ok(changes
            .iter()
            .filter_map(|item| self.parse_content_item(item).ok())
            .collect())
    }

    /// Parse content details
    fn parse_content_details(&self, data: &Value) -> Result<AggregatorContent> {
        self.parse_content_item(data)
    }

    /// Parse a single content item
    fn parse_content_item(&self, item: &Value) -> Result<AggregatorContent> {
        let id = item
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IngestionError::NormalizationFailed("Missing id".to_string()))?
            .to_string();

        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IngestionError::NormalizationFailed("Missing title".to_string()))?
            .to_string();

        Ok(AggregatorContent {
            id,
            title,
            overview: item
                .get("overview")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            year: item.get("year").and_then(|v| v.as_i64()).map(|y| y as i32),
            content_type: item
                .get("showType")
                .and_then(|v| v.as_str())
                .unwrap_or("movie")
                .to_string(),
            imdb_id: item
                .get("imdbId")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            tmdb_id: item
                .get("tmdbId")
                .and_then(|v| v.as_i64())
                .map(|i| i as i32),
            genres: item
                .get("genres")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|g| g.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            poster_url: item
                .get("posterURLs")
                .and_then(|p| p.get("342"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            rating: item
                .get("imdbRating")
                .and_then(|v| v.as_f64())
                .map(|r| r as f32),
            runtime: item
                .get("runtime")
                .and_then(|v| v.as_i64())
                .map(|r| r as i32),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = StreamingAvailabilityClient::new("test_key".to_string());
        assert_eq!(
            client.base_url,
            "https://streaming-availability.p.rapidapi.com"
        );
    }
}
