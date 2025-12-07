//! A/B Testing Framework for SONA Recommendations
//!
//! Provides experiment configuration, user assignment, and metrics collection
//! for testing different recommendation strategies.
//!
//! # Database Schema (Migration)
//! ```sql
//! CREATE TABLE experiments (
//!     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
//!     name VARCHAR(255) NOT NULL UNIQUE,
//!     description TEXT,
//!     status VARCHAR(50) NOT NULL DEFAULT 'draft',
//!     traffic_allocation FLOAT NOT NULL DEFAULT 1.0,
//!     created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
//!     updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
//! );
//!
//! CREATE TABLE experiment_variants (
//!     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
//!     experiment_id UUID NOT NULL REFERENCES experiments(id),
//!     name VARCHAR(255) NOT NULL,
//!     weight FLOAT NOT NULL DEFAULT 0.5,
//!     config JSONB NOT NULL DEFAULT '{}',
//!     UNIQUE(experiment_id, name)
//! );
//!
//! CREATE TABLE experiment_assignments (
//!     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
//!     experiment_id UUID NOT NULL REFERENCES experiments(id),
//!     user_id UUID NOT NULL,
//!     variant_id UUID NOT NULL REFERENCES experiment_variants(id),
//!     assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
//!     UNIQUE(experiment_id, user_id)
//! );
//!
//! CREATE TABLE experiment_metrics (
//!     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
//!     experiment_id UUID NOT NULL REFERENCES experiments(id),
//!     variant_id UUID NOT NULL REFERENCES experiment_variants(id),
//!     user_id UUID NOT NULL,
//!     metric_name VARCHAR(255) NOT NULL,
//!     metric_value FLOAT NOT NULL,
//!     recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
//! );
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::experiment_repository::{ExperimentRepository, PostgresExperimentRepository};

/// Experiment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum ExperimentStatus {
    Draft,
    Running,
    Paused,
    Completed,
}

impl Default for ExperimentStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// Experiment configuration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Experiment {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub traffic_allocation: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Experiment variant
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Variant {
    pub id: Uuid,
    pub experiment_id: Uuid,
    pub name: String,
    pub weight: f64,
    pub config: serde_json::Value,
}

/// Experiment assignment record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Assignment {
    pub id: Uuid,
    pub experiment_id: Uuid,
    pub user_id: Uuid,
    pub variant_id: Uuid,
    pub assigned_at: DateTime<Utc>,
}

/// Experiment metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentMetrics {
    pub experiment_id: Uuid,
    pub variant_metrics: Vec<VariantMetrics>,
}

/// Metrics for a single variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantMetrics {
    pub variant_id: Uuid,
    pub variant_name: String,
    pub exposures: i64,
    pub conversions: i64,
    pub conversion_rate: f64,
    pub avg_metric_value: f64,
}

/// A/B Testing service
pub struct ABTestingService {
    pool: PgPool,
    repository: Arc<dyn ExperimentRepository>,
}

impl ABTestingService {
    /// Create new A/B testing service with default PostgreSQL repository
    pub fn new(pool: PgPool) -> Self {
        let repository = Arc::new(PostgresExperimentRepository::new(pool.clone()));
        Self { pool, repository }
    }

    /// Create new A/B testing service with custom repository
    pub fn with_repository(pool: PgPool, repository: Arc<dyn ExperimentRepository>) -> Self {
        Self { pool, repository }
    }

    /// Get reference to the experiment repository
    pub fn repository(&self) -> &Arc<dyn ExperimentRepository> {
        &self.repository
    }

