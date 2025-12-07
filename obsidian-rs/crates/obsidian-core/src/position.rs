//! Position types for tracking locations in files

use serde::{Deserialize, Serialize};

/// A location in a text document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Location {
    /// Line number (0-indexed)
    pub line: usize,
    /// Column number (0-indexed)
    pub col: usize,
    /// Byte offset from start of document
    pub offset: usize,
}

impl Location {
    /// Create a new location
    pub fn new(line: usize, col: usize, offset: usize) -> Self {
        Self { line, col, offset }
    }

    /// Create a location at the start of the document
    pub fn start() -> Self {
        Self::default()
    }
}

/// A range in a text document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Position {
    /// Start location
    pub start: Location,
    /// End location
    pub end: Location,
}

impl Position {
    /// Create a new position
    pub fn new(start: Location, end: Location) -> Self {
        Self { start, end }
    }

    /// Create a position spanning a single point
    pub fn point(loc: Location) -> Self {
        Self {
            start: loc,
            end: loc,
        }
    }

    /// Get the length of this position in bytes
    pub fn len(&self) -> usize {
        self.end.offset.saturating_sub(self.start.offset)
    }

    /// Check if this position is empty
    pub fn is_empty(&self) -> bool {
        self.start.offset == self.end.offset
    }

    /// Check if this position contains another position
    pub fn contains(&self, other: &Position) -> bool {
        self.start.offset <= other.start.offset && self.end.offset >= other.end.offset
    }

    /// Check if this position overlaps with another
    pub fn overlaps(&self, other: &Position) -> bool {
        self.start.offset < other.end.offset && self.end.offset > other.start.offset
    }
}

/// A range of lines in a document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineRange {
    /// Start line (inclusive, 0-indexed)
    pub start: usize,
    /// End line (exclusive, 0-indexed)
    pub end: usize,
}

impl LineRange {
    /// Create a new line range
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Create a single line range
    pub fn single(line: usize) -> Self {
        Self {
            start: line,
            end: line + 1,
        }
    }

    /// Get the number of lines in this range
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if this range is empty
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location() {
        let loc = Location::new(10, 5, 150);
        assert_eq!(loc.line, 10);
        assert_eq!(loc.col, 5);
        assert_eq!(loc.offset, 150);
    }

    #[test]
    fn test_position_contains() {
        let outer = Position::new(Location::new(0, 0, 0), Location::new(10, 0, 100));

        let inner = Position::new(Location::new(2, 0, 20), Location::new(5, 0, 50));

        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_position_overlaps() {
        let pos1 = Position::new(Location::new(0, 0, 0), Location::new(5, 0, 50));

        let pos2 = Position::new(Location::new(3, 0, 30), Location::new(8, 0, 80));

        assert!(pos1.overlaps(&pos2));
        assert!(pos2.overlaps(&pos1));

        let pos3 = Position::new(Location::new(6, 0, 60), Location::new(10, 0, 100));

        assert!(!pos1.overlaps(&pos3));
    }
}
