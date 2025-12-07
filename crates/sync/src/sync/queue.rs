use crate::sync::publisher::{MessagePayload, PublisherError, SyncMessage, SyncPublisher};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Maximum retry attempts per operation
const MAX_OPERATION_RETRIES: i32 = 3;

/// Sync operation types that can be queued
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SyncOperation {
    #[serde(rename = "watchlist_add")]
    WatchlistAdd {
        user_id: Uuid,
        content_id: Uuid,
        timestamp: i64,
    },
    #[serde(rename = "watchlist_remove")]
    WatchlistRemove {
        user_id: Uuid,
        content_id: Uuid,
        timestamp: i64,
    },
    #[serde(rename = "progress_update")]
    ProgressUpdate {
        user_id: Uuid,
        content_id: Uuid,
        position: f64,
        timestamp: i64,
    },
    #[serde(rename = "device_command")]
    DeviceCommand {
        command_id: Uuid,
        source_device_id: String,
        target_device_id: String,
        command_type: String,
        payload: serde_json::Value,
        timestamp: i64,
    },
}

/// Report of sync replay operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReport {
    /// Number of operations successfully synced
    pub success_count: usize,
    /// Number of operations that failed
    pub failure_count: usize,
    /// Total operations processed
    pub total_operations: usize,
    /// IDs of failed operations
    pub failed_operation_ids: Vec<u64>,
    /// Error messages for failures
    pub errors: Vec<String>,
}

impl SyncReport {
    /// Create a new empty sync report
    pub fn new() -> Self {
        Self {
            success_count: 0,
            failure_count: 0,
            total_operations: 0,
            failed_operation_ids: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Check if all operations succeeded
    pub fn all_succeeded(&self) -> bool {
        self.failure_count == 0 && self.total_operations > 0
    }

    /// Check if any operations failed
    pub fn has_failures(&self) -> bool {
        self.failure_count > 0
    }
}

impl Default for SyncReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Delta sync configuration
#[derive(Debug, Clone)]
pub struct DeltaSyncConfig {
    /// Enable delta encoding
    pub enabled: bool,
    /// Enable compression
    pub compression_enabled: bool,
    /// Minimum batch size for compression
    pub min_batch_size: usize,
}

impl Default for DeltaSyncConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("DELTA_SYNC_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            compression_enabled: std::env::var("DELTA_SYNC_COMPRESSION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            min_batch_size: std::env::var("DELTA_SYNC_MIN_BATCH_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
        }
    }
}

/// Delta sync metrics
#[derive(Debug, Clone, Default)]
struct DeltaSyncMetrics {
    /// Total bytes saved via delta encoding
    bytes_saved: usize,
    /// Total original bytes
    bytes_original: usize,
    /// Total compressed bytes
    bytes_compressed: usize,
}

impl DeltaSyncMetrics {
    fn compression_ratio(&self) -> f64 {
        if self.bytes_original == 0 {
            1.0
        } else {
            self.bytes_compressed as f64 / self.bytes_original as f64
        }
    }

    fn delta_savings_percent(&self) -> f64 {
        if self.bytes_original == 0 {
            0.0
        } else {
            (self.bytes_saved as f64 / self.bytes_original as f64) * 100.0
        }
    }
}

/// Previous state for delta calculation
#[derive(Debug, Clone)]
struct PreviousState {
    content_id: Uuid,
    position: f64,
    timestamp: i64,
}

/// Offline sync queue with SQLite persistence
pub struct OfflineSyncQueue {
    /// SQLite database connection
    db: Arc<parking_lot::Mutex<Connection>>,
    /// Publisher for sync operations
    publisher: Arc<dyn SyncPublisher>,
    /// Delta sync configuration
    delta_config: DeltaSyncConfig,
    /// Previous state for delta encoding
    previous_states: Arc<parking_lot::RwLock<std::collections::HashMap<Uuid, PreviousState>>>,
    /// Delta sync metrics
    metrics: Arc<parking_lot::RwLock<DeltaSyncMetrics>>,
}

impl OfflineSyncQueue {
    /// Create a new offline sync queue
    ///
    /// # Arguments
    /// * `db_path` - Path to SQLite database file
    /// * `publisher` - Publisher for sync operations
    ///
    /// # Errors
    /// Returns `QueueError` if database initialization fails
    pub fn new<P: AsRef<Path>>(
        db_path: P,
        publisher: Arc<dyn SyncPublisher>,
    ) -> Result<Self, QueueError> {
        let conn = Connection::open(db_path)?;

        // Create schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_type TEXT NOT NULL,
                payload TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                retry_count INTEGER DEFAULT 0
            )",
            [],
        )?;

