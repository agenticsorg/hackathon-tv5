//! TMDb (The Movie Database) API client
//!
//! Rate limit: 40 requests per 10 seconds
//! Cache: 7 days for metadata enrichment

use super::{AggregatorContent, AggregatorResponse};
use crate::{IngestionError, Result};
use chrono::Utc;
use moka::future::Cache;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

/// TMDb API client for metadata enrichment
pub struct TMDbClient {
    client: Client,
    api_key: String,
    base_url: String,
    cache: Cache<String, AggregatorResponse>,
}

impl TMDbClient {
    /// Create a new TMDb API client
    ///
    /// # Arguments
    /// * `api_key` - TMDb API key (v3)
    pub fn new(api_key: String) -> Self {
        // Initialize cache with 7 day TTL
        let cache = Cache::builder()
            .max_capacity(50_000)
            .time_to_live(Duration::from_secs(7 * 86400))
            .build();

        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.themoviedb.org/3".to_string(),
            cache,
        }
    }

    /// Search for movies by title
    ///
    /// # Arguments
    /// * `query` - Movie title to search for
    /// * `year` - Optional release year for filtering
    ///
    /// # Returns
    /// Vector of matching movies
    pub async fn search_movie(
        &self,
        query: &str,
        year: Option<i32>,
    ) -> Result<Vec<AggregatorContent>> {
        let cache_key = format!("search_movie:{}:{:?}", query, year);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            return self.parse_search_response(&cached.data);
        }

        let mut url = format!(
            "{}/search/movie?api_key={}&query={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(query)
        );

        if let Some(y) = year {
            url.push_str(&format!("&year={}", y));
        }

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
            source: "tmdb".to_string(),
        };

        // Cache the response
        self.cache.insert(cache_key, response_data).await;

        self.parse_search_response(&data)
    }

    /// Search for TV shows by title
    ///
    /// # Arguments
    /// * `query` - TV show title to search for
    /// * `year` - Optional first air year for filtering
    ///
    /// # Returns
    /// Vector of matching TV shows
    pub async fn search_tv(
        &self,
        query: &str,
        year: Option<i32>,
    ) -> Result<Vec<AggregatorContent>> {
        let cache_key = format!("search_tv:{}:{:?}", query, year);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            return self.parse_search_response(&cached.data);
        }

        let mut url = format!(
            "{}/search/tv?api_key={}&query={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(query)
        );

        if let Some(y) = year {
            url.push_str(&format!("&first_air_date_year={}", y));
        }

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
            source: "tmdb".to_string(),
        };

        // Cache the response
        self.cache.insert(cache_key, response_data).await;

        self.parse_search_response(&data)
    }

    /// Get movie details by TMDb ID
    ///
    /// # Arguments
    /// * `movie_id` - TMDb movie ID
    ///
    /// # Returns
    /// Movie details
    pub async fn get_movie_details(&self, movie_id: i32) -> Result<AggregatorContent> {
        let cache_key = format!("movie_details:{}", movie_id);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            return self.parse_movie_details(&cached.data);
        }

        let url = format!(
            "{}/movie/{}?api_key={}",
            self.base_url, movie_id, self.api_key
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
            source: "tmdb".to_string(),
        };

        // Cache the response
        self.cache.insert(cache_key, response_data).await;

        self.parse_movie_details(&data)
    }

    /// Get TV show details by TMDb ID
    ///
    /// # Arguments
    /// * `tv_id` - TMDb TV show ID
    ///
    /// # Returns
    /// TV show details
    pub async fn get_tv_details(&self, tv_id: i32) -> Result<AggregatorContent> {
        let cache_key = format!("tv_details:{}", tv_id);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            return self.parse_tv_details(&cached.data);
        }

        let url = format!("{}/tv/{}?api_key={}", self.base_url, tv_id, self.api_key);

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
            source: "tmdb".to_string(),
        };

        // Cache the response
        self.cache.insert(cache_key, response_data).await;

        self.parse_tv_details(&data)
    }

    /// Get external IDs for a movie
    ///
    /// # Arguments
    /// * `movie_id` - TMDb movie ID
    ///
    /// # Returns
    /// External IDs (IMDb, etc.)
    pub async fn get_movie_external_ids(&self, movie_id: i32) -> Result<Value> {
        let cache_key = format!("movie_external_ids:{}", movie_id);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(cached.data);
        }

        let url = format!(
            "{}/movie/{}/external_ids?api_key={}",
            self.base_url, movie_id, self.api_key
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
            source: "tmdb".to_string(),
        };

        // Cache the response
        self.cache.insert(cache_key, response_data).await;

        Ok(data)
    }

    /// Parse search response
    fn parse_search_response(&self, data: &Value) -> Result<Vec<AggregatorContent>> {
        let results = data
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                IngestionError::NormalizationFailed("No results array in response".to_string())
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
            .get("title")
            .or_else(|| item.get("name"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| IngestionError::NormalizationFailed("Missing title".to_string()))?
            .to_string();

        // Extract year from release_date or first_air_date
        let year = item
            .get("release_date")
            .or_else(|| item.get("first_air_date"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.split('-').next())
            .and_then(|y| y.parse::<i32>().ok());

        Ok(AggregatorContent {
            id: id.clone(),
            title,
            overview: item
                .get("overview")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            year,
            content_type: if item.get("title").is_some() {
                "movie".to_string()
            } else {
                "series".to_string()
            },
            imdb_id: None, // Need to fetch external IDs separately
            tmdb_id: Some(id.parse().unwrap_or(0)),
            genres: item
                .get("genre_ids")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|g| g.as_i64().map(|i| i.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            poster_url: item
                .get("poster_path")
                .and_then(|v| v.as_str())
                .map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
            rating: item
                .get("vote_average")
                .and_then(|v| v.as_f64())
                .map(|r| r as f32),
            runtime: None, // Need to fetch details for runtime
        })
    }

    /// Parse movie details
    fn parse_movie_details(&self, data: &Value) -> Result<AggregatorContent> {
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

        let year = data
            .get("release_date")
            .and_then(|v| v.as_str())
            .and_then(|s| s.split('-').next())
            .and_then(|y| y.parse::<i32>().ok());

        Ok(AggregatorContent {
            id: id.clone(),
            title,
            overview: data
                .get("overview")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            year,
            content_type: "movie".to_string(),
            imdb_id: data
                .get("imdb_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            tmdb_id: Some(id.parse().unwrap_or(0)),
            genres: data
                .get("genres")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|g| {
                            g.get("name")
                                .and_then(|n| n.as_str())
                                .map(|s| s.to_string())
                        })
                        .collect()
                })
                .unwrap_or_default(),
            poster_url: data
                .get("poster_path")
                .and_then(|v| v.as_str())
                .map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
            rating: data
                .get("vote_average")
                .and_then(|v| v.as_f64())
                .map(|r| r as f32),
            runtime: data
                .get("runtime")
                .and_then(|v| v.as_i64())
                .map(|r| r as i32),
        })
    }

    /// Parse TV details
    fn parse_tv_details(&self, data: &Value) -> Result<AggregatorContent> {
        let id = data
            .get("id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| IngestionError::NormalizationFailed("Missing id".to_string()))?
            .to_string();

        let title = data
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IngestionError::NormalizationFailed("Missing name".to_string()))?
            .to_string();

        let year = data
            .get("first_air_date")
            .and_then(|v| v.as_str())
            .and_then(|s| s.split('-').next())
            .and_then(|y| y.parse::<i32>().ok());

        Ok(AggregatorContent {
            id: id.clone(),
            title,
            overview: data
                .get("overview")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            year,
            content_type: "series".to_string(),
            imdb_id: None, // Need external_ids endpoint
            tmdb_id: Some(id.parse().unwrap_or(0)),
            genres: data
                .get("genres")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|g| {
                            g.get("name")
                                .and_then(|n| n.as_str())
                                .map(|s| s.to_string())
                        })
                        .collect()
                })
                .unwrap_or_default(),
            poster_url: data
                .get("poster_path")
                .and_then(|v| v.as_str())
                .map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
            rating: data
                .get("vote_average")
                .and_then(|v| v.as_f64())
                .map(|r| r as f32),
            runtime: data
                .get("episode_run_time")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
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
        let client = TMDbClient::new("test_key".to_string());
        assert_eq!(client.base_url, "https://api.themoviedb.org/3");
    }
}
