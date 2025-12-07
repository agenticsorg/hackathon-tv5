/// Observed-Remove Set CRDT implementation
///
/// Used for watchlists and collections with add-wins bias
/// Each addition gets a unique tag to enable precise removal
use super::hlc::HLCTimestamp;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// OR-Set entry with unique tag
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ORSetEntry {
    /// Content identifier
    pub content_id: String,

    /// Unique tag for this add operation (UUID)
    pub unique_tag: String,

    /// HLC timestamp of addition
    pub timestamp: HLCTimestamp,

    /// Device that added the item
    pub device_id: String,
}

/// Observed-Remove Set for managing collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ORSet {
    /// Set of additions (each with unique tag)
    additions: HashMap<String, ORSetEntry>,

    /// Set of removed unique tags
    removals: HashSet<String>,
}

impl ORSet {
    /// Create new empty OR-Set
    pub fn new() -> Self {
        Self {
            additions: HashMap::new(),
            removals: HashSet::new(),
        }
    }

    /// Add content to set
    /// Returns the unique tag for this addition
    pub fn add(
        &mut self,
        content_id: String,
        timestamp: HLCTimestamp,
        device_id: String,
    ) -> String {
        let unique_tag = Uuid::new_v4().to_string();

        let entry = ORSetEntry {
            content_id,
            unique_tag: unique_tag.clone(),
            timestamp,
            device_id,
        };

        self.additions.insert(unique_tag.clone(), entry);
        unique_tag
    }

    /// Remove content from set by content_id
    /// Marks all tags for this content_id as removed
    pub fn remove(&mut self, content_id: &str) {
        let tags_to_remove: Vec<String> = self
            .additions
            .values()
            .filter(|e| e.content_id == content_id)
            .map(|e| e.unique_tag.clone())
            .collect();

        for tag in tags_to_remove {
            self.removals.insert(tag);
        }
    }

    /// Remove by specific unique tag
    pub fn remove_by_tag(&mut self, unique_tag: &str) {
        self.removals.insert(unique_tag.to_string());
    }

    /// Merge with another OR-Set (idempotent)
    pub fn merge(&mut self, other: &ORSet) {
        // Union of additions
        for (tag, entry) in &other.additions {
            self.additions.insert(tag.clone(), entry.clone());
        }

        // Union of removals
        for tag in &other.removals {
            self.removals.insert(tag.clone());
        }
    }

    /// Apply a delta operation (add or remove)
    pub fn apply_delta(&mut self, delta: ORSetDelta) {
        match delta.operation {
            ORSetOperation::Add => {
                self.additions.insert(
                    delta.unique_tag.clone(),
                    ORSetEntry {
                        content_id: delta.content_id,
                        unique_tag: delta.unique_tag,
                        timestamp: delta.timestamp,
                        device_id: delta.device_id,
                    },
                );
            }
            ORSetOperation::Remove => {
                self.removals.insert(delta.unique_tag);
            }
        }
    }

    /// Compute effective set (additions - removals)
    pub fn effective_items(&self) -> HashSet<String> {
        self.additions
            .values()
            .filter(|entry| !self.removals.contains(&entry.unique_tag))
            .map(|entry| entry.content_id.clone())
            .collect()
    }

    /// Get all effective entries with metadata
    pub fn effective_entries(&self) -> Vec<&ORSetEntry> {
        self.additions
            .values()
            .filter(|entry| !self.removals.contains(&entry.unique_tag))
            .collect()
    }

    /// Check if content is in the effective set
    pub fn contains(&self, content_id: &str) -> bool {
        self.effective_items().contains(content_id)
    }

    /// Get number of items in effective set
    pub fn len(&self) -> usize {
        self.effective_items().len()
    }

    /// Check if effective set is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all additions and removals (reset)
    pub fn clear(&mut self) {
        self.additions.clear();
        self.removals.clear();
    }
}

impl Default for ORSet {
    fn default() -> Self {
        Self::new()
    }
}

/// OR-Set operation delta for synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ORSetDelta {
    /// Operation type
    pub operation: ORSetOperation,

    /// Content identifier
    pub content_id: String,

    /// Unique tag
    pub unique_tag: String,

    /// HLC timestamp
    pub timestamp: HLCTimestamp,

    /// Device that made the change
    pub device_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ORSetOperation {
    Add,
    Remove,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_or_set_add_remove() {
        let mut set = ORSet::new();

        set.add(
            "content-1".to_string(),
            HLCTimestamp::from_components(1000, 0),
            "device-a".to_string(),
        );

        assert!(set.contains("content-1"));
        assert_eq!(set.len(), 1);

        set.remove("content-1");
        assert!(!set.contains("content-1"));
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_or_set_add_wins() {
        let mut set1 = ORSet::new();
        let mut set2 = ORSet::new();

        // Device A adds content-1
        let tag1 = set1.add(
            "content-1".to_string(),
            HLCTimestamp::from_components(1000, 0),
            "device-a".to_string(),
        );

        // Device B removes content-1 (removes tag1)
        set2.merge(&set1);
        set2.remove_by_tag(&tag1);

        // Device C adds content-1 again (new tag)
        let mut set3 = ORSet::new();
        set3.add(
            "content-1".to_string(),
            HLCTimestamp::from_components(2000, 0),
            "device-c".to_string(),
        );

        // Merge all
        set1.merge(&set2);
        set1.merge(&set3);

        // content-1 should be in set (add-wins: new addition after removal)
        assert!(set1.contains("content-1"));
    }

    #[test]
    fn test_or_set_merge() {
        let mut set1 = ORSet::new();
        set1.add(
            "content-1".to_string(),
            HLCTimestamp::from_components(1000, 0),
            "device-a".to_string(),
        );

        let mut set2 = ORSet::new();
        set2.add(
            "content-2".to_string(),
            HLCTimestamp::from_components(1000, 0),
            "device-b".to_string(),
        );

        set1.merge(&set2);

        assert!(set1.contains("content-1"));
        assert!(set1.contains("content-2"));
        assert_eq!(set1.len(), 2);
    }

    #[test]
    fn test_or_set_delta() {
        let mut set = ORSet::new();

        let delta = ORSetDelta {
            operation: ORSetOperation::Add,
            content_id: "content-1".to_string(),
            unique_tag: Uuid::new_v4().to_string(),
            timestamp: HLCTimestamp::from_components(1000, 0),
            device_id: "device-a".to_string(),
        };

        set.apply_delta(delta);
        assert!(set.contains("content-1"));
    }
}
