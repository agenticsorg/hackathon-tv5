/// Remote Command Router with PubNub Targeting
///
/// Routes remote commands to target devices via PubNub with validation,
/// TTL management, and acknowledgment tracking.
use crate::device::{CommandError, CommandType, DeviceInfo, DeviceRegistry};
use crate::pubnub::PubNubClient;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Command router for managing remote device commands
pub struct CommandRouter {
    /// Device registry for validation
    device_registry: Arc<DeviceRegistry>,

    /// PubNub client for publishing commands
    pubnub_client: Arc<PubNubClient>,

    /// Command acknowledgment tracking
    pending_acks: Arc<RwLock<HashMap<Uuid, CommandAck>>>,

    /// User ID for channel routing
    user_id: String,
}

impl CommandRouter {
    /// Create new command router
    pub fn new(
        device_registry: Arc<DeviceRegistry>,
        pubnub_client: Arc<PubNubClient>,
        user_id: String,
    ) -> Self {
        Self {
            device_registry,
            pubnub_client,
            pending_acks: Arc::new(RwLock::new(HashMap::new())),
            user_id,
        }
    }

    /// Validate command against target device capabilities and status
    pub fn validate_command(&self, command: &Command) -> Result<(), CommandError> {
        // Check command hasn't expired
        if command.is_expired() {
            return Err(CommandError::Expired);
        }

        // Get target device from registry
        let device = self
            .device_registry
            .get_device(&command.target_device_id)
            .ok_or(CommandError::DeviceOffline)?;

        // Check device is online
        if !device.is_online {
            return Err(CommandError::DeviceOffline);
        }

        // Check device supports remote control
        if !device.capabilities.remote_controllable {
            return Err(CommandError::NotSupported);
        }

        // Validate command type is supported by device
        self.validate_command_type(&command.command_type, &device)?;

        Ok(())
    }

    /// Validate specific command type against device capabilities
    fn validate_command_type(
        &self,
        command_type: &CommandType,
        device: &DeviceInfo,
    ) -> Result<(), CommandError> {
        match command_type {
            CommandType::CastTo {
                target_device_id, ..
            } => {
                // Check if source device can cast
                if !device.capabilities.can_cast {
                    return Err(CommandError::NotSupported);
                }

                // Check target device exists and is online
                let target = self
                    .device_registry
                    .get_device(target_device_id)
                    .ok_or(CommandError::DeviceOffline)?;

                if !target.is_online {
                    return Err(CommandError::DeviceOffline);
                }
            }
            CommandType::VolumeSet { level } => {
                // Validate volume level is in valid range
                if *level < 0.0 || *level > 1.0 {
                    return Err(CommandError::InvalidParameters);
                }
            }
            CommandType::Seek {
                position_seconds: _,
            } => {
                // Seek command validation - position will be validated by player
                // No device-specific capability check needed
            }
            // Other commands (Play, Pause, Stop, Mute, Unmute, LoadContent) are universally supported
            _ => {}
        }

        Ok(())
    }

    /// Route command to target device via PubNub
    pub async fn route_command(&self, command: Command) -> Result<Uuid, CommandError> {
        // Validate command before routing
        self.validate_command(&command)?;

        // Create PubNub message with device targeting
        let message = DeviceCommandMessage {
            command_id: command.command_id,
            command_type: command.command_type.clone(),
            source_device_id: command.source_device_id.clone(),
            target_device_id: command.target_device_id.clone(),
            payload: command.payload.clone(),
            created_at: command.created_at,
            expires_at: command.expires_at,
        };

        // Publish to user's device channel
        let channel = format!("user.{}.devices", self.user_id);

        self.pubnub_client
            .publish(&channel, &message)
            .await
            .map_err(|e| {
                tracing::error!("Failed to publish command to PubNub: {}", e);
                CommandError::InvalidParameters
            })?;

        // Track pending acknowledgment
        let ack = CommandAck {
            command_id: command.command_id,
            sent_at: Utc::now(),
            acknowledged: false,
            error: None,
        };

        self.pending_acks.write().insert(command.command_id, ack);

        tracing::info!(
            "Routed command {} from {} to {} via PubNub",
            command.command_id,
            command.source_device_id,
            command.target_device_id
        );

        Ok(command.command_id)
    }

    /// Acknowledge command execution
    pub fn acknowledge_command(&self, command_id: Uuid, success: bool, error: Option<String>) {
        let mut acks = self.pending_acks.write();

        if let Some(ack) = acks.get_mut(&command_id) {
            ack.acknowledged = true;
            ack.error = error;

            tracing::info!(
                "Command {} acknowledged: success={}, error={:?}",
                command_id,
                success,
                ack.error
            );
        }
    }

