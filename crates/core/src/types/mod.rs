//! Core type definitions for the Media Gateway platform
//!
//! This module contains fundamental enums and type aliases used throughout
//! the platform for content classification, platform identification, and
//! availability management.

use serde::{Deserialize, Serialize};

/// Content type classification
///
/// Represents the different types of media content supported by the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    /// Feature-length movie
    Movie,
    /// Multi-episode series
    Series,
    /// Individual episode within a series
    Episode,
    /// Short-form content (typically < 30 minutes)
    Short,
    /// Documentary content
    Documentary,
    /// Special one-off content (e.g., holiday specials)
    Special,
}

/// Streaming platform identifiers
///
/// Represents the major streaming platforms supported by Media Gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Netflix,
    PrimeVideo,
    DisneyPlus,
    Hulu,
    AppleTVPlus,
    HBOMax,
    Peacock,
    ParamountPlus,
    YouTube,
    Crave,
    BBCiPlayer,
}

/// Content availability type
///
/// Defines how content can be accessed on a given platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityType {
    /// Included with subscription
    Subscription,
    /// Available for temporary rental
    Rental,
    /// Available for permanent purchase
    Purchase,
    /// Available at no cost
    Free,
}

/// Content genre classification
///
/// Comprehensive genre taxonomy following industry standards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Genre {
    Action,
    Adventure,
    Animation,
    Comedy,
    Crime,
    Documentary,
    Drama,
    Family,
    Fantasy,
    Horror,
    Mystery,
    Romance,
    SciFi,
    Thriller,
    Western,
    Musical,
    War,
    Biography,
    History,
    Sport,
    GameShow,
    RealityTV,
    TalkShow,
    News,
}

/// Geographic region identifier
///
/// Uses ISO 3166-1 alpha-2 country codes (e.g., "US", "CA", "GB").
/// Maximum length: 2 characters
pub type Region = String;

/// Video quality levels
///
/// Represents the available video quality options for content playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum VideoQuality {
    /// Standard Definition (480p)
    SD,
    /// High Definition (720p/1080p)
    HD,
    /// Ultra High Definition (4K)
    UHD,
    /// High Dynamic Range (any resolution with HDR)
    HDR,
}

/// Audio quality levels
///
/// Represents the available audio quality options for content playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioQuality {
    /// Stereo audio
    Stereo,
    /// 5.1 surround sound
    Surround51,
    /// 7.1 surround sound
    Surround71,
    /// Dolby Atmos
    Atmos,
    /// DTS:X
    DtsX,
}

/// Subtitle/caption format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleFormat {
    /// Closed captions
    ClosedCaptions,
    /// Subtitles for the deaf and hard of hearing
    SDH,
    /// Standard subtitles
    Standard,
}

/// Maturity rating classification
///
/// Content rating systems vary by region; this provides a normalized set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum MaturityRating {
    /// General audiences
    G,
    /// Parental guidance suggested
    PG,
    /// Parents strongly cautioned
    PG13,
    /// Restricted (under 17 requires guardian)
    R,
    /// No one 17 and under admitted
    NC17,
    /// Not rated
    NR,
    /// TV-Y (All children)
    TVY,
    /// TV-Y7 (Older children)
    TVY7,
    /// TV-G (General audience)
    TVG,
    /// TV-PG (Parental guidance)
    TVPG,
    /// TV-14 (Parents strongly cautioned)
    TV14,
    /// TV-MA (Mature audience only)
    TVMA,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_serialization() {
        let content_type = ContentType::Movie;
        let json = serde_json::to_string(&content_type).unwrap();
        assert_eq!(json, r#""movie""#);

        let deserialized: ContentType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, content_type);
    }

    #[test]
    fn test_platform_serialization() {
        let platform = Platform::Netflix;
        let json = serde_json::to_string(&platform).unwrap();
        assert_eq!(json, r#""netflix""#);
    }

    #[test]
    fn test_video_quality_ordering() {
        assert!(VideoQuality::HD > VideoQuality::SD);
        assert!(VideoQuality::UHD > VideoQuality::HD);
    }

    #[test]
    fn test_maturity_rating_ordering() {
        assert!(MaturityRating::PG > MaturityRating::G);
        assert!(MaturityRating::R > MaturityRating::PG13);
    }
}
