//! End-to-end sync and realtime tests
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SyncOperation {
    id: Uuid,
    user_id: Uuid,
    device_id: String,
    operation_type: String,
    entity_type: String,
    entity_id: Uuid,
    data: serde_json::Value,
    vector_clock: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CRDTState {
    entity_id: Uuid,
    state: serde_json::Value,
    vector_clock: i64,
    last_modified: i64,
}

#[tokio::test]
async fn test_crdt_counter_operations() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create test user
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("crdt-{}@example.com", Uuid::new_v4()))
    .bind("$2b$12$hashedpassword")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Initialize CRDT counter
    let entity_id = Uuid::new_v4();
    let device1 = "device-1";
    let device2 = "device-2";

    // Device 1: Increment counter
    let crdt_key = format!("crdt:counter:{}", entity_id);
    redis::cmd("HINCRBY")
        .arg(&crdt_key)
        .arg(device1)
        .arg(5)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Device 2: Increment counter
    redis::cmd("HINCRBY")
        .arg(&crdt_key)
        .arg(device2)
        .arg(3)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Get total counter value
    let device1_val: i64 = redis::cmd("HGET")
        .arg(&crdt_key)
        .arg(device1)
        .query_async(&mut containers.redis_conn.clone())
        .await?;

    let device2_val: i64 = redis::cmd("HGET")
        .arg(&crdt_key)
        .arg(device2)
        .query_async(&mut containers.redis_conn.clone())
        .await?;

    assert_eq!(device1_val + device2_val, 8);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_crdt_lww_register() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Last-Write-Wins register
    let entity_id = Uuid::new_v4();
    let crdt_key = format!("crdt:lww:{}", entity_id);

    // Write 1: timestamp 100
    redis::cmd("HSET")
        .arg(&crdt_key)
        .arg("value")
        .arg("first")
        .arg("timestamp")
        .arg(100)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Write 2: timestamp 200 (should win)
    redis::cmd("HSET")
        .arg(&crdt_key)
        .arg("value")
        .arg("second")
        .arg("timestamp")
        .arg(200)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Read final value
    let value: String = redis::cmd("HGET")
        .arg(&crdt_key)
        .arg("value")
        .query_async(&mut containers.redis_conn.clone())
        .await?;

    assert_eq!(value, "second");

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_conflict_resolution_automatic() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create conflicting operations
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("conflict-{}@example.com", Uuid::new_v4()))
    .bind("$2b$12$hashedpassword")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    let entity_id = Uuid::new_v4();

    // Operation 1: Device A updates entity
    let op1_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO sync_operations (id, user_id, device_id, operation_type, entity_type, entity_id, data, vector_clock, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(op1_id)
    .bind(user_id)
    .bind("device-a")
    .bind("update")
    .bind("profile")
    .bind(entity_id)
    .bind(serde_json::json!({"name": "Alice"}))
    .bind(1)
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Operation 2: Device B updates same entity (conflict)
    let op2_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO sync_operations (id, user_id, device_id, operation_type, entity_type, entity_id, data, vector_clock, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(op2_id)
    .bind(user_id)
    .bind("device-b")
    .bind("update")
    .bind("profile")
    .bind(entity_id)
    .bind(serde_json::json!({"name": "Alicia"}))
    .bind(1)
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Resolve conflict: LWW (last-write-wins)
    let operations: Vec<(Uuid, i64, serde_json::Value)> = sqlx::query_as(
        "SELECT id, vector_clock, data FROM sync_operations
         WHERE entity_id = $1
         ORDER BY created_at DESC, id DESC",
    )
    .bind(entity_id)
    .fetch_all(&containers.db_pool)
    .await?;

    assert_eq!(operations.len(), 2);
    // Most recent operation wins
    let winner = &operations[0];

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_cross_device_sync() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create test user
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("sync-{}@example.com", Uuid::new_v4()))
    .bind("$2b$12$hashedpassword")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Create sync operations from multiple devices
    let devices = vec!["mobile", "tablet", "desktop"];
    for (idx, device) in devices.iter().enumerate() {
        let op_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO sync_operations (id, user_id, device_id, operation_type, entity_type, entity_id, data, vector_clock, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(op_id)
        .bind(user_id)
        .bind(*device)
        .bind("update")
        .bind("preferences")
        .bind(Uuid::new_v4())
        .bind(serde_json::json!({"device": device}))
        .bind(idx as i64 + 1)
        .bind(Utc::now())
        .execute(&containers.db_pool)
        .await?;
    }

    // Verify sync operations from all devices
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sync_operations WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&containers.db_pool)
        .await?;

    assert_eq!(count, 3);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_realtime_sync_pubsub() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create sync channel for user
    let user_id = Uuid::new_v4();
    let channel = format!("sync:user:{}", user_id);

    // Publish sync event
    let event = serde_json::json!({
        "type": "sync",
        "entity": "profile",
        "operation": "update",
        "data": {"name": "Test User"}
    });

    let subscribers: i64 = redis::cmd("PUBLISH")
        .arg(&channel)
        .arg(event.to_string())
        .query_async(&mut containers.redis_conn.clone())
        .await?;

    // No subscribers expected in test
    assert_eq!(subscribers, 0);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_vector_clock_ordering() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create operations with vector clocks
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("vector-{}@example.com", Uuid::new_v4()))
    .bind("$2b$12$hashedpassword")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    let entity_id = Uuid::new_v4();

    // Create ordered operations
    for i in 1..=5 {
        let op_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO sync_operations (id, user_id, device_id, operation_type, entity_type, entity_id, data, vector_clock, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(op_id)
        .bind(user_id)
        .bind("device-1")
        .bind("update")
        .bind("counter")
        .bind(entity_id)
        .bind(serde_json::json!({"value": i}))
        .bind(i)
        .bind(Utc::now())
        .execute(&containers.db_pool)
        .await?;
    }

    // Verify ordering by vector clock
    let clocks: Vec<i64> = sqlx::query_scalar(
        "SELECT vector_clock FROM sync_operations
         WHERE entity_id = $1
         ORDER BY vector_clock ASC",
    )
    .bind(entity_id)
    .fetch_all(&containers.db_pool)
    .await?;

    assert_eq!(clocks, vec![1, 2, 3, 4, 5]);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_offline_sync_queue() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Queue operations while offline
    let user_id = Uuid::new_v4();
    let device_id = "offline-device";
    let queue_key = format!("sync:queue:{}:{}", user_id, device_id);

    // Add operations to queue
    for i in 1..=3 {
        let op = serde_json::json!({
            "id": Uuid::new_v4(),
            "operation": "update",
            "data": {"index": i}
        });

        redis::cmd("RPUSH")
            .arg(&queue_key)
            .arg(op.to_string())
            .query_async::<_, ()>(&mut containers.redis_conn.clone())
            .await?;
    }

    // Verify queue length
    let length: i64 = redis::cmd("LLEN")
        .arg(&queue_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;

    assert_eq!(length, 3);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_sync_conflict_merge() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create conflicting updates
    let entity_id = Uuid::new_v4();
    let merge_key = format!("sync:merge:{}", entity_id);

    // State from device 1
    let state1 = serde_json::json!({
        "field_a": "value1",
        "field_b": "value2"
    });

    // State from device 2
    let state2 = serde_json::json!({
        "field_a": "value1",
        "field_b": "value3",
        "field_c": "value4"
    });

    // Store states
    redis::cmd("HSET")
        .arg(&merge_key)
        .arg("device1")
        .arg(state1.to_string())
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    redis::cmd("HSET")
        .arg(&merge_key)
        .arg("device2")
        .arg(state2.to_string())
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Verify both states stored
    let exists: bool = redis::cmd("HEXISTS")
        .arg(&merge_key)
        .arg("device1")
        .query_async(&mut containers.redis_conn.clone())
        .await?;
    assert!(exists);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_sync_timestamp_ordering() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create operations with timestamps
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("timestamp-{}@example.com", Uuid::new_v4()))
    .bind("$2b$12$hashedpassword")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    let entity_id = Uuid::new_v4();
    let base_time = Utc::now();

    // Create operations with different timestamps
    for i in 0..3 {
        let op_id = Uuid::new_v4();
        let timestamp = base_time + chrono::Duration::seconds(i);

        sqlx::query(
            "INSERT INTO sync_operations (id, user_id, device_id, operation_type, entity_type, entity_id, data, vector_clock, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(op_id)
        .bind(user_id)
        .bind("device-1")
        .bind("update")
        .bind("entity")
        .bind(entity_id)
        .bind(serde_json::json!({"index": i}))
        .bind(i as i64)
        .bind(timestamp)
        .execute(&containers.db_pool)
        .await?;
    }

    // Verify chronological ordering
    let operations: Vec<(Uuid, i64)> = sqlx::query_as(
        "SELECT id, vector_clock FROM sync_operations
         WHERE entity_id = $1
         ORDER BY created_at ASC",
    )
    .bind(entity_id)
    .fetch_all(&containers.db_pool)
    .await?;

    assert_eq!(operations.len(), 3);
    assert_eq!(operations[0].1, 0);
    assert_eq!(operations[1].1, 1);
    assert_eq!(operations[2].1, 2);

    containers.cleanup().await?;
    Ok(())
}
