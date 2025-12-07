//! Playback progress tracking and persistence

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Playback progress record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
pub struct ProgressRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub platform_id: String,
    pub progress_seconds: i32,
    pub duration_seconds: i32,
    pub progress_percentage: f32,
    pub last_position_ms: i64,
    pub is_completed: bool,
    pub device_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to update playback progress
#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProgressRequest {
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub platform_id: String,
    pub progress_seconds: i32,
    pub duration_seconds: i32,
    pub device_id: Option<Uuid>,
}

/// Progress repository for database operations
#[derive(Clone)]
pub struct ProgressRepository {
    pool: PgPool,
}

impl ProgressRepository {
    /// Create new repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Upsert playback progress (INSERT or UPDATE on conflict)
    pub async fn upsert_progress(
        &self,
        request: UpdateProgressRequest,
    ) -> Result<ProgressRecord, ProgressError> {
        let record = sqlx::query_as::<_, ProgressRecord>(
            r#"
            INSERT INTO playback_progress (
                user_id, content_id, platform_id, progress_seconds,
                duration_seconds, device_id
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id, content_id, platform_id)
            DO UPDATE SET
                progress_seconds = EXCLUDED.progress_seconds,
                duration_seconds = EXCLUDED.duration_seconds,
                device_id = EXCLUDED.device_id,
                updated_at = NOW()
            RETURNING
                id, user_id, content_id, platform_id, progress_seconds,
                duration_seconds, progress_percentage, last_position_ms,
                is_completed, device_id, created_at, updated_at
            "#,
        )
        .bind(request.user_id)
        .bind(request.content_id)
        .bind(request.platform_id)
        .bind(request.progress_seconds)
        .bind(request.duration_seconds)
        .bind(request.device_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ProgressError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Get progress for specific user and content
    pub async fn get_progress(
        &self,
        user_id: Uuid,
        content_id: Uuid,
        platform_id: &str,
    ) -> Result<Option<ProgressRecord>, ProgressError> {
        let record = sqlx::query_as::<_, ProgressRecord>(
            r#"
            SELECT
                id, user_id, content_id, platform_id, progress_seconds,
                duration_seconds, progress_percentage, last_position_ms,
                is_completed, device_id, created_at, updated_at
            FROM playback_progress
            WHERE user_id = $1 AND content_id = $2 AND platform_id = $3
            "#,
        )
        .bind(user_id)
        .bind(content_id)
        .bind(platform_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ProgressError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Get all incomplete progress for a user (for continue watching)
    pub async fn get_user_incomplete_progress(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ProgressRecord>, ProgressError> {
        let records = sqlx::query_as::<_, ProgressRecord>(
            r#"
            SELECT
                id, user_id, content_id, platform_id, progress_seconds,
                duration_seconds, progress_percentage, last_position_ms,
                is_completed, device_id, created_at, updated_at
            FROM playback_progress
            WHERE user_id = $1 AND is_completed = false
            ORDER BY updated_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ProgressError::Database(e.to_string()))?;

        Ok(records)
    }

    /// Get all progress for a user (including completed)
    pub async fn get_user_all_progress(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ProgressRecord>, ProgressError> {
        let records = sqlx::query_as::<_, ProgressRecord>(
            r#"
            SELECT
                id, user_id, content_id, platform_id, progress_seconds,
                duration_seconds, progress_percentage, last_position_ms,
                is_completed, device_id, created_at, updated_at
            FROM playback_progress
            WHERE user_id = $1
            ORDER BY updated_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ProgressError::Database(e.to_string()))?;

        Ok(records)
    }

    /// Delete stale progress (older than 30 days and completed)
    pub async fn cleanup_stale_progress(&self, days: i32) -> Result<u64, ProgressError> {
        let result = sqlx::query(
            r#"
            DELETE FROM playback_progress
            WHERE updated_at < NOW() - INTERVAL '1 day' * $1
            AND is_completed = true
            "#,
        )
        .bind(days)
        .execute(&self.pool)
        .await
        .map_err(|e| ProgressError::Database(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// Mark content as completed
    pub async fn mark_completed(
        &self,
        user_id: Uuid,
        content_id: Uuid,
        platform_id: &str,
    ) -> Result<(), ProgressError> {
        sqlx::query(
            r#"
            UPDATE playback_progress
            SET is_completed = true, updated_at = NOW()
            WHERE user_id = $1 AND content_id = $2 AND platform_id = $3
            "#,
        )
        .bind(user_id)
        .bind(content_id)
        .bind(platform_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ProgressError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete progress record
    pub async fn delete_progress(
        &self,
        user_id: Uuid,
        content_id: Uuid,
        platform_id: &str,
    ) -> Result<(), ProgressError> {
        sqlx::query(
            r#"
            DELETE FROM playback_progress
            WHERE user_id = $1 AND content_id = $2 AND platform_id = $3
            "#,
        )
        .bind(user_id)
        .bind(content_id)
        .bind(platform_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ProgressError::Database(e.to_string()))?;

        Ok(())
    }
}

/// Progress-related errors
#[derive(Debug, thiserror::Error)]
pub enum ProgressError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Invalid progress data: {0}")]
    InvalidData(String),
}

impl actix_web::ResponseError for ProgressError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
            "error": self.to_string()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/media_gateway_test".to_string()
        });

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    async fn cleanup_test_data(pool: &PgPool, user_id: Uuid) {
        sqlx::query("DELETE FROM playback_progress WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .ok();
    }

    #[tokio::test]
    async fn test_upsert_progress_insert() {
        let pool = setup_test_db().await;
        let repo = ProgressRepository::new(pool.clone());
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let request = UpdateProgressRequest {
            user_id,
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 1200,
            duration_seconds: 6000,
            device_id: Some(Uuid::new_v4()),
        };

        let result = repo.upsert_progress(request.clone()).await;
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record.user_id, user_id);
        assert_eq!(record.content_id, content_id);
        assert_eq!(record.progress_seconds, 1200);
        assert_eq!(record.duration_seconds, 6000);
        assert_eq!(record.progress_percentage, 20.0);
        assert!(!record.is_completed);

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_upsert_progress_update() {
        let pool = setup_test_db().await;
        let repo = ProgressRepository::new(pool.clone());
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let request1 = UpdateProgressRequest {
            user_id,
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 1200,
            duration_seconds: 6000,
            device_id: None,
        };

        repo.upsert_progress(request1).await.unwrap();

        let request2 = UpdateProgressRequest {
            user_id,
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 3000,
            duration_seconds: 6000,
            device_id: None,
        };

        let result = repo.upsert_progress(request2).await;
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record.progress_seconds, 3000);
        assert_eq!(record.progress_percentage, 50.0);

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_completion_threshold() {
        let pool = setup_test_db().await;
        let repo = ProgressRepository::new(pool.clone());
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let request = UpdateProgressRequest {
            user_id,
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 5700,
            duration_seconds: 6000,
            device_id: None,
        };

        let result = repo.upsert_progress(request).await.unwrap();
        assert_eq!(result.progress_percentage, 95.0);
        assert!(result.is_completed);

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_get_progress() {
        let pool = setup_test_db().await;
        let repo = ProgressRepository::new(pool.clone());
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let request = UpdateProgressRequest {
            user_id,
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 1200,
            duration_seconds: 6000,
            device_id: None,
        };

        repo.upsert_progress(request).await.unwrap();

        let result = repo.get_progress(user_id, content_id, "netflix").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        let result = repo.get_progress(user_id, Uuid::new_v4(), "netflix").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_get_user_incomplete_progress() {
        let pool = setup_test_db().await;
        let repo = ProgressRepository::new(pool.clone());
        let user_id = Uuid::new_v4();

        for i in 0..5 {
            let request = UpdateProgressRequest {
                user_id,
                content_id: Uuid::new_v4(),
                platform_id: "netflix".to_string(),
                progress_seconds: 1200 + (i * 100),
                duration_seconds: 6000,
                device_id: None,
            };
            repo.upsert_progress(request).await.unwrap();
        }

        let result = repo.get_user_incomplete_progress(user_id, 10).await;
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 5);
        assert!(records.iter().all(|r| !r.is_completed));

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_cleanup_stale_progress() {
        let pool = setup_test_db().await;
        let repo = ProgressRepository::new(pool.clone());
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let request = UpdateProgressRequest {
            user_id,
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 5700,
            duration_seconds: 6000,
            device_id: None,
        };

        repo.upsert_progress(request).await.unwrap();

        sqlx::query(
            "UPDATE playback_progress SET updated_at = NOW() - INTERVAL '31 days' WHERE user_id = $1"
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

        let result = repo.cleanup_stale_progress(30).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_mark_completed() {
        let pool = setup_test_db().await;
        let repo = ProgressRepository::new(pool.clone());
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let request = UpdateProgressRequest {
            user_id,
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 3000,
            duration_seconds: 6000,
            device_id: None,
        };

        repo.upsert_progress(request).await.unwrap();

        let result = repo.mark_completed(user_id, content_id, "netflix").await;
        assert!(result.is_ok());

        let record = repo
            .get_progress(user_id, content_id, "netflix")
            .await
            .unwrap()
            .unwrap();
        assert!(record.is_completed);

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_delete_progress() {
        let pool = setup_test_db().await;
        let repo = ProgressRepository::new(pool.clone());
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let request = UpdateProgressRequest {
            user_id,
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 1200,
            duration_seconds: 6000,
            device_id: None,
        };

        repo.upsert_progress(request).await.unwrap();

        let result = repo.delete_progress(user_id, content_id, "netflix").await;
        assert!(result.is_ok());

        let record = repo
            .get_progress(user_id, content_id, "netflix")
            .await
            .unwrap();
        assert!(record.is_none());
    }
}
