//! Integration tests for deep linking functionality

use media_gateway_playback::deep_link::{
    ContentType, DeepLinkGenerator, DeepLinkRequest, DeviceCapabilities, Platform,
};

#[test]
fn test_all_platforms_generate_valid_deep_links() {
    let generator = DeepLinkGenerator::new();

    // Test each platform with valid content
    let platforms = vec![
        (Platform::Netflix, "80123456", ContentType::Video),
        (
            Platform::Spotify,
            "3n3Ppam7vgaVa1iaRUc9Lp",
            ContentType::Track,
        ),
        (Platform::AppleMusic, "1234567890", ContentType::Track),
        (Platform::Hulu, "abc123", ContentType::Video),
        (Platform::DisneyPlus, "xyz789", ContentType::Video),
        (Platform::HboMax, "def456", ContentType::Video),
        (Platform::PrimeVideo, "B08XYZ", ContentType::Video),
    ];

    for (platform, content_id, content_type) in platforms {
        let request = DeepLinkRequest {
            platform,
            content_type,
            content_id: content_id.to_string(),
            start_position: None,
            device_capabilities: None,
        };

        let result = generator.generate(&request);
        assert!(
            result.is_ok(),
            "Failed to generate deep link for {:?}",
            platform
        );

        let deep_link = result.unwrap();
        assert!(!deep_link.deep_link_url.is_empty());
        assert!(!deep_link.web_fallback_url.is_empty());
        assert_eq!(deep_link.platform, platform);
    }
}

