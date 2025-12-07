//! Vault configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for a vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Vault root path
    pub path: PathBuf,
    /// Vault name
    pub name: String,
    /// Patterns to ignore
    pub ignore_patterns: Vec<String>,
    /// File extensions to include
    pub include_extensions: Vec<String>,
    /// Whether to watch for changes
    pub watch_enabled: bool,
    /// Debounce interval in milliseconds
    pub watch_debounce_ms: u64,
    /// Whether to use trash instead of permanent delete
    pub use_trash: bool,
    /// Attachment folder path (relative to vault)
    pub attachment_folder: String,
    /// Default location for new notes
    pub default_location: String,
    /// Whether to use wikilinks
    pub use_wikilinks: bool,
    /// Whether to update links automatically on rename
    pub auto_update_links: bool,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            name: String::new(),
            ignore_patterns: vec![
                ".git".to_string(),
                ".obsidian".to_string(),
                "node_modules".to_string(),
                ".trash".to_string(),
            ],
            include_extensions: vec![
                "md".to_string(),
                "markdown".to_string(),
                "txt".to_string(),
            ],
            watch_enabled: true,
            watch_debounce_ms: 100,
            use_trash: true,
            attachment_folder: "attachments".to_string(),
            default_location: "".to_string(),
            use_wikilinks: true,
            auto_update_links: true,
        }
    }
}

impl VaultConfig {
    /// Create a new config with the given path
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Vault".to_string());

        Self {
            path,
            name,
            ..Default::default()
        }
    }

    /// Set the vault name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Add an ignore pattern
    pub fn with_ignore(mut self, pattern: impl Into<String>) -> Self {
        self.ignore_patterns.push(pattern.into());
        self
    }

    /// Set whether to watch for changes
    pub fn with_watch(mut self, enabled: bool) -> Self {
        self.watch_enabled = enabled;
        self
    }

    /// Set the watch debounce interval
    pub fn with_debounce(mut self, ms: u64) -> Self {
        self.watch_debounce_ms = ms;
        self
    }

    /// Set whether to use trash
    pub fn with_trash(mut self, enabled: bool) -> Self {
        self.use_trash = enabled;
        self
    }

    /// Set the attachment folder
    pub fn with_attachment_folder(mut self, folder: impl Into<String>) -> Self {
        self.attachment_folder = folder.into();
        self
    }

    /// Check if a path should be ignored
    pub fn should_ignore(&self, path: &std::path::Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.ignore_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Check if a file extension is included
    pub fn is_included_extension(&self, path: &std::path::Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            self.include_extensions.iter().any(|e| e.to_lowercase() == ext_str)
        } else {
            false
        }
    }

    /// Check if a path is a markdown file
    pub fn is_markdown_file(&self, path: &std::path::Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            ext_str == "md" || ext_str == "markdown"
        } else {
            false
        }
    }

    /// Get the database path
    pub fn database_path(&self) -> PathBuf {
        self.path.join(".obsidian-rs").join("cache.redb")
    }

    /// Get the config directory path
    pub fn config_dir(&self) -> PathBuf {
        self.path.join(".obsidian-rs")
    }

    /// Get the trash directory path
    pub fn trash_dir(&self) -> PathBuf {
        self.path.join(".trash")
    }

    /// Load config from a vault path
    pub fn load(vault_path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let vault_path = vault_path.into();
        let config_path = vault_path.join(".obsidian-rs").join("config.json");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let mut config: VaultConfig = serde_json::from_str(&content)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            config.path = vault_path;
            Ok(config)
        } else {
            Ok(Self::new(vault_path))
        }
    }

    /// Save config to the vault
    pub fn save(&self) -> std::io::Result<()> {
        let config_dir = self.config_dir();
        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(config_path, content)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = VaultConfig::default();
        assert!(config.watch_enabled);
        assert!(config.use_trash);
        assert!(!config.ignore_patterns.is_empty());
    }

    #[test]
    fn test_config_builder() {
        let config = VaultConfig::new("/test/vault")
            .with_name("My Vault")
            .with_watch(false)
            .with_trash(false);

        assert_eq!(config.name, "My Vault");
        assert!(!config.watch_enabled);
        assert!(!config.use_trash);
    }

    #[test]
    fn test_should_ignore() {
        let config = VaultConfig::default();

        assert!(config.should_ignore(std::path::Path::new("/vault/.git/config")));
        assert!(config.should_ignore(std::path::Path::new("/vault/.obsidian/plugins")));
        assert!(!config.should_ignore(std::path::Path::new("/vault/notes/test.md")));
    }

    #[test]
    fn test_is_markdown() {
        let config = VaultConfig::default();

        assert!(config.is_markdown_file(std::path::Path::new("test.md")));
        assert!(config.is_markdown_file(std::path::Path::new("test.markdown")));
        assert!(!config.is_markdown_file(std::path::Path::new("test.txt")));
        assert!(!config.is_markdown_file(std::path::Path::new("test.png")));
    }
}
