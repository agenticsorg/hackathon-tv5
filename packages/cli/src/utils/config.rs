use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// CLI-only configuration (NOT site information - that belongs in llms.txt)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArwConfig {
    /// CLI-specific settings
    pub cli: CliConfig,
}

/// CLI preferences and tool settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// File patterns to watch
    pub watch_patterns: Vec<String>,
    /// Output directory for generated files
    pub output_dir: String,
    /// Patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Chunk strategy (semantic, heading-based, etc.)
    pub chunk_strategy: String,
}

// Legacy structs kept for backwards compatibility during migration
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    pub title: String,
    pub description: String,
    pub homepage: String,
    pub contact: Option<String>,
    pub languages: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub output_dir: String,
    pub chunk_strategy: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub allow_training: bool,
    pub allow_inference: bool,
    pub require_attribution: bool,
    pub rate_limit: Option<String>,
}

impl Default for ArwConfig {
    fn default() -> Self {
        Self {
            cli: CliConfig {
                watch_patterns: vec!["**/*.html".to_string(), "**/*.md".to_string()],
                output_dir: ".".to_string(),
                exclude_patterns: vec![
                    "node_modules/**".to_string(),
                    ".git/**".to_string(),
                    "target/**".to_string(),
                    "dist/**".to_string(),
                ],
                chunk_strategy: "semantic".to_string(),
            },
        }
    }
}

impl ArwConfig {
    /// Load configuration from .arw/config.yaml
    #[allow(dead_code)]
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_path = path.as_ref().join(".arw").join("config.yaml");

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

