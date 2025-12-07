//! Experiment Repository - PostgreSQL storage for A/B testing experiments
//!
//! This module provides the storage layer for managing experiments, variants,
//! user assignments, and metrics collection.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::ab_testing::{Assignment, Experiment, ExperimentMetrics, Variant, VariantMetrics};

/// Experiment repository trait for abstraction
#[async_trait::async_trait]
pub trait ExperimentRepository: Send + Sync {
    /// Create a new experiment
    async fn create_experiment(
        &self,
        name: &str,
        description: Option<&str>,
        traffic_allocation: f64,
    ) -> Result<Experiment>;

    /// Get experiment by ID
    async fn get_experiment(&self, experiment_id: Uuid) -> Result<Option<Experiment>>;

    /// List all experiments
    async fn list_experiments(&self, status_filter: Option<&str>) -> Result<Vec<Experiment>>;

    /// Update experiment
    async fn update_experiment(
        &self,
        experiment_id: Uuid,
        status: Option<&str>,
        traffic_allocation: Option<f64>,
    ) -> Result<()>;

    /// Delete experiment
    async fn delete_experiment(&self, experiment_id: Uuid) -> Result<()>;

    /// Record user assignment to variant
    async fn record_assignment(
        &self,
        experiment_id: Uuid,
        user_id: Uuid,
        variant_id: Uuid,
    ) -> Result<Assignment>;

    /// Record experiment metric (exposure or conversion)
    async fn record_metric(
        &self,
        experiment_id: Uuid,
        variant_id: Uuid,
        user_id: Uuid,
        metric_name: &str,
        value: f64,
        metadata: Option<serde_json::Value>,
    ) -> Result<()>;

    /// Add variant to experiment
    async fn add_variant(
        &self,
        experiment_id: Uuid,
        name: &str,
        weight: f64,
        config: serde_json::Value,
    ) -> Result<Variant>;

    /// Get variants for experiment
    async fn get_variants(&self, experiment_id: Uuid) -> Result<Vec<Variant>>;

    /// Get experiment metrics
    async fn get_experiment_metrics(&self, experiment_id: Uuid) -> Result<ExperimentMetrics>;
}

/// PostgreSQL implementation of ExperimentRepository
#[derive(Clone)]
pub struct PostgresExperimentRepository {
    pool: PgPool,
}

