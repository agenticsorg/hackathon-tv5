use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a viewing pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PatternId(pub uuid::Uuid);

impl PatternId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for PatternId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PatternId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a device (TV)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(pub String);

impl DeviceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for content (movie, series, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentId(pub String);

impl ContentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Time of day categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeSlot {
    Morning,    // 6am - 12pm
    Afternoon,  // 12pm - 6pm
    Evening,    // 6pm - 10pm
    Night,      // 10pm - 6am
}

impl TimeSlot {
    pub fn from_hour(hour: u32) -> Self {
        match hour {
            6..=11 => TimeSlot::Morning,
            12..=17 => TimeSlot::Afternoon,
            18..=21 => TimeSlot::Evening,
            _ => TimeSlot::Night,
        }
    }
}

/// Day type categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DayType {
    Weekday,
    Weekend,
}

impl DayType {
    pub fn from_weekday(weekday: chrono::Weekday) -> Self {
        match weekday {
            chrono::Weekday::Sat | chrono::Weekday::Sun => DayType::Weekend,
            _ => DayType::Weekday,
        }
    }
}

/// Content type categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentType {
    Movie,
    Series,
    Documentary,
    Live,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentType::Movie => write!(f, "movie"),
            ContentType::Series => write!(f, "series"),
            ContentType::Documentary => write!(f, "documentary"),
            ContentType::Live => write!(f, "live"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_id_creation() {
        let id1 = PatternId::new();
        let id2 = PatternId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_time_slot_from_hour() {
        assert_eq!(TimeSlot::from_hour(8), TimeSlot::Morning);
        assert_eq!(TimeSlot::from_hour(14), TimeSlot::Afternoon);
        assert_eq!(TimeSlot::from_hour(19), TimeSlot::Evening);
        assert_eq!(TimeSlot::from_hour(23), TimeSlot::Night);
    }

    #[test]
    fn test_day_type_from_weekday() {
        assert_eq!(DayType::from_weekday(chrono::Weekday::Mon), DayType::Weekday);
        assert_eq!(DayType::from_weekday(chrono::Weekday::Sat), DayType::Weekend);
    }
}
