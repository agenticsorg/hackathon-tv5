use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub name: String,
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    pub last_checked: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    pub name: String,
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    pub last_checked: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedHealth {
    pub status: HealthStatus,
    pub timestamp: DateTime<Utc>,
    pub services: Vec<ServiceHealth>,
    pub dependencies: Vec<DependencyHealth>,
    pub overall_latency_ms: u64,
}

#[derive(Debug, Clone)]
struct ServiceHealthCheck {
    name: String,
    url: String,
    timeout: Duration,
}

#[derive(Debug, Clone)]
struct DependencyHealthCheck {
    name: String,
    check_type: DependencyCheckType,
    timeout: Duration,
}

#[derive(Debug, Clone)]
enum DependencyCheckType {
    PostgreSQL(String),
    Redis(String),
    Qdrant(String),
}

#[derive(Debug, Clone)]
struct CachedHealth {
    health: AggregatedHealth,
    cached_at: Instant,
}

pub struct HealthAggregator {
    services: Vec<ServiceHealthCheck>,
    dependencies: Vec<DependencyHealthCheck>,
    cache: Arc<RwLock<Option<CachedHealth>>>,
    cache_ttl: Duration,
    http_client: reqwest::Client,
}

impl HealthAggregator {
    pub fn new(cache_ttl: Duration) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        let services = vec![
            ServiceHealthCheck {
                name: "discovery".to_string(),
                url: "http://discovery:8081/health".to_string(),
                timeout: Duration::from_secs(2),
            },
            ServiceHealthCheck {
                name: "sona".to_string(),
                url: "http://sona:8082/health".to_string(),
                timeout: Duration::from_secs(2),
            },
            ServiceHealthCheck {
                name: "auth".to_string(),
                url: "http://auth:8083/health".to_string(),
                timeout: Duration::from_secs(2),
            },
            ServiceHealthCheck {
                name: "sync".to_string(),
                url: "http://sync:8084/health".to_string(),
                timeout: Duration::from_secs(2),
            },
            ServiceHealthCheck {
                name: "ingestion".to_string(),
                url: "http://ingestion:8085/health".to_string(),
                timeout: Duration::from_secs(2),
            },
            ServiceHealthCheck {
                name: "playback".to_string(),
                url: "http://playback:8086/health".to_string(),
                timeout: Duration::from_secs(2),
            },
        ];

        let dependencies = vec![
            DependencyHealthCheck {
                name: "postgresql".to_string(),
                check_type: DependencyCheckType::PostgreSQL(
                    std::env::var("DATABASE_URL").unwrap_or_default(),
                ),
                timeout: Duration::from_secs(1),
            },
            DependencyHealthCheck {
                name: "redis".to_string(),
                check_type: DependencyCheckType::Redis(
                    std::env::var("REDIS_URL").unwrap_or_default(),
                ),
                timeout: Duration::from_secs(1),
            },
            DependencyHealthCheck {
                name: "qdrant".to_string(),
                check_type: DependencyCheckType::Qdrant(
                    std::env::var("QDRANT_URL")
                        .unwrap_or_else(|_| "http://qdrant:6333".to_string()),
                ),
                timeout: Duration::from_secs(1),
            },
        ];

