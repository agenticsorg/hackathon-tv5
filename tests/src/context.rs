use anyhow::{Context, Result};
use redis::aio::ConnectionManager;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub struct TestContext {
    pub db_pool: PgPool,
    pub redis: ConnectionManager,
    pub auth_url: String,
    pub discovery_url: String,
    pub playback_url: String,
}

impl TestContext {
    pub async fn new() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/media_gateway_test".to_string()
        });

        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let auth_url = std::env::var("AUTH_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8081".to_string());

        let discovery_url = std::env::var("DISCOVERY_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8082".to_string());

        let playback_url = std::env::var("PLAYBACK_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8083".to_string());

        // Create database pool
        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&database_url)
            .await
            .context("Failed to connect to test database")?;

        // Create Redis connection
        let redis_client =
            redis::Client::open(redis_url.as_str()).context("Failed to create Redis client")?;

        let redis = ConnectionManager::new(redis_client)
            .await
            .context("Failed to connect to Redis")?;

        Ok(Self {
            db_pool,
            redis,
            auth_url,
            discovery_url,
            playback_url,
        })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("../migrations")
            .run(&self.db_pool)
            .await
            .context("Failed to run migrations")?;
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<()> {
        // Clean up test data in reverse dependency order
        sqlx::query("TRUNCATE TABLE playback_sessions CASCADE")
            .execute(&self.db_pool)
            .await
            .context("Failed to truncate playback_sessions")?;

        sqlx::query("TRUNCATE TABLE search_history CASCADE")
            .execute(&self.db_pool)
            .await
            .context("Failed to truncate search_history")?;

        sqlx::query("TRUNCATE TABLE content CASCADE")
            .execute(&self.db_pool)
            .await
            .context("Failed to truncate content")?;

        sqlx::query("TRUNCATE TABLE users CASCADE")
            .execute(&self.db_pool)
            .await
            .context("Failed to truncate users")?;

        // Clear Redis
        redis::cmd("FLUSHDB")
            .query_async::<_, ()>(&mut self.redis.clone())
            .await
            .context("Failed to flush Redis")?;

        Ok(())
    }

    pub async fn teardown(self) -> Result<()> {
        self.cleanup().await?;
        self.db_pool.close().await;
        Ok(())
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Note: async cleanup in Drop is not straightforward
        // Prefer explicit teardown() calls
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_creation() {
        let ctx = TestContext::new()
            .await
            .expect("Failed to create test context");
        assert!(ctx.auth_url.starts_with("http"));
        assert!(ctx.discovery_url.starts_with("http"));
        assert!(ctx.playback_url.starts_with("http"));
    }

    #[tokio::test]
    async fn test_migrations() {
        let ctx = TestContext::new()
            .await
            .expect("Failed to create test context");
        ctx.run_migrations()
            .await
            .expect("Failed to run migrations");
        ctx.teardown().await.expect("Failed to teardown");
    }

    #[tokio::test]
    async fn test_cleanup() {
        let ctx = TestContext::new()
            .await
            .expect("Failed to create test context");
        ctx.run_migrations()
            .await
            .expect("Failed to run migrations");
        ctx.cleanup().await.expect("Failed to cleanup");
        ctx.teardown().await.expect("Failed to teardown");
    }
}
