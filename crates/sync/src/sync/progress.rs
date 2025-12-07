/// Watch progress synchronization using LWW-Register CRDT
///
/// Tracks playback position with last-writer-wins conflict resolution
use crate::crdt::{HLCTimestamp, HybridLogicalClock, PlaybackPosition, PlaybackState};
use crate::sync::publisher::SyncPublisher;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Progress sync manager
pub struct ProgressSync {
    /// User identifier
    user_id: String,

    /// Device identifier
    device_id: String,

    /// Map of content_id -> PlaybackPosition
    positions: Arc<RwLock<HashMap<String, PlaybackPosition>>>,

    /// HLC for timestamp generation
    hlc: Arc<HybridLogicalClock>,

    /// Optional publisher for real-time sync
    publisher: Option<Arc<dyn SyncPublisher>>,
}

impl ProgressSync {
    /// Create new progress sync manager
    pub fn new(user_id: String, device_id: String) -> Self {
        Self {
            user_id,
            device_id,
            positions: Arc::new(RwLock::new(HashMap::new())),
            hlc: Arc::new(HybridLogicalClock::new()),
            publisher: None,
        }
    }

    /// Create new progress sync manager with publisher
    pub fn new_with_publisher(
        user_id: String,
        device_id: String,
        publisher: Arc<dyn SyncPublisher>,
    ) -> Self {
        info!(
            "Creating ProgressSync with publisher for user {} on device {}",
            user_id, device_id
        );
        Self {
            user_id,
            device_id,
            positions: Arc::new(RwLock::new(HashMap::new())),
            hlc: Arc::new(HybridLogicalClock::new()),
            publisher: Some(publisher),
        }
    }

    /// Set publisher for this sync manager
    pub fn set_publisher(&mut self, publisher: Arc<dyn SyncPublisher>) {
        self.publisher = Some(publisher);
    }

    /// Update watch progress for content
    pub fn update_progress(
        &self,
        content_id: String,
        position_seconds: u32,
        duration_seconds: u32,
        state: PlaybackState,
    ) -> ProgressUpdate {
        let timestamp = self.hlc.now();

        let position = PlaybackPosition::new(
            content_id.clone(),
            position_seconds,
            duration_seconds,
            state,
            timestamp,
            self.device_id.clone(),
        );

        let mut positions = self.positions.write();
        positions.insert(content_id.clone(), position.clone());

        let update = ProgressUpdate {
            content_id: content_id.clone(),
            position_seconds,
            duration_seconds,
            state,
            timestamp,
            device_id: self.device_id.clone(),
        };

        // Publish update if publisher is available
        if let Some(ref publisher) = self.publisher {
            let publisher = Arc::clone(publisher);
            let update_clone = update.clone();
            tokio::spawn(async move {
                if let Err(e) = publisher.publish_progress_update(update_clone).await {
                    error!("Failed to publish progress update: {}", e);
                }
            });
        }

        debug!(
            "Updated progress for content {}: {}s/{}s ({:.1}%)",
            content_id,
            position_seconds,
            duration_seconds,
            update.completion_percent() * 100.0
        );

        update
    }

    /// Get progress for content
    pub fn get_progress(&self, content_id: &str) -> Option<PlaybackPosition> {
        let positions = self.positions.read();
        positions.get(content_id).cloned()
    }

    /// Get all progress entries
    pub fn get_all_progress(&self) -> Vec<PlaybackPosition> {
        let positions = self.positions.read();
        positions.values().cloned().collect()
    }

    /// Apply remote progress update
    pub fn apply_remote_update(&self, update: ProgressUpdate) {
        // Update HLC with received timestamp
        self.hlc.update(update.timestamp);

        let new_position = PlaybackPosition::new(
            update.content_id.clone(),
            update.position_seconds,
            update.duration_seconds,
            update.state,
            update.timestamp,
            update.device_id,
        );

        let mut positions = self.positions.write();

        // Merge with existing position (LWW semantics)
        if let Some(existing) = positions.get_mut(&update.content_id) {
            existing.merge(&new_position);
        } else {
            positions.insert(update.content_id.clone(), new_position);
        }
    }

    /// Calculate resume position for content
    /// Returns None if content hasn't been started or is completed
    pub fn get_resume_position(&self, content_id: &str) -> Option<u32> {
        let positions = self.positions.read();
        positions.get(content_id).and_then(|pos| {
            if pos.is_completed() || pos.position_seconds == 0 {
                None
            } else {
                Some(pos.position_seconds)
            }
        })
    }

