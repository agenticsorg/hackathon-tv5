//! Selection management

use crate::cursor::CursorPosition;
use serde::{Deserialize, Serialize};

/// A range in the document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionRange {
    /// Start position
    pub start: CursorPosition,
    /// End position
    pub end: CursorPosition,
}

impl SelectionRange {
    /// Create a new range
    pub fn new(start: CursorPosition, end: CursorPosition) -> Self {
        // Ensure start <= end
        let (start, end) = if start.is_before(&end) {
            (start, end)
        } else {
            (end, start)
        };

        Self { start, end }
    }

    /// Create a range from line/column coordinates
    pub fn from_coords(
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Self {
        Self::new(
            CursorPosition::new(start_line, start_col),
            CursorPosition::new(end_line, end_col),
        )
    }

    /// Create a single-point range (for cursor without selection)
    pub fn point(pos: CursorPosition) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }

    /// Check if this is an empty range (same start and end)
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Check if a position is within this range
    pub fn contains(&self, pos: CursorPosition) -> bool {
        !pos.is_before(&self.start) && pos.is_before(&self.end)
    }

    /// Check if this range overlaps with another
    pub fn overlaps(&self, other: &Self) -> bool {
        !(self.end.is_before(&other.start) || other.end.is_before(&self.start))
    }

    /// Check if this range is on a single line
    pub fn is_single_line(&self) -> bool {
        self.start.line == self.end.line
    }

    /// Get the number of lines spanned
    pub fn line_count(&self) -> usize {
        self.end.line - self.start.line + 1
    }

    /// Expand range to include a position
    pub fn expand_to(&self, pos: CursorPosition) -> Self {
        let start = self.start.min(pos);
        let end = self.end.max(pos);
        Self { start, end }
    }

    /// Merge with another range
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// Multi-cursor selection support
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Selection {
    /// All selection ranges
    ranges: Vec<SelectionRange>,
    /// Primary (active) selection index
    primary: usize,
}

impl Selection {
    /// Create a new empty selection
    pub fn new() -> Self {
        Self {
            ranges: Vec::new(),
            primary: 0,
        }
    }

    /// Create a selection with a single range
    pub fn single(range: SelectionRange) -> Self {
        Self {
            ranges: vec![range],
            primary: 0,
        }
    }

    /// Create a selection at a point
    pub fn at(pos: CursorPosition) -> Self {
        Self::single(SelectionRange::point(pos))
    }

    /// Add a range to the selection
    pub fn add(&mut self, range: SelectionRange) {
        // Check for overlaps and merge
        let mut merged = range;
        let mut indices_to_remove = Vec::new();

        for (i, existing) in self.ranges.iter().enumerate() {
            if merged.overlaps(existing) {
                merged = merged.merge(existing);
                indices_to_remove.push(i);
            }
        }

        // Remove overlapping ranges in reverse order
        for i in indices_to_remove.into_iter().rev() {
            self.ranges.remove(i);
            if self.primary > i {
                self.primary = self.primary.saturating_sub(1);
            }
        }

        self.ranges.push(merged);
        self.primary = self.ranges.len() - 1;
    }

    /// Set to a single range, clearing others
    pub fn set(&mut self, range: SelectionRange) {
        self.ranges.clear();
        self.ranges.push(range);
        self.primary = 0;
    }

    /// Clear all selections
    pub fn clear(&mut self) {
        self.ranges.clear();
        self.primary = 0;
    }

    /// Get the primary selection range
    pub fn primary_range(&self) -> Option<&SelectionRange> {
        self.ranges.get(self.primary)
    }

    /// Get all selection ranges
    pub fn ranges(&self) -> &[SelectionRange] {
        &self.ranges
    }

    /// Get number of selections
    pub fn count(&self) -> usize {
        self.ranges.len()
    }

    /// Check if there are any selections
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// Check if any selection has content (non-empty range)
    pub fn has_content(&self) -> bool {
        self.ranges.iter().any(|r| !r.is_empty())
    }

    /// Get all lines that are part of any selection
    pub fn selected_lines(&self) -> Vec<usize> {
        let mut lines: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();

        for range in &self.ranges {
            for line in range.start.line..=range.end.line {
                lines.insert(line);
            }
        }

        lines.into_iter().collect()
    }

    /// Cycle to next selection
    pub fn next(&mut self) {
        if !self.ranges.is_empty() {
            self.primary = (self.primary + 1) % self.ranges.len();
        }
    }

    /// Cycle to previous selection
    pub fn prev(&mut self) {
        if !self.ranges.is_empty() {
            self.primary = if self.primary == 0 {
                self.ranges.len() - 1
            } else {
                self.primary - 1
            };
        }
    }

    /// Sort ranges by position
    pub fn sort(&mut self) {
        self.ranges.sort_by(|a, b| a.start.cmp(&b.start));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_range() {
        let range = SelectionRange::from_coords(0, 5, 1, 10);

        assert!(!range.is_empty());
        assert!(!range.is_single_line());
        assert_eq!(range.line_count(), 2);

        let pos = CursorPosition::new(0, 7);
        assert!(range.contains(pos));
    }

    #[test]
    fn test_selection_range_overlap() {
        let range1 = SelectionRange::from_coords(0, 0, 0, 10);
        let range2 = SelectionRange::from_coords(0, 5, 0, 15);
        let range3 = SelectionRange::from_coords(1, 0, 1, 10);

        assert!(range1.overlaps(&range2));
        assert!(!range1.overlaps(&range3));
    }

    #[test]
    fn test_selection_merge() {
        let mut selection = Selection::new();

        selection.add(SelectionRange::from_coords(0, 0, 0, 10));
        selection.add(SelectionRange::from_coords(0, 5, 0, 20));

        // Should merge into one
        assert_eq!(selection.count(), 1);

        let range = selection.primary_range().unwrap();
        assert_eq!(range.start.column, 0);
        assert_eq!(range.end.column, 20);
    }

    #[test]
    fn test_selected_lines() {
        let mut selection = Selection::new();
        selection.add(SelectionRange::from_coords(2, 0, 5, 0));
        selection.add(SelectionRange::from_coords(8, 0, 10, 0));

        let lines = selection.selected_lines();
        assert!(lines.contains(&2));
        assert!(lines.contains(&5));
        assert!(lines.contains(&8));
        assert!(lines.contains(&10));
        assert!(!lines.contains(&6));
    }
}
