//! Production-ready health check system for Media Gateway services
//!
//! Provides comprehensive health monitoring for all service dependencies including
//! PostgreSQL, Redis, and Qdrant vector database. Health checks run in parallel with
//! configurable timeouts and support both simple and detailed health endpoints.
//!
//! ## Features
//!
//! - Parallel health check execution for minimal latency
//! - Per-check 2-second timeout protection
//! - Critical vs non-critical component classification
//! - Degraded state detection for partial outages
//! - Latency tracking for all components
//! - Version information in responses
//!
//! ## Usage
//!
//! ```rust,no_run
//! use media_gateway_core::health::{HealthChecker, AggregatedHealth};
//!
//! async fn health_endpoint(checker: &HealthChecker) -> AggregatedHealth {
//!     checker.check_all().await
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, warn};

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Some non-critical components failing
    Degraded,
    /// Critical components failing
    Unhealthy,
}

impl HealthStatus {
    /// Check if status is acceptable for serving traffic
    pub fn is_ready(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Get HTTP status code for this health status
    pub fn http_status_code(&self) -> u16 {
        match self {
            HealthStatus::Healthy => 200,
            HealthStatus::Degraded => 200, // Still serving traffic
            HealthStatus::Unhealthy => 503,
        }
    }
}

/// Health check result for a single component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name (e.g., "postgres", "redis", "qdrant")
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Check latency in milliseconds
    pub latency_ms: u64,
    /// Optional status message or error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Whether this component is critical for service operation
    pub critical: bool,
}

impl ComponentHealth {
    /// Create a healthy component result
    pub fn healthy(name: impl Into<String>, latency_ms: u64, critical: bool) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            latency_ms,
            message: None,
            critical,
        }
    }

    /// Create an unhealthy component result
    pub fn unhealthy(
        name: impl Into<String>,
        latency_ms: u64,
        critical: bool,
        message: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            latency_ms,
            message: Some(message.into()),
            critical,
        }
    }
}

/// Aggregated health status for the entire service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedHealth {
    /// Overall service status
    pub status: HealthStatus,
    /// Individual component health checks
    pub components: Vec<ComponentHealth>,
    /// Service version
    pub version: String,
    /// Timestamp of health check
    pub timestamp: DateTime<Utc>,
    /// Total time to complete all health checks (ms)
    pub total_latency_ms: u64,
}

impl AggregatedHealth {
    /// Determine overall status from component health checks
    pub fn from_components(components: Vec<ComponentHealth>, total_latency_ms: u64) -> Self {
        let status = if components
            .iter()
            .any(|c| c.critical && c.status == HealthStatus::Unhealthy)
        {
            HealthStatus::Unhealthy
        } else if components.iter().any(|c| c.status != HealthStatus::Healthy) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        Self {
            status,
            components,
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: Utc::now(),
            total_latency_ms,
        }
    }

    /// Get HTTP status code for the aggregated health
    pub fn http_status_code(&self) -> u16 {
        self.status.http_status_code()
    }

    /// Check if service is ready to serve traffic
    pub fn is_ready(&self) -> bool {
        self.status.is_ready()
    }
}

/// Simple health response for /health endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleHealth {
    pub status: HealthStatus,
    pub version: String,
}

impl From<&AggregatedHealth> for SimpleHealth {
    fn from(health: &AggregatedHealth) -> Self {
        Self {
            status: health.status,
            version: health.version.clone(),
        }
    }
}

/// Trait for implementing health checks
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform the health check
    async fn check(&self) -> ComponentHealth;

    /// Get the component name
    fn name(&self) -> &str;

    /// Is this a critical component?
    fn is_critical(&self) -> bool;
}

/// PostgreSQL health checker
pub struct PostgresHealthCheck {
    pool: PgPool,
    name: String,
    critical: bool,
}

impl PostgresHealthCheck {
    /// Create new PostgreSQL health checker
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            name: "postgres".to_string(),
            critical: true, // Database is critical
        }
    }

    /// Create with custom name
    pub fn with_name(pool: PgPool, name: impl Into<String>) -> Self {
        Self {
            pool,
            name: name.into(),
            critical: true,
        }
    }

    /// Set whether this check is critical
    pub fn set_critical(mut self, critical: bool) -> Self {
        self.critical = critical;
        self
    }
}

#[async_trait::async_trait]
impl HealthCheck for PostgresHealthCheck {
    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();

