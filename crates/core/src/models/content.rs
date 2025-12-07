//! Content models for the Media Gateway platform
//!
//! This module contains the core data structures for representing media content,
//! including canonical content records, platform availability, and metadata.

use crate::types::{
    AudioQuality, AvailabilityType, ContentType, Genre, MaturityRating, Platform, Region,
    SubtitleFormat, VideoQuality,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// External identifier mappings for cross-platform content identification
///
/// Maps content to various external database and platform identifiers
/// for comprehensive content tracking and linking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct ExternalIds {
    /// Entertainment Identifier Registry (EIDR) ID
    #[validate(length(min = 1, max = 100))]
    pub eidr_id: Option<String>,

    /// Internet Movie Database (IMDb) ID
    #[validate(regex(path = "crate::validation::IMDB_ID_REGEX"))]
    pub imdb_id: Option<String>,

    /// The Movie Database (TMDb) ID
    pub tmdb_id: Option<i64>,

    /// TheTVDB ID
    pub tvdb_id: Option<i64>,

    /// Gracenote TMS ID
    #[validate(length(min = 1, max = 100))]
    pub gracenote_tms_id: Option<String>,

    /// Platform-specific identifiers (e.g., Netflix ID, Prime Video ID)
    pub platform_ids: HashMap<Platform, String>,
}

/// Platform availability information
///
/// Represents how and where content is available on a specific platform,
/// including pricing, quality options, and temporal availability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct PlatformAvailability {
    /// The platform where content is available
    pub platform: Platform,

    /// Type of availability (subscription, rental, purchase, free)
    pub availability_type: AvailabilityType,

    /// Price for rental or purchase (in USD cents, None for subscription/free)
    #[validate(range(min = 0))]
    pub price_cents: Option<i32>,

    /// Available video quality options
    #[validate(length(min = 1))]
    pub video_qualities: Vec<VideoQuality>,

    /// Available audio quality options
    #[validate(length(min = 1))]
    pub audio_qualities: Vec<AudioQuality>,

    /// Available subtitle languages (ISO 639-1 codes)
    pub subtitle_languages: Vec<String>,

    /// Subtitle format options
    pub subtitle_formats: Vec<SubtitleFormat>,

    /// Regions where this availability applies (ISO 3166-1 alpha-2)
    #[validate(length(min = 1))]
    pub regions: Vec<Region>,

    /// When this content became available on the platform
    pub available_from: Option<DateTime<Utc>>,

    /// When this content will no longer be available (if known)
    pub available_until: Option<DateTime<Utc>>,

    /// Direct URL to the content on the platform
    #[validate(url)]
    pub platform_url: Option<String>,
}

/// Series-specific metadata
///
/// Additional information for series and episode content types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct SeriesMetadata {
    /// Total number of seasons
    #[validate(range(min = 1))]
    pub total_seasons: i32,

    /// Total number of episodes across all seasons
    #[validate(range(min = 1))]
    pub total_episodes: i32,

    /// Season number (for episodes)
    #[validate(range(min = 1))]
    pub season_number: Option<i32>,

    /// Episode number within season (for episodes)
    #[validate(range(min = 1))]
    pub episode_number: Option<i32>,

    /// Parent series ID (for episodes)
    pub series_id: Option<Uuid>,

    /// Series status (ongoing, ended, cancelled)
    pub status: Option<SeriesStatus>,
}

/// Series production status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeriesStatus {
    Ongoing,
    Ended,
    Cancelled,
    Hiatus,
}

/// Content image assets
///
/// URLs and metadata for various image assets associated with content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct ContentImages {
    /// Primary poster image URL
    #[validate(url)]
    pub poster_url: Option<String>,

    /// Backdrop/banner image URL
    #[validate(url)]
    pub backdrop_url: Option<String>,

    /// Thumbnail image URL
    #[validate(url)]
    pub thumbnail_url: Option<String>,

    /// Logo image URL
    #[validate(url)]
    pub logo_url: Option<String>,

    /// Additional image URLs by type
    pub additional_images: HashMap<String, String>,
}

