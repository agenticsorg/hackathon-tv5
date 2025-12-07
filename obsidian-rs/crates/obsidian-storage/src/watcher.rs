//! File system watcher for real-time updates

use crate::config::VaultConfig;
use crate::error::StorageResult;
use notify::{
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, error, info};

/// Kind of file event
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEventKind {
    /// File was created
    Created,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed (old path, new path)
    Renamed,
}

/// A file system event
#[derive(Debug, Clone)]
pub struct FileEvent {
    /// Event kind
    pub kind: FileEventKind,
    /// Primary path affected
    pub path: PathBuf,
    /// Secondary path (for renames)
    pub new_path: Option<PathBuf>,
}

impl FileEvent {
    /// Create a new event
    pub fn new(kind: FileEventKind, path: impl Into<PathBuf>) -> Self {
        Self {
            kind,
            path: path.into(),
            new_path: None,
        }
    }

    /// Create a rename event
    pub fn rename(old_path: impl Into<PathBuf>, new_path: impl Into<PathBuf>) -> Self {
        Self {
            kind: FileEventKind::Renamed,
            path: old_path.into(),
            new_path: Some(new_path.into()),
        }
    }
}

/// File system watcher for a vault
pub struct VaultWatcher {
    /// Notify watcher
    watcher: RecommendedWatcher,
    /// Broadcast sender for events
    tx: broadcast::Sender<FileEvent>,
    /// Vault configuration
    config: Arc<VaultConfig>,
    /// Pending rename (for tracking rename pairs)
    pending_rename: std::sync::Mutex<Option<PathBuf>>,
}

impl VaultWatcher {
    /// Create a new watcher for a vault
    pub fn new(config: Arc<VaultConfig>) -> StorageResult<Self> {
        let (tx, _) = broadcast::channel(1024);
        let tx_clone = tx.clone();
        let config_clone = config.clone();

        // Create sync channel for notify events
        let (notify_tx, notify_rx) = channel::<Result<Event, notify::Error>>();

        // Start event processing in background
        let pending_rename = std::sync::Mutex::new(None);

        // Create the watcher
        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = notify_tx.send(res);
            },
            Config::default()
                .with_poll_interval(Duration::from_millis(config.watch_debounce_ms)),
        )?;

        let watcher_instance = Self {
            watcher,
            tx,
            config,
            pending_rename,
        };

        // Spawn event processor
        std::thread::spawn(move || {
            Self::process_events(notify_rx, tx_clone, config_clone);
        });

        Ok(watcher_instance)
    }

    /// Process notify events
    fn process_events(
        rx: Receiver<Result<Event, notify::Error>>,
        tx: broadcast::Sender<FileEvent>,
        config: Arc<VaultConfig>,
    ) {
        let mut pending_rename_from: Option<PathBuf> = None;

        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    debug!("Raw notify event: {:?}", event);

                    // Filter by config
                    let relevant_paths: Vec<_> = event
                        .paths
                        .iter()
                        .filter(|p| !config.should_ignore(p))
                        .cloned()
                        .collect();

                    if relevant_paths.is_empty() {
                        continue;
                    }

                    // Convert to FileEvent
                    let file_events = match event.kind {
                        EventKind::Create(CreateKind::File) => {
                            relevant_paths
                                .into_iter()
                                .map(|p| FileEvent::new(FileEventKind::Created, p))
                                .collect()
                        }
                        EventKind::Modify(ModifyKind::Data(_)) => {
                            relevant_paths
                                .into_iter()
                                .map(|p| FileEvent::new(FileEventKind::Modified, p))
                                .collect()
                        }
                        EventKind::Remove(RemoveKind::File) => {
                            relevant_paths
                                .into_iter()
                                .map(|p| FileEvent::new(FileEventKind::Deleted, p))
                                .collect()
                        }
                        EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                            if let Some(path) = relevant_paths.into_iter().next() {
                                pending_rename_from = Some(path);
                            }
                            vec![]
                        }
                        EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                            if let (Some(old_path), Some(new_path)) = (
                                pending_rename_from.take(),
                                relevant_paths.into_iter().next(),
                            ) {
                                vec![FileEvent::rename(old_path, new_path)]
                            } else {
                                vec![]
                            }
                        }
                        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                            if relevant_paths.len() >= 2 {
                                vec![FileEvent::rename(
                                    relevant_paths[0].clone(),
                                    relevant_paths[1].clone(),
                                )]
                            } else {
                                vec![]
                            }
                        }
                        _ => vec![],
                    };

                    // Send events
                    for file_event in file_events {
                        debug!("Sending file event: {:?}", file_event);
                        let _ = tx.send(file_event);
                    }
                }
                Ok(Err(e)) => {
                    error!("Watch error: {}", e);
                }
                Err(_) => {
                    // Channel closed
                    info!("Watcher channel closed");
                    break;
                }
            }
        }
    }

    /// Start watching the vault
    pub fn start(&mut self) -> StorageResult<()> {
        info!("Starting vault watcher for {:?}", self.config.path);
        self.watcher
            .watch(&self.config.path, RecursiveMode::Recursive)?;
        Ok(())
    }

    /// Stop watching
    pub fn stop(&mut self) -> StorageResult<()> {
        info!("Stopping vault watcher");
        self.watcher.unwatch(&self.config.path)?;
        Ok(())
    }

    /// Subscribe to file events
    pub fn subscribe(&self) -> broadcast::Receiver<FileEvent> {
        self.tx.subscribe()
    }

    /// Get the number of subscribers
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }

    /// Check if path is a markdown file we should track
    pub fn should_track(&self, path: &Path) -> bool {
        self.config.is_markdown_file(path) && !self.config.should_ignore(path)
    }
}

