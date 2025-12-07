//! Deep linking support for platform-specific playback URLs
//!
//! Provides deep link generation for major streaming platforms with web fallback
//! and device capability detection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during deep link generation
#[derive(Debug, Error)]
pub enum DeepLinkError {
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("Invalid content ID format: {0}")]
    InvalidContentId(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),
}

/// Supported streaming platforms for deep linking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Netflix,
    Spotify,
    AppleMusic,
    Hulu,
    DisneyPlus,
    HboMax,
    PrimeVideo,
}

impl Platform {
    /// Get the platform name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Netflix => "netflix",
            Platform::Spotify => "spotify",
            Platform::AppleMusic => "applemusic",
            Platform::Hulu => "hulu",
            Platform::DisneyPlus => "disneyplus",
            Platform::HboMax => "hbomax",
            Platform::PrimeVideo => "primevideo",
        }
    }
}

/// Content type for platform-specific deep links
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    /// Video content (movies, TV shows)
    Video,
    /// Music track
    Track,
    /// Music album
    Album,
    /// Playlist
    Playlist,
}

/// Device capabilities for deep link support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// Operating system (ios, android, web, etc.)
    pub os: String,
    /// OS version
    pub os_version: Option<String>,
    /// Installed apps that support deep linking
    pub installed_apps: Vec<String>,
    /// Browser support for universal links
    pub supports_universal_links: bool,
}

impl DeviceCapabilities {
    /// Create a new DeviceCapabilities instance
    pub fn new(os: String) -> Self {
        Self {
            os,
            os_version: None,
            installed_apps: Vec::new(),
            supports_universal_links: false,
        }
    }

    /// Check if a platform app is installed
    pub fn has_platform_app(&self, platform: Platform) -> bool {
        let app_name = platform.as_str();
        self.installed_apps
            .iter()
            .any(|app| app.to_lowercase().contains(app_name))
    }

    /// Check if the device supports deep linking for the platform
    pub fn supports_deep_link(&self, platform: Platform) -> bool {
        // iOS and Android typically support deep links if app is installed
        if self.os.to_lowercase() == "ios" || self.os.to_lowercase() == "android" {
            return self.has_platform_app(platform);
        }

        // Web browsers may support universal links
        if self.os.to_lowercase() == "web" {
            return self.supports_universal_links;
        }

        false
    }
}

/// Deep link generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLinkRequest {
    /// Platform to generate deep link for
    pub platform: Platform,
    /// Content type
    pub content_type: ContentType,
    /// Platform-specific content ID
    pub content_id: String,
    /// Optional start position in seconds
    pub start_position: Option<u32>,
    /// Device capabilities for optimization
    pub device_capabilities: Option<DeviceCapabilities>,
}

/// Generated deep link with fallback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLink {
    /// Platform
    pub platform: Platform,
    /// Primary deep link URL (app-specific)
    pub deep_link_url: String,
    /// Web fallback URL
    pub web_fallback_url: String,
    /// Universal link (if supported)
    pub universal_link: Option<String>,
    /// Whether deep link is supported on device
    pub is_supported: bool,
}

/// Deep link generator service
pub struct DeepLinkGenerator {
    /// Platform-specific URL templates
    url_templates: HashMap<Platform, PlatformUrlTemplates>,
}

/// URL templates for a platform
struct PlatformUrlTemplates {
    /// Deep link scheme and template
    deep_link_template: String,
    /// Web fallback template
    web_template: String,
    /// Universal link template (optional)
    universal_template: Option<String>,
}

