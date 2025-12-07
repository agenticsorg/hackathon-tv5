//! A/B Testing Framework Usage Example
//!
//! This example demonstrates the complete A/B testing workflow:
//! 1. Create an experiment with variants
//! 2. Start the experiment
//! 3. Assign users to variants
//! 4. Record exposures and conversions
//! 5. Analyze metrics
//!
//! Run with:
//! ```bash
//! cargo run --example ab_testing_example
//! ```

use anyhow::Result;
use media_gateway_sona::{
    Experiment, ExperimentRepository, ExperimentStatus,
};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("=== A/B Testing Framework Example ===\n");

    // Connect to database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/media_gateway".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let repo = ExperimentRepository::new(pool);

    // ========================================================================
    // Step 1: Create an experiment
    // ========================================================================
    println!("Step 1: Creating experiment...");

    let mut experiment = Experiment::new(
        format!("lora_boost_test_{}", Uuid::new_v4()),
        Some("Test different LoRA boost factors for personalization".to_string()),
        0.8, // Only 80% of users included
    );

    // Add variants with different configurations
    experiment.add_variant(
        "control".to_string(),
        0.33,
        json!({
            "lora_boost": 0.3,
            "algorithm": "baseline"
        }),
    );

    experiment.add_variant(
        "treatment_a".to_string(),
        0.34,
        json!({
            "lora_boost": 0.5,
            "algorithm": "enhanced"
        }),
    );

    experiment.add_variant(
        "treatment_b".to_string(),
        0.33,
        json!({
            "lora_boost": 0.7,
            "algorithm": "aggressive"
        }),
    );

    let experiment_id = repo.create_experiment(&experiment).await?;
    println!("✓ Created experiment: {} ({})", experiment.name, experiment_id);
    println!("  Variants: {}", experiment.variants.len());
    println!("  Traffic allocation: {}%\n", experiment.traffic_allocation * 100.0);

    // ========================================================================
    // Step 2: Start the experiment
    // ========================================================================
    println!("Step 2: Starting experiment...");

    repo.update_status(experiment_id, ExperimentStatus::Running).await?;
    println!("✓ Experiment is now RUNNING\n");

    // ========================================================================
    // Step 3: Simulate user assignments
    // ========================================================================
    println!("Step 3: Assigning users to variants...");

    let num_users = 100;
    let mut variant_counts = std::collections::HashMap::new();

    for i in 0..num_users {
        let user_id = Uuid::new_v4();

        match repo.assign_variant(experiment_id, user_id).await {
            Ok(variant) => {
                *variant_counts.entry(variant.name.clone()).or_insert(0) += 1;

                // Record exposure
                repo.record_exposure(
                    experiment_id,
                    variant.id,
                    user_id,
                    Some(json!({
                        "device": "mobile",
                        "session_id": format!("session_{}", i)
                    })),
                ).await?;

                // Simulate conversion (50% conversion rate for demo)
                if i % 2 == 0 {
                    repo.record_conversion(
                        experiment_id,
                        variant.id,
                        user_id,
                        "watch_completion",
                        1.0,
                        Some(json!({
                            "duration_seconds": 3600 + (i * 10)
                        })),
                    ).await?;
                }

                // Simulate click-through (30% rate)
                if i % 3 == 0 {
                    repo.record_conversion(
                        experiment_id,
                        variant.id,
                        user_id,
                        "click_through",
                        1.0,
                        None,
                    ).await?;
                }
            }
            Err(_) => {
                // User not in traffic allocation
            }
        }
    }

    println!("✓ Assigned {} users across variants:", num_users);
    for (variant_name, count) in &variant_counts {
        let percentage = (*count as f64 / num_users as f64) * 100.0;
        println!("  - {}: {} users ({:.1}%)", variant_name, count, percentage);
    }
    println!();

    // ========================================================================
    // Step 4: Retrieve and analyze metrics
    // ========================================================================
    println!("Step 4: Analyzing experiment metrics...");

    let metrics = repo.get_experiment_metrics(experiment_id).await?;

    println!("✓ Metrics computed at: {}\n", metrics.computed_at);

    for (variant_id, variant_metrics) in &metrics.variant_metrics {
        println!("Variant: {} ({})", variant_metrics.variant_name, variant_id);
        println!("  Exposures: {}", variant_metrics.exposures);
        println!("  Unique users: {}", variant_metrics.unique_users);

        if let Some(watch_stats) = variant_metrics.conversions.get("watch_completion") {
            println!("  Watch Completions:");
            println!("    - Count: {}", watch_stats.count);
            println!("    - Conversion rate: {:.2}%", watch_stats.conversion_rate * 100.0);
            println!("    - Mean value: {:.2}", watch_stats.mean);
        }

        if let Some(click_stats) = variant_metrics.conversions.get("click_through") {
            println!("  Click-throughs:");
            println!("    - Count: {}", click_stats.count);
            println!("    - Conversion rate: {:.2}%", click_stats.conversion_rate * 100.0);
        }

        println!();
    }

    // ========================================================================
    // Step 5: Compare variants (statistical analysis placeholder)
    // ========================================================================
    println!("Step 5: Variant comparison...");

    let mut variant_scores: Vec<_> = metrics.variant_metrics.iter().collect();
    variant_scores.sort_by(|a, b| {
        let a_rate = a.1.conversions
            .get("watch_completion")
            .map(|s| s.conversion_rate)
            .unwrap_or(0.0);
        let b_rate = b.1.conversions
            .get("watch_completion")
            .map(|s| s.conversion_rate)
            .unwrap_or(0.0);
        b_rate.partial_cmp(&a_rate).unwrap()
    });

    println!("Variants ranked by watch completion rate:");
    for (rank, (_, vm)) in variant_scores.iter().enumerate() {
        let rate = vm.conversions
            .get("watch_completion")
            .map(|s| s.conversion_rate * 100.0)
            .unwrap_or(0.0);
        println!("  {}. {} - {:.2}%", rank + 1, vm.variant_name, rate);
    }
    println!();

    // ========================================================================
    // Step 6: Complete the experiment
    // ========================================================================
    println!("Step 6: Completing experiment...");

    repo.update_status(experiment_id, ExperimentStatus::Completed).await?;
    println!("✓ Experiment marked as COMPLETED\n");

    // ========================================================================
    // Verify consistency
    // ========================================================================
    println!("Step 7: Verifying user assignment consistency...");

    let test_user = Uuid::new_v4();
    let variant1 = repo.assign_variant(experiment_id, test_user).await;
    let variant2 = repo.assign_variant(experiment_id, test_user).await;

    match (variant1, variant2) {
        (Ok(v1), Ok(v2)) => {
            if v1.id == v2.id {
                println!("✓ User assignment is consistent (same variant on repeat calls)");
                println!("  User {} → Variant {}\n", test_user, v1.name);
            } else {
                println!("✗ ERROR: User assignment is NOT consistent!");
            }
        }
        (Err(e1), Err(e2)) => {
            println!("✓ User consistently not in experiment: {}", e1);
        }
        _ => {
            println!("✗ ERROR: Inconsistent assignment results!");
        }
    }

    // ========================================================================
    // Summary
    // ========================================================================
    println!("=== Example Complete ===\n");
    println!("Summary:");
    println!("- Experiment ID: {}", experiment_id);
    println!("- Total users processed: {}", num_users);
    println!("- Variants tested: {}", experiment.variants.len());
    println!("- Metrics tracked: watch_completion, click_through");
    println!();
    println!("Next steps:");
    println!("1. Implement statistical significance testing");
    println!("2. Set up automated reporting");
    println!("3. Create monitoring dashboards");
    println!("4. Define stopping criteria");

    Ok(())
}

