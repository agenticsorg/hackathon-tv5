//! Persistent database for metadata caching

use crate::error::StorageResult;
use obsidian_core::metadata::CachedMetadata;
use obsidian_core::note::FileStat;
use redb::{Database as RedbDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Table for storing note metadata
const METADATA_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("metadata");

/// Table for storing file stats
const FILE_STATS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("file_stats");

/// Table for storing link index (target -> sources)
const LINKS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("links");

/// Table for storing tag index (tag -> files)
const TAGS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("tags");

/// Stored metadata entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataEntry {
    /// File path (relative to vault)
    pub path: String,
    /// Cached metadata
    pub metadata: CachedMetadata,
    /// File modification time when cached
    pub mtime: i64,
    /// Content hash for change detection
    pub content_hash: u64,
}

/// Database for persistent storage
pub struct Database {
    db: RedbDatabase,
    path: PathBuf,
}

impl Database {
    /// Open or create a database
    pub fn open(path: impl AsRef<Path>) -> StorageResult<Self> {
        let path = path.as_ref().to_path_buf();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        info!("Opening database at {:?}", path);
        let db = RedbDatabase::create(&path)?;

        // Initialize tables
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(METADATA_TABLE)?;
            let _ = write_txn.open_table(FILE_STATS_TABLE)?;
            let _ = write_txn.open_table(LINKS_TABLE)?;
            let _ = write_txn.open_table(TAGS_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self { db, path })
    }

    /// Get the database path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Store metadata for a note
    pub fn store_metadata(&self, path: &str, entry: &MetadataEntry) -> StorageResult<()> {
        let data = bincode::serialize(entry)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(METADATA_TABLE)?;
            table.insert(path, data.as_slice())?;
        }
        write_txn.commit()?;

