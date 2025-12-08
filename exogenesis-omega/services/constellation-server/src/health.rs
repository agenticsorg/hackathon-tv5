//! Health check implementation

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, warn};

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

/// Overall health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub components: Vec<ComponentHealth>,
    pub timestamp: i64,
}

/// Health checker
pub struct HealthChecker {
    postgres_pool: PgPool,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(postgres_pool: PgPool) -> Self {
        Self { postgres_pool }
    }

    /// Perform comprehensive health check
    pub async fn check(&self) -> HealthCheckResult {
        let mut components = Vec::new();

        // Check PostgreSQL
        components.push(self.check_postgres().await);

        // Determine overall status
        let status = if components.iter().any(|c| c.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else if components.iter().any(|c| c.status == HealthStatus::Degraded) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        HealthCheckResult {
            status,
            components,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Check PostgreSQL connection and RuVector extension
    async fn check_postgres(&self) -> ComponentHealth {
        let start = std::time::Instant::now();

        // Try simple query
        let result = sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.postgres_pool)
            .await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(1) => {
                debug!("PostgreSQL health check passed ({}ms)", latency_ms);

                // Also check if RuVector extension is available
                let ruvector_check = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'ruvector')"
                )
                .fetch_one(&self.postgres_pool)
                .await;

                match ruvector_check {
                    Ok(true) => ComponentHealth {
                        name: "postgres".to_string(),
                        status: HealthStatus::Healthy,
                        message: Some("PostgreSQL with RuVector extension".to_string()),
                        latency_ms: Some(latency_ms),
                    },
                    Ok(false) => {
                        warn!("RuVector extension not installed");
                        ComponentHealth {
                            name: "postgres".to_string(),
                            status: HealthStatus::Degraded,
                            message: Some("RuVector extension not found".to_string()),
                            latency_ms: Some(latency_ms),
                        }
                    }
                    Err(e) => {
                        warn!("Failed to check RuVector extension: {}", e);
                        ComponentHealth {
                            name: "postgres".to_string(),
                            status: HealthStatus::Degraded,
                            message: Some(format!("Extension check failed: {}", e)),
                            latency_ms: Some(latency_ms),
                        }
                    }
                }
            }
            Ok(_) => {
                warn!("PostgreSQL health check returned unexpected value");
                ComponentHealth {
                    name: "postgres".to_string(),
                    status: HealthStatus::Unhealthy,
                    message: Some("Unexpected response from database".to_string()),
                    latency_ms: Some(latency_ms),
                }
            }
            Err(e) => {
                warn!("PostgreSQL health check failed: {}", e);
                ComponentHealth {
                    name: "postgres".to_string(),
                    status: HealthStatus::Unhealthy,
                    message: Some(format!("Database error: {}", e)),
                    latency_ms: Some(latency_ms),
                }
            }
        }
    }

    /// Quick health check (just overall status)
    pub async fn is_healthy(&self) -> bool {
        let result = self.check().await;
        result.status == HealthStatus::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_serialization() {
        let health = ComponentHealth {
            name: "test".to_string(),
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: Some(10),
        };

        let json = serde_json::to_string(&health).unwrap();
        assert!(json.contains("\"status\":\"healthy\""));
    }

    #[test]
    fn test_overall_status_degraded() {
        let result = HealthCheckResult {
            status: HealthStatus::Degraded,
            components: vec![
                ComponentHealth {
                    name: "postgres".to_string(),
                    status: HealthStatus::Healthy,
                    message: None,
                    latency_ms: Some(5),
                },
                ComponentHealth {
                    name: "cache".to_string(),
                    status: HealthStatus::Degraded,
                    message: Some("High latency".to_string()),
                    latency_ms: Some(500),
                },
            ],
            timestamp: chrono::Utc::now().timestamp(),
        };

        assert_eq!(result.status, HealthStatus::Degraded);
        assert_eq!(result.components.len(), 2);
    }
}
