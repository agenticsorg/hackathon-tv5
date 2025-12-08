use crate::events::{ContentMetadata, Recommendation};
use crate::patterns::{GlobalPatterns, PatternDelta};
use crate::types::{ContentId, DeviceId};
use serde::{Deserialize, Serialize};

/// Request to push local patterns to constellation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRequest {
    /// Device sending the patterns
    pub device_id: DeviceId,

    /// Compressed delta of patterns (zstd compressed bincode)
    pub patterns_delta: Vec<u8>,

    /// Local version number for tracking sync state
    pub local_version: u64,

    /// Pattern quality metrics
    pub qualities: Vec<PatternQuality>,
}

impl PushRequest {
    pub fn new(device_id: DeviceId, patterns_delta: Vec<u8>, local_version: u64) -> Self {
        Self {
            device_id,
            patterns_delta,
            local_version,
            qualities: Vec::new(),
        }
    }

    pub fn with_qualities(mut self, qualities: Vec<PatternQuality>) -> Self {
        self.qualities = qualities;
        self
    }
}

/// Quality information for a pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternQuality {
    /// Pattern identifier (as string to avoid circular deps)
    pub pattern_id: String,

    /// Success rate (0.0-1.0)
    pub success_rate: f32,

    /// Number of samples
    pub sample_count: u32,
}

impl PatternQuality {
    pub fn new(pattern_id: String, success_rate: f32, sample_count: u32) -> Self {
        Self {
            pattern_id,
            success_rate,
            sample_count,
        }
    }
}

/// Response to push request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResponse {
    /// Whether the push was accepted
    pub accepted: bool,

    /// New global version after accepting this push
    pub global_version: u64,

    /// Error message if not accepted
    pub error: Option<String>,
}

impl PushResponse {
    pub fn success(global_version: u64) -> Self {
        Self {
            accepted: true,
            global_version,
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            accepted: false,
            global_version: 0,
            error: Some(error),
        }
    }
}

/// Request to pull global patterns from constellation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// Device requesting patterns
    pub device_id: DeviceId,

    /// Last sync version device knows about
    pub last_sync_version: u64,

    /// Content IDs for which device needs embeddings
    pub content_ids: Vec<ContentId>,
}

impl PullRequest {
    pub fn new(device_id: DeviceId, last_sync_version: u64) -> Self {
        Self {
            device_id,
            last_sync_version,
            content_ids: Vec::new(),
        }
    }

    pub fn with_content_ids(mut self, content_ids: Vec<ContentId>) -> Self {
        self.content_ids = content_ids;
        self
    }
}

/// Response to pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResponse {
    /// Compressed global patterns (zstd compressed bincode)
    pub global_patterns: Vec<u8>,

    /// Current global version
    pub global_version: u64,

    /// Embeddings for requested content
    pub new_content: Vec<ContentEmbedding>,

    /// Trending signals
    pub trends: Vec<TrendSignal>,
}

impl PullResponse {
    pub fn new(global_patterns: Vec<u8>, global_version: u64) -> Self {
        Self {
            global_patterns,
            global_version,
            new_content: Vec::new(),
            trends: Vec::new(),
        }
    }

    pub fn with_content(mut self, content: Vec<ContentEmbedding>) -> Self {
        self.new_content = content;
        self
    }

    pub fn with_trends(mut self, trends: Vec<TrendSignal>) -> Self {
        self.trends = trends;
        self
    }
}

/// Content embedding for new content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEmbedding {
    /// Content identifier
    pub content_id: ContentId,

    /// Embedding vector (Float16 encoded as bytes, 384 dims = 768 bytes)
    pub embedding: Vec<u8>,

    /// Content metadata
    pub metadata: ContentMetadata,
}

impl ContentEmbedding {
    pub fn new(content_id: ContentId, embedding: Vec<u8>, metadata: ContentMetadata) -> Self {
        Self {
            content_id,
            embedding,
            metadata,
        }
    }
}

/// Trending signal for content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendSignal {
    /// Trending content ID
    pub content_id: ContentId,

    /// Trending score (0.0-1.0)
    pub trending_score: f32,

    /// Geographic region for this trend
    pub region: String,
}

impl TrendSignal {
    pub fn new(content_id: ContentId, trending_score: f32, region: String) -> Self {
        Self {
            content_id,
            trending_score,
            region,
        }
    }
}

/// Bidirectional sync message for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Push patterns to constellation
    Push(PushRequest),

    /// Response to push
    PushAck(PushResponse),

    /// Pull patterns from constellation
    Pull(PullRequest),

    /// Response to pull
    PullData(PullResponse),

    /// Heartbeat to keep connection alive
    Heartbeat { timestamp: u64 },

    /// Error message
    Error { message: String },
}

impl SyncMessage {
    pub fn heartbeat() -> Self {
        Self::Heartbeat {
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_request_creation() {
        let device_id = DeviceId::new("device-123");
        let req = PushRequest::new(device_id.clone(), vec![1, 2, 3], 1);

        assert_eq!(req.device_id, device_id);
        assert_eq!(req.local_version, 1);
        assert_eq!(req.patterns_delta.len(), 3);
    }

    #[test]
    fn test_push_response_success() {
        let resp = PushResponse::success(42);
        assert!(resp.accepted);
        assert_eq!(resp.global_version, 42);
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_push_response_error() {
        let resp = PushResponse::error("Test error".to_string());
        assert!(!resp.accepted);
        assert!(resp.error.is_some());
    }

    #[test]
    fn test_pull_request_builder() {
        let device_id = DeviceId::new("device-456");
        let req = PullRequest::new(device_id.clone(), 10)
            .with_content_ids(vec![ContentId::new("content1")]);

        assert_eq!(req.last_sync_version, 10);
        assert_eq!(req.content_ids.len(), 1);
    }

    #[test]
    fn test_sync_message_variants() {
        let heartbeat = SyncMessage::heartbeat();
        match heartbeat {
            SyncMessage::Heartbeat { .. } => {},
            _ => panic!("Expected Heartbeat variant"),
        }

        let error = SyncMessage::error("test error");
        match error {
            SyncMessage::Error { message } => assert_eq!(message, "test error"),
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_trend_signal_creation() {
        let trend = TrendSignal::new(
            ContentId::new("trending-content"),
            0.85,
            "us-west".to_string(),
        );

        assert_eq!(trend.trending_score, 0.85);
        assert_eq!(trend.region, "us-west");
    }
}