        serde_yaml::from_str(&content)
            .with_context(|| "Failed to parse config file")
    }

    /// Save configuration to .arw/config.yaml
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let config_dir = path.as_ref().join(".arw");
        fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create config directory: {:?}", config_dir))?;

        let config_path = config_dir.join("config.yaml");
        let content = serde_yaml::to_string(self)
            .with_context(|| "Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        Ok(())
    }

    /// Check if .arw directory exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().join(".arw").exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_arw_config_default() {
        let config = ArwConfig::default();

        assert_eq!(config.cli.output_dir, ".");
        assert_eq!(config.cli.chunk_strategy, "semantic");
        assert!(config.cli.watch_patterns.contains(&"**/*.html".to_string()));
        assert!(config.cli.watch_patterns.contains(&"**/*.md".to_string()));
        assert!(config.cli.exclude_patterns.contains(&"node_modules/**".to_string()));
        assert!(config.cli.exclude_patterns.contains(&".git/**".to_string()));
    }

    #[test]
    fn test_arw_config_exists_true() {
        let temp_dir = TempDir::new().unwrap();
        let arw_dir = temp_dir.path().join(".arw");
        std::fs::create_dir_all(&arw_dir).unwrap();

        assert!(ArwConfig::exists(temp_dir.path()));
    }

    #[test]
    fn test_arw_config_exists_false() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!ArwConfig::exists(temp_dir.path()));
    }

    #[test]
    fn test_arw_config_load_default_when_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let config = ArwConfig::load(temp_dir.path()).unwrap();

        // Should return default config when file doesn't exist
        assert_eq!(config.cli.output_dir, ".");
    }

    #[test]
    fn test_arw_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = ArwConfig::default();
        config.cli.output_dir = "custom/output".to_string();
        config.cli.chunk_strategy = "heading-based".to_string();

        // Save config
        config.save(temp_dir.path()).unwrap();

        // Verify .arw directory was created
        assert!(ArwConfig::exists(temp_dir.path()));

        // Load config back
        let loaded_config = ArwConfig::load(temp_dir.path()).unwrap();
        assert_eq!(loaded_config.cli.output_dir, "custom/output");
        assert_eq!(loaded_config.cli.chunk_strategy, "heading-based");
    }

    #[test]
    fn test_arw_config_save_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = ArwConfig::default();

        config.save(temp_dir.path()).unwrap();

        let arw_dir = temp_dir.path().join(".arw");
        assert!(arw_dir.exists());
        assert!(arw_dir.is_dir());
    }

    #[test]
    fn test_arw_config_save_creates_yaml_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = ArwConfig::default();

        config.save(temp_dir.path()).unwrap();

        let config_file = temp_dir.path().join(".arw").join("config.yaml");
        assert!(config_file.exists());
        assert!(config_file.is_file());
    }

    #[test]
    fn test_cli_config_watch_patterns() {
        let config = ArwConfig::default();

        assert_eq!(config.cli.watch_patterns.len(), 2);
        assert!(config.cli.watch_patterns.contains(&"**/*.html".to_string()));
        assert!(config.cli.watch_patterns.contains(&"**/*.md".to_string()));
    }

    #[test]
    fn test_cli_config_exclude_patterns() {
        let config = ArwConfig::default();

        assert_eq!(config.cli.exclude_patterns.len(), 4);
        assert!(config.cli.exclude_patterns.contains(&"node_modules/**".to_string()));
        assert!(config.cli.exclude_patterns.contains(&".git/**".to_string()));
        assert!(config.cli.exclude_patterns.contains(&"target/**".to_string()));
        assert!(config.cli.exclude_patterns.contains(&"dist/**".to_string()));
    }

    #[test]
    fn test_arw_config_custom_values() {
        let config = ArwConfig {
            cli: CliConfig {
                watch_patterns: vec!["*.txt".to_string()],
                output_dir: "/tmp/output".to_string(),
                exclude_patterns: vec!["build/**".to_string()],
                chunk_strategy: "custom".to_string(),
            },
        };

        assert_eq!(config.cli.watch_patterns.len(), 1);
        assert_eq!(config.cli.output_dir, "/tmp/output");
        assert_eq!(config.cli.chunk_strategy, "custom");
    }

    #[test]
    fn test_arw_config_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let original = ArwConfig {
            cli: CliConfig {
                watch_patterns: vec!["**/*.rs".to_string(), "**/*.toml".to_string()],
                output_dir: "target/docs".to_string(),
                exclude_patterns: vec!["**/*.bak".to_string()],
                chunk_strategy: "ast-based".to_string(),
            },
        };

        original.save(temp_dir.path()).unwrap();
        let loaded = ArwConfig::load(temp_dir.path()).unwrap();

        assert_eq!(loaded.cli.watch_patterns, original.cli.watch_patterns);
        assert_eq!(loaded.cli.output_dir, original.cli.output_dir);
        assert_eq!(loaded.cli.exclude_patterns, original.cli.exclude_patterns);
        assert_eq!(loaded.cli.chunk_strategy, original.cli.chunk_strategy);
    }

    #[test]
    fn test_site_config_legacy_struct() {
        let site = SiteConfig {
            title: "Test Site".to_string(),
            description: "A test site".to_string(),
            homepage: "https://example.com".to_string(),
            contact: Some("test@example.com".to_string()),
            languages: vec!["en".to_string(), "es".to_string()],
        };

        assert_eq!(site.title, "Test Site");
        assert_eq!(site.homepage, "https://example.com");
        assert_eq!(site.contact, Some("test@example.com".to_string()));
        assert_eq!(site.languages.len(), 2);
    }

    #[test]
    fn test_generation_config_legacy_struct() {
        let gen_config = GenerationConfig {
            output_dir: "output".to_string(),
            chunk_strategy: "semantic".to_string(),
            include_patterns: vec!["**/*.md".to_string()],
            exclude_patterns: vec!["node_modules/**".to_string()],
        };

        assert_eq!(gen_config.output_dir, "output");
        assert_eq!(gen_config.chunk_strategy, "semantic");
        assert_eq!(gen_config.include_patterns.len(), 1);
        assert_eq!(gen_config.exclude_patterns.len(), 1);
    }

    #[test]
    fn test_policy_config_legacy_struct() {
        let policy = PolicyConfig {
            allow_training: true,
            allow_inference: false,
            require_attribution: true,
            rate_limit: Some("100/hour".to_string()),
        };

        assert!(policy.allow_training);
        assert!(!policy.allow_inference);
        assert!(policy.require_attribution);
        assert_eq!(policy.rate_limit, Some("100/hour".to_string()));
    }

    #[test]
    fn test_arw_config_clone() {
        let config = ArwConfig::default();
        let cloned = config.clone();

        assert_eq!(config.cli.output_dir, cloned.cli.output_dir);
        assert_eq!(config.cli.chunk_strategy, cloned.cli.chunk_strategy);
    }

    #[test]
    fn test_arw_config_debug_format() {
        let config = ArwConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("ArwConfig"));
        assert!(debug_str.contains("cli"));
    }
}
