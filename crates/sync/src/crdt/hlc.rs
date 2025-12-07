use serde::{Deserialize, Serialize};
/// Hybrid Logical Clock implementation for distributed timestamp ordering
///
/// Provides total ordering of events without physical clock synchronization
/// Format: 48-bit physical time (microseconds) + 16-bit logical counter
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// HLC timestamp combining physical and logical time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HLCTimestamp(pub i64);

impl HLCTimestamp {
    /// Create a new HLCTimestamp with physical time and logical counter
    /// The device_id is included for compatibility but not stored in the timestamp
    pub fn new(physical: u64, logical: u16, _device_id: String) -> Self {
        Self::from_components(physical as i64, logical)
    }

    /// Extract physical time component (microseconds since epoch)
    pub fn physical_time(&self) -> i64 {
        self.0 >> 16
    }

    /// Extract logical counter component
    pub fn logical_counter(&self) -> u16 {
        (self.0 & 0xFFFF) as u16
    }

    /// Create from components
    pub fn from_components(physical: i64, logical: u16) -> Self {
        Self((physical << 16) | (logical as i64 & 0xFFFF))
    }
}

/// Hybrid Logical Clock
pub struct HybridLogicalClock {
    /// Logical counter
    logical: AtomicI64,

    /// Last physical timestamp (microseconds)
    last_physical: AtomicI64,
}

impl HybridLogicalClock {
    /// Create new HLC instance
    pub fn new() -> Self {
        Self {
            logical: AtomicI64::new(0),
            last_physical: AtomicI64::new(0),
        }
    }

    /// Generate new HLC timestamp for local event
    pub fn now(&self) -> HLCTimestamp {
        // Get current physical time (microseconds since UNIX epoch)
        let physical = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_micros() as i64;

        let last_physical = self.last_physical.load(Ordering::SeqCst);
        let logical = self.logical.load(Ordering::SeqCst);

        let (new_physical, new_logical) = if physical > last_physical {
            // Physical clock advanced, reset logical counter
            (physical, 0)
        } else {
            // Physical clock same or behind, increment logical counter
            (last_physical, logical + 1)
        };

        // Update stored values
        self.last_physical.store(new_physical, Ordering::SeqCst);
        self.logical.store(new_logical, Ordering::SeqCst);

        HLCTimestamp::from_components(new_physical, new_logical as u16)
    }

    /// Update clock based on received timestamp from another node
    pub fn update(&self, received: HLCTimestamp) {
        let received_physical = received.physical_time();
        let received_logical = received.logical_counter() as i64;

        let physical = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_micros() as i64;

        let last_physical = self.last_physical.load(Ordering::SeqCst);
        let logical = self.logical.load(Ordering::SeqCst);

        let (new_physical, new_logical) =
            if physical > last_physical && physical > received_physical {
                // Local physical clock is newest
                (physical, 0)
            } else if received_physical > last_physical {
                // Received timestamp is newer
                (received_physical, received_logical + 1)
            } else {
                // Same physical time, increment logical counter
                (last_physical, std::cmp::max(logical, received_logical) + 1)
            };

        self.last_physical.store(new_physical, Ordering::SeqCst);
        self.logical.store(new_logical, Ordering::SeqCst);
    }

    /// Compare two timestamps (for ordering)
    pub fn compare(a: HLCTimestamp, b: HLCTimestamp) -> std::cmp::Ordering {
        a.cmp(&b)
    }
}

impl Default for HybridLogicalClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hlc_monotonic() {
        let hlc = HybridLogicalClock::new();

        let t1 = hlc.now();
        let t2 = hlc.now();
        let t3 = hlc.now();

        assert!(t2 > t1);
        assert!(t3 > t2);
    }

    #[test]
    fn test_hlc_update() {
        let hlc1 = HybridLogicalClock::new();
        let hlc2 = HybridLogicalClock::new();

        let t1 = hlc1.now();
        hlc2.update(t1);

        let t2 = hlc2.now();
        assert!(t2 > t1);
    }

    #[test]
    fn test_timestamp_components() {
        let ts = HLCTimestamp::from_components(1000, 5);
        assert_eq!(ts.physical_time(), 1000);
        assert_eq!(ts.logical_counter(), 5);
    }
}
