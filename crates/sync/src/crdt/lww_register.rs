/// Last-Writer-Wins Register CRDT implementation
///
/// Used for watch progress and user preferences where latest update wins
/// Conflict resolution: timestamp-based with device_id tie-breaker
use super::hlc::HLCTimestamp;
use serde::{Deserialize, Serialize};

/// LWW-Register for single-value eventual consistency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LWWRegister<T> {
    /// Current value
    pub value: T,

    /// HLC timestamp of last write
    pub timestamp: HLCTimestamp,

    /// Device that made the last write (tie-breaker)
    pub device_id: String,
}

impl<T: Clone> LWWRegister<T> {
    /// Create new LWW-Register with initial value
    pub fn new(value: T, timestamp: HLCTimestamp, device_id: String) -> Self {
        Self {
            value,
            timestamp,
            device_id,
        }
    }

    /// Update the register with a new value
    pub fn set(&mut self, value: T, timestamp: HLCTimestamp, device_id: String) {
        if self.should_accept_update(timestamp, &device_id) {
            self.value = value;
            self.timestamp = timestamp;
            self.device_id = device_id;
        }
    }

    /// Merge with another LWW-Register (idempotent)
    pub fn merge(&mut self, other: &LWWRegister<T>) {
        if self.should_accept_update(other.timestamp, &other.device_id) {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.device_id = other.device_id.clone();
        }
    }

    /// Check if update should be accepted based on timestamp and device_id
    fn should_accept_update(&self, timestamp: HLCTimestamp, device_id: &str) -> bool {
        if timestamp > self.timestamp {
            // Other timestamp is newer
            true
        } else if timestamp == self.timestamp {
            // Timestamps equal, use device_id as tie-breaker (lexicographic order)
            device_id > self.device_id.as_str()
        } else {
            // Current timestamp is newer
            false
        }
    }

    /// Get current value
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Get timestamp of last write
    pub fn get_timestamp(&self) -> HLCTimestamp {
        self.timestamp
    }
}

/// Playback position using LWW-Register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackPosition {
    /// Content identifier
    pub content_id: String,

    /// Position in seconds
    pub position_seconds: u32,

    /// Total duration in seconds
    pub duration_seconds: u32,

    /// Playback state
    pub state: PlaybackState,

    /// HLC timestamp
    pub timestamp: HLCTimestamp,

    /// Device that updated the position
    pub device_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

impl PlaybackPosition {
    /// Create new playback position
    pub fn new(
        content_id: String,
        position_seconds: u32,
        duration_seconds: u32,
        state: PlaybackState,
        timestamp: HLCTimestamp,
        device_id: String,
    ) -> Self {
        Self {
            content_id,
            position_seconds,
            duration_seconds,
            state,
            timestamp,
            device_id,
        }
    }

    /// Merge with another playback position (LWW semantics)
    pub fn merge(&mut self, other: &PlaybackPosition) {
        if other.content_id != self.content_id {
            return; // Different content, no merge
        }

        if other.timestamp > self.timestamp {
            // Other is newer, adopt all fields
            self.position_seconds = other.position_seconds;
            self.duration_seconds = other.duration_seconds;
            self.state = other.state;
            self.timestamp = other.timestamp;
            self.device_id = other.device_id.clone();
        } else if other.timestamp == self.timestamp && other.device_id > self.device_id {
            // Tie-breaker: device_id
            self.position_seconds = other.position_seconds;
            self.duration_seconds = other.duration_seconds;
            self.state = other.state;
            self.device_id = other.device_id.clone();
        }
    }

    /// Calculate completion percentage
    pub fn completion_percent(&self) -> f32 {
        if self.duration_seconds == 0 {
            0.0
        } else {
            self.position_seconds as f32 / self.duration_seconds as f32
        }
    }

    /// Check if content is considered "completed" (>90% watched)
    pub fn is_completed(&self) -> bool {
        self.completion_percent() > 0.9
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lww_register_basic() {
        let mut reg = LWWRegister::new(
            100,
            HLCTimestamp::from_components(1000, 0),
            "device-a".to_string(),
        );

        // Update with newer timestamp
        reg.set(
            200,
            HLCTimestamp::from_components(2000, 0),
            "device-b".to_string(),
        );
        assert_eq!(*reg.get(), 200);
    }

    #[test]
    fn test_lww_register_merge() {
        let mut reg1 = LWWRegister::new(
            100,
            HLCTimestamp::from_components(1000, 0),
            "device-a".to_string(),
        );

        let reg2 = LWWRegister::new(
            200,
            HLCTimestamp::from_components(2000, 0),
            "device-b".to_string(),
        );

        reg1.merge(&reg2);
        assert_eq!(*reg1.get(), 200);
    }

    #[test]
    fn test_lww_register_tie_breaker() {
        let mut reg1 = LWWRegister::new(
            100,
            HLCTimestamp::from_components(1000, 0),
            "device-a".to_string(),
        );

        let reg2 = LWWRegister::new(
            200,
            HLCTimestamp::from_components(1000, 0),
            "device-z".to_string(),
        );

        reg1.merge(&reg2);
        assert_eq!(*reg1.get(), 200); // device-z > device-a
    }

    #[test]
    fn test_playback_position() {
        let mut pos = PlaybackPosition::new(
            "content-1".to_string(),
            100,
            1000,
            PlaybackState::Playing,
            HLCTimestamp::from_components(1000, 0),
            "device-a".to_string(),
        );

        assert_eq!(pos.completion_percent(), 0.1);
        assert!(!pos.is_completed());

        let other = PlaybackPosition::new(
            "content-1".to_string(),
            950,
            1000,
            PlaybackState::Paused,
            HLCTimestamp::from_components(2000, 0),
            "device-b".to_string(),
        );

        pos.merge(&other);
        assert_eq!(pos.position_seconds, 950);
        assert!(pos.is_completed());
    }
}