    /// Get pending acknowledgments (for monitoring/debugging)
    pub fn get_pending_acks(&self) -> Vec<CommandAck> {
        let acks = self.pending_acks.read();
        acks.values()
            .filter(|ack| !ack.acknowledged)
            .cloned()
            .collect()
    }

    /// Clean up expired pending acknowledgments
    pub fn cleanup_expired_acks(&self) {
        let mut acks = self.pending_acks.write();
        let now = Utc::now();

        // Remove acks older than 30 seconds
        acks.retain(|_, ack| now.signed_duration_since(ack.sent_at).num_seconds() < 30);
    }
}

/// Remote command structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// Unique command identifier
    pub command_id: Uuid,

    /// Command type (Play, Pause, Seek, etc.)
    pub command_type: CommandType,

    /// Source device ID (sender)
    pub source_device_id: String,

    /// Target device ID (receiver)
    pub target_device_id: String,

    /// Command payload (additional parameters)
    pub payload: serde_json::Value,

    /// Command creation timestamp
    pub created_at: DateTime<Utc>,

    /// Time-to-live in seconds (default: 5)
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u8,

    /// Command expiration timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl Command {
    /// Create new command with default TTL
    pub fn new(
        command_type: CommandType,
        source_device_id: String,
        target_device_id: String,
    ) -> Self {
        let created_at = Utc::now();
        let ttl_seconds = 5;
        let expires_at = created_at + chrono::Duration::seconds(ttl_seconds as i64);

        Self {
            command_id: Uuid::new_v4(),
            command_type,
            source_device_id,
            target_device_id,
            payload: serde_json::json!({}),
            created_at,
            ttl_seconds,
            expires_at: Some(expires_at),
        }
    }

    /// Create command with custom TTL
    pub fn with_ttl(
        command_type: CommandType,
        source_device_id: String,
        target_device_id: String,
        ttl_seconds: u8,
    ) -> Self {
        let created_at = Utc::now();
        let expires_at = created_at + chrono::Duration::seconds(ttl_seconds as i64);

        Self {
            command_id: Uuid::new_v4(),
            command_type,
            source_device_id,
            target_device_id,
            payload: serde_json::json!({}),
            created_at,
            ttl_seconds,
            expires_at: Some(expires_at),
        }
    }

    /// Set custom payload
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }

    /// Check if command has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            // If no expiration set, calculate from created_at + ttl
            let expires_at = self.created_at + chrono::Duration::seconds(self.ttl_seconds as i64);
            Utc::now() > expires_at
        }
    }

    /// Get remaining time-to-live in seconds
    pub fn remaining_ttl(&self) -> i64 {
        if let Some(expires_at) = self.expires_at {
            let remaining = expires_at.signed_duration_since(Utc::now());
            remaining.num_seconds().max(0)
        } else {
            let expires_at = self.created_at + chrono::Duration::seconds(self.ttl_seconds as i64);
            let remaining = expires_at.signed_duration_since(Utc::now());
            remaining.num_seconds().max(0)
        }
    }
}

/// Device command message for PubNub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCommandMessage {
    /// Command identifier
    pub command_id: Uuid,

    /// Command type
    pub command_type: CommandType,

    /// Source device ID
    pub source_device_id: String,

    /// Target device ID (for client-side filtering)
    pub target_device_id: String,

    /// Command payload
    pub payload: serde_json::Value,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Expiration timestamp
    pub expires_at: Option<DateTime<Utc>>,
}

/// Command acknowledgment tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAck {
    /// Command identifier
    pub command_id: Uuid,

    /// Timestamp when command was sent
    pub sent_at: DateTime<Utc>,

    /// Whether command was acknowledged
    pub acknowledged: bool,

    /// Error message if command failed
    pub error: Option<String>,
}

