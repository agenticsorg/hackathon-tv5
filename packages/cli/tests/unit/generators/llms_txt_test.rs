#[cfg(test)]
mod llms_txt_generator_tests {
    use arw_lib::{ArwConfig, generate_llms_txt};

    #[test]
    fn test_generate_minimal_manifest() {
        let config = ArwConfig {
            site_name: "Test Site".to_string(),
            homepage: "https://example.com".to_string(),
            contact: "ai@example.com".to_string(),
            profile: "ARW-1".to_string(),
            description: None,
        };

        let result = generate_llms_txt(&config);
        assert!(result.is_ok(), "Should successfully generate manifest");

        let content = result.unwrap();

        // Check for required fields
        assert!(content.contains("version: 1.0"));
        assert!(content.contains("profile: ARW-1"));
        assert!(content.contains("name: 'Test Site'"));
        assert!(content.contains("homepage: 'https://example.com'"));
        assert!(content.contains("contact: 'ai@example.com'"));

        // Check for policy defaults
        assert!(content.contains("training:"));
        assert!(content.contains("allowed: false"));
        assert!(content.contains("inference:"));
        assert!(content.contains("allowed: true"));
        assert!(content.contains("attribution:"));
        assert!(content.contains("required: true"));
    }

    #[test]
    fn test_generate_manifest_with_description() {
        let config = ArwConfig {
            site_name: "My Blog".to_string(),
            homepage: "https://myblog.com".to_string(),
            contact: "ai@myblog.com".to_string(),
            profile: "ARW-2".to_string(),
            description: Some("A technical blog about AI".to_string()),
        };

        let result = generate_llms_txt(&config);
        assert!(result.is_ok());

        let content = result.unwrap();
        assert!(content.contains("description: 'A technical blog about AI'"));
        assert!(content.contains("profile: ARW-2"));
    }

    #[test]
    fn test_generated_manifest_is_valid() {
        let config = ArwConfig {
            site_name: "Test Site".to_string(),
            homepage: "https://example.com".to_string(),
            contact: "ai@example.com".to_string(),
            profile: "ARW-1".to_string(),
            description: None,
        };

        let manifest_content = generate_llms_txt(&config).unwrap();

        // Parse as YAML and validate
        let manifest: serde_json::Value = serde_yaml::from_str(&manifest_content)
            .expect("Generated manifest should be valid YAML");

        let errors = arw_lib::validate_manifest(&manifest)
            .expect("Should validate successfully");

        assert_eq!(errors.len(), 0, "Generated manifest should have no validation errors");
    }

    #[test]
    fn test_generate_all_profile_levels() {
        for profile in &["ARW-1", "ARW-2", "ARW-3", "ARW-4"] {
            let config = ArwConfig {
                site_name: "Test Site".to_string(),
                homepage: "https://example.com".to_string(),
                contact: "ai@example.com".to_string(),
                profile: profile.to_string(),
                description: None,
            };

            let result = generate_llms_txt(&config);
            assert!(result.is_ok(), "Should generate manifest for {}", profile);

            let content = result.unwrap();
            assert!(content.contains(&format!("profile: {}", profile)));
        }
    }
}