/// Builder for VaultWatcher
pub struct WatcherBuilder {
    config: VaultConfig,
}

impl WatcherBuilder {
    /// Create a new builder
    pub fn new(vault_path: impl Into<PathBuf>) -> Self {
        Self {
            config: VaultConfig::new(vault_path),
        }
    }

    /// Set debounce interval
    pub fn debounce(mut self, ms: u64) -> Self {
        self.config.watch_debounce_ms = ms;
        self
    }

    /// Add ignore pattern
    pub fn ignore(mut self, pattern: impl Into<String>) -> Self {
        self.config.ignore_patterns.push(pattern.into());
        self
    }

    /// Build the watcher
    pub fn build(self) -> StorageResult<VaultWatcher> {
        VaultWatcher::new(Arc::new(self.config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_event_creation() {
        let event = FileEvent::new(FileEventKind::Created, "/test/file.md");
        assert_eq!(event.kind, FileEventKind::Created);
        assert_eq!(event.path, PathBuf::from("/test/file.md"));
        assert!(event.new_path.is_none());
    }

    #[test]
    fn test_file_event_rename() {
        let event = FileEvent::rename("/test/old.md", "/test/new.md");
        assert_eq!(event.kind, FileEventKind::Renamed);
        assert_eq!(event.path, PathBuf::from("/test/old.md"));
        assert_eq!(event.new_path, Some(PathBuf::from("/test/new.md")));
    }

    #[test]
    fn test_watcher_builder() {
        let dir = tempdir().unwrap();

        let watcher = WatcherBuilder::new(dir.path())
            .debounce(200)
            .ignore("*.tmp")
            .build();

        assert!(watcher.is_ok());
    }

    #[test]
    fn test_should_track() {
        let dir = tempdir().unwrap();
        let config = Arc::new(VaultConfig::new(dir.path()));
        let watcher = VaultWatcher::new(config).unwrap();

        assert!(watcher.should_track(Path::new("note.md")));
        assert!(!watcher.should_track(Path::new("image.png")));
        assert!(!watcher.should_track(Path::new(".git/config")));
    }
}
