//! Platform normalizers for converting platform-specific data to canonical format

use crate::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod apple_tv_plus;
pub mod circuit_breaker_integration;
pub mod disney_plus;
pub mod generic;
pub mod hbo_max;
pub mod hulu;
pub mod netflix;
pub mod paramount_plus;
pub mod peacock;
pub mod prime_video;
pub mod youtube;

/// Raw content item from a platform API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawContent {
    /// Platform-specific ID
    pub id: String,
    /// Platform identifier
    pub platform: String,
    /// Raw JSON data from the platform
    pub data: serde_json::Value,
    /// Fetch timestamp
    pub fetched_at: DateTime<Utc>,
}

/// Canonical content representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalContent {
    /// Platform-specific ID
    pub platform_content_id: String,
    /// Platform identifier
    pub platform_id: String,
    /// Resolved entity ID (if matched)
    pub entity_id: Option<String>,
    /// Content title
    pub title: String,
    /// Content description/overview
    pub overview: Option<String>,
    /// Content type (movie, series, episode, etc.)
    pub content_type: ContentType,
    /// Release year
    pub release_year: Option<i32>,
    /// Runtime in minutes
    pub runtime_minutes: Option<i32>,
    /// Genres (canonical taxonomy)
    pub genres: Vec<String>,
    /// External IDs (IMDb, TMDb, EIDR, etc.)
    pub external_ids: HashMap<String, String>,
    /// Availability information
    pub availability: AvailabilityInfo,
    /// Image URLs
    pub images: ImageSet,
    /// Content rating (PG, R, etc.)
    pub rating: Option<String>,
    /// Average user rating (0-10 scale)
    pub user_rating: Option<f32>,
    /// Content embedding vector (768 dimensions)
    pub embedding: Option<Vec<f32>>,
    /// Metadata timestamp
    pub updated_at: DateTime<Utc>,
}

/// Content type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Movie,
    Series,
    Episode,
    Short,
    Documentary,
}

/// Availability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityInfo {
    /// Available regions (ISO 3166-1 alpha-2 codes)
    pub regions: Vec<String>,
    /// Subscription required
    pub subscription_required: bool,
    /// Purchase price (if available)
    pub purchase_price: Option<f32>,
    /// Rental price (if available)
    pub rental_price: Option<f32>,
    /// Currency code (ISO 4217)
    pub currency: Option<String>,
    /// Availability start date
    pub available_from: Option<DateTime<Utc>>,
    /// Availability end date (if expiring)
    pub available_until: Option<DateTime<Utc>>,
}

/// Image URLs for different sizes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageSet {
    pub poster_small: Option<String>,
    pub poster_medium: Option<String>,
    pub poster_large: Option<String>,
    pub backdrop: Option<String>,
}

/// Rate limit configuration for a platform
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per time window
    pub max_requests: u32,
    /// Time window duration
    pub window: std::time::Duration,
    /// API keys for rotation (if applicable)
    pub api_keys: Vec<String>,
}

/// Platform normalizer trait
///
/// Implementations convert platform-specific data formats to the canonical
/// Media Gateway format, handling API differences and data mapping.
#[async_trait]
pub trait PlatformNormalizer: Send + Sync {
    /// Get the platform identifier (e.g., "netflix", "prime_video")
    fn platform_id(&self) -> &'static str;

    /// Fetch catalog delta since a given timestamp
    ///
    /// # Arguments
    /// * `since` - Fetch content added/updated since this timestamp
    /// * `region` - ISO 3166-1 alpha-2 region code
    ///
    /// # Returns
    /// Vector of raw content items
    async fn fetch_catalog_delta(
        &self,
        since: DateTime<Utc>,
        region: &str,
    ) -> Result<Vec<RawContent>>;

    /// Normalize raw platform data to canonical format
    ///
    /// # Arguments
    /// * `raw` - Raw content from platform API
    ///
    /// # Returns
    /// Canonical content representation
    fn normalize(&self, raw: RawContent) -> Result<CanonicalContent>;

    /// Generate deep link for content
    ///
    /// # Arguments
    /// * `content_id` - Platform-specific content ID
    ///
    /// # Returns
    /// Deep link result with mobile and web URLs
    fn generate_deep_link(&self, content_id: &str) -> crate::deep_link::DeepLinkResult;

    /// Get rate limit configuration for this platform
    fn rate_limit_config(&self) -> RateLimitConfig;
}

/// Helper function to extract string from JSON value
pub(crate) fn extract_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(|s| s.to_string())
}

/// Helper function to extract i64 from JSON value
pub(crate) fn extract_i64(value: &serde_json::Value, key: &str) -> Option<i64> {
    value.get(key)?.as_i64()
}

/// Helper function to extract f64 from JSON value
pub(crate) fn extract_f64(value: &serde_json::Value, key: &str) -> Option<f64> {
    value.get(key)?.as_f64()
}

/// Helper function to extract array from JSON value
pub(crate) fn extract_array<'a>(
    value: &'a serde_json::Value,
    key: &str,
) -> Option<&'a Vec<serde_json::Value>> {
    value.get(key)?.as_array()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_serialization() {
        let content_type = ContentType::Movie;
        let json = serde_json::to_string(&content_type).unwrap();
        assert_eq!(json, r#""movie""#);
    }

    #[test]
    fn test_extract_helpers() {
        let json = serde_json::json!({
            "name": "Test Movie",
            "year": 2024,
            "rating": 8.5,
            "genres": ["action", "thriller"]
        });

        assert_eq!(
            extract_string(&json, "name"),
            Some("Test Movie".to_string())
        );
        assert_eq!(extract_i64(&json, "year"), Some(2024));
        assert_eq!(extract_f64(&json, "rating"), Some(8.5));
        assert!(extract_array(&json, "genres").is_some());
    }
}
