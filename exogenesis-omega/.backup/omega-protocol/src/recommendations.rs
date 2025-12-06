use serde::{Deserialize, Serialize};

/// A recommendation for the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub content_id: String,
    pub content_title: String,
    pub score: f32,
    pub confidence: f32,
    pub reason: String,
}

impl Recommendation {
    pub fn new(content_id: String, content_title: String, score: f32) -> Self {
        Self {
            content_id,
            content_title,
            score,
            confidence: score,
            reason: "Based on your viewing patterns".to_string(),
        }
    }
}
