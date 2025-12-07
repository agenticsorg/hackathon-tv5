use anyhow::{Context, Result};
use redis::aio::ConnectionManager;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use testcontainers::{clients::Cli, Container, GenericImage};
use testcontainers_modules::{postgres::Postgres, redis::Redis};

pub struct TestContainers<'d> {
    pub postgres: Container<'d, Postgres>,
    pub redis: Container<'d, Redis>,
    pub qdrant: Container<'d, GenericImage>,
    pub db_pool: PgPool,
    pub redis_conn: ConnectionManager,
    pub qdrant_url: String,
}

impl<'d> TestContainers<'d> {
    pub async fn new(docker: &'d Cli) -> Result<Self> {
        // Start PostgreSQL container
        let postgres = docker.run(Postgres::default());
        let pg_port = postgres.get_host_port_ipv4(5432);
        let database_url = format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            pg_port
        );

        // Start Redis container
        let redis = docker.run(Redis::default());
        let redis_port = redis.get_host_port_ipv4(6379);
        let redis_url = format!("redis://localhost:{}", redis_port);

        // Start Qdrant container
        let qdrant = docker.run(
            GenericImage::new("qdrant/qdrant", "v1.7.4")
                .with_exposed_port(6333)
                .with_wait_for(testcontainers::core::WaitFor::message_on_stdout(
                    "Qdrant gRPC listening",
                )),
        );
        let qdrant_port = qdrant.get_host_port_ipv4(6333);
        let qdrant_url = format!("http://localhost:{}", qdrant_port);

        // Create database pool
        let db_pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&database_url)
            .await
            .context("Failed to connect to PostgreSQL container")?;

        // Run migrations
        sqlx::migrate!("../migrations")
            .run(&db_pool)
            .await
            .context("Failed to run migrations")?;

        // Create Redis connection
        let redis_client =
            redis::Client::open(redis_url.as_str()).context("Failed to create Redis client")?;
        let redis_conn = ConnectionManager::new(redis_client)
            .await
            .context("Failed to connect to Redis container")?;

        Ok(Self {
            postgres,
            redis,
            qdrant,
            db_pool,
            redis_conn,
            qdrant_url,
        })
    }

    pub async fn cleanup(&self) -> Result<()> {
        // Clean up test data in reverse dependency order
        sqlx::query("TRUNCATE TABLE playback_progress CASCADE")
            .execute(&self.db_pool)
            .await
            .ok();

        sqlx::query("TRUNCATE TABLE playback_sessions CASCADE")
            .execute(&self.db_pool)
            .await
            .ok();

        sqlx::query("TRUNCATE TABLE search_history CASCADE")
            .execute(&self.db_pool)
            .await
            .ok();

        sqlx::query("TRUNCATE TABLE sync_operations CASCADE")
            .execute(&self.db_pool)
            .await
            .ok();

        sqlx::query("TRUNCATE TABLE content CASCADE")
            .execute(&self.db_pool)
            .await
            .ok();

        sqlx::query("TRUNCATE TABLE user_profiles CASCADE")
            .execute(&self.db_pool)
            .await
            .ok();

        sqlx::query("TRUNCATE TABLE users CASCADE")
            .execute(&self.db_pool)
            .await
            .ok();

        // Clear Redis
        redis::cmd("FLUSHDB")
            .query_async::<_, ()>(&mut self.redis_conn.clone())
            .await
            .ok();

        Ok(())
    }
}
