use serde::{Deserialize, Serialize};
use crate::events::{TimeSlot, DayType, ContentType, ViewingEvent};

pub type PatternId = String;

/// A learned viewing pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewingPattern {
    pub id: PatternId,
    pub embedding: Vec<f32>,
    pub success_rate: f32,
    pub sample_count: u32,
    pub context: PatternContext,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternContext {
    pub time_of_day: TimeSlot,
    pub day_of_week: DayType,
    pub content_type: ContentType,
    pub genre_hints: Vec<String>,
}

/// Delta update for sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDelta {
    pub patterns_added: Vec<ViewingPattern>,
    pub patterns_updated: Vec<PatternUpdate>,
    pub patterns_removed: Vec<PatternId>,
    pub local_version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternUpdate {
    pub id: PatternId,
    pub new_success_rate: f32,
    pub additional_samples: u32,
}

/// Global patterns from constellation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPatterns {
    pub similar: Vec<AggregatedPattern>,
    pub trending: Vec<AggregatedPattern>,
}

/// Aggregated pattern from multiple sources (federated learning)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedPattern {
    /// Federated-averaged embedding vector
    pub embedding: Vec<f32>,

    /// Quality score (weighted by success_rate and sample_count)
    pub quality: f32,

    /// Number of devices that contributed to this pattern
    pub source_count: usize,
}

impl ViewingPattern {
    pub fn from_event(event: ViewingEvent, embedding: Vec<f32>) -> Self {
        let now = chrono::Utc::now().timestamp() as u64;

        Self {
            id: event.event_id.clone(),
            embedding,
            success_rate: if event.watch_percentage >= 0.8 { 0.9 } else { 0.5 },
            sample_count: 1,
            context: PatternContext {
                time_of_day: event.context.time_of_day,
                day_of_week: event.context.day_of_week,
                content_type: ContentType::Movie, // Default for now
                genre_hints: vec![],
            },
            created_at: now,
            updated_at: now,
        }
    }
}

impl PatternDelta {
    pub fn new(local_version: u64) -> Self {
        Self {
            patterns_added: Vec::new(),
            patterns_updated: Vec::new(),
            patterns_removed: Vec::new(),
            local_version,
        }
    }
}
