//! Application state management

use obsidian_graph::KnowledgeGraph;
use obsidian_plugins::PluginRegistry;
use obsidian_search::SearchEngine;
use obsidian_storage::Vault;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Theme (light, dark, system)
    pub theme: String,
    /// Editor font family
    pub editor_font: String,
    /// Editor font size
    pub editor_font_size: u32,
    /// Show line numbers
    pub show_line_numbers: bool,
    /// Spellcheck enabled
    pub spellcheck: bool,
    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval: u32,
    /// Recent vaults
    pub recent_vaults: Vec<RecentVault>,
    /// Custom CSS
    pub custom_css: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            editor_font: "monospace".to_string(),
            editor_font_size: 16,
            show_line_numbers: true,
            spellcheck: true,
            auto_save_interval: 0,
            recent_vaults: Vec::new(),
            custom_css: None,
        }
    }
}

/// Recent vault entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentVault {
    /// Vault name
    pub name: String,
    /// Vault path
    pub path: String,
    /// Last opened timestamp
    pub last_opened: i64,
}

/// Application state
pub struct AppState {
    /// Current vault
    pub vault: RwLock<Option<Vault>>,
    /// Knowledge graph
    pub graph: RwLock<Option<KnowledgeGraph>>,
    /// Search engine
    pub search: RwLock<Option<SearchEngine>>,
    /// Plugin registry
    pub plugins: RwLock<Option<PluginRegistry>>,
    /// Application settings
    pub settings: RwLock<AppSettings>,
    /// Active file path
    pub active_file: RwLock<Option<PathBuf>>,
    /// Open files
    pub open_files: RwLock<Vec<PathBuf>>,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Self {
        Self {
            vault: RwLock::new(None),
            graph: RwLock::new(None),
            search: RwLock::new(None),
            plugins: RwLock::new(None),
            settings: RwLock::new(AppSettings::default()),
            active_file: RwLock::new(None),
            open_files: RwLock::new(Vec::new()),
        }
    }

    /// Check if a vault is open
    pub fn has_vault(&self) -> bool {
        self.vault.read().is_some()
    }

    /// Get vault path
    pub fn vault_path(&self) -> Option<PathBuf> {
        self.vault.read().as_ref().map(|v| v.path().to_path_buf())
    }

    /// Add a recent vault
    pub fn add_recent_vault(&self, name: &str, path: &str) {
        let mut settings = self.settings.write();

        // Remove if already exists
        settings.recent_vaults.retain(|v| v.path != path);

        // Add to front
        settings.recent_vaults.insert(
            0,
            RecentVault {
                name: name.to_string(),
                path: path.to_string(),
                last_opened: chrono::Utc::now().timestamp(),
            },
        );

        // Keep only last 10
        settings.recent_vaults.truncate(10);
    }

    /// Set active file
    pub fn set_active_file(&self, path: Option<PathBuf>) {
        *self.active_file.write() = path.clone();

        // Add to open files if not already there
        if let Some(ref p) = path {
            let mut open_files = self.open_files.write();
            if !open_files.contains(p) {
                open_files.push(p.clone());
            }
        }
    }

    /// Close a file
    pub fn close_file(&self, path: &PathBuf) {
        let mut open_files = self.open_files.write();
        open_files.retain(|p| p != path);

        // Clear active file if it was the closed one
        let mut active = self.active_file.write();
        if active.as_ref() == Some(path) {
            *active = open_files.first().cloned();
        }
    }

    /// Load settings from disk
    pub fn load_settings(&self, app_data_dir: &PathBuf) -> Result<(), String> {
        let settings_path = app_data_dir.join("settings.json");

        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)
                .map_err(|e| format!("Failed to read settings: {}", e))?;

            let loaded: AppSettings = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse settings: {}", e))?;

            *self.settings.write() = loaded;
        }

        Ok(())
    }

    /// Save settings to disk
    pub fn save_settings(&self, app_data_dir: &PathBuf) -> Result<(), String> {
        std::fs::create_dir_all(app_data_dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;

        let settings_path = app_data_dir.join("settings.json");
        let settings = self.settings.read();

        let content = serde_json::to_string_pretty(&*settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        std::fs::write(&settings_path, content)
            .map_err(|e| format!("Failed to write settings: {}", e))?;

        Ok(())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