#[test]
fn test_netflix_complete_flow() {
    let generator = DeepLinkGenerator::new();
    let content_id = "80123456";

    // Test without device capabilities
    let request = DeepLinkRequest {
        platform: Platform::Netflix,
        content_type: ContentType::Video,
        content_id: content_id.to_string(),
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
    assert_eq!(
        result.universal_link.unwrap(),
        "https://www.netflix.com/watch/80123456"
    );
    assert!(result.is_supported); // Assumes supported when no capabilities provided

    // Test with iOS device that has Netflix installed
    let mut ios_caps = DeviceCapabilities::new("ios".to_string());
    ios_caps.installed_apps = vec!["Netflix".to_string()];

    let request_ios = DeepLinkRequest {
        platform: Platform::Netflix,
        content_type: ContentType::Video,
        content_id: content_id.to_string(),
        start_position: Some(120), // Start at 2 minutes
        device_capabilities: Some(ios_caps),
    };

    let result_ios = generator.generate(&request_ios).unwrap();
    assert!(result_ios.is_supported);
    assert!(result_ios.deep_link_url.contains("?t=120"));
    assert!(result_ios.web_fallback_url.contains("?t=120"));

    // Test with device that doesn't have Netflix
    let android_caps = DeviceCapabilities::new("android".to_string());

    let request_android = DeepLinkRequest {
        platform: Platform::Netflix,
        content_type: ContentType::Video,
        content_id: content_id.to_string(),
        start_position: None,
        device_capabilities: Some(android_caps),
    };

    let result_android = generator.generate(&request_android).unwrap();
    assert!(!result_android.is_supported);
}

#[test]
fn test_spotify_music_types() {
    let generator = DeepLinkGenerator::new();

    // Test track deep link
    let track_request = DeepLinkRequest {
        platform: Platform::Spotify,
        content_type: ContentType::Track,
        content_id: "3n3Ppam7vgaVa1iaRUc9Lp".to_string(),
        start_position: None,
        device_capabilities: None,
    };

    let track_result = generator.generate(&track_request).unwrap();
    assert_eq!(
        track_result.deep_link_url,
        "spotify://track/3n3Ppam7vgaVa1iaRUc9Lp"
    );
    assert_eq!(
        track_result.web_fallback_url,
        "https://open.spotify.com/track/3n3Ppam7vgaVa1iaRUc9Lp"
    );

    // Test album deep link
    let album_request = DeepLinkRequest {
        platform: Platform::Spotify,
        content_type: ContentType::Album,
        content_id: "6DEjYFkNZh67HP7R9PSZvv".to_string(),
        start_position: None,
        device_capabilities: None,
    };

    let album_result = generator.generate(&album_request).unwrap();
    assert_eq!(
        album_result.deep_link_url,
        "spotify://album/6DEjYFkNZh67HP7R9PSZvv"
    );
    assert_eq!(
        album_result.web_fallback_url,
        "https://open.spotify.com/album/6DEjYFkNZh67HP7R9PSZvv"
    );

    // Test playlist deep link
    let playlist_request = DeepLinkRequest {
        platform: Platform::Spotify,
        content_type: ContentType::Playlist,
        content_id: "37i9dQZF1DXcBWIGoYBM5M".to_string(),
        start_position: None,
        device_capabilities: None,
    };

    let playlist_result = generator.generate(&playlist_request).unwrap();
    assert_eq!(
        playlist_result.deep_link_url,
        "spotify://playlist/37i9dQZF1DXcBWIGoYBM5M"
    );
    assert_eq!(
        playlist_result.web_fallback_url,
        "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M"
    );
}

#[test]
fn test_apple_music_deep_links() {
    let generator = DeepLinkGenerator::new();

    let request = DeepLinkRequest {
        platform: Platform::AppleMusic,
        content_type: ContentType::Album,
        content_id: "1440857781".to_string(),
        start_position: None,
        device_capabilities: None,
    };

    let result = generator.generate(&request).unwrap();
    assert_eq!(
        result.deep_link_url,
        "music://itunes.apple.com/us/album/1440857781"
    );
    assert_eq!(
        result.web_fallback_url,
        "https://music.apple.com/us/album/1440857781"
    );
    assert!(result.universal_link.is_some());
}

#[test]
fn test_hulu_deep_links() {
    let generator = DeepLinkGenerator::new();

    let request = DeepLinkRequest {
        platform: Platform::Hulu,
        content_type: ContentType::Video,
        content_id: "series-123".to_string(),
        start_position: Some(600), // 10 minutes
        device_capabilities: None,
    };

    let result = generator.generate(&request).unwrap();
    assert_eq!(result.deep_link_url, "hulu://watch/series-123?t=600");
    assert_eq!(
        result.web_fallback_url,
        "https://www.hulu.com/watch/series-123?t=600"
    );
}

#[test]
fn test_disney_plus_deep_links() {
    let generator = DeepLinkGenerator::new();

    let request = DeepLinkRequest {
        platform: Platform::DisneyPlus,
        content_type: ContentType::Video,
        content_id: "mandalorian-s1e1".to_string(),
        start_position: None,
        device_capabilities: None,
    };

    let result = generator.generate(&request).unwrap();
    assert_eq!(
        result.deep_link_url,
        "disneyplus://content/mandalorian-s1e1"
    );
    assert_eq!(
        result.web_fallback_url,
        "https://www.disneyplus.com/video/mandalorian-s1e1"
    );
}

#[test]
fn test_hbo_max_deep_links() {
    let generator = DeepLinkGenerator::new();

    let request = DeepLinkRequest {
        platform: Platform::HboMax,
        content_type: ContentType::Video,
        content_id: "got-finale".to_string(),
        start_position: Some(1800), // 30 minutes
        device_capabilities: None,
    };

    let result = generator.generate(&request).unwrap();
    assert_eq!(result.deep_link_url, "hbomax://content/got-finale?t=1800");
    assert_eq!(
        result.web_fallback_url,
        "https://www.max.com/video/got-finale?t=1800"
    );
}

#[test]
fn test_prime_video_deep_links() {
    let generator = DeepLinkGenerator::new();

    let request = DeepLinkRequest {
        platform: Platform::PrimeVideo,
        content_type: ContentType::Video,
        content_id: "B08XYZABC".to_string(),
        start_position: None,
        device_capabilities: None,
    };

    let result = generator.generate(&request).unwrap();
    assert_eq!(result.deep_link_url, "primevideo://detail?id=B08XYZABC");
    assert_eq!(
        result.web_fallback_url,
        "https://www.amazon.com/gp/video/detail/B08XYZABC"
    );
}

#[test]
fn test_device_capabilities_ios() {
    let mut ios_caps = DeviceCapabilities::new("ios".to_string());
    ios_caps.os_version = Some("15.0".to_string());
    ios_caps.installed_apps = vec![
        "Netflix".to_string(),
        "Spotify".to_string(),
        "Apple Music".to_string(),
    ];

    assert!(ios_caps.has_platform_app(Platform::Netflix));
    assert!(ios_caps.has_platform_app(Platform::Spotify));
    assert!(ios_caps.has_platform_app(Platform::AppleMusic));
    assert!(!ios_caps.has_platform_app(Platform::Hulu));

    assert!(ios_caps.supports_deep_link(Platform::Netflix));
    assert!(ios_caps.supports_deep_link(Platform::Spotify));
    assert!(!ios_caps.supports_deep_link(Platform::DisneyPlus));
}

#[test]
fn test_device_capabilities_android() {
    let mut android_caps = DeviceCapabilities::new("android".to_string());
    android_caps.os_version = Some("12.0".to_string());
    android_caps.installed_apps = vec![
        "com.netflix.mediaclient".to_string(),
        "com.hulu.plus".to_string(),
    ];

    assert!(android_caps.has_platform_app(Platform::Netflix));
    assert!(android_caps.has_platform_app(Platform::Hulu));
    assert!(!android_caps.has_platform_app(Platform::Spotify));

    assert!(android_caps.supports_deep_link(Platform::Netflix));
    assert!(!android_caps.supports_deep_link(Platform::Spotify));
}

#[test]
fn test_device_capabilities_web_with_universal_links() {
    let mut web_caps = DeviceCapabilities::new("web".to_string());
    web_caps.supports_universal_links = true;

    // Web with universal links support should work for all platforms
    assert!(web_caps.supports_deep_link(Platform::Netflix));
    assert!(web_caps.supports_deep_link(Platform::Spotify));
    assert!(web_caps.supports_deep_link(Platform::AppleMusic));
}

#[test]
fn test_device_capabilities_web_without_universal_links() {
    let web_caps = DeviceCapabilities::new("web".to_string());

    // Web without universal links should not support deep links
    assert!(!web_caps.supports_deep_link(Platform::Netflix));
    assert!(!web_caps.supports_deep_link(Platform::Spotify));
}

#[test]
fn test_generate_all_platforms() {
    let generator = DeepLinkGenerator::new();

    // Generate deep links for all platforms
    let all_links = generator.generate_all("test-content-123", ContentType::Video, None);

    // Verify all 7 platforms are included
    assert_eq!(all_links.len(), 7);

    // Verify each platform
    assert!(all_links.contains_key(&Platform::Netflix));
    assert!(all_links.contains_key(&Platform::Spotify));
    assert!(all_links.contains_key(&Platform::AppleMusic));
    assert!(all_links.contains_key(&Platform::Hulu));
    assert!(all_links.contains_key(&Platform::DisneyPlus));
    assert!(all_links.contains_key(&Platform::HboMax));
    assert!(all_links.contains_key(&Platform::PrimeVideo));

    // Verify each has valid URLs
    for (platform, link) in all_links.iter() {
        assert!(
            !link.deep_link_url.is_empty(),
            "Empty deep link for {:?}",
            platform
        );
        assert!(
            !link.web_fallback_url.is_empty(),
            "Empty fallback for {:?}",
            platform
        );
        assert!(
            link.deep_link_url.contains("test-content-123"),
            "Missing content ID in {:?}",
            platform
        );
    }
}

#[test]
fn test_generate_all_with_device_capabilities() {
    let generator = DeepLinkGenerator::new();

    let mut ios_caps = DeviceCapabilities::new("ios".to_string());
    ios_caps.installed_apps = vec!["Netflix".to_string(), "Hulu".to_string()];

    let all_links = generator.generate_all("test-123", ContentType::Video, Some(ios_caps));

    // Netflix should be supported
    let netflix_link = all_links.get(&Platform::Netflix).unwrap();
    assert!(netflix_link.is_supported);

    // Hulu should be supported
    let hulu_link = all_links.get(&Platform::Hulu).unwrap();
    assert!(hulu_link.is_supported);

    // Spotify should not be supported
    let spotify_link = all_links.get(&Platform::Spotify).unwrap();
    assert!(!spotify_link.is_supported);
}

#[test]
fn test_start_position_formats() {
    let generator = DeepLinkGenerator::new();

    // Test with no start position
    let request_no_pos = DeepLinkRequest {
        platform: Platform::Netflix,
        content_type: ContentType::Video,
        content_id: "123".to_string(),
        start_position: None,
        device_capabilities: None,
    };
    let result = generator.generate(&request_no_pos).unwrap();
    assert!(!result.deep_link_url.contains("?t="));
    assert!(!result.web_fallback_url.contains("?t="));

    // Test with start position at beginning
    let request_zero = DeepLinkRequest {
        platform: Platform::Netflix,
        content_type: ContentType::Video,
        content_id: "123".to_string(),
        start_position: Some(0),
        device_capabilities: None,
    };
    let result = generator.generate(&request_zero).unwrap();
    assert!(result.deep_link_url.contains("?t=0"));

    // Test with mid-point position
    let request_mid = DeepLinkRequest {
        platform: Platform::Netflix,
        content_type: ContentType::Video,
        content_id: "123".to_string(),
        start_position: Some(3600), // 1 hour
        device_capabilities: None,
    };
    let result = generator.generate(&request_mid).unwrap();
    assert!(result.deep_link_url.contains("?t=3600"));
    assert!(result.web_fallback_url.contains("?t=3600"));
}

#[test]
fn test_platform_string_conversion() {
    assert_eq!(Platform::Netflix.as_str(), "netflix");
    assert_eq!(Platform::Spotify.as_str(), "spotify");
    assert_eq!(Platform::AppleMusic.as_str(), "applemusic");
    assert_eq!(Platform::Hulu.as_str(), "hulu");
    assert_eq!(Platform::DisneyPlus.as_str(), "disneyplus");
    assert_eq!(Platform::HboMax.as_str(), "hbomax");
    assert_eq!(Platform::PrimeVideo.as_str(), "primevideo");
}

#[test]
fn test_all_content_types_supported() {
    let generator = DeepLinkGenerator::new();

    let content_types = vec![
        ContentType::Video,
        ContentType::Track,
        ContentType::Album,
        ContentType::Playlist,
    ];

    for content_type in content_types {
        let request = DeepLinkRequest {
            platform: Platform::Spotify,
            content_type,
            content_id: "test123".to_string(),
            start_position: None,
            device_capabilities: None,
        };

        let result = generator.generate(&request);
        assert!(result.is_ok(), "Failed for content type {:?}", content_type);
    }
}

#[test]
fn test_edge_cases() {
    let generator = DeepLinkGenerator::new();

    // Test with very long content ID
    let long_id = "a".repeat(100);
    let request = DeepLinkRequest {
        platform: Platform::Netflix,
        content_type: ContentType::Video,
        content_id: long_id.clone(),
        start_position: None,
        device_capabilities: None,
    };
    let result = generator.generate(&request).unwrap();
    assert!(result.deep_link_url.contains(&long_id));

    // Test with special characters in content ID
    let special_id = "test-123_abc.xyz";
    let request = DeepLinkRequest {
        platform: Platform::Spotify,
        content_type: ContentType::Track,
        content_id: special_id.to_string(),
        start_position: None,
        device_capabilities: None,
    };
    let result = generator.generate(&request).unwrap();
    assert!(result.deep_link_url.contains(special_id));

    // Test with maximum position value
    let request = DeepLinkRequest {
        platform: Platform::Hulu,
        content_type: ContentType::Video,
        content_id: "test".to_string(),
        start_position: Some(u32::MAX),
        device_capabilities: None,
    };
    let result = generator.generate(&request).unwrap();
    assert!(result.deep_link_url.contains(&u32::MAX.to_string()));
}
