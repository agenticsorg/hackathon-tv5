//! Settings commands

use crate::state::{AppSettings, AppState};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::debug;

/// Get application settings
#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<AppSettings, String> {
    debug!("Getting settings");
    Ok(state.settings.read().clone())
}

/// Set application settings
#[tauri::command]
pub async fn set_settings(
    settings: AppSettings,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    debug!("Setting settings");
    *state.settings.write() = settings;
    Ok(())
}
