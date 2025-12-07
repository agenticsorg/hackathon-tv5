//! Cursor and position management

use serde::{Deserialize, Serialize};

/// A position in the document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CursorPosition {
    /// Line number (0-indexed)
    pub line: usize,
    /// Column number (0-indexed, in grapheme clusters)
    pub column: usize,
}

impl CursorPosition {
    /// Create a new position
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Position at start of document
    pub fn start() -> Self {
        Self::new(0, 0)
    }

    /// Check if this position is before another
    pub fn is_before(&self, other: &Self) -> bool {
        self.line < other.line || (self.line == other.line && self.column < other.column)
    }

    /// Check if this position is after another
    pub fn is_after(&self, other: &Self) -> bool {
        other.is_before(self)
    }

    /// Get the minimum of two positions
    pub fn min(self, other: Self) -> Self {
        if self.is_before(&other) {
            self
        } else {
            other
        }
    }

    /// Get the maximum of two positions
    pub fn max(self, other: Self) -> Self {
        if self.is_after(&other) {
            self
        } else {
            other
        }
    }
}

impl PartialOrd for CursorPosition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CursorPosition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.line.cmp(&other.line) {
            std::cmp::Ordering::Equal => self.column.cmp(&other.column),
            ord => ord,
        }
    }
}

/// Cursor state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor {
    /// Current position
    pub position: CursorPosition,
    /// Anchor position (for selections)
    pub anchor: Option<CursorPosition>,
    /// Preferred column (for vertical movement)
    pub preferred_column: Option<usize>,
    /// Whether the cursor is visible
    pub visible: bool,
    /// Blink state
    pub blink_on: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            position: CursorPosition::start(),
            anchor: None,
            preferred_column: None,
            visible: true,
            blink_on: true,
        }
    }
}

impl Cursor {
    /// Create a new cursor
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a cursor at a specific position
    pub fn at(line: usize, column: usize) -> Self {
        Self {
            position: CursorPosition::new(line, column),
            ..Default::default()
        }
    }

    /// Move the cursor to a position
    pub fn move_to(&mut self, line: usize, column: usize) {
        self.position = CursorPosition::new(line, column);
        self.anchor = None;
        self.preferred_column = None;
    }

    /// Move cursor up
    pub fn move_up(&mut self) {
        if self.position.line > 0 {
            self.position.line -= 1;
            if let Some(col) = self.preferred_column {
                self.position.column = col;
            }
        }
        self.anchor = None;
    }

    /// Move cursor down
    pub fn move_down(&mut self, max_line: usize) {
        if self.position.line < max_line {
            self.position.line += 1;
            if let Some(col) = self.preferred_column {
                self.position.column = col;
            }
        }
        self.anchor = None;
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.position.column > 0 {
            self.position.column -= 1;
        } else if self.position.line > 0 {
            // Move to end of previous line - actual column set by buffer
            self.position.line -= 1;
            self.position.column = usize::MAX; // Marker for "end of line"
        }
        self.anchor = None;
        self.preferred_column = None;
    }

    /// Move cursor right
    pub fn move_right(&mut self, line_len: usize, max_line: usize) {
        if self.position.column < line_len {
            self.position.column += 1;
        } else if self.position.line < max_line {
            self.position.line += 1;
            self.position.column = 0;
        }
        self.anchor = None;
        self.preferred_column = None;
    }

    /// Move to start of line
    pub fn move_to_line_start(&mut self) {
        self.position.column = 0;
        self.anchor = None;
        self.preferred_column = None;
    }

    /// Move to end of line
    pub fn move_to_line_end(&mut self, line_len: usize) {
        self.position.column = line_len;
        self.anchor = None;
        self.preferred_column = None;
    }

    /// Move to start of document
    pub fn move_to_start(&mut self) {
        self.position = CursorPosition::start();
        self.anchor = None;
        self.preferred_column = None;
    }

    /// Move to end of document
    pub fn move_to_end(&mut self, last_line: usize, last_line_len: usize) {
        self.position = CursorPosition::new(last_line, last_line_len);
        self.anchor = None;
        self.preferred_column = None;
    }

    /// Start a selection from current position
    pub fn start_selection(&mut self) {
        self.anchor = Some(self.position);
    }

    /// Extend selection to current position
    pub fn extend_selection(&mut self, line: usize, column: usize) {
        if self.anchor.is_none() {
            self.anchor = Some(self.position);
        }
        self.position = CursorPosition::new(line, column);
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

    /// Check if there is an active selection
    pub fn has_selection(&self) -> bool {
        self.anchor.is_some() && self.anchor != Some(self.position)
    }

    /// Get selection range (start, end)
    pub fn selection_range(&self) -> Option<(CursorPosition, CursorPosition)> {
        self.anchor.map(|anchor| {
            let start = anchor.min(self.position);
            let end = anchor.max(self.position);
            (start, end)
        })
    }

    /// Store current column as preferred (for vertical movement)
    pub fn store_preferred_column(&mut self) {
        self.preferred_column = Some(self.position.column);
    }

    /// Toggle blink state
    pub fn toggle_blink(&mut self) {
        self.blink_on = !self.blink_on;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_position_comparison() {
        let a = CursorPosition::new(0, 5);
        let b = CursorPosition::new(1, 0);
        let c = CursorPosition::new(0, 10);

        assert!(a.is_before(&b));
        assert!(a.is_before(&c));
        assert!(b.is_after(&a));
        assert!(c.is_after(&a));
    }

    #[test]
    fn test_cursor_movement() {
        let mut cursor = Cursor::new();

        cursor.move_to(5, 10);
        assert_eq!(cursor.position.line, 5);
        assert_eq!(cursor.position.column, 10);

        cursor.move_up();
        assert_eq!(cursor.position.line, 4);

        cursor.move_down(10);
        assert_eq!(cursor.position.line, 5);

        cursor.move_to_line_start();
        assert_eq!(cursor.position.column, 0);
    }

    #[test]
    fn test_cursor_selection() {
        let mut cursor = Cursor::at(1, 5);

        cursor.start_selection();
        assert!(cursor.anchor.is_some());

        cursor.extend_selection(1, 15);
        assert!(cursor.has_selection());

        let (start, end) = cursor.selection_range().unwrap();
        assert_eq!(start.column, 5);
        assert_eq!(end.column, 15);

        cursor.clear_selection();
        assert!(!cursor.has_selection());
    }
}