        Self {
            services,
            dependencies,
            cache: Arc::new(RwLock::new(None)),
            cache_ttl,
            http_client,
        }
    }

    pub async fn check_health(&self) -> Result<AggregatedHealth, anyhow::Error> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.cached_at.elapsed() < self.cache_ttl {
                    return Ok(cached.health.clone());
                }
            }
        }

        // Perform health checks
        let start = Instant::now();

        let (services, dependencies) =
            tokio::join!(self.check_all_services(), self.check_all_dependencies());

        let overall_latency_ms = start.elapsed().as_millis() as u64;

        let status = Self::calculate_overall_status(&services, &dependencies);

        let health = AggregatedHealth {
            status,
            timestamp: Utc::now(),
            services,
            dependencies,
            overall_latency_ms,
        };

        // Update cache
        {
            let mut cache = self.cache.write().await;
            *cache = Some(CachedHealth {
                health: health.clone(),
                cached_at: Instant::now(),
            });
        }

        Ok(health)
    }

    async fn check_all_services(&self) -> Vec<ServiceHealth> {
        let futures: Vec<_> = self
            .services
            .iter()
            .map(|service| self.check_service(service))
            .collect();

        futures::future::join_all(futures).await
    }

    async fn check_service(&self, service: &ServiceHealthCheck) -> ServiceHealth {
        let start = Instant::now();
        let last_checked = Utc::now();

        match tokio::time::timeout(service.timeout, self.http_client.get(&service.url).send()).await
        {
            Ok(Ok(response)) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                if response.status().is_success() {
                    ServiceHealth {
                        name: service.name.clone(),
                        status: HealthStatus::Healthy,
                        latency_ms: Some(latency_ms),
                        last_checked,
                        error: None,
                    }
                } else {
                    ServiceHealth {
                        name: service.name.clone(),
                        status: HealthStatus::Unhealthy,
                        latency_ms: Some(latency_ms),
                        last_checked,
                        error: Some(format!("HTTP {}", response.status())),
                    }
                }
            }
            Ok(Err(e)) => ServiceHealth {
                name: service.name.clone(),
                status: HealthStatus::Unhealthy,
                latency_ms: None,
                last_checked,
                error: Some(e.to_string()),
            },
            Err(_) => ServiceHealth {
                name: service.name.clone(),
                status: HealthStatus::Unhealthy,
                latency_ms: None,
                last_checked,
                error: Some("Timeout".to_string()),
            },
        }
    }

    async fn check_all_dependencies(&self) -> Vec<DependencyHealth> {
        let futures: Vec<_> = self
            .dependencies
            .iter()
            .map(|dep| self.check_dependency(dep))
            .collect();

        futures::future::join_all(futures).await
    }

    async fn check_dependency(&self, dep: &DependencyHealthCheck) -> DependencyHealth {
        let start = Instant::now();
        let last_checked = Utc::now();

        let result = match &dep.check_type {
            DependencyCheckType::PostgreSQL(url) => {
                tokio::time::timeout(dep.timeout, Self::check_postgresql(url)).await
            }
            DependencyCheckType::Redis(url) => {
                tokio::time::timeout(dep.timeout, Self::check_redis(url)).await
            }
            DependencyCheckType::Qdrant(url) => {
                tokio::time::timeout(dep.timeout, self.check_qdrant(url)).await
            }
        };

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(())) => DependencyHealth {
                name: dep.name.clone(),
                status: HealthStatus::Healthy,
                latency_ms: Some(latency_ms),
                last_checked,
                error: None,
            },
            Ok(Err(e)) => DependencyHealth {
                name: dep.name.clone(),
                status: HealthStatus::Unhealthy,
                latency_ms: Some(latency_ms),
                last_checked,
                error: Some(e.to_string()),
            },
            Err(_) => DependencyHealth {
                name: dep.name.clone(),
                status: HealthStatus::Unhealthy,
                latency_ms: None,
                last_checked,
                error: Some("Timeout".to_string()),
            },
        }
    }

    async fn check_postgresql(url: &str) -> Result<(), anyhow::Error> {
        if url.is_empty() {
            return Err(anyhow::anyhow!("PostgreSQL URL not configured"));
        }

        use sqlx::postgres::PgPoolOptions;
        let pool = PgPoolOptions::new().max_connections(1).connect(url).await?;

        sqlx::query("SELECT 1").execute(&pool).await?;

        Ok(())
    }

    async fn check_redis(url: &str) -> Result<(), anyhow::Error> {
        if url.is_empty() {
            return Err(anyhow::anyhow!("Redis URL not configured"));
        }

        let client = redis::Client::open(url)?;
        let mut conn = client.get_multiplexed_async_connection().await?;
        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await?;

        Ok(())
    }

    async fn check_qdrant(&self, url: &str) -> Result<(), anyhow::Error> {
        if url.is_empty() {
            return Err(anyhow::anyhow!("Qdrant URL not configured"));
        }

        let health_url = format!("{}/health", url.trim_end_matches('/'));
        let response = self.http_client.get(&health_url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Qdrant health check failed: {}",
                response.status()
            ))
        }
    }

    fn calculate_overall_status(
        services: &[ServiceHealth],
        dependencies: &[DependencyHealth],
    ) -> HealthStatus {
        let core_services = ["discovery", "auth"];

        let all_core_healthy = core_services.iter().all(|&name| {
            services
                .iter()
                .find(|s| s.name == name)
                .map(|s| s.status == HealthStatus::Healthy)
                .unwrap_or(false)
        });

        let all_deps_healthy = dependencies
            .iter()
            .all(|d| d.status == HealthStatus::Healthy);

        let all_services_healthy = services.iter().all(|s| s.status == HealthStatus::Healthy);

        if all_services_healthy && all_deps_healthy {
            HealthStatus::Healthy
        } else if all_core_healthy && all_deps_healthy {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_overall_status_all_healthy() {
        let services = vec![
            ServiceHealth {
                name: "discovery".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(10),
                last_checked: Utc::now(),
                error: None,
            },
            ServiceHealth {
                name: "auth".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(8),
                last_checked: Utc::now(),
                error: None,
            },
            ServiceHealth {
                name: "sona".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(12),
                last_checked: Utc::now(),
                error: None,
            },
        ];

        let dependencies = vec![
            DependencyHealth {
                name: "postgresql".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(3),
                last_checked: Utc::now(),
                error: None,
            },
            DependencyHealth {
                name: "redis".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(1),
                last_checked: Utc::now(),
                error: None,
            },
        ];

        let status = HealthAggregator::calculate_overall_status(&services, &dependencies);
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_calculate_overall_status_degraded() {
        let services = vec![
            ServiceHealth {
                name: "discovery".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(10),
                last_checked: Utc::now(),
                error: None,
            },
            ServiceHealth {
                name: "auth".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(8),
                last_checked: Utc::now(),
                error: None,
            },
            ServiceHealth {
                name: "sona".to_string(),
                status: HealthStatus::Unhealthy,
                latency_ms: None,
                last_checked: Utc::now(),
                error: Some("Connection refused".to_string()),
            },
        ];

        let dependencies = vec![DependencyHealth {
            name: "postgresql".to_string(),
            status: HealthStatus::Healthy,
            latency_ms: Some(3),
            last_checked: Utc::now(),
            error: None,
        }];

        let status = HealthAggregator::calculate_overall_status(&services, &dependencies);
        assert_eq!(status, HealthStatus::Degraded);
    }

    #[test]
    fn test_calculate_overall_status_unhealthy_core_service() {
        let services = vec![
            ServiceHealth {
                name: "discovery".to_string(),
                status: HealthStatus::Unhealthy,
                latency_ms: None,
                last_checked: Utc::now(),
                error: Some("Timeout".to_string()),
            },
            ServiceHealth {
                name: "auth".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(8),
                last_checked: Utc::now(),
                error: None,
            },
        ];

        let dependencies = vec![DependencyHealth {
            name: "postgresql".to_string(),
            status: HealthStatus::Healthy,
            latency_ms: Some(3),
            last_checked: Utc::now(),
            error: None,
        }];

        let status = HealthAggregator::calculate_overall_status(&services, &dependencies);
        assert_eq!(status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_calculate_overall_status_unhealthy_dependency() {
        let services = vec![
            ServiceHealth {
                name: "discovery".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(10),
                last_checked: Utc::now(),
                error: None,
            },
            ServiceHealth {
                name: "auth".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(8),
                last_checked: Utc::now(),
                error: None,
            },
        ];

        let dependencies = vec![DependencyHealth {
            name: "postgresql".to_string(),
            status: HealthStatus::Unhealthy,
            latency_ms: None,
            last_checked: Utc::now(),
            error: Some("Connection failed".to_string()),
        }];

        let status = HealthAggregator::calculate_overall_status(&services, &dependencies);
        assert_eq!(status, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_health_aggregator_creation() {
        let aggregator = HealthAggregator::new(Duration::from_secs(5));
        assert_eq!(aggregator.services.len(), 6);
        assert_eq!(aggregator.dependencies.len(), 3);
        assert_eq!(aggregator.cache_ttl, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_cached_health_ttl() {
        let aggregator = HealthAggregator::new(Duration::from_millis(100));

        // Store a cached value
        {
            let mut cache = aggregator.cache.write().await;
            *cache = Some(CachedHealth {
                health: AggregatedHealth {
                    status: HealthStatus::Healthy,
                    timestamp: Utc::now(),
                    services: vec![],
                    dependencies: vec![],
                    overall_latency_ms: 0,
                },
                cached_at: Instant::now(),
            });
        }

        // Should use cache
        {
            let cache = aggregator.cache.read().await;
            assert!(cache.is_some());
        }

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Cache should be expired (but still present)
        {
            let cache = aggregator.cache.read().await;
            if let Some(cached) = cache.as_ref() {
                assert!(cached.cached_at.elapsed() > aggregator.cache_ttl);
            }
        }
    }
}
