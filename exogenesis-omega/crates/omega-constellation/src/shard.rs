//! Shard management for device coordination
//!
//! Each shard handles ~4M devices with pattern storage and sync coordination.

use crate::{
    storage::PatternStorage, ConstellationError, DeviceId, DeviceState, GlobalPatterns,
    PatternDelta, Result, ShardStats, ViewingPattern,
};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Shard configuration
#[derive(Debug, Clone)]
pub struct ShardConfig {
    pub shard_id: u32,
    pub region: String,
    pub max_devices: usize,
    pub quality_threshold: f32,
}

impl Default for ShardConfig {
    fn default() -> Self {
        Self {
            shard_id: 0,
            region: "default".to_string(),
            max_devices: 4_000_000,
            quality_threshold: 0.7,
        }
    }
}

/// Shard manager - handles device coordination and pattern storage
pub struct ShardManager {
    shard_id: u32,
    storage: Arc<dyn PatternStorage>,
    devices: DashMap<DeviceId, DeviceState>,
    config: ShardConfig,
}

impl ShardManager {
    /// Create a new shard manager
    pub async fn new(config: ShardConfig, storage: Arc<dyn PatternStorage>) -> Result<Self> {
        info!(
            "Initializing shard {} in region {}",
            config.shard_id, config.region
        );

        Ok(Self {
            shard_id: config.shard_id,
            storage,
            devices: DashMap::new(),
            config,
        })
    }

    /// Handle device sync request
    pub async fn handle_sync(
        &self,
        device_id: DeviceId,
        delta: PatternDelta,
    ) -> Result<GlobalPatterns> {
        debug!(
            "Handling sync for device {} (version {})",
            device_id, delta.local_version
        );

        // 1. Validate device capacity
        self.validate_device(&device_id).await?;

        // 2. Filter and store high-quality patterns
        let quality_patterns: Vec<ViewingPattern> = delta
            .patterns_added
            .into_iter()
            .filter(|p| p.success_rate >= self.config.quality_threshold)
            .collect();

        if !quality_patterns.is_empty() {
            info!(
                "Storing {} quality patterns from device {}",
                quality_patterns.len(),
                device_id
            );
            self.storage
                .store_patterns(&device_id, quality_patterns)
                .await?;
        }

        // 3. Update existing patterns
        for update in delta.patterns_updated {
            self.storage
                .update_pattern(
                    &update.id,
                    update.new_success_rate,
                    update.additional_samples,
                )
                .await?;
        }

        // 4. Remove deprecated patterns
        for pattern_id in delta.patterns_removed {
            self.storage.remove_pattern(&pattern_id).await?;
        }

        // 5. Get global patterns for this device
        let global = self.get_global_patterns(&device_id).await?;

        // 6. Update device state
        self.update_device_state(&device_id, delta.local_version)
            .await;

        // 7. Record metrics
        metrics::counter!("constellation_sync_requests_total", "shard_id" => self.shard_id.to_string())
            .increment(1);

        Ok(global)
    }

    /// Validate device and check capacity
    async fn validate_device(&self, device_id: &DeviceId) -> Result<()> {
        if self.devices.len() >= self.config.max_devices && !self.devices.contains_key(device_id)
        {
            warn!(
                "Shard {} at capacity ({} devices)",
                self.shard_id,
                self.devices.len()
            );
            return Err(ConstellationError::ShardOverload(format!(
                "Shard {} at capacity",
                self.shard_id
            )));
        }
        Ok(())
    }

    /// Get global patterns for device
    async fn get_global_patterns(&self, device_id: &DeviceId) -> Result<GlobalPatterns> {
        // Get similar patterns from other devices
        let similar = self
            .storage
            .get_similar_patterns(device_id, 100)
            .await?;

        // Get trending content
        let trending = self.storage.get_trending(&self.config.region, 50).await?;

        Ok(GlobalPatterns {
            similar,
            trending,
            global_version: chrono::Utc::now().timestamp() as u64,
        })
    }

    /// Update device state
    async fn update_device_state(&self, device_id: &DeviceId, local_version: u64) {
        let pattern_count = self
            .storage
            .get_device_pattern_count(device_id)
            .await
            .unwrap_or(0);

        let state = DeviceState {
            device_id: *device_id,
            last_sync: chrono::Utc::now().timestamp(),
            local_version,
            pattern_count,
            region: self.config.region.clone(),
        };

        self.devices.insert(*device_id, state);
    }

    /// Get shard statistics
    pub async fn get_stats(&self) -> ShardStats {
        let now = chrono::Utc::now().timestamp();
        let active_devices = self
            .devices
            .iter()
            .filter(|entry| now - entry.value().last_sync < 900) // Active in last 15 min
            .count();

        let patterns_stored = self.storage.get_total_patterns().await.unwrap_or(0);

        ShardStats {
            shard_id: self.shard_id,
            total_devices: self.devices.len(),
            active_devices,
            patterns_stored,
            sync_requests_per_min: 0.0, // TODO: Calculate from metrics
            avg_sync_latency_ms: 0.0,   // TODO: Calculate from metrics
        }
    }

    /// Get device count
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Get shard ID
    pub fn shard_id(&self) -> u32 {
        self.shard_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryStorage;

    #[tokio::test]
    async fn test_shard_creation() {
        let config = ShardConfig::default();
        let storage = Arc::new(InMemoryStorage::new());
        let shard = ShardManager::new(config, storage).await.unwrap();
        assert_eq!(shard.device_count(), 0);
    }

    #[tokio::test]
    async fn test_shard_capacity() {
        let config = ShardConfig {
            max_devices: 2,
            ..Default::default()
        };
        let storage = Arc::new(InMemoryStorage::new());
        let shard = ShardManager::new(config, storage).await.unwrap();

        // Add 2 devices - should succeed
        let device1 = DeviceId::new_v4();
        let device2 = DeviceId::new_v4();
        let delta = PatternDelta {
            patterns_added: vec![],
            patterns_updated: vec![],
            patterns_removed: vec![],
            local_version: 1,
        };

        shard.handle_sync(device1, delta.clone()).await.unwrap();
        shard.handle_sync(device2, delta.clone()).await.unwrap();

        // Third device should fail
        let device3 = DeviceId::new_v4();
        let result = shard.handle_sync(device3, delta).await;
        assert!(result.is_err());
    }
}
