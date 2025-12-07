//! Obsidian RS - A Rust-based Obsidian-compatible note-taking application

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod state;

use state::AppState;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,obsidian_rs=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Obsidian RS v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(Arc::new(AppState::new()))
        .invoke_handler(tauri::generate_handler![
            // Vault commands
            commands::vault::open_vault,
            commands::vault::create_vault,
            commands::vault::close_vault,
            commands::vault::get_vault_info,
            commands::vault::list_recent_vaults,
            // Note commands
            commands::note::read_note,
            commands::note::write_note,
            commands::note::create_note,
            commands::note::delete_note,
            commands::note::rename_note,
            commands::note::list_notes,
            commands::note::get_file_tree,
            // Search commands
            commands::search::search_notes,
            commands::search::search_content,
            commands::search::quick_switch,
            // Graph commands
            commands::graph::get_graph_data,
            commands::graph::get_local_graph,
            commands::graph::get_backlinks,
            // Markdown commands
            commands::markdown::render_markdown,
            commands::markdown::parse_frontmatter,
            // Plugin commands
            commands::plugin::list_plugins,
            commands::plugin::enable_plugin,
            commands::plugin::disable_plugin,
            // Settings commands
            commands::settings::get_settings,
            commands::settings::set_settings,
        ])
        .setup(|app| {
            info!("Application setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
