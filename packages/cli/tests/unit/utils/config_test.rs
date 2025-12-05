use arw_cli::utils::config::*;
use tempfile::TempDir;
use std::fs;

/// Additional tests for config.rs to achieve 100% coverage
/// These tests cover the missing 8 lines from the existing coverage

#[test]
fn test_load_invalid_yaml_format() {
    let temp_dir = TempDir::new().unwrap();
    let arw_dir = temp_dir.path().join(".arw");
    fs::create_dir_all(&arw_dir).unwrap();

    // Write invalid YAML
    let config_file = arw_dir.join("config.yaml");
    fs::write(&config_file, "invalid: yaml: content: [unclosed").unwrap();

    // Should fail to parse
    let result = ArwConfig::load(temp_dir.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("parse"));
}

#[test]
fn test_load_malformed_yaml_structure() {
    let temp_dir = TempDir::new().unwrap();
    let arw_dir = temp_dir.path().join(".arw");
    fs::create_dir_all(&arw_dir).unwrap();

    // Write YAML with wrong structure
    let config_file = arw_dir.join("config.yaml");
    fs::write(&config_file, "not_the_right_structure: true").unwrap();

    let result = ArwConfig::load(temp_dir.path());
    assert!(result.is_err());
}

#[test]
fn test_save_to_readonly_directory() {
    // This test verifies error handling when saving to a readonly location
    let temp_dir = TempDir::new().unwrap();
    let config = ArwConfig::default();

    // On Unix systems, we can create a readonly directory
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir(&readonly_dir).unwrap();

        // Make directory readonly
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o444);
        fs::set_permissions(&readonly_dir, perms).unwrap();

        let result = config.save(&readonly_dir);

        // Should fail due to permissions
        assert!(result.is_err());

        // Cleanup: restore permissions
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&readonly_dir, perms).unwrap();
    }
}

#[test]
fn test_save_with_unwritable_parent() {
    let temp_dir = TempDir::new().unwrap();
    let config = ArwConfig::default();

    // Try to save to a path that doesn't exist and can't be created
    let invalid_path = temp_dir.path().join("nonexistent").join("deeply").join("nested");

    // Create first level as a file (not a directory) to block creation
    let blocker = temp_dir.path().join("nonexistent");
    fs::write(&blocker, "blocker").unwrap();

    let result = config.save(&invalid_path);
    assert!(result.is_err());
}

#[test]
fn test_load_from_nonexistent_parent_directory() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("does_not_exist");

    // Should return default config without error
    let result = ArwConfig::load(&nonexistent);
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.cli.output_dir, ".");
}

#[test]
fn test_site_config_with_none_contact() {
    let site = SiteConfig {
        title: "Test".to_string(),
        description: "Desc".to_string(),
        homepage: "https://example.com".to_string(),
        contact: None,
        languages: vec![],
    };

    assert_eq!(site.contact, None);
    assert_eq!(site.languages.len(), 0);
}

#[test]
fn test_policy_config_with_none_rate_limit() {
    let policy = PolicyConfig {
        allow_training: false,
        allow_inference: true,
        require_attribution: false,
        rate_limit: None,
    };

    assert_eq!(policy.rate_limit, None);
}

#[test]
fn test_cli_config_serialization() {
    let config = ArwConfig {
        cli: CliConfig {
            watch_patterns: vec!["*.rs".to_string()],
            output_dir: "/tmp".to_string(),
            exclude_patterns: vec!["*.bak".to_string()],
            chunk_strategy: "test".to_string(),
        },
    };

    let yaml = serde_yaml::to_string(&config).unwrap();
    assert!(yaml.contains("watch_patterns"));
    assert!(yaml.contains("*.rs"));
    assert!(yaml.contains("/tmp"));
}

#[test]
fn test_cli_config_deserialization() {
    let yaml = r#"
cli:
  watch_patterns:
    - "*.rs"
  output_dir: "/tmp"
  exclude_patterns:
    - "*.bak"
  chunk_strategy: "test"
"#;

    let config: ArwConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.cli.watch_patterns, vec!["*.rs".to_string()]);
    assert_eq!(config.cli.output_dir, "/tmp");
}

#[test]
fn test_arw_config_empty_patterns() {
    let config = ArwConfig {
        cli: CliConfig {
            watch_patterns: vec![],
            output_dir: ".".to_string(),
            exclude_patterns: vec![],
            chunk_strategy: "semantic".to_string(),
        },
    };

    assert_eq!(config.cli.watch_patterns.len(), 0);
    assert_eq!(config.cli.exclude_patterns.len(), 0);
}

#[test]
fn test_save_and_overwrite_existing_config() {
    let temp_dir = TempDir::new().unwrap();

    // Save first config
    let mut config1 = ArwConfig::default();
    config1.cli.output_dir = "first".to_string();
    config1.save(temp_dir.path()).unwrap();

    // Save second config (overwrite)
    let mut config2 = ArwConfig::default();
    config2.cli.output_dir = "second".to_string();
    config2.save(temp_dir.path()).unwrap();

    // Load and verify second config won
    let loaded = ArwConfig::load(temp_dir.path()).unwrap();
    assert_eq!(loaded.cli.output_dir, "second");
}

#[test]
fn test_legacy_structs_serialization() {
    let site = SiteConfig {
        title: "Test".to_string(),
        description: "Desc".to_string(),
        homepage: "https://example.com".to_string(),
        contact: Some("test@example.com".to_string()),
        languages: vec!["en".to_string()],
    };

    let yaml = serde_yaml::to_string(&site).unwrap();
    assert!(yaml.contains("title"));
    assert!(yaml.contains("Test"));
}

#[test]
fn test_generation_config_deserialization() {
    let yaml = r#"
output_dir: "output"
chunk_strategy: "semantic"
include_patterns:
  - "**/*.md"
exclude_patterns:
  - "node_modules/**"
"#;

    let config: GenerationConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.output_dir, "output");
    assert_eq!(config.chunk_strategy, "semantic");
}

#[test]
fn test_policy_config_all_false() {
    let policy = PolicyConfig {
        allow_training: false,
        allow_inference: false,
        require_attribution: false,
        rate_limit: None,
    };

    assert!(!policy.allow_training);
    assert!(!policy.allow_inference);
    assert!(!policy.require_attribution);
}