/// Content credits and cast information
///
/// Information about people involved in creating the content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct Credits {
    /// Directors
    pub directors: Vec<Person>,

    /// Writers
    pub writers: Vec<Person>,

    /// Main cast members
    pub cast: Vec<CastMember>,

    /// Producers
    pub producers: Vec<Person>,
}

/// Person involved in content creation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct Person {
    /// Person's name
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// External ID (e.g., IMDb person ID)
    pub external_id: Option<String>,
}

/// Cast member with role information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct CastMember {
    /// Person information
    #[validate]
    pub person: Person,

    /// Character name
    #[validate(length(min = 1, max = 255))]
    pub character: Option<String>,

    /// Billing order (lower numbers = higher billing)
    #[validate(range(min = 0))]
    pub order: Option<i32>,
}

/// Canonical content record
///
/// The primary data structure representing a piece of media content
/// in the Media Gateway platform. This structure aggregates information
/// from multiple sources and platforms into a single, normalized record.
///
/// Complexity targets:
/// - Lookup: O(1) via hash-based indexing on canonical_id
/// - Storage: ~20KB per content item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct CanonicalContent {
    /// Unique canonical identifier for this content
    pub canonical_id: Uuid,

    /// Content type (movie, series, episode, etc.)
    pub content_type: ContentType,

    /// Primary title
    #[validate(length(min = 1, max = 500))]
    pub title: String,

    /// Original title (in original language)
    #[validate(length(min = 1, max = 500))]
    pub original_title: Option<String>,

    /// Alternative titles and translations
    pub alternate_titles: HashMap<String, String>,

    /// Content description/synopsis
    #[validate(length(max = 5000))]
    pub description: Option<String>,

    /// Primary release year
    #[validate(range(min = 1800, max = 2100))]
    pub release_year: i32,

    /// Exact release date (if known)
    pub release_date: Option<DateTime<Utc>>,

    /// Runtime in minutes
    #[validate(range(min = 1))]
    pub runtime_minutes: Option<i32>,

    /// Content genres
    #[validate(length(min = 1))]
    pub genres: Vec<Genre>,

    /// Maturity rating
    pub maturity_rating: Option<MaturityRating>,

    /// External identifier mappings
    #[validate]
    pub external_ids: ExternalIds,

    /// Platform availability records
    #[validate]
    pub platform_availability: Vec<PlatformAvailability>,

    /// Series metadata (for series/episode content)
    #[validate]
    pub series_metadata: Option<SeriesMetadata>,

    /// Image assets
    #[validate]
    pub images: ContentImages,

    /// Credits and cast
    #[validate]
    pub credits: Credits,

    /// Average user rating (0.0 - 10.0)
    #[validate(range(min = 0.0, max = 10.0))]
    pub average_rating: Option<f32>,

    /// Number of user ratings
    #[validate(range(min = 0))]
    pub rating_count: Option<i32>,

    /// Popularity score (platform-specific calculation)
    #[validate(range(min = 0.0))]
    pub popularity_score: Option<f32>,

    /// Original language (ISO 639-1 code)
    #[validate(length(equal = 2))]
    pub original_language: Option<String>,

    /// Available audio languages (ISO 639-1 codes)
    pub audio_languages: Vec<String>,

    /// Production countries (ISO 3166-1 alpha-2)
    pub production_countries: Vec<Region>,

    /// Production companies
    pub production_companies: Vec<String>,

    /// Keywords and tags for content discovery
    pub keywords: Vec<String>,

    /// When this record was created
    pub created_at: DateTime<Utc>,

    /// When this record was last updated
    pub updated_at: DateTime<Utc>,

    /// Data quality score (0.0 - 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub data_quality_score: f32,

    /// Source platforms that contributed to this canonical record
    pub source_platforms: Vec<Platform>,
}

