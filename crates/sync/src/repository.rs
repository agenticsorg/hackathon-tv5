//! Sync service repository for PostgreSQL persistence
//!
//! Provides CRUD operations for watchlists, progress, and devices

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::crdt::{HLCTimestamp, ORSet, ORSetEntry, PlaybackPosition, PlaybackState};
use crate::device::{
    AudioCodec, DeviceCapabilities, DeviceInfo, DevicePlatform, DeviceType, HDRFormat,
    VideoResolution,
};

/// Sync repository trait for persistence operations
#[async_trait]
pub trait SyncRepository: Send + Sync {
    // Watchlist operations
    async fn load_watchlist(&self, user_id: &str) -> Result<ORSet>;
    async fn save_watchlist(&self, user_id: &str, or_set: &ORSet) -> Result<()>;
    async fn add_watchlist_item(
        &self,
        user_id: &str,
        content_id: &str,
        entry: &ORSetEntry,
    ) -> Result<()>;
    async fn remove_watchlist_item(&self, user_id: &str, unique_tag: &str) -> Result<()>;

    // Progress operations
    async fn load_progress(&self, user_id: &str) -> Result<Vec<PlaybackPosition>>;
    async fn save_progress(&self, user_id: &str, position: &PlaybackPosition) -> Result<()>;
    async fn get_progress(
        &self,
        user_id: &str,
        content_id: &str,
    ) -> Result<Option<PlaybackPosition>>;
    async fn delete_progress(&self, user_id: &str, content_id: &str) -> Result<()>;

    // Device operations
    async fn load_devices(&self, user_id: &str) -> Result<Vec<DeviceInfo>>;
    async fn save_device(&self, user_id: &str, device: &DeviceInfo) -> Result<()>;
    async fn get_device(&self, user_id: &str, device_id: &str) -> Result<Option<DeviceInfo>>;
    async fn delete_device(&self, user_id: &str, device_id: &str) -> Result<()>;
    async fn update_device_heartbeat(&self, user_id: &str, device_id: &str) -> Result<()>;
}

/// PostgreSQL implementation of SyncRepository
pub struct PostgresSyncRepository {
    pool: PgPool,
}

