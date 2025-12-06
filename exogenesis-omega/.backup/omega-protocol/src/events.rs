use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A viewing event from a TV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewingEvent {
    pub event_id: String,
    pub content_id: String,
    pub content_title: String,
    pub watch_percentage: f32,
    pub watch_time_secs: u64,
    pub timestamp: u64,
    pub context: ViewContext,
}

/// Context for viewing or recommendation requests
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ViewContext {
    pub time_of_day: TimeSlot,
    pub day_of_week: DayType,
    pub mood: Option<String>,
    pub recent_watches: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeSlot {
    Morning,   // 6-12
    Afternoon, // 12-18
    Evening,   // 18-22
    Night,     // 22-6
}

impl Default for TimeSlot {
    fn default() -> Self {
        Self::Evening
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DayType {
    Weekday,
    Weekend,
}

impl Default for DayType {
    fn default() -> Self {
        Self::Weekday
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentType {
    Movie,
    Series,
    Documentary,
    Sports,
    News,
}

impl ViewingEvent {
    pub fn new(
        content_id: String,
        content_title: String,
        watch_percentage: f32,
        watch_time_secs: u64,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            content_id,
            content_title,
            watch_percentage,
            watch_time_secs,
            timestamp: chrono::Utc::now().timestamp() as u64,
            context: ViewContext::default(),
        }
    }

    /// Convert event to text for embedding
    pub fn to_embedding_text(&self) -> String {
        format!(
            "title: {} time: {:?} day: {:?} watch: {:.0}%",
            self.content_title,
            self.context.time_of_day,
            self.context.day_of_week,
            self.watch_percentage * 100.0
        )
    }
}

impl ViewContext {
    pub fn to_embedding_text(&self) -> String {
        let mood = self.mood.as_deref().unwrap_or("neutral");
        let recent = if self.recent_watches.is_empty() {
            "none".to_string()
        } else {
            self.recent_watches.join(", ")
        };

        format!(
            "time: {:?} day: {:?} mood: {} recent: {}",
            self.time_of_day, self.day_of_week, mood, recent
        )
    }
}

/// User mood inferred from behavior
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Mood {
    Relaxed,
    Energetic,
    Focused,
    Social,
}

/// Content metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub title: String,
    pub content_type: ContentType,
    pub genres: Vec<String>,
    pub duration_minutes: u32,
    pub year: Option<u32>,
}

/// Recommendation reason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationReason {
    SimilarToPast,
    Trending,
    ContextualMatch,
    NewRelease,
}