        debug!("Stored metadata for {}", path);
        Ok(())
    }

    /// Get metadata for a note
    pub fn get_metadata(&self, path: &str) -> StorageResult<Option<MetadataEntry>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(METADATA_TABLE)?;

        match table.get(path)? {
            Some(data) => {
                let entry: MetadataEntry = bincode::deserialize(data.value())?;
                Ok(Some(entry))
            }
            None => Ok(None),
        }
    }

    /// Remove metadata for a note
    pub fn remove_metadata(&self, path: &str) -> StorageResult<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(METADATA_TABLE)?;
            table.remove(path)?;
        }
        write_txn.commit()?;

        debug!("Removed metadata for {}", path);
        Ok(())
    }

    /// Get all stored metadata paths
    pub fn list_metadata_paths(&self) -> StorageResult<Vec<String>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(METADATA_TABLE)?;

        let mut paths = Vec::new();
        for entry in table.iter()? {
            let (key, _) = entry?;
            paths.push(key.value().to_string());
        }

        Ok(paths)
    }

    /// Store file stat
    pub fn store_file_stat(&self, path: &str, stat: &FileStat) -> StorageResult<()> {
        let data = bincode::serialize(stat)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(FILE_STATS_TABLE)?;
            table.insert(path, data.as_slice())?;
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Get file stat
    pub fn get_file_stat(&self, path: &str) -> StorageResult<Option<FileStat>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(FILE_STATS_TABLE)?;

        match table.get(path)? {
            Some(data) => {
                let stat: FileStat = bincode::deserialize(data.value())?;
                Ok(Some(stat))
            }
            None => Ok(None),
        }
    }

    /// Remove file stat
    pub fn remove_file_stat(&self, path: &str) -> StorageResult<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(FILE_STATS_TABLE)?;
            table.remove(path)?;
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Update link index for a file
    pub fn update_links(&self, source_path: &str, targets: &[String]) -> StorageResult<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(LINKS_TABLE)?;

            // For each target, add source to its backlinks
            for target in targets {
                // Get existing sources, clone data before dropping guard
                let mut sources: Vec<String> = {
                    let existing = table.get(target.as_str())?;
                    match existing {
                        Some(data) => bincode::deserialize(data.value())?,
                        None => Vec::new(),
                    }
                };

                if !sources.contains(&source_path.to_string()) {
                    sources.push(source_path.to_string());
                    let data = bincode::serialize(&sources)?;
                    table.insert(target.as_str(), data.as_slice())?;
                }
            }
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Get backlinks for a file
    pub fn get_backlinks(&self, target: &str) -> StorageResult<Vec<String>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(LINKS_TABLE)?;

        match table.get(target)? {
            Some(data) => {
                let sources: Vec<String> = bincode::deserialize(data.value())?;
                Ok(sources)
            }
            None => Ok(Vec::new()),
        }
    }

    /// Remove links from a source file
    pub fn remove_links_from(&self, source_path: &str) -> StorageResult<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(LINKS_TABLE)?;

            // Iterate through all targets and remove source from their backlinks
            let mut updates: Vec<(String, Vec<String>)> = Vec::new();

            for entry in table.iter()? {
                let (key, value) = entry?;
                let target = key.value().to_string();
                let mut sources: Vec<String> = bincode::deserialize(value.value())?;

                if sources.contains(&source_path.to_string()) {
                    sources.retain(|s| s != source_path);
                    updates.push((target, sources));
                }
            }

            for (target, sources) in updates {
                if sources.is_empty() {
                    table.remove(target.as_str())?;
                } else {
                    let data = bincode::serialize(&sources)?;
                    table.insert(target.as_str(), data.as_slice())?;
                }
            }
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Update tag index for a file
    pub fn update_tags(&self, path: &str, tags: &[String]) -> StorageResult<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TAGS_TABLE)?;

            for tag in tags {
                // Get existing files, clone data before dropping guard
                let mut files: Vec<String> = {
                    let existing = table.get(tag.as_str())?;
                    match existing {
                        Some(data) => bincode::deserialize(data.value())?,
                        None => Vec::new(),
                    }
                };

                if !files.contains(&path.to_string()) {
                    files.push(path.to_string());
                    let data = bincode::serialize(&files)?;
                    table.insert(tag.as_str(), data.as_slice())?;
                }
            }
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Get files with a tag
    pub fn get_files_with_tag(&self, tag: &str) -> StorageResult<Vec<String>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TAGS_TABLE)?;

        match table.get(tag)? {
            Some(data) => {
                let files: Vec<String> = bincode::deserialize(data.value())?;
                Ok(files)
            }
            None => Ok(Vec::new()),
        }
    }

    /// Get all tags
    pub fn list_tags(&self) -> StorageResult<Vec<String>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TAGS_TABLE)?;

        let mut tags = Vec::new();
        for entry in table.iter()? {
            let (key, _) = entry?;
            tags.push(key.value().to_string());
        }

        Ok(tags)
    }

    /// Remove tags for a file
    pub fn remove_tags_for(&self, path: &str) -> StorageResult<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TAGS_TABLE)?;

            let mut updates: Vec<(String, Vec<String>)> = Vec::new();

            for entry in table.iter()? {
                let (key, value) = entry?;
                let tag = key.value().to_string();
                let mut files: Vec<String> = bincode::deserialize(value.value())?;

                if files.contains(&path.to_string()) {
                    files.retain(|f| f != path);
                    updates.push((tag, files));
                }
            }

            for (tag, files) in updates {
                if files.is_empty() {
                    table.remove(tag.as_str())?;
                } else {
                    let data = bincode::serialize(&files)?;
                    table.insert(tag.as_str(), data.as_slice())?;
                }
            }
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Compact the database
    pub fn compact(&mut self) -> StorageResult<()> {
        info!("Compacting database");
        self.db.compact()?;
        Ok(())
    }

    /// Clear all data
    pub fn clear(&self) -> StorageResult<()> {
        let write_txn = self.db.begin_write()?;
        {
            // Clear metadata table
            let mut metadata_table = write_txn.open_table(METADATA_TABLE)?;
            let metadata_keys: Vec<String> = metadata_table
                .iter()?
                .filter_map(|e| e.ok())
                .map(|(k, _)| k.value().to_string())
                .collect();
            for key in metadata_keys {
                metadata_table.remove(key.as_str())?;
            }

            // Clear file stats table
            let mut stats_table = write_txn.open_table(FILE_STATS_TABLE)?;
            let stats_keys: Vec<String> = stats_table
                .iter()?
                .filter_map(|e| e.ok())
                .map(|(k, _)| k.value().to_string())
                .collect();
            for key in stats_keys {
                stats_table.remove(key.as_str())?;
            }

            // Clear links table
            let mut links_table = write_txn.open_table(LINKS_TABLE)?;
            let links_keys: Vec<String> = links_table
                .iter()?
                .filter_map(|e| e.ok())
                .map(|(k, _)| k.value().to_string())
                .collect();
            for key in links_keys {
                links_table.remove(key.as_str())?;
            }

            // Clear tags table
            let mut tags_table = write_txn.open_table(TAGS_TABLE)?;
            let tags_keys: Vec<String> = tags_table
                .iter()?
                .filter_map(|e| e.ok())
                .map(|(k, _)| k.value().to_string())
                .collect();
            for key in tags_keys {
                tags_table.remove(key.as_str())?;
            }
        }
        write_txn.commit()?;

        info!("Database cleared");
        Ok(())
    }
}

