use serde::{Deserialize, Serialize};
use crate::patterns::{PatternDelta, GlobalPatterns};

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub patterns_pushed: usize,
    pub patterns_received: usize,
    pub global_version: u64,
    pub success: bool,
}

impl SyncResult {
    pub fn success(patterns_pushed: usize, patterns_received: usize, global_version: u64) -> Self {
        Self {
            patterns_pushed,
            patterns_received,
            global_version,
            success: true,
        }
    }

    pub fn failure() -> Self {
        Self {
            patterns_pushed: 0,
            patterns_received: 0,
            global_version: 0,
            success: false,
        }
    }
}