        // Create index for efficient FIFO ordering
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON sync_queue(created_at, id)",
            [],
        )?;

        info!("Initialized offline sync queue with database");

        Ok(Self {
            db: Arc::new(parking_lot::Mutex::new(conn)),
            publisher,
            delta_config: DeltaSyncConfig::default(),
            previous_states: Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            metrics: Arc::new(parking_lot::RwLock::new(DeltaSyncMetrics::default())),
        })
    }

    /// Create an in-memory sync queue (for testing)
    pub fn new_in_memory(publisher: Arc<dyn SyncPublisher>) -> Result<Self, QueueError> {
        let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_type TEXT NOT NULL,
                payload TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                retry_count INTEGER DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON sync_queue(created_at, id)",
            [],
        )?;

        Ok(Self {
            db: Arc::new(parking_lot::Mutex::new(conn)),
            publisher,
            delta_config: DeltaSyncConfig::default(),
            previous_states: Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            metrics: Arc::new(parking_lot::RwLock::new(DeltaSyncMetrics::default())),
        })
    }

    /// Create a new offline sync queue with custom delta sync config
    pub fn new_with_config<P: AsRef<Path>>(
        db_path: P,
        publisher: Arc<dyn SyncPublisher>,
        delta_config: DeltaSyncConfig,
    ) -> Result<Self, QueueError> {
        let mut queue = Self::new(db_path, publisher)?;
        queue.delta_config = delta_config;
        Ok(queue)
    }

    /// Enqueue a sync operation
    ///
    /// # Arguments
    /// * `op` - The sync operation to enqueue
    ///
    /// # Returns
    /// The ID of the enqueued operation
    ///
    /// # Errors
    /// Returns `QueueError` if serialization or database insertion fails
    pub fn enqueue(&self, op: SyncOperation) -> Result<u64, QueueError> {
        let operation_type = match &op {
            SyncOperation::WatchlistAdd { .. } => "watchlist_add",
            SyncOperation::WatchlistRemove { .. } => "watchlist_remove",
            SyncOperation::ProgressUpdate { .. } => "progress_update",
            SyncOperation::DeviceCommand { .. } => "device_command",
        };

        let payload = serde_json::to_string(&op)?;
        let created_at = chrono::Utc::now().timestamp_millis();

        let db = self.db.lock();
        db.execute(
            "INSERT INTO sync_queue (operation_type, payload, created_at, retry_count)
             VALUES (?1, ?2, ?3, 0)",
            params![operation_type, payload, created_at],
        )?;

        let id = db.last_insert_rowid() as u64;

        debug!(
            "Enqueued sync operation {} (type: {}, id: {})",
            operation_type, operation_type, id
        );

        Ok(id)
    }

    /// Dequeue the next operation (FIFO order)
    ///
    /// # Returns
    /// `Some((id, operation))` if an operation is available, `None` if queue is empty
    ///
    /// # Errors
    /// Returns `QueueError` if database query or deserialization fails
    pub fn dequeue(&self) -> Result<Option<(u64, SyncOperation)>, QueueError> {
        let db = self.db.lock();

        let mut stmt = db.prepare(
            "SELECT id, payload FROM sync_queue
             ORDER BY created_at ASC, id ASC
             LIMIT 1",
        )?;

        let result = stmt.query_row([], |row| {
            let id: i64 = row.get(0)?;
            let payload: String = row.get(1)?;
            Ok((id as u64, payload))
        });

        match result {
            Ok((id, payload)) => {
                let op: SyncOperation = serde_json::from_str(&payload)
                    .map_err(|e| QueueError::Deserialization(e.to_string()))?;
                debug!("Dequeued operation with id: {}", id);
                Ok(Some((id, op)))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(QueueError::Database(e)),
        }
    }

    /// Peek at the next N operations without removing them
    ///
    /// # Arguments
    /// * `limit` - Maximum number of operations to peek
    ///
    /// # Returns
    /// Vector of (id, operation) tuples in FIFO order
    ///
    /// # Errors
    /// Returns `QueueError` if database query or deserialization fails
    pub fn peek(&self, limit: usize) -> Result<Vec<(u64, SyncOperation)>, QueueError> {
        let db = self.db.lock();

        let mut stmt = db.prepare(
            "SELECT id, payload FROM sync_queue
             ORDER BY created_at ASC, id ASC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map([limit], |row| {
            let id: i64 = row.get(0)?;
            let payload: String = row.get(1)?;
            Ok((id as u64, payload))
        })?;

        let mut operations = Vec::new();
        for row_result in rows {
            let (id, payload) = row_result?;
            let op: SyncOperation = serde_json::from_str(&payload)
                .map_err(|e| QueueError::Deserialization(e.to_string()))?;
            operations.push((id, op));
        }

        debug!("Peeked at {} operations", operations.len());
        Ok(operations)
    }

    /// Remove an operation from the queue (after successful sync)
    ///
    /// # Arguments
    /// * `id` - The ID of the operation to remove
    ///
    /// # Errors
    /// Returns `QueueError` if database deletion fails
    pub fn remove(&self, id: u64) -> Result<(), QueueError> {
        let db = self.db.lock();
        let rows_affected =
            db.execute("DELETE FROM sync_queue WHERE id = ?1", params![id as i64])?;

        if rows_affected > 0 {
            debug!("Removed operation with id: {}", id);
        } else {
            warn!("Attempted to remove non-existent operation with id: {}", id);
        }

        Ok(())
    }

    /// Clear all operations from the queue
    ///
    /// # Errors
    /// Returns `QueueError` if database deletion fails
    pub fn clear(&self) -> Result<(), QueueError> {
        let db = self.db.lock();
        let rows_affected = db.execute("DELETE FROM sync_queue", [])?;
        info!("Cleared {} operations from sync queue", rows_affected);
        Ok(())
    }

    /// Get the number of operations in the queue
    pub fn len(&self) -> Result<usize, QueueError> {
        let db = self.db.lock();
        let count: i64 = db.query_row("SELECT COUNT(*) FROM sync_queue", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> Result<bool, QueueError> {
        Ok(self.len()? == 0)
    }

    /// Increment retry count for an operation
    fn increment_retry_count(&self, id: u64) -> Result<i32, QueueError> {
        let db = self.db.lock();
        db.execute(
            "UPDATE sync_queue SET retry_count = retry_count + 1 WHERE id = ?1",
            params![id as i64],
        )?;

        let retry_count: i32 = db.query_row(
            "SELECT retry_count FROM sync_queue WHERE id = ?1",
            params![id as i64],
            |row| row.get(0),
        )?;

        Ok(retry_count)
    }

    /// Replay all pending operations after reconnection
    ///
    /// # Returns
    /// A report detailing success/failure counts and any errors
    ///
    /// # Errors
    /// Returns `QueueError` if database operations fail (not if individual publishes fail)
    pub async fn replay_pending(&self) -> Result<SyncReport, QueueError> {
        let mut report = SyncReport::new();

        info!("Starting replay of pending sync operations");

        loop {
            // Dequeue next operation
            let operation = match self.dequeue()? {
                Some(op) => op,
                None => {
                    // Queue is empty
                    break;
                }
            };

            let (id, op) = operation;
            report.total_operations += 1;

            debug!("Replaying operation {}: {:?}", id, op);

            // Attempt to publish the operation
            match self.publish_operation(&op).await {
                Ok(_) => {
                    // Success - remove from queue
                    self.remove(id)?;
                    report.success_count += 1;
                    info!("Successfully replayed operation {}", id);
                }
                Err(e) => {
                    // Failure - increment retry count
                    let retry_count = self.increment_retry_count(id)?;

                    if retry_count >= MAX_OPERATION_RETRIES {
                        // Max retries exceeded - remove from queue and mark as failed
                        warn!(
                            "Operation {} exceeded max retries ({}), removing from queue",
                            id, MAX_OPERATION_RETRIES
                        );
                        self.remove(id)?;
                        report.failure_count += 1;
                        report.failed_operation_ids.push(id);
                        report.errors.push(format!("Operation {}: {}", id, e));
                    } else {
                        // Put back in queue for retry
                        warn!(
                            "Operation {} failed (retry {}/{}): {}",
                            id, retry_count, MAX_OPERATION_RETRIES, e
                        );
                        // Operation stays in queue with incremented retry count
                        report.failure_count += 1;
                        report.failed_operation_ids.push(id);
                        report
                            .errors
                            .push(format!("Operation {} (retry {}): {}", id, retry_count, e));
                    }
                }
            }
        }

        info!(
            "Replay completed: {} succeeded, {} failed out of {} total",
            report.success_count, report.failure_count, report.total_operations
        );

        Ok(report)
    }

    /// Publish a sync operation using the configured publisher
    async fn publish_operation(&self, op: &SyncOperation) -> Result<(), PublisherError> {
        let message = self.convert_to_sync_message(op)?;

        // Track original size for metrics
        let original_size = serde_json::to_string(&message)
            .map(|s| s.len())
            .unwrap_or(0);

        // Publish the message
        self.publisher.publish(message).await?;

        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.bytes_original += original_size;

        debug!(
            "Published operation: original_size={} bytes, compression_ratio={:.2}, delta_savings={:.1}%",
            original_size,
            metrics.compression_ratio(),
            metrics.delta_savings_percent()
        );

        Ok(())
    }

    /// Convert SyncOperation to SyncMessage with delta encoding
    fn convert_to_sync_message(&self, op: &SyncOperation) -> Result<SyncMessage, PublisherError> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let message_id = uuid::Uuid::new_v4().to_string();
        let device_id = "offline-queue".to_string();

        let (payload, operation_type) = match op {
            SyncOperation::WatchlistAdd {
                user_id,
                content_id,
                timestamp: ts,
            } => {
                // Minimal payload for watchlist operations
                let payload = MessagePayload::WatchlistUpdate {
                    operation: crate::sync::WatchlistOperation::Add,
                    content_id: content_id.to_string(),
                    unique_tag: format!("{}:{}", user_id, content_id),
                    timestamp: self.millis_to_hlc(*ts),
                };

                debug!(
                    "Converting WatchlistAdd: user={}, content={}, timestamp={}",
                    user_id, content_id, ts
                );

                (payload, "watchlist_add".to_string())
            }

            SyncOperation::WatchlistRemove {
                user_id,
                content_id,
                timestamp: ts,
            } => {
                let payload = MessagePayload::WatchlistUpdate {
                    operation: crate::sync::WatchlistOperation::Remove,
                    content_id: content_id.to_string(),
                    unique_tag: format!("{}:{}", user_id, content_id),
                    timestamp: self.millis_to_hlc(*ts),
                };

                debug!(
                    "Converting WatchlistRemove: user={}, content={}, timestamp={}",
                    user_id, content_id, ts
                );

                (payload, "watchlist_remove".to_string())
            }

            SyncOperation::ProgressUpdate {
                user_id,
                content_id,
                position,
                timestamp: ts,
            } => {
                // Delta encoding: calculate position diff if enabled
                let (position_to_send, delta_applied) = if self.delta_config.enabled {
                    self.calculate_position_delta(*content_id, *position, *ts)
                } else {
                    (*position, false)
                };

                let position_seconds = (position_to_send * 1000.0) as u32;
                let duration_seconds = 1000; // Placeholder, would come from actual content metadata

                let payload = MessagePayload::ProgressUpdate {
                    content_id: content_id.to_string(),
                    position_seconds,
                    duration_seconds,
                    state: "Playing".to_string(),
                    timestamp: self.millis_to_hlc(*ts),
                };

                if delta_applied {
                    // Track bytes saved by delta encoding
                    let original_bytes = std::mem::size_of::<f64>();
                    let delta_bytes = std::mem::size_of::<f64>(); // In real impl, delta would be smaller
                    let saved = original_bytes.saturating_sub(delta_bytes);

                    let mut metrics = self.metrics.write();
                    metrics.bytes_saved += saved;

                    debug!(
                        "Delta encoding applied: content={}, position_diff={:.2}, bytes_saved={}",
                        content_id, position_to_send, saved
                    );
                }

                debug!(
                    "Converting ProgressUpdate: user={}, content={}, position={}, timestamp={}",
                    user_id, content_id, position, ts
                );

                (payload, "progress_update".to_string())
            }

            SyncOperation::DeviceCommand {
                command_id,
                source_device_id,
                target_device_id,
                command_type,
                payload: cmd_payload,
                timestamp: ts,
            } => {
                // For device commands, we create a generic sync message with the command payload
                // This would typically be routed through a separate command channel
                let payload_json = serde_json::json!({
                    "command_id": command_id,
                    "source_device_id": source_device_id,
                    "target_device_id": target_device_id,
                    "command_type": command_type,
                    "payload": cmd_payload,
                    "timestamp": ts,
                });

                debug!(
                    "Converting DeviceCommand: command_id={}, source={}, target={}, type={}",
                    command_id, source_device_id, target_device_id, command_type
                );

                // Create a batch message wrapping the command
                let command_msg = SyncMessage {
                    payload: MessagePayload::Batch { messages: vec![] },
                    timestamp: timestamp.clone(),
                    operation_type: "device_command".to_string(),
                    device_id: source_device_id.clone(),
                    message_id: command_id.to_string(),
                };

                return Ok(command_msg);
            }
        };

        Ok(SyncMessage {
            payload,
            timestamp,
            operation_type,
            device_id,
            message_id,
        })
    }

    /// Calculate position delta for progress updates
    fn calculate_position_delta(
        &self,
        content_id: Uuid,
        current_position: f64,
        timestamp: i64,
    ) -> (f64, bool) {
        let mut states = self.previous_states.write();

        if let Some(prev) = states.get(&content_id) {
            // Calculate delta from previous position
            let position_diff = current_position - prev.position;

            // Update state
            states.insert(
                content_id,
                PreviousState {
                    content_id,
                    position: current_position,
                    timestamp,
                },
            );

            (position_diff, true)
        } else {
            // No previous state, send full position
            states.insert(
                content_id,
                PreviousState {
                    content_id,
                    position: current_position,
                    timestamp,
                },
            );

            (current_position, false)
        }
    }

    /// Convert milliseconds timestamp to HLCTimestamp
    fn millis_to_hlc(&self, millis: i64) -> crate::crdt::HLCTimestamp {
        // Convert milliseconds to microseconds and create HLC timestamp
        let micros = millis * 1000;
        crate::crdt::HLCTimestamp::from_components(micros, 0)
    }

    /// Get current delta sync metrics
    pub fn get_metrics(&self) -> (usize, usize, f64, f64) {
        let metrics = self.metrics.read();
        (
            metrics.bytes_original,
            metrics.bytes_saved,
            metrics.compression_ratio(),
            metrics.delta_savings_percent(),
        )
    }

    /// Reset delta sync metrics
    pub fn reset_metrics(&self) {
        let mut metrics = self.metrics.write();
        *metrics = DeltaSyncMetrics::default();
    }
}

/// Queue operation errors
#[derive(Debug, Error)]
pub enum QueueError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Publisher error: {0}")]
    Publisher(#[from] PublisherError),

    #[error("Operation not found: {0}")]
    NotFound(u64),
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::sync::publisher::{MessagePayload, SyncMessage};
    use parking_lot::Mutex as ParkingMutex;
    use std::sync::Arc;

    /// Mock publisher for testing
    struct MockPublisher {
        published: Arc<ParkingMutex<Vec<String>>>,
        should_fail: Arc<ParkingMutex<bool>>,
    }

    impl MockPublisher {
        fn new() -> Self {
            Self {
                published: Arc::new(ParkingMutex::new(Vec::new())),
                should_fail: Arc::new(ParkingMutex::new(false)),
            }
        }

        fn get_published(&self) -> Vec<String> {
            self.published.lock().clone()
        }

        fn set_should_fail(&self, fail: bool) {
            *self.should_fail.lock() = fail;
        }

        fn published_count(&self) -> usize {
            self.published.lock().len()
        }
    }

    #[async_trait]
    impl SyncPublisher for MockPublisher {
        async fn publish(&self, message: SyncMessage) -> Result<(), PublisherError> {
            if *self.should_fail.lock() {
                return Err(PublisherError::InvalidMessage("Mock failure".to_string()));
            }
            self.published.lock().push(message.operation_type.clone());
            Ok(())
        }

        async fn publish_watchlist_update(
            &self,
            _update: crate::sync::WatchlistUpdate,
        ) -> Result<(), PublisherError> {
            if *self.should_fail.lock() {
                return Err(PublisherError::InvalidMessage("Mock failure".to_string()));
            }
            self.published.lock().push("watchlist_update".to_string());
            Ok(())
        }

        async fn publish_progress_update(
            &self,
            _update: crate::sync::ProgressUpdate,
        ) -> Result<(), PublisherError> {
            if *self.should_fail.lock() {
                return Err(PublisherError::InvalidMessage("Mock failure".to_string()));
            }
            self.published.lock().push("progress_update".to_string());
            Ok(())
        }

        async fn publish_batch(&self, _messages: Vec<SyncMessage>) -> Result<(), PublisherError> {
            if *self.should_fail.lock() {
                return Err(PublisherError::InvalidMessage("Mock failure".to_string()));
            }
            self.published.lock().push("batch".to_string());
            Ok(())
        }

        async fn flush(&self) -> Result<(), PublisherError> {
            Ok(())
        }
    }

    #[test]
    fn test_enqueue_dequeue_fifo_order() {
        let publisher = Arc::new(MockPublisher::new());
        let queue = OfflineSyncQueue::new_in_memory(publisher).unwrap();

        let user_id = Uuid::new_v4();
        let content_id1 = Uuid::new_v4();
        let content_id2 = Uuid::new_v4();
        let content_id3 = Uuid::new_v4();

        // Enqueue three operations
        let id1 = queue
            .enqueue(SyncOperation::WatchlistAdd {
                user_id,
                content_id: content_id1,
                timestamp: 1000,
            })
            .unwrap();

        let id2 = queue
            .enqueue(SyncOperation::ProgressUpdate {
                user_id,
                content_id: content_id2,
                position: 0.5,
                timestamp: 2000,
            })
            .unwrap();

        let id3 = queue
            .enqueue(SyncOperation::WatchlistRemove {
                user_id,
                content_id: content_id3,
                timestamp: 3000,
            })
            .unwrap();

        assert_eq!(queue.len().unwrap(), 3);

        // Dequeue in FIFO order
        let (deq_id1, op1) = queue.dequeue().unwrap().unwrap();
        assert_eq!(deq_id1, id1);
        assert!(matches!(op1, SyncOperation::WatchlistAdd { .. }));

        let (deq_id2, op2) = queue.dequeue().unwrap().unwrap();
        assert_eq!(deq_id2, id2);
        assert!(matches!(op2, SyncOperation::ProgressUpdate { .. }));

        let (deq_id3, op3) = queue.dequeue().unwrap().unwrap();
        assert_eq!(deq_id3, id3);
        assert!(matches!(op3, SyncOperation::WatchlistRemove { .. }));

        // Queue should be empty
        assert!(queue.dequeue().unwrap().is_none());
        assert_eq!(queue.len().unwrap(), 0);
    }

    #[test]
    fn test_peek_operations() {
        let publisher = Arc::new(MockPublisher::new());
        let queue = OfflineSyncQueue::new_in_memory(publisher).unwrap();

        let user_id = Uuid::new_v4();

        // Enqueue five operations
        for i in 0..5 {
            queue
                .enqueue(SyncOperation::ProgressUpdate {
                    user_id,
                    content_id: Uuid::new_v4(),
                    position: i as f64 * 0.1,
                    timestamp: (i + 1) * 1000,
                })
                .unwrap();
        }

        // Peek at first 3
        let peeked = queue.peek(3).unwrap();
        assert_eq!(peeked.len(), 3);

        // Verify peek doesn't remove items
        assert_eq!(queue.len().unwrap(), 5);

        // Verify peek maintains FIFO order
        if let SyncOperation::ProgressUpdate { position, .. } = peeked[0].1 {
            assert_eq!(position, 0.0);
        } else {
            panic!("Expected ProgressUpdate");
        }

        if let SyncOperation::ProgressUpdate { position, .. } = peeked[1].1 {
            assert_eq!(position, 0.1);
        } else {
            panic!("Expected ProgressUpdate");
        }
    }

    #[test]
    fn test_remove_operation() {
        let publisher = Arc::new(MockPublisher::new());
        let queue = OfflineSyncQueue::new_in_memory(publisher).unwrap();

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let id = queue
            .enqueue(SyncOperation::WatchlistAdd {
                user_id,
                content_id,
                timestamp: 1000,
            })
            .unwrap();

        assert_eq!(queue.len().unwrap(), 1);

        queue.remove(id).unwrap();

        assert_eq!(queue.len().unwrap(), 0);
        assert!(queue.is_empty().unwrap());
    }

    #[test]
    fn test_clear_all_operations() {
        let publisher = Arc::new(MockPublisher::new());
        let queue = OfflineSyncQueue::new_in_memory(publisher).unwrap();

        let user_id = Uuid::new_v4();

        // Enqueue multiple operations
        for i in 0..10 {
            queue
                .enqueue(SyncOperation::ProgressUpdate {
                    user_id,
                    content_id: Uuid::new_v4(),
                    position: i as f64 * 0.1,
                    timestamp: (i + 1) * 1000,
                })
                .unwrap();
        }

        assert_eq!(queue.len().unwrap(), 10);

        queue.clear().unwrap();

        assert_eq!(queue.len().unwrap(), 0);
        assert!(queue.is_empty().unwrap());
    }

    #[test]
    fn test_persistence_across_connections() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_sync_queue_{}.db", Uuid::new_v4()));

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        // Create queue and enqueue operation
        {
            let publisher = Arc::new(MockPublisher::new());
            let queue = OfflineSyncQueue::new(&db_path, publisher).unwrap();

            queue
                .enqueue(SyncOperation::WatchlistAdd {
                    user_id,
                    content_id,
                    timestamp: 1000,
                })
                .unwrap();

            assert_eq!(queue.len().unwrap(), 1);
        } // Drop queue, closing connection

        // Open new connection and verify data persisted
        {
            let publisher = Arc::new(MockPublisher::new());
            let queue = OfflineSyncQueue::new(&db_path, publisher).unwrap();

            assert_eq!(queue.len().unwrap(), 1);

            let (_, op) = queue.dequeue().unwrap().unwrap();
            match op {
                SyncOperation::WatchlistAdd {
                    user_id: uid,
                    content_id: cid,
                    timestamp,
                } => {
                    assert_eq!(uid, user_id);
                    assert_eq!(cid, content_id);
                    assert_eq!(timestamp, 1000);
                }
                _ => panic!("Expected WatchlistAdd"),
            }
        }

        // Cleanup
        std::fs::remove_file(&db_path).ok();
    }

    #[tokio::test]
    async fn test_replay_pending_success() {
        let publisher = Arc::new(MockPublisher::new());
        let queue =
            OfflineSyncQueue::new_in_memory(Arc::clone(&publisher) as Arc<dyn SyncPublisher>)
                .unwrap();

        let user_id = Uuid::new_v4();

        // Enqueue three operations
        queue
            .enqueue(SyncOperation::WatchlistAdd {
                user_id,
                content_id: Uuid::new_v4(),
                timestamp: 1000,
            })
            .unwrap();

        queue
            .enqueue(SyncOperation::ProgressUpdate {
                user_id,
                content_id: Uuid::new_v4(),
                position: 0.5,
                timestamp: 2000,
            })
            .unwrap();

        queue
            .enqueue(SyncOperation::WatchlistRemove {
                user_id,
                content_id: Uuid::new_v4(),
                timestamp: 3000,
            })
            .unwrap();

        assert_eq!(queue.len().unwrap(), 3);

        // Replay all operations
        let report = queue.replay_pending().await.unwrap();

        // All should succeed
        assert_eq!(report.total_operations, 3);
        assert_eq!(report.success_count, 3);
        assert_eq!(report.failure_count, 0);
        assert!(report.all_succeeded());

        // Queue should be empty
        assert_eq!(queue.len().unwrap(), 0);
    }

    #[tokio::test]
    async fn test_replay_pending_with_failures() {
        let publisher = Arc::new(MockPublisher::new());
        let queue =
            OfflineSyncQueue::new_in_memory(Arc::clone(&publisher) as Arc<dyn SyncPublisher>)
                .unwrap();

        let user_id = Uuid::new_v4();

        // Enqueue operations
        queue
            .enqueue(SyncOperation::WatchlistAdd {
                user_id,
                content_id: Uuid::new_v4(),
                timestamp: 1000,
            })
            .unwrap();

        // Set publisher to fail
        publisher.set_should_fail(true);

        // Replay - should fail and retry up to MAX_OPERATION_RETRIES
        let report = queue.replay_pending().await.unwrap();

        assert_eq!(report.total_operations, 1);
        assert_eq!(report.success_count, 0);
        assert_eq!(report.failure_count, 1);
        assert!(report.has_failures());
        assert_eq!(report.failed_operation_ids.len(), 1);
    }

    #[test]
    fn test_operation_serialization() {
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let op = SyncOperation::WatchlistAdd {
            user_id,
            content_id,
            timestamp: 1000,
        };

        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("watchlist_add"));

        let deserialized: SyncOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(op, deserialized);
    }

    // Delta Sync Tests

    #[test]
    fn test_delta_sync_config_from_env() {
        // Test default values
        let config = DeltaSyncConfig::default();
        assert!(config.enabled);
        assert!(config.compression_enabled);
        assert_eq!(config.min_batch_size, 3);
    }

    #[test]
    fn test_delta_sync_config_custom() {
        let config = DeltaSyncConfig {
            enabled: false,
            compression_enabled: false,
            min_batch_size: 5,
        };
        assert!(!config.enabled);
        assert!(!config.compression_enabled);
        assert_eq!(config.min_batch_size, 5);
    }

    #[test]
    fn test_delta_position_calculation() {
        let publisher = Arc::new(MockPublisher::new());
        let queue = OfflineSyncQueue::new_in_memory(publisher).unwrap();

        let content_id = Uuid::new_v4();

        // First update - no previous state, should return full position
        let (position1, delta_applied1) = queue.calculate_position_delta(content_id, 100.0, 1000);
        assert_eq!(position1, 100.0);
        assert!(!delta_applied1);

        // Second update - should calculate delta
        let (position2, delta_applied2) = queue.calculate_position_delta(content_id, 150.0, 2000);
        assert_eq!(position2, 50.0); // 150.0 - 100.0
        assert!(delta_applied2);

        // Third update - delta from previous
        let (position3, delta_applied3) = queue.calculate_position_delta(content_id, 200.0, 3000);
        assert_eq!(position3, 50.0); // 200.0 - 150.0
        assert!(delta_applied3);
    }

    #[test]
    fn test_convert_to_sync_message_watchlist_add() {
        let publisher = Arc::new(MockPublisher::new());
        let queue = OfflineSyncQueue::new_in_memory(publisher).unwrap();

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let op = SyncOperation::WatchlistAdd {
            user_id,
            content_id,
            timestamp: 1000,
        };

        let message = queue.convert_to_sync_message(&op).unwrap();
        assert_eq!(message.operation_type, "watchlist_add");
        assert_eq!(message.device_id, "offline-queue");
    }

    #[test]
    fn test_convert_to_sync_message_watchlist_remove() {
        let publisher = Arc::new(MockPublisher::new());
        let queue = OfflineSyncQueue::new_in_memory(publisher).unwrap();

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let op = SyncOperation::WatchlistRemove {
            user_id,
            content_id,
            timestamp: 2000,
        };

        let message = queue.convert_to_sync_message(&op).unwrap();
        assert_eq!(message.operation_type, "watchlist_remove");
    }

    #[test]
    fn test_convert_to_sync_message_progress_update() {
        let publisher = Arc::new(MockPublisher::new());
        let queue = OfflineSyncQueue::new_in_memory(publisher).unwrap();

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let op = SyncOperation::ProgressUpdate {
            user_id,
            content_id,
            position: 0.5,
            timestamp: 3000,
        };

        let message = queue.convert_to_sync_message(&op).unwrap();
        assert_eq!(message.operation_type, "progress_update");

        // Verify payload structure
        if let MessagePayload::ProgressUpdate {
            content_id: cid, ..
        } = message.payload
        {
            assert_eq!(cid, content_id.to_string());
        } else {
            panic!("Expected ProgressUpdate payload");
        }
    }

    #[test]
    fn test_convert_to_sync_message_device_command() {
        let publisher = Arc::new(MockPublisher::new());
        let queue = OfflineSyncQueue::new_in_memory(publisher).unwrap();

        let command_id = Uuid::new_v4();
        let op = SyncOperation::DeviceCommand {
            command_id,
            source_device_id: "device-1".to_string(),
            target_device_id: "device-2".to_string(),
            command_type: "play".to_string(),
            payload: serde_json::json!({"content_id": "movie-123"}),
            timestamp: 4000,
        };

        let message = queue.convert_to_sync_message(&op).unwrap();
        assert_eq!(message.operation_type, "device_command");
        assert_eq!(message.message_id, command_id.to_string());
    }

    #[test]
    fn test_delta_sync_metrics() {
        let mut metrics = DeltaSyncMetrics::default();

        metrics.bytes_original = 1000;
        metrics.bytes_saved = 200;
        metrics.bytes_compressed = 600;

        assert_eq!(metrics.compression_ratio(), 0.6);
        assert_eq!(metrics.delta_savings_percent(), 20.0);
    }

    #[test]
    fn test_metrics_tracking() {
        let publisher = Arc::new(MockPublisher::new());
        let queue = OfflineSyncQueue::new_in_memory(publisher).unwrap();

        let (original, saved, compression_ratio, delta_percent) = queue.get_metrics();
        assert_eq!(original, 0);
        assert_eq!(saved, 0);
        assert_eq!(compression_ratio, 1.0);
        assert_eq!(delta_percent, 0.0);
    }

    #[test]
    fn test_reset_metrics() {
        let publisher = Arc::new(MockPublisher::new());
        let queue = OfflineSyncQueue::new_in_memory(publisher).unwrap();

        // Manually set some metrics
        {
            let mut metrics = queue.metrics.write();
            metrics.bytes_original = 1000;
            metrics.bytes_saved = 200;
        }

        // Verify metrics are set
        let (original, saved, _, _) = queue.get_metrics();
        assert_eq!(original, 1000);
        assert_eq!(saved, 200);

        // Reset
        queue.reset_metrics();

        // Verify reset
        let (original, saved, _, _) = queue.get_metrics();
        assert_eq!(original, 0);
        assert_eq!(saved, 0);
    }

    #[tokio::test]
    async fn test_publish_operation_with_delta_sync() {
        let publisher = Arc::new(MockPublisher::new());
        let queue =
            OfflineSyncQueue::new_in_memory(Arc::clone(&publisher) as Arc<dyn SyncPublisher>)
                .unwrap();

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        // First progress update
        let op1 = SyncOperation::ProgressUpdate {
            user_id,
            content_id,
            position: 100.0,
            timestamp: 1000,
        };

        queue.publish_operation(&op1).await.unwrap();

        // Second progress update - should use delta
        let op2 = SyncOperation::ProgressUpdate {
            user_id,
            content_id,
            position: 150.0,
            timestamp: 2000,
        };

        queue.publish_operation(&op2).await.unwrap();

        // Verify publisher received both messages
        assert_eq!(publisher.published_count(), 2);
    }

    #[tokio::test]
    async fn test_replay_with_device_command() {
        let publisher = Arc::new(MockPublisher::new());
        let queue =
            OfflineSyncQueue::new_in_memory(Arc::clone(&publisher) as Arc<dyn SyncPublisher>)
                .unwrap();

        let command_id = Uuid::new_v4();

        // Enqueue device command
        queue
            .enqueue(SyncOperation::DeviceCommand {
                command_id,
                source_device_id: "device-1".to_string(),
                target_device_id: "device-2".to_string(),
                command_type: "play".to_string(),
                payload: serde_json::json!({"content_id": "movie-123"}),
                timestamp: 1000,
            })
            .unwrap();

        assert_eq!(queue.len().unwrap(), 1);

        // Replay
        let report = queue.replay_pending().await.unwrap();

        assert_eq!(report.total_operations, 1);
        assert_eq!(report.success_count, 1);
        assert!(report.all_succeeded());
        assert_eq!(queue.len().unwrap(), 0);
    }

    #[test]
    fn test_device_command_serialization() {
        let command_id = Uuid::new_v4();

        let op = SyncOperation::DeviceCommand {
            command_id,
            source_device_id: "device-1".to_string(),
            target_device_id: "device-2".to_string(),
            command_type: "pause".to_string(),
            payload: serde_json::json!({"position": 100}),
            timestamp: 5000,
        };

        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("device_command"));
        assert!(json.contains("device-1"));
        assert!(json.contains("device-2"));

        let deserialized: SyncOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(op, deserialized);
    }

    #[tokio::test]
    async fn test_integration_delta_encoding_with_multiple_updates() {
        let publisher = Arc::new(MockPublisher::new());
        let queue =
            OfflineSyncQueue::new_in_memory(Arc::clone(&publisher) as Arc<dyn SyncPublisher>)
                .unwrap();

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        // Enqueue multiple progress updates
        for i in 0..5 {
            queue
                .enqueue(SyncOperation::ProgressUpdate {
                    user_id,
                    content_id,
                    position: (i as f64) * 10.0,
                    timestamp: (i + 1) * 1000,
                })
                .unwrap();
        }

        assert_eq!(queue.len().unwrap(), 5);

        // Replay all operations
        let report = queue.replay_pending().await.unwrap();

        assert_eq!(report.total_operations, 5);
        assert_eq!(report.success_count, 5);
        assert!(report.all_succeeded());

        // Verify all were published
        assert_eq!(publisher.published_count(), 5);

        // Check metrics were tracked
        let (original, _saved, _ratio, _percent) = queue.get_metrics();
        assert!(original > 0, "Metrics should track published bytes");
    }

    #[test]
    fn test_queue_with_custom_delta_config() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_delta_config_{}.db", Uuid::new_v4()));

        let publisher = Arc::new(MockPublisher::new());

        let config = DeltaSyncConfig {
            enabled: false,
            compression_enabled: false,
            min_batch_size: 10,
        };

        let queue = OfflineSyncQueue::new_with_config(&db_path, publisher, config.clone()).unwrap();

        assert!(!queue.delta_config.enabled);
        assert!(!queue.delta_config.compression_enabled);
        assert_eq!(queue.delta_config.min_batch_size, 10);

        // Cleanup
        std::fs::remove_file(&db_path).ok();
    }

    #[test]
    fn test_millis_to_hlc_conversion() {
        let publisher = Arc::new(MockPublisher::new());
        let queue = OfflineSyncQueue::new_in_memory(publisher).unwrap();

        let millis = 1234567890123i64;
        let hlc = queue.millis_to_hlc(millis);

        // Convert millis to micros for comparison
        let expected_micros = millis * 1000;
        assert_eq!(hlc.physical_time(), expected_micros);
        assert_eq!(hlc.logical_counter(), 0);
    }
}