/// Hash content for change detection
pub fn hash_content(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_open() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.redb");

        let db = Database::open(&db_path).unwrap();
        assert!(db.path().exists());
    }

    #[test]
    fn test_metadata_crud() {
        let dir = tempdir().unwrap();
        let db = Database::open(dir.path().join("test.redb")).unwrap();

        let entry = MetadataEntry {
            path: "test.md".to_string(),
            metadata: CachedMetadata::default(),
            mtime: 12345,
            content_hash: hash_content("test content"),
        };

        // Store
        db.store_metadata("test.md", &entry).unwrap();

        // Get
        let retrieved = db.get_metadata("test.md").unwrap().unwrap();
        assert_eq!(retrieved.path, "test.md");
        assert_eq!(retrieved.mtime, 12345);

        // List
        let paths = db.list_metadata_paths().unwrap();
        assert_eq!(paths, vec!["test.md"]);

        // Remove
        db.remove_metadata("test.md").unwrap();
        assert!(db.get_metadata("test.md").unwrap().is_none());
    }

    #[test]
    fn test_links_index() {
        let dir = tempdir().unwrap();
        let db = Database::open(dir.path().join("test.redb")).unwrap();

        // Create links: note1 -> [note2, note3]
        db.update_links("note1.md", &["note2.md".to_string(), "note3.md".to_string()])
            .unwrap();

        // Check backlinks
        let backlinks = db.get_backlinks("note2.md").unwrap();
        assert_eq!(backlinks, vec!["note1.md"]);

        // Remove links
        db.remove_links_from("note1.md").unwrap();
        let backlinks = db.get_backlinks("note2.md").unwrap();
        assert!(backlinks.is_empty());
    }

    #[test]
    fn test_tags_index() {
        let dir = tempdir().unwrap();
        let db = Database::open(dir.path().join("test.redb")).unwrap();

        // Add tags to files
        db.update_tags("note1.md", &["rust".to_string(), "programming".to_string()])
            .unwrap();
        db.update_tags("note2.md", &["rust".to_string()])
            .unwrap();

        // Check files with tag
        let files = db.get_files_with_tag("rust").unwrap();
        assert_eq!(files.len(), 2);

        // List all tags
        let tags = db.list_tags().unwrap();
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"programming".to_string()));

        // Remove tags
        db.remove_tags_for("note1.md").unwrap();
        let files = db.get_files_with_tag("rust").unwrap();
        assert_eq!(files, vec!["note2.md"]);
    }

    #[test]
    fn test_hash_content() {
        let hash1 = hash_content("hello");
        let hash2 = hash_content("hello");
        let hash3 = hash_content("world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
