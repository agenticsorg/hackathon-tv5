//! Undo/redo history

use crate::cursor::CursorPosition;
use crate::error::{EditorError, EditorResult};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// An edit operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditOperation {
    /// Insert text at position
    Insert {
        position: CursorPosition,
        text: String,
    },
    /// Delete text in range
    Delete {
        start: CursorPosition,
        end: CursorPosition,
        deleted_text: String,
    },
    /// Replace text in range
    Replace {
        start: CursorPosition,
        end: CursorPosition,
        old_text: String,
        new_text: String,
    },
    /// Batch of operations
    Batch(Vec<EditOperation>),
}

impl EditOperation {
    /// Create an insert operation
    pub fn insert(position: CursorPosition, text: impl Into<String>) -> Self {
        Self::Insert {
            position,
            text: text.into(),
        }
    }

    /// Create a delete operation
    pub fn delete(
        start: CursorPosition,
        end: CursorPosition,
        deleted_text: impl Into<String>,
    ) -> Self {
        Self::Delete {
            start,
            end,
            deleted_text: deleted_text.into(),
        }
    }

    /// Create a replace operation
    pub fn replace(
        start: CursorPosition,
        end: CursorPosition,
        old_text: impl Into<String>,
        new_text: impl Into<String>,
    ) -> Self {
        Self::Replace {
            start,
            end,
            old_text: old_text.into(),
            new_text: new_text.into(),
        }
    }

    /// Create a batch operation
    pub fn batch(operations: Vec<EditOperation>) -> Self {
        Self::Batch(operations)
    }

    /// Get the inverse operation (for undo)
    pub fn inverse(&self) -> Self {
        match self {
            Self::Insert { position, text } => {
                // To undo an insert, delete the inserted text
                let end = calculate_end_position(*position, text);
                Self::Delete {
                    start: *position,
                    end,
                    deleted_text: text.clone(),
                }
            }
            Self::Delete {
                start,
                deleted_text,
                ..
            } => {
                // To undo a delete, insert the deleted text back
                Self::Insert {
                    position: *start,
                    text: deleted_text.clone(),
                }
            }
            Self::Replace {
                start,
                old_text,
                new_text,
                ..
            } => {
                // To undo a replace, replace with the old text
                let end = calculate_end_position(*start, new_text);
                Self::Replace {
                    start: *start,
                    end,
                    old_text: new_text.clone(),
                    new_text: old_text.clone(),
                }
            }
            Self::Batch(ops) => {
                // Reverse the batch and invert each operation
                Self::Batch(ops.iter().rev().map(|op| op.inverse()).collect())
            }
        }
    }

    /// Get the cursor position after this operation
    pub fn cursor_after(&self) -> CursorPosition {
        match self {
            Self::Insert { position, text } => calculate_end_position(*position, text),
            Self::Delete { start, .. } => *start,
            Self::Replace { start, new_text, .. } => calculate_end_position(*start, new_text),
            Self::Batch(ops) => ops
                .last()
                .map(|op| op.cursor_after())
                .unwrap_or_default(),
        }
    }
}

/// Calculate end position after inserting text
fn calculate_end_position(start: CursorPosition, text: &str) -> CursorPosition {
    let lines: Vec<&str> = text.split('\n').collect();

    if lines.len() == 1 {
        CursorPosition::new(start.line, start.column + text.len())
    } else {
        CursorPosition::new(
            start.line + lines.len() - 1,
            lines.last().map(|l| l.len()).unwrap_or(0),
        )
    }
}

/// A history entry with timestamp
#[derive(Debug, Clone)]
struct HistoryEntry {
    operation: EditOperation,
    timestamp: Instant,
}

/// Edit history with undo/redo
pub struct EditHistory {
    /// Undo stack
    undo_stack: Vec<HistoryEntry>,
    /// Redo stack
    redo_stack: Vec<HistoryEntry>,
    /// Maximum history size
    max_size: usize,
    /// Merge threshold (operations within this time are merged)
    merge_threshold: Duration,
    /// Current batch (for grouping operations)
    current_batch: Option<Vec<EditOperation>>,
}