impl PostgresExperimentRepository {
    /// Create new repository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ExperimentRepository for PostgresExperimentRepository {
    #[instrument(skip(self))]
    async fn create_experiment(
        &self,
        name: &str,
        description: Option<&str>,
        traffic_allocation: f64,
    ) -> Result<Experiment> {
        let experiment = sqlx::query_as::<_, Experiment>(
            r#"
            INSERT INTO experiments (name, description, traffic_allocation)
            VALUES ($1, $2, $3)
            RETURNING id, name, description, status, traffic_allocation, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(traffic_allocation)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create experiment")?;

        info!(experiment_id = %experiment.id, name = %name, "Created experiment");
        Ok(experiment)
    }

    async fn get_experiment(&self, experiment_id: Uuid) -> Result<Option<Experiment>> {
        let experiment = sqlx::query_as::<_, Experiment>(
            r#"
            SELECT id, name, description, status, traffic_allocation, created_at, updated_at
            FROM experiments
            WHERE id = $1
            "#,
        )
        .bind(experiment_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch experiment")?;

        Ok(experiment)
    }

    async fn list_experiments(&self, status_filter: Option<&str>) -> Result<Vec<Experiment>> {
        let experiments = if let Some(status) = status_filter {
            sqlx::query_as::<_, Experiment>(
                r#"
                SELECT id, name, description, status, traffic_allocation, created_at, updated_at
                FROM experiments
                WHERE status = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(status)
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch experiments by status")?
        } else {
            sqlx::query_as::<_, Experiment>(
                r#"
                SELECT id, name, description, status, traffic_allocation, created_at, updated_at
                FROM experiments
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch all experiments")?
        };

        Ok(experiments)
    }

    #[instrument(skip(self))]
    async fn update_experiment(
        &self,
        experiment_id: Uuid,
        status: Option<&str>,
        traffic_allocation: Option<f64>,
    ) -> Result<()> {
        // Build dynamic update query based on provided fields
        let mut updates = Vec::new();
        let mut param_count = 1;

        if status.is_some() {
            updates.push(format!("status = ${}", param_count));
            param_count += 1;
        }
        if traffic_allocation.is_some() {
            updates.push(format!("traffic_allocation = ${}", param_count));
            param_count += 1;
        }
        updates.push(format!("updated_at = ${}", param_count));

        if updates.is_empty() {
            return Ok(());
        }

        let query = format!(
            "UPDATE experiments SET {} WHERE id = ${}",
            updates.join(", "),
            param_count + 1
        );

        let mut q = sqlx::query(&query);
        if let Some(s) = status {
            q = q.bind(s);
        }
        if let Some(t) = traffic_allocation {
            q = q.bind(t);
        }
        q = q.bind(Utc::now()).bind(experiment_id);

        q.execute(&self.pool)
            .await
            .context("Failed to update experiment")?;

        info!(experiment_id = %experiment_id, "Updated experiment");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete_experiment(&self, experiment_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM experiments WHERE id = $1")
            .bind(experiment_id)
            .execute(&self.pool)
            .await
            .context("Failed to delete experiment")?;

        info!(experiment_id = %experiment_id, "Deleted experiment");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn record_assignment(
        &self,
        experiment_id: Uuid,
        user_id: Uuid,
        variant_id: Uuid,
    ) -> Result<Assignment> {
        let assignment = sqlx::query_as::<_, Assignment>(
            r#"
            INSERT INTO experiment_assignments (experiment_id, user_id, variant_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (experiment_id, user_id) DO UPDATE SET variant_id = EXCLUDED.variant_id
            RETURNING id, experiment_id, user_id, variant_id, assigned_at
            "#,
        )
        .bind(experiment_id)
        .bind(user_id)
        .bind(variant_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to record assignment")?;

        debug!(
            experiment_id = %experiment_id,
            user_id = %user_id,
            variant_id = %variant_id,
            "Recorded assignment"
        );
        Ok(assignment)
    }

    #[instrument(skip(self, metadata))]
    async fn record_metric(
        &self,
        experiment_id: Uuid,
        variant_id: Uuid,
        user_id: Uuid,
        metric_name: &str,
        value: f64,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        // Check if we're using the old experiment_metrics table or new split tables
        // Try inserting into experiment_exposures or experiment_conversions based on metric_name
        if metric_name == "exposure" {
            sqlx::query(
                r#"
                INSERT INTO experiment_exposures (experiment_id, variant_id, user_id, context)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(experiment_id)
            .bind(variant_id)
            .bind(user_id)
            .bind(metadata.unwrap_or(serde_json::json!({})))
            .execute(&self.pool)
            .await
            .context("Failed to record exposure")?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO experiment_conversions (experiment_id, variant_id, user_id, metric_name, value, metadata)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(experiment_id)
            .bind(variant_id)
            .bind(user_id)
            .bind(metric_name)
            .bind(value)
            .bind(metadata.unwrap_or(serde_json::json!({})))
            .execute(&self.pool)
            .await
            .context("Failed to record conversion")?;
        }

        debug!(
            experiment_id = %experiment_id,
            variant_id = %variant_id,
            metric_name = %metric_name,
            value = %value,
            "Recorded metric"
        );
        Ok(())
    }

    #[instrument(skip(self))]
    async fn add_variant(
        &self,
        experiment_id: Uuid,
        name: &str,
        weight: f64,
        config: serde_json::Value,
    ) -> Result<Variant> {
        let variant = sqlx::query_as::<_, Variant>(
            r#"
            INSERT INTO experiment_variants (experiment_id, name, weight, config)
            VALUES ($1, $2, $3, $4)
            RETURNING id, experiment_id, name, weight, config
            "#,
        )
        .bind(experiment_id)
        .bind(name)
        .bind(weight)
        .bind(&config)
        .fetch_one(&self.pool)
        .await
        .context("Failed to add variant")?;

        info!(
            variant_id = %variant.id,
            experiment_id = %experiment_id,
            name = %name,
            "Added variant"
        );
        Ok(variant)
    }

    async fn get_variants(&self, experiment_id: Uuid) -> Result<Vec<Variant>> {
        let variants = sqlx::query_as::<_, Variant>(
            "SELECT id, experiment_id, name, weight, config FROM experiment_variants WHERE experiment_id = $1",
        )
        .bind(experiment_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch variants")?;

        Ok(variants)
    }

    #[instrument(skip(self))]
    async fn get_experiment_metrics(&self, experiment_id: Uuid) -> Result<ExperimentMetrics> {
        let variants = self.get_variants(experiment_id).await?;
        let mut variant_metrics = Vec::new();

        for variant in variants {
            // Count exposures
            let exposures: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM experiment_exposures WHERE experiment_id = $1 AND variant_id = $2",
            )
            .bind(experiment_id)
            .bind(variant.id)
            .fetch_one(&self.pool)
            .await?;

            // Count conversions
            let conversions: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM experiment_conversions WHERE experiment_id = $1 AND variant_id = $2",
            )
            .bind(experiment_id)
            .bind(variant.id)
            .fetch_one(&self.pool)
            .await?;

            // Average conversion value
            let avg_value: (Option<f64>,) = sqlx::query_as(
                "SELECT AVG(value) FROM experiment_conversions WHERE experiment_id = $1 AND variant_id = $2",
            )
            .bind(experiment_id)
            .bind(variant.id)
            .fetch_one(&self.pool)
            .await?;

            let conversion_rate = if exposures.0 > 0 {
                conversions.0 as f64 / exposures.0 as f64
            } else {
                0.0
            };

            variant_metrics.push(VariantMetrics {
                variant_id: variant.id,
                variant_name: variant.name,
                exposures: exposures.0,
                conversions: conversions.0,
                conversion_rate,
                avg_metric_value: avg_value.0.unwrap_or(0.0),
            });
        }

        Ok(ExperimentMetrics {
            experiment_id,
            variant_metrics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_trait_bounds() {
        // Ensure PostgresExperimentRepository implements required traits
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PostgresExperimentRepository>();
    }
}
