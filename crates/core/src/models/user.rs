//! User models for the Media Gateway platform
//!
//! This module contains data structures for user profiles, preferences,
//! devices, and privacy settings.

use crate::types::{AudioQuality, Genre, MaturityRating, Platform, Region, VideoQuality};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use validator::Validate;

/// User device information
///
/// Represents a device registered to a user account for content playback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct Device {
    /// Unique device identifier
    pub device_id: Uuid,

    /// User-friendly device name
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    /// Device type (e.g., "smart_tv", "mobile", "tablet", "desktop", "streaming_device")
    #[validate(length(min = 1, max = 50))]
    pub device_type: String,

    /// Operating system
    #[validate(length(min = 1, max = 100))]
    pub os: Option<String>,

    /// Device model
    #[validate(length(min = 1, max = 100))]
    pub model: Option<String>,

    /// Maximum supported video quality
    pub max_video_quality: VideoQuality,

    /// Maximum supported audio quality
    pub max_audio_quality: AudioQuality,

    /// When this device was registered
    pub registered_at: DateTime<Utc>,

    /// When this device was last used
    pub last_used_at: Option<DateTime<Utc>>,

    /// Whether this device is currently active
    pub is_active: bool,
}

/// Privacy settings for user account
///
/// Controls visibility and data sharing preferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct PrivacySettings {
    /// Whether user's watch history should be tracked
    pub track_watch_history: bool,

    /// Whether to allow personalized recommendations
    pub allow_personalized_recommendations: bool,

    /// Whether to allow data sharing with third parties
    pub allow_data_sharing: bool,

    /// Whether profile is visible to other users
    pub profile_visibility: ProfileVisibility,

    /// Whether to show adult content
    pub show_adult_content: bool,

    /// Whether to collect analytics data
    pub collect_analytics: bool,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Profile visibility options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileVisibility {
    /// Profile visible to all users
    Public,
    /// Profile visible only to friends
    FriendsOnly,
    /// Profile not visible to anyone
    Private,
}

/// User preferences for content discovery and playback
///
/// Stores user-specific settings and preferences for customized experience.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct UserPreferences {
    /// Preferred content languages (ISO 639-1 codes)
    pub preferred_languages: Vec<String>,

    /// Preferred subtitle language (ISO 639-1 code)
    #[validate(length(equal = 2))]
    pub preferred_subtitle_language: Option<String>,

    /// Preferred audio language (ISO 639-1 code)
    #[validate(length(equal = 2))]
    pub preferred_audio_language: Option<String>,

    /// Preferred video quality
    pub preferred_video_quality: VideoQuality,

    /// Preferred audio quality
    pub preferred_audio_quality: AudioQuality,

    /// Whether to auto-play next episode
    pub autoplay_next_episode: bool,

    /// Whether to show subtitles by default
    pub subtitles_enabled: bool,

    /// Favorite genres
    pub favorite_genres: Vec<Genre>,

    /// Genres to exclude from recommendations
    pub excluded_genres: Vec<Genre>,

    /// Maximum maturity rating to show
    pub max_maturity_rating: Option<MaturityRating>,

    /// Preferred streaming platforms
    pub preferred_platforms: Vec<Platform>,

    /// Notification preferences by type
    pub notification_preferences: HashMap<NotificationType, bool>,

    /// Content type preferences (e.g., prefer movies over series)
    pub content_type_weights: HashMap<String, f32>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Types of notifications users can receive
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// New content available from followed actors/directors
    NewReleases,
    /// Price drops for wishlisted content
    PriceDrops,
    /// Content leaving platforms soon
    ExpiringContent,
    /// New episodes of followed series
    NewEpisodes,
    /// Recommendations based on watch history
    Recommendations,
    /// Platform subscription reminders
    Subscriptions,
    /// Friend activity (if social features enabled)
    SocialActivity,
}

/// Subscription information for a platform
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct PlatformSubscription {
    /// The platform being subscribed to
    pub platform: Platform,

    /// Subscription tier/plan name
    #[validate(length(min = 1, max = 100))]
    pub tier: String,

    /// When subscription started
    pub started_at: DateTime<Utc>,

    /// When subscription renews (if recurring)
    pub renews_at: Option<DateTime<Utc>>,

    /// When subscription ends (if not recurring)
    pub ends_at: Option<DateTime<Utc>>,

    /// Whether subscription auto-renews
    pub auto_renew: bool,

    /// Monthly cost in USD cents
    #[validate(range(min = 0))]
    pub monthly_cost_cents: i32,

    /// Whether subscription is currently active
    pub is_active: bool,
}