    /// Get list of in-progress content
    pub fn get_in_progress(&self) -> Vec<PlaybackPosition> {
        let positions = self.positions.read();
        positions
            .values()
            .filter(|pos| !pos.is_completed() && pos.position_seconds > 0)
            .cloned()
            .collect()
    }

    /// Get list of completed content
    pub fn get_completed(&self) -> Vec<PlaybackPosition> {
        let positions = self.positions.read();
        positions
            .values()
            .filter(|pos| pos.is_completed())
            .cloned()
            .collect()
    }

    /// Remove progress for content
    pub fn remove_progress(&self, content_id: &str) -> bool {
        let mut positions = self.positions.write();
        positions.remove(content_id).is_some()
    }

    /// Clear all progress
    pub fn clear_all(&self) {
        let mut positions = self.positions.write();
        positions.clear();
    }
}

/// Progress update message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub content_id: String,
    pub position_seconds: u32,
    pub duration_seconds: u32,
    pub state: PlaybackState,
    pub timestamp: HLCTimestamp,
    pub device_id: String,
}

impl ProgressUpdate {
    /// Calculate completion percentage
    pub fn completion_percent(&self) -> f32 {
        if self.duration_seconds == 0 {
            0.0
        } else {
            self.position_seconds as f32 / self.duration_seconds as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_update() {
        let sync = ProgressSync::new("user-1".to_string(), "device-a".to_string());

        let update =
            sync.update_progress("content-1".to_string(), 100, 1000, PlaybackState::Playing);

        assert_eq!(update.position_seconds, 100);
        assert_eq!(update.completion_percent(), 0.1);

        let progress = sync.get_progress("content-1").unwrap();
        assert_eq!(progress.position_seconds, 100);
    }

    #[test]
    fn test_progress_remote_update() {
        let sync1 = ProgressSync::new("user-1".to_string(), "device-a".to_string());
        let sync2 = ProgressSync::new("user-1".to_string(), "device-b".to_string());

        let update =
            sync1.update_progress("content-1".to_string(), 100, 1000, PlaybackState::Playing);

        sync2.apply_remote_update(update);

        let progress = sync2.get_progress("content-1").unwrap();
        assert_eq!(progress.position_seconds, 100);
    }

    #[test]
    fn test_progress_lww_conflict() {
        let sync = ProgressSync::new("user-1".to_string(), "device-a".to_string());

        // First update at 100s
        let update1 =
            sync.update_progress("content-1".to_string(), 100, 1000, PlaybackState::Playing);

        std::thread::sleep(std::time::Duration::from_millis(10));

        // Second update at 200s (newer timestamp)
        let update2 =
            sync.update_progress("content-1".to_string(), 200, 1000, PlaybackState::Paused);

        // Apply updates in reverse order (simulating out-of-order delivery)
        let sync2 = ProgressSync::new("user-1".to_string(), "device-b".to_string());
        sync2.apply_remote_update(update2.clone());
        sync2.apply_remote_update(update1);

        // Should have the newer update (200s)
        let progress = sync2.get_progress("content-1").unwrap();
        assert_eq!(progress.position_seconds, 200);
    }

    #[test]
    fn test_resume_position() {
        let sync = ProgressSync::new("user-1".to_string(), "device-a".to_string());

        sync.update_progress("content-1".to_string(), 500, 1000, PlaybackState::Paused);

        assert_eq!(sync.get_resume_position("content-1"), Some(500));

        // Completed content should not have resume position
        sync.update_progress("content-2".to_string(), 950, 1000, PlaybackState::Stopped);
        assert_eq!(sync.get_resume_position("content-2"), None);
    }

    #[test]
    fn test_in_progress_list() {
        let sync = ProgressSync::new("user-1".to_string(), "device-a".to_string());

        sync.update_progress("content-1".to_string(), 100, 1000, PlaybackState::Paused);
        sync.update_progress("content-2".to_string(), 950, 1000, PlaybackState::Stopped);
        sync.update_progress("content-3".to_string(), 500, 1000, PlaybackState::Playing);

        let in_progress = sync.get_in_progress();
        assert_eq!(in_progress.len(), 2); // content-1 and content-3

        let completed = sync.get_completed();
        assert_eq!(completed.len(), 1); // content-2
    }
}