        // Execute health check with 2-second timeout
        let result = timeout(Duration::from_secs(2), async {
            sqlx::query("SELECT 1").fetch_one(&self.pool).await
        })
        .await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(_)) => {
                debug!("PostgreSQL health check passed ({}ms)", latency_ms);
                ComponentHealth::healthy(&self.name, latency_ms, self.critical)
            }
            Ok(Err(e)) => {
                warn!("PostgreSQL health check failed: {}", e);
                ComponentHealth::unhealthy(
                    &self.name,
                    latency_ms,
                    self.critical,
                    format!("Database query failed: {}", e),
                )
            }
            Err(_) => {
                warn!("PostgreSQL health check timed out");
                ComponentHealth::unhealthy(
                    &self.name,
                    2000, // Timeout duration
                    self.critical,
                    "Health check timed out after 2s",
                )
            }
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_critical(&self) -> bool {
        self.critical
    }
}

/// Redis health checker
pub struct RedisHealthCheck {
    client: redis::Client,
    name: String,
    critical: bool,
}

impl RedisHealthCheck {
    /// Create new Redis health checker
    pub fn new(client: redis::Client) -> Self {
        Self {
            client,
            name: "redis".to_string(),
            critical: false, // Cache is typically non-critical
        }
    }

    /// Create with custom name
    pub fn with_name(client: redis::Client, name: impl Into<String>) -> Self {
        Self {
            client,
            name: name.into(),
            critical: false,
        }
    }

    /// Set whether this check is critical
    pub fn set_critical(mut self, critical: bool) -> Self {
        self.critical = critical;
        self
    }
}

#[async_trait::async_trait]
impl HealthCheck for RedisHealthCheck {
    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();

        // Execute health check with 2-second timeout
        let result = timeout(Duration::from_secs(2), async {
            let mut conn = self.client.get_multiplexed_async_connection().await?;
            redis::cmd("PING").query_async::<_, String>(&mut conn).await
        })
        .await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(response)) if response == "PONG" => {
                debug!("Redis health check passed ({}ms)", latency_ms);
                ComponentHealth::healthy(&self.name, latency_ms, self.critical)
            }
            Ok(Ok(response)) => {
                warn!("Redis health check unexpected response: {}", response);
                ComponentHealth::unhealthy(
                    &self.name,
                    latency_ms,
                    self.critical,
                    format!("Unexpected response: {}", response),
                )
            }
            Ok(Err(e)) => {
                warn!("Redis health check failed: {}", e);
                ComponentHealth::unhealthy(
                    &self.name,
                    latency_ms,
                    self.critical,
                    format!("Redis PING failed: {}", e),
                )
            }
            Err(_) => {
                warn!("Redis health check timed out");
                ComponentHealth::unhealthy(
                    &self.name,
                    2000, // Timeout duration
                    self.critical,
                    "Health check timed out after 2s",
                )
            }
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_critical(&self) -> bool {
        self.critical
    }
}

/// Qdrant health checker
pub struct QdrantHealthCheck {
    client: reqwest::Client,
    base_url: String,
    name: String,
    critical: bool,
}

impl QdrantHealthCheck {
    /// Create new Qdrant health checker
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            name: "qdrant".to_string(),
            critical: false, // Vector search is typically non-critical for basic operations
        }
    }

    /// Create with custom name
    pub fn with_name(base_url: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            name: name.into(),
            critical: false,
        }
    }

    /// Set whether this check is critical
    pub fn set_critical(mut self, critical: bool) -> Self {
        self.critical = critical;
        self
    }
}

#[async_trait::async_trait]
impl HealthCheck for QdrantHealthCheck {
    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();

        // Execute health check with 2-second timeout
        let health_url = format!("{}/health", self.base_url.trim_end_matches('/'));
        let result = timeout(Duration::from_secs(2), self.client.get(&health_url).send()).await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(response)) if response.status().is_success() => {
                debug!("Qdrant health check passed ({}ms)", latency_ms);
                ComponentHealth::healthy(&self.name, latency_ms, self.critical)
            }
            Ok(Ok(response)) => {
                let status = response.status();
                warn!("Qdrant health check failed with status: {}", status);
                ComponentHealth::unhealthy(
                    &self.name,
                    latency_ms,
                    self.critical,
                    format!("Health endpoint returned status: {}", status),
                )
            }
            Ok(Err(e)) => {
                warn!("Qdrant health check failed: {}", e);
                ComponentHealth::unhealthy(
                    &self.name,
                    latency_ms,
                    self.critical,
                    format!("HTTP request failed: {}", e),
                )
            }
            Err(_) => {
                warn!("Qdrant health check timed out");
                ComponentHealth::unhealthy(
                    &self.name,
                    2000, // Timeout duration
                    self.critical,
                    "Health check timed out after 2s",
                )
            }
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_critical(&self) -> bool {
        self.critical
    }
}

