//! Plugin loading and discovery

use crate::error::PluginResult;
use crate::manifest::PluginManifest;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// Plugin loader for discovering and loading plugins
pub struct PluginLoader {
    /// Plugin directories
    plugin_dirs: Vec<PathBuf>,
    /// Discovered plugins
    discovered: HashMap<String, DiscoveredPlugin>,
}

/// A discovered plugin before loading
#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Plugin directory
    pub directory: PathBuf,
    /// Main file path
    pub main_file: Option<PathBuf>,
    /// Styles file path
    pub styles_file: Option<PathBuf>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new() -> Self {
        Self {
            plugin_dirs: Vec::new(),
            discovered: HashMap::new(),
        }
    }

    /// Add a plugin directory
    pub fn add_directory(&mut self, path: impl Into<PathBuf>) {
        self.plugin_dirs.push(path.into());
    }

    /// Discover all plugins in registered directories
    pub fn discover(&mut self) -> PluginResult<Vec<DiscoveredPlugin>> {
        info!("Discovering plugins");
        self.discovered.clear();

        for dir in &self.plugin_dirs.clone() {
            if !dir.exists() {
                debug!("Plugin directory does not exist: {:?}", dir);
                continue;
            }

            self.discover_in_directory(dir)?;
        }

        info!("Discovered {} plugins", self.discovered.len());
        Ok(self.discovered.values().cloned().collect())
    }

    /// Discover plugins in a single directory
    fn discover_in_directory(&mut self, dir: &Path) -> PluginResult<()> {
        for entry in WalkDir::new(dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }

            match PluginManifest::load(&manifest_path) {
                Ok(manifest) => {
                    let main_file = Self::find_main_file(path);
                    let styles_file = Self::find_styles_file(path);

                    let discovered = DiscoveredPlugin {
                        manifest: manifest.clone(),
                        directory: path.to_path_buf(),
                        main_file,
                        styles_file,
                    };

                    debug!("Discovered plugin: {} v{}", manifest.id, manifest.version);
                    self.discovered.insert(manifest.id.clone(), discovered);
                }
                Err(e) => {
                    warn!("Failed to load manifest at {:?}: {}", manifest_path, e);
                }
            }
        }

        Ok(())
    }

    /// Find the main JavaScript file
    fn find_main_file(plugin_dir: &Path) -> Option<PathBuf> {
        let candidates = ["main.js", "index.js", "plugin.js"];

        for candidate in &candidates {
            let path = plugin_dir.join(candidate);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    /// Find the styles file
    fn find_styles_file(plugin_dir: &Path) -> Option<PathBuf> {
        let candidates = ["styles.css", "style.css", "main.css"];

        for candidate in &candidates {
            let path = plugin_dir.join(candidate);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    /// Get a discovered plugin by ID
    pub fn get(&self, id: &str) -> Option<&DiscoveredPlugin> {
        self.discovered.get(id)
    }

    /// Check if a plugin is discovered
    pub fn has(&self, id: &str) -> bool {
        self.discovered.contains_key(id)
    }

    /// Get all discovered plugins
    pub fn all(&self) -> Vec<&DiscoveredPlugin> {
        self.discovered.values().collect()
    }

    /// Get community plugins directory (default location)
    pub fn community_plugins_dir(vault_path: impl AsRef<Path>) -> PathBuf {
        vault_path.as_ref().join(".obsidian").join("plugins")
    }

    /// Get core plugins directory (default location)
    pub fn core_plugins_dir(app_data: impl AsRef<Path>) -> PathBuf {
        app_data.as_ref().join("plugins")
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_plugin(dir: &Path, id: &str) {
        let plugin_dir = dir.join(id);
        std::fs::create_dir_all(&plugin_dir).unwrap();

        let manifest = serde_json::json!({
            "id": id,
            "name": format!("Test Plugin {}", id),
            "version": "1.0.0",
            "minAppVersion": "1.0.0",
            "description": "A test plugin",
            "author": "Test"
        });

        std::fs::write(
            plugin_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        std::fs::write(plugin_dir.join("main.js"), "// Plugin code").unwrap();
    }

    #[test]
    fn test_plugin_discovery() {
        let dir = tempdir().unwrap();

        create_test_plugin(dir.path(), "plugin-a");
        create_test_plugin(dir.path(), "plugin-b");

        let mut loader = PluginLoader::new();
        loader.add_directory(dir.path());

        let plugins = loader.discover().unwrap();

        assert_eq!(plugins.len(), 2);
        assert!(loader.has("plugin-a"));
        assert!(loader.has("plugin-b"));
    }

    #[test]
    fn test_get_plugin() {
        let dir = tempdir().unwrap();
        create_test_plugin(dir.path(), "test-plugin");

        let mut loader = PluginLoader::new();
        loader.add_directory(dir.path());
        loader.discover().unwrap();

        let plugin = loader.get("test-plugin").unwrap();
        assert_eq!(plugin.manifest.id, "test-plugin");
        assert!(plugin.main_file.is_some());
    }
}