/// Watch history entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct WatchHistoryEntry {
    /// Content canonical ID
    pub content_id: Uuid,

    /// Platform where content was watched
    pub platform: Platform,

    /// Device used for watching
    pub device_id: Uuid,

    /// When watching started
    pub started_at: DateTime<Utc>,

    /// When watching ended (if completed)
    pub ended_at: Option<DateTime<Utc>>,

    /// Progress in seconds
    #[validate(range(min = 0))]
    pub progress_seconds: i32,

    /// Total duration in seconds
    #[validate(range(min = 1))]
    pub duration_seconds: i32,

    /// Whether content was fully watched
    pub completed: bool,
}

/// Watchlist entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct WatchlistEntry {
    /// Content canonical ID
    pub content_id: Uuid,

    /// When content was added to watchlist
    pub added_at: DateTime<Utc>,

    /// Optional priority/ordering
    pub priority: Option<i32>,

    /// Optional notes from user
    #[validate(length(max = 500))]
    pub notes: Option<String>,
}

/// User rating for content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct UserRating {
    /// Content canonical ID
    pub content_id: Uuid,

    /// Rating value (0.0 - 10.0)
    #[validate(range(min = 0.0, max = 10.0))]
    pub rating: f32,

    /// When rating was given
    pub rated_at: DateTime<Utc>,

    /// Optional review text
    #[validate(length(max = 2000))]
    pub review: Option<String>,
}

/// User profile
///
/// The primary data structure representing a user in the Media Gateway platform.
/// Contains personal information, preferences, subscriptions, and activity data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct UserProfile {
    /// Unique user identifier
    pub user_id: Uuid,

    /// User's email address
    #[validate(email)]
    pub email: String,

    /// User's display name
    #[validate(length(min = 1, max = 100))]
    pub display_name: String,

    /// User's home region (ISO 3166-1 alpha-2)
    pub home_region: Region,

    /// User preferences
    #[validate]
    pub preferences: UserPreferences,

    /// Privacy settings
    #[validate]
    pub privacy: PrivacySettings,

    /// Active platform subscriptions
    #[validate]
    pub subscriptions: Vec<PlatformSubscription>,

    /// Registered devices
    #[validate]
    pub devices: Vec<Device>,

    /// Watch history (limited to recent entries for performance)
    #[validate]
    pub watch_history: Vec<WatchHistoryEntry>,

    /// Watchlist
    #[validate]
    pub watchlist: Vec<WatchlistEntry>,

    /// User ratings and reviews
    #[validate]
    pub ratings: Vec<UserRating>,

    /// Followed content creators (actor/director IDs)
    pub followed_creators: HashSet<String>,

    /// Followed series (canonical content IDs)
    pub followed_series: HashSet<Uuid>,

    /// Account creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last login timestamp
    pub last_login_at: Option<DateTime<Utc>>,

    /// Last activity timestamp
    pub last_activity_at: DateTime<Utc>,

    /// Whether account is active
    pub is_active: bool,

    /// Whether account is verified
    pub is_verified: bool,
}

impl UserProfile {
    /// Create a new user profile with default values
    pub fn new(email: String, display_name: String, home_region: Region) -> Self {
        let now = Utc::now();
        Self {
            user_id: Uuid::new_v4(),
            email,
            display_name,
            home_region,
            preferences: UserPreferences {
                preferred_languages: vec!["en".to_string()],
                preferred_subtitle_language: None,
                preferred_audio_language: None,
                preferred_video_quality: VideoQuality::HD,
                preferred_audio_quality: AudioQuality::Stereo,
                autoplay_next_episode: true,
                subtitles_enabled: false,
                favorite_genres: Vec::new(),
                excluded_genres: Vec::new(),
                max_maturity_rating: None,
                preferred_platforms: Vec::new(),
                notification_preferences: HashMap::new(),
                content_type_weights: HashMap::new(),
                updated_at: now,
            },
            privacy: PrivacySettings {
                track_watch_history: true,
                allow_personalized_recommendations: true,
                allow_data_sharing: false,
                profile_visibility: ProfileVisibility::Private,
                show_adult_content: false,
                collect_analytics: true,
                updated_at: now,
            },
            subscriptions: Vec::new(),
            devices: Vec::new(),
            watch_history: Vec::new(),
            watchlist: Vec::new(),
            ratings: Vec::new(),
            followed_creators: HashSet::new(),
            followed_series: HashSet::new(),
            created_at: now,
            last_login_at: None,
            last_activity_at: now,
            is_active: true,
            is_verified: false,
        }
    }