impl DeepLinkGenerator {
    /// Create a new deep link generator with default templates
    pub fn new() -> Self {
        let mut url_templates = HashMap::new();

        // Netflix
        url_templates.insert(
            Platform::Netflix,
            PlatformUrlTemplates {
                deep_link_template: "netflix://title/{id}".to_string(),
                web_template: "https://www.netflix.com/watch/{id}".to_string(),
                universal_template: Some("https://www.netflix.com/watch/{id}".to_string()),
            },
        );

        // Spotify
        url_templates.insert(
            Platform::Spotify,
            PlatformUrlTemplates {
                deep_link_template: "spotify://{type}/{id}".to_string(),
                web_template: "https://open.spotify.com/{type}/{id}".to_string(),
                universal_template: Some("https://open.spotify.com/{type}/{id}".to_string()),
            },
        );

        // Apple Music
        url_templates.insert(
            Platform::AppleMusic,
            PlatformUrlTemplates {
                deep_link_template: "music://itunes.apple.com/{country}/{type}/{id}".to_string(),
                web_template: "https://music.apple.com/{country}/{type}/{id}".to_string(),
                universal_template: Some(
                    "https://music.apple.com/{country}/{type}/{id}".to_string(),
                ),
            },
        );

        // Hulu
        url_templates.insert(
            Platform::Hulu,
            PlatformUrlTemplates {
                deep_link_template: "hulu://watch/{id}".to_string(),
                web_template: "https://www.hulu.com/watch/{id}".to_string(),
                universal_template: Some("https://www.hulu.com/watch/{id}".to_string()),
            },
        );

        // Disney+
        url_templates.insert(
            Platform::DisneyPlus,
            PlatformUrlTemplates {
                deep_link_template: "disneyplus://content/{id}".to_string(),
                web_template: "https://www.disneyplus.com/video/{id}".to_string(),
                universal_template: Some("https://www.disneyplus.com/video/{id}".to_string()),
            },
        );

        // HBO Max
        url_templates.insert(
            Platform::HboMax,
            PlatformUrlTemplates {
                deep_link_template: "hbomax://content/{id}".to_string(),
                web_template: "https://www.max.com/video/{id}".to_string(),
                universal_template: Some("https://www.max.com/video/{id}".to_string()),
            },
        );

        // Prime Video
        url_templates.insert(
            Platform::PrimeVideo,
            PlatformUrlTemplates {
                deep_link_template: "primevideo://detail?id={id}".to_string(),
                web_template: "https://www.amazon.com/gp/video/detail/{id}".to_string(),
                universal_template: Some("https://www.amazon.com/gp/video/detail/{id}".to_string()),
            },
        );

        Self { url_templates }
    }

    /// Generate a deep link for the given request
    pub fn generate(&self, request: &DeepLinkRequest) -> Result<DeepLink, DeepLinkError> {
        let templates = self
            .url_templates
            .get(&request.platform)
            .ok_or_else(|| DeepLinkError::UnsupportedPlatform(format!("{:?}", request.platform)))?;

        // Check device support
        let is_supported = request
            .device_capabilities
            .as_ref()
            .map(|caps| caps.supports_deep_link(request.platform))
            .unwrap_or(true); // Assume supported if no capabilities provided

        // Generate deep link URL
        let deep_link_url = self.build_url(&templates.deep_link_template, request)?;

        // Generate web fallback URL
        let web_fallback_url = self.build_url(&templates.web_template, request)?;

        // Generate universal link if available
        let universal_link = templates
            .universal_template
            .as_ref()
            .map(|template| self.build_url(template, request))
            .transpose()?;

        Ok(DeepLink {
            platform: request.platform,
            deep_link_url,
            web_fallback_url,
            universal_link,
            is_supported,
        })
    }

    /// Build a URL from a template
    fn build_url(
        &self,
        template: &str,
        request: &DeepLinkRequest,
    ) -> Result<String, DeepLinkError> {
        let mut url = template.to_string();

        // Replace content ID
        url = url.replace("{id}", &request.content_id);

        // Replace content type for platforms that need it
        let type_str = match request.content_type {
            ContentType::Video => "video",
            ContentType::Track => "track",
            ContentType::Album => "album",
            ContentType::Playlist => "playlist",
        };
        url = url.replace("{type}", type_str);

        // Replace country for Apple Music (default to US)
        url = url.replace("{country}", "us");

        // Add start position if provided and supported
        if let Some(position) = request.start_position {
            if url.contains('?') {
                url.push_str(&format!("&t={}", position));
            } else {
                url.push_str(&format!("?t={}", position));
            }
        }

        Ok(url)
    }