/// Default TTL value (5 seconds)
fn default_ttl() -> u8 {
    5
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{
        AudioCodec, DeviceCapabilities, DevicePlatform, DeviceType, HDRFormat, VideoResolution,
    };
    use crate::pubnub::PubNubConfig;

    fn create_test_device(device_id: &str, is_online: bool, can_cast: bool) -> DeviceInfo {
        DeviceInfo {
            device_id: device_id.to_string(),
            device_type: DeviceType::TV,
            platform: DevicePlatform::Tizen,
            capabilities: DeviceCapabilities {
                max_resolution: VideoResolution::UHD_4K,
                hdr_support: vec![HDRFormat::HDR10],
                audio_codecs: vec![AudioCodec::AAC],
                remote_controllable: true,
                can_cast,
                screen_size: Some(65.0),
            },
            app_version: "1.0.0".to_string(),
            last_seen: Utc::now(),
            is_online,
            device_name: Some("Test TV".to_string()),
        }
    }

    fn create_test_router() -> CommandRouter {
        let user_id = "test-user".to_string();
        let device_registry = Arc::new(DeviceRegistry::new(user_id.clone()));
        let pubnub_config = PubNubConfig::default();
        let pubnub_client = Arc::new(PubNubClient::new(
            pubnub_config,
            user_id.clone(),
            "test-device".to_string(),
        ));

        CommandRouter::new(device_registry, pubnub_client, user_id)
    }

    #[test]
    fn test_command_creation() {
        let command = Command::new(
            CommandType::Play,
            "device-1".to_string(),
            "device-2".to_string(),
        );

        assert_eq!(command.ttl_seconds, 5);
        assert!(!command.is_expired());
        assert!(command.remaining_ttl() > 0);
    }

    #[test]
    fn test_command_expiration() {
        let mut command = Command::new(
            CommandType::Play,
            "device-1".to_string(),
            "device-2".to_string(),
        );

        // Set expiration to past
        command.expires_at = Some(Utc::now() - chrono::Duration::seconds(10));

        assert!(command.is_expired());
        assert_eq!(command.remaining_ttl(), 0);
    }

    #[test]
    fn test_command_with_custom_ttl() {
        let command = Command::with_ttl(
            CommandType::Pause,
            "device-1".to_string(),
            "device-2".to_string(),
            10,
        );

        assert_eq!(command.ttl_seconds, 10);
        assert!(command.remaining_ttl() >= 9);
    }

    #[test]
    fn test_command_with_payload() {
        let payload = serde_json::json!({
            "content_id": "movie-123",
            "position": 100
        });

        let command = Command::new(
            CommandType::LoadContent {
                content_id: "movie-123".to_string(),
                start_position: Some(100),
            },
            "device-1".to_string(),
            "device-2".to_string(),
        )
        .with_payload(payload.clone());

        assert_eq!(command.payload, payload);
    }

    #[test]
    fn test_validate_command_device_offline() {
        let router = create_test_router();

        // Register offline device
        let device = create_test_device("device-1", false, false);
        router.device_registry.register_device(device);

        let command = Command::new(
            CommandType::Play,
            "device-2".to_string(),
            "device-1".to_string(),
        );

        let result = router.validate_command(&command);
        assert!(matches!(result, Err(CommandError::DeviceOffline)));
    }

    #[test]
    fn test_validate_command_device_not_found() {
        let router = create_test_router();

        let command = Command::new(
            CommandType::Play,
            "device-1".to_string(),
            "device-999".to_string(), // Non-existent device
        );

        let result = router.validate_command(&command);
        assert!(matches!(result, Err(CommandError::DeviceOffline)));
    }

    #[test]
    fn test_validate_command_expired() {
        let router = create_test_router();

        // Register online device
        let device = create_test_device("device-1", true, false);
        router.device_registry.register_device(device);

        // Create expired command
        let mut command = Command::new(
            CommandType::Play,
            "device-2".to_string(),
            "device-1".to_string(),
        );
        command.expires_at = Some(Utc::now() - chrono::Duration::seconds(10));

        let result = router.validate_command(&command);
        assert!(matches!(result, Err(CommandError::Expired)));
    }

    #[test]
    fn test_validate_command_success() {
        let router = create_test_router();

        // Register online device
        let device = create_test_device("device-1", true, false);
        router.device_registry.register_device(device);

        let command = Command::new(
            CommandType::Play,
            "device-2".to_string(),
            "device-1".to_string(),
        );

        let result = router.validate_command(&command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_cast_command_not_supported() {
        let router = create_test_router();

        // Register device without cast capability
        let device = create_test_device("device-1", true, false);
        router.device_registry.register_device(device);

        let command = Command::new(
            CommandType::CastTo {
                target_device_id: "device-2".to_string(),
                content_id: "movie-123".to_string(),
            },
            "device-3".to_string(),
            "device-1".to_string(),
        );

        let result = router.validate_command(&command);
        assert!(matches!(result, Err(CommandError::NotSupported)));
    }

    #[test]
    fn test_validate_cast_command_success() {
        let router = create_test_router();

        // Register source device with cast capability
        let source = create_test_device("device-1", true, true);
        router.device_registry.register_device(source);

        // Register target device
        let target = create_test_device("device-2", true, false);
        router.device_registry.register_device(target);

        let command = Command::new(
            CommandType::CastTo {
                target_device_id: "device-2".to_string(),
                content_id: "movie-123".to_string(),
            },
            "device-3".to_string(),
            "device-1".to_string(),
        );

        let result = router.validate_command(&command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_volume_invalid() {
        let router = create_test_router();

        let device = create_test_device("device-1", true, false);
        router.device_registry.register_device(device);

        // Volume > 1.0 is invalid
        let command = Command::new(
            CommandType::VolumeSet { level: 1.5 },
            "device-2".to_string(),
            "device-1".to_string(),
        );

        let result = router.validate_command(&command);
        assert!(matches!(result, Err(CommandError::InvalidParameters)));
    }

    #[test]
    fn test_validate_volume_valid() {
        let router = create_test_router();

        let device = create_test_device("device-1", true, false);
        router.device_registry.register_device(device);

        let command = Command::new(
            CommandType::VolumeSet { level: 0.75 },
            "device-2".to_string(),
            "device-1".to_string(),
        );

        let result = router.validate_command(&command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_acknowledge_command() {
        let router = create_test_router();
        let command_id = Uuid::new_v4();

        // Add pending ack
        let ack = CommandAck {
            command_id,
            sent_at: Utc::now(),
            acknowledged: false,
            error: None,
        };
        router.pending_acks.write().insert(command_id, ack);

        // Acknowledge
        router.acknowledge_command(command_id, true, None);

        let acks = router.pending_acks.read();
        let ack = acks.get(&command_id).unwrap();
        assert!(ack.acknowledged);
        assert!(ack.error.is_none());
    }

    #[test]
    fn test_acknowledge_command_with_error() {
        let router = create_test_router();
        let command_id = Uuid::new_v4();

        // Add pending ack
        let ack = CommandAck {
            command_id,
            sent_at: Utc::now(),
            acknowledged: false,
            error: None,
        };
        router.pending_acks.write().insert(command_id, ack);

        // Acknowledge with error
        router.acknowledge_command(command_id, false, Some("Device busy".to_string()));

        let acks = router.pending_acks.read();
        let ack = acks.get(&command_id).unwrap();
        assert!(ack.acknowledged);
        assert_eq!(ack.error, Some("Device busy".to_string()));
    }

    #[test]
    fn test_get_pending_acks() {
        let router = create_test_router();

        // Add acknowledged ack
        let ack1 = CommandAck {
            command_id: Uuid::new_v4(),
            sent_at: Utc::now(),
            acknowledged: true,
            error: None,
        };
        router.pending_acks.write().insert(ack1.command_id, ack1);

        // Add pending ack
        let ack2 = CommandAck {
            command_id: Uuid::new_v4(),
            sent_at: Utc::now(),
            acknowledged: false,
            error: None,
        };
        router
            .pending_acks
            .write()
            .insert(ack2.command_id, ack2.clone());

        let pending = router.get_pending_acks();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].command_id, ack2.command_id);
    }

    #[test]
    fn test_cleanup_expired_acks() {
        let router = create_test_router();

        // Add old ack (40 seconds ago)
        let old_ack = CommandAck {
            command_id: Uuid::new_v4(),
            sent_at: Utc::now() - chrono::Duration::seconds(40),
            acknowledged: false,
            error: None,
        };
        router
            .pending_acks
            .write()
            .insert(old_ack.command_id, old_ack);

        // Add recent ack
        let recent_ack = CommandAck {
            command_id: Uuid::new_v4(),
            sent_at: Utc::now() - chrono::Duration::seconds(5),
            acknowledged: false,
            error: None,
        };
        router
            .pending_acks
            .write()
            .insert(recent_ack.command_id, recent_ack.clone());

        // Cleanup
        router.cleanup_expired_acks();

        let acks = router.pending_acks.read();
        assert_eq!(acks.len(), 1);
        assert!(acks.contains_key(&recent_ack.command_id));
    }

    #[test]
    fn test_command_serialization() {
        let command = Command::new(
            CommandType::Seek {
                position_seconds: 100,
            },
            "device-1".to_string(),
            "device-2".to_string(),
        );

        let json = serde_json::to_string(&command).unwrap();
        let deserialized: Command = serde_json::from_str(&json).unwrap();

        assert_eq!(command.command_id, deserialized.command_id);
        assert_eq!(command.source_device_id, deserialized.source_device_id);
        assert_eq!(command.target_device_id, deserialized.target_device_id);
    }
}
