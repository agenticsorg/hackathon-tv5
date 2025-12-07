//! Aggregator API clients for content metadata

use crate::{IngestionError, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod streaming_availability;
pub mod tmdb;
pub mod watchmode;

pub use streaming_availability::StreamingAvailabilityClient;
pub use tmdb::TMDbClient;
pub use watchmode::WatchmodeClient;

/// Common response structure for aggregator APIs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatorResponse {
    /// Response data
    pub data: Value,
    /// Response timestamp
    pub fetched_at: DateTime<Utc>,
    /// Source API
    pub source: String,
}

/// Content metadata from aggregator APIs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatorContent {
    /// Content ID from aggregator
    pub id: String,
    /// Content title
    pub title: String,
    /// Content overview/description
    pub overview: Option<String>,
    /// Release year
    pub year: Option<i32>,
    /// Content type (movie, series, etc.)
    pub content_type: String,
    /// IMDb ID
    pub imdb_id: Option<String>,
    /// TMDb ID
    pub tmdb_id: Option<i32>,
    /// Genres
    pub genres: Vec<String>,
    /// Poster image URL
    pub poster_url: Option<String>,
    /// Average rating
    pub rating: Option<f32>,
    /// Runtime in minutes
    pub runtime: Option<i32>,
}
