//! Watchmode API client
//!
//! Rate limit: 1000 requests per day

use super::{AggregatorContent, AggregatorResponse};
use crate::{IngestionError, Result};
use chrono::Utc;
use moka::future::Cache;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

/// Watchmode API client (fallback aggregator)
pub struct WatchmodeClient {
    client: Client,
    api_key: String,
    base_url: String,
    cache: Cache<String, AggregatorResponse>,
}

impl WatchmodeClient {
    /// Create a new Watchmode API client
    ///
    /// # Arguments
    /// * `api_key` - Watchmode API key
    pub fn new(api_key: String) -> Self {
        // Initialize cache with 24 hour TTL (due to daily rate limit)
        let cache = Cache::builder()
            .max_capacity(5_000)
            .time_to_live(Duration::from_secs(86400))
            .build();

        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.watchmode.com/v1".to_string(),
            cache,
        }
    }

    /// Search for content by title
    ///
    /// # Arguments
    /// * `title` - Content title to search for
    ///
    /// # Returns
    /// Vector of matching content items
    pub async fn search(&self, title: &str) -> Result<Vec<AggregatorContent>> {
        let cache_key = format!("search:{}", title);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            return self.parse_search_response(&cached.data);
        }

        let url = format!(
            "{}/search/?apiKey={}&search_field=name&search_value={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(title)
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(IngestionError::HttpError(
                response.error_for_status().unwrap_err(),
            ));
        }

        let data: Value = response.json().await?;
        let response_data = AggregatorResponse {
            data: data.clone(),
            fetched_at: Utc::now(),
            source: "watchmode".to_string(),
        };

        // Cache the response
        self.cache.insert(cache_key, response_data).await;

        self.parse_search_response(&data)
    }

    /// Get title details
    ///
    /// # Arguments
    /// * `title_id` - Watchmode title ID
    ///
    /// # Returns
    /// Content details
    pub async fn get_title_details(&self, title_id: &str) -> Result<AggregatorContent> {
        let cache_key = format!("details:{}", title_id);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            return self.parse_title_details(&cached.data);
        }

        let url = format!(
            "{}/title/{}/details/?apiKey={}",
            self.base_url, title_id, self.api_key
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(IngestionError::HttpError(
                response.error_for_status().unwrap_err(),
            ));
        }

        let data: Value = response.json().await?;
        let response_data = AggregatorResponse {
            data: data.clone(),
            fetched_at: Utc::now(),
            source: "watchmode".to_string(),
        };

        // Cache the response
        self.cache.insert(cache_key, response_data).await;

        self.parse_title_details(&data)
    }

    /// Get sources (streaming availability)
    ///
    /// # Arguments
    /// * `title_id` - Watchmode title ID
    ///
    /// # Returns
    /// Availability data as JSON
    pub async fn get_sources(&self, title_id: &str) -> Result<Value> {
        let cache_key = format!("sources:{}", title_id);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(cached.data);
        }

        let url = format!(
            "{}/title/{}/sources/?apiKey={}",
            self.base_url, title_id, self.api_key
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(IngestionError::HttpError(
                response.error_for_status().unwrap_err(),
            ));
        }

        let data: Value = response.json().await?;
        let response_data = AggregatorResponse {
            data: data.clone(),
            fetched_at: Utc::now(),
            source: "watchmode".to_string(),
        };

        // Cache the response
        self.cache.insert(cache_key, response_data).await;

        Ok(data)
    }

    /// Parse search response
    fn parse_search_response(&self, data: &Value) -> Result<Vec<AggregatorContent>> {
        let results = data
            .get("title_results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                IngestionError::NormalizationFailed(
                    "No title_results array in response".to_string(),
                )
            })?;

        Ok(results
            .iter()
            .filter_map(|item| self.parse_search_item(item).ok())
            .collect())
    }

    /// Parse search item
    fn parse_search_item(&self, item: &Value) -> Result<AggregatorContent> {
        let id = item
            .get("id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| IngestionError::NormalizationFailed("Missing id".to_string()))?
            .to_string();

        let title = item
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IngestionError::NormalizationFailed("Missing name".to_string()))?
            .to_string();

        Ok(AggregatorContent {
            id,
            title,
            overview: None,
            year: item.get("year").and_then(|v| v.as_i64()).map(|y| y as i32),
            content_type: item
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("movie")
                .to_string(),
            imdb_id: item
                .get("imdb_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            tmdb_id: item
                .get("tmdb_id")
                .and_then(|v| v.as_i64())
                .map(|i| i as i32),
            genres: vec![],
            poster_url: None,
            rating: None,
            runtime: None,
        })
    }

    /// Parse title details
    fn parse_title_details(&self, data: &Value) -> Result<AggregatorContent> {
        let id = data
            .get("id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| IngestionError::NormalizationFailed("Missing id".to_string()))?
            .to_string();

        let title = data
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IngestionError::NormalizationFailed("Missing title".to_string()))?
            .to_string();

        Ok(AggregatorContent {
            id,
            title,
            overview: data
                .get("plot_overview")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            year: data.get("year").and_then(|v| v.as_i64()).map(|y| y as i32),
            content_type: data
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("movie")
                .to_string(),
            imdb_id: data
                .get("imdb_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            tmdb_id: data
                .get("tmdb_id")
                .and_then(|v| v.as_i64())
                .map(|i| i as i32),
            genres: data
                .get("genre_names")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|g| g.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            poster_url: data
                .get("poster")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            rating: data
                .get("user_rating")
                .and_then(|v| v.as_f64())
                .map(|r| r as f32),
            runtime: data
                .get("runtime_minutes")
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
        let client = WatchmodeClient::new("test_key".to_string());
        assert_eq!(client.base_url, "https://api.watchmode.com/v1");
    }
}