    /// Generate deep links for multiple platforms
    pub fn generate_all(
        &self,
        content_id: &str,
        content_type: ContentType,
        device_capabilities: Option<DeviceCapabilities>,
    ) -> HashMap<Platform, DeepLink> {
        let platforms = vec![
            Platform::Netflix,
            Platform::Spotify,
            Platform::AppleMusic,
            Platform::Hulu,
            Platform::DisneyPlus,
            Platform::HboMax,
            Platform::PrimeVideo,
        ];

        platforms
            .into_iter()
            .filter_map(|platform| {
                let request = DeepLinkRequest {
                    platform,
                    content_type,
                    content_id: content_id.to_string(),
                    start_position: None,
                    device_capabilities: device_capabilities.clone(),
                };

                self.generate(&request).ok().map(|link| (platform, link))
            })
            .collect()
    }
}

impl Default for DeepLinkGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_netflix_deep_link() {
        let generator = DeepLinkGenerator::new();
        let request = DeepLinkRequest {
            platform: Platform::Netflix,
            content_type: ContentType::Video,
            content_id: "80123456".to_string(),
            start_position: None,
            device_capabilities: None,
        };

        let result = generator.generate(&request).unwrap();
        assert_eq!(result.deep_link_url, "netflix://title/80123456");
        assert_eq!(
            result.web_fallback_url,
            "https://www.netflix.com/watch/80123456"
        );
        assert!(result.universal_link.is_some());
    }

    #[test]
    fn test_spotify_track_deep_link() {
        let generator = DeepLinkGenerator::new();
        let request = DeepLinkRequest {
            platform: Platform::Spotify,
            content_type: ContentType::Track,
            content_id: "3n3Ppam7vgaVa1iaRUc9Lp".to_string(),
            start_position: None,
            device_capabilities: None,
        };

        let result = generator.generate(&request).unwrap();
        assert_eq!(
            result.deep_link_url,
            "spotify://track/3n3Ppam7vgaVa1iaRUc9Lp"
        );
        assert_eq!(
            result.web_fallback_url,
            "https://open.spotify.com/track/3n3Ppam7vgaVa1iaRUc9Lp"
        );
    }

    #[test]
    fn test_spotify_album_deep_link() {
        let generator = DeepLinkGenerator::new();
        let request = DeepLinkRequest {
            platform: Platform::Spotify,
            content_type: ContentType::Album,
            content_id: "6DEjYFkNZh67HP7R9PSZvv".to_string(),
            start_position: None,
            device_capabilities: None,
        };

        let result = generator.generate(&request).unwrap();
        assert_eq!(
            result.deep_link_url,
            "spotify://album/6DEjYFkNZh67HP7R9PSZvv"
        );
        assert_eq!(
            result.web_fallback_url,
            "https://open.spotify.com/album/6DEjYFkNZh67HP7R9PSZvv"
        );
    }

    #[test]
    fn test_apple_music_deep_link() {
        let generator = DeepLinkGenerator::new();
        let request = DeepLinkRequest {
            platform: Platform::AppleMusic,
            content_type: ContentType::Track,
            content_id: "1234567890".to_string(),
            start_position: None,
            device_capabilities: None,
        };

        let result = generator.generate(&request).unwrap();
        assert_eq!(
            result.deep_link_url,
            "music://itunes.apple.com/us/track/1234567890"
        );
        assert_eq!(
            result.web_fallback_url,
            "https://music.apple.com/us/track/1234567890"
        );
    }

    #[test]
    fn test_hulu_deep_link() {
        let generator = DeepLinkGenerator::new();
        let request = DeepLinkRequest {
            platform: Platform::Hulu,
            content_type: ContentType::Video,
            content_id: "abc123".to_string(),
            start_position: None,
            device_capabilities: None,
        };

        let result = generator.generate(&request).unwrap();
        assert_eq!(result.deep_link_url, "hulu://watch/abc123");
        assert_eq!(result.web_fallback_url, "https://www.hulu.com/watch/abc123");
    }

    #[test]
    fn test_disney_plus_deep_link() {
        let generator = DeepLinkGenerator::new();
        let request = DeepLinkRequest {
            platform: Platform::DisneyPlus,
            content_type: ContentType::Video,
            content_id: "xyz789".to_string(),
            start_position: None,
            device_capabilities: None,
        };

        let result = generator.generate(&request).unwrap();
        assert_eq!(result.deep_link_url, "disneyplus://content/xyz789");
        assert_eq!(
            result.web_fallback_url,
            "https://www.disneyplus.com/video/xyz789"
        );
    }

    #[test]
    fn test_hbo_max_deep_link() {
        let generator = DeepLinkGenerator::new();
        let request = DeepLinkRequest {
            platform: Platform::HboMax,
            content_type: ContentType::Video,
            content_id: "def456".to_string(),
            start_position: None,
            device_capabilities: None,
        };

        let result = generator.generate(&request).unwrap();
        assert_eq!(result.deep_link_url, "hbomax://content/def456");
        assert_eq!(result.web_fallback_url, "https://www.max.com/video/def456");
    }

    #[test]
    fn test_prime_video_deep_link() {
        let generator = DeepLinkGenerator::new();
        let request = DeepLinkRequest {
            platform: Platform::PrimeVideo,
            content_type: ContentType::Video,
            content_id: "B08XYZ".to_string(),
            start_position: None,
            device_capabilities: None,
        };

        let result = generator.generate(&request).unwrap();
        assert_eq!(result.deep_link_url, "primevideo://detail?id=B08XYZ");
        assert_eq!(
            result.web_fallback_url,
            "https://www.amazon.com/gp/video/detail/B08XYZ"
        );
    }

    #[test]
    fn test_deep_link_with_start_position() {
        let generator = DeepLinkGenerator::new();
        let request = DeepLinkRequest {
            platform: Platform::Netflix,
            content_type: ContentType::Video,
            content_id: "80123456".to_string(),
            start_position: Some(300), // 5 minutes
            device_capabilities: None,
        };

        let result = generator.generate(&request).unwrap();
        assert!(result.deep_link_url.contains("?t=300"));
        assert!(result.web_fallback_url.contains("?t=300"));
    }

    #[test]
    fn test_device_capabilities_detection() {
        let mut capabilities = DeviceCapabilities::new("ios".to_string());
        capabilities.installed_apps = vec!["Netflix".to_string(), "Spotify".to_string()];

        assert!(capabilities.has_platform_app(Platform::Netflix));
        assert!(capabilities.has_platform_app(Platform::Spotify));
        assert!(!capabilities.has_platform_app(Platform::Hulu));

        assert!(capabilities.supports_deep_link(Platform::Netflix));
        assert!(!capabilities.supports_deep_link(Platform::Hulu));
    }

    #[test]
    fn test_web_device_universal_links() {
        let mut capabilities = DeviceCapabilities::new("web".to_string());
        capabilities.supports_universal_links = true;

        assert!(capabilities.supports_deep_link(Platform::Netflix));

        capabilities.supports_universal_links = false;
        assert!(!capabilities.supports_deep_link(Platform::Netflix));
    }

    #[test]
    fn test_generate_all_platforms() {
        let generator = DeepLinkGenerator::new();
        let links = generator.generate_all("test123", ContentType::Video, None);

        assert_eq!(links.len(), 7);
        assert!(links.contains_key(&Platform::Netflix));
        assert!(links.contains_key(&Platform::Spotify));
        assert!(links.contains_key(&Platform::AppleMusic));
        assert!(links.contains_key(&Platform::Hulu));
        assert!(links.contains_key(&Platform::DisneyPlus));
        assert!(links.contains_key(&Platform::HboMax));
        assert!(links.contains_key(&Platform::PrimeVideo));
    }

    #[test]
    fn test_platform_as_str() {
        assert_eq!(Platform::Netflix.as_str(), "netflix");
        assert_eq!(Platform::Spotify.as_str(), "spotify");
        assert_eq!(Platform::AppleMusic.as_str(), "applemusic");
        assert_eq!(Platform::Hulu.as_str(), "hulu");
        assert_eq!(Platform::DisneyPlus.as_str(), "disneyplus");
        assert_eq!(Platform::HboMax.as_str(), "hbomax");
        assert_eq!(Platform::PrimeVideo.as_str(), "primevideo");
    }
}
