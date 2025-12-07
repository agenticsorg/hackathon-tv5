/// Device management and presence tracking
///
/// Handles device registration, capabilities, and remote control
use crate::crdt::HLCTimestamp;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Device registry for managing user devices
pub struct DeviceRegistry {
    /// User identifier
    user_id: String,

    /// Map of device_id -> DeviceInfo
    devices: Arc<RwLock<HashMap<String, DeviceInfo>>>,
}

impl DeviceRegistry {
    /// Create new device registry
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new device
    pub fn register_device(&self, device: DeviceInfo) {
        let mut devices = self.devices.write();
        devices.insert(device.device_id.clone(), device);
    }

    /// Update device heartbeat
    pub fn update_heartbeat(&self, device_id: &str, timestamp: DateTime<Utc>) -> bool {
        let mut devices = self.devices.write();
        if let Some(device) = devices.get_mut(device_id) {
            device.last_seen = timestamp;
            device.is_online = true;
            true
        } else {
            false
        }
    }

    /// Mark device as offline
    pub fn mark_offline(&self, device_id: &str) -> bool {
        let mut devices = self.devices.write();
        if let Some(device) = devices.get_mut(device_id) {
            device.is_online = false;
            true
        } else {
            false
        }
    }

    /// Get device information
    pub fn get_device(&self, device_id: &str) -> Option<DeviceInfo> {
        let devices = self.devices.read();
        devices.get(device_id).cloned()
    }

    /// Get all devices for user
    pub fn get_all_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.devices.read();
        devices.values().cloned().collect()
    }

    /// Get online devices
    pub fn get_online_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.devices.read();
        devices.values().filter(|d| d.is_online).cloned().collect()
    }

    /// Remove device
    pub fn remove_device(&self, device_id: &str) -> bool {
        let mut devices = self.devices.write();
        devices.remove(device_id).is_some()
    }

    /// Check for stale devices (no heartbeat in 60s)
    pub fn check_stale_devices(&self) {
        let now = Utc::now();
        let mut devices = self.devices.write();

        for device in devices.values_mut() {
            let elapsed = now.signed_duration_since(device.last_seen);
            if elapsed.num_seconds() > 60 {
                device.is_online = false;
            }
        }
    }
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Unique device identifier
    pub device_id: String,

    /// Device type
    pub device_type: DeviceType,

    /// Device platform
    pub platform: DevicePlatform,

    /// Device capabilities
    pub capabilities: DeviceCapabilities,

    /// App version
    pub app_version: String,

    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,

    /// Is device currently online
    pub is_online: bool,

    /// Device name (user-friendly)
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    TV,
    Phone,
    Tablet,
    Web,
    Desktop,
}

// Allow non-camel-case for iOS which follows Apple's official capitalization
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevicePlatform {
    Tizen,
    WebOS,
    Android,
    iOS, // Apple's official capitalization
    Web,
    Windows,
    MacOS,
    Linux,
}

/// Device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// Maximum supported video resolution
    pub max_resolution: VideoResolution,

    /// HDR support formats
    pub hdr_support: Vec<HDRFormat>,

    /// Audio codec support
    pub audio_codecs: Vec<AudioCodec>,

    /// Can receive remote commands
    pub remote_controllable: bool,

    /// Can cast content to other devices
    pub can_cast: bool,

    /// Screen size in inches (None for audio-only)
    pub screen_size: Option<f32>,
}

// Allow non-camel-case for industry-standard resolution names (UHD_4K, UHD_8K)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoResolution {
    SD,
    HD,
    FHD,
    UHD_4K, // Industry-standard naming for 4K resolution
    UHD_8K, // Industry-standard naming for 8K resolution
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HDRFormat {
    HDR10,
    DolbyVision,
    HLG,
    HDR10Plus,
}

// Allow non-camel-case for industry-standard audio codec names (DTS_X)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioCodec {
    AAC,
    DolbyAtmos,
    DTS_X, // DTS:X official branding with underscore
    TrueHD,
    AC3,
}

