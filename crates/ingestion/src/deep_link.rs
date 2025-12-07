//! Deep link generation for platform content

use serde::{Deserialize, Serialize};

/// Deep link result with URLs for different platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLinkResult {
    /// Mobile app deep link (if available)
    pub mobile_url: Option<String>,
    /// Web URL (always available)
    pub web_url: String,
    /// TV app deep link (if available)
    pub tv_url: Option<String>,
}

/// Deep link generator for content across platforms
pub struct DeepLinkGenerator;

impl DeepLinkGenerator {
    /// Create a new deep link generator
    pub fn new() -> Self {
        Self
    }

    /// Generate deep links for content
    ///
    /// # Arguments
    /// * `platform_id` - Platform identifier (e.g., "netflix", "prime_video")
    /// * `content_id` - Platform-specific content ID
    ///
    /// # Returns
    /// Deep link result with mobile, web, and TV URLs
    pub fn generate(&self, platform_id: &str, content_id: &str) -> DeepLinkResult {
        match platform_id {
            "netflix" => self.generate_netflix_links(content_id),
            "prime_video" => self.generate_prime_video_links(content_id),
            "disney_plus" => self.generate_disney_plus_links(content_id),
            "youtube" => self.generate_youtube_links(content_id),
            "hulu" => self.generate_hulu_links(content_id),
            "hbo_max" => self.generate_hbo_max_links(content_id),
            "apple_tv_plus" => self.generate_apple_tv_plus_links(content_id),
            "paramount_plus" => self.generate_paramount_plus_links(content_id),
            "peacock" => self.generate_peacock_links(content_id),
            _ => self.generate_generic_links(platform_id, content_id),
        }
    }

    /// Generate Netflix deep links
    fn generate_netflix_links(&self, content_id: &str) -> DeepLinkResult {
        DeepLinkResult {
            mobile_url: Some(format!("netflix://title/{}", content_id)),
            web_url: format!("https://www.netflix.com/title/{}", content_id),
            tv_url: Some(format!("netflix://title/{}", content_id)),
        }
    }

    /// Generate Prime Video deep links
    fn generate_prime_video_links(&self, content_id: &str) -> DeepLinkResult {
        DeepLinkResult {
            mobile_url: Some(format!("aiv://aiv/view?gti={}", content_id)),
            web_url: format!("https://www.amazon.com/gp/video/detail/{}", content_id),
            tv_url: Some(format!("aiv://aiv/view?gti={}", content_id)),
        }
    }

    /// Generate Disney+ deep links
    fn generate_disney_plus_links(&self, content_id: &str) -> DeepLinkResult {
        DeepLinkResult {
            mobile_url: Some(format!("disneyplus://content/{}", content_id)),
            web_url: format!("https://www.disneyplus.com/video/{}", content_id),
            tv_url: Some(format!("disneyplus://content/{}", content_id)),
        }
    }

    /// Generate YouTube deep links
    fn generate_youtube_links(&self, content_id: &str) -> DeepLinkResult {
        DeepLinkResult {
            mobile_url: Some(format!("vnd.youtube://watch?v={}", content_id)),
            web_url: format!("https://www.youtube.com/watch?v={}", content_id),
            tv_url: Some(format!("vnd.youtube://watch?v={}", content_id)),
        }
    }

    /// Generate Hulu deep links
    fn generate_hulu_links(&self, content_id: &str) -> DeepLinkResult {
        DeepLinkResult {
            mobile_url: Some(format!("hulu://watch/{}", content_id)),
            web_url: format!("https://www.hulu.com/watch/{}", content_id),
            tv_url: Some(format!("hulu://watch/{}", content_id)),
        }
    }

    /// Generate HBO Max deep links
    fn generate_hbo_max_links(&self, content_id: &str) -> DeepLinkResult {
        DeepLinkResult {
            mobile_url: Some(format!("hbomax://content/{}", content_id)),
            web_url: format!("https://www.hbomax.com/video/{}", content_id),
            tv_url: Some(format!("hbomax://content/{}", content_id)),
        }
    }

    /// Generate Apple TV+ deep links
    fn generate_apple_tv_plus_links(&self, content_id: &str) -> DeepLinkResult {
        DeepLinkResult {
            mobile_url: Some(format!("videos://watch/{}", content_id)),
            web_url: format!("https://tv.apple.com/us/video/{}", content_id),
            tv_url: Some(format!("com.apple.tv://watch/{}", content_id)),
        }
    }