    /// Create a new experiment
    #[instrument(skip(self))]
    pub async fn create_experiment(
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

    /// Add variant to experiment
    #[instrument(skip(self))]
    pub async fn add_variant(
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

        info!(variant_id = %variant.id, experiment_id = %experiment_id, name = %name, "Added variant");
        Ok(variant)
    }

    /// Start an experiment
    #[instrument(skip(self))]
    pub async fn start_experiment(&self, experiment_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE experiments SET status = 'running', updated_at = NOW() WHERE id = $1")
            .bind(experiment_id)
            .execute(&self.pool)
            .await
            .context("Failed to start experiment")?;

        info!(experiment_id = %experiment_id, "Started experiment");
        Ok(())
    }

    /// Get running experiments
    pub async fn get_running_experiments(&self) -> Result<Vec<Experiment>> {
        let experiments = sqlx::query_as::<_, Experiment>(
            "SELECT id, name, description, status, traffic_allocation, created_at, updated_at FROM experiments WHERE status = 'running'"
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch running experiments")?;

        Ok(experiments)
    }

    /// Get variants for experiment
    pub async fn get_variants(&self, experiment_id: Uuid) -> Result<Vec<Variant>> {
        let variants = sqlx::query_as::<_, Variant>(
            "SELECT id, experiment_id, name, weight, config FROM experiment_variants WHERE experiment_id = $1"
        )
        .bind(experiment_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch variants")?;

        Ok(variants)
    }

    /// Assign user to variant (deterministic based on user_id)
    #[instrument(skip(self))]
    pub async fn assign_variant(&self, experiment_id: Uuid, user_id: Uuid) -> Result<Variant> {
        // Check for existing assignment
        if let Some(existing) = sqlx::query_as::<_, Assignment>(
            "SELECT id, experiment_id, user_id, variant_id, assigned_at FROM experiment_assignments WHERE experiment_id = $1 AND user_id = $2"
        )
        .bind(experiment_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        {
            return self.get_variant_by_id(existing.variant_id).await;
        }

        // Get variants and select based on consistent hash
        let variants = self.get_variants(experiment_id).await?;
        if variants.is_empty() {
            anyhow::bail!("No variants defined for experiment {}", experiment_id);
        }

        let selected = self.select_variant_by_hash(&variants, user_id);

        // Record assignment
        sqlx::query(
            "INSERT INTO experiment_assignments (experiment_id, user_id, variant_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
        )
        .bind(experiment_id)
        .bind(user_id)
        .bind(selected.id)
        .execute(&self.pool)
        .await?;

        debug!(experiment_id = %experiment_id, user_id = %user_id, variant_id = %selected.id, "Assigned variant");
        Ok(selected)
    }

    /// Select variant using consistent hashing
    fn select_variant_by_hash(&self, variants: &[Variant], user_id: Uuid) -> Variant {
        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        let hash = hasher.finish();
        let normalized = (hash as f64) / (u64::MAX as f64);

        let total_weight: f64 = variants.iter().map(|v| v.weight).sum();
        let mut cumulative = 0.0;

        for variant in variants {
            cumulative += variant.weight / total_weight;
            if normalized < cumulative {
                return variant.clone();
            }
        }

        variants.last().unwrap().clone()
    }

    /// Get variant by ID
    async fn get_variant_by_id(&self, variant_id: Uuid) -> Result<Variant> {
        sqlx::query_as::<_, Variant>(
            "SELECT id, experiment_id, name, weight, config FROM experiment_variants WHERE id = $1",
        )
        .bind(variant_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch variant")
    }

    /// Record exposure (user saw recommendation with variant)
    #[instrument(skip(self))]
    pub async fn record_exposure(
        &self,
        experiment_id: Uuid,
        variant_id: Uuid,
        user_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO experiment_metrics (experiment_id, variant_id, user_id, metric_name, metric_value) VALUES ($1, $2, $3, 'exposure', 1.0)"
        )
        .bind(experiment_id)
        .bind(variant_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .context("Failed to record exposure")?;

        debug!(experiment_id = %experiment_id, variant_id = %variant_id, user_id = %user_id, "Recorded exposure");
        Ok(())
    }

    /// Record conversion (user clicked/watched recommended content)
    #[instrument(skip(self))]
    pub async fn record_conversion(
        &self,
        experiment_id: Uuid,
        variant_id: Uuid,
        user_id: Uuid,
        metric_name: &str,
        value: f64,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO experiment_metrics (experiment_id, variant_id, user_id, metric_name, metric_value) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(experiment_id)
        .bind(variant_id)
        .bind(user_id)
        .bind(metric_name)
        .bind(value)
        .execute(&self.pool)
        .await
        .context("Failed to record conversion")?;

        debug!(experiment_id = %experiment_id, metric_name = %metric_name, value = %value, "Recorded conversion");
        Ok(())
    }

    /// Get experiment metrics
    #[instrument(skip(self))]
    pub async fn get_experiment_metrics(&self, experiment_id: Uuid) -> Result<ExperimentMetrics> {
        let variants = self.get_variants(experiment_id).await?;
        let mut variant_metrics = Vec::new();

        for variant in variants {
            let exposures: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM experiment_metrics WHERE experiment_id = $1 AND variant_id = $2 AND metric_name = 'exposure'"
            )
            .bind(experiment_id)
            .bind(variant.id)
            .fetch_one(&self.pool)
            .await?;

            let conversions: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM experiment_metrics WHERE experiment_id = $1 AND variant_id = $2 AND metric_name = 'conversion'"
            )
            .bind(experiment_id)
            .bind(variant.id)
            .fetch_one(&self.pool)
            .await?;

            let avg_value: (Option<f64>,) = sqlx::query_as(
                "SELECT AVG(metric_value) FROM experiment_metrics WHERE experiment_id = $1 AND variant_id = $2 AND metric_name != 'exposure'"
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
    fn test_consistent_hashing() {
        let variants = vec![
            Variant {
                id: Uuid::new_v4(),
                experiment_id: Uuid::new_v4(),
                name: "control".to_string(),
                weight: 0.5,
                config: serde_json::json!({}),
            },
            Variant {
                id: Uuid::new_v4(),
                experiment_id: Uuid::new_v4(),
                name: "treatment".to_string(),
                weight: 0.5,
                config: serde_json::json!({}),
            },
        ];

        let user_id = Uuid::new_v4();
        let service = ABTestingService {
            pool: unsafe { std::mem::zeroed() },
        }; // Note: only for hash test

        // Same user should always get same variant
        let v1 = service.select_variant_by_hash(&variants, user_id);
        let v2 = service.select_variant_by_hash(&variants, user_id);
        assert_eq!(
            v1.id, v2.id,
            "Consistent hashing should return same variant"
        );
    }

    #[test]
    fn test_weight_distribution() {
        let variants = vec![
            Variant {
                id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                experiment_id: Uuid::new_v4(),
                name: "control".to_string(),
                weight: 0.8,
                config: serde_json::json!({}),
            },
            Variant {
                id: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                experiment_id: Uuid::new_v4(),
                name: "treatment".to_string(),
                weight: 0.2,
                config: serde_json::json!({}),
            },
        ];

        let service = ABTestingService {
            pool: unsafe { std::mem::zeroed() },
        };
        let mut control_count = 0;
        let mut treatment_count = 0;

        for i in 0..1000 {
            let user_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("user{}", i).as_bytes());
            let selected = service.select_variant_by_hash(&variants, user_id);
            if selected.name == "control" {
                control_count += 1;
            } else {
                treatment_count += 1;
            }
        }

        // Should be roughly 80/20 split (with some variance)
        let control_ratio = control_count as f64 / 1000.0;
        assert!(
            control_ratio > 0.7 && control_ratio < 0.9,
            "Control should be ~80%, got {}",
            control_ratio
        );
    }
}