impl PostgresSyncRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Helper: Convert VideoResolution to string
    fn video_resolution_to_string(res: VideoResolution) -> &'static str {
        match res {
            VideoResolution::SD => "SD",
            VideoResolution::HD => "HD",
            VideoResolution::FHD => "FHD",
            VideoResolution::UHD_4K => "UHD_4K",
            VideoResolution::UHD_8K => "UHD_8K",
        }
    }

    // Helper: Parse string to VideoResolution
    fn string_to_video_resolution(s: &str) -> VideoResolution {
        match s {
            "SD" => VideoResolution::SD,
            "HD" => VideoResolution::HD,
            "FHD" => VideoResolution::FHD,
            "UHD_4K" => VideoResolution::UHD_4K,
            "UHD_8K" => VideoResolution::UHD_8K,
            _ => VideoResolution::HD,
        }
    }

    // Helper: Convert DeviceType to string
    fn device_type_to_string(dt: DeviceType) -> &'static str {
        match dt {
            DeviceType::TV => "TV",
            DeviceType::Phone => "Phone",
            DeviceType::Tablet => "Tablet",
            DeviceType::Web => "Web",
            DeviceType::Desktop => "Desktop",
        }
    }

    // Helper: Parse string to DeviceType
    fn string_to_device_type(s: &str) -> DeviceType {
        match s {
            "TV" => DeviceType::TV,
            "Phone" => DeviceType::Phone,
            "Tablet" => DeviceType::Tablet,
            "Web" => DeviceType::Web,
            "Desktop" => DeviceType::Desktop,
            _ => DeviceType::Web,
        }
    }

    // Helper: Convert PlaybackState to string
    fn playback_state_to_string(state: PlaybackState) -> &'static str {
        match state {
            PlaybackState::Playing => "playing",
            PlaybackState::Paused => "paused",
            PlaybackState::Stopped => "stopped",
        }
    }

    // Helper: Parse string to PlaybackState
    fn string_to_playback_state(s: &str) -> PlaybackState {
        match s {
            "playing" => PlaybackState::Playing,
            "paused" => PlaybackState::Paused,
            "stopped" => PlaybackState::Stopped,
            _ => PlaybackState::Stopped,
        }
    }

    // Helper: Serialize DeviceCapabilities
    fn serialize_capabilities(caps: &DeviceCapabilities) -> serde_json::Value {
        json!({
            "max_resolution": Self::video_resolution_to_string(caps.max_resolution),
            "hdr_support": caps.hdr_support.iter().map(|h| format!("{:?}", h)).collect::<Vec<_>>(),
            "audio_codecs": caps.audio_codecs.iter().map(|a| format!("{:?}", a)).collect::<Vec<_>>(),
            "remote_controllable": caps.remote_controllable,
            "can_cast": caps.can_cast,
            "screen_size": caps.screen_size,
        })
    }

    // Helper: Deserialize DeviceCapabilities
    fn deserialize_capabilities(value: serde_json::Value) -> Result<DeviceCapabilities> {
        Ok(DeviceCapabilities {
            max_resolution: Self::string_to_video_resolution(
                value["max_resolution"].as_str().unwrap_or("HD"),
            ),
            hdr_support: value["hdr_support"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| match v.as_str()? {
                            "HDR10" => Some(HDRFormat::HDR10),
                            "DolbyVision" => Some(HDRFormat::DolbyVision),
                            "HLG" => Some(HDRFormat::HLG),
                            "HDR10Plus" => Some(HDRFormat::HDR10Plus),
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            audio_codecs: value["audio_codecs"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| match v.as_str()? {
                            "AAC" => Some(AudioCodec::AAC),
                            "DolbyAtmos" => Some(AudioCodec::DolbyAtmos),
                            "DTS_X" => Some(AudioCodec::DTS_X),
                            "TrueHD" => Some(AudioCodec::TrueHD),
                            "AC3" => Some(AudioCodec::AC3),
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            remote_controllable: value["remote_controllable"].as_bool().unwrap_or(false),
            can_cast: value["can_cast"].as_bool().unwrap_or(false),
            screen_size: value["screen_size"].as_f64().map(|f| f as f32),
        })
    }

    // Helper: Serialize HLCTimestamp
    fn serialize_hlc_timestamp(ts: HLCTimestamp) -> serde_json::Value {
        json!({
            "physical": ts.physical_time(),
            "logical": ts.logical_counter(),
        })
    }

    // Helper: Deserialize HLCTimestamp
    fn deserialize_hlc_timestamp(value: serde_json::Value) -> HLCTimestamp {
        let physical = value["physical"].as_u64().unwrap_or(0);
        let logical = value["logical"].as_u64().unwrap_or(0);
        HLCTimestamp::from_components(
            physical.try_into().expect("physical timestamp overflow"),
            logical.try_into().expect("logical counter overflow"),
        )
    }
}

#[async_trait]
impl SyncRepository for PostgresSyncRepository {
    async fn load_watchlist(&self, user_id: &str) -> Result<ORSet> {
        let user_uuid = Uuid::parse_str(user_id).context("Invalid user ID format")?;

        // Load additions
        let additions = sqlx::query(
            r#"
            SELECT content_id, unique_tag, timestamp_physical, timestamp_logical, device_id
            FROM user_watchlists
            WHERE user_id = $1 AND is_removed = false
            "#,
        )
        .bind(user_uuid)
        .fetch_all(&self.pool)
        .await
        .context("Failed to load watchlist additions")?;

        // Load removals
        let removals = sqlx::query(
            r#"
            SELECT unique_tag
            FROM user_watchlists
            WHERE user_id = $1 AND is_removed = true
            "#,
        )
        .bind(user_uuid)
        .fetch_all(&self.pool)
        .await
        .context("Failed to load watchlist removals")?;

        let mut or_set = ORSet::new();

        // Reconstruct OR-Set from database
        for add in additions {
            let content_id: String = add.try_get("content_id")?;
            let unique_tag: String = add.try_get("unique_tag")?;
            let timestamp_physical: i64 = add.try_get("timestamp_physical")?;
            let timestamp_logical: i32 = add.try_get("timestamp_logical")?;
            let device_id: String = add.try_get("device_id")?;

            let timestamp =
                HLCTimestamp::from_components(timestamp_physical, timestamp_logical as u16);
            let entry = ORSetEntry {
                content_id: content_id.clone(),
                unique_tag: unique_tag.clone(),
                timestamp,
                device_id,
            };
            // Direct insertion to bypass add() which generates new tags
            use crate::crdt::{ORSetDelta, ORSetOperation};
            or_set.apply_delta(ORSetDelta {
                operation: ORSetOperation::Add,
                content_id: entry.content_id.clone(),
                unique_tag: entry.unique_tag,
                timestamp: entry.timestamp,
                device_id: entry.device_id,
            });
        }

        for rem in removals {
            let unique_tag: String = rem.try_get("unique_tag")?;
            or_set.remove_by_tag(&unique_tag);
        }

        Ok(or_set)
    }

    async fn save_watchlist(&self, user_id: &str, or_set: &ORSet) -> Result<()> {
        let user_uuid = Uuid::parse_str(user_id).context("Invalid user ID format")?;

        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin transaction")?;

        // Clear existing watchlist
        sqlx::query("DELETE FROM user_watchlists WHERE user_id = $1")
            .bind(user_uuid)
            .execute(&mut *tx)
            .await
            .context("Failed to clear watchlist")?;

        // Insert effective entries
        for entry in or_set.effective_entries() {
            sqlx::query(
                r#"
                INSERT INTO user_watchlists (
                    user_id, content_id, unique_tag,
                    timestamp_physical, timestamp_logical,
                    device_id, is_removed
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(user_uuid)
            .bind(&entry.content_id)
            .bind(&entry.unique_tag)
            .bind(entry.timestamp.physical_time() as i64)
            .bind(entry.timestamp.logical_counter() as i32)
            .bind(&entry.device_id)
            .bind(false)
            .execute(&mut *tx)
            .await
            .context("Failed to insert watchlist entry")?;
        }

        tx.commit().await.context("Failed to commit watchlist")?;
        Ok(())
    }

    async fn add_watchlist_item(
        &self,
        user_id: &str,
        content_id: &str,
        entry: &ORSetEntry,
    ) -> Result<()> {
        let user_uuid = Uuid::parse_str(user_id).context("Invalid user ID format")?;

        sqlx::query(
            r#"
            INSERT INTO user_watchlists (
                user_id, content_id, unique_tag,
                timestamp_physical, timestamp_logical,
                device_id, is_removed
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (user_id, unique_tag) DO UPDATE SET
                content_id = EXCLUDED.content_id,
                timestamp_physical = EXCLUDED.timestamp_physical,
                timestamp_logical = EXCLUDED.timestamp_logical,
                device_id = EXCLUDED.device_id,
                is_removed = EXCLUDED.is_removed
            "#,
        )
        .bind(user_uuid)
        .bind(content_id)
        .bind(&entry.unique_tag)
        .bind(entry.timestamp.physical_time() as i64)
        .bind(entry.timestamp.logical_counter() as i32)
        .bind(&entry.device_id)
        .bind(false)
        .execute(&self.pool)
        .await
        .context("Failed to add watchlist item")?;

        Ok(())
    }

    async fn remove_watchlist_item(&self, user_id: &str, unique_tag: &str) -> Result<()> {
        let user_uuid = Uuid::parse_str(user_id).context("Invalid user ID format")?;

        sqlx::query(
            r#"
            UPDATE user_watchlists
            SET is_removed = true
            WHERE user_id = $1 AND unique_tag = $2
            "#,
        )
        .bind(user_uuid)
        .bind(unique_tag)
        .execute(&self.pool)
        .await
        .context("Failed to remove watchlist item")?;

        Ok(())
    }

    async fn load_progress(&self, user_id: &str) -> Result<Vec<PlaybackPosition>> {
        let user_uuid = Uuid::parse_str(user_id).context("Invalid user ID format")?;

        let rows = sqlx::query(
            r#"
            SELECT content_id, position_seconds, duration_seconds, state,
                   timestamp_physical, timestamp_logical, device_id
            FROM user_progress
            WHERE user_id = $1
            "#,
        )
        .bind(user_uuid)
        .fetch_all(&self.pool)
        .await
        .context("Failed to load progress")?;

        let positions = rows
            .into_iter()
            .map(|row| {
                let content_id: String = row.try_get("content_id")?;
                let position_seconds: i32 = row.try_get("position_seconds")?;
                let duration_seconds: i32 = row.try_get("duration_seconds")?;
                let state: String = row.try_get("state")?;
                let timestamp_physical: i64 = row.try_get("timestamp_physical")?;
                let timestamp_logical: i32 = row.try_get("timestamp_logical")?;
                let device_id: String = row.try_get("device_id")?;

                let timestamp =
                    HLCTimestamp::from_components(timestamp_physical, timestamp_logical as u16);
                Ok(PlaybackPosition::new(
                    content_id,
                    position_seconds as u32,
                    duration_seconds as u32,
                    Self::string_to_playback_state(&state),
                    timestamp,
                    device_id,
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(positions)
    }

    async fn save_progress(&self, user_id: &str, position: &PlaybackPosition) -> Result<()> {
        let user_uuid = Uuid::parse_str(user_id).context("Invalid user ID format")?;

        sqlx::query(
            r#"
            INSERT INTO user_progress (
                user_id, content_id, position_seconds, duration_seconds,
                state, timestamp_physical, timestamp_logical, device_id, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (user_id, content_id) DO UPDATE SET
                position_seconds = EXCLUDED.position_seconds,
                duration_seconds = EXCLUDED.duration_seconds,
                state = EXCLUDED.state,
                timestamp_physical = EXCLUDED.timestamp_physical,
                timestamp_logical = EXCLUDED.timestamp_logical,
                device_id = EXCLUDED.device_id,
                updated_at = EXCLUDED.updated_at
            WHERE
                user_progress.timestamp_physical < EXCLUDED.timestamp_physical
                OR (user_progress.timestamp_physical = EXCLUDED.timestamp_physical
                    AND user_progress.timestamp_logical < EXCLUDED.timestamp_logical)
            "#,
        )
        .bind(user_uuid)
        .bind(&position.content_id)
        .bind(position.position_seconds as i32)
        .bind(position.duration_seconds as i32)
        .bind(Self::playback_state_to_string(position.state))
        .bind(position.timestamp.physical_time() as i64)
        .bind(position.timestamp.logical_counter() as i32)
        .bind(&position.device_id)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .context("Failed to save progress")?;

        Ok(())
    }

    async fn get_progress(
        &self,
        user_id: &str,
        content_id: &str,
    ) -> Result<Option<PlaybackPosition>> {
        let user_uuid = Uuid::parse_str(user_id).context("Invalid user ID format")?;

        let row = sqlx::query(
            r#"
            SELECT content_id, position_seconds, duration_seconds, state,
                   timestamp_physical, timestamp_logical, device_id
            FROM user_progress
            WHERE user_id = $1 AND content_id = $2
            "#,
        )
        .bind(user_uuid)
        .bind(content_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get progress")?;

        Ok(row
            .map(|r| -> Result<PlaybackPosition> {
                let content_id: String = r.try_get("content_id")?;
                let position_seconds: i32 = r.try_get("position_seconds")?;
                let duration_seconds: i32 = r.try_get("duration_seconds")?;
                let state: String = r.try_get("state")?;
                let timestamp_physical: i64 = r.try_get("timestamp_physical")?;
                let timestamp_logical: i32 = r.try_get("timestamp_logical")?;
                let device_id: String = r.try_get("device_id")?;

                let timestamp =
                    HLCTimestamp::from_components(timestamp_physical, timestamp_logical as u16);
                Ok(PlaybackPosition::new(
                    content_id,
                    position_seconds as u32,
                    duration_seconds as u32,
                    Self::string_to_playback_state(&state),
                    timestamp,
                    device_id,
                ))
            })
            .transpose()?)
    }

    async fn delete_progress(&self, user_id: &str, content_id: &str) -> Result<()> {
        let user_uuid = Uuid::parse_str(user_id).context("Invalid user ID format")?;

        sqlx::query("DELETE FROM user_progress WHERE user_id = $1 AND content_id = $2")
            .bind(user_uuid)
            .bind(content_id)
            .execute(&self.pool)
            .await
            .context("Failed to delete progress")?;

        Ok(())
    }

    async fn load_devices(&self, user_id: &str) -> Result<Vec<DeviceInfo>> {
        let user_uuid = Uuid::parse_str(user_id).context("Invalid user ID format")?;

        let rows = sqlx::query(
            r#"
            SELECT device_id, device_type, platform, capabilities, app_version,
                   last_seen, is_online, device_name
            FROM user_devices
            WHERE user_id = $1
            "#,
        )
        .bind(user_uuid)
        .fetch_all(&self.pool)
        .await
        .context("Failed to load devices")?;

        let devices = rows
            .into_iter()
            .filter_map(|row| {
                let device_id: String = row.try_get("device_id").ok()?;
                let device_type: String = row.try_get("device_type").ok()?;
                let platform: String = row.try_get("platform").ok()?;
                let capabilities: serde_json::Value = row.try_get("capabilities").ok()?;
                let app_version: String = row.try_get("app_version").ok()?;
                let last_seen: DateTime<Utc> = row.try_get("last_seen").ok()?;
                let is_online: bool = row.try_get("is_online").ok()?;
                let device_name: Option<String> = row.try_get("device_name").ok()?;

                let capabilities = Self::deserialize_capabilities(capabilities).ok()?;
                let platform: DevicePlatform =
                    serde_json::from_str(&format!("\"{}\"", platform)).ok()?;

                Some(DeviceInfo {
                    device_id,
                    device_type: Self::string_to_device_type(&device_type),
                    platform,
                    capabilities,
                    app_version,
                    last_seen,
                    is_online,
                    device_name,
                })
            })
            .collect();

        Ok(devices)
    }

    async fn save_device(&self, user_id: &str, device: &DeviceInfo) -> Result<()> {
        let user_uuid = Uuid::parse_str(user_id).context("Invalid user ID format")?;

        let platform_str = format!("{:?}", device.platform);
        let capabilities = Self::serialize_capabilities(&device.capabilities);

        sqlx::query(
            r#"
            INSERT INTO user_devices (
                user_id, device_id, device_type, platform, capabilities,
                app_version, last_seen, is_online, device_name
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (user_id, device_id) DO UPDATE SET
                device_type = EXCLUDED.device_type,
                platform = EXCLUDED.platform,
                capabilities = EXCLUDED.capabilities,
                app_version = EXCLUDED.app_version,
                last_seen = EXCLUDED.last_seen,
                is_online = EXCLUDED.is_online,
                device_name = EXCLUDED.device_name
            "#,
        )
        .bind(user_uuid)
        .bind(&device.device_id)
        .bind(Self::device_type_to_string(device.device_type))
        .bind(platform_str)
        .bind(capabilities)
        .bind(&device.app_version)
        .bind(device.last_seen)
        .bind(device.is_online)
        .bind(&device.device_name)
        .execute(&self.pool)
        .await
        .context("Failed to save device")?;

        Ok(())
    }

    async fn get_device(&self, user_id: &str, device_id: &str) -> Result<Option<DeviceInfo>> {
        let user_uuid = Uuid::parse_str(user_id).context("Invalid user ID format")?;

        let row = sqlx::query(
            r#"
            SELECT device_id, device_type, platform, capabilities, app_version,
                   last_seen, is_online, device_name
            FROM user_devices
            WHERE user_id = $1 AND device_id = $2
            "#,
        )
        .bind(user_uuid)
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get device")?;

        Ok(row.and_then(|r| {
            let device_id: String = r.try_get("device_id").ok()?;
            let device_type: String = r.try_get("device_type").ok()?;
            let platform: String = r.try_get("platform").ok()?;
            let capabilities: serde_json::Value = r.try_get("capabilities").ok()?;
            let app_version: String = r.try_get("app_version").ok()?;
            let last_seen: DateTime<Utc> = r.try_get("last_seen").ok()?;
            let is_online: bool = r.try_get("is_online").ok()?;
            let device_name: Option<String> = r.try_get("device_name").ok()?;

            let capabilities = Self::deserialize_capabilities(capabilities).ok()?;
            let platform: DevicePlatform =
                serde_json::from_str(&format!("\"{}\"", platform)).ok()?;

            Some(DeviceInfo {
                device_id,
                device_type: Self::string_to_device_type(&device_type),
                platform,
                capabilities,
                app_version,
                last_seen,
                is_online,
                device_name,
            })
        }))
    }

    async fn delete_device(&self, user_id: &str, device_id: &str) -> Result<()> {
        let user_uuid = Uuid::parse_str(user_id).context("Invalid user ID format")?;

        sqlx::query("DELETE FROM user_devices WHERE user_id = $1 AND device_id = $2")
            .bind(user_uuid)
            .bind(device_id)
            .execute(&self.pool)
            .await
            .context("Failed to delete device")?;

        Ok(())
    }

    async fn update_device_heartbeat(&self, user_id: &str, device_id: &str) -> Result<()> {
        let user_uuid = Uuid::parse_str(user_id).context("Invalid user ID format")?;

        sqlx::query(
            r#"
            UPDATE user_devices
            SET last_seen = $1, is_online = true
            WHERE user_id = $2 AND device_id = $3
            "#,
        )
        .bind(Utc::now())
        .bind(user_uuid)
        .bind(device_id)
        .execute(&self.pool)
        .await
        .context("Failed to update device heartbeat")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_resolution_conversion() {
        assert_eq!(
            PostgresSyncRepository::video_resolution_to_string(VideoResolution::UHD_4K),
            "UHD_4K"
        );
        assert_eq!(
            PostgresSyncRepository::string_to_video_resolution("UHD_4K"),
            VideoResolution::UHD_4K
        );
    }

    #[test]
    fn test_device_type_conversion() {
        assert_eq!(
            PostgresSyncRepository::device_type_to_string(DeviceType::TV),
            "TV"
        );
        assert_eq!(
            PostgresSyncRepository::string_to_device_type("TV"),
            DeviceType::TV
        );
    }

    #[test]
    fn test_playback_state_conversion() {
        assert_eq!(
            PostgresSyncRepository::playback_state_to_string(PlaybackState::Playing),
            "playing"
        );
        assert_eq!(
            PostgresSyncRepository::string_to_playback_state("playing"),
            PlaybackState::Playing
        );
    }
}
