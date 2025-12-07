use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

use super::types::{AuditAction, AuditEvent, AuditFilter};

pub type Result<T> = std::result::Result<T, AuditError>;

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Invalid filter: {0}")]
    InvalidFilter(String),
}

#[async_trait]
pub trait AuditLogger: Send + Sync {
    async fn log(&self, event: AuditEvent) -> Result<()>;
    async fn log_batch(&self, events: Vec<AuditEvent>) -> Result<()>;
    async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEvent>>;
}

pub struct PostgresAuditLogger {
    pool: PgPool,
    buffer: Arc<Mutex<Vec<AuditEvent>>>,
    buffer_size: usize,
    flush_interval: Duration,
}

impl PostgresAuditLogger {
    pub fn new(pool: PgPool) -> Self {
        Self::with_config(pool, 100, Duration::from_secs(5))
    }

    pub fn with_config(pool: PgPool, buffer_size: usize, flush_interval: Duration) -> Self {
        let logger = Self {
            pool,
            buffer: Arc::new(Mutex::new(Vec::with_capacity(buffer_size))),
            buffer_size,
            flush_interval,
        };

        logger.start_auto_flush();
        logger
    }

    fn start_auto_flush(&self) {
        let buffer = Arc::clone(&self.buffer);
        let pool = self.pool.clone();
        let flush_interval = self.flush_interval;

        tokio::spawn(async move {
            let mut ticker = interval(flush_interval);
            loop {
                ticker.tick().await;
                let events = {
                    let mut buf = buffer.lock().await;
                    if buf.is_empty() {
                        continue;
                    }
                    buf.drain(..).collect::<Vec<_>>()
                };

                if !events.is_empty() {
                    if let Err(e) = Self::insert_batch(&pool, events).await {
                        eprintln!("Failed to flush audit log buffer: {}", e);
                    }
                }
            }
        });
    }

