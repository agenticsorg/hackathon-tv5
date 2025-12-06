use serde::{Deserialize, Serialize};
use std::fmt;

/// Viewing context for recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewContext {
    pub user_id: String,
    pub device_id: String,
    pub time_of_day: u8,        // 0-23
    pub day_of_week: u8,        // 0-6
    pub current_genre: Option<String>,
    pub previous_content: Vec<String>,
    pub session_duration_mins: u32,
}

impl fmt::Display for ViewContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "user={} time={:02}:00 dow={} genre={:?} session={}m",
            self.user_id,
            self.time_of_day,
            self.day_of_week,
            self.current_genre,
            self.session_duration_mins
        )
    }
}

/// Viewing event to be recorded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewingEvent {
    pub event_id: String,
    pub session_id: String,
    pub user_id: String,
    pub content_id: String,
    pub content_type: String,       // movie, series, live
    pub genre: String,
    pub watch_percentage: f32,      // 0.0-1.0
    pub engagement_score: f32,      // 0.0-1.0 (pauses, rewinds, etc.)
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: serde_json::Value,
}

/// Content recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub content_id: String,
    pub title: String,
    pub content_type: String,
    pub genre: String,
    pub confidence: f32,            // 0.0-1.0
    pub reason: String,
    pub metadata: serde_json::Value,
}

/// Sync result summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub patterns_pushed: usize,
    pub patterns_received: usize,
    pub sync_timestamp: chrono::DateTime<chrono::Utc>,
    pub delta_size_bytes: usize,
    pub global_size_bytes: usize,
}

/// Global pattern from constellation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPattern {
    pub name: String,
    pub description: String,
    pub embedding: Vec<f32>,
    pub usage_count: i32,
    pub success_rate: f64,
    pub metadata: serde_json::Value,
}

/// Global patterns response from sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPatternsResponse {
    pub patterns: Vec<GlobalPattern>,
    pub version: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
