use crate::types::ViewingEvent;
use omega_agentdb::ReflexionEpisode;
use omega_memory::{Memory, MemoryTier, MemoryContent};
use omega_loops::{CycleInput, LoopType};
use tracing::{debug, info};

/// Event observation processor
pub struct EventObserver;

impl EventObserver {
    pub fn new() -> Self {
        Self
    }

    /// Convert viewing event to ReflexionEpisode for AgentDB
    pub fn to_reflexion_episode(&self, event: &ViewingEvent, embedding: Vec<f32>) -> ReflexionEpisode {
        debug!(
            "Converting event {} to reflexion episode (watch: {:.2}%)",
            event.event_id,
            event.watch_percentage * 100.0
        );

        ReflexionEpisode {
            id: None,
            session_id: event.session_id.clone(),
            task: format!("watched_{}", event.content_id),
            input: serde_json::to_value(event).unwrap_or_default(),
            output: serde_json::json!({
                "watch_pct": event.watch_percentage,
                "engagement": event.engagement_score
            }),
            reward: self.calculate_reward(event),
            success: event.watch_percentage > 0.7,
            critique: self.generate_critique(event),
            latency_ms: 0,
            tokens: 0,
            timestamp: event.timestamp,
            embedding: Some(embedding),
        }
    }

    /// Convert viewing event to Memory for CosmicMemory
    pub fn to_memory(&self, event: &ViewingEvent, embedding: Vec<f32>) -> Memory {
        let importance = self.calculate_importance(event);

        debug!(
            "Converting event {} to memory (importance: {:.2})",
            event.event_id, importance
        );

        Memory::new(
            MemoryTier::Episodic,  // Recent viewing history
            MemoryContent::MultiModal {
                text: Some(format!(
                    "Watched {} ({}) for {:.1}% - Genre: {}, Engagement: {:.2}",
                    event.content_id,
                    event.content_type,
                    event.watch_percentage * 100.0,
                    event.genre,
                    event.engagement_score
                )),
                embedding: embedding.clone(),
                metadata: serde_json::json!({
                    "content_id": event.content_id,
                    "content_type": event.content_type,
                    "genre": event.genre,
                    "watch_percentage": event.watch_percentage,
                    "engagement_score": event.engagement_score,
                    "timestamp": event.timestamp.to_rfc3339(),
                }),
            },
            embedding,
            importance,
        )
    }

    /// Create CycleInput for reflexive loop processing
    pub fn to_cycle_input(&self, event: &ViewingEvent) -> CycleInput {
        debug!("Creating cycle input for reflexive loop");

        CycleInput {
            data: std::collections::HashMap::from([
                ("event".to_string(), serde_json::to_value(event).unwrap_or_default()),
                ("watch_percentage".to_string(), serde_json::json!(event.watch_percentage)),
                ("engagement_score".to_string(), serde_json::json!(event.engagement_score)),
                ("genre".to_string(), serde_json::json!(event.genre)),
            ]),
            context: format!("Process viewing event: {}", event.content_id),
            objectives: vec![
                "Update recommendation model".to_string(),
                "Learn content preferences".to_string(),
                "Adapt to viewing patterns".to_string(),
            ],
        }
    }

    /// Calculate reward signal for reinforcement learning
    fn calculate_reward(&self, event: &ViewingEvent) -> f64 {
        // Combine watch percentage and engagement
        let watch_component = event.watch_percentage as f64;
        let engagement_component = event.engagement_score as f64;

        // Weighted average: 70% watch, 30% engagement
        (watch_component * 0.7 + engagement_component * 0.3).clamp(0.0, 1.0)
    }

    /// Calculate memory importance
    fn calculate_importance(&self, event: &ViewingEvent) -> f64 {
        // High watch percentage and engagement = high importance
        let base_importance = self.calculate_reward(event);

        // Recent events are more important
        let recency_boost = 1.0;

        (base_importance * recency_boost).clamp(0.0, 1.0)
    }

    /// Generate critique for learning
    fn generate_critique(&self, event: &ViewingEvent) -> String {
        if event.watch_percentage > 0.9 {
            format!("Excellent recommendation - user watched {:.0}% of content", event.watch_percentage * 100.0)
        } else if event.watch_percentage > 0.7 {
            format!("Good recommendation - user watched {:.0}%", event.watch_percentage * 100.0)
        } else if event.watch_percentage > 0.3 {
            format!("Partial interest - user watched {:.0}%, consider similar content", event.watch_percentage * 100.0)
        } else {
            format!("Poor match - user watched only {:.0}%, avoid similar recommendations", event.watch_percentage * 100.0)
        }
    }
}

impl Default for EventObserver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_reward() {
        let observer = EventObserver::new();

        let event = ViewingEvent {
            event_id: "e1".to_string(),
            session_id: "s1".to_string(),
            user_id: "u1".to_string(),
            content_id: "c1".to_string(),
            content_type: "movie".to_string(),
            genre: "action".to_string(),
            watch_percentage: 0.9,
            engagement_score: 0.8,
            timestamp: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        };

        let reward = observer.calculate_reward(&event);
        assert!(reward > 0.8);
        assert!(reward <= 1.0);
    }

    #[test]
    fn test_critique_generation() {
        let observer = EventObserver::new();

        let mut event = ViewingEvent {
            event_id: "e1".to_string(),
            session_id: "s1".to_string(),
            user_id: "u1".to_string(),
            content_id: "c1".to_string(),
            content_type: "movie".to_string(),
            genre: "action".to_string(),
            watch_percentage: 0.95,
            engagement_score: 0.9,
            timestamp: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        };

        let critique = observer.generate_critique(&event);
        assert!(critique.contains("Excellent"));

        event.watch_percentage = 0.2;
        let critique = observer.generate_critique(&event);
        assert!(critique.contains("Poor match"));
    }
}
