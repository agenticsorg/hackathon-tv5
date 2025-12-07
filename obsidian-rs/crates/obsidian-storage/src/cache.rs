//! In-memory metadata cache

use crate::database::{hash_content, Database, MetadataEntry};
use crate::error::StorageResult;
use dashmap::DashMap;
use obsidian_core::metadata::CachedMetadata;
use obsidian_core::note::{FileStat, Note};
use obsidian_markdown::MarkdownParser;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info};

/// Cached note entry
#[derive(Debug, Clone)]
pub struct NoteCache {
    /// Note content
    pub content: String,
    /// Parsed metadata
    pub metadata: CachedMetadata,
    /// File stat
    pub stat: FileStat,
    /// Content hash
    pub hash: u64,
}

impl NoteCache {
    /// Create a new cache entry
    pub fn new(content: String, metadata: CachedMetadata, stat: FileStat) -> Self {
        let hash = hash_content(&content);
        Self {
            content,
            metadata,
            stat,
            hash,
        }
    }

    /// Check if content has changed
    pub fn content_changed(&self, new_content: &str) -> bool {
        hash_content(new_content) != self.hash
    }
}

/// In-memory metadata cache with persistent backing
pub struct MetadataCache {
    /// In-memory cache
    cache: DashMap<PathBuf, NoteCache>,
    /// Persistent database
    db: Arc<Database>,
    /// Vault root path
    vault_path: PathBuf,
    /// Markdown parser
    parser: MarkdownParser,
    /// Whether the cache is fully loaded
    loaded: RwLock<bool>,
}

impl MetadataCache {
    /// Create a new cache
    pub fn new(vault_path: impl Into<PathBuf>, db: Arc<Database>) -> Self {
        Self {
            cache: DashMap::new(),
            db,
            vault_path: vault_path.into(),
            parser: MarkdownParser::new(),
            loaded: RwLock::new(false),
        }
    }

