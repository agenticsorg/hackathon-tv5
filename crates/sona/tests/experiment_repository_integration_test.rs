//! Integration tests for ExperimentRepository
//!
//! These tests verify the PostgreSQL implementation of the ExperimentRepository trait
//! against a real database.

use anyhow::Result;
use media_gateway_sona::{ExperimentRepository, PostgresExperimentRepository};
use sqlx::PgPool;
use uuid::Uuid;

// Test database URL - should match development environment
const TEST_DATABASE_URL: &str =
    "postgresql://mediagateway:localdev123@localhost:5432/media_gateway";

async fn setup_test_pool() -> Result<PgPool> {
    let pool = PgPool::connect(TEST_DATABASE_URL).await?;
    Ok(pool)
}

#[tokio::test]
#[ignore] // Run with: cargo test --test experiment_repository_integration_test -- --ignored
async fn test_create_and_get_experiment() -> Result<()> {
    let pool = setup_test_pool().await?;
    let repo = PostgresExperimentRepository::new(pool);

    // Create experiment
    let name = format!("test_experiment_{}", Uuid::new_v4());
    let experiment = repo
        .create_experiment(&name, Some("Test description"), 1.0)
        .await?;

    assert_eq!(experiment.name, name);
    assert_eq!(experiment.description, Some("Test description".to_string()));
    assert_eq!(experiment.traffic_allocation, 1.0);

    // Get experiment by ID
    let retrieved = repo.get_experiment(experiment.id).await?;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, experiment.id);
    assert_eq!(retrieved.name, name);

    // Cleanup
    repo.delete_experiment(experiment.id).await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_list_experiments() -> Result<()> {
    let pool = setup_test_pool().await?;
    let repo = PostgresExperimentRepository::new(pool);

    // Create multiple experiments
    let exp1_name = format!("test_list_exp1_{}", Uuid::new_v4());
    let exp2_name = format!("test_list_exp2_{}", Uuid::new_v4());

    let exp1 = repo.create_experiment(&exp1_name, None, 0.5).await?;
    let exp2 = repo.create_experiment(&exp2_name, None, 0.8).await?;

    // List all experiments
    let all_experiments = repo.list_experiments(None).await?;
    assert!(all_experiments.len() >= 2);

    // List by status
    let draft_experiments = repo.list_experiments(Some("draft")).await?;
    assert!(draft_experiments.iter().any(|e| e.id == exp1.id));
    assert!(draft_experiments.iter().any(|e| e.id == exp2.id));

    // Cleanup
    repo.delete_experiment(exp1.id).await?;
    repo.delete_experiment(exp2.id).await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_update_experiment() -> Result<()> {
    let pool = setup_test_pool().await?;
    let repo = PostgresExperimentRepository::new(pool);

    // Create experiment
    let name = format!("test_update_{}", Uuid::new_v4());
    let experiment = repo.create_experiment(&name, None, 0.5).await?;

    // Update status
    repo.update_experiment(experiment.id, Some("running"), None)
        .await?;

    let updated = repo.get_experiment(experiment.id).await?.unwrap();
    assert_eq!(updated.status, "running");

    // Update traffic allocation
    repo.update_experiment(experiment.id, None, Some(0.75))
        .await?;

    let updated = repo.get_experiment(experiment.id).await?.unwrap();
    assert_eq!(updated.traffic_allocation, 0.75);

    // Cleanup
    repo.delete_experiment(experiment.id).await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_variants() -> Result<()> {
    let pool = setup_test_pool().await?;
    let repo = PostgresExperimentRepository::new(pool);

    // Create experiment
    let name = format!("test_variants_{}", Uuid::new_v4());
    let experiment = repo.create_experiment(&name, None, 1.0).await?;

    // Add variants
    let control = repo
        .add_variant(
            experiment.id,
            "control",
            0.5,
            serde_json::json!({"boost": 0.3}),
        )
        .await?;

    let treatment = repo
        .add_variant(
            experiment.id,
            "treatment",
            0.5,
            serde_json::json!({"boost": 0.5}),
        )
        .await?;

    assert_eq!(control.name, "control");
    assert_eq!(treatment.name, "treatment");

    // Get variants
    let variants = repo.get_variants(experiment.id).await?;
    assert_eq!(variants.len(), 2);
    assert!(variants.iter().any(|v| v.name == "control"));
    assert!(variants.iter().any(|v| v.name == "treatment"));

    // Cleanup
    repo.delete_experiment(experiment.id).await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_record_assignment() -> Result<()> {
    let pool = setup_test_pool().await?;
    let repo = PostgresExperimentRepository::new(pool);

    // Create experiment with variant
    let name = format!("test_assignment_{}", Uuid::new_v4());
    let experiment = repo.create_experiment(&name, None, 1.0).await?;
    let variant = repo
        .add_variant(experiment.id, "control", 1.0, serde_json::json!({}))
        .await?;

    // Record assignment
    let user_id = Uuid::new_v4();
    let assignment = repo
        .record_assignment(experiment.id, user_id, variant.id)
        .await?;

    assert_eq!(assignment.experiment_id, experiment.id);
    assert_eq!(assignment.user_id, user_id);
    assert_eq!(assignment.variant_id, variant.id);

    // Cleanup
    repo.delete_experiment(experiment.id).await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_record_metrics() -> Result<()> {
    let pool = setup_test_pool().await?;
    let repo = PostgresExperimentRepository::new(pool);

    // Create experiment with variant
    let name = format!("test_metrics_{}", Uuid::new_v4());
    let experiment = repo.create_experiment(&name, None, 1.0).await?;
    let variant = repo
        .add_variant(experiment.id, "control", 1.0, serde_json::json!({}))
        .await?;

    let user_id = Uuid::new_v4();

    // Record exposure
    repo.record_metric(
        experiment.id,
        variant.id,
        user_id,
        "exposure",
        1.0,
        Some(serde_json::json!({"device": "mobile"})),
    )
    .await?;

    // Record conversion
    repo.record_metric(
        experiment.id,
        variant.id,
        user_id,
        "watch_completion",
        0.85,
        Some(serde_json::json!({"duration_seconds": 120})),
    )
    .await?;

    // Get metrics
    let metrics = repo.get_experiment_metrics(experiment.id).await?;
    assert_eq!(metrics.experiment_id, experiment.id);
    assert_eq!(metrics.variant_metrics.len(), 1);

    let variant_metrics = &metrics.variant_metrics[0];
    assert_eq!(variant_metrics.variant_id, variant.id);
    assert_eq!(variant_metrics.exposures, 1);
    assert_eq!(variant_metrics.conversions, 1);
    assert_eq!(variant_metrics.conversion_rate, 1.0); // 1 conversion / 1 exposure

    // Cleanup
    repo.delete_experiment(experiment.id).await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_experiment_metrics_with_multiple_users() -> Result<()> {
    let pool = setup_test_pool().await?;
    let repo = PostgresExperimentRepository::new(pool);

    // Create experiment with two variants
    let name = format!("test_multi_metrics_{}", Uuid::new_v4());
    let experiment = repo.create_experiment(&name, None, 1.0).await?;

    let control = repo
        .add_variant(experiment.id, "control", 0.5, serde_json::json!({}))
        .await?;
    let treatment = repo
        .add_variant(experiment.id, "treatment", 0.5, serde_json::json!({}))
        .await?;

    // Simulate 10 users - 5 in control, 5 in treatment
    for i in 0..10 {
        let user_id = Uuid::new_v4();
        let variant = if i < 5 { &control } else { &treatment };

        // Record exposure
        repo.record_metric(experiment.id, variant.id, user_id, "exposure", 1.0, None)
            .await?;

        // 80% conversion rate for control, 90% for treatment
        let convert = if i < 5 { i < 4 } else { i < 9 };
        if convert {
            repo.record_metric(experiment.id, variant.id, user_id, "conversion", 1.0, None)
                .await?;
        }
    }

    // Get metrics
    let metrics = repo.get_experiment_metrics(experiment.id).await?;
    assert_eq!(metrics.variant_metrics.len(), 2);

    // Find control and treatment metrics
    let control_metrics = metrics
        .variant_metrics
        .iter()
        .find(|m| m.variant_name == "control")
        .unwrap();
    let treatment_metrics = metrics
        .variant_metrics
        .iter()
        .find(|m| m.variant_name == "treatment")
        .unwrap();

    assert_eq!(control_metrics.exposures, 5);
    assert_eq!(control_metrics.conversions, 4);
    assert_eq!(control_metrics.conversion_rate, 0.8);

    assert_eq!(treatment_metrics.exposures, 5);
    assert_eq!(treatment_metrics.conversions, 4);
    assert_eq!(treatment_metrics.conversion_rate, 0.8);

    // Cleanup
    repo.delete_experiment(experiment.id).await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_delete_experiment_cascades() -> Result<()> {
    let pool = setup_test_pool().await?;
    let repo = PostgresExperimentRepository::new(pool);

    // Create experiment with variant and metrics
    let name = format!("test_cascade_{}", Uuid::new_v4());
    let experiment = repo.create_experiment(&name, None, 1.0).await?;
    let variant = repo
        .add_variant(experiment.id, "control", 1.0, serde_json::json!({}))
        .await?;

    let user_id = Uuid::new_v4();
    repo.record_assignment(experiment.id, user_id, variant.id)
        .await?;
    repo.record_metric(experiment.id, variant.id, user_id, "exposure", 1.0, None)
        .await?;

    // Delete experiment
    repo.delete_experiment(experiment.id).await?;

    // Verify experiment is gone
    let retrieved = repo.get_experiment(experiment.id).await?;
    assert!(retrieved.is_none());

    // Verify variants are gone (cascade delete)
    let variants = repo.get_variants(experiment.id).await?;
    assert_eq!(variants.len(), 0);

    Ok(())
}
