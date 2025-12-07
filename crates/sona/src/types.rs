//! Core types for SONA personalization engine

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User viewing event for building preference profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewingEvent {
    pub content_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub completion_rate: f32,
    pub rating: Option<u8>,
    pub is_rewatch: bool,
    pub dismissed: bool,
}

/// Recommendation context (time, device, mood)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationContext {
    pub mood: Option<String>,
    pub time_of_day: Option<String>,
    pub device_type: Option<DeviceType>,
    pub viewing_with: Option<Vec<String>>,
}

/// Device type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    TV,
    Mobile,
    Desktop,
    Tablet,
}

/// Recommendation output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub content_id: Uuid,
    pub confidence_score: f32,
    pub recommendation_type: RecommendationType,
    pub based_on: Vec<String>,
    pub explanation: String,
    pub generated_at: DateTime<Utc>,
    pub ttl_seconds: u32,
    /// A/B test experiment variant (if user is in an active experiment)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experiment_variant: Option<String>,
}

/// Recommendation type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationType {
    Collaborative,
    ContentBased,
    GraphBased,
    ContextAware,
    Hybrid,
}

/// Scored content item
#[derive(Debug, Clone)]
pub struct ScoredContent {
    pub content_id: Uuid,
    pub score: f32,
    pub source: RecommendationType,
    pub based_on: Vec<String>,
}

/// Temporal context patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalContext {
    pub hourly_patterns: Vec<f32>,   // 24 elements
    pub weekday_patterns: Vec<f32>,  // 7 elements
    pub seasonal_patterns: Vec<f32>, // 4 elements
    pub recent_bias: f32,
}

impl Default for TemporalContext {
    fn default() -> Self {
        Self {
            hourly_patterns: vec![0.5; 24],
            weekday_patterns: vec![0.5; 7],
            seasonal_patterns: vec![0.5; 4],
            recent_bias: 0.8,
        }
    }
}

/// Mood state vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodState {
    pub timestamp: DateTime<Utc>,
    pub mood_vector: Vec<f32>, // [calm, energetic, happy, sad, focused, relaxed, social, introspective]
    pub context_tags: Vec<String>,
}

impl Default for MoodState {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            mood_vector: vec![0.5; 8],
            context_tags: vec![],
        }
    }
}

/// Genre affinity map
pub type GenreAffinities = std::collections::HashMap<String, f32>;

/// Content embedding vector (768-dim)
pub type ContentEmbedding = Vec<f32>;

/// User preference vector (512-dim)
pub type PreferenceVector = Vec<f32>;