/// Remote control command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCommand {
    /// Target device ID
    pub target_device_id: String,

    /// Source device ID
    pub source_device_id: String,

    /// Command type
    pub command: CommandType,

    /// Timestamp
    pub timestamp: HLCTimestamp,

    /// Command expiration (5s TTL)
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CommandType {
    /// Playback controls
    #[serde(rename = "play")]
    Play,

    #[serde(rename = "pause")]
    Pause,

    #[serde(rename = "stop")]
    Stop,

    #[serde(rename = "seek")]
    Seek { position_seconds: u32 },

    /// Volume controls
    #[serde(rename = "volume_set")]
    VolumeSet { level: f32 },

    #[serde(rename = "volume_mute")]
    VolumeMute,

    #[serde(rename = "volume_unmute")]
    VolumeUnmute,

    /// Content controls
    #[serde(rename = "load_content")]
    LoadContent {
        content_id: String,
        start_position: Option<u32>,
    },

    /// Casting
    #[serde(rename = "cast_to")]
    CastTo {
        target_device_id: String,
        content_id: String,
    },
}

impl RemoteCommand {
    /// Check if command is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Validate command against target device
    pub fn validate(&self, device: &DeviceInfo) -> Result<(), CommandError> {
        // Check device is online
        if !device.is_online {
            return Err(CommandError::DeviceOffline);
        }

        // Check device supports remote control
        if !device.capabilities.remote_controllable {
            return Err(CommandError::NotSupported);
        }

        // Check command hasn't expired
        if self.is_expired() {
            return Err(CommandError::Expired);
        }

        Ok(())
    }
}

/// Command validation errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum CommandError {
    #[error("Device is offline")]
    DeviceOffline,

    #[error("Command not supported by device")]
    NotSupported,

    #[error("Command has expired")]
    Expired,

    #[error("Invalid command parameters")]
    InvalidParameters,
}

/// Device handoff request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceHandoff {
    /// Source device ID
    pub source_device_id: String,

    /// Target device ID
    pub target_device_id: String,

    /// Content ID to handoff
    pub content_id: String,

    /// Current playback position (seconds)
    pub position_seconds: Option<u32>,

    /// Timestamp
    pub timestamp: HLCTimestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_device(device_id: &str) -> DeviceInfo {
        DeviceInfo {
            device_id: device_id.to_string(),
            device_type: DeviceType::TV,
            platform: DevicePlatform::Tizen,
            capabilities: DeviceCapabilities {
                max_resolution: VideoResolution::UHD_4K,
                hdr_support: vec![HDRFormat::HDR10, HDRFormat::DolbyVision],
                audio_codecs: vec![AudioCodec::AAC, AudioCodec::DolbyAtmos],
                remote_controllable: true,
                can_cast: false,
                screen_size: Some(65.0),
            },
            app_version: "1.0.0".to_string(),
            last_seen: Utc::now(),
            is_online: true,
            device_name: Some("Living Room TV".to_string()),
        }
    }

    #[test]
    fn test_device_registration() {
        let registry = DeviceRegistry::new("user-1".to_string());
        let device = create_test_device("device-1");

        registry.register_device(device.clone());

        let retrieved = registry.get_device("device-1").unwrap();
        assert_eq!(retrieved.device_id, "device-1");
    }

    #[test]
    fn test_device_heartbeat() {
        let registry = DeviceRegistry::new("user-1".to_string());
        let device = create_test_device("device-1");

        registry.register_device(device);
        assert!(registry.update_heartbeat("device-1", Utc::now()));
    }

    #[test]
    fn test_online_devices() {
        let registry = DeviceRegistry::new("user-1".to_string());

        let mut device1 = create_test_device("device-1");
        device1.is_online = true;
        registry.register_device(device1);

        let mut device2 = create_test_device("device-2");
        device2.is_online = false;
        registry.register_device(device2);

        let online = registry.get_online_devices();
        assert_eq!(online.len(), 1);
        assert_eq!(online[0].device_id, "device-1");
    }

    #[test]
    fn test_command_validation() {
        let device = create_test_device("device-1");

        let command = RemoteCommand {
            target_device_id: "device-1".to_string(),
            source_device_id: "device-2".to_string(),
            command: CommandType::Play,
            timestamp: HLCTimestamp::from_components(1000, 0),
            expires_at: Utc::now() + chrono::Duration::seconds(5),
        };

        assert!(command.validate(&device).is_ok());
    }

    #[test]
    fn test_command_expired() {
        let device = create_test_device("device-1");

        let command = RemoteCommand {
            target_device_id: "device-1".to_string(),
            source_device_id: "device-2".to_string(),
            command: CommandType::Play,
            timestamp: HLCTimestamp::from_components(1000, 0),
            expires_at: Utc::now() - chrono::Duration::seconds(1),
        };

        assert!(command.is_expired());
        assert!(matches!(
            command.validate(&device),
            Err(CommandError::Expired)
        ));
    }
}
