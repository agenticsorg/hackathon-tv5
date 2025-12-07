//! Vault management and file operations

use crate::cache::MetadataCache;
use crate::config::VaultConfig;
use crate::database::Database;
use crate::error::{StorageError, StorageResult};
use crate::watcher::{FileEvent, FileEventKind, VaultWatcher};
use obsidian_core::note::Note;
use obsidian_core::vault::{TAbstractFile, TFile, TFolder};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// A vault containing notes and attachments
pub struct Vault {
    /// Vault configuration
    config: Arc<VaultConfig>,
    /// Persistent database
    database: Arc<Database>,
    /// Metadata cache
    cache: MetadataCache,
    /// File watcher
    watcher: Option<VaultWatcher>,
    /// File tree
    files: HashMap<PathBuf, TAbstractFile>,
    /// Root folder
    root: TFolder,
}

impl Vault {
    /// Open or create a vault
    pub fn open(path: impl Into<PathBuf>) -> StorageResult<Self> {
        let path = path.into();

        if !path.exists() {
            return Err(StorageError::DirectoryNotFound(path));
        }

        info!("Opening vault at {:?}", path);

        // Load or create config
        let config = Arc::new(VaultConfig::load(&path).unwrap_or_else(|_| VaultConfig::new(&path)));

        // Ensure config directory exists
        std::fs::create_dir_all(config.config_dir())?;

        // Open database
        let database = Arc::new(Database::open(config.database_path())?);

        // Create cache
        let cache = MetadataCache::new(&path, database.clone());

        // Load cached metadata
        cache.load_from_db()?;

        // Create watcher if enabled
        let watcher = if config.watch_enabled {
            match VaultWatcher::new(config.clone()) {
                Ok(w) => Some(w),
                Err(e) => {
                    warn!("Failed to create watcher: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Create root folder
        let root = TFolder::root();

        let mut vault = Self {
            config,
            database,
            cache,
            watcher,
            files: HashMap::new(),
            root,
        };

        // Scan the vault
        vault.scan()?;

        Ok(vault)
    }

    /// Create a new vault at the given path
    pub fn create(path: impl Into<PathBuf>) -> StorageResult<Self> {
        let path = path.into();

        if path.exists() {
            if std::fs::read_dir(&path)?.next().is_some() {
                return Err(StorageError::FileExists(path));
            }
        } else {
            std::fs::create_dir_all(&path)?;
        }

        info!("Creating new vault at {:?}", path);

        let config = VaultConfig::new(&path);
        config.save()?;

        Self::open(path)
    }

    /// Get vault configuration
    pub fn config(&self) -> &VaultConfig {
        &self.config
    }

    /// Get vault path
    pub fn path(&self) -> &Path {
        &self.config.path
    }

    /// Get vault name
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Get the metadata cache
    pub fn cache(&self) -> &MetadataCache {
        &self.cache
    }

    /// Get the database
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// Get the root folder
    pub fn root(&self) -> &TFolder {
        &self.root
    }

    /// Start the file watcher
    pub fn start_watching(&mut self) -> StorageResult<()> {
        if let Some(ref mut watcher) = self.watcher {
            watcher.start()?;
        }
        Ok(())
    }

    /// Stop the file watcher
    pub fn stop_watching(&mut self) -> StorageResult<()> {
        if let Some(ref mut watcher) = self.watcher {
            watcher.stop()?;
        }
        Ok(())
    }

    /// Subscribe to file events
    pub fn subscribe(&self) -> Option<broadcast::Receiver<FileEvent>> {
        self.watcher.as_ref().map(|w| w.subscribe())
    }

    /// Convert absolute path to relative
    pub fn relative_path(&self, path: &Path) -> StorageResult<PathBuf> {
        path.strip_prefix(&self.config.path)
            .map(|p| p.to_path_buf())
            .map_err(|_| StorageError::PathOutsideVault(path.to_path_buf()))
    }

    /// Convert relative path to absolute
    pub fn absolute_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.config.path.join(path)
        }
    }

    /// Scan the vault for files
    pub fn scan(&mut self) -> StorageResult<()> {
        info!("Scanning vault");

        self.files.clear();
        let mut count = 0;

        for entry in WalkDir::new(&self.config.path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip ignored paths
            if self.config.should_ignore(path) {
                continue;
            }

            let rel_path = self.relative_path(path)?;

            if entry.file_type().is_file() {
                let rel_path_str = rel_path.to_string_lossy().to_string();
                let file = TFile::new(&rel_path_str);

                self.files.insert(path.to_path_buf(), TAbstractFile::File(file));

                // Cache markdown files
                if self.config.is_markdown_file(path) {
                    if let Err(e) = self.cache.get_or_parse(path) {
                        warn!("Failed to cache {:?}: {}", path, e);
                    }
                }

                count += 1;
            } else if entry.file_type().is_dir() {
                let rel_path_str = rel_path.to_string_lossy().to_string();
                let folder = TFolder::new(&rel_path_str);

                self.files
                    .insert(path.to_path_buf(), TAbstractFile::Folder(folder));
            }
        }

        info!("Scanned {} files", count);
        Ok(())
    }

    /// Get a file by path
    pub fn get_file(&self, path: &Path) -> Option<&TFile> {
        let abs_path = self.absolute_path(path);
        match self.files.get(&abs_path)? {
            TAbstractFile::File(f) => Some(f),
            _ => None,
        }
    }

    /// Get a folder by path
    pub fn get_folder(&self, path: &Path) -> Option<&TFolder> {
        let abs_path = self.absolute_path(path);
        match self.files.get(&abs_path)? {
            TAbstractFile::Folder(f) => Some(f),
            _ => None,
        }
    }

    /// List all markdown files
    pub fn list_markdown_files(&self) -> Vec<PathBuf> {
        self.files
            .iter()
            .filter_map(|(path, item)| {
                if let TAbstractFile::File(_) = item {
                    if self.config.is_markdown_file(path) {
                        return Some(path.clone());
                    }
                }
                None
            })
            .collect()
    }

    /// List files in a directory
    pub fn list_directory(&self, path: &Path) -> Vec<&TAbstractFile> {
        let dir_path = self.absolute_path(path);

        self.files
            .iter()
            .filter_map(|(file_path, item)| {
                if let Some(parent) = file_path.parent() {
                    if parent == dir_path {
                        return Some(item);
                    }
                }
                None
            })
            .collect()
    }

    /// Read a note
    pub fn read_note(&self, path: &Path) -> StorageResult<Note> {
        let abs_path = self.absolute_path(path);

        if !abs_path.exists() {
            return Err(StorageError::FileNotFound(abs_path));
        }

        let cache = self.cache.get_or_parse(&abs_path)?;
        self.cache.to_note(&abs_path, &cache)
    }

    /// Write a note
    pub fn write_note(&self, path: &Path, content: &str) -> StorageResult<()> {
        let abs_path = self.absolute_path(path);

        // Ensure parent directory exists
        if let Some(parent) = abs_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        debug!("Writing note: {:?}", abs_path);
        std::fs::write(&abs_path, content)?;

        // Update cache
        self.cache.update(&abs_path)?;

        Ok(())
    }

    /// Create a new note
    pub fn create_note(&self, path: &Path, content: &str) -> StorageResult<Note> {
        let abs_path = self.absolute_path(path);

        if abs_path.exists() {
            return Err(StorageError::FileExists(abs_path));
        }

        self.write_note(path, content)?;
        self.read_note(path)
    }

    /// Delete a note
    pub fn delete_note(&self, path: &Path) -> StorageResult<()> {
        let abs_path = self.absolute_path(path);

        if !abs_path.exists() {
            return Err(StorageError::FileNotFound(abs_path));
        }

        debug!("Deleting note: {:?}", abs_path);

        // Move to trash or delete permanently
        if self.config.use_trash {
            let trash_dir = self.config.trash_dir();
            std::fs::create_dir_all(&trash_dir)?;

            let trash_path = trash_dir.join(
                abs_path
                    .file_name()
                    .ok_or_else(|| StorageError::InvalidPath(abs_path.to_string_lossy().to_string()))?,
            );

            std::fs::rename(&abs_path, &trash_path)?;
        } else {
            std::fs::remove_file(&abs_path)?;
        }

        // Invalidate cache
        self.cache.invalidate(&abs_path)?;

        Ok(())
    }

    /// Rename/move a note
    pub fn rename_note(&self, old_path: &Path, new_path: &Path) -> StorageResult<()> {
        let old_abs = self.absolute_path(old_path);
        let new_abs = self.absolute_path(new_path);

        if !old_abs.exists() {
            return Err(StorageError::FileNotFound(old_abs));
        }

        if new_abs.exists() {
            return Err(StorageError::FileExists(new_abs));
        }

        // Ensure parent directory exists
        if let Some(parent) = new_abs.parent() {
            std::fs::create_dir_all(parent)?;
        }

        debug!("Renaming note: {:?} -> {:?}", old_abs, new_abs);
        std::fs::rename(&old_abs, &new_abs)?;

        // Update cache
        self.cache.rename(&old_abs, &new_abs)?;

        Ok(())
    }

    /// Search for notes by name
    pub fn search_by_name(&self, query: &str) -> Vec<PathBuf> {
        let query_lower = query.to_lowercase();

        self.list_markdown_files()
            .into_iter()
            .filter(|path| {
                path.file_stem()
                    .map(|s| s.to_string_lossy().to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get backlinks for a note
    pub fn get_backlinks(&self, path: &Path) -> StorageResult<Vec<String>> {
        let rel_path = self.relative_path(path)?;
        self.cache.get_backlinks(&rel_path.to_string_lossy())
    }

    /// Get all tags in the vault
    pub fn get_all_tags(&self) -> StorageResult<Vec<String>> {
        self.cache.get_all_tags()
    }

    /// Get notes with a specific tag
    pub fn get_notes_with_tag(&self, tag: &str) -> StorageResult<Vec<String>> {
        self.cache.get_files_with_tag(tag)
    }

    /// Create a folder
    pub fn create_folder(&self, path: &Path) -> StorageResult<()> {
        let abs_path = self.absolute_path(path);

        if abs_path.exists() {
            return Err(StorageError::FileExists(abs_path));
        }

        debug!("Creating folder: {:?}", abs_path);
        std::fs::create_dir_all(&abs_path)?;

        Ok(())
    }

    /// Delete a folder
    pub fn delete_folder(&self, path: &Path, recursive: bool) -> StorageResult<()> {
        let abs_path = self.absolute_path(path);

        if !abs_path.exists() {
            return Err(StorageError::DirectoryNotFound(abs_path));
        }

        debug!("Deleting folder: {:?}", abs_path);

        if recursive {
            // Invalidate all cached files in the folder
            for file_path in self.list_markdown_files() {
                if file_path.starts_with(&abs_path) {
                    self.cache.invalidate(&file_path)?;
                }
            }

            if self.config.use_trash {
                let trash_dir = self.config.trash_dir();
                std::fs::create_dir_all(&trash_dir)?;

                let trash_path = trash_dir.join(
                    abs_path.file_name().ok_or_else(|| {
                        StorageError::InvalidPath(abs_path.to_string_lossy().to_string())
                    })?,
                );

                // Move recursively to trash
                Self::move_dir_recursive(&abs_path, &trash_path)?;
            } else {
                std::fs::remove_dir_all(&abs_path)?;
            }
        } else {
            std::fs::remove_dir(&abs_path)?;
        }

        Ok(())
    }

    /// Move directory recursively
    fn move_dir_recursive(from: &Path, to: &Path) -> StorageResult<()> {
        std::fs::create_dir_all(to)?;

        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            let from_path = entry.path();
            let to_path = to.join(entry.file_name());

            if from_path.is_dir() {
                Self::move_dir_recursive(&from_path, &to_path)?;
            } else {
                std::fs::rename(&from_path, &to_path)?;
            }
        }

        std::fs::remove_dir(from)?;
        Ok(())
    }

    /// Handle a file event
    pub fn handle_event(&mut self, event: &FileEvent) -> StorageResult<()> {
        debug!("Handling event: {:?}", event);

        match event.kind {
            FileEventKind::Created => {
                if self.config.is_markdown_file(&event.path) {
                    self.cache.parse_and_cache(&event.path)?;
                }

                // Add to files map
                if event.path.is_file() {
                    let rel_path = self.relative_path(&event.path)?
                        .to_string_lossy()
                        .to_string();
                    let file = TFile::new(&rel_path);
                    self.files
                        .insert(event.path.clone(), TAbstractFile::File(file));
                }
            }
            FileEventKind::Modified => {
                if self.config.is_markdown_file(&event.path) {
                    self.cache.update(&event.path)?;
                }
            }
            FileEventKind::Deleted => {
                self.cache.invalidate(&event.path)?;
                self.files.remove(&event.path);
            }
            FileEventKind::Renamed => {
                if let Some(ref new_path) = event.new_path {
                    self.cache.rename(&event.path, new_path)?;

                    // Update files map
                    if let Some(item) = self.files.remove(&event.path) {
                        self.files.insert(new_path.clone(), item);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get vault statistics
    pub fn stats(&self) -> VaultStats {
        let markdown_count = self.list_markdown_files().len();
        let total_count = self.files.len();
        let cached_count = self.cache.len();

        VaultStats {
            total_files: total_count,
            markdown_files: markdown_count,
            cached_files: cached_count,
            tags: self.get_all_tags().unwrap_or_default().len(),
        }
    }
}

/// Vault statistics
#[derive(Debug, Clone)]
pub struct VaultStats {
    /// Total number of files
    pub total_files: usize,
    /// Number of markdown files
    pub markdown_files: usize,
    /// Number of cached files
    pub cached_files: usize,
    /// Number of unique tags
    pub tags: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_vault() -> (tempfile::TempDir, Vault) {
        let dir = tempdir().unwrap();
        let vault = Vault::create(dir.path()).unwrap();
        (dir, vault)
    }

    #[test]
    fn test_vault_create() {
        let (dir, vault) = setup_vault();

        assert!(vault.config().config_dir().exists());
        assert!(vault.config().database_path().exists());
    }

    #[test]
    fn test_note_crud() {
        let (dir, vault) = setup_vault();

        // Create
        let note = vault
            .create_note(Path::new("test.md"), "# Test\n\nHello world!")
            .unwrap();

        assert_eq!(note.content, "# Test\n\nHello world!");

        // Read
        let read_note = vault.read_note(Path::new("test.md")).unwrap();
        assert_eq!(read_note.content, "# Test\n\nHello world!");

        // Update
        vault
            .write_note(Path::new("test.md"), "# Updated\n\nNew content")
            .unwrap();
        let updated = vault.read_note(Path::new("test.md")).unwrap();
        assert_eq!(updated.content, "# Updated\n\nNew content");

        // Delete
        vault.delete_note(Path::new("test.md")).unwrap();
        assert!(vault.read_note(Path::new("test.md")).is_err());
    }

    #[test]
    fn test_rename_note() {
        let (dir, vault) = setup_vault();

        vault
            .create_note(Path::new("old.md"), "# Test")
            .unwrap();

        vault
            .rename_note(Path::new("old.md"), Path::new("new.md"))
            .unwrap();

        assert!(vault.read_note(Path::new("old.md")).is_err());
        assert!(vault.read_note(Path::new("new.md")).is_ok());
    }

    #[test]
    fn test_folder_operations() {
        let (dir, vault) = setup_vault();

        // Create folder
        vault.create_folder(Path::new("notes")).unwrap();
        assert!(dir.path().join("notes").exists());

        // Create note in folder
        vault
            .create_note(Path::new("notes/test.md"), "# Test")
            .unwrap();

        // Delete folder
        vault.delete_folder(Path::new("notes"), true).unwrap();
        assert!(!dir.path().join("notes").exists());
    }

    #[test]
    fn test_search_by_name() {
        let (dir, vault) = setup_vault();

        vault.create_note(Path::new("hello.md"), "# Hello").unwrap();
        vault.create_note(Path::new("world.md"), "# World").unwrap();
        vault
            .create_note(Path::new("hello-world.md"), "# Hello World")
            .unwrap();

        // Need to rescan after creating files
        let mut vault = Vault::open(dir.path()).unwrap();

        let results = vault.search_by_name("hello");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_vault_stats() {
        let (dir, vault) = setup_vault();

        vault.create_note(Path::new("note1.md"), "# Note 1 #tag1").unwrap();
        vault.create_note(Path::new("note2.md"), "# Note 2 #tag2").unwrap();

        let mut vault = Vault::open(dir.path()).unwrap();
        let stats = vault.stats();

        assert!(stats.markdown_files >= 2);
    }
}
