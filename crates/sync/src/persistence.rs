//! Persistence layer integration for sync service
//!
//! Provides automatic loading and saving of CRDT state

use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, info};

use crate::crdt::{ORSet, PlaybackPosition};
use crate::device::DeviceInfo;
use crate::repository::SyncRepository;

/// Persistence manager for sync service
pub struct SyncPersistence {
    repository: Arc<dyn SyncRepository>,
}

impl SyncPersistence {
    pub fn new(repository: Arc<dyn SyncRepository>) -> Self {
        Self { repository }
    }

    /// Load watchlist state from database on startup
    pub async fn load_watchlist_state(&self, user_id: &str) -> Result<ORSet> {
        info!("Loading watchlist state for user {}", user_id);
        let or_set = self.repository.load_watchlist(user_id).await?;
        debug!(
            "Loaded {} watchlist items for user {}",
            or_set.len(),
            user_id
        );
        Ok(or_set)
    }

    /// Load progress state from database on startup
    pub async fn load_progress_state(&self, user_id: &str) -> Result<Vec<PlaybackPosition>> {
        info!("Loading progress state for user {}", user_id);
        let positions = self.repository.load_progress(user_id).await?;
        debug!(
            "Loaded {} progress entries for user {}",
            positions.len(),
            user_id
        );
        Ok(positions)
    }

    /// Load devices from database on startup
    pub async fn load_devices(&self, user_id: &str) -> Result<Vec<DeviceInfo>> {
        info!("Loading devices for user {}", user_id);
        let devices = self.repository.load_devices(user_id).await?;
        debug!("Loaded {} devices for user {}", devices.len(), user_id);
        Ok(devices)
    }

    /// Persist watchlist state to database (debounced for performance)
    pub async fn persist_watchlist(&self, user_id: &str, or_set: &ORSet) -> Result<()> {
        debug!(
            "Persisting watchlist for user {} ({} items)",
            user_id,
            or_set.len()
        );
        self.repository.save_watchlist(user_id, or_set).await?;
        Ok(())
    }

    /// Persist progress update to database
    pub async fn persist_progress(&self, user_id: &str, position: &PlaybackPosition) -> Result<()> {
        debug!(
            "Persisting progress for user {} content {}",
            user_id, position.content_id
        );
        self.repository.save_progress(user_id, position).await?;
        Ok(())
    }

    /// Persist device registration to database
    pub async fn persist_device(&self, user_id: &str, device: &DeviceInfo) -> Result<()> {
        debug!(
            "Persisting device {} for user {}",
            device.device_id, user_id
        );
        self.repository.save_device(user_id, device).await?;
        Ok(())
    }

    /// Update device heartbeat
    pub async fn update_device_heartbeat(&self, user_id: &str, device_id: &str) -> Result<()> {
        self.repository
            .update_device_heartbeat(user_id, device_id)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::HLCTimestamp;
    use crate::repository::PostgresSyncRepository;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn test_persistence_manager_lifecycle() {
        // This test requires DATABASE_URL to be set
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await
                .unwrap();

            let repo = Arc::new(PostgresSyncRepository::new(pool));
            let persistence = SyncPersistence::new(repo);

            let user_id = uuid::Uuid::new_v4().to_string();

            // Create and save watchlist
            let mut or_set = ORSet::new();
            or_set.add(
                "content-1".to_string(),
                HLCTimestamp::from_components(1000, 0),
                "device-1".to_string(),
            );

            persistence
                .persist_watchlist(&user_id, &or_set)
                .await
                .unwrap();

            // Load and verify
            let loaded = persistence.load_watchlist_state(&user_id).await.unwrap();
            assert!(loaded.contains("content-1"));
        }
    }
}
