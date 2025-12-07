//! Integration tests for LoRA storage module
//!
//! These tests verify:
//! - Serialization/deserialization correctness
//! - Database persistence and retrieval
//! - Versioning behavior
//! - Performance characteristics (<2ms load target)
//!
//! Run with:
//! ```bash
//! export DATABASE_URL="postgres://postgres:postgres@localhost:5432/media_gateway_test"
//! cargo test --package media-gateway-sona lora_storage_test --ignored
//! ```

#[cfg(test)]
mod tests {
    use crate::lora::{ComputeLoRAForward, UserLoRAAdapter};
    use crate::lora_storage::LoRAStorage;
    use anyhow::Result;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Instant;
    use uuid::Uuid;

    async fn setup_test_pool() -> Result<sqlx::PgPool> {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/media_gateway_test".to_string()
        });

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        // Ensure table exists (run migration manually or create table)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS lora_adapters (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL,
                adapter_name VARCHAR(100) NOT NULL DEFAULT 'default',
                version INTEGER NOT NULL DEFAULT 1,
                weights BYTEA NOT NULL,
                size_bytes BIGINT NOT NULL,
                training_iterations INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE (user_id, adapter_name, version)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_lora_adapters_user_name_version
            ON lora_adapters(user_id, adapter_name, version DESC)
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(pool)
    }

    async fn cleanup_user(pool: &sqlx::PgPool, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM lora_adapters WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_save_and_load_roundtrip() -> Result<()> {
        let pool = setup_test_pool().await?;
        let storage = LoRAStorage::new(pool.clone());

        let user_id = Uuid::new_v4();
        let mut adapter = UserLoRAAdapter::new(user_id);
        adapter.initialize_random();
        adapter.training_iterations = 42;

        // Save adapter
        let version = storage.save_adapter(&adapter, "test_adapter").await?;
        assert_eq!(version, 1, "First version should be 1");

        // Load adapter
        let loaded = storage.load_adapter(user_id, "test_adapter").await?;

        // Verify metadata
        assert_eq!(loaded.user_id, adapter.user_id);
        assert_eq!(loaded.rank, adapter.rank);
        assert_eq!(loaded.scaling_factor, adapter.scaling_factor);
        assert_eq!(loaded.training_iterations, adapter.training_iterations);

        // Verify weight dimensions
        assert_eq!(
            loaded.base_layer_weights.shape(),
            adapter.base_layer_weights.shape()
        );
        assert_eq!(
            loaded.user_layer_weights.shape(),
            adapter.user_layer_weights.shape()
        );

        // Verify weight values (within epsilon)
        for (a, b) in loaded
            .base_layer_weights
            .iter()
            .zip(adapter.base_layer_weights.iter())
        {
            assert!(
                (a - b).abs() < 0.001,
                "Base layer weights mismatch: {} vs {}",
                a,
                b
            );
        }

        for (a, b) in loaded
            .user_layer_weights
            .iter()
            .zip(adapter.user_layer_weights.iter())
        {
            assert!(
                (a - b).abs() < 0.001,
                "User layer weights mismatch: {} vs {}",
                a,
                b
            );
        }

        cleanup_user(&pool, user_id).await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_versioning_behavior() -> Result<()> {
        let pool = setup_test_pool().await?;
        let storage = LoRAStorage::new(pool.clone());

        let user_id = Uuid::new_v4();

        // Save version 1
        let mut adapter_v1 = UserLoRAAdapter::new(user_id);
        adapter_v1.training_iterations = 1;
        let v1 = storage.save_adapter(&adapter_v1, "versioned").await?;
        assert_eq!(v1, 1);

        // Save version 2
        let mut adapter_v2 = UserLoRAAdapter::new(user_id);
        adapter_v2.training_iterations = 2;
        let v2 = storage.save_adapter(&adapter_v2, "versioned").await?;
        assert_eq!(v2, 2);

        // Save version 3
        let mut adapter_v3 = UserLoRAAdapter::new(user_id);
        adapter_v3.training_iterations = 3;
        let v3 = storage.save_adapter(&adapter_v3, "versioned").await?;
        assert_eq!(v3, 3);

        // Load latest (should be v3)
        let latest = storage.load_adapter(user_id, "versioned").await?;
        assert_eq!(latest.training_iterations, 3);

        // Load specific versions
        let loaded_v1 = storage
            .load_adapter_version(user_id, "versioned", 1)
            .await?;
        assert_eq!(loaded_v1.training_iterations, 1);

        let loaded_v2 = storage
            .load_adapter_version(user_id, "versioned", 2)
            .await?;
        assert_eq!(loaded_v2.training_iterations, 2);

        cleanup_user(&pool, user_id).await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_multiple_adapter_names() -> Result<()> {
        let pool = setup_test_pool().await?;
        let storage = LoRAStorage::new(pool.clone());

        let user_id = Uuid::new_v4();

        // Save different adapters
        let mut production = UserLoRAAdapter::new(user_id);
        production.training_iterations = 100;
        storage.save_adapter(&production, "production").await?;

        let mut experimental = UserLoRAAdapter::new(user_id);
        experimental.training_iterations = 50;
        storage.save_adapter(&experimental, "experimental").await?;

        let mut staging = UserLoRAAdapter::new(user_id);
        staging.training_iterations = 75;
        storage.save_adapter(&staging, "staging").await?;

        // Load each adapter
        let prod = storage.load_adapter(user_id, "production").await?;
        assert_eq!(prod.training_iterations, 100);

        let exp = storage.load_adapter(user_id, "experimental").await?;
        assert_eq!(exp.training_iterations, 50);

        let stg = storage.load_adapter(user_id, "staging").await?;
        assert_eq!(stg.training_iterations, 75);

        // List all adapters
        let adapters = storage.list_adapters(user_id).await?;
        assert_eq!(adapters.len(), 3);

        cleanup_user(&pool, user_id).await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_adapters_ordering() -> Result<()> {
        let pool = setup_test_pool().await?;
        let storage = LoRAStorage::new(pool.clone());

        let user_id = Uuid::new_v4();
        let adapter = UserLoRAAdapter::new(user_id);

        // Save multiple versions with delays to ensure different timestamps
        storage.save_adapter(&adapter, "default").await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        storage.save_adapter(&adapter, "experimental").await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        storage.save_adapter(&adapter, "default").await?; // v2

        let adapters = storage.list_adapters(user_id).await?;

        // Should be ordered by updated_at DESC (newest first)
        assert!(adapters.len() >= 2);
        for i in 0..adapters.len() - 1 {
            assert!(
                adapters[i].updated_at >= adapters[i + 1].updated_at,
                "Adapters not sorted by updated_at DESC"
            );
        }

        cleanup_user(&pool, user_id).await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_operations() -> Result<()> {
        let pool = setup_test_pool().await?;
        let storage = LoRAStorage::new(pool.clone());

        let user_id = Uuid::new_v4();
        let adapter = UserLoRAAdapter::new(user_id);

        // Save multiple versions
        storage.save_adapter(&adapter, "delete_test").await?;
        storage.save_adapter(&adapter, "delete_test").await?;
        storage.save_adapter(&adapter, "delete_test").await?;

        let count_before = storage.count_adapters(user_id).await?;
        assert_eq!(count_before, 3);

        // Delete specific version
        let deleted = storage
            .delete_adapter_version(user_id, "delete_test", 2)
            .await?;
        assert!(deleted);

        let count_after = storage.count_adapters(user_id).await?;
        assert_eq!(count_after, 2);

        // Delete all versions
        let deleted_count = storage.delete_adapter(user_id, "delete_test").await?;
        assert_eq!(deleted_count, 2);

        let count_final = storage.count_adapters(user_id).await?;
        assert_eq!(count_final, 0);

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_load_nonexistent_adapter() -> Result<()> {
        let pool = setup_test_pool().await?;
        let storage = LoRAStorage::new(pool.clone());

        let user_id = Uuid::new_v4();

        // Attempt to load non-existent adapter
        let result = storage.load_adapter(user_id, "nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_retrieval_latency() -> Result<()> {
        let pool = setup_test_pool().await?;
        let storage = LoRAStorage::new(pool.clone());

        let user_id = Uuid::new_v4();
        let mut adapter = UserLoRAAdapter::new(user_id);
        adapter.initialize_random();

        // Save adapter
        storage.save_adapter(&adapter, "latency_test").await?;

        // Warm up cache
        storage.load_adapter(user_id, "latency_test").await?;

        // Measure latency over multiple iterations
        let iterations = 100;
        let mut total_duration = std::time::Duration::ZERO;

        for _ in 0..iterations {
            let start = Instant::now();
            let _loaded = storage.load_adapter(user_id, "latency_test").await?;
            let elapsed = start.elapsed();
            total_duration += elapsed;
        }

        let avg_duration = total_duration / iterations;
        println!("Average load latency: {:?}", avg_duration);

        // Log warning if >2ms (don't fail test as it depends on hardware)
        if avg_duration.as_millis() > 2 {
            println!(
                "WARNING: Average latency ({:?}) exceeds 2ms target",
                avg_duration
            );
        }

        cleanup_user(&pool, user_id).await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_functional_correctness_after_roundtrip() -> Result<()> {
        let pool = setup_test_pool().await?;
        let storage = LoRAStorage::new(pool.clone());

        let user_id = Uuid::new_v4();
        let mut adapter = UserLoRAAdapter::new(user_id);
        adapter.initialize_random();

        // Compute forward pass before saving
        let input = vec![0.5; 512];
        let output_before = ComputeLoRAForward::execute(&adapter, &input)?;

        // Save and load
        storage.save_adapter(&adapter, "functional_test").await?;
        let loaded = storage.load_adapter(user_id, "functional_test").await?;

        // Compute forward pass after loading
        let output_after = ComputeLoRAForward::execute(&loaded, &input)?;

        // Verify outputs match
        assert_eq!(output_before.len(), output_after.len());
        for (before, after) in output_before.iter().zip(output_after.iter()) {
            assert!(
                (before - after).abs() < 0.001,
                "Forward pass output mismatch after roundtrip"
            );
        }

        cleanup_user(&pool, user_id).await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_storage_stats() -> Result<()> {
        let pool = setup_test_pool().await?;
        let storage = LoRAStorage::new(pool.clone());

        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();

        let adapter1 = UserLoRAAdapter::new(user1);
        let adapter2 = UserLoRAAdapter::new(user2);

        storage.save_adapter(&adapter1, "default").await?;
        storage.save_adapter(&adapter2, "default").await?;

        let stats = storage.get_storage_stats().await?;
        assert!(stats.total_adapters >= 2);
        assert!(stats.unique_users >= 2);
        assert!(stats.total_bytes > 0);
        assert!(stats.avg_bytes > 0.0);

        println!("Storage stats: {:?}", stats);

        cleanup_user(&pool, user1).await?;
        cleanup_user(&pool, user2).await?;

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_concurrent_saves() -> Result<()> {
        let pool = setup_test_pool().await?;
        let storage = LoRAStorage::new(pool.clone());

        let user_id = Uuid::new_v4();

        // Spawn multiple concurrent saves
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let storage_clone = LoRAStorage::new(pool.clone());
                let user_id = user_id.clone();
                tokio::spawn(async move {
                    let mut adapter = UserLoRAAdapter::new(user_id);
                    adapter.training_iterations = i;
                    storage_clone
                        .save_adapter(&adapter, "concurrent_test")
                        .await
                })
            })
            .collect();

        // Wait for all saves to complete
        for handle in handles {
            handle.await??;
        }

        // Verify we have 10 versions
        let count = storage.count_adapters(user_id).await?;
        assert_eq!(count, 10);

        // Verify versions are 1-10
        let adapters = storage.list_adapters(user_id).await?;
        let versions: Vec<_> = adapters.iter().map(|a| a.version).collect();
        for v in 1..=10 {
            assert!(versions.contains(&v), "Missing version {}", v);
        }

        cleanup_user(&pool, user_id).await?;
        Ok(())
    }
}
