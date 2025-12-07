//! Webhook metrics tracking

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Webhook metrics tracker
#[derive(Debug, Clone)]
pub struct WebhookMetrics {
    received: Arc<AtomicU64>,
    processed: Arc<AtomicU64>,
    failed: Arc<AtomicU64>,
    duplicates: Arc<AtomicU64>,
    rate_limited: Arc<AtomicU64>,
}

impl WebhookMetrics {
    /// Create a new metrics tracker
    pub fn new() -> Self {
        Self {
            received: Arc::new(AtomicU64::new(0)),
            processed: Arc::new(AtomicU64::new(0)),
            failed: Arc::new(AtomicU64::new(0)),
            duplicates: Arc::new(AtomicU64::new(0)),
            rate_limited: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Increment received counter
    pub fn increment_received(&self) {
        self.received.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment processed counter
    pub fn increment_processed(&self) {
        self.processed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment failed counter
    pub fn increment_failed(&self) {
        self.failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment duplicates counter
    pub fn increment_duplicates(&self) {
        self.duplicates.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment rate limited counter
    pub fn increment_rate_limited(&self) {
        self.rate_limited.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            received: self.received.load(Ordering::Relaxed),
            processed: self.processed.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
            duplicates: self.duplicates.load(Ordering::Relaxed),
            rate_limited: self.rate_limited.load(Ordering::Relaxed),
        }
    }

    /// Reset all counters
    pub fn reset(&self) {
        self.received.store(0, Ordering::Relaxed);
        self.processed.store(0, Ordering::Relaxed);
        self.failed.store(0, Ordering::Relaxed);
        self.duplicates.store(0, Ordering::Relaxed);
        self.rate_limited.store(0, Ordering::Relaxed);
    }
}

impl Default for WebhookMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub received: u64,
    pub processed: u64,
    pub failed: u64,
    pub duplicates: u64,
    pub rate_limited: u64,
}

impl MetricsSnapshot {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.received == 0 {
            return 0.0;
        }
        (self.processed as f64 / self.received as f64) * 100.0
    }

    /// Calculate failure rate
    pub fn failure_rate(&self) -> f64 {
        if self.received == 0 {
            return 0.0;
        }
        (self.failed as f64 / self.received as f64) * 100.0
    }

    /// Calculate duplicate rate
    pub fn duplicate_rate(&self) -> f64 {
        if self.received == 0 {
            return 0.0;
        }
        (self.duplicates as f64 / self.received as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_increment() {
        let metrics = WebhookMetrics::new();

        metrics.increment_received();
        metrics.increment_processed();
        metrics.increment_failed();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.received, 1);
        assert_eq!(snapshot.processed, 1);
        assert_eq!(snapshot.failed, 1);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = WebhookMetrics::new();

        metrics.increment_received();
        metrics.increment_processed();

        metrics.reset();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.received, 0);
        assert_eq!(snapshot.processed, 0);
    }

    #[test]
    fn test_success_rate() {
        let snapshot = MetricsSnapshot {
            received: 100,
            processed: 90,
            failed: 10,
            duplicates: 5,
            rate_limited: 2,
        };

        assert_eq!(snapshot.success_rate(), 90.0);
        assert_eq!(snapshot.failure_rate(), 10.0);
        assert_eq!(snapshot.duplicate_rate(), 5.0);
    }

    #[test]
    fn test_zero_division() {
        let snapshot = MetricsSnapshot {
            received: 0,
            processed: 0,
            failed: 0,
            duplicates: 0,
            rate_limited: 0,
        };

        assert_eq!(snapshot.success_rate(), 0.0);
        assert_eq!(snapshot.failure_rate(), 0.0);
    }
}
