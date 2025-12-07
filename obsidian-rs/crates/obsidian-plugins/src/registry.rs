//! Plugin registry and lifecycle management

use crate::api::PluginContext;
use crate::error::{PluginError, PluginResult};
use crate::events::{Event, EventEmitter};
use crate::loader::{DiscoveredPlugin, PluginLoader};
use crate::manifest::PluginManifest;
use crate::settings::PluginSettings;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Plugin state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Plugin is discovered but not loaded
    Discovered,
    /// Plugin is loading
    Loading,
    /// Plugin is loaded and running
    Loaded,
    /// Plugin failed to load
    Failed,
    /// Plugin is disabled
    Disabled,
}

/// A registered plugin
pub struct Plugin {
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Plugin directory
    pub directory: PathBuf,
    /// Current state
    pub state: PluginState,
    /// Plugin context
    pub context: Option<PluginContext>,
    /// Plugin settings
    pub settings: Option<PluginSettings>,
    /// Error message (if failed)
    pub error: Option<String>,
}

impl Plugin {
    /// Create from discovered plugin
    fn from_discovered(discovered: DiscoveredPlugin) -> Self {
        Self {
            manifest: discovered.manifest,
            directory: discovered.directory,
            state: PluginState::Discovered,
            context: None,
            settings: None,
            error: None,
        }
    }

    /// Check if plugin is loaded
    pub fn is_loaded(&self) -> bool {
        self.state == PluginState::Loaded
    }

    /// Check if plugin is enabled
    pub fn is_enabled(&self) -> bool {
        self.state != PluginState::Disabled
    }
}

/// Plugin registry managing all plugins
pub struct PluginRegistry {
    /// Registered plugins
    plugins: RwLock<HashMap<String, Plugin>>,
    /// Plugin loader
    loader: RwLock<PluginLoader>,
    /// Event emitter
    emitter: Arc<EventEmitter>,
    /// Data directory for plugin settings
    data_dir: PathBuf,
    /// Enabled plugins list
    enabled_plugins: RwLock<Vec<String>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        let data_dir = data_dir.into();

