//! Plugin management commands

use crate::state::AppState;
use obsidian_plugins::PluginState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::debug;

/// Plugin information for frontend
#[derive(Debug, Serialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub state: String,
    pub is_enabled: bool,
}

/// List all plugins
#[tauri::command]
pub async fn list_plugins(state: State<'_, Arc<AppState>>) -> Result<Vec<PluginInfo>, String> {
    debug!("Listing plugins");

    let plugins = state.plugins.read();
    let registry = plugins
        .as_ref()
        .ok_or_else(|| "No vault open".to_string())?;

    let enabled = registry.enabled_ids();

    Ok(registry
        .all()
        .into_iter()
        .map(|p| {
            let state_str = match p.state {
                PluginState::Discovered => "discovered",
                PluginState::Loading => "loading",
                PluginState::Loaded => "loaded",
                PluginState::Failed => "failed",
                PluginState::Disabled => "disabled",
            };

            PluginInfo {
                id: p.manifest.id.clone(),
                name: p.manifest.name.clone(),
                version: p.manifest.version.clone(),
                description: p.manifest.description.clone(),
                author: p.manifest.author.clone(),
                state: state_str.to_string(),
                is_enabled: enabled.contains(&p.manifest.id),
            }
        })
        .collect())
}

/// Enable a plugin
#[tauri::command]
pub async fn enable_plugin(
    id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    debug!("Enabling plugin: {}", id);

    let plugins = state.plugins.read();
    let registry = plugins
        .as_ref()
        .ok_or_else(|| "No vault open".to_string())?;

    registry
        .enable_plugin(&id)
        .map_err(|e| format!("Failed to enable plugin: {}", e))?;

    Ok(())
}

/// Disable a plugin
#[tauri::command]
pub async fn disable_plugin(
    id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    debug!("Disabling plugin: {}", id);

    let plugins = state.plugins.read();
    let registry = plugins
        .as_ref()
        .ok_or_else(|| "No vault open".to_string())?;

    registry
        .disable_plugin(&id)
        .map_err(|e| format!("Failed to disable plugin: {}", e))?;

    Ok(())
}