/// Main health checker that coordinates all component checks
pub struct HealthChecker {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    /// Add a health check
    pub fn add_check(mut self, check: impl HealthCheck + 'static) -> Self {
        self.checks.push(Box::new(check));
        self
    }

    /// Add PostgreSQL health check
    pub fn with_postgres(self, pool: PgPool) -> Self {
        self.add_check(PostgresHealthCheck::new(pool))
    }

    /// Add Redis health check
    pub fn with_redis(self, client: redis::Client) -> Self {
        self.add_check(RedisHealthCheck::new(client))
    }

    /// Add Qdrant health check
    pub fn with_qdrant(self, base_url: impl Into<String>) -> Self {
        self.add_check(QdrantHealthCheck::new(base_url))
    }

    /// Perform all health checks in parallel
    pub async fn check_all(&self) -> AggregatedHealth {
        let start = Instant::now();

        // Run all checks in parallel
        let futures: Vec<_> = self.checks.iter().map(|check| check.check()).collect();

        let components = futures::future::join_all(futures).await;

        let total_latency_ms = start.elapsed().as_millis() as u64;

        AggregatedHealth::from_components(components, total_latency_ms)
    }

    /// Get simple health status (for /health endpoint)
    pub async fn check_simple(&self) -> SimpleHealth {
        let health = self.check_all().await;
        SimpleHealth::from(&health)
    }

    /// Get ready status (for /health/ready endpoint)
    pub async fn check_ready(&self) -> AggregatedHealth {
        self.check_all().await
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_is_ready() {
        assert!(HealthStatus::Healthy.is_ready());
        assert!(HealthStatus::Degraded.is_ready());
        assert!(!HealthStatus::Unhealthy.is_ready());
    }

    #[test]
    fn test_health_status_http_codes() {
        assert_eq!(HealthStatus::Healthy.http_status_code(), 200);
        assert_eq!(HealthStatus::Degraded.http_status_code(), 200);
        assert_eq!(HealthStatus::Unhealthy.http_status_code(), 503);
    }

    #[test]
    fn test_component_health_healthy() {
        let health = ComponentHealth::healthy("test", 50, true);
        assert_eq!(health.name, "test");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.latency_ms, 50);
        assert!(health.message.is_none());
        assert!(health.critical);
    }

    #[test]
    fn test_component_health_unhealthy() {
        let health = ComponentHealth::unhealthy("test", 100, false, "Error occurred");
        assert_eq!(health.name, "test");
        assert_eq!(health.status, HealthStatus::Unhealthy);
        assert_eq!(health.latency_ms, 100);
        assert_eq!(health.message.as_deref(), Some("Error occurred"));
        assert!(!health.critical);
    }

    #[test]
    fn test_aggregated_health_all_healthy() {
        let components = vec![
            ComponentHealth::healthy("postgres", 10, true),
            ComponentHealth::healthy("redis", 5, false),
        ];
        let health = AggregatedHealth::from_components(components, 15);
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.is_ready());
        assert_eq!(health.http_status_code(), 200);
    }

    #[test]
    fn test_aggregated_health_critical_unhealthy() {
        let components = vec![
            ComponentHealth::unhealthy("postgres", 2000, true, "Timeout"),
            ComponentHealth::healthy("redis", 5, false),
        ];
        let health = AggregatedHealth::from_components(components, 2005);
        assert_eq!(health.status, HealthStatus::Unhealthy);
        assert!(!health.is_ready());
        assert_eq!(health.http_status_code(), 503);
    }

    #[test]
    fn test_aggregated_health_degraded() {
        let components = vec![
            ComponentHealth::healthy("postgres", 10, true),
            ComponentHealth::unhealthy("redis", 2000, false, "Timeout"),
        ];
        let health = AggregatedHealth::from_components(components, 2010);
        assert_eq!(health.status, HealthStatus::Degraded);
        assert!(health.is_ready()); // Still ready with degraded non-critical component
        assert_eq!(health.http_status_code(), 200);
    }

    #[test]
    fn test_simple_health_from_aggregated() {
        let components = vec![ComponentHealth::healthy("postgres", 10, true)];
        let aggregated = AggregatedHealth::from_components(components, 10);
        let simple: SimpleHealth = (&aggregated).into();
        assert_eq!(simple.status, HealthStatus::Healthy);
        assert_eq!(simple.version, aggregated.version);
    }
}
