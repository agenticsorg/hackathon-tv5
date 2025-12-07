/// Unit test for offline queue persistence with rusqlite
///
/// Verifies that:
/// 1. Queue operations persist across database connections
/// 2. FIFO ordering is maintained
/// 3. Rusqlite dependency is correctly integrated
use media_gateway_sync::{OfflineSyncQueue, SyncOperation};
use std::sync::Arc;
use uuid::Uuid;

/// Mock publisher for testing (minimal implementation)
struct TestPublisher;

#[async_trait::async_trait]
impl media_gateway_sync::sync::publisher::SyncPublisher for TestPublisher {
    async fn publish(
        &self,
        _message: media_gateway_sync::sync::publisher::SyncMessage,
    ) -> Result<(), media_gateway_sync::sync::publisher::PublisherError> {
        Ok(())
    }

    async fn publish_watchlist_update(
        &self,
        _update: media_gateway_sync::WatchlistUpdate,
    ) -> Result<(), media_gateway_sync::sync::publisher::PublisherError> {
        Ok(())
    }

    async fn publish_progress_update(
        &self,
        _update: media_gateway_sync::ProgressUpdate,
    ) -> Result<(), media_gateway_sync::sync::publisher::PublisherError> {
        Ok(())
    }

    async fn publish_batch(
        &self,
        _messages: Vec<media_gateway_sync::sync::publisher::SyncMessage>,
    ) -> Result<(), media_gateway_sync::sync::publisher::PublisherError> {
        Ok(())
    }

    async fn flush(&self) -> Result<(), media_gateway_sync::sync::publisher::PublisherError> {
        Ok(())
    }
}

#[test]
fn test_offline_queue_persistence() {
    let temp_dir = std::env::temp_dir();
    let db_path = temp_dir.join(format!("test_offline_queue_{}.db", Uuid::new_v4()));

    let user_id = Uuid::new_v4();
    let content_id1 = Uuid::new_v4();
    let content_id2 = Uuid::new_v4();

    // Create queue and enqueue operations
    {
        let publisher = Arc::new(TestPublisher);
        let queue = OfflineSyncQueue::new(&db_path, publisher).unwrap();

        // Enqueue first operation
        queue
            .enqueue(SyncOperation::WatchlistAdd {
                user_id,
                content_id: content_id1,
                timestamp: 1000,
            })
            .unwrap();

        // Enqueue second operation
        queue
            .enqueue(SyncOperation::ProgressUpdate {
                user_id,
                content_id: content_id2,
                position: 0.5,
                timestamp: 2000,
            })
            .unwrap();

        assert_eq!(queue.len().unwrap(), 2);
    } // Drop queue, closing connection

    // Open new connection and verify data persisted
    {
        let publisher = Arc::new(TestPublisher);
        let queue = OfflineSyncQueue::new(&db_path, publisher).unwrap();

        // Verify count persisted
        assert_eq!(queue.len().unwrap(), 2);

        // Dequeue and verify FIFO order
        let (_, op1) = queue.dequeue().unwrap().unwrap();
        match op1 {
            SyncOperation::WatchlistAdd {
                user_id: uid,
                content_id: cid,
                timestamp,
            } => {
                assert_eq!(uid, user_id);
                assert_eq!(cid, content_id1);
                assert_eq!(timestamp, 1000);
            }
            _ => panic!("Expected WatchlistAdd"),
        }

        let (_, op2) = queue.dequeue().unwrap().unwrap();
        match op2 {
            SyncOperation::ProgressUpdate {
                user_id: uid,
                content_id: cid,
                position,
                timestamp,
            } => {
                assert_eq!(uid, user_id);
                assert_eq!(cid, content_id2);
                assert_eq!(position, 0.5);
                assert_eq!(timestamp, 2000);
            }
            _ => panic!("Expected ProgressUpdate"),
        }

        // Verify queue is now empty
        assert!(queue.is_empty().unwrap());
    }

    // Cleanup
    std::fs::remove_file(&db_path).ok();
}

#[test]
fn test_rusqlite_dependency() {
    // Verify rusqlite is available and working
    let publisher = Arc::new(TestPublisher);
    let queue = OfflineSyncQueue::new_in_memory(publisher).unwrap();

    assert!(queue.is_empty().unwrap());
    assert_eq!(queue.len().unwrap(), 0);
}