    /// Generate Paramount+ deep links
    fn generate_paramount_plus_links(&self, content_id: &str) -> DeepLinkResult {
        DeepLinkResult {
            mobile_url: Some(format!("paramountplus://content/{}", content_id)),
            web_url: format!("https://www.paramountplus.com/movies/{}", content_id),
            tv_url: Some(format!("paramountplus://content/{}", content_id)),
        }
    }

    /// Generate Peacock deep links
    fn generate_peacock_links(&self, content_id: &str) -> DeepLinkResult {
        DeepLinkResult {
            mobile_url: Some(format!("peacock://watch/{}", content_id)),
            web_url: format!("https://www.peacocktv.com/watch/{}", content_id),
            tv_url: Some(format!("peacock://watch/{}", content_id)),
        }
    }

    /// Generate generic deep links (fallback)
    fn generate_generic_links(&self, platform_id: &str, content_id: &str) -> DeepLinkResult {
        DeepLinkResult {
            mobile_url: None,
            web_url: format!("https://{}.com/watch/{}", platform_id, content_id),
            tv_url: None,
        }
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
    fn test_netflix_deep_links() {
        let generator = DeepLinkGenerator::new();
        let result = generator.generate("netflix", "80057281");

        assert_eq!(
            result.mobile_url,
            Some("netflix://title/80057281".to_string())
        );
        assert_eq!(result.web_url, "https://www.netflix.com/title/80057281");
        assert_eq!(result.tv_url, Some("netflix://title/80057281".to_string()));
    }

    #[test]
    fn test_prime_video_deep_links() {
        let generator = DeepLinkGenerator::new();
        let result = generator.generate("prime_video", "B08WJM48TX");

        assert_eq!(
            result.mobile_url,
            Some("aiv://aiv/view?gti=B08WJM48TX".to_string())
        );
        assert_eq!(
            result.web_url,
            "https://www.amazon.com/gp/video/detail/B08WJM48TX"
        );
        assert_eq!(
            result.tv_url,
            Some("aiv://aiv/view?gti=B08WJM48TX".to_string())
        );
    }

    #[test]
    fn test_youtube_deep_links() {
        let generator = DeepLinkGenerator::new();
        let result = generator.generate("youtube", "dQw4w9WgXcQ");

        assert_eq!(
            result.mobile_url,
            Some("vnd.youtube://watch?v=dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            result.web_url,
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
        );
        assert_eq!(
            result.tv_url,
            Some("vnd.youtube://watch?v=dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn test_disney_plus_deep_links() {
        let generator = DeepLinkGenerator::new();
        let result = generator.generate("disney_plus", "123abc");

        assert_eq!(
            result.mobile_url,
            Some("disneyplus://content/123abc".to_string())
        );
        assert_eq!(result.web_url, "https://www.disneyplus.com/video/123abc");
        assert_eq!(
            result.tv_url,
            Some("disneyplus://content/123abc".to_string())
        );
    }

    #[test]
    fn test_hulu_deep_links() {
        let generator = DeepLinkGenerator::new();
        let result = generator.generate("hulu", "xyz789");

        assert_eq!(result.mobile_url, Some("hulu://watch/xyz789".to_string()));
        assert_eq!(result.web_url, "https://www.hulu.com/watch/xyz789");
        assert_eq!(result.tv_url, Some("hulu://watch/xyz789".to_string()));
    }

    #[test]
    fn test_apple_tv_plus_deep_links() {
        let generator = DeepLinkGenerator::new();
        let result = generator.generate("apple_tv_plus", "content123");

        assert_eq!(
            result.mobile_url,
            Some("videos://watch/content123".to_string())
        );
        assert_eq!(result.web_url, "https://tv.apple.com/us/video/content123");
        assert_eq!(
            result.tv_url,
            Some("com.apple.tv://watch/content123".to_string())
        );
    }

    #[test]
    fn test_generic_fallback() {
        let generator = DeepLinkGenerator::new();
        let result = generator.generate("unknown_platform", "content456");

        assert_eq!(result.mobile_url, None);
        assert_eq!(
            result.web_url,
            "https://unknown_platform.com/watch/content456"
        );
        assert_eq!(result.tv_url, None);
    }
}