        Self {
            plugins: RwLock::new(HashMap::new()),
            loader: RwLock::new(PluginLoader::new()),
            emitter: Arc::new(EventEmitter::new()),
            data_dir,
            enabled_plugins: RwLock::new(Vec::new()),
        }
    }

    /// Get the event emitter
    pub fn emitter(&self) -> &Arc<EventEmitter> {
        &self.emitter
    }

    /// Add a plugin directory
    pub fn add_plugin_directory(&self, path: impl Into<PathBuf>) {
        self.loader.write().add_directory(path);
    }

    /// Discover all plugins
    pub fn discover(&self) -> PluginResult<Vec<String>> {
        info!("Discovering plugins");

        let discovered = self.loader.write().discover()?;
        let mut plugins = self.plugins.write();
        let mut ids = Vec::new();

        for disc in discovered {
            let id = disc.manifest.id.clone();
            plugins.insert(id.clone(), Plugin::from_discovered(disc));
            ids.push(id);
        }

        Ok(ids)
    }

    /// Load enabled plugins
    pub fn load_enabled(&self) -> PluginResult<()> {
        let enabled = self.enabled_plugins.read().clone();

        for id in enabled {
            if let Err(e) = self.load_plugin(&id) {
                error!("Failed to load plugin {}: {}", id, e);
            }
        }

        Ok(())
    }

    /// Load a specific plugin
    pub fn load_plugin(&self, id: &str) -> PluginResult<()> {
        info!("Loading plugin: {}", id);

        let mut plugins = self.plugins.write();
        let plugin = plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;

        if plugin.state == PluginState::Loaded {
            return Err(PluginError::AlreadyLoaded(id.to_string()));
        }

        plugin.state = PluginState::Loading;

        // Create plugin context
        let context = PluginContext::new(id, self.emitter.clone());
        plugin.context = Some(context);

        // Create plugin settings
        let settings = PluginSettings::new(id, &self.data_dir);
        if let Err(e) = settings.load() {
            warn!("Failed to load settings for {}: {}", id, e);
        }
        plugin.settings = Some(settings);

        // Mark as loaded
        plugin.state = PluginState::Loaded;

        // Emit event
        drop(plugins);
        self.emitter.emit(&Event::PluginLoaded { id: id.to_string() });

        Ok(())
    }

    /// Unload a plugin
    pub fn unload_plugin(&self, id: &str) -> PluginResult<()> {
        info!("Unloading plugin: {}", id);

        let mut plugins = self.plugins.write();
        let plugin = plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;

        if plugin.state != PluginState::Loaded {
            return Ok(());
        }

        // Save settings
        if let Some(ref settings) = plugin.settings {
            if let Err(e) = settings.save() {
                warn!("Failed to save settings for {}: {}", id, e);
            }
        }

        // Clean up
        plugin.context = None;
        plugin.settings = None;
        plugin.state = PluginState::Discovered;

        // Emit event
        drop(plugins);
        self.emitter.emit(&Event::PluginUnloaded { id: id.to_string() });

        Ok(())
    }

    /// Enable a plugin
    pub fn enable_plugin(&self, id: &str) -> PluginResult<()> {
        {
            let plugins = self.plugins.read();
            if !plugins.contains_key(id) {
                return Err(PluginError::NotFound(id.to_string()));
            }
        }

        {
            let mut enabled = self.enabled_plugins.write();
            if !enabled.contains(&id.to_string()) {
                enabled.push(id.to_string());
            }
        }

        self.load_plugin(id)
    }

    /// Disable a plugin
    pub fn disable_plugin(&self, id: &str) -> PluginResult<()> {
        self.unload_plugin(id)?;

        {
            let mut enabled = self.enabled_plugins.write();
            enabled.retain(|p| p != id);
        }

        {
            let mut plugins = self.plugins.write();
            if let Some(plugin) = plugins.get_mut(id) {
                plugin.state = PluginState::Disabled;
            }
        }

        Ok(())
    }

    /// Get a plugin by ID
    pub fn get(&self, id: &str) -> Option<PluginInfo> {
        let plugins = self.plugins.read();
        plugins.get(id).map(|p| PluginInfo {
            manifest: p.manifest.clone(),
            state: p.state,
            directory: p.directory.clone(),
            error: p.error.clone(),
        })
    }

    /// Get all plugins
    pub fn all(&self) -> Vec<PluginInfo> {
        let plugins = self.plugins.read();
        plugins
            .values()
            .map(|p| PluginInfo {
                manifest: p.manifest.clone(),
                state: p.state,
                directory: p.directory.clone(),
                error: p.error.clone(),
            })
            .collect()
    }

    /// Get loaded plugins
    pub fn loaded(&self) -> Vec<PluginInfo> {
        self.all()
            .into_iter()
            .filter(|p| p.state == PluginState::Loaded)
            .collect()
    }

    /// Get enabled plugin IDs
    pub fn enabled_ids(&self) -> Vec<String> {
        self.enabled_plugins.read().clone()
    }

    /// Set enabled plugins
    pub fn set_enabled(&self, ids: Vec<String>) {
        *self.enabled_plugins.write() = ids;
    }

    /// Get plugin context
    pub fn context(&self, id: &str) -> Option<PluginContext> {
        // Note: This is a simplified version - in production,
        // we'd need proper lifetime management
        None
    }

    /// Get plugin settings
    pub fn settings(&self, id: &str) -> Option<HashMap<String, serde_json::Value>> {
        let plugins = self.plugins.read();
        plugins
            .get(id)
            .and_then(|p| p.settings.as_ref())
            .map(|s| s.all_values())
    }
}

/// Public plugin information
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Current state
    pub state: PluginState,
    /// Plugin directory
    pub directory: PathBuf,
    /// Error message (if any)
    pub error: Option<String>,
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
    }

    #[test]
    fn test_plugin_registry() {
        let plugins_dir = tempdir().unwrap();
        let data_dir = tempdir().unwrap();

        create_test_plugin(plugins_dir.path(), "test-plugin");

        let registry = PluginRegistry::new(data_dir.path());
        registry.add_plugin_directory(plugins_dir.path());

        // Discover
        let ids = registry.discover().unwrap();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&"test-plugin".to_string()));

        // Load
        registry.load_plugin("test-plugin").unwrap();
        let plugin = registry.get("test-plugin").unwrap();
        assert_eq!(plugin.state, PluginState::Loaded);

        // Unload
        registry.unload_plugin("test-plugin").unwrap();
        let plugin = registry.get("test-plugin").unwrap();
        assert_eq!(plugin.state, PluginState::Discovered);
    }

    #[test]
    fn test_enable_disable() {
        let plugins_dir = tempdir().unwrap();
        let data_dir = tempdir().unwrap();

        create_test_plugin(plugins_dir.path(), "test-plugin");

        let registry = PluginRegistry::new(data_dir.path());
        registry.add_plugin_directory(plugins_dir.path());
        registry.discover().unwrap();

        // Enable
        registry.enable_plugin("test-plugin").unwrap();
        assert!(registry.enabled_ids().contains(&"test-plugin".to_string()));
        assert_eq!(registry.get("test-plugin").unwrap().state, PluginState::Loaded);

        // Disable
        registry.disable_plugin("test-plugin").unwrap();
        assert!(!registry.enabled_ids().contains(&"test-plugin".to_string()));
        assert_eq!(registry.get("test-plugin").unwrap().state, PluginState::Disabled);
    }
}