    /// Get the vault path
    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }

    /// Load all cached metadata from database
    pub fn load_from_db(&self) -> StorageResult<()> {
        info!("Loading metadata cache from database");

        let paths = self.db.list_metadata_paths()?;
        let mut count = 0;

        for rel_path in paths {
            if let Ok(Some(entry)) = self.db.get_metadata(&rel_path) {
                if let Ok(Some(stat)) = self.db.get_file_stat(&rel_path) {
                    let abs_path = self.vault_path.join(&rel_path);

                    // Check if file still exists and hasn't changed
                    if abs_path.exists() {
                        let note_cache = NoteCache {
                            content: String::new(), // Don't load content by default
                            metadata: entry.metadata,
                            stat,
                            hash: entry.content_hash,
                        };
                        self.cache.insert(abs_path, note_cache);
                        count += 1;
                    } else {
                        // File was deleted, clean up
                        self.db.remove_metadata(&rel_path)?;
                        self.db.remove_file_stat(&rel_path)?;
                    }
                }
            }
        }

        *self.loaded.write() = true;
        info!("Loaded {} cached entries", count);
        Ok(())
    }

    /// Check if cache is loaded
    pub fn is_loaded(&self) -> bool {
        *self.loaded.read()
    }

    /// Get relative path from absolute
    fn relative_path(&self, path: &Path) -> StorageResult<String> {
        path.strip_prefix(&self.vault_path)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|_| crate::error::StorageError::PathOutsideVault(path.to_path_buf()))
    }

    /// Get cached metadata for a file
    pub fn get(&self, path: &Path) -> Option<NoteCache> {
        self.cache.get(path).map(|entry| entry.value().clone())
    }

    /// Get metadata, parsing if not cached
    pub fn get_or_parse(&self, path: &Path) -> StorageResult<NoteCache> {
        // Check in-memory cache
        if let Some(entry) = self.cache.get(path) {
            return Ok(entry.value().clone());
        }

        // Not in cache, parse the file
        self.parse_and_cache(path)
    }

    /// Parse a file and cache its metadata
    pub fn parse_and_cache(&self, path: &Path) -> StorageResult<NoteCache> {
        debug!("Parsing and caching: {:?}", path);

        let content = std::fs::read_to_string(path)?;
        let file_meta = std::fs::metadata(path)?;

        let stat = FileStat::from_metadata(&file_meta);

        // Parse markdown
        let parsed = self.parser.parse(&content);
        let metadata = parsed.to_cached_metadata();

        let cache_entry = NoteCache::new(content, metadata.clone(), stat.clone());

        // Store in memory
        self.cache.insert(path.to_path_buf(), cache_entry.clone());

        // Store in database
        let rel_path = self.relative_path(path)?;
        let db_entry = MetadataEntry {
            path: rel_path.clone(),
            metadata,
            mtime: stat.mtime.timestamp(),
            content_hash: cache_entry.hash,
        };
        self.db.store_metadata(&rel_path, &db_entry)?;
        self.db.store_file_stat(&rel_path, &stat)?;

        // Update indices
        let targets: Vec<String> = parsed.links.iter().map(|l| l.target.clone()).collect();
        self.db.update_links(&rel_path, &targets)?;

        let tags: Vec<String> = parsed.tags.iter().map(|t| t.name.clone()).collect();
        self.db.update_tags(&rel_path, &tags)?;

        Ok(cache_entry)
    }

    /// Invalidate cache for a file
    pub fn invalidate(&self, path: &Path) -> StorageResult<()> {
        debug!("Invalidating cache: {:?}", path);

        self.cache.remove(path);

        if let Ok(rel_path) = self.relative_path(path) {
            self.db.remove_metadata(&rel_path)?;
            self.db.remove_file_stat(&rel_path)?;
            self.db.remove_links_from(&rel_path)?;
            self.db.remove_tags_for(&rel_path)?;
        }

        Ok(())
    }

    /// Update cache for a file
    pub fn update(&self, path: &Path) -> StorageResult<NoteCache> {
        self.invalidate(path)?;
        self.parse_and_cache(path)
    }

    /// Rename a cached entry
    pub fn rename(&self, old_path: &Path, new_path: &Path) -> StorageResult<()> {
        debug!("Renaming cache: {:?} -> {:?}", old_path, new_path);

        // Remove old entry
        if let Some((_, entry)) = self.cache.remove(old_path) {
            // Insert with new path
            self.cache.insert(new_path.to_path_buf(), entry);
        }

        // Update database
        let old_rel = self.relative_path(old_path)?;
        let new_rel = self.relative_path(new_path)?;

        if let Ok(Some(entry)) = self.db.get_metadata(&old_rel) {
            let new_entry = MetadataEntry {
                path: new_rel.clone(),
                ..entry
            };
            self.db.remove_metadata(&old_rel)?;
            self.db.store_metadata(&new_rel, &new_entry)?;
        }

        if let Ok(Some(stat)) = self.db.get_file_stat(&old_rel) {
            self.db.remove_file_stat(&old_rel)?;
            self.db.store_file_stat(&new_rel, &stat)?;
        }

        Ok(())
    }

    /// Get all cached paths
    pub fn paths(&self) -> Vec<PathBuf> {
        self.cache.iter().map(|e| e.key().clone()).collect()
    }

    /// Get number of cached entries
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clear the in-memory cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get all files linking to a target
    pub fn get_backlinks(&self, target: &str) -> StorageResult<Vec<String>> {
        self.db.get_backlinks(target)
    }

    /// Get all files with a tag
    pub fn get_files_with_tag(&self, tag: &str) -> StorageResult<Vec<String>> {
        self.db.get_files_with_tag(tag)
    }

    /// Get all tags
    pub fn get_all_tags(&self) -> StorageResult<Vec<String>> {
        self.db.list_tags()
    }

    /// Convert cached entry to Note
    pub fn to_note(&self, path: &Path, cache: &NoteCache) -> StorageResult<Note> {
        let rel_path = self.relative_path(path)?;

        let mut note = Note::new(&rel_path, path, &cache.content);
        note.frontmatter = cache.metadata.frontmatter.clone();
        note.stat = cache.stat.clone();

        Ok(note)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_cache() -> (tempfile::TempDir, MetadataCache) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join(".obsidian-rs/cache.redb");
        let db = Arc::new(Database::open(&db_path).unwrap());
        let cache = MetadataCache::new(dir.path(), db);
        (dir, cache)
    }

    #[test]
    fn test_cache_parse() {
        let (dir, cache) = setup_cache();

        // Create a test file
        let note_path = dir.path().join("test.md");
        std::fs::write(&note_path, "# Test\n\n[[link]] #tag").unwrap();

        // Parse and cache
        let entry = cache.parse_and_cache(&note_path).unwrap();

        assert!(!entry.content.is_empty());
        assert_eq!(entry.metadata.links.len(), 1);
        assert_eq!(entry.metadata.tags.len(), 1);

        // Should be in cache now
        assert!(cache.get(&note_path).is_some());
    }

    #[test]
    fn test_cache_invalidate() {
        let (dir, cache) = setup_cache();

        let note_path = dir.path().join("test.md");
        std::fs::write(&note_path, "# Test").unwrap();

        cache.parse_and_cache(&note_path).unwrap();
        assert!(cache.get(&note_path).is_some());

        cache.invalidate(&note_path).unwrap();
        assert!(cache.get(&note_path).is_none());
    }

    #[test]
    fn test_cache_rename() {
        let (dir, cache) = setup_cache();

        let old_path = dir.path().join("old.md");
        let new_path = dir.path().join("new.md");

        std::fs::write(&old_path, "# Test").unwrap();
        cache.parse_and_cache(&old_path).unwrap();

        std::fs::rename(&old_path, &new_path).unwrap();
        cache.rename(&old_path, &new_path).unwrap();

        assert!(cache.get(&old_path).is_none());
        assert!(cache.get(&new_path).is_some());
    }

    #[test]
    fn test_content_changed() {
        let entry = NoteCache::new(
            "hello".to_string(),
            CachedMetadata::default(),
            FileStat::default(),
        );

        assert!(!entry.content_changed("hello"));
        assert!(entry.content_changed("world"));
    }
}
