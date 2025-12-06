//! Type definitions for omega-tv-sync protocol
//!
//! These types represent the data structures used in TV-to-Constellation synchronization.
//! They are optimized for compression and minimal bandwidth usage.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Delta update sent from TV to Constellation (~1KB compressed)
///
/// Contains local patterns learned by the TV that have high success rates
/// and should be shared with the global pattern database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDelta {
    /// Unique device identifier
    pub device_id: String,

    /// Local patterns to upload (filtered for quality ≥ 0.7)
    pub patterns: Vec<PatternData>,

    /// Monotonic version counter for conflict detection
    pub version: u64,

    /// Timestamp of delta creation
    pub timestamp: DateTime<Utc>,
}

impl SyncDelta {
    /// Create a new sync delta
    pub fn new(device_id: String, patterns: Vec<PatternData>, version: u64) -> Self {
        Self {
            device_id,
            patterns,
            version,
            timestamp: Utc::now(),
        }
    }

    /// Validate the delta before sending
    pub fn validate(&self) -> Result<(), crate::Error> {
        if self.device_id.is_empty() {
            return Err(crate::Error::InvalidPattern(
                "device_id cannot be empty".to_string(),
            ));
        }

        for pattern in &self.patterns {
            pattern.validate()?;
        }

        Ok(())
    }

    /// Estimate uncompressed size in bytes
    pub fn estimate_size(&self) -> usize {
        // Rough estimate: metadata + (patterns * embedding_size * 4 bytes)
        let metadata = 100; // device_id, version, timestamp
        let pattern_overhead = self.patterns.len() * 50; // metadata per pattern
        let embeddings = self.patterns.len() * 384 * 4; // f32 = 4 bytes
        metadata + pattern_overhead + embeddings
    }
}

/// Pattern data representing learned viewing behavior
///
/// Each pattern captures a successful recommendation that can be shared
/// across the constellation network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternData {
    /// Unique pattern identifier (UUID or content-based hash)
    pub id: String,

    /// 384-dimensional embedding vector (MiniLM size)
    pub embedding: Vec<f32>,

    /// Success rate (0.0 to 1.0, filtered ≥ 0.7)
    pub success_rate: f64,

    /// Number of samples this pattern was trained on
    pub sample_count: u64,

    /// Content genre/category for filtering
    pub genre: String,
}

impl PatternData {
    /// Create a new pattern
    pub fn new(
        id: String,
        embedding: Vec<f32>,
        success_rate: f64,
        sample_count: u64,
        genre: String,
    ) -> Self {
        Self {
            id,
            embedding,
            success_rate,
            sample_count,
            genre,
        }
    }

    /// Validate pattern data
    pub fn validate(&self) -> Result<(), crate::Error> {
        if self.embedding.len() != 384 {
            return Err(crate::Error::InvalidPattern(format!(
                "embedding must be 384-dimensional, got {}",
                self.embedding.len()
            )));
        }

        if !(0.0..=1.0).contains(&self.success_rate) {
            return Err(crate::Error::InvalidPattern(format!(
                "success_rate must be in [0.0, 1.0], got {}",
                self.success_rate
            )));
        }

        if self.sample_count == 0 {
            return Err(crate::Error::InvalidPattern(
                "sample_count must be > 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if this pattern meets quality threshold for sync
    pub fn meets_sync_threshold(&self) -> bool {
        self.success_rate >= 0.7 && self.sample_count >= 10
    }
}

/// Global patterns received from Constellation (~5KB compressed)
///
/// Contains aggregated patterns from the federated network and trending
/// content signals for improving local recommendations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPatterns {
    /// Federated patterns from constellation network
    pub patterns: Vec<PatternData>,

    /// Trending content signals
    pub trends: Vec<TrendSignal>,

    /// Global version number for tracking updates
    pub version: u64,

    /// Server timestamp
    pub timestamp: DateTime<Utc>,
}

impl GlobalPatterns {
    /// Create new global patterns response
    pub fn new(patterns: Vec<PatternData>, trends: Vec<TrendSignal>, version: u64) -> Self {
        Self {
            patterns,
            trends,
            version,
            timestamp: Utc::now(),
        }
    }

    /// Estimate uncompressed size in bytes
    pub fn estimate_size(&self) -> usize {
        let metadata = 100;
        let pattern_size = self.patterns.len() * (50 + 384 * 4);
        let trend_size = self.trends.len() * 100; // rough estimate
        metadata + pattern_size + trend_size
    }
}

/// Trending content signal from global network
///
/// Indicates content that is performing well across the constellation,
/// weighted by region and recency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendSignal {
    /// Content identifier
    pub content_id: String,

    /// Trend score (0.0 to 1.0, higher = more trending)
    pub score: f64,

    /// Geographic region code (ISO 3166-1 alpha-2)
    pub region: String,

    /// Genre/category
    pub genre: String,

    /// Timestamp of trend calculation
    pub calculated_at: DateTime<Utc>,
}

impl TrendSignal {
    /// Create a new trend signal
    pub fn new(content_id: String, score: f64, region: String, genre: String) -> Self {
        Self {
            content_id,
            score,
            region,
            genre,
            calculated_at: Utc::now(),
        }
    }

    /// Check if trend is still fresh (within 24 hours)
    pub fn is_fresh(&self) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.calculated_at);
        age.num_hours() < 24
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_validation() {
        let valid = PatternData::new(
            "test-1".to_string(),
            vec![0.0; 384],
            0.85,
            100,
            "action".to_string(),
        );
        assert!(valid.validate().is_ok());
        assert!(valid.meets_sync_threshold());

        let invalid_embedding = PatternData::new(
            "test-2".to_string(),
            vec![0.0; 128], // wrong size
            0.85,
            100,
            "action".to_string(),
        );
        assert!(invalid_embedding.validate().is_err());

        let low_quality = PatternData::new(
            "test-3".to_string(),
            vec![0.0; 384],
            0.5, // too low
            100,
            "action".to_string(),
        );
        assert!(!low_quality.meets_sync_threshold());
    }

    #[test]
    fn test_sync_delta_validation() {
        let delta = SyncDelta::new(
            "tv-123".to_string(),
            vec![PatternData::new(
                "p1".to_string(),
                vec![0.0; 384],
                0.9,
                50,
                "drama".to_string(),
            )],
            1,
        );
        assert!(delta.validate().is_ok());

        let empty_device = SyncDelta::new("".to_string(), vec![], 1);
        assert!(empty_device.validate().is_err());
    }

    #[test]
    fn test_trend_freshness() {
        let fresh = TrendSignal::new(
            "content-1".to_string(),
            0.95,
            "US".to_string(),
            "thriller".to_string(),
        );
        assert!(fresh.is_fresh());

        let old = TrendSignal {
            content_id: "content-2".to_string(),
            score: 0.8,
            region: "UK".to_string(),
            genre: "comedy".to_string(),
            calculated_at: Utc::now() - chrono::Duration::hours(48),
        };
        assert!(!old.is_fresh());
    }

    #[test]
    fn test_size_estimation() {
        let delta = SyncDelta::new(
            "tv-123".to_string(),
            vec![
                PatternData::new("p1".to_string(), vec![0.0; 384], 0.9, 50, "drama".to_string()),
                PatternData::new("p2".to_string(), vec![0.0; 384], 0.8, 30, "action".to_string()),
            ],
            1,
        );

        let size = delta.estimate_size();
        // Should be roughly: 100 + (2 * 50) + (2 * 384 * 4) = 100 + 100 + 3072 = 3272
        assert!(size > 3000 && size < 4000);
    }
}
