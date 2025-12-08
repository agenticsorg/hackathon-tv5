use crate::types::{Recommendation, ViewContext};
use omega_agentdb::VectorResult;
use omega_memory::{Memory, MemoryContent};
use tracing::{debug, warn};

/// Recommendation engine using AgentDB vector search
pub struct RecommendationEngine {
    min_confidence: f32,
}

impl RecommendationEngine {
    pub fn new() -> Self {
        Self {
            min_confidence: 0.5,
        }
    }

    /// Rank vector search results with memory context
    ///
    /// Uses:
    /// - AgentDB vector search results (SIMD accelerated)
    /// - CosmicMemory semantic tier for preferences
    /// - Context-aware filtering
    pub fn rank_results(
        &self,
        vector_results: Vec<VectorResult>,
        memories: Vec<Memory>,
        context: &ViewContext,
    ) -> Vec<Recommendation> {
        debug!(
            "Ranking {} vector results with {} memories for context: {}",
            vector_results.len(),
            memories.len(),
            context
        );

        let mut recommendations = Vec::new();

        for result in vector_results.iter().take(50) {
            // Get metadata from vector result (it's always a Value, not Option)
            let metadata = &result.metadata;

            // Skip if metadata is null
            if metadata.is_null() {
                warn!("Vector result has null metadata, skipping");
                continue;
            }

            // Extract content info
            let content_id = metadata
                .get("content_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            let title = metadata
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown Title");

            let content_type = metadata
                .get("content_type")
                .and_then(|v| v.as_str())
                .unwrap_or("video");

            let genre = metadata
                .get("genre")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            // Calculate confidence from similarity and memory boost
            let mut confidence: f32 = result.similarity as f32;

            // Boost confidence based on memory preferences
            for memory in &memories {
                // Extract metadata from MemoryContent
                let mem_metadata = match &memory.content {
                    MemoryContent::MultiModal { metadata, .. } => Some(metadata),
                    _ => None,
                };

                if let Some(meta) = mem_metadata {
                    if let Some(mem_genre) = meta.get("genre").and_then(|v| v.as_str()) {
                        if mem_genre == genre {
                            confidence *= 1.2; // Boost for preferred genre
                        }
                    }
                }
            }

            // Filter by minimum confidence
            if confidence < self.min_confidence {
                continue;
            }

            // Generate recommendation reason
            let reason = self.generate_reason(result.similarity as f32, genre, memories.len());

            recommendations.push(Recommendation {
                content_id: content_id.to_string(),
                title: title.to_string(),
                content_type: content_type.to_string(),
                genre: genre.to_string(),
                confidence: confidence.min(1.0),
                reason,
                metadata: metadata.clone(),
            });
        }

        // Sort by confidence descending
        recommendations.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return top 20
        recommendations.truncate(20);

        debug!("Generated {} recommendations", recommendations.len());
        recommendations
    }

    /// Generate human-readable recommendation reason
    fn generate_reason(&self, similarity: f32, genre: &str, memory_count: usize) -> String {
        if similarity > 0.9 {
            "Highly similar to your viewing history".to_string()
        } else if similarity > 0.8 {
            format!("Based on your interest in {}", genre)
        } else if memory_count > 0 {
            format!("Popular in {} category", genre)
        } else {
            "Recommended for you".to_string()
        }
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_results() {
        let engine = RecommendationEngine::new();

        let vector_results = vec![VectorResult {
            id: "1".to_string(),
            similarity: 0.95,
            metadata: serde_json::json!({
                "content_id": "movie-123",
                "title": "Test Movie",
                "content_type": "movie",
                "genre": "action"
            }),
        }];

        let memories = vec![];
        let context = ViewContext {
            user_id: "user1".to_string(),
            device_id: "device1".to_string(),
            time_of_day: 20,
            day_of_week: 5,
            current_genre: Some("action".to_string()),
            previous_content: vec![],
            session_duration_mins: 30,
        };

        let recommendations = engine.rank_results(vector_results, memories, &context);

        assert_eq!(recommendations.len(), 1);
        assert_eq!(recommendations[0].content_id, "movie-123");
        assert!(recommendations[0].confidence >= 0.95);
    }
}