// ============================================================================
// Additional helper functions
// ============================================================================

#[allow(dead_code)]
/// Calculate statistical significance (placeholder)
fn calculate_significance(
    control_conversions: i64,
    control_exposures: i64,
    treatment_conversions: i64,
    treatment_exposures: i64,
) -> (f64, f64) {
    // Simplified z-test for proportions
    let p1 = control_conversions as f64 / control_exposures as f64;
    let p2 = treatment_conversions as f64 / treatment_exposures as f64;

    let p = (control_conversions + treatment_conversions) as f64
        / (control_exposures + treatment_exposures) as f64;

    let se = (p * (1.0 - p) * (1.0 / control_exposures as f64 + 1.0 / treatment_exposures as f64)).sqrt();
    let z_score = (p2 - p1) / se;

    let lift = ((p2 - p1) / p1) * 100.0;

    (z_score, lift)
}

#[allow(dead_code)]
/// Check if sample size is sufficient (placeholder)
fn is_sample_size_sufficient(
    baseline_rate: f64,
    mde: f64, // Minimum detectable effect
    alpha: f64,
    power: f64,
) -> usize {
    // Simplified sample size calculation
    // In production, use proper statistical power analysis
    let z_alpha = 1.96; // For alpha = 0.05
    let z_beta = 0.84;  // For power = 0.80

    let p1 = baseline_rate;
    let p2 = baseline_rate * (1.0 + mde);

    let numerator = (z_alpha + z_beta).powi(2) * (p1 * (1.0 - p1) + p2 * (1.0 - p2));
    let denominator = (p2 - p1).powi(2);

    (numerator / denominator).ceil() as usize
}