    /// Check if user has an active subscription to a platform
    pub fn has_active_subscription(&self, platform: Platform) -> bool {
        self.subscriptions
            .iter()
            .any(|sub| sub.platform == platform && sub.is_active)
    }

    /// Get active device by ID
    pub fn get_device(&self, device_id: Uuid) -> Option<&Device> {
        self.devices.iter().find(|d| d.device_id == device_id)
    }

    /// Add content to watchlist
    pub fn add_to_watchlist(&mut self, content_id: Uuid) {
        if !self.is_in_watchlist(content_id) {
            self.watchlist.push(WatchlistEntry {
                content_id,
                added_at: Utc::now(),
                priority: None,
                notes: None,
            });
        }
    }

    /// Check if content is in watchlist
    pub fn is_in_watchlist(&self, content_id: Uuid) -> bool {
        self.watchlist
            .iter()
            .any(|entry| entry.content_id == content_id)
    }

    /// Remove content from watchlist
    pub fn remove_from_watchlist(&mut self, content_id: Uuid) {
        self.watchlist
            .retain(|entry| entry.content_id != content_id);
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity_at = Utc::now();
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_languages: vec!["en".to_string()],
            preferred_subtitle_language: None,
            preferred_audio_language: None,
            preferred_video_quality: VideoQuality::HD,
            preferred_audio_quality: AudioQuality::Stereo,
            autoplay_next_episode: true,
            subtitles_enabled: false,
            favorite_genres: Vec::new(),
            excluded_genres: Vec::new(),
            max_maturity_rating: None,
            preferred_platforms: Vec::new(),
            notification_preferences: HashMap::new(),
            content_type_weights: HashMap::new(),
            updated_at: Utc::now(),
        }
    }
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            track_watch_history: true,
            allow_personalized_recommendations: true,
            allow_data_sharing: false,
            profile_visibility: ProfileVisibility::Private,
            show_adult_content: false,
            collect_analytics: true,
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_profile_creation() {
        let user = UserProfile::new(
            "test@example.com".to_string(),
            "Test User".to_string(),
            "US".to_string(),
        );

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.display_name, "Test User");
        assert_eq!(user.home_region, "US");
        assert!(user.is_active);
        assert!(!user.is_verified);
    }

    #[test]
    fn test_watchlist_operations() {
        let mut user = UserProfile::new(
            "test@example.com".to_string(),
            "Test User".to_string(),
            "US".to_string(),
        );

        let content_id = Uuid::new_v4();

        assert!(!user.is_in_watchlist(content_id));

        user.add_to_watchlist(content_id);
        assert!(user.is_in_watchlist(content_id));
        assert_eq!(user.watchlist.len(), 1);

        // Adding again should not duplicate
        user.add_to_watchlist(content_id);
        assert_eq!(user.watchlist.len(), 1);

        user.remove_from_watchlist(content_id);
        assert!(!user.is_in_watchlist(content_id));
        assert_eq!(user.watchlist.len(), 0);
    }

    #[test]
    fn test_subscription_check() {
        let mut user = UserProfile::new(
            "test@example.com".to_string(),
            "Test User".to_string(),
            "US".to_string(),
        );

        assert!(!user.has_active_subscription(Platform::Netflix));

        user.subscriptions.push(PlatformSubscription {
            platform: Platform::Netflix,
            tier: "Premium".to_string(),
            started_at: Utc::now(),
            renews_at: None,
            ends_at: None,
            auto_renew: true,
            monthly_cost_cents: 1999,
            is_active: true,
        });

        assert!(user.has_active_subscription(Platform::Netflix));
        assert!(!user.has_active_subscription(Platform::PrimeVideo));
    }

    #[test]
    fn test_touch_updates_activity() {
        let mut user = UserProfile::new(
            "test@example.com".to_string(),
            "Test User".to_string(),
            "US".to_string(),
        );

        let original_time = user.last_activity_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        user.touch();

        assert!(user.last_activity_at > original_time);
    }
}
