//! Note management commands

use crate::state::AppState;
use obsidian_core::note::Note;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tracing::{debug, error};

/// Note information for frontend
#[derive(Debug, Serialize)]
pub struct NoteInfo {
    pub path: String,
    pub basename: String,
    pub content: String,
    pub frontmatter: Option<FrontmatterInfo>,
    pub modified: i64,
}

/// Frontmatter information
#[derive(Debug, Serialize)]
pub struct FrontmatterInfo {
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
}

impl From<&Note> for NoteInfo {
    fn from(note: &Note) -> Self {
        Self {
            path: note.path.clone(),
            basename: note.basename.clone(),
            content: note.content.clone(),
            frontmatter: Some(FrontmatterInfo {
                title: note.frontmatter.title.clone(),
                tags: note.frontmatter.tags.clone(),
                aliases: note.frontmatter.aliases.clone(),
            }),
            modified: note.stat.mtime,
        }
    }
}

/// Read a note
#[tauri::command]
pub async fn read_note(
    path: String,
    state: State<'_, Arc<AppState>>,
) -> Result<NoteInfo, String> {
    debug!("Reading note: {}", path);

    let vault = state.vault.read();
    let vault = vault
        .as_ref()
        .ok_or_else(|| "No vault open".to_string())?;

    let note = vault
        .read_note(&PathBuf::from(&path))
        .map_err(|e| format!("Failed to read note: {}", e))?;

    // Update active file
    state.set_active_file(Some(PathBuf::from(&path)));

    Ok(NoteInfo::from(&note))
}

/// Write a note
#[tauri::command]
pub async fn write_note(
    path: String,
    content: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    debug!("Writing note: {}", path);

    let vault = state.vault.read();
    let vault = vault
        .as_ref()
        .ok_or_else(|| "No vault open".to_string())?;

    vault
        .write_note(&PathBuf::from(&path), &content)
        .map_err(|e| format!("Failed to write note: {}", e))?;

    Ok(())
}

/// Create a new note
#[tauri::command]
pub async fn create_note(
    path: String,
    content: Option<String>,
    state: State<'_, Arc<AppState>>,
) -> Result<NoteInfo, String> {
    debug!("Creating note: {}", path);

    let vault = state.vault.read();
    let vault = vault
        .as_ref()
        .ok_or_else(|| "No vault open".to_string())?;

    let content = content.unwrap_or_default();

    let note = vault
        .create_note(&PathBuf::from(&path), &content)
        .map_err(|e| format!("Failed to create note: {}", e))?;

    // Update active file
    state.set_active_file(Some(PathBuf::from(&path)));

    Ok(NoteInfo::from(&note))
}

/// Delete a note
#[tauri::command]
pub async fn delete_note(
    path: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    debug!("Deleting note: {}", path);

    let vault = state.vault.read();
    let vault = vault
        .as_ref()
        .ok_or_else(|| "No vault open".to_string())?;

    vault
        .delete_note(&PathBuf::from(&path))
        .map_err(|e| format!("Failed to delete note: {}", e))?;

    // Close file if open
    state.close_file(&PathBuf::from(&path));

    Ok(())
}

/// Rename a note
#[tauri::command]
pub async fn rename_note(
    old_path: String,
    new_path: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    debug!("Renaming note: {} -> {}", old_path, new_path);

    let vault = state.vault.read();
    let vault = vault
        .as_ref()
        .ok_or_else(|| "No vault open".to_string())?;

    vault
        .rename_note(&PathBuf::from(&old_path), &PathBuf::from(&new_path))
        .map_err(|e| format!("Failed to rename note: {}", e))?;

    // Update open files
    {
        let mut open_files = state.open_files.write();
        for file in open_files.iter_mut() {
            if *file == PathBuf::from(&old_path) {
                *file = PathBuf::from(&new_path);
            }
        }
    }

    // Update active file
    {
        let mut active = state.active_file.write();
        if *active == Some(PathBuf::from(&old_path)) {
            *active = Some(PathBuf::from(&new_path));
        }
    }

    Ok(())
}

/// List note item for tree view
#[derive(Debug, Serialize)]
pub struct NoteListItem {
    pub path: String,
    pub name: String,
    pub is_folder: bool,
}

/// List all notes
#[tauri::command]
pub async fn list_notes(
    folder: Option<String>,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<NoteListItem>, String> {
    let vault = state.vault.read();
    let vault = vault
        .as_ref()
        .ok_or_else(|| "No vault open".to_string())?;

    let files = vault.list_markdown_files();

    let items: Vec<NoteListItem> = files
        .into_iter()
        .filter_map(|path| {
            // Filter by folder if specified
            if let Some(ref folder) = folder {
                let folder_path = PathBuf::from(folder);
                if !path.starts_with(&folder_path) {
                    return None;
                }
            }

            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            Some(NoteListItem {
                path: path.to_string_lossy().to_string(),
                name,
                is_folder: false,
            })
        })
        .collect();

    Ok(items)
}

/// File tree item for hierarchical view
#[derive(Debug, Serialize)]
pub struct FileTreeItem {
    pub path: String,
    pub name: String,
    pub is_folder: bool,
    pub children: Option<Vec<FileTreeItem>>,
}

/// Get file tree
#[tauri::command]
pub async fn get_file_tree(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<FileTreeItem>, String> {
    let vault = state.vault.read();
    let vault = vault
        .as_ref()
        .ok_or_else(|| "No vault open".to_string())?;

    let vault_path = vault.path();
    let files = vault.list_markdown_files();

    // Build tree structure
    let mut root_items: Vec<FileTreeItem> = Vec::new();
    let mut folders: std::collections::HashMap<String, Vec<FileTreeItem>> = std::collections::HashMap::new();

    for path in files {
        let rel_path = path.strip_prefix(vault_path).unwrap_or(&path);
        let path_str = rel_path.to_string_lossy().to_string();

        let parts: Vec<&str> = path_str.split('/').collect();
        let name = parts.last().unwrap_or(&"").to_string();

        if parts.len() == 1 {
            // Root level file
            root_items.push(FileTreeItem {
                path: path.to_string_lossy().to_string(),
                name: name.replace(".md", ""),
                is_folder: false,
                children: None,
            });
        } else {
            // File in subfolder
            let folder_path = parts[..parts.len() - 1].join("/");
            let entry = folders.entry(folder_path).or_insert_with(Vec::new);
            entry.push(FileTreeItem {
                path: path.to_string_lossy().to_string(),
                name: name.replace(".md", ""),
                is_folder: false,
                children: None,
            });
        }
    }

    // Convert folders to tree items
    let mut folder_items: Vec<FileTreeItem> = folders
        .into_iter()
        .map(|(folder_path, children)| {
            let name = folder_path.split('/').last().unwrap_or(&folder_path).to_string();
            FileTreeItem {
                path: vault_path.join(&folder_path).to_string_lossy().to_string(),
                name,
                is_folder: true,
                children: Some(children),
            }
        })
        .collect();

    // Sort folders first, then files
    folder_items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    root_items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Combine: folders first, then files
    folder_items.extend(root_items);

    Ok(folder_items)
}