    async fn insert_batch(pool: &PgPool, events: Vec<AuditEvent>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let mut tx = pool.begin().await?;

        for event in events {
            sqlx::query(
                r#"
                INSERT INTO audit_logs
                (id, timestamp, user_id, action, resource_type, resource_id, details, ip_address, user_agent)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(event.id)
            .bind(event.timestamp)
            .bind(event.user_id)
            .bind(event.action.as_str())
            .bind(&event.resource_type)
            .bind(&event.resource_id)
            .bind(&event.details)
            .bind(event.ip_address.as_deref())
            .bind(&event.user_agent)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn flush_buffer(&self) -> Result<()> {
        let events = {
            let mut buffer = self.buffer.lock().await;
            buffer.drain(..).collect::<Vec<_>>()
        };

        Self::insert_batch(&self.pool, events).await
    }
}

#[async_trait]
impl AuditLogger for PostgresAuditLogger {
    async fn log(&self, event: AuditEvent) -> Result<()> {
        let should_flush = {
            let mut buffer = self.buffer.lock().await;
            buffer.push(event);
            buffer.len() >= self.buffer_size
        };

        if should_flush {
            self.flush_buffer().await?;
        }

        Ok(())
    }

    async fn log_batch(&self, events: Vec<AuditEvent>) -> Result<()> {
        Self::insert_batch(&self.pool, events).await
    }

    async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEvent>> {
        let mut query = String::from(
            r#"
            SELECT id, timestamp, user_id, action, resource_type, resource_id,
                   details, host(ip_address) as ip_address, user_agent
            FROM audit_logs
            WHERE 1=1
            "#,
        );

        let mut param_count = 1;

        if filter.start_date.is_some() {
            query.push_str(&format!(" AND timestamp >= ${}", param_count));
            param_count += 1;
        }

        if filter.end_date.is_some() {
            query.push_str(&format!(" AND timestamp <= ${}", param_count));
            param_count += 1;
        }

        if filter.user_id.is_some() {
            query.push_str(&format!(" AND user_id = ${}", param_count));
            param_count += 1;
        }

        if filter.action.is_some() {
            query.push_str(&format!(" AND action = ${}", param_count));
            param_count += 1;
        }

        if filter.resource_type.is_some() {
            query.push_str(&format!(" AND resource_type = ${}", param_count));
            param_count += 1;
        }

        query.push_str(" ORDER BY timestamp DESC");

        if filter.limit.is_some() {
            query.push_str(&format!(" LIMIT ${}", param_count));
            param_count += 1;
        }

        if filter.offset.is_some() {
            query.push_str(&format!(" OFFSET ${}", param_count));
        }

        let mut sql_query = sqlx::query(&query);

        if let Some(start) = filter.start_date {
            sql_query = sql_query.bind(start);
        }
        if let Some(end) = filter.end_date {
            sql_query = sql_query.bind(end);
        }
        if let Some(user_id) = filter.user_id {
            sql_query = sql_query.bind(user_id);
        }
        if let Some(action) = filter.action {
            sql_query = sql_query.bind(action.as_str());
        }
        if let Some(resource_type) = &filter.resource_type {
            sql_query = sql_query.bind(resource_type);
        }
        if let Some(limit) = filter.limit {
            sql_query = sql_query.bind(limit);
        }
        if let Some(offset) = filter.offset {
            sql_query = sql_query.bind(offset);
        }

        let rows = sql_query.fetch_all(&self.pool).await?;

        let events = rows
            .into_iter()
            .filter_map(|row| {
                let action_str: String = row.try_get("action").ok()?;
                let action = AuditAction::from_str(&action_str)?;

                Some(AuditEvent {
                    id: row.try_get("id").ok()?,
                    timestamp: row.try_get("timestamp").ok()?,
                    user_id: row.try_get("user_id").ok()?,
                    action,
                    resource_type: row.try_get("resource_type").ok()?,
                    resource_id: row.try_get("resource_id").ok()?,
                    details: row.try_get("details").ok()?,
                    ip_address: row.try_get("ip_address").ok()?,
                    user_agent: row.try_get("user_agent").ok()?,
                })
            })
            .collect();

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_postgres_audit_logger_new() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
        });

        let pool = PgPool::connect(&database_url).await.unwrap();

        sqlx::query("DROP TABLE IF EXISTS audit_logs CASCADE")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE audit_logs (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                user_id UUID,
                action VARCHAR(50) NOT NULL,
                resource_type VARCHAR(50) NOT NULL,
                resource_id VARCHAR(255),
                details JSONB NOT NULL DEFAULT '{}',
                ip_address INET,
                user_agent TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let logger = PostgresAuditLogger::new(pool.clone());

        assert_eq!(logger.buffer_size, 100);
        assert_eq!(logger.flush_interval, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_audit_logger_log_single_event() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
        });

        let pool = PgPool::connect(&database_url).await.unwrap();

        sqlx::query("DROP TABLE IF EXISTS audit_logs CASCADE")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE audit_logs (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                user_id UUID,
                action VARCHAR(50) NOT NULL,
                resource_type VARCHAR(50) NOT NULL,
                resource_id VARCHAR(255),
                details JSONB NOT NULL DEFAULT '{}',
                ip_address INET,
                user_agent TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let logger = PostgresAuditLogger::with_config(pool.clone(), 1, Duration::from_millis(100));

        let user_id = Uuid::new_v4();
        let event = AuditEvent::new(AuditAction::AuthLogin, "user".to_string())
            .with_user_id(user_id)
            .with_ip_address("192.168.1.1".to_string());

        logger.log(event.clone()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;

        let filter = AuditFilter::new().with_user_id(user_id);
        let results = logger.query(filter).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action, AuditAction::AuthLogin);
        assert_eq!(results[0].user_id, Some(user_id));
    }

    #[tokio::test]
    async fn test_audit_logger_log_batch() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
        });

        let pool = PgPool::connect(&database_url).await.unwrap();

