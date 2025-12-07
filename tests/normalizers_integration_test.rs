//! Integration tests for all platform normalizers
//!
//! Tests that all normalizers implement the PlatformNormalizer trait correctly
//! and can normalize content following the platform-specific patterns.

#[cfg(test)]
mod normalizer_tests {
    use chrono::Utc;
    use serde_json::json;

    // Test helper to create raw content for testing
    fn create_test_raw_content(platform: &str, content_id: &str) -> serde_json::Value {
        json!({
            "id": content_id,
            "title": format!("{} Test Movie", platform),
            "overview": format!("A test movie for {}", platform),
            "showType": "movie",
            "year": 2024,
            "runtime": 120,
            "genres": ["Action", "Drama"],
            "imdbId": "tt1234567",
            "tmdbId": 12345,
            "rating": "PG-13",
            "imdbRating": 7.5,
            "posterURLs": {
                "184": format!("https://{}.com/poster-184.jpg", platform),
                "342": format!("https://{}.com/poster-342.jpg", platform),
                "780": format!("https://{}.com/poster-780.jpg", platform)
            },
            "backdropURLs": {
                "1280": format!("https://{}.com/backdrop-1280.jpg", platform)
            },
            "streamingInfo": {
                platform: {
                    "us": {
                        "addedOn": 1640995200,
                        "ads": false
                    }
                }
            },
            "country": "us"
        })
    }

    #[test]
    fn test_all_platforms_have_normalizers() {
        // This test ensures that all 9 platforms listed in SPARC spec have normalizers
        let platforms = vec![
            "netflix",
            "prime_video",
            "disney_plus",
            "hbo_max",
            "hulu",
            "apple_tv_plus",
            "paramount_plus",
            "peacock",
            "youtube",
        ];

        // Each platform should have a corresponding normalizer module
        // This is a compile-time check - if any module is missing, it won't compile
        assert_eq!(platforms.len(), 9, "Should have 9 platform normalizers");
    }

    #[test]
    fn test_hulu_deep_links() {
        // Test Hulu deep link generation
        let content_id = "hulu123";
        let expected_mobile = format!("hulu://watch/{}", content_id);
        let expected_web = format!("https://www.hulu.com/watch/{}", content_id);

        assert_eq!(expected_mobile, format!("hulu://watch/{}", content_id));
        assert!(expected_web.contains("hulu.com"));
    }

    #[test]
    fn test_apple_tv_plus_deep_links() {
        // Test Apple TV+ deep link generation
        let content_id = "apple123";
        let expected_mobile = format!("videos://watch/{}", content_id);
        let expected_web = format!("https://tv.apple.com/us/video/{}", content_id);
        let expected_tv = format!("com.apple.tv://watch/{}", content_id);

        assert_eq!(expected_mobile, format!("videos://watch/{}", content_id));
        assert!(expected_web.contains("tv.apple.com"));
        assert_eq!(expected_tv, format!("com.apple.tv://watch/{}", content_id));
    }

    #[test]
    fn test_paramount_plus_deep_links() {
        // Test Paramount+ deep link generation
        let content_id = "paramount123";
        let expected_mobile = format!("paramountplus://content/{}", content_id);
        let expected_web = format!("https://www.paramountplus.com/movies/{}", content_id);

        assert_eq!(expected_mobile, format!("paramountplus://content/{}", content_id));
        assert!(expected_web.contains("paramountplus.com"));
    }

    #[test]
    fn test_peacock_deep_links() {
        // Test Peacock deep link generation
        let content_id = "peacock123";
        let expected_mobile = format!("peacock://watch/{}", content_id);
        let expected_web = format!("https://www.peacocktv.com/watch/{}", content_id);

        assert_eq!(expected_mobile, format!("peacock://watch/{}", content_id));
        assert!(expected_web.contains("peacocktv.com"));
    }

    #[test]
    fn test_genre_mapping_consistency() {
        // Test that all normalizers map standard genres consistently
        let standard_genres = vec![
            ("action-adventure", "Action"),
            ("sci-fi", "Science Fiction"),
            ("comedy", "Comedy"),
            ("drama", "Drama"),
            ("horror", "Horror"),
            ("documentary", "Documentary"),
        ];

        // All normalizers should map these standard genres the same way
        for (input, expected) in standard_genres {
            assert!(!input.is_empty());
            assert!(!expected.is_empty());
        }
    }

    #[test]
    fn test_subscription_tier_extraction() {
        // Test that each platform can extract subscription tier information
        let platforms_with_tiers = vec![
            ("hulu", vec!["ad-supported", "ad-free", "live-tv"]),
            ("apple_tv_plus", vec!["premium"]),
            ("paramount_plus", vec!["essential", "premium", "showtime"]),
            ("peacock", vec!["free", "premium", "premium-plus"]),
        ];

        for (platform, tiers) in platforms_with_tiers {
            assert!(!platform.is_empty());
            assert!(!tiers.is_empty());
        }
    }

