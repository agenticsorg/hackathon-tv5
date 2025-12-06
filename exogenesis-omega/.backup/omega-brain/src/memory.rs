use anyhow::Result;
use omega_protocol::{ViewingPattern, PatternDelta, GlobalPatterns};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::Path;

/// AgentDB memory systems for pattern storage and learning
/// Combines ReflexionMemory, SkillLibrary, and ReasoningBank
pub struct AgentMemory {
    /// Episode storage (ReflexionMemory)
    episodes: RwLock<Vec<ViewingPattern>>,

    /// Learned patterns (SkillLibrary)
    skills: RwLock<HashMap<String, ViewingPattern>>,

    /// Consolidated approaches (ReasoningBank)
    reasoning: RwLock<Vec<ReasoningPattern>>,

    /// Local version counter for sync
    version: RwLock<u64>,
}

/// A consolidated reasoning pattern
#[derive(Debug, Clone)]
struct ReasoningPattern {
    pub pattern_id: String,
    pub success_rate: f32,
    pub sample_count: u32,
    pub last_updated: u64,
}

impl AgentMemory {
    /// Create a new memory system or load from storage
    pub fn load_or_create<P: AsRef<Path>>(storage_path: P) -> Result<Self> {
        // TODO: In production, load from persistent storage (SQLite, RocksDB, etc.)
        tracing::info!(
            "Initializing AgentMemory (in-memory) at {:?}",
            storage_path.as_ref()
        );

        Ok(Self {
            episodes: RwLock::new(Vec::new()),
            skills: RwLock::new(HashMap::new()),
            reasoning: RwLock::new(Vec::new()),
            version: RwLock::new(0),
        })
    }

    /// Record a new viewing pattern (episode)
    pub fn record(&self, pattern: ViewingPattern) {
        let pattern_id = pattern.id.clone();

        // Add to episodes
        {
            let mut episodes = self.episodes.write();
            episodes.push(pattern.clone());

            // Keep only recent episodes (limit to prevent unbounded growth)
            if episodes.len() > 1000 {
                episodes.drain(0..100); // Remove oldest 100
            }
        }

        // Update or create skill
        {
            let mut skills = self.skills.write();
            skills
                .entry(pattern_id.clone())
                .and_modify(|existing| {
                    // Update with weighted average
                    let total_count = existing.sample_count + pattern.sample_count;
                    existing.success_rate = (existing.success_rate
                        * existing.sample_count as f32
                        + pattern.success_rate * pattern.sample_count as f32)
                        / total_count as f32;
                    existing.sample_count = total_count;
                    existing.updated_at = pattern.updated_at;
                })
                .or_insert(pattern);
        }

        // Increment version
        *self.version.write() += 1;
    }

    /// Get changes since a specific version for sync
    pub fn get_changes_since(&self, since_version: u64) -> PatternDelta {
        let current_version = *self.version.read();

        // For simplicity, return high-quality patterns
        // In production, would track actual changes
        let skills = self.skills.read();
        let patterns_added: Vec<ViewingPattern> = skills
            .values()
            .filter(|p| p.success_rate >= 0.7)
            .cloned()
            .collect();

        PatternDelta {
            patterns_added,
            patterns_updated: vec![],
            patterns_removed: vec![],
            local_version: current_version,
        }
    }

    /// Merge global patterns from constellation
    pub fn merge_global_pattern(&self, pattern: &ViewingPattern) {
        let mut reasoning = self.reasoning.write();

        // Add or update reasoning pattern
        if let Some(existing) = reasoning
            .iter_mut()
            .find(|p| p.pattern_id == pattern.id)
        {
            existing.success_rate = pattern.success_rate;
            existing.sample_count = pattern.sample_count;
            existing.last_updated = pattern.updated_at;
        } else {
            reasoning.push(ReasoningPattern {
                pattern_id: pattern.id.clone(),
                success_rate: pattern.success_rate,
                sample_count: pattern.sample_count,
                last_updated: pattern.updated_at,
            });
        }

        // Keep only top patterns
        if reasoning.len() > 500 {
            reasoning.sort_by(|a, b| {
                b.success_rate
                    .partial_cmp(&a.success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            reasoning.truncate(500);
        }
    }

    /// Get all stored skills
    pub fn get_skills(&self) -> Vec<ViewingPattern> {
        self.skills.read().values().cloned().collect()
    }

    /// Get episode count
    pub fn episode_count(&self) -> usize {
        self.episodes.read().len()
    }

    /// Get skill count
    pub fn skill_count(&self) -> usize {
        self.skills.read().len()
    }

    /// Get current version
    pub fn version(&self) -> u64 {
        *self.version.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_protocol::events::{ViewingEvent, ViewContext};

    #[test]
    fn test_memory_record() {
        let memory = AgentMemory::load_or_create("/tmp/test.db").unwrap();

        let event = ViewingEvent::new(
            "content-123".to_string(),
            "Test Movie".to_string(),
            0.95,
            5400,
        );

        let embedding = vec![0.1; 384];
        let pattern = ViewingPattern::from_event(event, embedding);

        memory.record(pattern);

        assert_eq!(memory.episode_count(), 1);
        assert_eq!(memory.skill_count(), 1);
        assert_eq!(memory.version(), 1);
    }

    #[test]
    fn test_memory_get_changes() {
        let memory = AgentMemory::load_or_create("/tmp/test2.db").unwrap();

        // Record some patterns
        for i in 0..5 {
            let event = ViewingEvent::new(
                format!("content-{}", i),
                format!("Movie {}", i),
                0.8 + i as f32 * 0.02,
                5400,
            );

            let embedding = vec![0.1 * i as f32; 384];
            let pattern = ViewingPattern::from_event(event, embedding);
            memory.record(pattern);
        }

        let delta = memory.get_changes_since(0);
        assert!(delta.patterns_added.len() > 0);
        assert_eq!(delta.local_version, 5);
    }
}
