//! Vault management commands

use crate::state::{AppState, RecentVault};
use obsidian_graph::KnowledgeGraph;
use obsidian_plugins::PluginRegistry;
use obsidian_search::SearchEngine;
use obsidian_storage::Vault;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tracing::{error, info};

/// Vault information
#[derive(Debug, Serialize)]
pub struct VaultInfo {
    pub name: String,
    pub path: String,
    pub note_count: usize,
    pub tag_count: usize,
}

/// Open a vault
#[tauri::command]
pub async fn open_vault(
    path: String,
    state: State<'_, Arc<AppState>>,
) -> Result<VaultInfo, String> {
    info!("Opening vault: {}", path);

    let path_buf = PathBuf::from(&path);

    // Open vault
    let vault = Vault::open(&path_buf).map_err(|e| {
        error!("Failed to open vault: {}", e);
        format!("Failed to open vault: {}", e)
    })?;

    let name = vault.name().to_string();
    let note_count = vault.list_markdown_files().len();
    let tags = vault.get_all_tags().unwrap_or_default();
    let tag_count = tags.len();

    // Initialize graph
    let graph = KnowledgeGraph::new(&path_buf);

    // Initialize search (simplified - in production would set up index path)
    let search = SearchEngine::new();

    // Initialize plugin registry
    let plugins_dir = path_buf.join(".obsidian").join("plugins");
    let data_dir = path_buf.join(".obsidian-rs");
    let mut registry = PluginRegistry::new(&data_dir);

    if plugins_dir.exists() {
        registry.add_plugin_directory(&plugins_dir);
        if let Err(e) = registry.discover() {
            error!("Failed to discover plugins: {}", e);
        }
    }

    // Store in state
    *state.vault.write() = Some(vault);
    *state.graph.write() = Some(graph);
    *state.search.write() = Some(search);
    *state.plugins.write() = Some(registry);

    // Add to recent vaults
    state.add_recent_vault(&name, &path);

    Ok(VaultInfo {
        name,
        path,
        note_count,
        tag_count,
    })
}

/// Create a new vault
#[tauri::command]
pub async fn create_vault(
    path: String,
    name: Option<String>,
    state: State<'_, Arc<AppState>>,
) -> Result<VaultInfo, String> {
    info!("Creating vault: {}", path);

    let path_buf = PathBuf::from(&path);

    // Create vault
    let vault = Vault::create(&path_buf).map_err(|e| {
        error!("Failed to create vault: {}", e);
        format!("Failed to create vault: {}", e)
    })?;

    let vault_name = name.unwrap_or_else(|| vault.name().to_string());

    // Initialize components
    let graph = KnowledgeGraph::new(&path_buf);
    let search = SearchEngine::new();
    let data_dir = path_buf.join(".obsidian-rs");
    let registry = PluginRegistry::new(&data_dir);

    // Store in state
    *state.vault.write() = Some(vault);
    *state.graph.write() = Some(graph);
    *state.search.write() = Some(search);
    *state.plugins.write() = Some(registry);

    // Add to recent vaults
    state.add_recent_vault(&vault_name, &path);

    Ok(VaultInfo {
        name: vault_name,
        path,
        note_count: 0,
        tag_count: 0,
    })
}

/// Close the current vault
#[tauri::command]
pub async fn close_vault(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    info!("Closing vault");

    *state.vault.write() = None;
    *state.graph.write() = None;
    *state.search.write() = None;
    *state.plugins.write() = None;
    *state.active_file.write() = None;
    state.open_files.write().clear();

    Ok(())
}

/// Get current vault info
#[tauri::command]
pub async fn get_vault_info(state: State<'_, Arc<AppState>>) -> Result<Option<VaultInfo>, String> {
    let vault = state.vault.read();

    match vault.as_ref() {
        Some(v) => {
            let tags = v.get_all_tags().unwrap_or_default();
            Ok(Some(VaultInfo {
                name: v.name().to_string(),
                path: v.path().to_string_lossy().to_string(),
                note_count: v.list_markdown_files().len(),
                tag_count: tags.len(),
            }))
        }
        None => Ok(None),
    }
}

/// List recent vaults
#[tauri::command]
pub async fn list_recent_vaults(state: State<'_, Arc<AppState>>) -> Result<Vec<RecentVault>, String> {
    let settings = state.settings.read();
    Ok(settings.recent_vaults.clone())
}
