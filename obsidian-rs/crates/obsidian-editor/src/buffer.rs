//! Text buffer using rope data structure

use crate::cursor::CursorPosition;
use crate::error::{EditorError, EditorResult};
use crate::history::{EditHistory, EditOperation};
use ropey::Rope;
use tracing::debug;

/// Text buffer with rope-based storage
pub struct TextBuffer {
    /// The rope storing text
    rope: Rope,
    /// Edit history
    history: EditHistory,
    /// Whether the buffer is modified
    modified: bool,
    /// File path (if any)
    path: Option<String>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            history: EditHistory::new(),
            modified: false,
            path: None,
        }
    }

    /// Create a buffer from string
    pub fn from_str(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            history: EditHistory::new(),
            modified: false,
            path: None,
        }
    }

    /// Create a buffer from file
    pub fn from_file(path: &str) -> EditorResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut buffer = Self::from_str(&content);
        buffer.path = Some(path.to_string());
        Ok(buffer)
    }

    /// Get the file path
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Set the file path
    pub fn set_path(&mut self, path: impl Into<String>) {
        self.path = Some(path.into());
    }

    /// Check if buffer is modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Mark buffer as saved
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }

    /// Get total length in bytes
    pub fn len(&self) -> usize {
        self.rope.len_bytes()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.rope.len_bytes() == 0
    }

    /// Get number of lines
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// Get a line by index
    pub fn line(&self, idx: usize) -> Option<String> {
        if idx >= self.line_count() {
            return None;
        }
        Some(self.rope.line(idx).to_string())
    }

    /// Get line length (in characters)
    pub fn line_len(&self, idx: usize) -> usize {
        if idx >= self.line_count() {
            return 0;
        }
        let line = self.rope.line(idx);
        let len = line.len_chars();
        // Subtract 1 for newline if present
        if len > 0 && line.char(len - 1) == '\n' {
            len - 1
        } else {
            len
        }
    }

    /// Get all text
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    /// Get text in range
    pub fn text_range(&self, start: CursorPosition, end: CursorPosition) -> EditorResult<String> {
        let start_idx = self.position_to_char_idx(start)?;
        let end_idx = self.position_to_char_idx(end)?;

        if end_idx < start_idx {
            return Err(EditorError::InvalidRange(start_idx, end_idx));
        }

        Ok(self.rope.slice(start_idx..end_idx).to_string())
    }

    /// Convert position to character index
    pub fn position_to_char_idx(&self, pos: CursorPosition) -> EditorResult<usize> {
        if pos.line >= self.line_count() {
            return Err(EditorError::InvalidPosition {
                line: pos.line,
                column: pos.column,
            });
        }

        let line_start = self.rope.line_to_char(pos.line);
        let line_len = self.line_len(pos.line);

        let column = pos.column.min(line_len);
        Ok(line_start + column)
    }

    /// Convert character index to position
    pub fn char_idx_to_position(&self, idx: usize) -> CursorPosition {
        if idx >= self.rope.len_chars() {
            let last_line = self.line_count().saturating_sub(1);
            return CursorPosition::new(last_line, self.line_len(last_line));
        }

        let line = self.rope.char_to_line(idx);
        let line_start = self.rope.line_to_char(line);
        let column = idx - line_start;

        CursorPosition::new(line, column)
    }

    /// Insert text at position
    pub fn insert(&mut self, pos: CursorPosition, text: &str) -> EditorResult<()> {
        let idx = self.position_to_char_idx(pos)?;
        debug!("Inserting '{}' at position {:?} (idx {})", text, pos, idx);

        self.rope.insert(idx, text);
        self.modified = true;

        // Record in history
        self.history.push(EditOperation::insert(pos, text));

        Ok(())
    }

    /// Delete text in range
    pub fn delete(&mut self, start: CursorPosition, end: CursorPosition) -> EditorResult<String> {
        let start_idx = self.position_to_char_idx(start)?;
        let end_idx = self.position_to_char_idx(end)?;

        if end_idx <= start_idx {
            return Ok(String::new());
        }

        let deleted = self.rope.slice(start_idx..end_idx).to_string();
        debug!(
            "Deleting '{}' from {:?} to {:?}",
            deleted, start, end
        );

        self.rope.remove(start_idx..end_idx);
        self.modified = true;

        // Record in history
        self.history
            .push(EditOperation::delete(start, end, &deleted));

        Ok(deleted)
    }

    /// Delete a single character at position
    pub fn delete_char(&mut self, pos: CursorPosition) -> EditorResult<Option<char>> {
        let idx = self.position_to_char_idx(pos)?;

        if idx >= self.rope.len_chars() {
            return Ok(None);
        }

        let ch = self.rope.char(idx);
        let end = CursorPosition::new(
            pos.line + if ch == '\n' { 1 } else { 0 },
            if ch == '\n' { 0 } else { pos.column + 1 },
        );

        self.delete(pos, end)?;
        Ok(Some(ch))
    }

    /// Replace text in range
    pub fn replace(
        &mut self,
        start: CursorPosition,
        end: CursorPosition,
        text: &str,
    ) -> EditorResult<String> {
        let old_text = self.text_range(start, end)?;

        let start_idx = self.position_to_char_idx(start)?;
        let end_idx = self.position_to_char_idx(end)?;

        self.rope.remove(start_idx..end_idx);
        self.rope.insert(start_idx, text);
        self.modified = true;

        // Record in history
        self.history
            .push(EditOperation::replace(start, end, &old_text, text));

        Ok(old_text)
    }

    /// Undo the last operation
    pub fn undo(&mut self) -> EditorResult<CursorPosition> {
        let op = self.history.undo()?;
        self.apply_operation(&op)?;
        Ok(op.cursor_after())
    }

    /// Redo the last undone operation
    pub fn redo(&mut self) -> EditorResult<CursorPosition> {
        let op = self.history.redo()?;
        self.apply_operation(&op)?;
        Ok(op.cursor_after())
    }

    /// Apply an operation without recording in history
    fn apply_operation(&mut self, op: &EditOperation) -> EditorResult<()> {
        match op {
            EditOperation::Insert { position, text } => {
                let idx = self.position_to_char_idx(*position)?;
                self.rope.insert(idx, text);
                self.modified = true;
            }
            EditOperation::Delete { start, end, .. } => {
                let start_idx = self.position_to_char_idx(*start)?;
                let end_idx = self.position_to_char_idx(*end)?;
                if end_idx > start_idx {
                    self.rope.remove(start_idx..end_idx);
                    self.modified = true;
                }
            }
            EditOperation::Replace {
                start, new_text, ..
            } => {
                // For replace, we need to figure out what to remove based on old_text
                // But since this is applying an operation, we just insert at start
                let idx = self.position_to_char_idx(*start)?;
                self.rope.insert(idx, new_text);
                self.modified = true;
            }
            EditOperation::Batch(ops) => {
                for op in ops {
                    self.apply_operation(op)?;
                }
            }
        }
        Ok(())
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Begin a batch operation
    pub fn begin_batch(&mut self) {
        self.history.begin_batch();
    }

    /// End a batch operation
    pub fn end_batch(&mut self) {
        self.history.end_batch();
    }

    /// Clear all content
    pub fn clear(&mut self) {
        if !self.is_empty() {
            let end = CursorPosition::new(
                self.line_count().saturating_sub(1),
                self.line_len(self.line_count().saturating_sub(1)),
            );
            let _ = self.delete(CursorPosition::start(), end);
        }
    }

    /// Save to file
    pub fn save(&mut self) -> EditorResult<()> {
        if let Some(ref path) = self.path {
            std::fs::write(path, self.text())?;
            self.mark_saved();
            Ok(())
        } else {
            Err(EditorError::Buffer("No file path set".to_string()))
        }
    }

    /// Save to a specific file
    pub fn save_as(&mut self, path: &str) -> EditorResult<()> {
        std::fs::write(path, self.text())?;
        self.path = Some(path.to_string());
        self.mark_saved();
        Ok(())
    }

    /// Find all occurrences of a pattern
    pub fn find(&self, pattern: &str) -> Vec<CursorPosition> {
        let text = self.text();
        let mut positions = Vec::new();

        for (byte_idx, _) in text.match_indices(pattern) {
            let char_idx = text[..byte_idx].chars().count();
            positions.push(self.char_idx_to_position(char_idx));
        }

        positions
    }

    /// Find and replace all occurrences
    pub fn find_replace_all(&mut self, pattern: &str, replacement: &str) -> usize {
        let positions = self.find(pattern);
        let count = positions.len();

        if count == 0 {
            return 0;
        }

        self.begin_batch();

        // Replace in reverse order to not invalidate positions
        for pos in positions.into_iter().rev() {
            let end = CursorPosition::new(pos.line, pos.column + pattern.chars().count());
            let _ = self.replace(pos, end, replacement);
        }

        self.end_batch();
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer = TextBuffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.line_count(), 1);
    }

    #[test]
    fn test_from_str() {
        let buffer = TextBuffer::from_str("Hello\nWorld");
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.line(0), Some("Hello\n".to_string()));
        assert_eq!(buffer.line(1), Some("World".to_string()));
    }

    #[test]
    fn test_insert() {
        let mut buffer = TextBuffer::new();

        buffer.insert(CursorPosition::new(0, 0), "Hello").unwrap();
        assert_eq!(buffer.text(), "Hello");

        buffer.insert(CursorPosition::new(0, 5), " World").unwrap();
        assert_eq!(buffer.text(), "Hello World");
    }

    #[test]
    fn test_delete() {
        let mut buffer = TextBuffer::from_str("Hello World");

        let deleted = buffer
            .delete(CursorPosition::new(0, 5), CursorPosition::new(0, 11))
            .unwrap();

        assert_eq!(deleted, " World");
        assert_eq!(buffer.text(), "Hello");
    }

    #[test]
    fn test_replace() {
        let mut buffer = TextBuffer::from_str("Hello World");

        let old = buffer
            .replace(
                CursorPosition::new(0, 6),
                CursorPosition::new(0, 11),
                "Rust",
            )
            .unwrap();

        assert_eq!(old, "World");
        assert_eq!(buffer.text(), "Hello Rust");
    }

    #[test]
    fn test_undo_redo() {
        let mut buffer = TextBuffer::new();

        buffer.insert(CursorPosition::new(0, 0), "Hello").unwrap();
        assert_eq!(buffer.text(), "Hello");

        buffer.undo().unwrap();
        assert_eq!(buffer.text(), "");

        buffer.redo().unwrap();
        assert_eq!(buffer.text(), "Hello");
    }

    #[test]
    fn test_find() {
        let buffer = TextBuffer::from_str("Hello Hello World Hello");
        let positions = buffer.find("Hello");

        assert_eq!(positions.len(), 3);
        assert_eq!(positions[0].column, 0);
        assert_eq!(positions[1].column, 6);
        assert_eq!(positions[2].column, 18);
    }

    #[test]
    fn test_find_replace_all() {
        let mut buffer = TextBuffer::from_str("Hello Hello World Hello");
        let count = buffer.find_replace_all("Hello", "Hi");

        assert_eq!(count, 3);
        assert_eq!(buffer.text(), "Hi Hi World Hi");
    }

    #[test]
    fn test_multiline() {
        let mut buffer = TextBuffer::from_str("Line 1\nLine 2\nLine 3");

        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.line_len(0), 6);
        assert_eq!(buffer.line_len(1), 6);
        assert_eq!(buffer.line_len(2), 6);

        // Insert newline
        buffer.insert(CursorPosition::new(1, 6), "\nNew Line").unwrap();
        assert_eq!(buffer.line_count(), 4);
    }

    #[test]
    fn test_position_conversion() {
        let buffer = TextBuffer::from_str("Hello\nWorld");

        let idx = buffer.position_to_char_idx(CursorPosition::new(1, 3)).unwrap();
        let pos = buffer.char_idx_to_position(idx);

        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 3);
    }
}