impl Default for EditHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl EditHistory {
    /// Create a new history
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size: 1000,
            merge_threshold: Duration::from_millis(300),
            current_batch: None,
        }
    }

    /// Set maximum history size
    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }

    /// Set merge threshold
    pub fn with_merge_threshold(mut self, threshold: Duration) -> Self {
        self.merge_threshold = threshold;
        self
    }

    /// Push an operation to history
    pub fn push(&mut self, operation: EditOperation) {
        // Clear redo stack on new operation
        self.redo_stack.clear();

        // If we're in a batch, add to batch
        if let Some(ref mut batch) = self.current_batch {
            batch.push(operation);
            return;
        }

        let now = Instant::now();

        // Check if we should merge with previous operation
        if let Some(last) = self.undo_stack.last() {
            if now.duration_since(last.timestamp) < self.merge_threshold {
                // Try to merge
                if let Some(merged) = self.try_merge(&last.operation, &operation) {
                    self.undo_stack.pop();
                    self.undo_stack.push(HistoryEntry {
                        operation: merged,
                        timestamp: now,
                    });
                    return;
                }
            }
        }

        // Add as new entry
        self.undo_stack.push(HistoryEntry {
            operation,
            timestamp: now,
        });

        // Trim if over max size
        while self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
    }

    /// Try to merge two operations
    fn try_merge(&self, prev: &EditOperation, curr: &EditOperation) -> Option<EditOperation> {
        match (prev, curr) {
            // Merge consecutive inserts at the same position
            (
                EditOperation::Insert {
                    position: pos1,
                    text: text1,
                },
                EditOperation::Insert {
                    position: pos2,
                    text: text2,
                },
            ) => {
                let end_pos = calculate_end_position(*pos1, text1);
                if end_pos == *pos2 && !text1.contains('\n') && !text2.contains('\n') {
                    Some(EditOperation::Insert {
                        position: *pos1,
                        text: format!("{}{}", text1, text2),
                    })
                } else {
                    None
                }
            }
            // Merge consecutive deletes
            (
                EditOperation::Delete {
                    start: start1,
                    end: _,
                    deleted_text: text1,
                },
                EditOperation::Delete {
                    start: start2,
                    end: _,
                    deleted_text: text2,
                },
            ) => {
                // Forward delete
                if start1 == start2 {
                    Some(EditOperation::Delete {
                        start: *start1,
                        end: calculate_end_position(*start1, &format!("{}{}", text1, text2)),
                        deleted_text: format!("{}{}", text1, text2),
                    })
                }
                // Backspace delete
                else if *start2 == CursorPosition::new(start1.line, start1.column.saturating_sub(text2.len())) {
                    Some(EditOperation::Delete {
                        start: *start2,
                        end: calculate_end_position(*start2, &format!("{}{}", text2, text1)),
                        deleted_text: format!("{}{}", text2, text1),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Start a batch (group multiple operations)
    pub fn begin_batch(&mut self) {
        self.current_batch = Some(Vec::new());
    }

    /// End and commit the current batch
    pub fn end_batch(&mut self) {
        if let Some(batch) = self.current_batch.take() {
            if !batch.is_empty() {
                self.redo_stack.clear();
                self.undo_stack.push(HistoryEntry {
                    operation: EditOperation::Batch(batch),
                    timestamp: Instant::now(),
                });
            }
        }
    }

    /// Cancel the current batch
    pub fn cancel_batch(&mut self) {
        self.current_batch = None;
    }

    /// Get the operation to undo
    pub fn undo(&mut self) -> EditorResult<EditOperation> {
        let entry = self.undo_stack.pop().ok_or(EditorError::NothingToUndo)?;

        let inverse = entry.operation.inverse();
        self.redo_stack.push(HistoryEntry {
            operation: entry.operation,
            timestamp: Instant::now(),
        });

        Ok(inverse)
    }

    /// Get the operation to redo
    pub fn redo(&mut self) -> EditorResult<EditOperation> {
        let entry = self.redo_stack.pop().ok_or(EditorError::NothingToRedo)?;

        self.undo_stack.push(HistoryEntry {
            operation: entry.operation.clone(),
            timestamp: Instant::now(),
        });

        Ok(entry.operation)
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_batch = None;
    }

    /// Get undo stack size
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get redo stack size
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_operation_inverse() {
        let insert = EditOperation::insert(CursorPosition::new(0, 0), "hello");
        let inverse = insert.inverse();

        if let EditOperation::Delete {
            start, deleted_text, ..
        } = inverse
        {
            assert_eq!(start.line, 0);
            assert_eq!(start.column, 0);
            assert_eq!(deleted_text, "hello");
        } else {
            panic!("Expected Delete operation");
        }
    }

    #[test]
    fn test_history_undo_redo() {
        let mut history = EditHistory::new();

        let op = EditOperation::insert(CursorPosition::new(0, 0), "hello");
        history.push(op);

        assert!(history.can_undo());
        assert!(!history.can_redo());

        let undo_op = history.undo().unwrap();
        assert!(matches!(undo_op, EditOperation::Delete { .. }));

        assert!(!history.can_undo());
        assert!(history.can_redo());

        let redo_op = history.redo().unwrap();
        assert!(matches!(redo_op, EditOperation::Insert { .. }));
    }

    #[test]
    fn test_history_batch() {
        let mut history = EditHistory::new();

        history.begin_batch();
        history.push(EditOperation::insert(CursorPosition::new(0, 0), "a"));
        history.push(EditOperation::insert(CursorPosition::new(0, 1), "b"));
        history.end_batch();

        // Should be a single undo
        assert_eq!(history.undo_count(), 1);

        let undo_op = history.undo().unwrap();
        assert!(matches!(undo_op, EditOperation::Batch(_)));
    }

    #[test]
    fn test_calculate_end_position() {
        let start = CursorPosition::new(0, 0);

        // Single line
        let end = calculate_end_position(start, "hello");
        assert_eq!(end.line, 0);
        assert_eq!(end.column, 5);

        // Multi-line
        let end = calculate_end_position(start, "hello\nworld");
        assert_eq!(end.line, 1);
        assert_eq!(end.column, 5);
    }
}