impl CanonicalContent {
    /// Create a new canonical content record with default values
    pub fn new(content_type: ContentType, title: String, release_year: i32) -> Self {
        let now = Utc::now();
        Self {
            canonical_id: Uuid::new_v4(),
            content_type,
            title,
            original_title: None,
            alternate_titles: HashMap::new(),
            description: None,
            release_year,
            release_date: None,
            runtime_minutes: None,
            genres: Vec::new(),
            maturity_rating: None,
            external_ids: ExternalIds {
                eidr_id: None,
                imdb_id: None,
                tmdb_id: None,
                tvdb_id: None,
                gracenote_tms_id: None,
                platform_ids: HashMap::new(),
            },
            platform_availability: Vec::new(),
            series_metadata: None,
            images: ContentImages {
                poster_url: None,
                backdrop_url: None,
                thumbnail_url: None,
                logo_url: None,
                additional_images: HashMap::new(),
            },
            credits: Credits {
                directors: Vec::new(),
                writers: Vec::new(),
                cast: Vec::new(),
                producers: Vec::new(),
            },
            average_rating: None,
            rating_count: None,
            popularity_score: None,
            original_language: None,
            audio_languages: Vec::new(),
            production_countries: Vec::new(),
            production_companies: Vec::new(),
            keywords: Vec::new(),
            created_at: now,
            updated_at: now,
            data_quality_score: 0.0,
            source_platforms: Vec::new(),
        }
    }

    /// Check if content is available on a specific platform
    pub fn is_available_on(&self, platform: Platform) -> bool {
        self.platform_availability
            .iter()
            .any(|avail| avail.platform == platform)
    }

    /// Get availability for a specific platform
    pub fn get_platform_availability(&self, platform: Platform) -> Option<&PlatformAvailability> {
        self.platform_availability
            .iter()
            .find(|avail| avail.platform == platform)
    }

    /// Check if content is available in a specific region
    pub fn is_available_in_region(&self, region: &str) -> bool {
        self.platform_availability
            .iter()
            .any(|avail| avail.regions.iter().any(|r| r.eq_ignore_ascii_case(region)))
    }

    /// Update the timestamp to reflect changes
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_content_creation() {
        let content = CanonicalContent::new(ContentType::Movie, "Test Movie".to_string(), 2024);

        assert_eq!(content.content_type, ContentType::Movie);
        assert_eq!(content.title, "Test Movie");
        assert_eq!(content.release_year, 2024);
        assert!(content.canonical_id.is_nil() == false);
    }

    #[test]
    fn test_platform_availability_check() {
        let mut content = CanonicalContent::new(ContentType::Movie, "Test Movie".to_string(), 2024);

        assert!(!content.is_available_on(Platform::Netflix));

        content.platform_availability.push(PlatformAvailability {
            platform: Platform::Netflix,
            availability_type: AvailabilityType::Subscription,
            price_cents: None,
            video_qualities: vec![VideoQuality::HD],
            audio_qualities: vec![AudioQuality::Stereo],
            subtitle_languages: vec!["en".to_string()],
            subtitle_formats: vec![SubtitleFormat::ClosedCaptions],
            regions: vec!["US".to_string()],
            available_from: None,
            available_until: None,
            platform_url: None,
        });

        assert!(content.is_available_on(Platform::Netflix));
        assert!(!content.is_available_on(Platform::PrimeVideo));
    }

    #[test]
    fn test_region_availability_check() {
        let mut content = CanonicalContent::new(ContentType::Movie, "Test Movie".to_string(), 2024);

        content.platform_availability.push(PlatformAvailability {
            platform: Platform::Netflix,
            availability_type: AvailabilityType::Subscription,
            price_cents: None,
            video_qualities: vec![VideoQuality::HD],
            audio_qualities: vec![AudioQuality::Stereo],
            subtitle_languages: vec![],
            subtitle_formats: vec![],
            regions: vec!["US".to_string(), "CA".to_string()],
            available_from: None,
            available_until: None,
            platform_url: None,
        });

        assert!(content.is_available_in_region("US"));
        assert!(content.is_available_in_region("ca")); // Case insensitive
        assert!(!content.is_available_in_region("GB"));
    }

    #[test]
    fn test_touch_updates_timestamp() {
        let mut content = CanonicalContent::new(ContentType::Movie, "Test Movie".to_string(), 2024);
        let original_time = content.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        content.touch();

        assert!(content.updated_at > original_time);
    }
}
