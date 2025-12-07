//! Example: LoRA Model Persistence and Loading
//!
//! Demonstrates how to use the LoRAStorage module to save, load, and manage
//! UserLoRAAdapter models with PostgreSQL persistence.
//!
//! Run with:
//! ```bash
//! export DATABASE_URL="postgres://postgres:postgres@localhost:5432/media_gateway"
//! cargo run --example lora_storage_example
//! ```

use anyhow::Result;
use media_gateway_sona::{
    BuildUserPreferenceVector, LoRAStorage, UpdateUserLoRA, UserLoRAAdapter, UserProfile,
};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Connect to PostgreSQL
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Connected to PostgreSQL");

    // Initialize storage
    let storage = LoRAStorage::new(pool);

    // Example 1: Save and load adapter
    println!("\n=== Example 1: Basic Save/Load ===");
    let user_id = Uuid::new_v4();
    let mut adapter = UserLoRAAdapter::new(user_id);
    adapter.initialize_random();

    println!("Saving adapter for user {}...", user_id);
    let version = storage.save_adapter(&adapter, "default").await?;
    println!("✓ Saved adapter version {}", version);

    println!("Loading adapter...");
    let loaded = storage.load_adapter(user_id, "default").await?;
    println!(
        "✓ Loaded adapter (training_iterations: {})",
        loaded.training_iterations
    );

    // Example 2: Versioning
    println!("\n=== Example 2: Versioning ===");
    adapter.training_iterations = 5;
    let v2 = storage.save_adapter(&adapter, "default").await?;
    println!("✓ Saved version {}", v2);

    adapter.training_iterations = 10;
    let v3 = storage.save_adapter(&adapter, "default").await?;
    println!("✓ Saved version {}", v3);

    // Load latest
    let latest = storage.load_adapter(user_id, "default").await?;
    println!(
        "Latest version training_iterations: {}",
        latest.training_iterations
    );

    // Load specific version
    let v1 = storage.load_adapter_version(user_id, "default", 1).await?;
    println!("Version 1 training_iterations: {}", v1.training_iterations);

    // Example 3: List adapters
    println!("\n=== Example 3: List Adapters ===");
    let adapters = storage.list_adapters(user_id).await?;
    println!("Found {} adapter version(s):", adapters.len());
    for meta in &adapters {
        println!(
            "  - {} v{}: {} bytes, {} iterations, updated {}",
            meta.adapter_name,
            meta.version,
            meta.size_bytes,
            meta.updated_at.format("%Y-%m-%d %H:%M:%S"),
            meta.version
        );
    }

    // Example 4: A/B Testing
    println!("\n=== Example 4: A/B Testing ===");
    let mut experimental = UserLoRAAdapter::new(user_id);
    experimental.initialize_random();
    experimental.training_iterations = 20;

    storage.save_adapter(&experimental, "experimental").await?;
    println!("✓ Saved experimental adapter");

    // Simulate user buckets
    let use_experimental = true;
    let adapter_name = if use_experimental {
        "experimental"
    } else {
        "default"
    };

    let test_adapter = storage.load_adapter(user_id, adapter_name).await?;
    println!(
        "Loaded {} adapter (iterations: {})",
        adapter_name, test_adapter.training_iterations
    );

    // Example 5: Storage statistics
    println!("\n=== Example 5: Storage Statistics ===");
    let stats = storage.get_storage_stats().await?;
    println!("Total adapters: {}", stats.total_adapters);
    println!("Unique users: {}", stats.unique_users);
    println!("Total storage: {} MB", stats.total_bytes / 1_000_000);
    println!("Average size: {:.2} KB", stats.avg_bytes / 1_000.0);

    let count = storage.count_adapters(user_id).await?;
    println!("Adapters for user {}: {}", user_id, count);

    // Example 6: Cleanup
    println!("\n=== Example 6: Cleanup ===");

    // Delete specific version
    let deleted_v1 = storage
        .delete_adapter_version(user_id, "default", 1)
        .await?;
    println!("Deleted version 1: {}", deleted_v1);

    // Delete all versions of experimental
    let deleted_exp = storage.delete_adapter(user_id, "experimental").await?;
    println!("Deleted {} experimental version(s)", deleted_exp);

    // Delete all remaining adapters for user
    let deleted_all = storage.delete_adapter(user_id, "default").await?;
    println!("Deleted {} remaining version(s)", deleted_all);

    println!("\n✓ All examples completed successfully!");

    Ok(())
}