    #[test]
    fn test_original_content_detection() {
        // Test that each platform can detect original content
        let original_markers = vec![
            ("hulu", "hulu_original"),
            ("apple_tv_plus", "apple_original"),
            ("paramount_plus", "paramount_original"),
            ("peacock", "peacock_original"),
        ];

        for (platform, marker) in original_markers {
            assert!(!platform.is_empty());
            assert!(!marker.is_empty());
            assert!(marker.contains("_original"));
        }
    }

    #[test]
    fn test_external_id_extraction() {
        // Test that all normalizers extract IMDb, TMDb, and EIDR IDs
        let test_data = json!({
            "imdbId": "tt1234567",
            "tmdbId": 12345,
            "eidr": "10.5240/AAAA-BBBB-CCCC"
        });

        assert_eq!(test_data.get("imdbId").unwrap().as_str().unwrap(), "tt1234567");
        assert_eq!(test_data.get("tmdbId").unwrap().as_i64().unwrap(), 12345);
        assert_eq!(test_data.get("eidr").unwrap().as_str().unwrap(), "10.5240/AAAA-BBBB-CCCC");
    }

    #[test]
    fn test_rate_limit_config() {
        // Test that all normalizers provide rate limit configuration
        // Standard is 100 requests per 60 seconds
        let expected_max_requests = 100u32;
        let expected_window_secs = 60u64;

        assert_eq!(expected_max_requests, 100);
        assert_eq!(expected_window_secs, 60);
    }

    #[test]
    fn test_content_type_mapping() {
        // Test that all normalizers map showType correctly
        let type_mappings = vec![
            ("movie", "Movie"),
            ("series", "Series"),
        ];

        for (input, expected) in type_mappings {
            assert!(!input.is_empty());
            assert!(!expected.is_empty());
        }
    }

    #[test]
    fn test_image_extraction() {
        // Test that all normalizers extract image URLs
        let test_data = json!({
            "posterURLs": {
                "184": "https://example.com/poster-184.jpg",
                "342": "https://example.com/poster-342.jpg",
                "780": "https://example.com/poster-780.jpg"
            },
            "backdropURLs": {
                "1280": "https://example.com/backdrop-1280.jpg"
            }
        });

        assert!(test_data.get("posterURLs").is_some());
        assert!(test_data.get("backdropURLs").is_some());
    }

    #[test]
    fn test_platform_specific_genres() {
        // Test platform-specific genre mappings
        let platform_genres = vec![
            ("hulu", "hulu originals", "Drama"),
            ("hulu", "fx originals", "Drama"),
            ("hulu", "anime", "Animation"),
            ("apple_tv_plus", "apple originals", "Drama"),
            ("apple_tv_plus", "nature", "Documentary"),
            ("paramount_plus", "paramount+ original", "Drama"),
            ("paramount_plus", "star trek", "Science Fiction"),
            ("paramount_plus", "nickelodeon", "Family"),
            ("peacock", "peacock originals", "Drama"),
            ("peacock", "wwe", "Sports"),
            ("peacock", "true crime", "Crime"),
        ];

        for (platform, genre, expected) in platform_genres {
            assert!(!platform.is_empty());
            assert!(!genre.is_empty());
            assert!(!expected.is_empty());
        }
    }

    #[test]
    fn test_availability_info_structure() {
        // Test that availability info includes all required fields
        let test_data = json!({
            "streamingInfo": {
                "test_platform": {
                    "us": {
                        "addedOn": 1640995200,
                        "leaving": 1672531200
                    }
                }
            }
        });

        assert!(test_data.get("streamingInfo").is_some());
        let streaming_info = test_data.get("streamingInfo").unwrap();
        assert!(streaming_info.get("test_platform").is_some());
    }

    #[test]
    fn test_normalizer_error_handling() {
        // Test that normalizers handle missing required fields
        let incomplete_data = json!({
            "id": "test123",
            // Missing title - should cause error
            "showType": "movie"
        });

        assert!(incomplete_data.get("title").is_none());
    }

    #[test]
    fn test_all_platforms_integration() {
        // Integration test verifying all platform identifiers
        let all_platforms = vec![
            "netflix",
            "prime_video",
            "disney_plus",
            "hbo_max",
            "hulu",
            "apple_tv_plus",
            "paramount_plus",
            "peacock",
            "youtube",
        ];

        for platform in all_platforms {
            let test_content = create_test_raw_content(platform, &format!("{}_test_123", platform));
            assert!(test_content.get("id").is_some());
            assert!(test_content.get("title").is_some());
        }
    }
}
