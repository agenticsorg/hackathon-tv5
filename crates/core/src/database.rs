//! Shared PostgreSQL connection pool for Media Gateway services

use sqlx::{postgres::PgPoolOptions, Error as SqlxError, PgPool};
use std::time::Duration;
use tracing::info;

/// Database pool configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub idle_timeout: Duration,
    pub acquire_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost/media_gateway".to_string()),
            max_connections: 20,
            min_connections: 2,
            idle_timeout: Duration::from_secs(600), // 10 minutes
            acquire_timeout: Duration::from_secs(30),
        }
    }
}

/// Shared database connection pool
#[derive(Clone)]
pub struct DatabasePool {
    pool: PgPool,
}

impl DatabasePool {
    /// Create new database pool
    pub async fn new(config: &DatabaseConfig) -> Result<Self, SqlxError> {
        info!(
            "Connecting to database with max {} connections",
            config.max_connections
        );

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .idle_timeout(Some(config.idle_timeout))
            .acquire_timeout(config.acquire_timeout)
            .connect(&config.database_url)
            .await?;

        info!("Database connection pool established");
        Ok(Self { pool })
    }

    /// Create pool from DATABASE_URL environment variable
    pub async fn from_env() -> Result<Self, SqlxError> {
        Self::new(&DatabaseConfig::default()).await
    }

    /// Get reference to underlying pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Check if pool is healthy
    pub async fn is_healthy(&self) -> bool {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await.is_ok()
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 2);
        assert_eq!(config.idle_timeout, Duration::from_secs(600));
    }
}