        sqlx::query("DROP TABLE IF EXISTS audit_logs CASCADE")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE audit_logs (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                user_id UUID,
                action VARCHAR(50) NOT NULL,
                resource_type VARCHAR(50) NOT NULL,
                resource_id VARCHAR(255),
                details JSONB NOT NULL DEFAULT '{}',
                ip_address INET,
                user_agent TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let logger = PostgresAuditLogger::new(pool.clone());

        let user_id = Uuid::new_v4();
        let events = vec![
            AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id),
            AuditEvent::new(AuditAction::UserCreated, "user".to_string()).with_user_id(user_id),
            AuditEvent::new(AuditAction::ContentCreated, "content".to_string())
                .with_user_id(user_id),
        ];

        logger.log_batch(events).await.unwrap();

        let filter = AuditFilter::new().with_user_id(user_id);
        let results = logger.query(filter).await.unwrap();

        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_audit_logger_query_with_filters() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
        });

        let pool = PgPool::connect(&database_url).await.unwrap();

        sqlx::query("DROP TABLE IF EXISTS audit_logs CASCADE")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE audit_logs (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                user_id UUID,
                action VARCHAR(50) NOT NULL,
                resource_type VARCHAR(50) NOT NULL,
                resource_id VARCHAR(255),
                details JSONB NOT NULL DEFAULT '{}',
                ip_address INET,
                user_agent TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let logger = PostgresAuditLogger::new(pool.clone());

        let user_id_1 = Uuid::new_v4();
        let user_id_2 = Uuid::new_v4();

        let events = vec![
            AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id_1),
            AuditEvent::new(AuditAction::UserCreated, "user".to_string()).with_user_id(user_id_1),
            AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id_2),
            AuditEvent::new(AuditAction::ContentCreated, "content".to_string())
                .with_user_id(user_id_1),
        ];

        logger.log_batch(events).await.unwrap();

        let filter = AuditFilter::new()
            .with_user_id(user_id_1)
            .with_action(AuditAction::AuthLogin);
        let results = logger.query(filter).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action, AuditAction::AuthLogin);
        assert_eq!(results[0].user_id, Some(user_id_1));
    }

    #[tokio::test]
    async fn test_audit_logger_query_date_range() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
        });

        let pool = PgPool::connect(&database_url).await.unwrap();

        sqlx::query("DROP TABLE IF EXISTS audit_logs CASCADE")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE audit_logs (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                user_id UUID,
                action VARCHAR(50) NOT NULL,
                resource_type VARCHAR(50) NOT NULL,
                resource_id VARCHAR(255),
                details JSONB NOT NULL DEFAULT '{}',
                ip_address INET,
                user_agent TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let logger = PostgresAuditLogger::new(pool.clone());

        let now = Utc::now();
        let user_id = Uuid::new_v4();

        let events = vec![
            AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id),
            AuditEvent::new(AuditAction::UserCreated, "user".to_string()).with_user_id(user_id),
        ];

        logger.log_batch(events).await.unwrap();

        let filter = AuditFilter::new()
            .with_date_range(
                now - chrono::Duration::hours(1),
                now + chrono::Duration::hours(1),
            )
            .with_user_id(user_id);
        let results = logger.query(filter).await.unwrap();

        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_audit_logger_buffer_flush() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
        });

        let pool = PgPool::connect(&database_url).await.unwrap();

        sqlx::query("DROP TABLE IF EXISTS audit_logs CASCADE")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE audit_logs (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                user_id UUID,
                action VARCHAR(50) NOT NULL,
                resource_type VARCHAR(50) NOT NULL,
                resource_id VARCHAR(255),
                details JSONB NOT NULL DEFAULT '{}',
                ip_address INET,
                user_agent TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let logger = PostgresAuditLogger::with_config(pool.clone(), 5, Duration::from_millis(100));

        let user_id = Uuid::new_v4();

        for i in 0..5 {
            let event = AuditEvent::new(AuditAction::AuthLogin, "user".to_string())
                .with_user_id(user_id)
                .with_resource_id(format!("login-{}", i));
            logger.log(event).await.unwrap();
        }

        tokio::time::sleep(Duration::from_millis(50)).await;

        let filter = AuditFilter::new().with_user_id(user_id);
        let results = logger.query(filter).await.unwrap();

        assert_eq!(results.len(), 5);
    }
}
